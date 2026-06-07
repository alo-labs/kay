use std::collections::BTreeMap;
use std::time::Duration;

use bytes::Bytes;
use code_otel::otel_event_manager::OtelEventManager;
use eventsource_stream::Eventsource;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use serde_json::Value;
use serde_json::json;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::debug;
use tracing::trace;

use crate::ModelProviderInfo;
use crate::auth::AuthManager;
use crate::client_common::Prompt;
use crate::client_common::ResponseEvent;
use crate::client_common::ResponseStream;
use crate::client_common::replace_image_payloads_for_model;
use crate::client_common::rewrite_image_generation_calls_for_input;
use crate::debug_logger::DebugLogger;
use crate::error::CodexErr;
use crate::error::Result;
use crate::error::RetryLimitReachedError;
use crate::error::UnexpectedResponseError;
use crate::model_family::ChatCompletionsRoleStrategy;
use crate::model_family::ChatCompletionsReasoningStrategy;
use crate::model_family::ModelFamily;
use crate::model_provider_info::ChatCompletionsFormat;
use crate::openai_tools::create_tools_json_for_chat_completions_api;
use crate::util::backoff;
use code_protocol::models::ContentItem;
use code_protocol::models::ReasoningItemContent;
use code_protocol::models::ResponseItem;
use std::sync::Arc;
use std::sync::Mutex;

/// Implementation for the classic Chat Completions API.
pub(crate) fn build_chat_completions_payload(
    prompt: &Prompt,
    model_family: &ModelFamily,
    model_slug: &str,
    provider: &ModelProviderInfo,
) -> Result<Value> {
    let minimax = matches!(
        provider.chat_completions_format,
        ChatCompletionsFormat::MiniMax
    );
    let collapse_non_chat_roles = minimax
        || matches!(
            model_family.chat_completions_role_strategy,
            ChatCompletionsRoleStrategy::CollapseNonChatRolesToSystem
        );
    let reasoning_field_name = matches!(
        model_family.chat_completions_reasoning_strategy,
        ChatCompletionsReasoningStrategy::PreserveReasoningContent
    )
    .then_some("reasoning_content");
    let mut messages = Vec::<serde_json::Value>::new();
    let mut system_fragments = Vec::<String>::new();

    let full_instructions = prompt.get_full_instructions(model_family);
    if minimax {
        if !full_instructions.trim().is_empty() {
            system_fragments.push(full_instructions.to_string());
        }
    } else {
        messages.push(json!({"role": "system", "content": full_instructions}));
    }

    let mut input = prompt.get_formatted_input();
    rewrite_image_generation_calls_for_input(&mut input);
    replace_image_payloads_for_model(&mut input, model_slug);

    let mut reasoning_by_anchor_index: std::collections::HashMap<usize, String> =
        std::collections::HashMap::new();

    for (idx, item) in input.iter().enumerate() {
        if let ResponseItem::Reasoning {
            content: Some(items),
            ..
        } = item
        {
            let mut text = String::new();
            for c in items {
                match c {
                    ReasoningItemContent::ReasoningText { text: t }
                    | ReasoningItemContent::Text { text: t } => text.push_str(t),
                }
            }
            if text.trim().is_empty() {
                continue;
            }

            let mut attached = false;
            if idx > 0
                && let ResponseItem::Message { role, .. } = &input[idx - 1]
                && role == "assistant"
            {
                reasoning_by_anchor_index
                    .entry(idx - 1)
                    .and_modify(|v| v.push_str(&text))
                    .or_insert(text.clone());
                attached = true;
            }

            if !attached && idx + 1 < input.len() {
                match &input[idx + 1] {
                    ResponseItem::FunctionCall { .. }
                    | ResponseItem::ToolSearchCall { .. }
                    | ResponseItem::LocalShellCall { .. } => {
                        reasoning_by_anchor_index
                            .entry(idx + 1)
                            .and_modify(|v| v.push_str(&text))
                            .or_insert(text.clone());
                    }
                    ResponseItem::Message { role, .. } if role == "assistant" => {
                        reasoning_by_anchor_index
                            .entry(idx + 1)
                            .and_modify(|v| v.push_str(&text))
                            .or_insert(text.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    for (idx, item) in input.iter().enumerate() {
        match item {
            ResponseItem::Message { role, content, .. } => {
                let role = normalize_chat_role(role, collapse_non_chat_roles);
                if minimax && role == "system" {
                    push_system_fragment(&mut system_fragments, content);
                    continue;
                }

                let contains_image = content
                    .iter()
                    .any(|c| matches!(c, ContentItem::InputImage { .. }));
                let reasoning = reasoning_by_anchor_index.get(&idx).map(String::as_str);

                if contains_image && !minimax {
                    let mut parts = Vec::<serde_json::Value>::new();
                    for c in content {
                        match c {
                            ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                                parts.push(json!({ "type": "text", "text": text }));
                            }
                            ContentItem::InputImage { image_url } => {
                                parts.push(json!({
                                    "type": "image_url",
                                    "image_url": { "url": image_url }
                                }));
                            }
                        }
                    }
                    let mut message = json!({"role": role, "content": parts});
                    if let Some(field_name) = reasoning_field_name
                        && let Some(obj) = message.as_object_mut()
                    {
                        obj.insert(
                            field_name.to_string(),
                            json!(reasoning.unwrap_or_default()),
                        );
                    }
                    messages.push(message);
                } else {
                    let mut message = json!({"role": role, "content": content_text(content)});
                    if let Some(field_name) = reasoning_field_name
                        && let Some(obj) = message.as_object_mut()
                    {
                        obj.insert(
                            field_name.to_string(),
                            json!(reasoning.unwrap_or_default()),
                        );
                    }
                    messages.push(message);
                }
            }
            ResponseItem::CompactionSummary { .. } | ResponseItem::ContextCompaction { .. } => {
                // Compaction summaries are only meaningful to the Responses API; omit them
                // when translating to Chat Completions.
                continue;
            }
            ResponseItem::FunctionCall {
                name,
                arguments,
                call_id,
                ..
            } => {
                let reasoning = reasoning_by_anchor_index.get(&idx).map(String::as_str);
                let arguments = normalize_tool_arguments_for_chat_history(arguments);
                let tool_call = json!({
                    "id": call_id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": arguments,
                    }
                });
                push_tool_call_message(
                    &mut messages,
                    tool_call,
                    reasoning,
                    reasoning_field_name,
                );
            }
            ResponseItem::ToolSearchCall {
                call_id,
                status,
                execution,
                arguments,
                ..
            } => {
                let reasoning = reasoning_by_anchor_index.get(&idx).map(String::as_str);
                let tool_call = json!({
                    "id": call_id.clone().unwrap_or_default(),
                    "type": "tool_search_call",
                    "call_id": call_id,
                    "status": status,
                    "execution": execution,
                    "arguments": arguments,
                });
                push_tool_call_message(
                    &mut messages,
                    tool_call,
                    reasoning,
                    reasoning_field_name,
                );
            }
            ResponseItem::LocalShellCall {
                id,
                call_id: _,
                status,
                action,
            } => {
                let reasoning = reasoning_by_anchor_index.get(&idx).map(String::as_str);
                let tool_call = json!({
                    "id": id.clone().unwrap_or_default(),
                    "type": "local_shell_call",
                    "status": status,
                    "action": action,
                });
                push_tool_call_message(
                    &mut messages,
                    tool_call,
                    reasoning,
                    reasoning_field_name,
                );
            }
            ResponseItem::FunctionCallOutput { call_id, output } => {
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": output.to_string(),
                }));
            }
            ResponseItem::ToolSearchOutput {
                call_id,
                status,
                execution,
                tools,
            } => {
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call_id.clone().unwrap_or_default(),
                    "content": serde_json::json!({
                        "status": status,
                        "execution": execution,
                        "tools": tools,
                    })
                    .to_string(),
                }));
            }
            ResponseItem::CustomToolCall {
                id,
                call_id: _,
                name,
                input,
                status: _,
            } => {
                let reasoning = reasoning_by_anchor_index.get(&idx).map(String::as_str);
                let tool_call = json!({
                    "id": id,
                    "type": "custom",
                    "custom": {
                        "name": name,
                        "input": input,
                    }
                });
                push_tool_call_message(
                    &mut messages,
                    tool_call,
                    reasoning,
                    reasoning_field_name,
                );
            }
            ResponseItem::CustomToolCallOutput {
                call_id, output, ..
            } => {
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": output,
                }));
            }
            ResponseItem::Reasoning { .. }
            | ResponseItem::WebSearchCall { .. }
            | ResponseItem::ImageGenerationCall { .. }
            | ResponseItem::GhostSnapshot { .. }
            | ResponseItem::Other => {
                continue;
            }
        }
    }

    if minimax {
        let system_content = system_fragments
            .into_iter()
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        if !system_content.is_empty() {
            messages.insert(0, json!({"role": "system", "content": system_content}));
        }
    }

    let tools_json = create_tools_json_for_chat_completions_api(&prompt.tools)?;
    let has_tools = !tools_json.is_empty();
    let supports_native_output_schema = !minimax
        && model_family.supports_chat_completions_response_format_json_schema;
    if let Some(schema) = prompt.output_schema.as_ref()
        && (has_tools || !supports_native_output_schema)
    {
        insert_final_output_schema_instructions(&mut messages, schema);
    }
    let mut payload = json!({
        "model": model_slug,
        "messages": messages,
        "stream": true,
    });
    if minimax {
        payload["reasoning_split"] = json!(true);
        if !tools_json.is_empty()
            && let Some(obj) = payload.as_object_mut()
        {
            obj.insert("tools".to_string(), serde_json::Value::Array(tools_json));
        }
    } else {
        payload["tools"] = serde_json::Value::Array(tools_json);
    }
    if let Some(schema) = prompt.output_schema.as_ref()
        && !has_tools
        && supports_native_output_schema
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert(
            "response_format".to_string(),
            json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "code_output_schema",
                    "strict": true,
                    "schema": schema,
                }
            }),
        );
    }

    if let Some(openrouter_cfg) = provider.openrouter_config()
        && let Some(obj) = payload.as_object_mut()
    {
        let mut provider_payload = openrouter_cfg
            .provider
            .as_ref()
            .map(serde_json::to_value)
            .transpose()?
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();
        for (key, value) in &openrouter_cfg.extra {
            if is_openrouter_provider_config_key(key) {
                provider_payload
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }
        if !provider_payload.is_empty() {
            obj.insert("provider".to_string(), Value::Object(provider_payload));
        }
        if let Some(route) = &openrouter_cfg.route {
            obj.insert("route".to_string(), route.clone());
        }
        for (key, value) in &openrouter_cfg.extra {
            if is_openrouter_provider_config_key(key) {
                continue;
            }
            obj.entry(key.clone()).or_insert(value.clone());
        }
    }

    if let Ok(val) = std::env::var("CODEX_OLLAMA_NUM_CTX")
        && let Ok(n) = val.parse::<u64>()
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert("num_ctx".to_string(), json!(n));
        let mut options = serde_json::Map::new();
        options.insert("num_ctx".to_string(), json!(n));
        obj.entry("options").or_insert(json!(options));
    }

    Ok(payload)
}

