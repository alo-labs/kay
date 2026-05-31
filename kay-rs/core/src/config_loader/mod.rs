mod config_requirements;
mod macos;

use crate::config::CONFIG_TOML_FILE;
use crate::model_provider_info::MINIMAX_DEFAULT_BASE_URL;
use crate::model_provider_info::OPENCODE_GO_DEFAULT_BASE_URL;
use crate::model_provider_info::XIAOMI_DEFAULT_BASE_URL;
use config_requirements::ConfigRequirements;
use config_requirements::ConfigRequirementsToml;
use config_requirements::LegacyManagedConfigToml;
use macos::load_managed_admin_config_layer;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::runtime::{Builder as RuntimeBuilder, Handle};
use toml::Table as TomlTable;
use toml::Value as TomlValue;

#[cfg(unix)]
const CODE_MANAGED_CONFIG_SYSTEM_PATH: &str = "/etc/code/managed_config.toml";

#[cfg(unix)]
const CODE_REQUIREMENTS_SYSTEM_PATH: &str = "/etc/code/requirements.toml";

const LEGACY_KAY_TOML_FILE: &str = "kay.toml";

#[derive(Debug)]
pub(crate) struct LoadedConfigLayers {
    pub base: TomlValue,
    pub managed_config: Option<TomlValue>,
    pub managed_preferences: Option<TomlValue>,
}

#[derive(Debug, Default)]
pub(crate) struct LoaderOverrides {
    pub managed_config_path: Option<PathBuf>,
    pub requirements_path: Option<PathBuf>,
    #[cfg(target_os = "macos")]
    pub managed_preferences_base64: Option<String>,
}

// Configuration layering pipeline (top overrides bottom):
//
//        +-------------------------+
//        | Managed preferences (*) |
//        +-------------------------+
//                    ^
//                    |
//        +-------------------------+
//        |  managed_config.toml   |
//        +-------------------------+
//                    ^
//                    |
//        +-------------------------+
//        |    config.toml (base)   |
//        +-------------------------+
//
// (*) Only available on macOS via managed device profiles.

#[allow(dead_code)]
pub async fn load_config_as_toml(code_home: &Path) -> io::Result<TomlValue> {
    load_config_as_toml_with_overrides(code_home, LoaderOverrides::default()).await
}

fn default_empty_table() -> TomlValue {
    TomlValue::Table(Default::default())
}

#[allow(dead_code)]
pub(crate) async fn load_config_layers_with_overrides(
    code_home: &Path,
    overrides: LoaderOverrides,
) -> io::Result<LoadedConfigLayers> {
    load_config_layers_internal(code_home, overrides).await
}

pub(crate) fn load_config_as_toml_blocking(
    code_home: &Path,
    overrides: LoaderOverrides,
) -> io::Result<TomlValue> {
    let code_home = code_home.to_path_buf();
    block_on_loader(async move { load_config_as_toml_with_overrides(&code_home, overrides).await })
}

pub(crate) fn load_config_requirements_blocking(
    code_home: &Path,
    overrides: LoaderOverrides,
) -> io::Result<ConfigRequirements> {
    let code_home = code_home.to_path_buf();
    block_on_loader(async move { load_config_requirements_internal(&code_home, overrides).await })
}

async fn load_config_as_toml_with_overrides(
    code_home: &Path,
    overrides: LoaderOverrides,
) -> io::Result<TomlValue> {
    let layers = load_config_layers_internal(code_home, overrides).await?;
    Ok(apply_managed_layers(layers))
}

async fn load_config_layers_internal(
    code_home: &Path,
    overrides: LoaderOverrides,
) -> io::Result<LoadedConfigLayers> {
    #[cfg(target_os = "macos")]
    let LoaderOverrides {
        managed_config_path,
        requirements_path: _,
        managed_preferences_base64,
    } = overrides;

    #[cfg(not(target_os = "macos"))]
    let LoaderOverrides {
        managed_config_path,
        requirements_path: _,
    } = overrides;

    let managed_config_path =
        managed_config_path.unwrap_or_else(|| managed_config_default_path(code_home));

    let user_config_path = code_home.join(CONFIG_TOML_FILE);
    let user_config = match read_config_from_path(&user_config_path, true).await? {
        Some(config) => Some(config),
        None => read_legacy_kay_config(code_home).await?,
    };
    let managed_config = read_config_from_path(&managed_config_path, false).await?;

    #[cfg(target_os = "macos")]
    let managed_preferences =
        load_managed_admin_config_layer(managed_preferences_base64.as_deref()).await?;

    #[cfg(not(target_os = "macos"))]
    let managed_preferences = load_managed_admin_config_layer(None).await?;

    Ok(LoadedConfigLayers {
        base: user_config.unwrap_or_else(default_empty_table),
        managed_config,
        managed_preferences,
    })
}

