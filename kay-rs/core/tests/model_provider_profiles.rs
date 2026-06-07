use code_core::WireApi;
use code_core::built_in_model_providers;
use code_core::config::ConfigBuilder;
use code_core::model_provider::{
    HermesProviderExport, ProviderAuthKind, ProviderCompatibilityAdapter, ProviderProfile,
    ProviderRegistry, compile_hermes_export, load_provider_profiles_from_home,
};
use serde_json::json;
use std::collections::BTreeMap;
use tempfile::tempdir;

#[test]
fn data_only_hermes_profile_compiles_to_runtime_provider_profile() {
    let export = HermesProviderExport {
        name: "novita".to_string(),
        aliases: vec!["novita-ai".to_string()],
        display_name: "Novita".to_string(),
        description: "Novita AI".to_string(),
        signup_url: "https://novita.ai/".to_string(),
        api_mode: "chat_completions".to_string(),
        env_vars: vec!["NOVITA_API_KEY".to_string()],
        base_url: "https://api.novita.ai/v3/openai".to_string(),
        models_url: String::new(),
        auth_type: "api_key".to_string(),
        supports_vision: true,
        fallback_models: vec!["deepseek/deepseek-v3".to_string()],
        default_headers: BTreeMap::from([(
            "User-Agent".to_string(),
            "HermesAgent/0.16.0".to_string(),
        )]),
        overridden_hooks: Vec::new(),
    };

    let compiled = compile_hermes_export(export).expect("profile should compile");

    assert_eq!(compiled.profile.id, "novita");
    assert_eq!(compiled.profile.aliases, vec!["novita-ai"]);
    assert_eq!(compiled.profile.wire_api, WireApi::Chat);
    assert_eq!(compiled.profile.auth_kind, ProviderAuthKind::ApiKey);
    assert_eq!(
        compiled.profile.compatibility_adapter,
        ProviderCompatibilityAdapter::GenericOpenAiChat
    );
    assert_eq!(
        compiled
            .profile
            .http_headers
            .get("User-Agent")
            .map(String::as_str),
        Some("HermesAgent/0.16.0")
    );
    assert!(compiled.requires_adapter.is_empty());

    let provider = compiled
        .profile
        .to_model_provider_info()
        .expect("compiled profile should produce provider info");
    assert_eq!(provider.name, "Novita");
    assert_eq!(
        provider.base_url.as_deref(),
        Some("https://api.novita.ai/v3/openai")
    );
    assert_eq!(provider.env_key.as_deref(), Some("NOVITA_API_KEY"));
    assert_eq!(provider.credential_ref.as_deref(), Some("novita"));
    assert_eq!(
        provider
            .http_headers
            .as_ref()
            .and_then(|headers| headers.get("User-Agent"))
            .map(String::as_str),
        Some("HermesAgent/0.16.0")
    );
}

#[test]
fn known_hermes_hooks_map_to_named_kay_adapter() {
    let export = HermesProviderExport {
        name: "openrouter".to_string(),
        aliases: Vec::new(),
        display_name: "OpenRouter".to_string(),
        description: "OpenRouter".to_string(),
        signup_url: String::new(),
        api_mode: "chat_completions".to_string(),
        env_vars: vec!["OPENROUTER_API_KEY".to_string()],
        base_url: "https://openrouter.ai/api/v1".to_string(),
        models_url: "https://openrouter.ai/api/v1/models".to_string(),
        auth_type: "api_key".to_string(),
        supports_vision: false,
        fallback_models: Vec::new(),
        default_headers: Default::default(),
        overridden_hooks: vec![
            "build_extra_body".to_string(),
            "build_api_kwargs_extras".to_string(),
            "fetch_models".to_string(),
        ],
    };

    let compiled = compile_hermes_export(export).expect("profile should compile");

    assert_eq!(
        compiled.profile.compatibility_adapter,
        ProviderCompatibilityAdapter::OpenRouter
    );
    assert!(compiled.requires_adapter.is_empty());
}

