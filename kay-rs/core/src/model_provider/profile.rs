use crate::model_provider::types::ProviderAuthKind;
use crate::model_provider::types::ProviderCapabilities;
use crate::model_provider::types::ProviderCompatibilityAdapter;
use crate::model_provider_info::ChatCompletionsFormat;
use crate::model_provider_info::ModelProviderInfo;
use crate::model_provider_info::OpenRouterConfig;
use crate::model_provider_info::WireApi;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use thiserror::Error;

pub trait ModelProvider: std::fmt::Debug + Send + Sync {
    fn profile(&self) -> &ProviderProfile;

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }

    fn provider_info(&self) -> Result<ModelProviderInfo, ProviderProfileError> {
        self.profile().to_model_provider_info()
    }
}

#[derive(Debug, Clone)]
pub struct ConfiguredModelProvider {
    profile: ProviderProfile,
    capabilities: ProviderCapabilities,
}

impl ConfiguredModelProvider {
    pub fn new(profile: ProviderProfile) -> Self {
        Self {
            profile,
            capabilities: ProviderCapabilities::default(),
        }
    }

    pub fn with_capabilities(profile: ProviderProfile, capabilities: ProviderCapabilities) -> Self {
        Self {
            profile,
            capabilities,
        }
    }
}

impl ModelProvider for ConfiguredModelProvider {
    fn profile(&self) -> &ProviderProfile {
        &self.profile
    }

    fn capabilities(&self) -> ProviderCapabilities {
        self.capabilities.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProviderProfile {
    pub id: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub wire_api: WireApi,
    #[serde(default)]
    pub auth_kind: ProviderAuthKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_vars: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<String>,
    #[serde(default)]
    pub compatibility_adapter: ProviderCompatibilityAdapter,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub http_headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires_adapter: Vec<String>,
}

impl ProviderProfile {
    pub fn to_model_provider_info(&self) -> Result<ModelProviderInfo, ProviderProfileError> {
        if !self.requires_adapter.is_empty() {
            return Err(ProviderProfileError::RequiresAdapter {
                provider_id: self.id.clone(),
                hooks: self.requires_adapter.clone(),
            });
        }

        let env_key = self.env_vars.first().cloned();
        let credential_ref = match self.auth_kind {
            ProviderAuthKind::ApiKey => self
                .credential_ref
                .clone()
                .or_else(|| Some(self.id.to_ascii_lowercase())),
            ProviderAuthKind::None
            | ProviderAuthKind::AwsSdk
            | ProviderAuthKind::OauthDeviceCode
            | ProviderAuthKind::OauthExternal
            | ProviderAuthKind::Copilot => self.credential_ref.clone(),
        };
        let openrouter = if self.compatibility_adapter == ProviderCompatibilityAdapter::OpenRouter {
            Some(OpenRouterConfig::default())
        } else {
            None
        };

        Ok(ModelProviderInfo {
            name: if self.display_name.trim().is_empty() {
                self.id.clone()
            } else {
                self.display_name.clone()
            },
            base_url: self.base_url.clone(),
            env_key,
            env_key_instructions: self.env_vars.first().map(|env_key| {
                format!(
                    "Set {env_key} or run `kay login --provider {} --with-api-key`.",
                    self.id
                )
            }),
            experimental_bearer_token: None,
            auth: None,
            credential_ref,
            wire_api: self.wire_api,
            chat_completions_format: ChatCompletionsFormat::OpenAi,
            query_params: None,
            http_headers: (!self.http_headers.is_empty())
                .then(|| self.http_headers.clone().into_iter().collect()),
            env_http_headers: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            openrouter,
        })
    }
}

#[derive(Debug, Error)]
pub enum ProviderProfileError {
    #[error("provider `{provider_id}` requires a Kay adapter for Hermes hooks: {}", hooks.join(", "))]
    RequiresAdapter {
        provider_id: String,
        hooks: Vec<String>,
    },
    #[error("unsupported Hermes api_mode `{api_mode}` for provider `{provider_id}`")]
    UnsupportedApiMode {
        provider_id: String,
        api_mode: String,
    },
    #[error("unsupported Hermes auth_type `{auth_type}` for provider `{provider_id}`")]
    UnsupportedAuthType {
        provider_id: String,
        auth_type: String,
    },
}
