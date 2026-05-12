use crate::auth::AuthManager;
use crate::model_family::provider_model_slug;
use crate::MINIMAX_PROVIDER_ID;
use crate::OPENCODE_GO_PROVIDER_ID;

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
    let opencode_go_visible = auth.provider_api_key(OPENCODE_GO_PROVIDER_ID).is_some();
    let minimax_visible = auth.provider_api_key(MINIMAX_PROVIDER_ID).is_some();

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

fn matches_opencode_go_namespace(model: &str) -> bool {
    provider_model_slug(OPENCODE_GO_PROVIDER_ID, model).as_ref() != model
}

fn matches_minimax_model(model: &str) -> bool {
    model.trim().eq_ignore_ascii_case("MiniMax-M2.7")
}

fn matches_openai_model(model: &str) -> bool {
    let slug = provider_model_slug("openai", model);
    let slug = slug.as_ref().trim().to_ascii_lowercase();
    slug.starts_with("gpt-") && !slug.starts_with("gpt-oss")
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
        && matches_opencode_go_namespace(preset.visibility_model())
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
    fn opencode_go_namespace_matching_is_exact() {
        assert!(matches_opencode_go_namespace("opencode-go/kimi-k2.6"));
        assert!(!matches_opencode_go_namespace("xopencode-go/kimi-k2.6"));
        assert!(!matches_opencode_go_namespace("opencode-gox/kimi-k2.6"));
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
    fn provider_keys_control_visibility() {
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
}
