mod common;

use common::load_default_config_for_test;
use common::wait_for_event;

use code_core::config::{Config, ConfigOverrides, ConfigToml};
use code_core::model_family::{derive_default_model_family, find_family_for_model};
use code_core::protocol::{AskForApproval, EventMsg, InputItem, Op, SandboxPolicy};
use code_core::{
    built_in_model_providers, CodexAuth, ConversationManager, WireApi,
    OPENCODE_GO_DEFAULT_BASE_URL, OPENCODE_GO_PROVIDER_ID,
};
use serial_test::serial;
use tempfile::TempDir;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        // SAFETY: the test that uses this guard is marked `#[serial]`, so
        // environment mutations are not racing with other tests in this process.
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => {
                // SAFETY: see `set` above.
                unsafe { std::env::set_var(self.key, value) };
            }
            None => {
                // SAFETY: see `set` above.
                unsafe { std::env::remove_var(self.key) };
            }
        }
    }
}

fn sse_response(body: String) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .insert_header("content-type", "text/event-stream")
        .set_body_string(body)
}

#[test]
fn built_in_opencode_go_provider_uses_chat_completions_and_provider_credentials() {
    let providers = built_in_model_providers(None);
    let opencode_go = providers
        .get(OPENCODE_GO_PROVIDER_ID)
        .expect("opencode-go provider should exist");

    assert_eq!(opencode_go.name, "OpenCode Go");
    assert_eq!(
        opencode_go.base_url.as_deref(),
        Some(OPENCODE_GO_DEFAULT_BASE_URL)
    );
    assert_eq!(opencode_go.env_key.as_deref(), Some("OPENCODE_GO_API_KEY"));
    assert_eq!(
        opencode_go.credential_ref.as_deref(),
        Some(OPENCODE_GO_PROVIDER_ID)
    );
    assert_eq!(opencode_go.wire_api, WireApi::Chat);
    assert!(!opencode_go.requires_openai_auth);
}

#[test]
fn namespaced_model_with_hyphenated_provider_id_resolves() {
    let family = find_family_for_model("opencode-go/gpt-5.1")
        .expect("hyphenated provider namespace should resolve");

    assert_eq!(family.slug, "opencode-go/gpt-5.1");
    assert_eq!(family.family, "gpt-5.1");
}

#[test]
fn opencode_go_builtin_provider_can_be_selected_without_custom_config() -> std::io::Result<()> {
    let cwd = TempDir::new().unwrap();
    std::fs::write(cwd.path().join(".git"), "gitdir: nowhere")?;
    let code_home = TempDir::new().unwrap();
    let cfg = ConfigToml {
        model: Some("opencode-go/kimi-k2.6".to_string()),
        model_provider: Some(OPENCODE_GO_PROVIDER_ID.to_string()),
        ..Default::default()
    };

    let config = Config::load_from_base_config_with_overrides(
        cfg,
        ConfigOverrides {
            cwd: Some(cwd.path().to_path_buf()),
            ..Default::default()
        },
        code_home.path().to_path_buf(),
    )?;

    assert_eq!(config.model, "opencode-go/kimi-k2.6");
    assert_eq!(config.model_provider_id, OPENCODE_GO_PROVIDER_ID);
    assert_eq!(config.model_provider.wire_api, WireApi::Chat);
    assert!(!config.model_provider.requires_openai_auth);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn opencode_go_namespaced_model_slug_is_stripped_in_chat_completions_request() {
    let _env_guard = EnvVarGuard::set("OPENCODE_GO_API_KEY", "sk-opencode-go-test");

    let server = MockServer::start().await;
    let response_body = concat!(
        "data: {\"id\":\"cmpl-1\",\"model\":\"kimi-k2.6\",\"choices\":[{\"delta\":{\"content\":\"OK\"},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n",
    )
    .to_string();

    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(sse_response(response_body))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let code_home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let mut config = load_default_config_for_test(&code_home);
    config.cwd = cwd.path().to_path_buf();
    config.approval_policy = AskForApproval::Never;
    config.sandbox_policy = SandboxPolicy::DangerFullAccess;
    config.model = "opencode-go/kimi-k2.6".to_string();
    config.model_family = derive_default_model_family(&config.model);

    let mut provider = built_in_model_providers(None)[OPENCODE_GO_PROVIDER_ID].clone();
    provider.base_url = Some(format!("{}/v1", server.uri()));
    config.model_provider = provider;
    config.model_provider_id = OPENCODE_GO_PROVIDER_ID.to_string();

    let conversation_manager = ConversationManager::with_auth(CodexAuth::from_api_key("Test API Key"));
    let codex = conversation_manager
        .new_conversation(config)
        .await
        .expect("create conversation")
        .conversation;

    codex
        .submit(Op::UserInput {
            items: vec![InputItem::Text {
                text: "hello opencode".into(),
            }],
            final_output_json_schema: None,
        })
        .await
        .unwrap();

    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    let requests = server.received_requests().await.expect("requests");
    assert_eq!(requests.len(), 1);
    let body: serde_json::Value = requests[0].body_json().expect("json body");
    assert_eq!(body["model"], "kimi-k2.6");
    assert!(
        body["messages"]
            .as_array()
            .expect("messages array")
            .iter()
            .any(|message| message["content"] == "hello opencode"),
        "expected the user prompt to be present in the chat payload"
    );
}