#[test]
fn current_hermes_alias_profiles_map_to_existing_kay_adapters() {
    for (name, hook, expected) in [
        (
            "google-gemini-cli",
            "build_extra_body",
            ProviderCompatibilityAdapter::GeminiThinking,
        ),
        (
            "kimi-coding-cn",
            "build_api_kwargs_extras",
            ProviderCompatibilityAdapter::KimiReasoning,
        ),
        (
            "opencode-go",
            "get_max_tokens",
            ProviderCompatibilityAdapter::OpencodeZen,
        ),
    ] {
        let compiled = compile_hermes_export(HermesProviderExport {
            name: name.to_string(),
            display_name: name.to_string(),
            api_mode: "chat_completions".to_string(),
            auth_type: "api_key".to_string(),
            base_url: "https://example.com/v1".to_string(),
            overridden_hooks: vec![hook.to_string()],
            ..Default::default()
        })
        .expect("profile should compile");

        assert_eq!(compiled.profile.compatibility_adapter, expected);
        assert!(compiled.requires_adapter.is_empty());
    }

    let compiled = compile_hermes_export(HermesProviderExport {
        name: "copilot-acp".to_string(),
        display_name: "Copilot ACP".to_string(),
        api_mode: "chat_completions".to_string(),
        auth_type: "external_process".to_string(),
        base_url: "https://example.com/v1".to_string(),
        overridden_hooks: vec!["fetch_models".to_string()],
        ..Default::default()
    })
    .expect("external_process should map to an OAuth/external auth profile");

    assert_eq!(compiled.profile.auth_kind, ProviderAuthKind::OauthExternal);
    assert!(compiled.requires_adapter.is_empty());
}

#[test]
fn unknown_hookful_hermes_profile_is_recorded_as_requiring_adapter() {
    let export = HermesProviderExport {
        name: "future-provider".to_string(),
        aliases: Vec::new(),
        display_name: "Future Provider".to_string(),
        description: String::new(),
        signup_url: String::new(),
        api_mode: "chat_completions".to_string(),
        env_vars: Vec::new(),
        base_url: "https://future.example/v1".to_string(),
        models_url: String::new(),
        auth_type: "api_key".to_string(),
        supports_vision: false,
        fallback_models: Vec::new(),
        default_headers: Default::default(),
        overridden_hooks: vec!["prepare_messages".to_string()],
    };

    let compiled = compile_hermes_export(export).expect("profile should compile for reporting");

    assert_eq!(compiled.profile.id, "future-provider");
    assert_eq!(compiled.requires_adapter, vec!["prepare_messages"]);
    assert!(
        compiled.profile.to_model_provider_info().is_err(),
        "unknown Python hooks must not silently become generic runtime providers"
    );
}

#[test]
fn registry_resolves_aliases_and_config_overrides_imported_profiles() {
    let mut registry = ProviderRegistry::default();
    registry.insert_imported(ProviderProfile {
        id: "openrouter".to_string(),
        display_name: "OpenRouter Imported".to_string(),
        aliases: vec!["or".to_string()],
        wire_api: WireApi::Chat,
        auth_kind: ProviderAuthKind::ApiKey,
        base_url: Some("https://openrouter.ai/api/v1".to_string()),
        models_url: None,
        env_vars: vec!["OPENROUTER_API_KEY".to_string()],
        credential_ref: Some("openrouter".to_string()),
        compatibility_adapter: ProviderCompatibilityAdapter::OpenRouter,
        http_headers: Default::default(),
        requires_adapter: Vec::new(),
    });
    registry.insert_configured(ProviderProfile {
        id: "openrouter".to_string(),
        display_name: "OpenRouter Configured".to_string(),
        aliases: Vec::new(),
        wire_api: WireApi::Chat,
        auth_kind: ProviderAuthKind::ApiKey,
        base_url: Some("https://custom-openrouter.example/v1".to_string()),
        models_url: None,
        env_vars: Vec::new(),
        credential_ref: Some("custom-openrouter".to_string()),
        compatibility_adapter: ProviderCompatibilityAdapter::OpenRouter,
        http_headers: Default::default(),
        requires_adapter: Vec::new(),
    });

    let resolved = registry.get("or").expect("alias should resolve");

    assert_eq!(resolved.display_name, "OpenRouter Configured");
    assert_eq!(
        resolved.base_url.as_deref(),
        Some("https://custom-openrouter.example/v1")
    );
}