async fn load_config_requirements_internal(
    code_home: &Path,
    overrides: LoaderOverrides,
) -> io::Result<ConfigRequirements> {
    #[cfg(target_os = "macos")]
    let LoaderOverrides {
        managed_config_path,
        requirements_path,
        managed_preferences_base64,
    } = overrides;

    #[cfg(not(target_os = "macos"))]
    let LoaderOverrides {
        managed_config_path,
        requirements_path,
    } = overrides;

    let managed_config_path =
        managed_config_path.unwrap_or_else(|| managed_config_default_path(code_home));
    let requirements_path = requirements_path.unwrap_or_else(|| requirements_default_path(code_home));

    let mut requirements = if let Some(value) = read_config_from_path(&requirements_path, false).await? {
        let parsed: ConfigRequirementsToml =
            value.try_into().map_err(|err: toml::de::Error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse config requirements TOML: {err}"),
                )
            })?;
        ConfigRequirements::try_from(parsed)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?
    } else {
        ConfigRequirements::default()
    };

    let managed_config = read_config_from_path(&managed_config_path, false).await?;

    #[cfg(target_os = "macos")]
    let managed_preferences =
        load_managed_admin_config_layer(managed_preferences_base64.as_deref()).await?;

    #[cfg(not(target_os = "macos"))]
    let managed_preferences = None;

    // If multiple legacy layers specify approval_policy (e.g. both a managed_config
    // file and macOS managed preferences), allow the later/higher-precedence layer
    // to override earlier ones.
    let mut legacy_approval_policy = None;

    for legacy in [managed_config, managed_preferences].into_iter().flatten() {
        let legacy: LegacyManagedConfigToml = legacy.try_into().map_err(|err: toml::de::Error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse legacy managed_config TOML: {err}"),
            )
        })?;

        if let Some(approval_policy) = legacy.approval_policy {
            legacy_approval_policy = Some(approval_policy);
        }
    }

    if let Some(approval_policy) = legacy_approval_policy {
        requirements.approval_policy.can_set(&approval_policy)?;
        requirements.approval_policy = crate::config::Constrained::allow_only(approval_policy);
    }

    Ok(requirements)
}

async fn read_config_from_path(
    path: &Path,
    log_missing_as_info: bool,
) -> io::Result<Option<TomlValue>> {
    match fs::read_to_string(path).await {
        Ok(contents) => match toml::from_str::<TomlValue>(&contents) {
            Ok(value) => Ok(Some(value)),
            Err(err) => {
                tracing::error!("Failed to parse {}: {err}", path.display());
                Err(io::Error::new(io::ErrorKind::InvalidData, err))
            }
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            if log_missing_as_info {
                tracing::info!("{} not found, using defaults", path.display());
            } else {
                tracing::debug!("{} not found", path.display());
            }
            Ok(None)
        }
        Err(err) => {
            tracing::error!("Failed to read {}: {err}", path.display());
            Err(err)
        }
    }
}

async fn read_legacy_kay_config(code_home: &Path) -> io::Result<Option<TomlValue>> {
    let path = code_home.join(LEGACY_KAY_TOML_FILE);
    read_config_from_path(&path, false)
        .await
        .map(|config| config.map(translate_legacy_kay_config))
}

