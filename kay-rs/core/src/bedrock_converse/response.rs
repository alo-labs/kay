use code_protocol::models::ContentItem;
use code_protocol::models::ResponseItem;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::client_common::ResponseEvent;
use crate::client_common::ResponseStream;
use crate::error::CodexErr;
use crate::error::Result;
use crate::protocol::TokenUsage;

pub(crate) async fn response_stream_from_bedrock_response(
    response_id: String,
    response_model: Option<String>,
    response_headers: Value,
    body: Value,
) -> Result<ResponseStream> {
    let events = parse_bedrock_output_items(&body)?;
    let token_usage = parse_bedrock_usage(&body);
    let channel_capacity = events.len().saturating_add(3).max(1);
    let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent>>(channel_capacity);
    tx_event
        .send(Ok(ResponseEvent::ResponseHeaders(response_headers)))
        .await
        .map_err(|_| CodexErr::InternalAgentDied)?;
    tx_event
        .send(Ok(ResponseEvent::Created {
            response_id: Some(response_id.clone()),
            response_model,
        }))
        .await
        .map_err(|_| CodexErr::InternalAgentDied)?;

    for event in events {
        tx_event
            .send(Ok(event))
            .await
            .map_err(|_| CodexErr::InternalAgentDied)?;
    }
    tx_event
        .send(Ok(ResponseEvent::Completed {
            response_id,
            token_usage,
        }))
        .await
        .map_err(|_| CodexErr::InternalAgentDied)?;

    Ok(ResponseStream { rx_event })
}

pub(crate) fn parse_bedrock_output_items(body: &Value) -> Result<Vec<ResponseEvent>> {
    let content = body
        .pointer("/output/message/content")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CodexErr::UnsupportedOperation(
                "Bedrock Converse response did not contain output.message.content".to_string(),
            )
        })?;
    let mut events = Vec::new();
    let mut text = String::new();

    for block in content {
        if let Some(part) = block.get("text").and_then(Value::as_str) {
            text.push_str(part);
            events.push(ResponseEvent::OutputTextDelta {
                delta: part.to_string(),
                item_id: None,
                sequence_number: None,
                output_index: None,
            });
        }
        if let Some(tool_use) = block.get("toolUse") {
            if !text.is_empty() {
                events.push(message_done(std::mem::take(&mut text)));
            }
            let name = tool_use
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let call_id = tool_use
                .get("toolUseId")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let arguments = serde_json::to_string(
                tool_use
                    .get("input")
                    .unwrap_or(&Value::Object(serde_json::Map::new())),
            )?;
            events.push(ResponseEvent::OutputItemDone {
                item: ResponseItem::FunctionCall {
                    id: None,
                    name,
                    namespace: None,
                    arguments,
                    call_id,
                },
                sequence_number: None,
                output_index: None,
            });
        }
    }

    if !text.is_empty() {
        events.push(message_done(text));
    }

    Ok(events)
}

fn message_done(text: String) -> ResponseEvent {
    ResponseEvent::OutputItemDone {
        item: ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText { text }],
            end_turn: None,
            phase: None,
        },
        sequence_number: None,
        output_index: None,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BedrockUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

pub(crate) fn parse_bedrock_usage(body: &Value) -> Option<TokenUsage> {
    let usage = serde_json::from_value::<BedrockUsage>(body.get("usage")?.clone()).ok()?;
    let input_tokens = usage.input_tokens.unwrap_or(0);
    let output_tokens = usage.output_tokens.unwrap_or(0);
    Some(TokenUsage {
        input_tokens,
        cached_input_tokens: 0,
        output_tokens,
        reasoning_output_tokens: 0,
        total_tokens: usage
            .total_tokens
            .unwrap_or_else(|| input_tokens.saturating_add(output_tokens)),
    })
}
