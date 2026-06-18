#![allow(dead_code)]

use std::path::Path;

use code_core::model_family::ModelCompatibilityProfile;
use code_core::model_family::compatibility_profile_for_model;
use code_core::model_family::response_model_matches_request;
use code_core::model_family::wire_model_slug;
use code_core::model_family::uses_opencode_go_anthropic_messages;
use serde_json::Value;

pub fn env_flag_enabled(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .ok()
            .map(|value| value.trim().to_string())
            .as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

pub fn tui_live_smoke_enabled() -> bool {
    env_flag_enabled("KAY_TUI_PROVIDER_LIVE_SMOKE")
}

pub fn tui_ux_live_smoke_enabled() -> bool {
    env_flag_enabled("KAY_TUI_UX_LIVE_SMOKE")
}

pub fn live_smoke_enabled() -> bool {
    matches!(
        std::env::var("KAY_PROVIDER_MODEL_LIVE_SMOKE")
            .ok()
            .map(|value| value.trim().to_string())
            .as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

pub fn selected_models(all_models: &[String]) -> Vec<String> {
    let Some(filter) = std::env::var("KAY_PROVIDER_MODEL_LIVE_SMOKE_MODEL_FILTER")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return all_models.to_vec();
    };

    let allowed: Vec<String> = filter
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect();

    all_models
        .iter()
        .filter(|model| {
            allowed.iter().any(|entry| {
                entry.eq_ignore_ascii_case(model.as_str())
                    || model.ends_with(&format!("/{entry}"))
                    || model.ends_with(entry)
            })
        })
        .cloned()
        .collect()
}

pub fn compatibility_profile(provider_id: &str, model: &str) -> ModelCompatibilityProfile {
    compatibility_profile_for_model(provider_id, model)
}

pub fn assert_wire_profile(provider_id: &str, model: &str, profile: &ModelCompatibilityProfile) {
    assert_eq!(
        wire_model_slug(provider_id, model),
        profile.expected_wire_slug,
        "wire slug mismatch for {model}"
    );
    assert_eq!(
        uses_opencode_go_anthropic_messages(provider_id, model),
        profile.uses_anthropic_messages_wire,
        "anthropic wire routing mismatch for {model}"
    );
    assert!(
        response_model_matches_request(model, &profile.expected_wire_slug),
        "response slug normalization should accept wire slug for {model}"
    );
}

pub fn parse_thread_events(stdout: &str) -> Vec<Value> {
    stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect()
}

fn exec_command_succeeded(event: &Value, needle: &str) -> bool {
    let Some(msg_type) = event.pointer("/msg/type").and_then(Value::as_str) else {
        return false;
    };
    if msg_type != "exec_command_end" {
        return false;
    }
    if event.pointer("/msg/exit_code").and_then(Value::as_i64) != Some(0) {
        return false;
    }

    let command = event
        .pointer("/msg/command")
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .or_else(|| event.pointer("/msg/command").and_then(Value::as_str).map(str::to_string));
    if command
        .as_deref()
        .is_some_and(|command| command.contains(needle))
    {
        return true;
    }

    event
        .pointer("/msg/stdout")
        .and_then(Value::as_str)
        .is_some_and(|stdout| stdout.contains(needle))
}

fn patch_apply_succeeded(event: &Value, path: &str) -> bool {
    match event.pointer("/msg/type").and_then(Value::as_str) {
        Some("patch_apply_begin") => event
            .pointer("/msg/changes")
            .and_then(Value::as_object)
            .is_some_and(|changes| changes.keys().any(|key| key.ends_with(path))),
        Some("patch_apply_end") => event.pointer("/msg/success").and_then(Value::as_bool) == Some(true),
        _ => false,
    }
}

pub fn completed_shell_command(events: &[Value], needle: &str) -> bool {
    events.iter().any(|event| {
        if event.get("type").and_then(Value::as_str) == Some("item.completed")
            && event.pointer("/item/type").and_then(Value::as_str) == Some("command_execution")
            && event.pointer("/item/status").and_then(Value::as_str) == Some("completed")
            && event
                .pointer("/item/command")
                .and_then(Value::as_str)
                .is_some_and(|command| command.contains(needle))
        {
            return true;
        }

        exec_command_succeeded(event, needle)
    })
}

pub fn completed_file_change(events: &[Value], path: &str) -> bool {
    events.iter().any(|event| {
        if event.get("type").and_then(Value::as_str) == Some("item.completed")
            && event.pointer("/item/type").and_then(Value::as_str) == Some("file_change")
            && event.pointer("/item/status").and_then(Value::as_str) == Some("completed")
            && event
                .pointer("/item/changes")
                .and_then(Value::as_array)
                .is_some_and(|changes| {
                    changes.iter().any(|change| {
                        change.get("path").and_then(Value::as_str) == Some(path)
                    })
                })
        {
            return true;
        }

        patch_apply_succeeded(event, path)
    })
}

pub fn read_workspace_file(workspace: &Path, relative_path: &str) -> String {
    std::fs::read_to_string(workspace.join(relative_path))
        .unwrap_or_else(|err| panic!("read {}: {err}", workspace.join(relative_path).display()))
}

pub fn exact_ok_prompt() -> &'static str {
    "Reply with exactly OK."
}

fn tui_models_for_filter(
    default_models: &[String],
    filter_env: &str,
) -> Vec<String> {
    let Some(filter) = std::env::var(filter_env)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return default_models.to_vec();
    };

    let allowed: Vec<String> = filter
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect();

    default_models
        .iter()
        .filter(|model| {
            allowed.iter().any(|entry| {
                entry.eq_ignore_ascii_case(model.as_str())
                    || model.ends_with(&format!("/{entry}"))
                    || model.ends_with(entry)
            })
        })
        .cloned()
        .collect()
}

pub fn tui_selected_models(default_models: &[String]) -> Vec<String> {
    tui_models_for_filter(default_models, "KAY_TUI_PROVIDER_LIVE_SMOKE_MODEL_FILTER")
}

pub fn tui_ux_selected_models(default_models: &[String]) -> Vec<String> {
    tui_models_for_filter(default_models, "KAY_TUI_UX_LIVE_SMOKE_MODEL_FILTER")
}

pub fn shell_tool_prompt() -> &'static str {
    "Use the shell tool to run exactly: echo SHELL_OK. \
     After the command completes successfully, reply with exactly DONE."
}

pub fn apply_patch_prompt(relative_path: &str) -> String {
    format!(
        "The workspace contains `{relative_path}` with exactly `before`. \
         Use apply_patch to replace the entire file content with exactly `PATCH_OK`. \
         Do not use shell redirection or heredocs to edit the file. \
         After the patch succeeds, reply with exactly DONE."
    )
}

pub fn malformed_apply_patch_prompt(relative_path: &str) -> String {
    format!(
        "The workspace contains `{relative_path}` with exactly `before`. \
         Use apply_patch once to replace the entire file content with exactly `RECOVERY_OK`. \
         If your patch hunk header ends with a trailing `@@`, that is acceptable. \
         After the patch succeeds, reply with exactly DONE."
    )
}

pub fn status_contract_prompt() -> &'static str {
    "Run the shell tool with exactly: echo STATUS_PROBE. \
     After the command completes, reply with ONLY this block:\n\
     STATUS: PASS\nFILES_CHANGED: []\nTESTS_RUN: echo STATUS_PROBE"
}
