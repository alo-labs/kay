//! OpenCode Go routes some models (e.g. `qwen3.7-max`) through Anthropic Messages
//! (`/v1/messages` + `x-api-key`) instead of OpenAI chat completions (`oa-compat`).

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use bytes::Bytes;
use code_otel::otel_event_manager::OtelEventManager;
use eventsource_stream::Eventsource;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest::StatusCode;
use serde_json::Value;
use serde_json::json;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::debug;

use crate::ModelProviderInfo;
use crate::auth::AuthManager;
use crate::chat_completions::build_chat_completions_payload;
use crate::chat_completions::header_map_to_json;
use crate::client_common::Prompt;
use crate::client_common::ResponseEvent;
use crate::client_common::ResponseStream;
use crate::debug_logger::DebugLogger;
use crate::error::CodexErr;
use crate::error::Result;
use crate::error::RetryLimitReachedError;
use crate::error::UnexpectedResponseError;
use crate::model_family::ModelFamily;
use crate::model_family::uses_opencode_go_anthropic_messages;
use crate::model_provider_info::OPENCODE_GO_PROVIDER_ID;
use crate::util::backoff;
use code_protocol::models::ContentItem;
use code_protocol::models::ResponseItem;

pub(crate) async fn stream_opencode_go_anthropic_messages(
    prompt: &Prompt,
    model_family: &ModelFamily,
    model_slug: &str,
    client: &reqwest::Client,
    provider: &ModelProviderInfo,
    responses_originator_header: &str,
    debug_logger: &Arc<Mutex<DebugLogger>>,
    auth_manager: Option<Arc<AuthManager>>,
    otel_event_manager: Option<OtelEventManager>,
    log_tag: Option<&str>,
) -> Result<ResponseStream> {
    debug_assert!(uses_opencode_go_anthropic_messages(
        OPENCODE_GO_PROVIDER_ID,
        model_slug
    ));

    let chat_payload =
        build_chat_completions_payload(prompt, model_family, model_slug, provider)?;
    let payload = chat_payload_to_anthropic_payload(&chat_payload);
    let endpoint = anthropic_messages_url(provider);

    debug!(
        "POST to {}: {}",
        endpoint,
        serde_json::to_string_pretty(&payload).unwrap_or_default()
    );

    let mut attempt = 0;
    let max_retries = provider.request_max_retries();
    let mut request_id = String::new();

    loop {
        attempt += 1;

        let api_key = resolve_opencode_go_api_key(provider, auth_manager.as_ref())?;
        let mut req_builder = client
            .post(&endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CONTENT_TYPE, "application/json");
        req_builder = req_builder.headers(crate::default_client::requested_model_headers(
            Some(responses_originator_header),
            model_slug,
        ));

        if request_id.is_empty() {
            let header_snapshot = req_builder
                .try_clone()
                .and_then(|builder| builder.build().ok())
                .map(|req| header_map_to_json(req.headers()));
            if let Ok(logger) = debug_logger.lock() {
                request_id = logger
                    .start_request_log(&endpoint, &payload, header_snapshot.as_ref(), log_tag)
                    .unwrap_or_default();
            }
        }

        let res = req_builder.json(&payload).send().await;

        match res {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(logger) = debug_logger.lock() {
                    let _ = logger.append_response_event(
                        &request_id,
                        "stream_initiated",
                        &json!({
                            "status": "success",
                            "status_code": resp.status().as_u16()
                        }),
                    );
                }
                let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent>>(1600);
                let stream = resp.bytes_stream().map_err(CodexErr::Reqwest);
                let debug_logger_clone = Arc::clone(debug_logger);
                let request_id_clone = request_id.clone();
                tokio::spawn(process_anthropic_sse(
                    stream,
                    tx_event,
                    provider.stream_idle_timeout(),
                    debug_logger_clone,
                    request_id_clone,
                    otel_event_manager.clone(),
                ));
                return Ok(ResponseStream { rx_event });
            }
            Ok(res) => {
                let status = res.status();
                let body = res.text().await.unwrap_or_default();
                if let Ok(logger) = debug_logger.lock() {
                    let _ = logger.append_response_event(
                        &request_id,
                        "error",
                        &json!({ "status": status.as_u16(), "body": body }),
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
                        retryable: true,
                    }));
                }
                tokio::time::sleep(backoff(attempt)).await;
            }
            Err(err) => {
                if attempt > max_retries {
                    return Err(CodexErr::Reqwest(err));
                }
                tokio::time::sleep(backoff(attempt)).await;
            }
        }
    }
}