fn is_openrouter_provider_config_key(key: &str) -> bool {
    matches!(
        key,
        "order"
            | "allow_fallbacks"
            | "require_parameters"
            | "data_collection"
            | "zdr"
            | "only"
            | "ignore"
            | "quantizations"
            | "sort"
            | "max_price"
    )
}

fn normalize_tool_arguments_for_chat_history(arguments: &str) -> String {
    if serde_json::from_str::<serde_json::Value>(arguments).is_ok() {
        return arguments.to_string();
    }

    serde_json::json!({ "_raw": arguments }).to_string()
}

fn normalize_chat_role(role: &str, collapse_non_chat_roles: bool) -> &str {
    if !collapse_non_chat_roles {
        return role;
    }

    match role {
        "user" | "assistant" | "tool" | "system" => role,
        _ => "system",
    }
}

fn push_system_fragment(system_fragments: &mut Vec<String>, content: &[ContentItem]) {
    let text = content_text(content);
    if !text.trim().is_empty() {
        system_fragments.push(text);
    }
}

fn content_text(content: &[ContentItem]) -> String {
    let mut text = String::new();
    for c in content {
        match c {
            ContentItem::InputText { text: t } | ContentItem::OutputText { text: t } => {
                text.push_str(t);
            }
            ContentItem::InputImage { .. } => {}
        }
    }
    text
}

fn insert_final_output_schema_instructions(messages: &mut Vec<Value>, schema: &Value) {
    let schema_json = serde_json::to_string(schema).unwrap_or_else(|_| schema.to_string());
    let content = format!(
        "Final output contract:\n\
         This schema applies only to the final assistant message after required tool work is complete. \
         Do not answer early just to satisfy the schema. When finishing, return a single JSON object \
         that matches this schema and no markdown fences or prose.\n\
         Schema: {schema_json}"
    );
    let insert_at = messages
        .iter()
        .position(|message| message.get("role").and_then(Value::as_str) != Some("system"))
        .unwrap_or(messages.len());
    messages.insert(insert_at, json!({"role": "system", "content": content}));
}

