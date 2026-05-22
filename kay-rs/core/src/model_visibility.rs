use crate::auth::AuthManager;
use crate::model_family::provider_model_slug;
use crate::MINIMAX_PROVIDER_ID;
use crate::OPENCODE_GO_PROVIDER_ID;

const OPENCODE_GO_SUPPORTED_MODELS: &[&str] = &[
    "glm-5.1",
    "kimi-k2.6",
    "mimo-v2.5-pro",
    "mimo-v2.5",
    "minimax-m2.7",
    "qwen3.6-plus",
    "deepseek-v4-pro",
    "deepseek-v4-flash",
];

const MINIMAX_SUPPORTED_MODELS: &[&str] = &["MiniMax-M2.7"];

/// Provider buckets are intentionally locked so the picker can render them in
/// a predictable order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VisibleProvider {
    OpenCodeGo,
    MiniMax,
    OpenAI,
}

impl VisibleProvider {
    pub const ORDER: [Self; 3] = [Self::OpenCodeGo, Self::MiniMax, Self::OpenAI];

    pub fn label(self) -> &'static str {
        match self {
            Self::OpenCodeGo => "OpenCode Go",
            Self::MiniMax => "MiniMax",
            Self::OpenAI => "OpenAI",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleProviderModels<P> {
    pub provider: VisibleProvider,
    pub label: &'static str,
    pub presets: Vec<P>,
}

/// Minimal adapter required by the provider-aware visibility helper.
///
/// The helper intentionally only needs the model slug and the existing picker
/// / auth flags so it can stay reusable across core, TUI, and any future API
/// surfaces without knowing the concrete preset type.
pub trait VisibleModelPreset {
    fn visibility_model(&self) -> &str;
    fn visibility_show_in_picker(&self) -> bool;
    fn visibility_pro_only(&self) -> bool;
}

pub fn visible_model_groups<P>(auth: &AuthManager, presets: &[P]) -> Vec<VisibleProviderModels<P>>
where
    P: VisibleModelPreset + Clone,
{
    let auth_snapshot = auth.auth();
    let opencode_go_visible =
        provider_credential_visible(auth, OPENCODE_GO_PROVIDER_ID, "OPENCODE_GO_API_KEY");
    let minimax_visible = provider_credential_visible(auth, MINIMAX_PROVIDER_ID, "MINIMAX_API_KEY");

    VisibleProvider::ORDER
        .into_iter()
        .filter_map(|provider| {
            let provider_presets = match provider {
                VisibleProvider::OpenCodeGo => presets
                    .iter()
                    .filter(|preset| {
                        is_visible_to_opencode_go(
                            auth_snapshot.as_ref(),
                            opencode_go_visible,
                            *preset,
                        )
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
                VisibleProvider::MiniMax => presets
                    .iter()
                    .filter(|preset| {
                        is_visible_to_minimax(auth_snapshot.as_ref(), minimax_visible, *preset)
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
                VisibleProvider::OpenAI => presets
                    .iter()
                    .filter(|preset| is_visible_to_openai(auth_snapshot.as_ref(), *preset))
                    .cloned()
                    .collect::<Vec<_>>(),
            };

            (!provider_presets.is_empty()).then_some(VisibleProviderModels {
                provider,
                label: provider.label(),
                presets: provider_presets,
            })
        })
        .collect()
}

pub fn visible_model_presets<P>(auth: &AuthManager, presets: &[P]) -> Vec<P>
where
    P: VisibleModelPreset + Clone,
{
    visible_model_groups(auth, presets)
        .into_iter()
        .flat_map(|group| group.presets)
        .collect()
}

fn provider_credential_visible(auth: &AuthManager, provider_ref: &str, env_key: &str) -> bool {
    auth.provider_api_key(provider_ref).is_some()
        || std::env::var(env_key)
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
}

fn matches_minimax_model(model: &str) -> bool {
    MINIMAX_SUPPORTED_MODELS
        .iter()
        .any(|supported| model.trim().eq_ignore_ascii_case(supported))
}

fn matches_openai_model(model: &str) -> bool {
    let slug = provider_model_slug("openai", model);
    let slug = slug.as_ref().trim().to_ascii_lowercase();
    slug.starts_with("gpt-") && !slug.starts_with("gpt-oss")
}

fn matches_opencode_go_supported_model(model: &str) -> bool {
    let Some((namespace, _)) = model.trim().split_once('/') else {
        return false;
    };
    if !namespace.eq_ignore_ascii_case(OPENCODE_GO_PROVIDER_ID) {
        return false;
    }
    let slug = provider_model_slug(OPENCODE_GO_PROVIDER_ID, model);
    OPENCODE_GO_SUPPORTED_MODELS
        .iter()
        .any(|supported| slug.as_ref().trim().eq_ignore_ascii_case(supported))
}

fn is_visible_to_openai<P>(auth: Option<&crate::auth::CodexAuth>, preset: &P) -> bool
where
    P: VisibleModelPreset,
{
    let Some(auth) = auth else {
        return false;
    };
    if !preset.visibility_show_in_picker() {
        return false;
    }
    if preset.visibility_pro_only()
        && !(auth.mode.is_chatgpt() && auth.supports_pro_only_models())
    {
        return false;
    }
    matches_openai_model(preset.visibility_model())
}

fn is_visible_to_opencode_go<P>(
    _auth: Option<&crate::auth::CodexAuth>,
    provider_key_visible: bool,
    preset: &P,
) -> bool
where
    P: VisibleModelPreset,
{
    provider_key_visible
        && preset.visibility_show_in_picker()
        && matches_opencode_go_supported_model(preset.visibility_model())
}

fn is_visible_to_minimax<P>(
    _auth: Option<&crate::auth::CodexAuth>,
    provider_key_visible: bool,
    preset: &P,
) -> bool
where
    P: VisibleModelPreset,
{
    provider_key_visible
        && preset.visibility_show_in_picker()
        && matches_minimax_model(preset.visibility_model())
}

#[cfg(test)]
mod tests {
    use super::*;
    use code_app_server_protocol::AuthMode;
    use crate::auth::AuthManager;
    use crate::auth::CodexAuth;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    #[derive(Clone)]
    struct TestPreset {
        model: &'static str,
        show_in_picker: bool,
        pro_only: bool,
    }

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            unsafe {
                std::env::remove_var(key);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.key, value),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    impl TestPreset {
        fn new(model: &'static str) -> Self {
            Self {
                model,
                show_in_picker: true,
                pro_only: false,
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

    #[test]
    fn opencode_go_model_matching_is_whitelist_based() {
        assert!(matches_opencode_go_supported_model("opencode-go/kimi-k2.6"));
        assert!(!matches_opencode_go_supported_model("xopencode-go/kimi-k2.6"));
        assert!(!matches_opencode_go_supported_model("opencode-gox/kimi-k2.6"));
        assert!(matches_opencode_go_supported_model("opencode-go/minimax-m2.7"));
        assert!(!matches_opencode_go_supported_model("opencode-go/minimax-m2.7-beta"));
    }

    #[test]
    fn minimax_model_matching_is_exact() {
        assert!(matches_minimax_model("MiniMax-M2.7"));
        assert!(matches_minimax_model("minimax-m2.7"));
        assert!(!matches_minimax_model("MiniMax-M2.7-beta"));
        assert!(!matches_minimax_model("preMiniMax-M2.7"));
    }

    #[test]
    fn visible_model_groups_orders_visible_providers() {
        let code_home = TempDir::new().unwrap();
        crate::auth::login_with_api_key(code_home.path(), "sk-openai")
            .expect("openai key should be saved");
        crate::auth::save_provider_api_key(code_home.path(), "opencode-go", "sk-opencode")
            .expect("opencode key should be saved");
        crate::auth::save_provider_api_key(code_home.path(), "minimax", "sk-minimax")
            .expect("minimax key should be saved");

        let auth = AuthManager::shared_with_mode_and_originator(
            code_home.path().to_path_buf(),
            AuthMode::ApiKey,
            "code_cli_rs".to_string(),
        );
        let presets = vec![
            TestPreset::new("opencode-go/kimi-k2.6"),
            TestPreset::new("MiniMax-M2.7"),
            TestPreset::new("gpt-5.4"),
            TestPreset::new("foo/bar"),
        ];

        let groups = visible_model_groups(&auth, &presets);
        assert_eq!(
            groups.iter().map(|group| group.provider).collect::<Vec<_>>(),
            vec![
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
            vec!["opencode-go/kimi-k2.6", "MiniMax-M2.7", "gpt-5.4"]
        );
    }

    #[test]
    fn visible_model_groups_honor_provider_keys_under_chatgpt_auth() {
        let code_home = TempDir::new().unwrap();
        let auth = crate::auth::AuthDotJson {
            provider_credentials: [
                (
                    crate::MINIMAX_PROVIDER_ID.to_string(),
                    crate::auth::ProviderCredentialEntry {
                        api_key: "sk-minimax".to_string(),
                    },
                ),
                (
                    crate::OPENCODE_GO_PROVIDER_ID.to_string(),
                    crate::auth::ProviderCredentialEntry {
                        api_key: "sk-opencode".to_string(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        crate::auth::write_auth_json(&crate::auth::get_auth_file(code_home.path()), &auth)
            .expect("auth json should be written");

        let auth = AuthManager::from_auth(
            CodexAuth::create_dummy_chatgpt_auth_for_testing(),
            code_home.path().to_path_buf(),
            "code_cli_rs".to_string(),
        );
        let presets = vec![
            TestPreset::new("opencode-go/kimi-k2.6"),
            TestPreset::new("MiniMax-M2.7"),
            TestPreset::new("gpt-5.4"),
        ];

        let groups = visible_model_groups(&auth, &presets);
        assert_eq!(
            groups.iter().map(|group| group.provider).collect::<Vec<_>>(),
            vec![
                VisibleProvider::OpenCodeGo,
                VisibleProvider::MiniMax,
                VisibleProvider::OpenAI,
            ]
        );
    }

    #[test]
    fn hidden_model_families_stay_out_of_visible_sets() {
        let auth = AuthManager::from_auth_for_testing(CodexAuth::from_api_key("sk-openai"));
        let presets = vec![
            TestPreset::new("gpt-5.4"),
            TestPreset::new("foo/bar"),
            TestPreset::new("gpt-oss:20b"),
        ];

        let visible = visible_model_presets(&auth, &presets);
        assert_eq!(visible.iter().map(|preset| preset.model).collect::<Vec<_>>(), vec!["gpt-5.4"]);
    }

    #[test]
    #[serial_test::serial]
    fn provider_keys_control_visibility() {
        let _opencode_env = EnvGuard::unset("OPENCODE_GO_API_KEY");
        let _minimax_env = EnvGuard::unset("MINIMAX_API_KEY");
        let code_home = TempDir::new().unwrap();
        let presets = vec![TestPreset::new("opencode-go/kimi-k2.6")];
        let auth = AuthManager::shared_with_mode_and_originator(
            code_home.path().to_path_buf(),
            AuthMode::ApiKey,
            "code_cli_rs".to_string(),
        );
        assert!(visible_model_groups(&auth, &presets).is_empty());

        crate::auth::save_provider_api_key(code_home.path(), "opencode-go", "sk-opencode")
            .expect("opencode key should be saved");
        let auth = AuthManager::shared_with_mode_and_originator(
            code_home.path().to_path_buf(),
            AuthMode::ApiKey,
            "code_cli_rs".to_string(),
        );
        let groups = visible_model_groups(&auth, &presets);
        assert_eq!(
            groups.iter().map(|group| group.provider).collect::<Vec<_>>(),
            vec![VisibleProvider::OpenCodeGo]
        );
    }

    #[test]
    #[serial_test::serial]
    fn provider_env_keys_control_visibility_without_stored_credentials() {
        let _opencode_env = EnvGuard::set("OPENCODE_GO_API_KEY", "sk-opencode");
        let _minimax_env = EnvGuard::set("MINIMAX_API_KEY", "sk-minimax");

        let code_home = TempDir::new().unwrap();
        let auth = AuthManager::shared_with_mode_and_originator(
            code_home.path().to_path_buf(),
            AuthMode::ApiKey,
            "code_cli_rs".to_string(),
        );
        let presets = vec![
            TestPreset::new("opencode-go/kimi-k2.6"),
            TestPreset::new("MiniMax-M2.7"),
        ];

        let groups = visible_model_groups(&auth, &presets);
        assert_eq!(
            groups.iter().map(|group| group.provider).collect::<Vec<_>>(),
            vec![VisibleProvider::OpenCodeGo, VisibleProvider::MiniMax]
        );
    }
}
