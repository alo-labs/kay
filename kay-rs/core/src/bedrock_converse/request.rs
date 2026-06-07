use std::collections::BTreeMap;

use http::HeaderMap;
use serde_json::Value;
use serde_json::json;

use crate::ModelProviderInfo;
use crate::chat_completions::build_chat_completions_payload;
use crate::client_common::Prompt;
use crate::error::CodexErr;
use crate::error::Result;
use crate::model_family::ModelFamily;

pub(crate) fn build_bedrock_converse_payload(
    prompt: &Prompt,
    model_family: &ModelFamily,
    model_slug: &str,
    provider: &ModelProviderInfo,
) -> Result<Value> {
    let chat_payload = build_chat_completions_payload(prompt, model_family, model_slug, provider)?;
    compile_chat_payload_to_bedrock_converse(&chat_payload)
}

pub(crate) fn compile_chat_payload_to_bedrock_converse(chat_payload: &Value) -> Result<Value> {
    let mut system = Vec::<Value>::new();
    let mut messages = Vec::<Value>::new();

    let chat_messages = chat_payload
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CodexErr::UnsupportedOperation(
                "Bedrock Converse adapter expected chat payload messages".to_string(),
            )
        })?;

    for message in chat_messages {
        let role = message.get("role").and_then(Value::as_str).unwrap_or("user");
        match role {
            "system" | "developer" => {
                push_system_content(&mut system, message.get("content"))?;
            }
            "assistant" => {
                messages.push(compile_assistant_message(message)?);
            }
            "tool" => {
                messages.push(compile_tool_result_message(message)?);
            }
            "user" => {
                messages.push(json!({
                    "role": "user",
                    "content": compile_content_blocks(message.get("content"))?,
                }));
            }
            other => {
                system.push(json!({
                    "text": format!("{other}: {}", content_as_text(message.get("content"))?)
                }));
            }
        }
    }

    let mut payload = serde_json::Map::new();
    payload.insert("messages".to_string(), Value::Array(messages));
    if !system.is_empty() {
        payload.insert("system".to_string(), Value::Array(system));
    }
    if let Some(tool_config) = compile_tool_config(chat_payload.get("tools"))? {
        payload.insert("toolConfig".to_string(), tool_config);
    }

    Ok(Value::Object(payload))
}

fn push_system_content(system: &mut Vec<Value>, content: Option<&Value>) -> Result<()> {
    let text = content_as_text(content)?;
    if !text.trim().is_empty() {
        system.push(json!({ "text": text }));
    }
    Ok(())
}

fn compile_assistant_message(message: &Value) -> Result<Value> {
    let mut content = Vec::<Value>::new();
    if let Some(value) = message.get("content")
        && !value.is_null()
    {
        content.extend(compile_content_blocks(Some(value))?);
    }
    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for tool_call in tool_calls {
            let function = tool_call.get("function").unwrap_or(&Value::Null);
            let name = function.get("name").and_then(Value::as_str).unwrap_or("");
            let arguments = function
                .get("arguments")
                .and_then(Value::as_str)
                .map(parse_tool_arguments)
                .transpose()?
                .unwrap_or_else(|| json!({}));
            let tool_use_id = tool_call
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("tool-use");
            content.push(json!({
                "toolUse": {
                    "toolUseId": tool_use_id,
                    "name": name,
                    "input": arguments,
                }
            }));
        }
    }

    Ok(json!({
        "role": "assistant",
        "content": content,
    }))
}

fn parse_tool_arguments(raw: &str) -> Result<Value> {
    Ok(serde_json::from_str::<Value>(raw).unwrap_or_else(|_| json!({ "_raw": raw })))
}

fn compile_tool_result_message(message: &Value) -> Result<Value> {
    let tool_use_id = message
        .get("tool_call_id")
        .and_then(Value::as_str)
        .unwrap_or("tool-use");
    let content = content_as_text(message.get("content"))?;
    Ok(json!({
        "role": "user",
        "content": [{
            "toolResult": {
                "toolUseId": tool_use_id,
                "content": [{ "text": content }],
            }
        }],
    }))
}

