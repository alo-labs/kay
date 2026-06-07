use crate::model_provider::ids::normalize_provider_id;
use crate::model_provider::profile::ProviderProfile;
use crate::model_provider::profile::ProviderProfileError;
use crate::model_provider::types::ProviderAuthKind;
use crate::model_provider::types::ProviderCompatibilityAdapter;
use crate::model_provider_info::WireApi;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct HermesProviderExport {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub signup_url: String,
    #[serde(default)]
    pub api_mode: String,
    #[serde(default)]
    pub env_vars: Vec<String>,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub models_url: String,
    #[serde(default)]
    pub auth_type: String,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub fallback_models: Vec<String>,
    #[serde(default)]
    pub default_headers: BTreeMap<String, String>,
    #[serde(default)]
    pub overridden_hooks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CompiledHermesProvider {
    pub profile: ProviderProfile,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires_adapter: Vec<String>,
}

pub fn compile_hermes_export(
    export: HermesProviderExport,
) -> Result<CompiledHermesProvider, ProviderProfileError> {
    let provider_id = normalize_provider_id(&export.name);
    let wire_api = wire_api_from_hermes(&provider_id, &export.api_mode)?;
    let auth_kind = auth_kind_from_hermes(&provider_id, &export.auth_type)?;
    let adapter = adapter_for_hermes_provider(&provider_id, wire_api);
    let mut hooks = export.overridden_hooks.clone();
    hooks.sort();
    hooks.dedup();
    let requires_adapter = if hooks.is_empty() || adapter_supports_hooks(&adapter, &hooks) {
        Vec::new()
    } else {
        hooks
    };

    let profile = ProviderProfile {
        id: provider_id.clone(),
        display_name: if export.display_name.trim().is_empty() {
            provider_id.clone()
        } else {
            export.display_name
        },
        aliases: export
            .aliases
            .into_iter()
            .map(|alias| alias.trim().to_ascii_lowercase())
            .filter(|alias| !alias.is_empty())
            .collect(),
        wire_api,
        auth_kind,
        base_url: non_empty_string(export.base_url),
        models_url: non_empty_string(export.models_url),
        env_vars: export
            .env_vars
            .into_iter()
            .map(|env| env.trim().to_string())
            .filter(|env| !env.is_empty())
            .collect(),
        credential_ref: Some(provider_id),
        compatibility_adapter: adapter,
        http_headers: non_empty_headers(export.default_headers),
        requires_adapter: requires_adapter.clone(),
    };

    Ok(CompiledHermesProvider {
        profile,
        requires_adapter,
    })
}

pub fn compile_hermes_provider_exports(
    exports: Vec<HermesProviderExport>,
) -> Vec<Result<CompiledHermesProvider, ProviderProfileError>> {
    exports.into_iter().map(compile_hermes_export).collect()
}

fn non_empty_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn non_empty_headers(headers: BTreeMap<String, String>) -> BTreeMap<String, String> {
    headers
        .into_iter()
        .map(|(name, value)| (name.trim().to_string(), value))
        .filter(|(name, _)| !name.is_empty())
        .collect()
}

fn wire_api_from_hermes(provider_id: &str, api_mode: &str) -> Result<WireApi, ProviderProfileError> {
    match api_mode.trim() {
        "" | "chat_completions" => Ok(WireApi::Chat),
        "codex_responses" => Ok(WireApi::Responses),
        "anthropic_messages" => Ok(WireApi::AnthropicMessages),
        "bedrock_converse" => Ok(WireApi::BedrockConverse),
        "gemini_native" => Ok(WireApi::GeminiNative),
        other => Err(ProviderProfileError::UnsupportedApiMode {
            provider_id: provider_id.to_string(),
            api_mode: other.to_string(),
        }),
    }
}

fn auth_kind_from_hermes(
    provider_id: &str,
    auth_type: &str,
) -> Result<ProviderAuthKind, ProviderProfileError> {
    match auth_type.trim() {
        "" | "api_key" => Ok(ProviderAuthKind::ApiKey),
        "aws_sdk" => Ok(ProviderAuthKind::AwsSdk),
        "oauth_device_code" => Ok(ProviderAuthKind::OauthDeviceCode),
        "oauth_external" | "external_process" => Ok(ProviderAuthKind::OauthExternal),
        "copilot" => Ok(ProviderAuthKind::Copilot),
        other => Err(ProviderProfileError::UnsupportedAuthType {
            provider_id: provider_id.to_string(),
            auth_type: other.to_string(),
        }),
    }
}

fn adapter_for_hermes_provider(id: &str, wire_api: WireApi) -> ProviderCompatibilityAdapter {
    match id {
        "openrouter" => ProviderCompatibilityAdapter::OpenRouter,
        "nous" => ProviderCompatibilityAdapter::Nous,
        "gemini" | "google-gemini-cli" => ProviderCompatibilityAdapter::GeminiThinking,
        "deepseek" => ProviderCompatibilityAdapter::DeepseekReasoning,
        "kimi-coding" | "kimi-coding-cn" => ProviderCompatibilityAdapter::KimiReasoning,
        "qwen-oauth" => ProviderCompatibilityAdapter::QwenOauth,
        "opencode-zen" | "opencode-go" => ProviderCompatibilityAdapter::OpencodeZen,
        "bedrock" | "amazon-bedrock" => ProviderCompatibilityAdapter::BedrockConverse,
        "anthropic" => ProviderCompatibilityAdapter::AnthropicMessages,
        "copilot" => ProviderCompatibilityAdapter::Copilot,
        "copilot-acp" => ProviderCompatibilityAdapter::CopilotAcp,
        "custom" => ProviderCompatibilityAdapter::Custom,
        _ if matches!(wire_api, WireApi::BedrockConverse) => {
            ProviderCompatibilityAdapter::BedrockConverse
        }
        _ if matches!(wire_api, WireApi::AnthropicMessages) => {
            ProviderCompatibilityAdapter::AnthropicMessages
        }
        _ => ProviderCompatibilityAdapter::GenericOpenAiChat,
    }
}

fn adapter_supports_hooks(adapter: &ProviderCompatibilityAdapter, hooks: &[String]) -> bool {
    let supported: &[&str] = match adapter {
        ProviderCompatibilityAdapter::OpenRouter => {
            &["build_api_kwargs_extras", "build_extra_body", "fetch_models"]
        }
        ProviderCompatibilityAdapter::Nous => &["build_api_kwargs_extras", "build_extra_body"],
        ProviderCompatibilityAdapter::GeminiThinking => &["build_extra_body"],
        ProviderCompatibilityAdapter::DeepseekReasoning => &["build_api_kwargs_extras"],
        ProviderCompatibilityAdapter::KimiReasoning => &["build_api_kwargs_extras"],
        ProviderCompatibilityAdapter::QwenOauth => {
            &["build_api_kwargs_extras", "build_extra_body", "prepare_messages"]
        }
        ProviderCompatibilityAdapter::OpencodeZen => &["build_api_kwargs_extras", "get_max_tokens"],
        ProviderCompatibilityAdapter::BedrockConverse => &["fetch_models"],
        ProviderCompatibilityAdapter::AnthropicMessages => &["fetch_models"],
        ProviderCompatibilityAdapter::Copilot => &["build_api_kwargs_extras"],
        ProviderCompatibilityAdapter::CopilotAcp => &["fetch_models"],
        ProviderCompatibilityAdapter::Custom => &["build_api_kwargs_extras", "fetch_models"],
        ProviderCompatibilityAdapter::GenericOpenAiChat => &[],
    };
    hooks.iter().all(|hook| supported.contains(&hook.as_str()))
}
