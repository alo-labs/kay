use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use code_core::provider_models::{
    minimax_preset_ids, opencode_go_preset_ids, xiaomi_preset_ids,
};
use serde_json::Value;
use tempfile::TempDir;

mod common;
use common::SessionPreserver;
use common::provider_compat::{
    apply_patch_prompt, assert_wire_profile, compatibility_profile, completed_file_change,
    completed_shell_command, live_smoke_enabled, malformed_apply_patch_prompt, parse_thread_events,
    read_workspace_file, selected_models, shell_tool_prompt, status_contract_prompt,
};

const EXEC_TIMEOUT_SECS: &str = "900";

#[derive(Clone)]
struct ProviderAcceptanceSpec {
    provider_id: &'static str,
    api_key_env: &'static str,
    models: Vec<String>,
}

fn opencode_go_spec() -> ProviderAcceptanceSpec {
    ProviderAcceptanceSpec {
        provider_id: "opencode-go",
        api_key_env: "OPENCODE_GO_LIVE_API_KEY",
        models: opencode_go_preset_ids(),
    }
}

fn minimax_spec() -> ProviderAcceptanceSpec {
    ProviderAcceptanceSpec {
        provider_id: "minimax",
        api_key_env: "MINIMAX_LIVE_API_KEY",
        models: minimax_preset_ids(),
    }
}

fn xiaomi_spec() -> ProviderAcceptanceSpec {
    ProviderAcceptanceSpec {
        provider_id: "xiaomi",
        api_key_env: "XIAOMI_LIVE_API_KEY",
        models: xiaomi_preset_ids(),
    }
}