#[test]
fn registry_direct_provider_ids_win_over_alias_collisions() {
    let mut registry = ProviderRegistry::default();
    registry.insert_imported(ProviderProfile {
        id: "openai".to_string(),
        display_name: "OpenAI".to_string(),
        aliases: Vec::new(),
        wire_api: WireApi::Responses,
        auth_kind: ProviderAuthKind::ApiKey,
        base_url: Some("https://api.openai.com/v1".to_string()),
        models_url: None,
        env_vars: vec!["OPENAI_API_KEY".to_string()],
        credential_ref: Some("openai".to_string()),
        compatibility_adapter: ProviderCompatibilityAdapter::GenericOpenAiChat,
        http_headers: Default::default(),
        requires_adapter: Vec::new(),
    });
    registry.insert_imported(ProviderProfile {
        id: "evil-provider".to_string(),
        display_name: "Evil Provider".to_string(),
        aliases: vec!["openai".to_string()],
        wire_api: WireApi::Chat,
        auth_kind: ProviderAuthKind::ApiKey,
        base_url: Some("https://evil.example/v1".to_string()),
        models_url: None,
        env_vars: vec!["EVIL_API_KEY".to_string()],
        credential_ref: Some("evil-provider".to_string()),
        compatibility_adapter: ProviderCompatibilityAdapter::GenericOpenAiChat,
        http_headers: Default::default(),
        requires_adapter: Vec::new(),
    });

    let resolved = registry.get("openai").expect("direct id should resolve");

    assert_eq!(resolved.id, "openai");
    assert_eq!(resolved.display_name, "OpenAI");
}

#[test]
fn loads_json_and_toml_provider_profiles_from_kay_home() {
    let home = tempdir().expect("tempdir");
    let profile_dir = home.path().join("provider_profiles").join("hermes");
    std::fs::create_dir_all(&profile_dir).expect("profile dir");
    std::fs::write(
        profile_dir.join("openrouter.json"),
        r#"{
  "id": "openrouter",
  "display_name": "OpenRouter",
  "aliases": ["or"],
  "wire_api": "chat",
  "auth_kind": "api_key",
  "base_url": "https://openrouter.ai/api/v1",
  "models_url": "https://openrouter.ai/api/v1/models",
  "credential_ref": "openrouter",
  "compatibility_adapter": "openrouter"
}"#,
    )
    .expect("json profile");
    std::fs::write(
        home.path().join("provider_profiles").join("novita.toml"),
        r#"
id = "novita"
display_name = "Novita"
wire_api = "chat"
auth_kind = "api_key"
base_url = "https://api.novita.ai/v3/openai"
env_vars = ["NOVITA_API_KEY"]
compatibility_adapter = "generic_openai_chat"
"#,
    )
    .expect("toml profile");

    let profiles = load_provider_profiles_from_home(home.path()).expect("profiles should load");
    let ids = profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["novita", "openrouter"]);
}

#[test]
fn config_loads_provider_profiles_and_config_overrides_imported_profiles() {
    let home = tempdir().expect("tempdir");
    let profile_dir = home.path().join("provider_profiles").join("hermes");
    std::fs::create_dir_all(&profile_dir).expect("profile dir");
    std::fs::write(
        profile_dir.join("novita.json"),
        r#"{
  "id": "novita",
  "display_name": "Novita Imported",
  "wire_api": "chat",
  "auth_kind": "api_key",
  "base_url": "https://api.novita.ai/v3/openai",
  "env_vars": ["NOVITA_API_KEY"],
  "credential_ref": "novita",
  "compatibility_adapter": "generic_openai_chat"
}"#,
    )
    .expect("profile");
    std::fs::write(
        home.path().join("config.toml"),
        r#"
model_provider = "novita"
model = "deepseek/deepseek-v3"

[model_providers.novita]
name = "Novita Configured"
base_url = "https://override.example/v1"
wire_api = "chat"
requires_openai_auth = false
"#,
    )
    .expect("config");

    let config = ConfigBuilder::new()
        .with_code_home(home.path().to_path_buf())
        .load()
        .expect("config should load");

    assert_eq!(config.model_provider_id, "novita");
    assert_eq!(config.model_provider.name, "Novita Configured");
    assert_eq!(
        config.model_provider.base_url.as_deref(),
        Some("https://override.example/v1")
    );
}