fn anthropic_messages_url(provider: &ModelProviderInfo) -> String {
    let base = provider
        .base_url
        .as_deref()
        .unwrap_or("https://opencode.ai/zen/go/v1");
    format!("{}/messages", base.trim_end_matches('/'))
}

fn resolve_opencode_go_api_key(
    provider: &ModelProviderInfo,
    auth_manager: Option<&Arc<AuthManager>>,
) -> Result<String> {
    if let Some(credential_ref) = provider.credential_ref.as_deref()
        && let Some(manager) = auth_manager
        && let Some(api_key) = manager.provider_api_key(credential_ref)
        && !api_key.trim().is_empty()
    {
        return Ok(api_key);
    }
    if let Some(env_key) = provider.env_key.as_deref()
        && let Ok(api_key) = std::env::var(env_key)
        && !api_key.trim().is_empty()
    {
        return Ok(api_key);
    }
    if let Ok(api_key) = std::env::var("CUSTOM_OPENCODE_GO_API_KEY")
        && !api_key.trim().is_empty()
    {
        return Ok(api_key);
    }
    Err(CodexErr::UnsupportedOperation(
        "OpenCode Go API key missing for Anthropic Messages transport".to_string(),
    ))
}

fn chat_payload_to_anthropic_payload(chat: &Value) -> Value {
    let mut system_parts = Vec::<String>::new();
    let mut messages = Vec::<Value>::new();
    let mut pending_tool_results = Vec::<Value>::new();

    if let Some(chat_messages) = chat.get("messages").and_then(Value::as_array) {
        for message in chat_messages {
            let role = message.get("role").and_then(Value::as_str).unwrap_or("");
            match role {
                "system" => {
                    if let Some(text) = message_content_as_string(message) {
                        system_parts.push(text);
                    }
                }
                "user" => {
                    flush_tool_results(&mut messages, &mut pending_tool_results);
                    if let Some(text) = message_content_as_string(message) {
                        messages.push(json!({"role": "user", "content": text}));
                    }
                }
                "assistant" => {
                    flush_tool_results(&mut messages, &mut pending_tool_results);
                    let mut blocks = Vec::<Value>::new();
                    if let Some(text) = message_content_as_string(message) {
                        blocks.push(json!({"type": "text", "text": text}));
                    }
                    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
                        for tool_call in tool_calls {
                            let id = tool_call
                                .get("id")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            let function = tool_call.get("function").unwrap_or(tool_call);
                            let name = function
                                .get("name")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            let arguments = function
                                .get("arguments")
                                .and_then(Value::as_str)
                                .unwrap_or("{}");
                            let input = serde_json::from_str(arguments).unwrap_or_else(|_| json!({}));
                            blocks.push(json!({
                                "type": "tool_use",
                                "id": id,
                                "name": name,
                                "input": input,
                            }));
                        }
                    }
                    if !blocks.is_empty() {
                        messages.push(json!({"role": "assistant", "content": blocks}));
                    }
                }
                "tool" => {
                    let call_id = message
                        .get("tool_call_id")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    let content = message_content_as_string(message).unwrap_or_default();
                    pending_tool_results.push(json!({
                        "type": "tool_result",
                        "tool_use_id": call_id,
                        "content": content,
                    }));
                }
                _ => {}
            }
        }
    }

    flush_tool_results(&mut messages, &mut pending_tool_results);

    let mut payload = json!({
        "model": chat.get("model").cloned().unwrap_or(Value::Null),
        "max_tokens": 8192,
        "stream": true,
        "messages": messages,
    });
    if !system_parts.is_empty()
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert(
            "system".to_string(),
            json!(system_parts.join("\n\n")),
        );
    }
    if let Some(tools) = chat.get("tools").and_then(Value::as_array)
        && !tools.is_empty()
        && let Some(obj) = payload.as_object_mut()
    {
        let anthropic_tools = tools
            .iter()
            .filter_map(chat_tool_to_anthropic_tool)
            .collect::<Vec<_>>();
        if !anthropic_tools.is_empty() {
            obj.insert("tools".to_string(), json!(anthropic_tools));
        }
    }
    payload
}