fn live_key(env_var: &str) -> Option<String> {
    std::env::var(env_var)
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

fn login_provider(kay_home: &TempDir, provider_id: &str, api_key: &str) {
    let mut child = Command::new(code_bin())
        .arg("login")
        .arg("--provider")
        .arg(provider_id)
        .arg("--with-api-key")
        .env("KAY_HOME", kay_home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kay login");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(api_key.as_bytes())
        .expect("write api key");

    let output = child.wait_with_output().expect("wait for kay login");
    assert!(
        output.status.success(),
        "kay login failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

struct ExecRunOutput {
    stdout: String,
    last_message: String,
}

fn base_exec_command(
    kay_home: &TempDir,
    provider_id: &str,
    model: &str,
    workspace: Option<&Path>,
) -> Command {
    let mut command = Command::new(code_bin());
    command
        .arg("exec")
        .arg("--max-seconds")
        .arg(EXEC_TIMEOUT_SECS)
        .arg("--json")
        .arg("--skip-git-repo-check")
        .arg("--full-auto")
        .arg("-c")
        .arg(format!("model_provider={provider_id}"))
        .arg("-c")
        .arg(format!("model={model}"))
        .env("KAY_HOME", kay_home.path())
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null());

    if let Some(workspace) = workspace {
        command
            .arg("--sandbox")
            .arg("workspace-write")
            .arg("--cd")
            .arg(workspace);
    }

    command
}

fn run_exec_prompt(
    kay_home: &TempDir,
    provider_id: &str,
    model: &str,
    prompt: &str,
    last_message_path: &Path,
) -> String {
    let output = base_exec_command(kay_home, provider_id, model, None)
        .arg("--output-last-message")
        .arg(last_message_path)
        .arg(prompt)
        .output()
        .expect("run kay exec");

    assert!(
        output.status.success(),
        "kay exec failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::read_to_string(last_message_path).expect("read last message file")
}

fn run_exec_prompt_with_demo(
    kay_home: &TempDir,
    provider_id: &str,
    model: &str,
    developer: &str,
    prompt: &str,
    last_message_path: &Path,
) -> String {
    let output = Command::new(code_bin())
        .arg("--demo")
        .arg(developer)
        .arg("exec")
        .arg("--max-seconds")
        .arg(EXEC_TIMEOUT_SECS)
        .arg("--json")
        .arg("--skip-git-repo-check")
        .arg("--output-last-message")
        .arg(last_message_path)
        .arg("-c")
        .arg(format!("model_provider={provider_id}"))
        .arg("-c")
        .arg(format!("model={model}"))
        .arg(prompt)
        .env("KAY_HOME", kay_home.path())
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null())
        .output()
        .expect("run kay exec with demo");

    assert!(
        output.status.success(),
        "kay exec with demo failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::read_to_string(last_message_path).expect("read last message file")
}

fn run_exec_prompt_with_output_schema(
    kay_home: &TempDir,
    provider_id: &str,
    model: &str,
    prompt: &str,
    output_schema: &Path,
    last_message_path: &Path,
) -> String {
    let output = base_exec_command(kay_home, provider_id, model, None)
        .arg("--output-schema")
        .arg(output_schema)
        .arg("--output-last-message")
        .arg(last_message_path)
        .arg(prompt)
        .output()
        .expect("run kay exec with output schema");

    assert!(
        output.status.success(),
        "kay exec with output schema failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::read_to_string(last_message_path).expect("read last message file")
}

fn run_exec_workspace_prompt(
    kay_home: &TempDir,
    provider_id: &str,
    model: &str,
    workspace: &Path,
    prompt: &str,
    last_message_path: &Path,
) -> ExecRunOutput {
    let output = base_exec_command(kay_home, provider_id, model, Some(workspace))
        .arg("--output-last-message")
        .arg(last_message_path)
        .arg(prompt)
        .output()
        .expect("run kay exec in workspace");

    assert!(
        output.status.success(),
        "kay exec in workspace failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    ExecRunOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        last_message: std::fs::read_to_string(last_message_path).expect("read last message file"),
    }
}

fn output_has_exact_line(output: &str, expected: &str) -> bool {
    output.lines().any(|line| line.trim() == expected)
}

fn output_has_okish_line(output: &str) -> bool {
    output.lines().any(|line| line.trim_start().starts_with("OK"))
}

fn output_has_status_pass(output: &str) -> bool {
    output.lines().any(|line| {
        line.trim()
            .to_ascii_uppercase()
            .starts_with("STATUS: PASS")
    })
}

fn first_json_object(output: &str) -> Option<Value> {
    let start = output.find('{')?;
    let end = output.rfind('}')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&output[start..=end]).ok()
}

fn assert_basic_acceptance(
    kay_home: &TempDir,
    spec: &ProviderAcceptanceSpec,
    model: &str,
    schema_path: &Path,
) {
    let plain_last_message = tempfile::NamedTempFile::new_in(kay_home.path())
        .expect("create plain last-message file");
    let plain = run_exec_prompt(
        kay_home,
        spec.provider_id,
        model,
        "Reply with exactly OK.",
        plain_last_message.path(),
    );
    assert!(
        output_has_okish_line(&plain),
        "expected OK response for {model}, got:\n{plain}"
    );

    let dev_last_message = tempfile::NamedTempFile::new_in(kay_home.path())
        .expect("create dev last-message file");
    let dev = run_exec_prompt_with_demo(
        kay_home,
        spec.provider_id,
        model,
        "You are a terse assistant. Follow the user's instruction exactly.",
        "Reply with exactly DEV_OK. Do not add punctuation or explanation.",
        dev_last_message.path(),
    );
    assert!(
        output_has_exact_line(&dev, "DEV_OK"),
        "expected exact DEV_OK response for {model}, got:\n{dev}"
    );

    let profile = compatibility_profile(spec.provider_id, model);
    if profile.supports_json_schema_output {
        let json_last_message = tempfile::NamedTempFile::new_in(kay_home.path())
            .expect("create json last-message file");
        let json = run_exec_prompt_with_output_schema(
            kay_home,
            spec.provider_id,
            model,
            &format!(
                "Please return a single JSON object with exactly these fields and values: provider={provider}, model={model}, ok=true. Do not include markdown or extra text.",
                provider = spec.provider_id,
                model = model
            ),
            schema_path,
            json_last_message.path(),
        );
        let parsed = first_json_object(&json)
            .unwrap_or_else(|| panic!("expected JSON object for {model}, got:\n{json}"));
        assert_eq!(parsed["provider"], spec.provider_id);
        assert_eq!(parsed["model"], model);
        assert_eq!(parsed["ok"], true);
    } else {
        eprintln!("skipping structured output for {model}: family does not support json_schema");
    }

    let markdown_last_message = tempfile::NamedTempFile::new_in(kay_home.path())
        .expect("create markdown last-message file");
    let markdown = run_exec_prompt(
        kay_home,
        spec.provider_id,
        model,
        &format!(
            "Return a short markdown note with the exact heading `# Acceptance for {provider}/{model}`. Include bullet lines `- provider: {provider}` and `- model: {model}`, then a fenced code block containing `OK`. Do not add any prose.",
            provider = spec.provider_id,
            model = model
        ),
        markdown_last_message.path(),
    );
    assert!(
        markdown.contains(&format!("# Acceptance for {}/{}", spec.provider_id, model)),
        "expected markdown heading for {model}, got:\n{markdown}"
    );
    assert!(
        markdown.contains(&format!("- provider: {}", spec.provider_id)),
        "expected provider bullet for {model}, got:\n{markdown}"
    );
    assert!(
        markdown.contains(&format!("- model: {}", model)),
        "expected model bullet for {model}, got:\n{markdown}"
    );
    assert!(
        markdown.contains("```"),
        "expected fenced code block for {model}, got:\n{markdown}"
    );
}

fn assert_compatibility_dimensions(
    kay_home: &TempDir,
    spec: &ProviderAcceptanceSpec,
    model: &str,
) {
    let profile = compatibility_profile(spec.provider_id, model);
    assert_wire_profile(spec.provider_id, model, &profile);

    let workspace = TempDir::new().expect("compatibility workspace");
    let patch_target = "patch-target.txt";
    std::fs::write(workspace.path().join(patch_target), "before").expect("seed patch file");

    let shell_last_message = tempfile::NamedTempFile::new_in(kay_home.path())
        .expect("create shell last-message file");
    let shell = run_exec_workspace_prompt(
        kay_home,
        spec.provider_id,
        model,
        workspace.path(),
        shell_tool_prompt(),
        shell_last_message.path(),
    );
    let shell_events = parse_thread_events(&shell.stdout);
    assert!(
        completed_shell_command(&shell_events, "SHELL_OK"),
        "expected completed shell tool run for {model}, events:\n{}",
        shell.stdout
    );
    assert!(
        output_has_exact_line(&shell.last_message, "DONE"),
        "expected DONE after shell tool for {model}, got:\n{}",
        shell.last_message
    );

    if profile.needs_apply_patch {
        let patch_workspace = TempDir::new().expect("apply patch workspace");
        std::fs::write(patch_workspace.path().join(patch_target), "before")
            .expect("seed apply patch file");
        let patch_last_message = tempfile::NamedTempFile::new_in(kay_home.path())
            .expect("create patch last-message file");
        let patch = run_exec_workspace_prompt(
            kay_home,
            spec.provider_id,
            model,
            patch_workspace.path(),
            &apply_patch_prompt(patch_target),
            patch_last_message.path(),
        );
        let patch_events = parse_thread_events(&patch.stdout);
        assert!(
            completed_file_change(&patch_events, patch_target)
                || read_workspace_file(patch_workspace.path(), patch_target).trim() == "PATCH_OK",
            "expected apply_patch to update {patch_target} for {model}, events:\n{}",
            patch.stdout
        );
        assert!(
            output_has_exact_line(&patch.last_message, "DONE"),
            "expected DONE after apply_patch for {model}, got:\n{}",
            patch.last_message
        );
    }

    if profile.repairs_malformed_apply_patch {
        let recovery_workspace = TempDir::new().expect("recovery workspace");
        std::fs::write(recovery_workspace.path().join(patch_target), "before")
            .expect("seed recovery patch file");
        let recovery_last_message = tempfile::NamedTempFile::new_in(kay_home.path())
            .expect("create recovery last-message file");
        let recovery = run_exec_workspace_prompt(
            kay_home,
            spec.provider_id,
            model,
            recovery_workspace.path(),
            &malformed_apply_patch_prompt(patch_target),
            recovery_last_message.path(),
        );
        let recovery_events = parse_thread_events(&recovery.stdout);
        assert!(
            completed_file_change(&recovery_events, patch_target)
                || read_workspace_file(recovery_workspace.path(), patch_target).trim()
                    == "RECOVERY_OK",
            "expected malformed apply_patch recovery for {model}, events:\n{}",
            recovery.stdout
        );
    }

    if profile.uses_local_shell_tool || profile.needs_apply_patch {
        let status_last_message = tempfile::NamedTempFile::new_in(kay_home.path())
            .expect("create status last-message file");
        let status = run_exec_workspace_prompt(
            kay_home,
            spec.provider_id,
            model,
            workspace.path(),
            status_contract_prompt(),
            status_last_message.path(),
        );
        assert!(
            output_has_status_pass(&status.last_message),
            "expected STATUS contract for {model}, got:\n{}",
            status.last_message
        );
    }
}

fn assert_provider_acceptance(spec: &ProviderAcceptanceSpec) {
    let Some(api_key) = live_key(spec.api_key_env) else {
        eprintln!(
            "skipping provider acceptance: {} is not set",
            spec.api_key_env
        );
        return;
    };

    let kay_home = TempDir::new().expect("temp KAY_HOME");
    let _sessions = SessionPreserver::new(
        kay_home.path(),
        format!("provider_model_acceptance_{}", spec.provider_id),
    );
    login_provider(&kay_home, spec.provider_id, &api_key);

    let schema = serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["provider", "model", "ok"],
        "properties": {
            "provider": { "type": "string" },
            "model": { "type": "string" },
            "ok": { "type": "boolean" },
        },
    });
    let schema_path = kay_home.path().join("acceptance-output-schema.json");
    std::fs::write(&schema_path, serde_json::to_string_pretty(&schema).unwrap())
        .expect("write acceptance output schema");

    let models = selected_models(&spec.models);
    assert!(
        !models.is_empty(),
        "model filter excluded every model for provider {}",
        spec.provider_id
    );

    for model in models {
        eprintln!(
            "[provider_model_acceptance] provider={} model={} live_smoke={}",
            spec.provider_id,
            model,
            live_smoke_enabled()
        );

        assert_basic_acceptance(&kay_home, spec, &model, &schema_path);

        if live_smoke_enabled() {
            assert_compatibility_dimensions(&kay_home, spec, &model);
        }
    }
}

#[test]
fn opencode_go_provider_model_acceptance_matrix() {
    assert_provider_acceptance(&opencode_go_spec());
}

#[test]
fn minimax_provider_model_acceptance_matrix() {
    assert_provider_acceptance(&minimax_spec());
}

#[test]
fn xiaomi_provider_model_acceptance_matrix() {
    assert_provider_acceptance(&xiaomi_spec());
}
