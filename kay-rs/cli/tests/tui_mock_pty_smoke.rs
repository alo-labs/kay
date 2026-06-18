#![cfg(test)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use code_core::auth::AuthDotJson;
use code_login::AuthMode;
use serde_json::json;
use tempfile::TempDir;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[path = "common/tui_live_harness.rs"]
mod tui_live_harness;

use tui_live_harness::{
    code_bin, default_live_config_toml, wait_for_exact_response, DEFAULT_PROMPT_TIMEOUT,
    TuiLiveHarness, TuiLiveSpawnOptions,
};

fn sse_response(body: String) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .insert_header("content-type", "text/event-stream")
        .set_body_string(body)
}

fn login_opencode_go(kay_home: &TempDir, api_key: &str) {
    let mut child = Command::new(code_bin())
        .arg("login")
        .arg("--provider")
        .arg("opencode-go")
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

fn write_openai_auth(kay_home: &TempDir) {
    let auth = AuthDotJson {
        auth_mode: Some(AuthMode::ApiKey),
        openai_api_key: Some("mock-openai-key".to_string()),
        ..AuthDotJson::default()
    };
    std::fs::write(
        kay_home.path().join("auth.json"),
        serde_json::to_string_pretty(&auth).expect("serialize auth.json"),
    )
    .expect("write auth.json");
}

fn openai_mock_config_toml(base_url: &str) -> String {
    format!(
        "model_provider = \"openai\"\n\
model = \"gpt-5.1-codex\"\n\
openai_base_url = \"{base_url}\"\n\
approval_policy = \"never\"\n\
sandbox_mode = \"danger-full-access\"\n\
\n\
[notice]\n\
hide_gpt5_1_migration_prompt = true\n\
hide_gpt-5.1-codex-max_migration_prompt = true\n\
hide_gpt5_2_migration_prompt = true\n\
hide_gpt5_2_codex_migration_prompt = true\n\
\n\
[tools]\n\
browser = false\n\
view_image = false\n\
\n\
[subagents]\n\
enabled = false\n"
    )
}

#[test]
fn mock_pty_shows_composer_and_model_selector_without_live_api() {
    let kay_home = TempDir::new().expect("temp KAY_HOME");
    let workspace = TempDir::new().expect("temp workspace");
    login_opencode_go(&kay_home, "mock-opencode-go-key");

    let mut harness = TuiLiveHarness::spawn(&TuiLiveSpawnOptions {
        kay_home: kay_home.path().to_path_buf(),
        cwd: workspace.path().to_path_buf(),
        extra_env: Default::default(),
        session_log_path: None,
        config_toml: Some(default_live_config_toml(
            "opencode-go",
            "opencode-go/glm-5.2",
        )),
    });

    harness.wait_for(
        &["Model:", "What can I code"],
        DEFAULT_PROMPT_TIMEOUT,
    );
    harness.write_composer_line("/model opencode-go/deepseek-v4-flash");
    let screen = harness.wait_for(
        &["Model: opencode-go/deepseek-v4-flash"],
        DEFAULT_PROMPT_TIMEOUT,
    );
    assert!(
        screen.contains("opencode-go/deepseek-v4-flash"),
        "expected /model slash command to update the session model, got:\n{screen}"
    );
}

#[test]
fn mock_pty_streams_mock_openai_response_without_live_api() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .expect("tokio runtime");

    let server = runtime.block_on(async { MockServer::start().await });
    let message_item = json!({
        "type": "response.output_item.done",
        "item": {
            "type": "message",
            "id": "msg-1",
            "role": "assistant",
            "content": [{"type": "output_text", "text": "OK"}],
        }
    });
    let completed = json!({
        "type": "response.completed",
        "response": {
            "id": "resp-1",
            "usage": {
                "input_tokens": 0,
                "input_tokens_details": null,
                "output_tokens": 0,
                "output_tokens_details": null,
                "total_tokens": 0
            }
        }
    });
    let body = format!(
        "event: response.output_item.done\ndata: {message_item}\n\n\
event: response.completed\ndata: {completed}\n\n",
    );
    runtime.block_on(async {
        Mock::given(method("POST"))
            .and(path_regex(".*/responses$"))
            .respond_with(sse_response(body))
            .up_to_n_times(1)
            .mount(&server)
            .await;
    });

    let kay_home = TempDir::new().expect("temp KAY_HOME");
    let workspace = TempDir::new().expect("temp workspace");
    write_openai_auth(&kay_home);

    let base_url = format!("{}/v1", server.uri());
    let mut harness = TuiLiveHarness::spawn(&TuiLiveSpawnOptions {
        kay_home: kay_home.path().to_path_buf(),
        cwd: workspace.path().to_path_buf(),
        extra_env: HashMap::from([("OPENAI_API_KEY".to_string(), "mock-openai-key".to_string())]),
        session_log_path: None,
        config_toml: Some(openai_mock_config_toml(&base_url)),
    });

    harness.wait_for(
        &["Model:", "What can I code"],
        DEFAULT_PROMPT_TIMEOUT,
    );
    harness.write_composer_line("Reply with exactly OK.");
    wait_for_exact_response(
        &mut harness,
        "gpt-5.1-codex",
        "OK",
        Duration::from_secs(60),
    );

    let server = Arc::new(server);
    let server_for_join = Arc::clone(&server);
    let join = thread::spawn(move || {
        runtime.block_on(async move {
            let requests = server_for_join.received_requests().await.expect("requests");
            assert!(
                !requests.is_empty(),
                "expected Kay to call the mock OpenAI responses endpoint"
            );
        });
    });
    join.join().expect("mock request assertion thread");
}