fn flush_tool_results(messages: &mut Vec<Value>, pending: &mut Vec<Value>) {
    if pending.is_empty() {
        return;
    }
    messages.push(json!({
        "role": "user",
        "content": std::mem::take(pending),
    }));
}

fn message_content_as_string(message: &Value) -> Option<String> {
    match message.get("content")? {
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => {
            let mut out = String::new();
            for item in items {
                if item.get("type").and_then(Value::as_str) == Some("text")
                    && let Some(text) = item.get("text").and_then(Value::as_str)
                {
                    out.push_str(text);
                }
            }
            if out.is_empty() { None } else { Some(out) }
        }
        _ => None,
    }
}

fn chat_tool_to_anthropic_tool(tool: &Value) -> Option<Value> {
    let function = tool.get("function").unwrap_or(tool);
    let name = function.get("name")?.as_str()?;
    let description = function
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or(name);
    let input_schema = function
        .get("parameters")
        .cloned()
        .unwrap_or_else(|| json!({"type": "object", "properties": {}}));
    Some(json!({
        "name": name,
        "description": description,
        "input_schema": input_schema,
    }))
}

async fn process_anthropic_sse<S>(
    stream: S,
    tx_event: mpsc::Sender<Result<ResponseEvent>>,
    idle_timeout: Duration,
    debug_logger: Arc<Mutex<DebugLogger>>,
    request_id: String,
    otel_event_manager: Option<OtelEventManager>,
) where
    S: Stream<Item = Result<Bytes>> + Unpin,
{
    let _ = otel_event_manager;
    let mut stream = stream.eventsource();
    let mut response_id = String::new();
    let mut created_emitted = false;
    let mut assistant_text = String::new();
    let mut reasoning_text = String::new();
    let mut tool_name = String::new();
    let mut tool_id = String::new();
    let mut tool_input_json = String::new();
    let mut in_tool_use = false;

    loop {
        let next_event = timeout(idle_timeout, stream.next()).await;
        let chunk = match next_event {
            Ok(Some(Ok(chunk))) => chunk,
            Ok(Some(Err(err))) => {
                let _ = tx_event
                    .send(Err(CodexErr::Stream(
                        format!("[transport] {err}"),
                        None,
                        Some(request_id.clone()),
                    )))
                    .await;
                break;
            }
            Ok(None) => break,
            Err(_) => {
                let _ = tx_event
                    .send(Err(CodexErr::Stream(
                        "[idle] timeout waiting for SSE".into(),
                        None,
                        Some(request_id.clone()),
                    )))
                    .await;
                break;
            }
        };

        if chunk.event != "message" && chunk.event.is_empty() {
            continue;
        }

        let data: Value = match serde_json::from_str(&chunk.data) {
            Ok(data) => data,
            Err(_) => continue,
        };

        match data.get("type").and_then(Value::as_str) {
            Some("message_start") => {
                response_id = data
                    .pointer("/message/id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let response_model = data
                    .pointer("/message/model")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                if !created_emitted {
                    created_emitted = true;
                    let _ = tx_event
                        .send(Ok(ResponseEvent::Created {
                            response_id: Some(response_id.clone()),
                            response_model,
                        }))
                        .await;
                }
            }
            Some("content_block_start") => {
                if data
                    .pointer("/content_block/type")
                    .and_then(Value::as_str)
                    == Some("tool_use")
                {
                    in_tool_use = true;
                    tool_name = data
                        .pointer("/content_block/name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    tool_id = data
                        .pointer("/content_block/id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    tool_input_json.clear();
                }
            }
            Some("content_block_delta") => {
                let delta_type = data
                    .pointer("/delta/type")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                match delta_type {
                    "text_delta" => {
                        if let Some(delta) = data.pointer("/delta/text").and_then(Value::as_str) {
                            assistant_text.push_str(delta);
                            let _ = tx_event
                                .send(Ok(ResponseEvent::OutputTextDelta {
                                    delta: delta.to_string(),
                                    item_id: Some(response_id.clone()),
                                    sequence_number: None,
                                    output_index: None,
                                }))
                                .await;
                        }
                    }
                    "thinking_delta" => {
                        if let Some(delta) =
                            data.pointer("/delta/thinking").and_then(Value::as_str)
                        {
                            reasoning_text.push_str(delta);
                            let _ = tx_event
                                .send(Ok(ResponseEvent::ReasoningContentDelta {
                                    delta: delta.to_string(),
                                    item_id: Some(response_id.clone()),
                                    sequence_number: None,
                                    output_index: None,
                                    content_index: None,
                                }))
                                .await;
                        }
                    }
                    "input_json_delta" if in_tool_use => {
                        if let Some(delta) =
                            data.pointer("/delta/partial_json").and_then(Value::as_str)
                        {
                            tool_input_json.push_str(delta);
                        }
                    }
                    _ => {}
                }
            }
            Some("content_block_stop") if in_tool_use => {
                in_tool_use = false;
                let item = ResponseItem::FunctionCall {
                    name: std::mem::take(&mut tool_name),
                    arguments: std::mem::take(&mut tool_input_json),
                    call_id: std::mem::take(&mut tool_id),
                    id: None,
                    namespace: None,
                };
                let _ = tx_event
                    .send(Ok(ResponseEvent::OutputItemDone {
                        item,
                        sequence_number: None,
                        output_index: None,
                    }))
                    .await;
            }
            Some("message_stop") => {
                if !assistant_text.is_empty() {
                    let item = ResponseItem::Message {
                        role: "assistant".to_string(),
                        content: vec![ContentItem::OutputText {
                            text: std::mem::take(&mut assistant_text),
                        }],
                        id: Some(response_id.clone()),
                        end_turn: None,
                        phase: None,
                    };
                    let _ = tx_event
                        .send(Ok(ResponseEvent::OutputItemDone {
                            item,
                            sequence_number: None,
                            output_index: None,
                        }))
                        .await;
                }
                if !reasoning_text.is_empty() {
                    let item = ResponseItem::Reasoning {
                        id: response_id.clone(),
                        summary: Vec::new(),
                        content: Some(vec![code_protocol::models::ReasoningItemContent::ReasoningText {
                            text: std::mem::take(&mut reasoning_text),
                        }]),
                        encrypted_content: None,
                    };
                    let _ = tx_event
                        .send(Ok(ResponseEvent::OutputItemDone {
                            item,
                            sequence_number: None,
                            output_index: None,
                        }))
                        .await;
                }
                let _ = tx_event
                    .send(Ok(ResponseEvent::Completed {
                        response_id: response_id.clone(),
                        token_usage: None,
                    }))
                    .await;
                if let Ok(logger) = debug_logger.lock() {
                    let _ = logger.end_request_log(&request_id);
                }
                break;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_family::find_family_for_model;

    #[test]
    fn qwen37_models_use_anthropic_messages_wire() {
        assert!(uses_opencode_go_anthropic_messages(
            OPENCODE_GO_PROVIDER_ID,
            "qwen3.7-max"
        ));
        assert!(uses_opencode_go_anthropic_messages(
            OPENCODE_GO_PROVIDER_ID,
            "opencode-go/qwen3.7-max"
        ));
        assert!(uses_opencode_go_anthropic_messages(
            OPENCODE_GO_PROVIDER_ID,
            "qwen3.7-plus"
        ));
        assert!(!uses_opencode_go_anthropic_messages(
            OPENCODE_GO_PROVIDER_ID,
            "qwen3.6-plus"
        ));
    }

    #[test]
    fn minimax_models_use_anthropic_messages_wire_on_opencode_go() {
        assert!(uses_opencode_go_anthropic_messages(
            OPENCODE_GO_PROVIDER_ID,
            "minimax-m3"
        ));
        assert!(uses_opencode_go_anthropic_messages(
            OPENCODE_GO_PROVIDER_ID,
            "opencode-go/minimax-m2.7"
        ));
    }

    #[test]
    fn chat_payload_converts_tools_and_system_for_anthropic() {
        let provider = crate::model_provider_info::create_opencode_go_provider();
        let family = find_family_for_model("qwen3.7-max").expect("qwen family");
        let prompt = Prompt {
            input: vec![ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "hello".to_string(),
                }],
                end_turn: None,
                phase: None,
            }],
            base_instructions_override: Some("base".to_string()),
            include_additional_instructions: false,
            ..Default::default()
        };
        let chat =
            build_chat_completions_payload(&prompt, &family, "qwen3.7-max", &provider).unwrap();
        let anthropic = chat_payload_to_anthropic_payload(&chat);
        assert_eq!(anthropic["model"], "qwen3.7-max");
        assert_eq!(anthropic["system"], "base");
        assert!(anthropic["messages"].as_array().is_some());
    }
}