fn compile_content_blocks(content: Option<&Value>) -> Result<Vec<Value>> {
    match content {
        Some(Value::String(text)) => Ok(non_empty_text_blocks(text)),
        Some(Value::Array(parts)) => {
            let mut blocks = Vec::new();
            for part in parts {
                match part.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        if let Some(text) = part.get("text").and_then(Value::as_str)
                            && !text.is_empty()
                        {
                            blocks.push(json!({ "text": text }));
                        }
                    }
                    Some("image_url") => {
                        return Err(CodexErr::UnsupportedOperation(
                            "Bedrock Converse image_url translation is not implemented yet"
                                .to_string(),
                        ));
                    }
                    _ => {}
                }
            }
            Ok(blocks)
        }
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(other) => Ok(non_empty_text_blocks(&other.to_string())),
    }
}

fn non_empty_text_blocks(text: &str) -> Vec<Value> {
    if text.is_empty() {
        Vec::new()
    } else {
        vec![json!({ "text": text })]
    }
}

fn content_as_text(content: Option<&Value>) -> Result<String> {
    match content {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(Value::Array(parts)) => {
            let mut text = String::new();
            for part in parts {
                match part.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        if let Some(part_text) = part.get("text").and_then(Value::as_str) {
                            text.push_str(part_text);
                        }
                    }
                    Some("image_url") => {
                        return Err(CodexErr::UnsupportedOperation(
                            "Bedrock Converse image_url translation is not implemented yet"
                                .to_string(),
                        ));
                    }
                    _ => {}
                }
            }
            Ok(text)
        }
        Some(Value::Null) | None => Ok(String::new()),
        Some(other) => Ok(other.to_string()),
    }
}

fn compile_tool_config(tools: Option<&Value>) -> Result<Option<Value>> {
    let Some(tools) = tools.and_then(Value::as_array) else {
        return Ok(None);
    };
    let mut bedrock_tools = Vec::new();
    for tool in tools {
        let Some(function) = tool.get("function") else {
            continue;
        };
        let Some(name) = function.get("name").and_then(Value::as_str) else {
            continue;
        };
        let description = function
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("");
        let input_schema = function
            .get("parameters")
            .cloned()
            .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
        bedrock_tools.push(json!({
            "toolSpec": {
                "name": name,
                "description": description,
                "inputSchema": { "json": input_schema },
            }
        }));
    }

    if bedrock_tools.is_empty() {
        Ok(None)
    } else {
        Ok(Some(json!({ "tools": bedrock_tools })))
    }
}

pub(crate) fn bedrock_converse_url(base_url: &str, model_slug: &str) -> String {
    let (base, query) = base_url.split_once('?').unwrap_or((base_url, ""));
    let encoded_model = urlencoding::encode(model_slug);
    let mut endpoint = format!(
        "{}/model/{}/converse",
        base.trim_end_matches('/'),
        encoded_model
    );
    if !query.is_empty() {
        endpoint.push('?');
        endpoint.push_str(query);
    }
    endpoint
}

pub(crate) fn bedrock_region_from_url(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_string))
        .and_then(|host| {
            let parts = host.split('.').collect::<Vec<_>>();
            if parts.first() == Some(&"bedrock-runtime") {
                parts.get(1).map(|region| (*region).to_string())
            } else {
                None
            }
        })
}

pub(crate) fn header_map_to_json(headers: &HeaderMap) -> Value {
    let mut ordered: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, value) in headers {
        let entry = ordered.entry(name.as_str().to_string()).or_default();
        entry.push(value.to_str().unwrap_or_default().to_string());
    }

    serde_json::to_value(ordered).unwrap_or(Value::Null)
}