fn translate_legacy_kay_config(mut config: TomlValue) -> TomlValue {
    let Some(root) = config.as_table_mut() else {
        return config;
    };
    let Some(provider_table) = root.get("provider").and_then(TomlValue::as_table).cloned() else {
        return config;
    };

    let default_model = provider_table
        .get("default_model")
        .and_then(TomlValue::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .map(ToString::to_string);

    if let Some(model) = default_model.as_ref()
        && !root.contains_key("model")
    {
        root.insert("model".to_string(), TomlValue::String(model.clone()));
    }

    if root.contains_key("model_provider") {
        return config;
    }

    let Some(provider_id) = infer_legacy_provider_id(default_model.as_deref(), &provider_table)
    else {
        return config;
    };

    let provider_config = provider_table
        .get(&provider_id)
        .and_then(TomlValue::as_table)
        .or_else(|| find_legacy_provider_table(&provider_table, &provider_id));
    let effective_provider_id =
        maybe_insert_legacy_model_provider(root, &provider_id, provider_config);
    root.insert(
        "model_provider".to_string(),
        TomlValue::String(effective_provider_id),
    );

    config
}

fn infer_legacy_provider_id(
    default_model: Option<&str>,
    provider_table: &TomlTable,
) -> Option<String> {
    if let Some(model) = default_model {
        let normalized_model = model.trim().to_ascii_lowercase();
        if normalized_model.starts_with("opencode-go/") {
            return Some("opencode-go".to_string());
        }
        if normalized_model.starts_with("xiaomi/") {
            return Some("xiaomi".to_string());
        }
        if normalized_model.contains("minimax") {
            return Some("minimax".to_string());
        }
    }

    let mut providers = provider_table
        .iter()
        .filter_map(|(key, value)| value.as_table().map(|_| normalize_legacy_provider_id(key)))
        .collect::<Vec<_>>();
    providers.sort();
    providers.dedup();
    if providers.len() == 1 {
        providers.into_iter().next()
    } else {
        None
    }
}

fn find_legacy_provider_table<'a>(
    provider_table: &'a TomlTable,
    provider_id: &str,
) -> Option<&'a TomlTable> {
    provider_table.iter().find_map(|(key, value)| {
        if normalize_legacy_provider_id(key) == provider_id {
            value.as_table()
        } else {
            None
        }
    })
}

fn normalize_legacy_provider_id(provider_id: &str) -> String {
    let normalized = provider_id.trim().to_ascii_lowercase().replace('_', "-");
    match normalized.as_str() {
        "mini-max" => "minimax".to_string(),
        "open-code-go" | "opencodego" => "opencode-go".to_string(),
        _ => normalized,
    }
}

fn maybe_insert_legacy_model_provider(
    root: &mut TomlTable,
    provider_id: &str,
    provider_config: Option<&TomlTable>,
) -> String {
    let Some(provider_config) = provider_config else {
        return provider_id.to_string();
    };

    let api_key = provider_config
        .get("api_key")
        .and_then(TomlValue::as_str)
        .map(str::trim)
        .filter(|key| !key.is_empty());
    let base_url = legacy_provider_base_url(provider_id, provider_config);

    if api_key.is_none() && base_url.is_none() {
        return provider_id.to_string();
    }

    let legacy_provider_id = format!("kay-legacy-{provider_id}");
    let model_providers = root
        .entry("model_providers".to_string())
        .or_insert_with(|| TomlValue::Table(TomlTable::new()));
    let Some(model_providers) = model_providers.as_table_mut() else {
        return provider_id.to_string();
    };
    if model_providers.contains_key(&legacy_provider_id) {
        return legacy_provider_id;
    }

    let mut provider = TomlTable::new();
    provider.insert(
        "name".to_string(),
        TomlValue::String(legacy_provider_name(provider_id).to_string()),
    );
    if let Some(base_url) = base_url {
        provider.insert("base_url".to_string(), TomlValue::String(base_url));
    }
    if let Some(api_key) = api_key {
        provider.insert(
            "experimental_bearer_token".to_string(),
            TomlValue::String(api_key.to_string()),
        );
    } else {
        insert_known_provider_auth_fields(&mut provider, provider_id);
    }
    provider.insert("wire_api".to_string(), TomlValue::String("chat".to_string()));
    provider.insert(
        "chat_completions_format".to_string(),
        TomlValue::String(legacy_chat_completions_format(provider_id).to_string()),
    );
    provider.insert("requires_openai_auth".to_string(), TomlValue::Boolean(false));
    model_providers.insert(legacy_provider_id.clone(), TomlValue::Table(provider));
    legacy_provider_id
}

