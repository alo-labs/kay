use std::collections::BTreeMap;

use code_protocol::models::ContentItem;
use code_protocol::models::ResponseItem;
use http::HeaderMap;
use pretty_assertions::assert_eq;
use serde_json::json;

use crate::client_common::Prompt;
use crate::client_common::ResponseEvent;
use crate::model_family::derive_default_model_family;
use crate::model_provider_info::create_amazon_bedrock_provider;
use crate::openai_tools::JsonSchema;
use crate::openai_tools::OpenAiTool;
use crate::openai_tools::ResponsesApiTool;
use crate::protocol::TokenUsage;

use super::request::bedrock_converse_url;
use super::request::bedrock_region_from_url;
use super::request::build_bedrock_converse_payload;
use super::request::compile_chat_payload_to_bedrock_converse;
use super::request::header_map_to_json;
use super::response::parse_bedrock_output_items;
use super::response::parse_bedrock_usage;
use super::response::response_stream_from_bedrock_response;

fn basic_prompt() -> Prompt {
    Prompt {
        input: vec![ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "hello".to_string(),
            }],
            end_turn: None,
            phase: None,
        }],
        base_instructions_override: Some("be terse".to_string()),
        include_additional_instructions: false,
        ..Prompt::default()
    }
}

#[test]
fn compiles_prompt_and_function_tools_to_bedrock_converse() {
    let mut prompt = basic_prompt();
    prompt.set_tools(vec![OpenAiTool::Function(ResponsesApiTool {
        name: "lookup".to_string(),
        description: "Look up a value".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties: BTreeMap::from([(
                "key".to_string(),
                JsonSchema::String {
                    description: None,
                    allowed_values: None,
                },
            )]),
            required: Some(vec!["key".to_string()]),
            additional_properties: Some(false.into()),
        },
    })]);
    let provider = create_amazon_bedrock_provider();
    let payload = build_bedrock_converse_payload(
        &prompt,
        &derive_default_model_family("anthropic.claude-3-5-sonnet-20240620-v1:0"),
        "anthropic.claude-3-5-sonnet-20240620-v1:0",
        &provider,
    )
    .expect("payload should compile");

    assert_eq!(payload["system"], json!([{ "text": "be terse" }]));
    assert_eq!(
        payload["messages"],
        json!([{ "role": "user", "content": [{ "text": "hello" }] }])
    );
    assert_eq!(
        payload["toolConfig"]["tools"][0]["toolSpec"]["name"],
        "lookup"
    );
    assert_eq!(
        payload["toolConfig"]["tools"][0]["toolSpec"]["inputSchema"]["json"]["properties"]["key"]
            ["type"],
        "string"
    );
}

#[test]
fn compiles_tool_history_to_bedrock_converse_blocks() {
    let chat_payload = json!({
        "messages": [
            { "role": "assistant", "content": null, "tool_calls": [{
                "id": "call_1",
                "type": "function",
                "function": {
                    "name": "lookup",
                    "arguments": "{\"key\":\"value\"}"
                }
            }]},
            { "role": "tool", "tool_call_id": "call_1", "content": "done" }
        ],
        "tools": []
    });
    let payload =
        compile_chat_payload_to_bedrock_converse(&chat_payload).expect("payload should compile");

    assert_eq!(
        payload["messages"],
        json!([
            {
                "role": "assistant",
                "content": [{
                    "toolUse": {
                        "toolUseId": "call_1",
                        "name": "lookup",
                        "input": { "key": "value" }
                    }
                }]
            },
            {
                "role": "user",
                "content": [{
                    "toolResult": {
                        "toolUseId": "call_1",
                        "content": [{ "text": "done" }]
                    }
                }]
            }
        ])
    );
}

#[test]
fn preserves_invalid_tool_arguments_as_raw_payload() {
    let chat_payload = json!({
        "messages": [{
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": "call_1",
                "type": "function",
                "function": {
                    "name": "lookup",
                    "arguments": "not-json"
                }
            }]
        }],
        "tools": []
    });
    let payload =
        compile_chat_payload_to_bedrock_converse(&chat_payload).expect("payload should compile");

    assert_eq!(
        payload["messages"][0]["content"][0]["toolUse"]["input"],
        json!({ "_raw": "not-json" })
    );
}

