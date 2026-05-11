use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use serde_json::Value;
use tempfile::TempDir;

#[derive(Clone, Copy)]
struct ProviderAcceptanceSpec {
    provider_id: &'static str,
    api_key_env: &'static str,
    models: &'static [&'static str],
}

const OPENCODE_GO_MODELS: &[&str] = &[
    "opencode-go/glm-5.1",
    "opencode-go/kimi-k2.6",
    "opencode-go/mimo-v2.5-pro",
    "opencode-go/mimo-v2.5",
    "opencode-go/minimax-m2.7",
    "opencode-go/qwen3.6-plus",
    "opencode-go/deepseek-v4-pro",
    "opencode-go/deepseek-v4-flash",
];

const MINIMAX_MODELS: &[&str] = &["MiniMax-M2.7"];

const OPENCODE_GO_SPEC: ProviderAcceptanceSpec = ProviderAcceptanceSpec {
    provider_id: "opencode-go",
    api_key_env: "OPENCODE_GO_LIVE_API_KEY",
    models: OPENCODE_GO_MODELS,
};

const MINIMAX_SPEC: ProviderAcceptanceSpec = ProviderAcceptanceSpec {
    provider_id: "minimax",
    api_key_env: "MINIMAX_LIVE_API_KEY",
    models: MINIMAX_MODELS,
};

fn live_key(env_var: &str) -> Option<String> {
    std::env::var(env_var)
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

fn login_provider(code_home: &TempDir, provider_id: &str, api_key: &str) {
    let mut child = Command::new(code_bin())
        .arg("login")
        .arg("--provider")
        .arg(provider_id)
        .arg("--with-api-key")
        .env("CODE_HOME", code_home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn code login");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(api_key.as_bytes())
        .expect("write api key");

    let output = child.wait_with_output().expect("wait for code login");
    assert!(
        output.status.success(),
        "code login failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_exec_prompt(
    code_home: &TempDir,
    provider_id: &str,
    model: &str,
    prompt: &str,
    last_message_path: &std::path::Path,
) -> String {
    let output = Command::new(code_bin())
        .arg("exec")
        .arg("--json")
        .arg("--skip-git-repo-check")
        .arg("--output-last-message")
        .arg(last_message_path)
        .arg("-c")
        .arg(format!("model_provider={provider_id}"))
        .arg("-c")
        .arg(format!("model={model}"))
        .arg(prompt)
        .env("CODE_HOME", code_home.path())
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null())
        .output()
        .expect("run code exec");

    assert!(
        output.status.success(),
        "code exec failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::read_to_string(last_message_path).expect("read last message file")
}

fn run_exec_prompt_with_demo(
    code_home: &TempDir,
    provider_id: &str,
    model: &str,
    developer: &str,
    prompt: &str,
    last_message_path: &std::path::Path,
) -> String {
    let output = Command::new(code_bin())
        .arg("--demo")
        .arg(developer)
        .arg("exec")
        .arg("--json")
        .arg("--skip-git-repo-check")
        .arg("--output-last-message")
        .arg(last_message_path)
        .arg("-c")
        .arg(format!("model_provider={provider_id}"))
        .arg("-c")
        .arg(format!("model={model}"))
        .arg(prompt)
        .env("CODE_HOME", code_home.path())
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null())
        .output()
        .expect("run code exec with demo");

    assert!(
        output.status.success(),
        "code exec with demo failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::read_to_string(last_message_path).expect("read last message file")
}

fn run_exec_prompt_with_output_schema(
    code_home: &TempDir,
    provider_id: &str,
    model: &str,
    prompt: &str,
    output_schema: &std::path::Path,
    last_message_path: &std::path::Path,
) -> String {
    let output = Command::new(code_bin())
        .arg("exec")
        .arg("--json")
        .arg("--skip-git-repo-check")
        .arg("--output-schema")
        .arg(output_schema)
        .arg("--output-last-message")
        .arg(last_message_path)
        .arg("-c")
        .arg(format!("model_provider={provider_id}"))
        .arg("-c")
        .arg(format!("model={model}"))
        .arg(prompt)
        .env("CODE_HOME", code_home.path())
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null())
        .output()
        .expect("run code exec with output schema");

    assert!(
        output.status.success(),
        "code exec with output schema failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::read_to_string(last_message_path).expect("read last message file")
}

fn output_has_exact_line(output: &str, expected: &str) -> bool {
    output.lines().any(|line| line.trim() == expected)
}

fn output_has_okish_line(output: &str) -> bool {
    output.lines().any(|line| line.trim_start().starts_with("OK"))
}

fn first_json_object(output: &str) -> Option<Value> {
    let start = output.find('{')?;
    let end = output.rfind('}')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&output[start..=end]).ok()
}

fn assert_provider_acceptance(spec: &ProviderAcceptanceSpec) {
    let Some(api_key) = live_key(spec.api_key_env) else {
        eprintln!(
            "skipping provider acceptance: {} is not set",
            spec.api_key_env
        );
        return;
    };

    let code_home = TempDir::new().expect("temp CODE_HOME");
    login_provider(&code_home, spec.provider_id, &api_key);

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
    let schema_path = code_home.path().join("acceptance-output-schema.json");
    std::fs::write(&schema_path, serde_json::to_string_pretty(&schema).unwrap())
        .expect("write acceptance output schema");

    for &model in spec.models {
        let plain_last_message = tempfile::NamedTempFile::new_in(code_home.path())
            .expect("create plain last-message file");
        let plain = run_exec_prompt(
            &code_home,
            spec.provider_id,
            model,
            "Reply with exactly OK.",
            plain_last_message.path(),
        );
        assert!(
            output_has_okish_line(&plain),
            "expected OK response for {model}, got:\n{plain}"
        );

        let dev_last_message = tempfile::NamedTempFile::new_in(code_home.path())
            .expect("create dev last-message file");
        let dev = run_exec_prompt_with_demo(
            &code_home,
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

        let json_last_message = tempfile::NamedTempFile::new_in(code_home.path())
            .expect("create json last-message file");
        let json = run_exec_prompt_with_output_schema(
            &code_home,
            spec.provider_id,
            model,
            &format!(
                "Return only the requested object with provider={provider}, model={model}, ok=true.",
                provider = spec.provider_id,
                model = model
            ),
            &schema_path,
            json_last_message.path(),
        );
        let parsed = first_json_object(&json)
            .unwrap_or_else(|| panic!("expected JSON object for {model}, got:\n{json}"));
        assert_eq!(parsed["provider"], spec.provider_id);
        assert_eq!(parsed["model"], model);
        assert_eq!(parsed["ok"], true);

        let markdown_last_message = tempfile::NamedTempFile::new_in(code_home.path())
            .expect("create markdown last-message file");
        let markdown = run_exec_prompt(
            &code_home,
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
}

#[test]
fn opencode_go_provider_model_acceptance_matrix() {
    assert_provider_acceptance(&OPENCODE_GO_SPEC);
}

#[test]
fn minimax_provider_model_acceptance_matrix() {
    assert_provider_acceptance(&MINIMAX_SPEC);
}