fn legacy_provider_name(provider_id: &str) -> &str {
    match provider_id {
        "minimax" => "MiniMax",
        "xiaomi" => "Xiaomi",
        "opencode-go" => "OpenCode Go",
        _ => provider_id,
    }
}

fn legacy_provider_base_url(provider_id: &str, provider_config: &TomlTable) -> Option<String> {
    provider_config
        .get("base_url")
        .or_else(|| provider_config.get("endpoint"))
        .and_then(TomlValue::as_str)
        .and_then(normalize_legacy_provider_endpoint)
        .or_else(|| match provider_id {
            "minimax" => Some(MINIMAX_DEFAULT_BASE_URL.to_string()),
            "xiaomi" => Some(XIAOMI_DEFAULT_BASE_URL.to_string()),
            "opencode-go" => Some(OPENCODE_GO_DEFAULT_BASE_URL.to_string()),
            _ => None,
        })
}

fn normalize_legacy_provider_endpoint(endpoint: &str) -> Option<String> {
    let endpoint = endpoint.trim().trim_end_matches('/');
    if endpoint.is_empty() {
        return None;
    }

    for suffix in ["/text/chatcompletion_v2", "/chat/completions", "/responses"] {
        if let Some(base) = endpoint.strip_suffix(suffix) {
            return Some(base.trim_end_matches('/').to_string());
        }
    }

    Some(endpoint.to_string())
}

fn insert_known_provider_auth_fields(provider: &mut TomlTable, provider_id: &str) {
    match provider_id {
        "minimax" => {
            provider.insert(
                "env_key".to_string(),
                TomlValue::String("MINIMAX_API_KEY".to_string()),
            );
            provider.insert(
                "credential_ref".to_string(),
                TomlValue::String("minimax".to_string()),
            );
        }
        "opencode-go" => {
            provider.insert(
                "env_key".to_string(),
                TomlValue::String("OPENCODE_GO_API_KEY".to_string()),
            );
            provider.insert(
                "credential_ref".to_string(),
                TomlValue::String("opencode-go".to_string()),
            );
        }
        "xiaomi" => {
            provider.insert(
                "env_key".to_string(),
                TomlValue::String("XIAOMI_API_KEY".to_string()),
            );
            provider.insert(
                "credential_ref".to_string(),
                TomlValue::String("xiaomi".to_string()),
            );
        }
        _ => {}
    }
}

fn legacy_chat_completions_format(provider_id: &str) -> &str {
    match provider_id {
        "minimax" => "minimax",
        _ => "openai",
    }
}

/// Merge config `overlay` into `base`, giving `overlay` precedence.
pub(crate) fn merge_toml_values(base: &mut TomlValue, overlay: &TomlValue) {
    if let TomlValue::Table(overlay_table) = overlay
        && let TomlValue::Table(base_table) = base
    {
        for (key, value) in overlay_table {
            if let Some(existing) = base_table.get_mut(key) {
                merge_toml_values(existing, value);
            } else {
                base_table.insert(key.clone(), value.clone());
            }
        }
    } else {
        *base = overlay.clone();
    }
}

fn managed_config_default_path(code_home: &Path) -> PathBuf {
    #[cfg(unix)]
    {
        let _ = code_home;
        PathBuf::from(CODE_MANAGED_CONFIG_SYSTEM_PATH)
    }

    #[cfg(not(unix))]
    {
        code_home.join("managed_config.toml")
    }
}

fn requirements_default_path(code_home: &Path) -> PathBuf {
    #[cfg(unix)]
    {
        let _ = code_home;
        PathBuf::from(CODE_REQUIREMENTS_SYSTEM_PATH)
    }

    #[cfg(not(unix))]
    {
        code_home.join("requirements.toml")
    }
}

fn apply_managed_layers(layers: LoadedConfigLayers) -> TomlValue {
    let LoadedConfigLayers {
        mut base,
        managed_config,
        managed_preferences,
    } = layers;

    for overlay in [managed_config, managed_preferences].into_iter().flatten() {
        merge_toml_values(&mut base, &overlay);
    }

    base
}