pub(crate) async fn stream_chat_completions(
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
    let payload = build_chat_completions_payload(prompt, model_family, model_slug, provider)?;

    let endpoint = provider.get_full_url(&None);
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

        let base_auth = auth_manager.as_ref().and_then(|m| m.auth());
        let provider_api_key = provider
            .credential_ref
            .as_deref()
            .and_then(|credential_ref| {
                auth_manager
                    .as_ref()
                    .and_then(|manager| manager.provider_api_key(credential_ref))
            });
        let auth = provider
            .effective_auth_with_provider_key(&base_auth, provider_api_key.as_deref())
            .await?;
        let mut req_builder = provider
            .create_request_builder_with_auth(client, &auth)
            .await?;
        req_builder = req_builder.headers(crate::default_client::requested_model_headers(
            Some(responses_originator_header),
            model_slug,
        ));

        if let Some(auth) = auth.as_ref() {
            if auth.mode.is_chatgpt() {
                if let Some(account_id) = auth.get_account_id() {
                    req_builder = req_builder.header("chatgpt-account-id", account_id);
                }
            }
        }

        req_builder = req_builder
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .json(&payload);

        if request_id.is_empty() {
            let endpoint_for_log = provider.get_full_url(&auth);
            let header_snapshot = req_builder
                .try_clone()
                .and_then(|builder| builder.build().ok())
                .map(|req| header_map_to_json(req.headers()));

            if let Ok(logger) = debug_logger.lock() {
                request_id = logger
                    .start_request_log(
                        &endpoint_for_log,
                        &payload,
                        header_snapshot.as_ref(),
                        log_tag,
                    )
                    .unwrap_or_default();
            }
        }

        let res = req_builder.send().await;

        match res {
            Ok(resp) if resp.status().is_success() => {
                // Log successful response initiation
                if let Ok(logger) = debug_logger.lock() {
                    let _ = logger.append_response_event(
                        &request_id,
                        "stream_initiated",
                        &serde_json::json!({
                            "status": "success",
                            "status_code": resp.status().as_u16()
                        }),
                    );
                }
                let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent>>(1600);
                let stream = resp.bytes_stream().map_err(CodexErr::Reqwest);
                let debug_logger_clone = Arc::clone(&debug_logger);
                let request_id_clone = request_id.clone();
                tokio::spawn(process_chat_sse(
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
                if status == StatusCode::UNAUTHORIZED && provider.has_command_auth() {
                    provider.invalidate_cached_auth_token();
                    if attempt > max_retries {
                        return Err(CodexErr::RetryLimit(RetryLimitReachedError {
                            status,
                            request_id: None,
                            retryable: true,
                        }));
                    }
                    let delay = backoff(attempt);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                if !(status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()) {
                    let body = (res.text().await).unwrap_or_default();
                    if let Ok(logger) = debug_logger.lock() {
                        let _ = logger.append_response_event(
                            &request_id,
                            "error",
                            &serde_json::json!({
                                "status": status.as_u16(),
                                "body": body
                            }),
                        );
                        let _ = logger.end_request_log(&request_id);
                    }
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

                let retry_after_secs = res
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());

                let delay = retry_after_secs
                    .map(|s| Duration::from_millis(s * 1_000))
                    .unwrap_or_else(|| backoff(attempt));
                tokio::time::sleep(delay).await;
            }
            Err(e) => {
                let is_connectivity = e.is_connect() || e.is_timeout() || e.is_request();
                if attempt > max_retries {
                    if let Ok(logger) = debug_logger.lock() {
                        let _ = logger.append_response_event(
                            &request_id,
                            "network_error",
                            &serde_json::json!({
                                "error": e.to_string()
                            }),
                        );
                        let _ = logger.end_request_log(&request_id);
                    }
                    if is_connectivity {
                        let req_id = (!request_id.is_empty()).then(|| request_id.clone());
                        return Err(CodexErr::Stream(
                            format!("[transport] network unavailable: {e}"),
                            None,
                            req_id,
                        ));
                    }
                    return Err(e.into());
                }
                let delay = backoff(attempt);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

fn push_tool_call_message(
    messages: &mut Vec<Value>,
    tool_call: Value,
    reasoning: Option<&str>,
    reasoning_field_name: Option<&str>,
) {
    // Chat Completions requires that tool calls are grouped into a single assistant message
    // (with `tool_calls: [...]`) followed by tool role responses.
    if let Some(Value::Object(obj)) = messages.last_mut()
        && obj.get("role").and_then(Value::as_str) == Some("assistant")
        && obj.get("content").is_some_and(Value::is_null)
        && let Some(tool_calls) = obj.get_mut("tool_calls").and_then(Value::as_array_mut)
    {
        tool_calls.push(tool_call);
        if let Some(field_name) = reasoning_field_name {
            let reasoning_text = reasoning.unwrap_or_default();
            if let Some(Value::String(existing)) = obj.get_mut(field_name) {
                if !reasoning_text.is_empty() {
                    if !existing.is_empty() {
                        existing.push('\n');
                    }
                    existing.push_str(reasoning_text);
                }
            } else {
                obj.insert(
                    field_name.to_string(),
                    Value::String(reasoning_text.to_string()),
                );
            }
        }
        return;
    }

    let mut msg = json!({
        "role": "assistant",
        "content": null,
        "tool_calls": [tool_call],
    });
    if let Some(field_name) = reasoning_field_name
        && let Some(obj) = msg.as_object_mut()
    {
        obj.insert(field_name.to_string(), json!(reasoning.unwrap_or_default()));
    }
    messages.push(msg);
}

/// Lightweight SSE processor for the Chat Completions streaming format. The
/// output is mapped onto Codex's internal [`ResponseEvent`] so that the rest
/// of the pipeline can stay agnostic of the underlying wire format.
async fn process_chat_sse<S>(
    stream: S,
    tx_event: mpsc::Sender<Result<ResponseEvent>>,
    idle_timeout: Duration,
    debug_logger: Arc<Mutex<DebugLogger>>,
    request_id: String,
    otel_event_manager: Option<OtelEventManager>,
) where
    S: Stream<Item = Result<Bytes>> + Unpin,
{
    let mut stream = stream.eventsource();

    // State to accumulate a function call across streaming chunks.
    // OpenAI may split the `arguments` string over multiple `delta` events
    // until the chunk whose `finish_reason` is `tool_calls` is emitted. We
    // keep collecting the pieces here and forward a single
    // `ResponseItem::FunctionCall` once the call is complete.
    #[derive(Default)]
    struct FunctionCallState {
        name: Option<String>,
        arguments: String,
        call_id: Option<String>,
        active: bool,
    }

    let mut fn_call_state = FunctionCallState::default();
    let mut assistant_text = String::new();
    let mut reasoning_text = String::new();
    let mut current_item_id: Option<String> = None;
    let mut current_response_id: Option<String> = None;
    let mut current_response_model: Option<String> = None;
    let mut created_emitted = false;

    async fn flush_and_complete(
        tx_event: &mpsc::Sender<Result<ResponseEvent>>,
        assistant_text: &mut String,
        reasoning_text: &mut String,
        current_item_id: &Option<String>,
        response_id: Option<&str>,
        debug_logger: &Arc<Mutex<DebugLogger>>,
        request_id: &str,
    ) {
        // Emit any finalized items before closing so downstream consumers receive
        // terminal events for both assistant content and raw reasoning.
        if !assistant_text.is_empty() {
            let item = ResponseItem::Message {
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: std::mem::take(assistant_text),
                }],
                id: current_item_id.clone(),
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
                id: current_item_id.clone().unwrap_or_else(String::new),
                summary: Vec::new(),
                content: Some(vec![ReasoningItemContent::ReasoningText {
                    text: std::mem::take(reasoning_text),
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
                response_id: response_id.unwrap_or_default().to_string(),
                token_usage: None,
            }))
            .await;
        if let Ok(logger) = debug_logger.lock() {
            let _ = logger.end_request_log(request_id);
        }
    }

    loop {
        let next_event = if let Some(manager) = otel_event_manager.as_ref() {
            manager
                .log_sse_event(|| timeout(idle_timeout, stream.next()))
                .await
        } else {
            timeout(idle_timeout, stream.next()).await
        };

        let sse = match next_event {
            Ok(Some(Ok(ev))) => ev,
            Ok(Some(Err(e))) => {
                let _ = tx_event
                    .send(Err(CodexErr::Stream(
                        format!("[transport] {e}"),
                        None,
                        Some(request_id.clone()),
                    )))
                    .await;
                return;
            }
            Ok(None) => {
                // Stream closed by server without an explicit end marker – log for diagnostics
                tracing::debug!("chat SSE stream closed without [DONE] marker");
                if let Ok(logger) = debug_logger.lock() {
                    let _ = logger.append_response_event(
                        &request_id,
                        "stream_closed_without_done",
                        &serde_json::json!({
                            "assistant_len": assistant_text.len(),
                            "reasoning_len": reasoning_text.len(),
                        }),
                    );
                }
                flush_and_complete(
                    &tx_event,
                    &mut assistant_text,
                    &mut reasoning_text,
                    &current_item_id,
                    current_response_id.as_deref(),
                    &debug_logger,
                    &request_id,
                )
                .await;
                return;
            }
            Err(_) => {
                let _ = tx_event
                    .send(Err(CodexErr::Stream(
                        "[idle] timeout waiting for SSE".into(),
                        None,
                        Some(request_id.clone()),
                    )))
                    .await;
                return;
            }
        };

        let data = sse.data.trim();

        if data.is_empty() {
            continue;
        }

        // OpenAI Chat streaming sends a literal string "[DONE]" when finished.
        if data == "[DONE]" || data == "DONE" {
            flush_and_complete(
                &tx_event,
                &mut assistant_text,
                &mut reasoning_text,
                &current_item_id,
                current_response_id.as_deref(),
                &debug_logger,
                &request_id,
            )
            .await;
            return;
        }

        // Parse JSON chunk
        let chunk: serde_json::Value = match serde_json::from_str(data) {
            Ok(v) => v,
            Err(e) => {
                // Surface parse errors to logs and debug logger for diagnostics, then skip
                let mut excerpt = sse.data.clone();
                const MAX: usize = 600;
                if excerpt.len() > MAX {
                    excerpt.truncate(MAX);
                }
                tracing::debug!("chat SSE parse error: {} | data: {}", e, excerpt);
                if let Ok(logger) = debug_logger.lock() {
                    let _ = logger.append_response_event(
                        &request_id,
                        "sse_parse_error",
                        &serde_json::json!({
                            "error": e.to_string(),
                            "data_excerpt": excerpt,
                        }),
                    );
                }
                continue;
            }
        };
        trace!("chat_completions received SSE chunk: {chunk:?}");

        // Log the SSE chunk to debug log
        if let Ok(logger) = debug_logger.lock() {
            let _ = logger.append_response_event(&request_id, "sse_event", &chunk);
        }

        if current_response_id.is_none() {
            current_response_id = chunk
                .get("id")
                .and_then(|id| id.as_str())
                .map(ToString::to_string);
        }
        if current_response_model.is_none() {
            current_response_model = chunk
                .get("model")
                .and_then(|model| model.as_str())
                .map(ToString::to_string);
        }
        if !created_emitted && (current_response_id.is_some() || current_response_model.is_some()) {
            let _ = tx_event
                .send(Ok(ResponseEvent::Created {
                    response_id: current_response_id.clone(),
                    response_model: current_response_model.clone(),
                }))
                .await;
            created_emitted = true;
        }

        // Extract item_id if present at the top level or in choice
        if let Some(item_id) = chunk.get("item_id").and_then(|id| id.as_str()) {
            current_item_id = Some(item_id.to_string());
        }

        let choice_opt = chunk.get("choices").and_then(|c| c.get(0));

        if let Some(choice) = choice_opt {
            // Check for item_id in the choice as well
            if let Some(item_id) = choice.get("item_id").and_then(|id| id.as_str()) {
                current_item_id = Some(item_id.to_string());
            }

            // Handle assistant content tokens as streaming deltas.
            if let Some(content) = choice
                .get("delta")
                .and_then(|d| d.get("content"))
                .and_then(|c| c.as_str())
            {
                if !content.is_empty() {
                    assistant_text.push_str(content);
                    let _ = tx_event
                        .send(Ok(ResponseEvent::OutputTextDelta {
                            delta: content.to_string(),
                            item_id: current_item_id.clone(),
                            sequence_number: None,
                            output_index: None,
                        }))
                        .await;
                }
            }

            // Forward any reasoning/thinking deltas if present.
            // Some providers stream `reasoning` as a plain string while others
            // nest the text under an object (e.g. `{ "reasoning": { "text": "…" } }`).
            if let Some(reasoning_val) = choice
                .get("delta")
                .and_then(|d| d.get("reasoning").or_else(|| d.get("reasoning_content")))
            {
                let mut maybe_text = reasoning_val
                    .as_str()
                    .map(str::to_string)
                    .filter(|s| !s.is_empty());

                if maybe_text.is_none() && reasoning_val.is_object() {
                    if let Some(s) = reasoning_val
                        .get("text")
                        .and_then(|t| t.as_str())
                        .filter(|s| !s.is_empty())
                    {
                        maybe_text = Some(s.to_string());
                    } else if let Some(s) = reasoning_val
                        .get("content")
                        .and_then(|t| t.as_str())
                        .filter(|s| !s.is_empty())
                    {
                        maybe_text = Some(s.to_string());
                    }
                }

                if let Some(reasoning) = maybe_text {
                    // Accumulate so we can emit a terminal Reasoning item at the end.
                    reasoning_text.push_str(&reasoning);
                    let _ = tx_event
                        .send(Ok(ResponseEvent::ReasoningContentDelta {
                            delta: reasoning,
                            item_id: current_item_id.clone(),
                            sequence_number: None,
                            output_index: None,
                            content_index: None,
                        }))
                        .await;
                }
            }

            // Some providers only include reasoning on the final message object.
            if let Some(message_reasoning) = choice
                .get("message")
                .and_then(|m| m.get("reasoning").or_else(|| m.get("reasoning_content")))
            {
                // Accept either a plain string or an object with { text | content }
                if let Some(s) = message_reasoning.as_str() {
                    if !s.is_empty() {
                        reasoning_text.push_str(s);
                        let _ = tx_event
                            .send(Ok(ResponseEvent::ReasoningContentDelta {
                                delta: s.to_string(),
                                item_id: current_item_id.clone(),
                                sequence_number: None,
                                output_index: None,
                                content_index: None,
                            }))
                            .await;
                    }
                } else if let Some(obj) = message_reasoning.as_object() {
                    if let Some(s) = obj
                        .get("text")
                        .and_then(|v| v.as_str())
                        .or_else(|| obj.get("content").and_then(|v| v.as_str()))
                    {
                        if !s.is_empty() {
                            reasoning_text.push_str(s);
                            let _ = tx_event
                                .send(Ok(ResponseEvent::ReasoningContentDelta {
                                    delta: s.to_string(),
                                    item_id: current_item_id.clone(),
                                    sequence_number: None,
                                    output_index: None,
                                    content_index: None,
                                }))
                                .await;
                        }
                    }
                }
            }

            // Handle streaming function / tool calls.
            if let Some(tool_calls) = choice
                .get("delta")
                .and_then(|d| d.get("tool_calls"))
                .and_then(|tc| tc.as_array())
            {
                if let Some(tool_call) = tool_calls.first() {
                    // Mark that we have an active function call in progress.
                    fn_call_state.active = true;

                    // Extract call_id if present.
                    if let Some(id) = tool_call.get("id").and_then(|v| v.as_str()) {
                        fn_call_state.call_id.get_or_insert_with(|| id.to_string());
                    }

                    // Extract function details if present.
                    if let Some(function) = tool_call.get("function") {
                        if let Some(name) = function.get("name").and_then(|n| n.as_str()) {
                            fn_call_state.name.get_or_insert_with(|| name.to_string());
                        }

                        if let Some(args_fragment) =
                            function.get("arguments").and_then(|a| a.as_str())
                        {
                            fn_call_state.arguments.push_str(args_fragment);
                        }
                    }
                }
            }

            // Emit end-of-turn when finish_reason signals completion.
            if let Some(finish_reason) = choice.get("finish_reason").and_then(|v| v.as_str()) {
                match finish_reason {
                    "tool_calls" if fn_call_state.active => {
                        // First, flush the terminal raw reasoning so UIs can finalize
                        // the reasoning stream before any exec/tool events begin.
                        if !reasoning_text.is_empty() {
                            let item = ResponseItem::Reasoning {
                                id: current_item_id.clone().unwrap_or_else(String::new),
                                summary: Vec::new(),
                                content: Some(vec![ReasoningItemContent::ReasoningText {
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

                        // Then emit the FunctionCall response item.
                        let item = ResponseItem::FunctionCall {
                            id: current_item_id.clone(),
                            name: fn_call_state.name.clone().unwrap_or_else(|| "".to_string()),
                            namespace: None,
                            arguments: fn_call_state.arguments.clone(),
                            call_id: fn_call_state.call_id.clone().unwrap_or_else(String::new),
                        };

                        let _ = tx_event
                            .send(Ok(ResponseEvent::OutputItemDone {
                                item,
                                sequence_number: None,
                                output_index: None,
                            }))
                            .await;
                    }
                    "stop" => {
                        // Regular turn without tool-call. Emit the final assistant message
                        // as a single OutputItemDone so non-delta consumers see the result.
                        if !assistant_text.is_empty() {
                            let item = ResponseItem::Message {
                                role: "assistant".to_string(),
                                content: vec![ContentItem::OutputText {
                                    text: std::mem::take(&mut assistant_text),
                                }],
                                id: current_item_id.clone(),
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
                        // Also emit a terminal Reasoning item so UIs can finalize raw reasoning.
                        if !reasoning_text.is_empty() {
                            let item = ResponseItem::Reasoning {
                                id: current_item_id.clone().unwrap_or_else(String::new),
                                summary: Vec::new(),
                                content: Some(vec![ReasoningItemContent::ReasoningText {
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
                    }
                    _ => {}
                }

                // Emit Completed regardless of reason so the agent can advance.
                let _ = tx_event
                    .send(Ok(ResponseEvent::Completed {
                        response_id: String::new(),
                        token_usage: None,
                    }))
                    .await;

                // Prepare for potential next turn (should not happen in same stream).
                // fn_call_state = FunctionCallState::default();

                // Mark the request log as complete
                if let Ok(logger) = debug_logger.lock() {
                    let _ = logger.end_request_log(&request_id);
                }

                return; // End processing for this SSE stream.
            }
        }
    }
}

/// Optional client-side aggregation helper
///
/// Stream adapter that merges the incremental `OutputItemDone` chunks coming from
/// [`process_chat_sse`] into a *running* assistant message, **suppressing the
/// per-token deltas**.  The stream stays silent while the model is thinking
/// and only emits two events per turn:
///
///   1. `ResponseEvent::OutputItemDone` with the *complete* assistant message
///      (fully concatenated).
///   2. The original `ResponseEvent::Completed` right after it.
///
/// This mirrors the behaviour the TypeScript CLI exposes to its higher layers.
///
/// The adapter is intentionally *lossless*: callers who do **not** opt in via
/// [`AggregateStreamExt::aggregate()`] keep receiving the original unmodified
/// events.
#[derive(Copy, Clone, Eq, PartialEq)]
enum AggregateMode {
    AggregatedOnly,
    Streaming,
}
pub(crate) struct AggregatedChatStream<S> {
    inner: S,
    cumulative: String,
    cumulative_reasoning: String,
    cumulative_item_id: Option<String>,
    pending: std::collections::VecDeque<ResponseEvent>,
    mode: AggregateMode,
}

impl<S> Stream for AggregatedChatStream<S>
where
    S: Stream<Item = Result<ResponseEvent>> + Unpin,
{
    type Item = Result<ResponseEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // First, flush any buffered events from the previous call.
        if let Some(ev) = this.pending.pop_front() {
            return Poll::Ready(Some(Ok(ev)));
        }

        loop {
            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(Some(Ok(ResponseEvent::OutputItemDone {
                    item,
                    sequence_number: _,
                    ..
                }))) => {
                    // If this is an incremental assistant message chunk, accumulate but
                    // do NOT emit yet. Forward any other item (e.g. FunctionCall) right
                    // away so downstream consumers see it.

                    let is_assistant_delta = matches!(&item, code_protocol::models::ResponseItem::Message { role, .. } if role == "assistant");

                    if is_assistant_delta {
                        // Only use the final assistant message if we have not
                        // seen any deltas; otherwise, deltas already built the
                        // cumulative text and this would duplicate it.
                        if this.cumulative.is_empty() {
                            if let ResponseItem::Message { content, id, .. } = &item {
                                // Capture the item_id if present
                                if let Some(item_id) = id {
                                    this.cumulative_item_id = Some(item_id.clone());
                                }
                                if let Some(text) = content.iter().find_map(|c| match c {
                                    ContentItem::OutputText { text } => Some(text),
                                    _ => None,
                                }) {
                                    this.cumulative.push_str(text);
                                }
                            }
                        }
                        continue;
                    }

                    // Also capture item_id from Reasoning items
                    if let ResponseItem::Reasoning { id, .. } = &item {
                        if !id.is_empty() {
                            this.cumulative_item_id = Some(id.clone());
                        }
                    }

                    // Not an assistant message – forward immediately.
                    return Poll::Ready(Some(Ok(ResponseEvent::OutputItemDone {
                        item,
                        sequence_number: None,
                        output_index: None,
                    })));
                }
                Poll::Ready(Some(Ok(ResponseEvent::RateLimits(snapshot)))) => {
                    return Poll::Ready(Some(Ok(ResponseEvent::RateLimits(snapshot))));
                }
                Poll::Ready(Some(Ok(ResponseEvent::ServerReasoningIncluded(included)))) => {
                    return Poll::Ready(Some(Ok(ResponseEvent::ServerReasoningIncluded(included))));
                }
                Poll::Ready(Some(Ok(ResponseEvent::ModelsEtag(etag)))) => {
                    return Poll::Ready(Some(Ok(ResponseEvent::ModelsEtag(etag))));
                }
                Poll::Ready(Some(Ok(ResponseEvent::ResponseHeaders(headers)))) => {
                    return Poll::Ready(Some(Ok(ResponseEvent::ResponseHeaders(headers))));
                }
                Poll::Ready(Some(Ok(ResponseEvent::Completed {
                    response_id,
                    token_usage,
                }))) => {
                    // Build any aggregated items in the correct order: Reasoning first, then Message.
                    let mut emitted_any = false;

                    if !this.cumulative_reasoning.is_empty()
                        && matches!(this.mode, AggregateMode::AggregatedOnly)
                    {
                        let aggregated_reasoning = ResponseItem::Reasoning {
                            id: this.cumulative_item_id.clone().unwrap_or_else(String::new),
                            summary: Vec::new(),
                            content: Some(vec![ReasoningItemContent::ReasoningText {
                                text: std::mem::take(&mut this.cumulative_reasoning),
                            }]),
                            encrypted_content: None,
                        };
                        this.pending.push_back(ResponseEvent::OutputItemDone {
                            item: aggregated_reasoning,
                            sequence_number: None,
                            output_index: None,
                        });
                        emitted_any = true;
                    }

                    // Always emit the final aggregated assistant message when any
                    // content deltas have been observed. In AggregatedOnly mode this
                    // is the sole assistant output; in Streaming mode this finalizes
                    // the streamed deltas into a terminal OutputItemDone so callers
                    // can persist/render the message once per turn.
                    if !this.cumulative.is_empty() {
                        let aggregated_message = ResponseItem::Message {
                            id: this.cumulative_item_id.clone(),
                            role: "assistant".to_string(),
                            content: vec![code_protocol::models::ContentItem::OutputText {
                                text: std::mem::take(&mut this.cumulative),
                            }],
                            end_turn: None,
                            phase: None,
                        };
                        this.pending.push_back(ResponseEvent::OutputItemDone {
                            item: aggregated_message,
                            sequence_number: None,
                            output_index: None,
                        });
                        emitted_any = true;
                    }

                    // Always emit Completed last when anything was aggregated.
                    if emitted_any {
                        this.pending.push_back(ResponseEvent::Completed {
                            response_id: response_id.clone(),
                            token_usage: token_usage.clone(),
                        });
                        // Return the first pending event now.
                        if let Some(ev) = this.pending.pop_front() {
                            return Poll::Ready(Some(Ok(ev)));
                        }
                    }

                    // Nothing aggregated – forward Completed directly.
                    return Poll::Ready(Some(Ok(ResponseEvent::Completed {
                        response_id,
                        token_usage,
                    })));
                }
                Poll::Ready(Some(Ok(ResponseEvent::Created {
                    response_id,
                    response_model,
                }))) => {
                    // Preserve response metadata so downstream consumers can
                    // surface effective model routing details uniformly.
                    return Poll::Ready(Some(Ok(ResponseEvent::Created {
                        response_id,
                        response_model,
                    })));
                }
                Poll::Ready(Some(Ok(ResponseEvent::OutputTextDelta {
                    delta,
                    item_id,
                    sequence_number,
                    ..
                }))) => {
                    // Always accumulate deltas so we can emit a final OutputItemDone at Completed.
                    this.cumulative.push_str(&delta);
                    // Capture the item_id if we haven't already
                    if item_id.is_some() && this.cumulative_item_id.is_none() {
                        this.cumulative_item_id = item_id.clone();
                    }
                    if matches!(this.mode, AggregateMode::Streaming) {
                        // In streaming mode, also forward the delta immediately.
                        return Poll::Ready(Some(Ok(ResponseEvent::OutputTextDelta {
                            delta,
                            item_id,
                            sequence_number,
                            output_index: None,
                        })));
                    } else {
                        continue;
                    }
                }
                Poll::Ready(Some(Ok(ResponseEvent::ReasoningContentDelta {
                    delta,
                    item_id,
                    sequence_number,
                    ..
                }))) => {
                    // Always accumulate reasoning deltas so we can emit a final Reasoning item at Completed.
                    this.cumulative_reasoning.push_str(&delta);
                    // Capture the item_id if we haven't already
                    if item_id.is_some() && this.cumulative_item_id.is_none() {
                        this.cumulative_item_id = item_id.clone();
                    }
                    if matches!(this.mode, AggregateMode::Streaming) {
                        // In streaming mode, also forward the delta immediately.
                        return Poll::Ready(Some(Ok(ResponseEvent::ReasoningContentDelta {
                            delta,
                            item_id,
                            sequence_number,
                            output_index: None,
                            content_index: None,
                        })));
                    } else {
                        continue;
                    }
                }
                Poll::Ready(Some(Ok(ResponseEvent::ReasoningSummaryDelta { .. }))) => {
                    continue;
                }
                Poll::Ready(Some(Ok(ResponseEvent::ReasoningSummaryPartAdded))) => {
                    continue;
                }
                Poll::Ready(Some(Ok(ResponseEvent::WebSearchCallBegin { call_id }))) => {
                    return Poll::Ready(Some(Ok(ResponseEvent::WebSearchCallBegin { call_id })));
                }
                Poll::Ready(Some(Ok(ResponseEvent::WebSearchCallCompleted { call_id, query }))) => {
                    return Poll::Ready(Some(Ok(ResponseEvent::WebSearchCallCompleted {
                        call_id,
                        query,
                    })));
                }
            }
        }
    }
}

/// Extension trait that activates aggregation on any stream of [`ResponseEvent`].
pub(crate) trait AggregateStreamExt: Stream<Item = Result<ResponseEvent>> + Sized {
    /// Returns a new stream that emits **only** the final assistant message
    /// per turn instead of every incremental delta.  The produced
    /// `ResponseEvent` sequence for a typical text turn looks like:
    ///
    /// ```ignore
    ///     OutputItemDone { item: <full message>, .. }
    ///     Completed
    /// ```
    ///
    /// No other `OutputItemDone` events will be seen by the caller.
    ///
    /// Usage:
    ///
    /// ```ignore
    /// let agg_stream = client.stream(&prompt).await?.aggregate();
    /// while let Some(event) = agg_stream.next().await {
    ///     // event now contains cumulative text
    /// }
    /// ```
    fn aggregate(self) -> AggregatedChatStream<Self> {
        AggregatedChatStream::new(self, AggregateMode::AggregatedOnly)
    }
}

impl<T> AggregateStreamExt for T where T: Stream<Item = Result<ResponseEvent>> + Sized {}

impl<S> AggregatedChatStream<S> {
    fn new(inner: S, mode: AggregateMode) -> Self {
        AggregatedChatStream {
            inner,
            cumulative: String::new(),
            cumulative_reasoning: String::new(),
            cumulative_item_id: None,
            pending: std::collections::VecDeque::new(),
            mode,
        }
    }

    pub(crate) fn streaming_mode(inner: S) -> Self {
        Self::new(inner, AggregateMode::Streaming)
    }
}

fn header_map_to_json(headers: &HeaderMap) -> serde_json::Value {
    let mut ordered: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, value) in headers.iter() {
        let entry = ordered.entry(name.as_str().to_string()).or_default();
        entry.push(value.to_str().unwrap_or_default().to_string());
    }

    serde_json::to_value(ordered).unwrap_or(serde_json::Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_logger::DebugLogger;
    use crate::model_family::derive_default_model_family;
    use crate::model_family::ChatCompletionsRoleStrategy;
    use crate::model_family::ChatCompletionsReasoningStrategy;
    use code_protocol::models::ContentItem;
    use code_protocol::models::ReasoningItemContent;
    use futures::stream;
    use pretty_assertions::assert_eq;

    #[test]
    fn minimax_chat_payload_collapses_non_chat_roles_into_one_system_message() {
        let provider = crate::model_provider_info::create_minimax_provider();
        let model_family = derive_default_model_family("MiniMax-M2.7");
        let prompt = Prompt {
            input: vec![
                ResponseItem::Message {
                    id: None,
                    role: "developer".to_string(),
                    content: vec![ContentItem::InputText {
                        text: "developer guidance".to_string(),
                    }],
                    end_turn: None,
                    phase: None,
                },
                ResponseItem::Message {
                    id: None,
                    role: "user".to_string(),
                    content: vec![ContentItem::InputText {
                        text: "hello".to_string(),
                    }],
                    end_turn: None,
                    phase: None,
                },
            ],
            base_instructions_override: Some("base instructions".to_string()),
            include_additional_instructions: false,
            ..Default::default()
        };

        let payload =
            build_chat_completions_payload(&prompt, &model_family, "MiniMax-M2.7", &provider)
                .expect("payload should build");
        assert_eq!(payload["model"], "MiniMax-M2.7");
        assert_eq!(payload["reasoning_split"], true);

        let messages = payload["messages"].as_array().expect("messages array");
        assert_eq!(
            messages
                .iter()
                .filter(|message| message["role"] == "system")
                .count(),
            1
        );
        assert!(
            messages[0]["content"]
                .as_str()
                .expect("system content")
                .contains("base instructions")
        );
        assert!(
            messages[0]["content"]
                .as_str()
                .expect("system content")
                .contains("developer guidance")
        );
        assert!(
            messages
                .iter()
                .all(|message| message["role"] != "developer"),
            "MiniMax payload must not include unsupported developer role"
        );
    }

    #[test]
    fn openrouter_shorthand_routing_keys_are_nested_under_provider() {
        let mut provider = crate::model_provider_info::create_openrouter_provider();
        provider.openrouter = Some(crate::model_provider_info::OpenRouterConfig {
            provider: Some(crate::model_provider_info::OpenRouterProviderConfig {
                allow_fallbacks: Some(false),
                ..Default::default()
            }),
            route: None,
            extra: BTreeMap::from([
                ("order".to_string(), json!(["Anthropic", "Google"])),
                ("require_parameters".to_string(), json!(true)),
                ("transforms".to_string(), json!(["middle-out"])),
            ]),
        });
        let model_family = derive_default_model_family("anthropic/claude-sonnet-4.5");
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
            include_additional_instructions: false,
            ..Default::default()
        };

        let payload = build_chat_completions_payload(
            &prompt,
            &model_family,
            "anthropic/claude-sonnet-4.5",
            &provider,
        )
        .expect("payload should build");

        assert_eq!(payload["provider"]["allow_fallbacks"], false);
        assert_eq!(payload["provider"]["require_parameters"], true);
        assert_eq!(payload["provider"]["order"], json!(["Anthropic", "Google"]));
        assert_eq!(payload["transforms"], json!(["middle-out"]));
        assert!(payload.get("require_parameters").is_none());
    }

    #[test]
    fn minimax_payload_repairs_invalid_tool_call_arguments_json() {
        let provider = crate::model_provider_info::create_minimax_provider();
        let model_family = derive_default_model_family("MiniMax-M2.7");

        let prompt = Prompt {
            input: vec![ResponseItem::FunctionCall {
                id: Some("item-1".to_string()),
                name: "shell".to_string(),
                namespace: None,
                arguments: "cat /tmp/SKILL.md".to_string(),
                call_id: "call-1".to_string(),
            }],
            ..Prompt::default()
        };

        let payload =
            build_chat_completions_payload(&prompt, &model_family, "MiniMax-M2.7", &provider)
                .expect("payload");
        let messages = payload["messages"].as_array().expect("messages array");
        let tool_message = messages
            .iter()
            .find(|message| message.get("tool_calls").is_some())
            .expect("tool call message");
        let arguments = tool_message["tool_calls"][0]["function"]["arguments"]
            .as_str()
            .expect("arguments string");
        let parsed: serde_json::Value =
            serde_json::from_str(arguments).expect("MiniMax tool arguments must be valid JSON");

        assert_eq!(parsed["_raw"], "cat /tmp/SKILL.md");
    }

    #[test]
    fn opencode_go_payload_repairs_invalid_tool_call_arguments_json() {
        let provider = crate::model_provider_info::create_opencode_go_provider();
        let model_family = crate::model_family::find_family_for_model("opencode-go/kimi-k2.6")
            .expect("known kimi model");

        let prompt = Prompt {
            input: vec![ResponseItem::FunctionCall {
                id: Some("item-1".to_string()),
                name: "shell".to_string(),
                namespace: None,
                arguments: "cat package.json".to_string(),
                call_id: "call-1".to_string(),
            }],
            ..Prompt::default()
        };

        let payload = build_chat_completions_payload(
            &prompt,
            &model_family,
            "opencode-go/kimi-k2.6",
            &provider,
        )
        .expect("payload");
        let messages = payload["messages"].as_array().expect("messages array");
        let tool_message = messages
            .iter()
            .find(|message| message.get("tool_calls").is_some())
            .expect("tool call message");
        let arguments = tool_message["tool_calls"][0]["function"]["arguments"]
            .as_str()
            .expect("arguments string");
        let parsed: serde_json::Value =
            serde_json::from_str(arguments).expect("OpenCode Go tool arguments must be valid JSON");

        assert_eq!(parsed["_raw"], "cat package.json");
    }

    #[test]
    fn openai_compatible_non_mimo_chat_payload_serializes_output_schema() {
        let provider = crate::model_provider_info::create_opencode_go_provider();
        let model_family = crate::model_family::find_family_for_model(
            "opencode-go/deepseek-v4-flash",
        )
        .expect("known opencode go deepseek model");
        let schema = json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["summary"],
            "properties": {
                "summary": { "type": "string" }
            }
        });
        let prompt = Prompt {
            input: vec![ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "return JSON".to_string(),
                }],
                end_turn: None,
                phase: None,
            }],
            output_schema: Some(schema.clone()),
            ..Prompt::default()
        };

        let payload = build_chat_completions_payload(
            &prompt,
            &model_family,
            "deepseek-v4-flash",
            &provider,
        )
        .expect("payload");

        assert_eq!(payload["response_format"]["type"], "json_schema");
        assert_eq!(
            payload["response_format"]["json_schema"]["name"],
            "code_output_schema"
        );
        assert_eq!(
            payload["response_format"]["json_schema"]["strict"],
            true
        );
        assert_eq!(
            payload["response_format"]["json_schema"]["schema"],
            schema
        );
    }

    #[test]
    fn mimo_chat_payload_uses_schema_guidance_instead_of_response_format() {
        let provider = crate::model_provider_info::create_xiaomi_provider();
        let model_family = crate::model_family::find_family_for_model("xiaomi/mimo-v2.5-pro")
            .expect("known xiaomi mimo model");
        let prompt = Prompt {
            input: vec![ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "return JSON".to_string(),
                }],
                end_turn: None,
                phase: None,
            }],
            output_schema: Some(json!({
                "type": "object",
                "required": ["summary"],
                "properties": {
                    "summary": { "type": "string" }
                }
            })),
            ..Prompt::default()
        };

        let payload = build_chat_completions_payload(
            &prompt,
            &model_family,
            "mimo-v2.5-pro",
            &provider,
        )
        .expect("payload");

        assert!(
            payload.get("response_format").is_none(),
            "MiMo chat payloads should avoid response_format because direct Xiaomi can disconnect"
        );
        let messages = payload["messages"].as_array().expect("messages array");
        assert!(
            messages.iter().any(|message| {
                message["role"] == "system"
                    && message["content"]
                        .as_str()
                        .is_some_and(|content| content.contains("Final output contract"))
            }),
            "MiMo should receive schema guidance for no-tool structured output"
        );
    }

    #[test]
    fn openai_compatible_chat_payload_defers_output_schema_when_tools_are_available() {
        let provider = crate::model_provider_info::create_xiaomi_provider();
        let model_family = crate::model_family::find_family_for_model("xiaomi/mimo-v2.5-pro")
            .expect("known xiaomi mimo model");
        let prompt = Prompt {
            input: vec![ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "edit files first".to_string(),
                }],
                end_turn: None,
                phase: None,
            }],
            tools: vec![crate::openai_tools::OpenAiTool::LocalShell {}],
            output_schema: Some(json!({
                "type": "object",
                "required": ["summary"],
                "properties": {
                    "summary": { "type": "string" }
                }
            })),
            ..Prompt::default()
        };

        let payload = build_chat_completions_payload(
            &prompt,
            &model_family,
            "mimo-v2.5-pro",
            &provider,
        )
        .expect("payload");

        assert!(
            payload.get("response_format").is_none(),
            "response_format should not short-circuit tool-capable Chat Completions turns"
        );
        assert!(payload["tools"].as_array().is_some_and(|tools| !tools.is_empty()));
        let messages = payload["messages"].as_array().expect("messages array");
        assert!(
            messages.iter().any(|message| {
                message["role"] == "system"
                    && message["content"]
                        .as_str()
                        .is_some_and(|content| content.contains("Final output contract"))
            }),
            "tool-capable turns should still carry bounded final-output schema guidance"
        );
    }

    #[test]
    fn minimax_chat_payload_carries_output_schema_guidance_without_failing() {
        let provider = crate::model_provider_info::create_minimax_provider();
        let model_family = derive_default_model_family("MiniMax-M2.7");
        let prompt = Prompt {
            input: vec![ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "return JSON".to_string(),
                }],
                end_turn: None,
                phase: None,
            }],
            output_schema: Some(json!({
                "type": "object",
                "required": ["summary"],
                "properties": {
                    "summary": { "type": "string" }
                }
            })),
            ..Prompt::default()
        };

        let payload = build_chat_completions_payload(&prompt, &model_family, "MiniMax-M2.7", &provider)
            .expect("payload");

        assert!(
            payload.get("response_format").is_none(),
            "MiniMax chat payloads should not use response_format"
        );
        let messages = payload["messages"].as_array().expect("messages array");
        assert!(
            messages.iter().any(|message| {
                message["role"] == "system"
                    && message["content"]
                        .as_str()
                        .is_some_and(|content| content.contains("Final output contract"))
            }),
            "MiniMax should receive schema guidance instead of failing the turn"
        );
    }

    #[test]
    fn qwen_and_deepseek_chat_payload_collapses_developer_role_into_system_message() {
        let provider = crate::model_provider_info::create_opencode_go_provider();
        for model in ["qwen3.6-plus", "deepseek-v4-pro"] {
            let model_family = crate::model_family::find_family_for_model(model)
                .expect("known collapsed-role model");
            assert_eq!(
                model_family.chat_completions_role_strategy,
                ChatCompletionsRoleStrategy::CollapseNonChatRolesToSystem
            );
            let prompt = Prompt {
                input: vec![
                    ResponseItem::Message {
                        id: None,
                        role: "developer".to_string(),
                        content: vec![ContentItem::InputText {
                            text: "developer guidance".to_string(),
                        }],
                        end_turn: None,
                        phase: None,
                    },
                    ResponseItem::Message {
                        id: None,
                        role: "user".to_string(),
                        content: vec![ContentItem::InputText {
                            text: "hello".to_string(),
                        }],
                        end_turn: None,
                        phase: None,
                    },
                ],
                base_instructions_override: Some("base instructions".to_string()),
                include_additional_instructions: false,
                ..Default::default()
            };

            let payload = build_chat_completions_payload(&prompt, &model_family, model, &provider)
                .expect("payload should build");

            assert_eq!(payload["model"], model);

            let messages = payload["messages"].as_array().expect("messages array");
            assert_eq!(messages.len(), 3);
            assert_eq!(messages[0]["role"], "system");
            assert_eq!(messages[1]["role"], "system");
            assert_eq!(messages[2]["role"], "user");
            assert!(
                messages[1]["content"]
                    .as_str()
                    .expect("system content")
                    .contains("developer guidance")
            );
            assert!(
                messages
                    .iter()
                    .all(|message| message["role"] != "developer"),
                "{model} payload must not include unsupported developer role"
            );
        }
    }

    #[test]
    fn kimi_chat_payload_preserves_reasoning_content_on_tool_call_message() {
        let provider = crate::model_provider_info::create_opencode_go_provider();
        let model_family = crate::model_family::find_family_for_model("opencode-go/kimi-k2.6")
            .expect("known kimi model");
        assert_eq!(
            model_family.chat_completions_reasoning_strategy,
            ChatCompletionsReasoningStrategy::PreserveReasoningContent
        );

        let prompt = Prompt {
            input: vec![
                ResponseItem::Message {
                    id: None,
                    role: "user".to_string(),
                    content: vec![ContentItem::InputText {
                        text: "please search".to_string(),
                    }],
                    end_turn: None,
                    phase: None,
                },
                ResponseItem::Reasoning {
                    id: "reasoning-1".to_string(),
                    summary: Vec::new(),
                    content: Some(vec![ReasoningItemContent::ReasoningText {
                        text: "thinking".to_string(),
                    }]),
                    encrypted_content: None,
                },
                ResponseItem::FunctionCall {
                    id: Some("assistant-1".to_string()),
                    name: "search".to_string(),
                    namespace: None,
                    arguments: r#"{"query":"notes"}"#.to_string(),
                    call_id: "call-1".to_string(),
                },
            ],
            base_instructions_override: Some("base instructions".to_string()),
            include_additional_instructions: false,
            ..Default::default()
        };

        let payload = build_chat_completions_payload(
            &prompt,
            &model_family,
            "opencode-go/kimi-k2.6",
            &provider,
        )
        .expect("payload should build");

        let messages = payload["messages"].as_array().expect("messages array");
        let assistant = messages
            .iter()
            .find(|message| {
                message["role"] == "assistant"
                    && message["tool_calls"]
                        .as_array()
                        .is_some_and(|calls| !calls.is_empty())
            })
            .expect("assistant tool-call message should be present");

        assert_eq!(assistant["content"], serde_json::Value::Null);
        assert_eq!(assistant["reasoning_content"], "thinking");
        assert_eq!(assistant["tool_calls"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn deepseek_chat_payload_preserves_reasoning_content_on_assistant_message() {
        let provider = crate::model_provider_info::create_opencode_go_provider();
        let model_family = crate::model_family::find_family_for_model("deepseek-v4-pro")
            .expect("known deepseek model");
        assert_eq!(
            model_family.chat_completions_reasoning_strategy,
            ChatCompletionsReasoningStrategy::PreserveReasoningContent
        );

        let prompt = Prompt {
            input: vec![
                ResponseItem::Message {
                    id: None,
                    role: "user".to_string(),
                    content: vec![ContentItem::InputText {
                        text: "hello".to_string(),
                    }],
                    end_turn: None,
                    phase: None,
                },
                ResponseItem::Message {
                    id: None,
                    role: "assistant".to_string(),
                    content: vec![ContentItem::OutputText {
                        text: "assistant reply".to_string(),
                    }],
                    end_turn: None,
                    phase: None,
                },
                ResponseItem::Reasoning {
                    id: "reasoning-1".to_string(),
                    summary: Vec::new(),
                    content: Some(vec![ReasoningItemContent::ReasoningText {
                        text: "thinking".to_string(),
                    }]),
                    encrypted_content: None,
                },
                ResponseItem::Message {
                    id: None,
                    role: "user".to_string(),
                    content: vec![ContentItem::InputText {
                        text: "follow up".to_string(),
                    }],
                    end_turn: None,
                    phase: None,
                },
            ],
            base_instructions_override: Some("base instructions".to_string()),
            include_additional_instructions: false,
            ..Default::default()
        };

        let payload = build_chat_completions_payload(&prompt, &model_family, "deepseek-v4-pro", &provider)
            .expect("payload should build");

        let messages = payload["messages"].as_array().expect("messages array");
        let assistant = messages
            .iter()
            .find(|message| {
                message["role"] == "assistant"
                    && message["content"] == "assistant reply"
                    && message["reasoning_content"] == "thinking"
            })
            .expect("assistant message should preserve reasoning_content");

        assert_eq!(assistant["content"], "assistant reply");
        assert_eq!(assistant["reasoning_content"], "thinking");
    }

    #[tokio::test]
    async fn chat_sse_accepts_minimax_reasoning_content_delta() {
        let (tx, mut rx) = mpsc::channel::<Result<ResponseEvent>>(8);
        let bytes = Bytes::from_static(
            br#"data: {"id":"cmpl-1","model":"MiniMax-M2.7","choices":[{"delta":{"reasoning_content":"thinking","content":"OK"},"finish_reason":"stop"}]}

data: [DONE]

"#,
        );
        let debug_logger = Arc::new(Mutex::new(DebugLogger::new(false).unwrap()));

        process_chat_sse(
            stream::iter(vec![Ok(bytes)]),
            tx,
            Duration::from_secs(1),
            debug_logger,
            "test-request".to_string(),
            None,
        )
        .await;

        let mut saw_reasoning = false;
        while let Some(event) = rx.recv().await {
            if let ResponseEvent::ReasoningContentDelta { delta, .. } =
                event.expect("event should parse")
            {
                saw_reasoning = delta == "thinking";
            }
        }

        assert!(
            saw_reasoning,
            "MiniMax reasoning_content delta should be preserved"
        );
    }

    #[tokio::test]
    async fn chat_sse_accepts_xiaomi_mimo_content_and_reasoning_deltas() {
        let (tx, mut rx) = mpsc::channel::<Result<ResponseEvent>>(8);
        let bytes = Bytes::from_static(
            br#"data: {"id":"cmpl-xiaomi","model":"xiaomi/mimo-v2.5-pro-20260422","choices":[{"delta":{"reasoning_content":"thinking","content":"OK"},"finish_reason":"stop"}]}

data: [DONE]

"#,
        );
        let debug_logger = Arc::new(Mutex::new(DebugLogger::new(false).unwrap()));

        process_chat_sse(
            stream::iter(vec![Ok(bytes)]),
            tx,
            Duration::from_secs(1),
            debug_logger,
            "test-request".to_string(),
            None,
        )
        .await;

        let mut saw_model = false;
        let mut saw_reasoning = false;
        let mut saw_text = false;
        let mut saw_completed = false;
        while let Some(event) = rx.recv().await {
            match event.expect("event should parse") {
                ResponseEvent::Created { response_model, .. } => {
                    saw_model =
                        response_model.as_deref() == Some("xiaomi/mimo-v2.5-pro-20260422");
                }
                ResponseEvent::ReasoningContentDelta { delta, .. } => {
                    saw_reasoning = delta == "thinking";
                }
                ResponseEvent::OutputTextDelta { delta, .. } => {
                    saw_text = delta == "OK";
                }
                ResponseEvent::Completed { .. } => {
                    saw_completed = true;
                }
                _ => {}
            }
        }

        assert!(saw_model, "Xiaomi MiMo response model should be preserved");
        assert!(saw_reasoning, "Xiaomi MiMo reasoning_content should parse");
        assert!(saw_text, "Xiaomi MiMo assistant content should parse");
        assert!(saw_completed, "Xiaomi MiMo stream should complete");
    }

    #[tokio::test]
    async fn chat_sse_accepts_xiaomi_mimo_tool_call_deltas() {
        let (tx, mut rx) = mpsc::channel::<Result<ResponseEvent>>(8);
        let bytes = Bytes::from_static(
            br#"data: {"id":"cmpl-tool","model":"xiaomi/mimo-v2.5","choices":[{"delta":{"tool_calls":[{"id":"call_1","type":"function","function":{"name":"apply_patch","arguments":"{\"patch\":\""}}]},"finish_reason":null}]}

data: {"id":"cmpl-tool","model":"xiaomi/mimo-v2.5","choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"ok\"}"}}]},"finish_reason":"tool_calls"}]}

data: [DONE]

"#,
        );
        let debug_logger = Arc::new(Mutex::new(DebugLogger::new(false).unwrap()));

        process_chat_sse(
            stream::iter(vec![Ok(bytes)]),
            tx,
            Duration::from_secs(1),
            debug_logger,
            "test-request".to_string(),
            None,
        )
        .await;

        let mut tool_call = None;
        while let Some(event) = rx.recv().await {
            if let ResponseEvent::OutputItemDone {
                item:
                    ResponseItem::FunctionCall {
                        name,
                        arguments,
                        call_id,
                        ..
                    },
                ..
            } = event.expect("event should parse")
            {
                tool_call = Some((name, arguments, call_id));
            }
        }

        assert_eq!(
            tool_call,
            Some((
                "apply_patch".to_string(),
                r#"{"patch":"ok"}"#.to_string(),
                "call_1".to_string(),
            ))
        );
    }

    #[tokio::test]
    async fn aggregate_suppresses_raw_final_assistant_item_after_deltas() {
        let events = stream::iter(vec![
            Ok(ResponseEvent::OutputTextDelta {
                delta: "O".to_string(),
                item_id: Some("msg_1".to_string()),
                sequence_number: None,
                output_index: None,
            }),
            Ok(ResponseEvent::OutputTextDelta {
                delta: "K".to_string(),
                item_id: Some("msg_1".to_string()),
                sequence_number: None,
                output_index: None,
            }),
            Ok(ResponseEvent::OutputItemDone {
                item: ResponseItem::Message {
                    id: Some("msg_1".to_string()),
                    role: "assistant".to_string(),
                    content: vec![ContentItem::OutputText {
                        text: "OK".to_string(),
                    }],
                    end_turn: None,
                    phase: None,
                },
                sequence_number: None,
                output_index: None,
            }),
            Ok(ResponseEvent::Completed {
                response_id: "cmpl_1".to_string(),
                token_usage: None,
            }),
        ]);

        let collected: Vec<ResponseEvent> = events
            .aggregate()
            .map(|event| event.expect("aggregate event"))
            .collect()
            .await;

        assert_eq!(collected.len(), 2);
        match &collected[0] {
            ResponseEvent::OutputItemDone {
                item: ResponseItem::Message { content, .. },
                ..
            } => {
                assert_eq!(
                    content,
                    &vec![ContentItem::OutputText {
                        text: "OK".to_string()
                    }]
                );
            }
            other => panic!("expected one aggregated assistant item, got {other:?}"),
        }
        assert!(matches!(collected[1], ResponseEvent::Completed { .. }));
    }
}