#[test]
fn parses_bedrock_text_and_tool_use_response() {
    let body = json!({
        "output": {
            "message": {
                "role": "assistant",
                "content": [
                    { "text": "checking" },
                    {
                        "toolUse": {
                            "toolUseId": "tool_1",
                            "name": "lookup",
                            "input": { "key": "value" }
                        }
                    }
                ]
            }
        },
        "usage": {
            "inputTokens": 10,
            "outputTokens": 5,
            "totalTokens": 15
        }
    });

    let events = parse_bedrock_output_items(&body).expect("response should parse");
    assert!(matches!(
        &events[0],
        ResponseEvent::OutputTextDelta { delta, .. } if delta == "checking"
    ));
    assert!(matches!(
        &events[1],
        ResponseEvent::OutputItemDone {
            item: ResponseItem::Message { content, .. },
            ..
        } if content == &vec![ContentItem::OutputText { text: "checking".to_string() }]
    ));
    assert!(matches!(
        &events[2],
        ResponseEvent::OutputItemDone {
            item: ResponseItem::FunctionCall { name, call_id, arguments, .. },
            ..
        } if name == "lookup" && call_id == "tool_1" && arguments == "{\"key\":\"value\"}"
    ));
    assert_eq!(
        parse_bedrock_usage(&body),
        Some(TokenUsage {
            input_tokens: 10,
            cached_input_tokens: 0,
            output_tokens: 5,
            reasoning_output_tokens: 0,
            total_tokens: 15,
        })
    );
}

#[test]
fn builds_model_specific_converse_url_and_region() {
    let endpoint = bedrock_converse_url(
        "https://bedrock-runtime.us-west-2.amazonaws.com?trace=on",
        "anthropic.claude-3-5-sonnet-20240620-v1:0",
    );

    assert_eq!(
        endpoint,
        "https://bedrock-runtime.us-west-2.amazonaws.com/model/anthropic.claude-3-5-sonnet-20240620-v1%3A0/converse?trace=on"
    );
    assert_eq!(bedrock_region_from_url(&endpoint).as_deref(), Some("us-west-2"));
}

#[test]
fn rejects_bedrock_image_url_translation_until_native_image_blocks_exist() {
    let chat_payload = json!({
        "messages": [{
            "role": "user",
            "content": [{
                "type": "image_url",
                "image_url": { "url": "data:image/png;base64,AAA=" }
            }]
        }]
    });

    let err = compile_chat_payload_to_bedrock_converse(&chat_payload)
        .expect_err("image_url should require native adapter work");
    assert!(err.to_string().contains("image_url translation"));
}

#[test]
fn converts_headers_to_json_deterministically() {
    let mut headers = HeaderMap::new();
    headers.insert("x-test", http::HeaderValue::from_static("a"));
    headers.append("x-test", http::HeaderValue::from_static("b"));

    assert_eq!(header_map_to_json(&headers), json!({ "x-test": ["a", "b"] }));
}

#[tokio::test]
async fn response_stream_from_many_blocks_does_not_block_before_receiver_poll() {
    let content = (0..40)
        .map(|index| json!({ "text": format!("{index};") }))
        .collect::<Vec<_>>();
    let body = json!({
        "output": {
            "message": {
                "role": "assistant",
                "content": content
            }
        }
    });
    let stream = tokio::time::timeout(
        std::time::Duration::from_millis(250),
        response_stream_from_bedrock_response(
            "response-id".to_string(),
            Some("model".to_string()),
            json!({}),
            body,
        ),
    )
    .await
    .expect("adapter should not deadlock before returning receiver")
    .expect("response should convert");

    let mut rx_event = stream.rx_event;
    let mut event_count = 0;
    while let Some(event) = rx_event.recv().await {
        event.expect("queued event should be valid");
        event_count += 1;
    }
    assert_eq!(event_count, 44);
}