fn block_on_loader<F, T>(future: F) -> io::Result<T>
where
    F: std::future::Future<Output = io::Result<T>> + Send + 'static,
    T: Send + 'static,
{
    if Handle::try_current().is_ok() {
        std::thread::Builder::new()
            .name("config-loader".to_string())
            .spawn(move || run_future(future))
            .map_err(|err| io::Error::other(format!("config loader thread spawn failed: {err}")))?
            .join()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "config loader thread panicked"))?
    } else {
        run_future(future)
    }
}

fn run_future<F, T>(future: F) -> io::Result<T>
where
    F: std::future::Future<Output = io::Result<T>>,
    T: Send + 'static,
{
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    runtime.block_on(future)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn merges_managed_config_layer_on_top() {
        let tmp = tempdir().expect("tempdir");
        let managed_path = tmp.path().join("managed_config.toml");

        std::fs::write(
            tmp.path().join(CONFIG_TOML_FILE),
            r#"foo = 1

[nested]
value = "base"
"#,
        )
        .expect("write base");
        std::fs::write(
            &managed_path,
            r#"foo = 2

[nested]
value = "managed_config"
extra = true
"#,
        )
        .expect("write managed config");

        let overrides = LoaderOverrides {
            managed_config_path: Some(managed_path),
            requirements_path: None,
            #[cfg(target_os = "macos")]
            managed_preferences_base64: None,
        };

        let loaded = load_config_as_toml_with_overrides(tmp.path(), overrides)
            .await
            .expect("load config");
        let table = loaded.as_table().expect("top-level table expected");

        assert_eq!(table.get("foo"), Some(&TomlValue::Integer(2)));
        let nested = table
            .get("nested")
            .and_then(|v| v.as_table())
            .expect("nested");
        assert_eq!(
            nested.get("value"),
            Some(&TomlValue::String("managed_config".to_string()))
        );
        assert_eq!(nested.get("extra"), Some(&TomlValue::Boolean(true)));
    }

    #[tokio::test]
    async fn returns_empty_when_all_layers_missing() {
        let tmp = tempdir().expect("tempdir");
        let managed_path = tmp.path().join("managed_config.toml");
        let overrides = LoaderOverrides {
            managed_config_path: Some(managed_path),
            requirements_path: None,
            #[cfg(target_os = "macos")]
            managed_preferences_base64: None,
        };

        let layers = load_config_layers_with_overrides(tmp.path(), overrides)
            .await
            .expect("load layers");
        let base_table = layers.base.as_table().expect("base table expected");
        assert!(
            base_table.is_empty(),
            "expected empty base layer when configs missing"
        );
        assert!(
            layers.managed_config.is_none(),
            "managed config layer should be absent when file missing"
        );

        #[cfg(not(target_os = "macos"))]
        {
            let loaded = load_config_as_toml(tmp.path()).await.expect("load config");
            let table = loaded.as_table().expect("top-level table expected");
            assert!(
                table.is_empty(),
                "expected empty table when configs missing"
            );
        }
    }

    #[tokio::test]
    async fn loads_legacy_kay_toml_when_config_toml_is_missing() {
        let tmp = tempdir().expect("tempdir");
        let managed_path = tmp.path().join("managed_config.toml");
        std::fs::write(
            tmp.path().join("kay.toml"),
            r#"[provider]
default_model = "MiniMax-M2.7"

[provider.minimax]
api_key = "sk-minimax"
endpoint = "https://api.minimax.io/v1/text/chatcompletion_v2"
"#,
        )
        .expect("write kay.toml");

        let overrides = LoaderOverrides {
            managed_config_path: Some(managed_path),
            requirements_path: None,
            #[cfg(target_os = "macos")]
            managed_preferences_base64: None,
        };

        let loaded = load_config_as_toml_with_overrides(tmp.path(), overrides)
            .await
            .expect("load config");
        let table = loaded.as_table().expect("top-level table expected");

        assert_eq!(
            table.get("model"),
            Some(&TomlValue::String("MiniMax-M2.7".to_string()))
        );
        assert_eq!(
            table.get("model_provider"),
            Some(&TomlValue::String("kay-legacy-minimax".to_string()))
        );
        let providers = table
            .get("model_providers")
            .and_then(|value| value.as_table())
            .expect("model_providers table");
        let minimax = providers
            .get("kay-legacy-minimax")
            .and_then(|value| value.as_table())
            .expect("legacy MiniMax provider");
        assert_eq!(
            minimax.get("experimental_bearer_token"),
            Some(&TomlValue::String("sk-minimax".to_string()))
        );
        assert_eq!(
            minimax.get("base_url"),
            Some(&TomlValue::String("https://api.minimax.io/v1".to_string()))
        );
        assert_eq!(
            minimax.get("chat_completions_format"),
            Some(&TomlValue::String("minimax".to_string()))
        );
    }

    #[tokio::test]
    async fn config_toml_takes_precedence_over_legacy_kay_toml() {
        let tmp = tempdir().expect("tempdir");
        let managed_path = tmp.path().join("managed_config.toml");
        std::fs::write(
            tmp.path().join(CONFIG_TOML_FILE),
            r#"model = "gpt-5.4"
model_provider = "openai"
"#,
        )
        .expect("write config.toml");
        std::fs::write(
            tmp.path().join("kay.toml"),
            r#"[provider]
default_model = "MiniMax-M2.7"

[provider.minimax]
api_key = "sk-minimax"
"#,
        )
        .expect("write kay.toml");

        let overrides = LoaderOverrides {
            managed_config_path: Some(managed_path),
            requirements_path: None,
            #[cfg(target_os = "macos")]
            managed_preferences_base64: None,
        };

        let loaded = load_config_as_toml_with_overrides(tmp.path(), overrides)
            .await
            .expect("load config");
        let table = loaded.as_table().expect("top-level table expected");

        assert_eq!(
            table.get("model"),
            Some(&TomlValue::String("gpt-5.4".to_string()))
        );
        assert_eq!(
            table.get("model_provider"),
            Some(&TomlValue::String("openai".to_string()))
        );
        assert!(
            table.get("provider").is_none(),
            "legacy kay.toml should not be merged when config.toml exists"
        );
    }

    #[test]
    fn legacy_kay_toml_provider_becomes_runtime_config() {
        let tmp = tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join("kay.toml"),
            r#"[provider]
default_model = "MiniMax-M2.7"

[provider.minimax]
api_key = "sk-minimax"
endpoint = "https://api.minimax.io/v1/text/chatcompletion_v2"
"#,
        )
        .expect("write kay.toml");

        let config = crate::config::ConfigBuilder::new()
            .with_code_home(tmp.path().to_path_buf())
            .load()
            .expect("load runtime config");

        assert_eq!(config.model, "MiniMax-M2.7");
        assert_eq!(config.model_provider_id, "kay-legacy-minimax");
        assert_eq!(
            config.model_provider.experimental_bearer_token.as_deref(),
            Some("sk-minimax")
        );
        assert!(!config.model_provider.requires_openai_auth);
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn managed_preferences_take_highest_precedence() {
        use base64::Engine;

        let managed_payload = r#"
[nested]
value = "managed"
flag = false
"#;
        let encoded = base64::prelude::BASE64_STANDARD.encode(managed_payload.as_bytes());
        let tmp = tempdir().expect("tempdir");
        let managed_path = tmp.path().join("managed_config.toml");

        std::fs::write(
            tmp.path().join(CONFIG_TOML_FILE),
            r#"[nested]
value = "base"
"#,
        )
        .expect("write base");
        std::fs::write(
            &managed_path,
            r#"[nested]
value = "managed_config"
flag = true
"#,
        )
        .expect("write managed config");

        let overrides = LoaderOverrides {
            managed_config_path: Some(managed_path),
            requirements_path: None,
            managed_preferences_base64: Some(encoded),
        };

        let loaded = load_config_as_toml_with_overrides(tmp.path(), overrides)
            .await
            .expect("load config");
        let nested = loaded
            .get("nested")
            .and_then(|v| v.as_table())
            .expect("nested table");
        assert_eq!(
            nested.get("value"),
            Some(&TomlValue::String("managed".to_string()))
        );
        assert_eq!(nested.get("flag"), Some(&TomlValue::Boolean(false)));
    }
}
