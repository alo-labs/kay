mod common;

use common::load_default_config_for_test;
use common::wait_for_event;

use code_core::config::{Config, ConfigOverrides, ConfigToml};
use code_core::model_family::{
    derive_default_model_family, infer_model_provider_id, provider_model_slug,
};
use code_core::protocol::{AskForApproval, EventMsg, InputItem, Op, SandboxPolicy};
use code_core::{
    built_in_model_providers, ChatCompletionsFormat, CodexAuth, ConversationManager, WireApi,
};
use serial_test::serial;
use std::time::Duration;
use tempfile::TempDir;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const XIAOMI_PROVIDER_ID: &str = "xiaomi";
const XIAOMI_DEFAULT_BASE_URL: &str = "https://token-plan-sgp.xiaomimimo.com/v1";
const XIAOMI_MODELS: &[&str] = &["mimo-v2.5-pro", "mimo-v2.5"];

#[test]
fn built_in_xiaomi_provider_uses_openai_chat_completions_and_provider_credentials() {
    let providers = built_in_model_providers(None);
    let xiaomi = providers
        .get(XIAOMI_PROVIDER_ID)
        .expect("xiaomi provider should exist");

    assert_eq!(xiaomi.name, "Xiaomi");
    assert_eq!(xiaomi.base_url.as_deref(), Some(XIAOMI_DEFAULT_BASE_URL));
    assert_eq!(xiaomi.env_key.as_deref(), Some("XIAOMI_API_KEY"));
    assert_eq!(xiaomi.credential_ref.as_deref(), Some(XIAOMI_PROVIDER_ID));
    assert_eq!(xiaomi.wire_api, WireApi::Chat);
    assert_eq!(xiaomi.chat_completions_format, ChatCompletionsFormat::OpenAi);
    assert_eq!(xiaomi.stream_idle_timeout_ms, Some(300_000));
    assert_eq!(xiaomi.stream_idle_timeout(), Duration::from_millis(300_000));
    assert!(!xiaomi.requires_openai_auth);
}

#[test]
fn xiaomi_model_slugs_strip_to_provider_local_names() {
    for model in XIAOMI_MODELS {
        let slug = format!("xiaomi/{model}");
        assert_eq!(
            provider_model_slug(XIAOMI_PROVIDER_ID, &slug).as_ref(),
            *model
        );
        assert_eq!(
            infer_model_provider_id(&slug),
            Some(XIAOMI_PROVIDER_ID),
            "{slug} should infer the Xiaomi provider"
        );
    }
}

#[test]
fn xiaomi_models_can_be_selected_without_custom_config() -> std::io::Result<()> {
    let cwd = TempDir::new().unwrap();
    std::fs::write(cwd.path().join(".git"), "gitdir: nowhere")?;
    let code_home = TempDir::new().unwrap();

    for model in XIAOMI_MODELS {
        let cfg = ConfigToml {
            model: Some(format!("xiaomi/{model}")),
            model_provider: Some(XIAOMI_PROVIDER_ID.to_string()),
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

        assert_eq!(config.model, format!("xiaomi/{model}"));
        assert_eq!(config.model_provider_id, XIAOMI_PROVIDER_ID);
        assert_eq!(config.model_provider.wire_api, WireApi::Chat);
        assert!(!config.model_provider.requires_openai_auth);
        assert_eq!(provider_model_slug(XIAOMI_PROVIDER_ID, &config.model), *model);
    }

    Ok(())
}

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn xiaomi_namespaced_model_slug_is_stripped_in_chat_completions_request() {
    let _env_guard = EnvVarGuard::set("XIAOMI_API_KEY", "sk-xiaomi-test");

    let server = MockServer::start().await;
    let response_body = concat!(
        "data: {\"id\":\"cmpl-1\",\"model\":\"xiaomi/mimo-v2.5-pro-20260422\",\"choices\":[{\"delta\":{\"content\":\"OK\"},\"finish_reason\":\"stop\"}]}\n\n",
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
    config.model = "xiaomi/mimo-v2.5-pro".to_string();
    config.model_family = derive_default_model_family(&config.model);

    let mut provider = built_in_model_providers(None)[XIAOMI_PROVIDER_ID].clone();
    provider.base_url = Some(format!("{}/v1", server.uri()));
    config.model_provider = provider;
    config.model_provider_id = XIAOMI_PROVIDER_ID.to_string();

    let conversation_manager = ConversationManager::with_auth(CodexAuth::from_api_key("Test API Key"));
    let codex = conversation_manager
        .new_conversation(config)
        .await
        .expect("create conversation")
        .conversation;

    codex
        .submit(Op::UserInput {
            items: vec![InputItem::Text {
                text: "hello xiaomi".into(),
            }],
            final_output_json_schema: None,
        })
        .await
        .unwrap();

    wait_for_event(&codex, |ev| matches!(ev, EventMsg::TaskComplete(_))).await;

    let requests = server.received_requests().await.expect("requests");
    let chat_requests = requests
        .iter()
        .filter(|request| {
            request.method.as_str() == "POST" && request.url.path().ends_with("/chat/completions")
        })
        .collect::<Vec<_>>();
    assert_eq!(chat_requests.len(), 1);
    let body: serde_json::Value = chat_requests[0].body_json().expect("json body");
    assert_eq!(body["model"], "mimo-v2.5-pro");
    assert!(
        body["messages"]
            .as_array()
            .expect("messages array")
            .iter()
            .any(|message| message["content"] == "hello xiaomi"),
        "expected the user prompt to be present in the chat payload"
    );
}

#[test]
fn xiaomi_model_namespace_infers_provider_from_config_model() -> std::io::Result<()> {
    let code_home = TempDir::new().unwrap();
    let mut cfg = Config::load_from_base_config_with_overrides(
        ConfigToml::default(),
        ConfigOverrides {
            cwd: Some(code_home.path().to_path_buf()),
            ..Default::default()
        },
        code_home.path().to_path_buf(),
    )?;

    assert!(cfg.sync_model_settings_for_model("xiaomi/mimo-v2.5"));
    assert_eq!(cfg.model_provider_id, XIAOMI_PROVIDER_ID);
    assert_eq!(cfg.model_provider.wire_api, WireApi::Chat);
    assert_eq!(cfg.model_family.slug, "xiaomi/mimo-v2.5");

    Ok(())
}
