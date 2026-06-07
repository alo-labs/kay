mod request;
mod response;

use std::sync::Arc;
use std::sync::Mutex;

use bytes::Bytes;
use code_aws_auth::AwsAuthConfig;
use code_aws_auth::AwsAuthContext;
use code_aws_auth::AwsAuthError;
use code_aws_auth::AwsRequestToSign;
use code_otel::otel_event_manager::OtelEventManager;
use http::Method;
use reqwest::StatusCode;
use serde_json::Value;
use serde_json::json;
use tracing::debug;

use crate::ModelProviderInfo;
use crate::client_common::Prompt;
use crate::client_common::ResponseStream;
use crate::debug_logger::DebugLogger;
use crate::error::CodexErr;
use crate::error::Result;
use crate::error::RetryLimitReachedError;
use crate::error::UnexpectedResponseError;
use crate::model_family::find_family_for_model;
use crate::model_family::ModelFamily;
use crate::model_family::provider_model_slug;
use crate::util::backoff;

use request::bedrock_converse_url;
use request::bedrock_region_from_url;
use request::build_bedrock_converse_payload;
use request::header_map_to_json;
use response::response_stream_from_bedrock_response;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct BedrockConverseRequest<'a> {
    pub(crate) prompt: &'a Prompt,
    pub(crate) model_family: &'a ModelFamily,
    pub(crate) model_provider_id: &'a str,
    pub(crate) model: &'a str,
    pub(crate) client: &'a reqwest::Client,
    pub(crate) provider: &'a ModelProviderInfo,
    pub(crate) responses_originator_header: &'a str,
    pub(crate) debug_logger: &'a Arc<Mutex<DebugLogger>>,
    pub(crate) otel_event_manager: Option<OtelEventManager>,
    pub(crate) log_tag: Option<&'a str>,
}

pub(crate) async fn stream_bedrock_converse(
    request: BedrockConverseRequest<'_>,
) -> Result<ResponseStream> {
    let request_model = request
        .prompt
        .model_override
        .as_deref()
        .unwrap_or(request.model);
    let wire_model_slug = provider_model_slug(request.model_provider_id, request_model);
    let request_model = wire_model_slug.as_ref();
    let request_family = request
        .prompt
        .model_family_override
        .clone()
        .or_else(|| find_family_for_model(request_model))
        .unwrap_or_else(|| request.model_family.clone());
    let payload = build_bedrock_converse_payload(
        request.prompt,
        &request_family,
        request_model,
        request.provider,
    )?;
    let payload_body = serde_json::to_string(&payload)?;
    let endpoint = bedrock_converse_url(&request.provider.get_full_url(&None), request_model);
    let region = bedrock_region_from_url(&endpoint);
    let aws_auth = AwsAuthContext::load(AwsAuthConfig {
        profile: None,
        region,
        service: "bedrock".to_string(),
    })
    .await
    .map_err(map_aws_auth_error)?;

    debug!(
        "POST to {}: {}",
        endpoint,
        serde_json::to_string_pretty(&payload).unwrap_or_default()
    );

    let mut attempt = 0;
    let max_retries = request.provider.request_max_retries();
    let mut request_id = String::new();

    loop {
        attempt += 1;
        let url = reqwest::Url::parse(&endpoint)
            .map_err(|err| CodexErr::UnsupportedOperation(format!("invalid Bedrock URL: {err}")))?;
        let mut req_builder = request
            .provider
            .create_request_builder_for_url_with_auth(
                request.client,
                &None,
                reqwest::Method::POST,
                url,
            )
            .await?;
        req_builder = req_builder.headers(crate::default_client::requested_model_headers(
            Some(request.responses_originator_header),
            request_model,
        ));
        req_builder = req_builder
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(payload_body.clone());

        let mut req = req_builder.build()?;
        let signed = aws_auth
            .sign(AwsRequestToSign {
                method: Method::POST,
                url: endpoint.clone(),
                headers: req.headers().clone(),
                body: Bytes::from(payload_body.clone()),
            })
            .await
            .map_err(map_aws_auth_error)?;
        *req.headers_mut() = signed.headers;

        if request_id.is_empty() {
            let header_snapshot = Some(header_map_to_json(req.headers()));
            if let Ok(logger) = request.debug_logger.lock() {
                request_id = logger
                    .start_request_log(&endpoint, &payload, header_snapshot.as_ref(), request.log_tag)
                    .unwrap_or_default();
            }
        }

        let res = if let Some(otel) = request.otel_event_manager.as_ref() {
            otel.log_request(attempt, || request.client.execute(req)).await
        } else {
            request.client.execute(req).await
        };

        match res {
            Ok(resp) if resp.status().is_success() => {
                let response_headers = header_map_to_json(resp.headers());
                let response_id = resp
                    .headers()
                    .get("x-amzn-requestid")
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_string)
                    .unwrap_or_else(|| request_id.clone());
                let body = resp.text().await?;
                let json = serde_json::from_str::<Value>(&body)?;
                if let Ok(logger) = request.debug_logger.lock() {
                    let _ = logger.append_response_event(
                        &request_id,
                        "response",
                        &json!({
                            "status": "success",
                            "headers": response_headers,
                            "body": json,
                        }),
                    );
                    let _ = logger.end_request_log(&request_id);
                }
                return response_stream_from_bedrock_response(
                    response_id,
                    Some(request_model.to_string()),
                    response_headers,
                    json,
                )
                .await;
            }
            Ok(res) => {
                let status = res.status();
                let body = res.text().await.unwrap_or_default();
                if let Ok(logger) = request.debug_logger.lock() {
                    let _ = logger.append_response_event(
                        &request_id,
                        "error",
                        &json!({
                            "status": status.as_u16(),
                            "body": body
                        }),
                    );
                }
                if !(status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()) {
                    return Err(CodexErr::UnexpectedStatus(UnexpectedResponseError {
                        status,
                        body,
                        request_id: None,
                    }));
                }
                if attempt > max_retries {
                    return Err(CodexErr::RetryLimit(RetryLimitReachedError {
                        status,
                        request_id: None,
                        retryable: status.is_server_error()
                            || status == StatusCode::TOO_MANY_REQUESTS,
                    }));
                }
                tokio::time::sleep(backoff(attempt)).await;
            }
            Err(err) => {
                if attempt > max_retries {
                    if let Ok(logger) = request.debug_logger.lock() {
                        let _ = logger.append_response_event(
                            &request_id,
                            "network_error",
                            &json!({ "error": err.to_string() }),
                        );
                        let _ = logger.end_request_log(&request_id);
                    }
                    return Err(err.into());
                }
                tokio::time::sleep(backoff(attempt)).await;
            }
        }
    }
}

fn map_aws_auth_error(error: AwsAuthError) -> CodexErr {
    let retry_hint = if error.is_retryable() {
        " The credential provider reported a retryable failure."
    } else {
        ""
    };
    CodexErr::UnsupportedOperation(format!(
        "Amazon Bedrock AWS authentication failed: {error}.{retry_hint} Configure AWS credentials with AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY, AWS_PROFILE, or the standard AWS SDK credential chain."
    ))
}
