use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAuthKind {
    #[default]
    ApiKey,
    None,
    AwsSdk,
    OauthDeviceCode,
    OauthExternal,
    Copilot,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCompatibilityAdapter {
    #[default]
    #[serde(rename = "generic_openai_chat")]
    GenericOpenAiChat,
    #[serde(rename = "openrouter")]
    OpenRouter,
    Nous,
    GeminiThinking,
    DeepseekReasoning,
    KimiReasoning,
    QwenOauth,
    OpencodeZen,
    BedrockConverse,
    AnthropicMessages,
    Copilot,
    CopilotAcp,
    Custom,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCatalogSource {
    #[default]
    GenericModelsEndpoint,
    OpenRouter,
    BedrockConverse,
    Static,
    None,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct ProviderCapabilities {
    pub vision: bool,
    pub tools: bool,
    pub namespace_tools: bool,
    pub image_generation: bool,
    pub web_search: bool,
}
