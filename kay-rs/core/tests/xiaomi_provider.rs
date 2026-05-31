use code_core::config::{Config, ConfigOverrides, ConfigToml};
use code_core::model_family::{infer_model_provider_id, provider_model_slug};
use code_core::{built_in_model_providers, WireApi};
use tempfile::TempDir;

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