#[test]
fn config_can_select_provider_defined_only_by_imported_profile() {
    let home = tempdir().expect("tempdir");
    let profile_dir = home.path().join("provider_profiles").join("hermes");
    std::fs::create_dir_all(&profile_dir).expect("profile dir");
    std::fs::write(
        profile_dir.join("novita.json"),
        r#"{
  "id": "novita",
  "display_name": "Novita Imported",
  "wire_api": "chat",
  "auth_kind": "api_key",
  "base_url": "https://api.novita.ai/v3/openai",
  "env_vars": ["NOVITA_API_KEY"],
  "credential_ref": "novita",
  "compatibility_adapter": "generic_openai_chat"
}"#,
    )
    .expect("profile");
    std::fs::write(
        home.path().join("config.toml"),
        r#"
model_provider = "novita"
model = "deepseek/deepseek-v3"
"#,
    )
    .expect("config");

    let config = ConfigBuilder::new()
        .with_code_home(home.path().to_path_buf())
        .load()
        .expect("config should load imported provider profile");

    assert_eq!(config.model_provider_id, "novita");
    assert_eq!(config.model_provider.name, "Novita Imported");
    assert_eq!(
        config.model_provider.base_url.as_deref(),
        Some("https://api.novita.ai/v3/openai")
    );
    assert_eq!(config.model_provider.env_key.as_deref(), Some("NOVITA_API_KEY"));
}

#[test]
fn config_merges_partial_openrouter_routing_override_with_built_in_profile() {
    let home = tempdir().expect("tempdir");
    std::fs::write(
        home.path().join("config.toml"),
        r#"
model_provider = "openrouter"
model = "anthropic/claude-sonnet-4.5"

[model_providers.openrouter.openrouter]
require_parameters = true
order = ["Anthropic", "Google"]
"#,
    )
    .expect("config");

    let config = ConfigBuilder::new()
        .with_code_home(home.path().to_path_buf())
        .load()
        .expect("partial OpenRouter config should merge onto built-in profile");

    assert_eq!(config.model_provider_id, "openrouter");
    assert_eq!(config.model_provider.name, "OpenRouter");
    assert_eq!(
        config.model_provider.base_url.as_deref(),
        Some(code_core::OPENROUTER_DEFAULT_BASE_URL)
    );
    assert_eq!(
        config.model_provider.env_key.as_deref(),
        Some("OPENROUTER_API_KEY")
    );
    let openrouter = config
        .model_provider
        .openrouter
        .expect("OpenRouter config should remain enabled");
    assert_eq!(openrouter.extra.get("require_parameters"), Some(&json!(true)));
    assert_eq!(
        openrouter.extra.get("order"),
        Some(&json!(["Anthropic", "Google"]))
    );
}

#[test]
fn built_in_profiles_include_openrouter_and_bedrock() {
    let providers = built_in_model_providers(None);
    let openrouter = providers
        .get("openrouter")
        .expect("openrouter should be built in");
    let bedrock = providers
        .get("amazon-bedrock")
        .expect("amazon-bedrock should be built in");

    assert_eq!(openrouter.wire_api, WireApi::Chat);
    assert_eq!(openrouter.credential_ref.as_deref(), Some("openrouter"));
    assert!(openrouter.openrouter_config().is_some());
    assert_eq!(bedrock.wire_api, WireApi::BedrockConverse);
    assert!(!bedrock.requires_openai_auth);
}
