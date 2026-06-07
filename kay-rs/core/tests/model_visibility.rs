use std::sync::Arc;

use code_app_server_protocol::AuthMode;
use code_core::auth::{login_with_api_key, remove_provider_api_key, save_provider_api_key, AuthManager};
use code_core::model_visibility::{
    visible_model_groups, visible_model_presets, VisibleModelPreset, VisibleProvider,
};
use pretty_assertions::assert_eq;
use serial_test::serial;
use tempfile::TempDir;

const PROVIDER_ENV_KEYS: &[&str] = &[
    "CODEX_API_KEY",
    "OPENAI_API_KEY",
    "XIAOMI_API_KEY",
    "OPENCODE_GO_API_KEY",
    "MINIMAX_API_KEY",
];

struct EnvGuard {
    previous: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn unset(keys: &[&'static str]) -> Self {
        let previous = keys
            .iter()
            .map(|key| {
                let value = std::env::var(key).ok();
                // SAFETY: tests using this guard are marked `#[serial]`, so
                // environment mutations do not race with other tests here.
                unsafe { std::env::remove_var(key) };
                (*key, value)
            })
            .collect();
        Self { previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.previous {
            // SAFETY: see `EnvGuard::unset`.
            unsafe {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}

fn clear_provider_env() -> EnvGuard {
    EnvGuard::unset(PROVIDER_ENV_KEYS)
}

#[derive(Clone)]
struct TestPreset {
    model: &'static str,
    show_in_picker: bool,
    pro_only: bool,
}

impl TestPreset {
    fn new(model: &'static str) -> Self {
        Self {
            model,
            show_in_picker: true,
            pro_only: false,
        }
    }

    fn pro_only(model: &'static str) -> Self {
        Self {
            model,
            show_in_picker: true,
            pro_only: true,
        }
    }
}

impl VisibleModelPreset for TestPreset {
    fn visibility_model(&self) -> &str {
        self.model
    }

    fn visibility_show_in_picker(&self) -> bool {
        self.show_in_picker
    }

    fn visibility_pro_only(&self) -> bool {
        self.pro_only
    }
}

fn load_auth_manager(code_home: &TempDir) -> Arc<AuthManager> {
    AuthManager::shared_with_mode_and_originator(
        code_home.path().to_path_buf(),
        AuthMode::ApiKey,
        "code_cli_rs".to_string(),
    )
}

fn visible_models(presets: &[TestPreset], auth: &AuthManager) -> Vec<&'static str> {
    visible_model_presets(auth, presets)
        .iter()
        .map(|preset| preset.model)
        .collect()
}

#[test]
#[serial]
fn opencode_go_visibility_is_credential_driven_and_whitelist_based() {
    let _env = clear_provider_env();
    let code_home = TempDir::new().unwrap();
    let presets = vec![
        TestPreset::new("opencode-go/kimi-k2.6"),
        TestPreset::new("xopencode-go/kimi-k2.6"),
        TestPreset::new("opencode-go/minimax-m2.7"),
        TestPreset::new("opencode-go/unsupported-model"),
    ];

    let auth = load_auth_manager(&code_home);
    assert!(visible_model_groups(&auth, &presets).is_empty());

    save_provider_api_key(code_home.path(), "opencode-go", "sk-opencode")
        .expect("opencode key should be saved");
    let auth = load_auth_manager(&code_home);
    assert_eq!(
        visible_model_groups(&auth, &presets)
            .iter()
            .map(|group| group.provider)
            .collect::<Vec<_>>(),
        vec![VisibleProvider::OpenCodeGo]
    );
    assert_eq!(
        visible_models(&presets, &auth),
        vec!["opencode-go/kimi-k2.6", "opencode-go/minimax-m2.7"]
    );

    remove_provider_api_key(code_home.path(), "opencode-go")
        .expect("opencode key should be removed");
    let auth = load_auth_manager(&code_home);
    assert!(visible_model_groups(&auth, &presets).is_empty());
}

#[test]
#[serial]
fn minimax_visibility_is_credential_driven_and_exact() {
    let _env = clear_provider_env();
    let code_home = TempDir::new().unwrap();
    let presets = vec![
        TestPreset::new("MiniMax-M3"),
        TestPreset::new("MiniMax-M2.7"),
        TestPreset::new("MiniMax-M3-beta"),
        TestPreset::new("MiniMax-M2.7-beta"),
        TestPreset::new("preMiniMax-M2.7"),
    ];

    let auth = load_auth_manager(&code_home);
    assert!(visible_model_groups(&auth, &presets).is_empty());

    save_provider_api_key(code_home.path(), "minimax", "sk-minimax")
        .expect("minimax key should be saved");
    let auth = load_auth_manager(&code_home);
    assert_eq!(
        visible_model_groups(&auth, &presets)
            .iter()
            .map(|group| group.provider)
            .collect::<Vec<_>>(),
        vec![VisibleProvider::MiniMax]
    );
    assert_eq!(
        visible_models(&presets, &auth),
        vec!["MiniMax-M3", "MiniMax-M2.7"]
    );

    remove_provider_api_key(code_home.path(), "minimax")
        .expect("minimax key should be removed");
    let auth = load_auth_manager(&code_home);
    assert!(visible_model_groups(&auth, &presets).is_empty());
}

#[test]
#[serial]
fn xiaomi_visibility_is_credential_driven_and_whitelist_based() {
    let _env = clear_provider_env();
    let code_home = TempDir::new().unwrap();
    let presets = vec![
        TestPreset::new("xiaomi/mimo-v2.5-pro"),
        TestPreset::new("xiaomi/mimo-v2.5"),
        TestPreset::new("xiaomi/unsupported-model"),
        TestPreset::new("opencode-go/mimo-v2.5"),
    ];

    let auth = load_auth_manager(&code_home);
    assert!(visible_model_groups(&auth, &presets).is_empty());

    save_provider_api_key(code_home.path(), "xiaomi", "sk-xiaomi")
        .expect("xiaomi key should be saved");
    let auth = load_auth_manager(&code_home);
    assert_eq!(
        visible_model_groups(&auth, &presets)
            .iter()
            .map(|group| group.provider)
            .collect::<Vec<_>>(),
        vec![VisibleProvider::Xiaomi]
    );
    assert_eq!(
        visible_models(&presets, &auth),
        vec!["xiaomi/mimo-v2.5-pro", "xiaomi/mimo-v2.5"]
    );

    remove_provider_api_key(code_home.path(), "xiaomi").expect("xiaomi key should be removed");
    let auth = load_auth_manager(&code_home);
    assert!(visible_model_groups(&auth, &presets).is_empty());
}

#[test]
#[serial]
fn openai_remains_visible_through_the_existing_openai_auth_path() {
    let _env = clear_provider_env();
    let code_home = TempDir::new().unwrap();
    let presets = vec![
        TestPreset::new("gpt-5.4"),
        TestPreset::pro_only("gpt-5.3-codex-spark"),
        TestPreset::new("gpt-oss:20b"),
        TestPreset::new("foo/bar"),
    ];

    let auth = load_auth_manager(&code_home);
    assert!(visible_model_groups(&auth, &presets).is_empty());

    login_with_api_key(code_home.path(), "sk-openai").expect("openai key should be saved");
    let auth = load_auth_manager(&code_home);
    assert_eq!(
        visible_model_groups(&auth, &presets)
            .iter()
            .map(|group| group.provider)
            .collect::<Vec<_>>(),
        vec![VisibleProvider::OpenAI]
    );
    assert_eq!(visible_models(&presets, &auth), vec!["gpt-5.4"]);
}

#[test]
#[serial]
fn provider_order_is_locked_xiaomi_then_open_code_go_then_minimax_then_openai() {
    let _env = clear_provider_env();
    let code_home = TempDir::new().unwrap();
    login_with_api_key(code_home.path(), "sk-openai").expect("openai key should be saved");
    save_provider_api_key(code_home.path(), "xiaomi", "sk-xiaomi")
        .expect("xiaomi key should be saved");
    save_provider_api_key(code_home.path(), "opencode-go", "sk-opencode")
        .expect("opencode key should be saved");
    save_provider_api_key(code_home.path(), "minimax", "sk-minimax")
        .expect("minimax key should be saved");

    let auth = load_auth_manager(&code_home);
    let presets = vec![
        TestPreset::new("xiaomi/mimo-v2.5-pro"),
        TestPreset::new("MiniMax-M3"),
        TestPreset::new("opencode-go/kimi-k2.6"),
        TestPreset::new("gpt-5.4"),
    ];

    let groups = visible_model_groups(&auth, &presets);
    assert_eq!(
        groups.iter().map(|group| group.provider).collect::<Vec<_>>(),
        vec![
            VisibleProvider::Xiaomi,
            VisibleProvider::OpenCodeGo,
            VisibleProvider::MiniMax,
            VisibleProvider::OpenAI,
        ]
    );
    assert_eq!(
        visible_model_presets(&auth, &presets)
            .iter()
            .map(|preset| preset.model)
            .collect::<Vec<_>>(),
        vec![
            "xiaomi/mimo-v2.5-pro",
            "opencode-go/kimi-k2.6",
            "MiniMax-M3",
            "gpt-5.4"
        ]
    );
}
