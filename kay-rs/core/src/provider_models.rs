//! Provider model menu entries loaded from [`provider-models.json`].
//!
//! Third-party `/model` picker entries and visibility whitelists are driven from
//! this manifest instead of hardcoded Rust lists.

use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::MINIMAX_PROVIDER_ID;
use crate::OPENCODE_GO_PROVIDER_ID;
use crate::XIAOMI_PROVIDER_ID;
use crate::model_family::provider_model_slug;

const PROVIDER_MODELS_JSON: &str = include_str!("../provider-models.json");

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderModelEntry {
    pub slug: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProviderModelsSection {
    #[serde(default)]
    description: Option<String>,
    models: Vec<ProviderModelEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProviderModelsManifest {
    providers: HashMap<String, ProviderModelsSection>,
}

static MANIFEST: Lazy<ProviderModelsManifest> = Lazy::new(|| {
    serde_json::from_str(PROVIDER_MODELS_JSON)
        .expect("provider-models.json must deserialize")
});

static OPENCODE_GO_MODELS: Lazy<Vec<ProviderModelEntry>> = Lazy::new(|| {
    MANIFEST
        .providers
        .get(OPENCODE_GO_PROVIDER_ID)
        .map(|section| section.models.clone())
        .unwrap_or_default()
});

static MINIMAX_MODELS: Lazy<Vec<ProviderModelEntry>> = Lazy::new(|| {
    MANIFEST
        .providers
        .get(MINIMAX_PROVIDER_ID)
        .map(|section| section.models.clone())
        .unwrap_or_default()
});

static XIAOMI_MODELS: Lazy<Vec<ProviderModelEntry>> = Lazy::new(|| {
    MANIFEST
        .providers
        .get(XIAOMI_PROVIDER_ID)
        .map(|section| section.models.clone())
        .unwrap_or_default()
});

fn provider_description(provider_id: &str, fallback: &str) -> String {
    MANIFEST
        .providers
        .get(provider_id)
        .and_then(|section| section.description.as_deref())
        .unwrap_or(fallback)
        .to_string()
}

pub fn opencode_go_models() -> &'static [ProviderModelEntry] {
    OPENCODE_GO_MODELS.as_slice()
}

pub fn minimax_models() -> &'static [ProviderModelEntry] {
    MINIMAX_MODELS.as_slice()
}

pub fn xiaomi_models() -> &'static [ProviderModelEntry] {
    XIAOMI_MODELS.as_slice()
}

pub fn opencode_go_model_slugs() -> Vec<String> {
    opencode_go_models()
        .iter()
        .map(|entry| entry.slug.clone())
        .collect()
}

pub fn opencode_go_preset_ids() -> Vec<String> {
    opencode_go_models()
        .iter()
        .map(|entry| format!("{OPENCODE_GO_PROVIDER_ID}/{}", entry.slug))
        .collect()
}

pub fn minimax_preset_ids() -> Vec<String> {
    minimax_models()
        .iter()
        .map(|entry| entry.slug.clone())
        .collect()
}

pub fn xiaomi_preset_ids() -> Vec<String> {
    xiaomi_models()
        .iter()
        .map(|entry| format!("{XIAOMI_PROVIDER_ID}/{}", entry.slug))
        .collect()
}

pub fn opencode_go_default_description() -> String {
    provider_description(OPENCODE_GO_PROVIDER_ID, "OpenCode Go coding model.")
}

pub fn xiaomi_default_description() -> String {
    provider_description(XIAOMI_PROVIDER_ID, "Xiaomi MiMo coding model.")
}

pub fn matches_opencode_go_supported_model(model: &str) -> bool {
    let Some((namespace, _)) = model.trim().split_once('/') else {
        return false;
    };
    if !namespace.eq_ignore_ascii_case(OPENCODE_GO_PROVIDER_ID) {
        return false;
    }
    let slug = provider_model_slug(OPENCODE_GO_PROVIDER_ID, model);
    opencode_go_models()
        .iter()
        .any(|entry| slug.as_ref().trim().eq_ignore_ascii_case(&entry.slug))
}

pub fn matches_minimax_model(model: &str) -> bool {
    minimax_models()
        .iter()
        .any(|entry| model.trim().eq_ignore_ascii_case(&entry.slug))
}

pub fn matches_xiaomi_supported_model(model: &str) -> bool {
    let Some((namespace, _)) = model.trim().split_once('/') else {
        return false;
    };
    if !namespace.eq_ignore_ascii_case(XIAOMI_PROVIDER_ID) {
        return false;
    }
    let slug = provider_model_slug(XIAOMI_PROVIDER_ID, model);
    xiaomi_models()
        .iter()
        .any(|entry| slug.as_ref().trim().eq_ignore_ascii_case(&entry.slug))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_lists_all_opencode_go_docs_models() {
        let slugs = opencode_go_model_slugs();
        for expected in [
            "glm-5.1",
            "glm-5.2",
            "glm-5",
            "kimi-k2.7-code",
            "kimi-k2.6",
            "mimo-v2.5-pro",
            "mimo-v2.5",
            "minimax-m3",
            "minimax-m2.7",
            "qwen3.7-max",
            "qwen3.7-plus",
            "qwen3.6-plus",
            "deepseek-v4-pro",
            "deepseek-v4-flash",
        ] {
            assert!(
                slugs.iter().any(|slug| slug == expected),
                "missing OpenCode Go model {expected}"
            );
        }
        assert_eq!(slugs.len(), 14);
    }

    #[test]
    fn manifest_lists_minimax_m3() {
        assert!(matches_minimax_model("MiniMax-M3"));
        assert!(minimax_models().iter().any(|entry| entry.slug == "MiniMax-M3"));
    }

    #[test]
    fn opencode_go_visibility_matches_manifest() {
        assert!(matches_opencode_go_supported_model("opencode-go/qwen3.7-plus"));
        assert!(matches_opencode_go_supported_model("opencode-go/minimax-m3"));
        assert!(!matches_opencode_go_supported_model("opencode-go/qwen3.7-max-beta"));
    }
}
