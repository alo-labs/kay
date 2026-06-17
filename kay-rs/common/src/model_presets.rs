use std::collections::HashMap;

use code_app_server_protocol::AuthMode;
use code_core::config_types::TextVerbosity as TextVerbosityConfig;
use code_core::protocol_config_types::ReasoningEffort;
use code_core::provider_models::{
    minimax_models, opencode_go_default_description, opencode_go_models, xiaomi_default_description,
    xiaomi_models,
};
use code_core::{OPENCODE_GO_PROVIDER_ID, XIAOMI_PROVIDER_ID};
use once_cell::sync::Lazy;

pub const HIDE_GPT5_1_MIGRATION_PROMPT_CONFIG: &str = "hide_gpt5_1_migration_prompt";
pub const HIDE_GPT_5_1_CODEX_MAX_MIGRATION_PROMPT_CONFIG: &str =
    "hide_gpt-5.1-codex-max_migration_prompt";
pub const HIDE_GPT_5_2_MIGRATION_PROMPT_CONFIG: &str = "hide_gpt5_2_migration_prompt";
pub const HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG: &str = "hide_gpt5_2_codex_migration_prompt";

/// A reasoning effort option surfaced for a model.
#[derive(Debug, Clone)]
pub struct ReasoningEffortPreset {
    pub effort: ReasoningEffort,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ModelUpgrade {
    pub id: String,
    pub reasoning_effort_mapping: Option<HashMap<ReasoningEffort, ReasoningEffort>>,
    pub migration_config_key: String,
}

/// Metadata describing a Kay-supported model.
#[derive(Debug, Clone)]
pub struct ModelPreset {
    pub id: String,
    pub model: String,
    pub display_name: String,
    pub description: String,
    pub default_reasoning_effort: ReasoningEffort,
    pub supported_reasoning_efforts: Vec<ReasoningEffortPreset>,
    pub supported_text_verbosity: &'static [TextVerbosityConfig],
    pub is_default: bool,
    pub upgrade: Option<ModelUpgrade>,
    pub pro_only: bool,
    pub show_in_picker: bool,
}

const ALL_TEXT_VERBOSITY: &[TextVerbosityConfig] = &[
    TextVerbosityConfig::Low,
    TextVerbosityConfig::Medium,
    TextVerbosityConfig::High,
];

static PRESETS: Lazy<Vec<ModelPreset>> = Lazy::new(|| {
    let third_party_preset = |model: &str,
                              display_name: &str,
                              description: &str|
     -> ModelPreset {
        ModelPreset {
            id: model.to_string(),
            model: model.to_string(),
            display_name: display_name.to_string(),
            description: description.to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: Vec::new(),
            supported_text_verbosity: &[TextVerbosityConfig::Medium],
            is_default: false,
            upgrade: None,
            pro_only: false,
            show_in_picker: true,
        }
    };

    let mut presets = vec![
        ModelPreset {
            id: "gpt-5.5".to_string(),
            model: "gpt-5.5".to_string(),
            display_name: "GPT-5.5".to_string(),
            description: "Frontier model for complex coding, research, and real-world work."
                .to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fast responses with lighter reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Balances speed and reasoning depth for everyday tasks".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Greater reasoning depth for complex problems".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning depth for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: None,
            pro_only: false,
            show_in_picker: true,
        },
        ModelPreset {
            id: "gpt-5.4".to_string(),
            model: "gpt-5.4".to_string(),
            display_name: "gpt-5.4".to_string(),
            description: "Frontier flagship model.".to_string(),
            default_reasoning_effort: ReasoningEffort::XHigh,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fast responses with lighter reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Balances speed and reasoning depth for everyday tasks".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex problems".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning depth for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: None,
            pro_only: false,
            show_in_picker: true,
        },
        ModelPreset {
            id: "gpt-5.4-mini".to_string(),
            model: "gpt-5.4-mini".to_string(),
            display_name: "gpt-5.4-mini".to_string(),
            description: "Smaller GPT-5.4 variant tuned for faster coding loops.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fast responses with lighter reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Balances speed and reasoning depth for everyday tasks".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex problems".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning depth for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: None,
            pro_only: false,
            show_in_picker: true,
        },
    ];

    let opencode_go_description = opencode_go_default_description();
    for entry in opencode_go_models() {
        let model = format!("{OPENCODE_GO_PROVIDER_ID}/{}", entry.slug);
        presets.push(third_party_preset(
            &model,
            &entry.display_name,
            entry
                .description
                .as_deref()
                .unwrap_or(&opencode_go_description),
        ));
    }

    for entry in minimax_models() {
        presets.push(third_party_preset(
            &entry.slug,
            &entry.display_name,
            entry
                .description
                .as_deref()
                .unwrap_or("MiniMax coding model."),
        ));
    }

    let xiaomi_description = xiaomi_default_description();
    for entry in xiaomi_models() {
        let model = format!("{XIAOMI_PROVIDER_ID}/{}", entry.slug);
        presets.push(third_party_preset(
            &model,
            &entry.display_name,
            entry
                .description
                .as_deref()
                .unwrap_or(&xiaomi_description),
        ));
    }

    presets.extend([
        ModelPreset {
            id: "gpt-5.3-codex".to_string(),
            model: "gpt-5.3-codex".to_string(),
            display_name: "gpt-5.3-codex".to_string(),
            description: "Latest frontier agentic coding model.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fast responses with lighter reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Balances speed and reasoning depth for everyday tasks".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex problems".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning depth for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: &[TextVerbosityConfig::Medium],
            is_default: false,
            upgrade: None,
            pro_only: false,
            show_in_picker: true,
        },
        ModelPreset {
            id: "gpt-5.3-codex-spark".to_string(),
            model: "gpt-5.3-codex-spark".to_string(),
            display_name: "gpt-5.3-codex-spark".to_string(),
            description: "Fast codex variant tuned for responsive coding loops and smaller edits."
                .to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fast responses with lighter reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Balances speed and reasoning depth for everyday tasks".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex problems".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning depth for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: &[TextVerbosityConfig::Medium],
            is_default: false,
            upgrade: None,
            pro_only: true,
            show_in_picker: true,
        },
        ModelPreset {
            id: "gpt-5.2-codex".to_string(),
            model: "gpt-5.2-codex".to_string(),
            display_name: "gpt-5.2-codex".to_string(),
            description: "Frontier agentic coding model.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fast responses with lighter reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Balances speed and reasoning depth for everyday tasks".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex problems".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning depth for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: &[TextVerbosityConfig::Medium],
            is_default: true,
            upgrade: Some(ModelUpgrade {
                id: "gpt-5.3-codex".to_string(),
                reasoning_effort_mapping: None,
                migration_config_key: HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG.to_string(),
            }),
            pro_only: false,
            show_in_picker: true,
        },
        ModelPreset {
            id: "gpt-5.2".to_string(),
            model: "gpt-5.2".to_string(),
            display_name: "gpt-5.2".to_string(),
            description:
                "Latest frontier model with improvements across knowledge, reasoning and coding"
                    .to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description:
                        "Balances speed with some reasoning; useful for straightforward queries and short explanations"
                            .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description:
                        "Provides a solid balance of reasoning depth and latency for general-purpose tasks"
                            .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex or ambiguous problems"
                        .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning depth for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: Some(ModelUpgrade {
                id: "gpt-5.3-codex".to_string(),
                reasoning_effort_mapping: None,
                migration_config_key: HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG.to_string(),
            }),
            pro_only: false,
            show_in_picker: true,
        },
        ModelPreset {
            id: "bengalfox".to_string(),
            model: "bengalfox".to_string(),
            display_name: "bengalfox".to_string(),
            description: "bengalfox".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fast responses with lighter reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Balances speed and reasoning depth for everyday tasks".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Greater reasoning depth for complex problems".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning depth for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: &[TextVerbosityConfig::Medium],
            is_default: false,
            upgrade: None,
            pro_only: false,
            show_in_picker: false,
        },
        ModelPreset {
            id: "boomslang".to_string(),
            model: "boomslang".to_string(),
            display_name: "boomslang".to_string(),
            description: "boomslang".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description:
                        "Balances speed with some reasoning; useful for straightforward queries and short explanations"
                            .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description:
                        "Provides a solid balance of reasoning depth and latency for general-purpose tasks"
                            .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex or ambiguous problems"
                        .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: None,
            pro_only: false,
            show_in_picker: false,
        },
        ModelPreset {
            id: "gpt-5.1-codex-max".to_string(),
            model: "gpt-5.1-codex-max".to_string(),
            display_name: "gpt-5.1-codex-max".to_string(),
            description: "Latest Codex-optimized flagship for deep and fast reasoning.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fast responses with lighter reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Balances speed and reasoning depth for everyday tasks".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex problems".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "Extra high reasoning depth for complex problems".to_string(),
                },
            ],
            supported_text_verbosity: &[TextVerbosityConfig::Medium],
            is_default: false,
            upgrade: Some(ModelUpgrade {
                id: "gpt-5.3-codex".to_string(),
                reasoning_effort_mapping: None,
                migration_config_key: HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG.to_string(),
            }),
            pro_only: false,
            show_in_picker: true,
        },
        ModelPreset {
            id: "gpt-5.1-codex".to_string(),
            model: "gpt-5.1-codex".to_string(),
            display_name: "gpt-5.1-codex".to_string(),
            description: "Optimized for Kay.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fastest responses with limited reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Dynamically adjusts reasoning based on the task".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex or ambiguous problems"
                        .to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: Some(ModelUpgrade {
                id: "gpt-5.3-codex".to_string(),
                reasoning_effort_mapping: None,
                migration_config_key: HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG.to_string(),
            }),
            pro_only: false,
            show_in_picker: false,
        },
        ModelPreset {
            id: "gpt-5.1-codex-mini".to_string(),
            model: "gpt-5.1-codex-mini".to_string(),
            display_name: "gpt-5.1-codex-mini".to_string(),
            description: "Optimized for Kay. Cheaper, faster, but less capable.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Dynamically adjusts reasoning based on the task".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex or ambiguous problems"
                        .to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: Some(ModelUpgrade {
                id: "gpt-5.3-codex".to_string(),
                reasoning_effort_mapping: None,
                migration_config_key: HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG.to_string(),
            }),
            pro_only: false,
            show_in_picker: true,
        },
        ModelPreset {
            id: "gpt-5.1".to_string(),
            model: "gpt-5.1".to_string(),
            display_name: "gpt-5.1".to_string(),
            description: "Broad world knowledge with strong general reasoning.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description:
                        "Balances speed with some reasoning; useful for straightforward queries and short explanations"
                            .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description:
                        "Provides a solid balance of reasoning depth and latency for general-purpose tasks"
                            .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex or ambiguous problems".to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: Some(ModelUpgrade {
                id: "gpt-5.3-codex".to_string(),
                reasoning_effort_mapping: None,
                migration_config_key: HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG.to_string(),
            }),
            pro_only: false,
            show_in_picker: false,
        },
        // Deprecated GPT-5 variants kept for migrations / config compatibility.
        ModelPreset {
            id: "gpt-5-codex".to_string(),
            model: "gpt-5-codex".to_string(),
            display_name: "gpt-5-codex".to_string(),
            description: "Optimized for Kay.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Fastest responses with limited reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Dynamically adjusts reasoning based on the task".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex or ambiguous problems".to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: Some(ModelUpgrade {
                id: "gpt-5.3-codex".to_string(),
                reasoning_effort_mapping: None,
                migration_config_key: HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG.to_string(),
            }),
            pro_only: false,
            show_in_picker: false,
        },
        ModelPreset {
            id: "gpt-5-codex-mini".to_string(),
            model: "gpt-5-codex-mini".to_string(),
            display_name: "gpt-5-codex-mini".to_string(),
            description: "Optimized for Kay. Cheaper, faster, but less capable.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "Dynamically adjusts reasoning based on the task".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex or ambiguous problems".to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: Some(ModelUpgrade {
                id: "gpt-5.3-codex".to_string(),
                reasoning_effort_mapping: None,
                migration_config_key: HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG.to_string(),
            }),
            pro_only: false,
            show_in_picker: false,
        },
        ModelPreset {
            id: "gpt-5".to_string(),
            model: "gpt-5".to_string(),
            display_name: "gpt-5".to_string(),
            description: "Broad world knowledge with strong general reasoning.".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Minimal,
                    description: "Fastest responses with little reasoning".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description:
                        "Balances speed with some reasoning; useful for straightforward queries and short explanations"
                            .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description:
                        "Provides a solid balance of reasoning depth and latency for general-purpose tasks"
                            .to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "Maximizes reasoning depth for complex or ambiguous problems".to_string(),
                },
            ],
            supported_text_verbosity: ALL_TEXT_VERBOSITY,
            is_default: false,
            upgrade: Some(ModelUpgrade {
                id: "gpt-5.3-codex".to_string(),
                reasoning_effort_mapping: None,
                migration_config_key: HIDE_GPT_5_2_CODEX_MIGRATION_PROMPT_CONFIG.to_string(),
            }),
            pro_only: false,
            show_in_picker: false,
        },
    ]);

    presets
});

pub fn model_preset_available_for_auth(
    preset: &ModelPreset,
    auth_mode: Option<AuthMode>,
    supports_pro_only_models: bool,
) -> bool {
    let is_chatgpt_auth = auth_mode.is_some_and(AuthMode::is_chatgpt);
    if preset.pro_only && !(is_chatgpt_auth && supports_pro_only_models) {
        return false;
    }

    match auth_mode {
        Some(AuthMode::ApiKey) => {
            preset.id != "gpt-5.2-codex" && preset.id != "gpt-5.3-codex"
        }
        _ => true,
    }
}

pub fn builtin_model_presets(
    auth_mode: Option<AuthMode>,
    supports_pro_only_models: bool,
) -> Vec<ModelPreset> {
    PRESETS
        .iter()
        .filter(|preset| {
            model_preset_available_for_auth(preset, auth_mode, supports_pro_only_models)
        })
        .filter(|preset| preset.show_in_picker)
        .cloned()
        .collect()
}

// todo(aibrahim): remove this once we migrate tests
pub fn all_model_presets() -> &'static Vec<ModelPreset> {
    &PRESETS
}

fn find_preset_for_model(model: &str) -> Option<&'static ModelPreset> {
    let model_lower = model.to_ascii_lowercase();

    PRESETS.iter().find(|preset| {
        preset.model.eq_ignore_ascii_case(&model_lower)
            || preset.id.eq_ignore_ascii_case(&model_lower)
            || preset.display_name.eq_ignore_ascii_case(&model_lower)
    })
}

fn reasoning_effort_rank(effort: ReasoningEffort) -> u8 {
    match effort {
        ReasoningEffort::None => 0,
        ReasoningEffort::Minimal => 0,
        ReasoningEffort::Low => 1,
        ReasoningEffort::Medium => 2,
        ReasoningEffort::High => 3,
        ReasoningEffort::XHigh => 4,
    }
}

pub fn clamp_reasoning_effort_for_model(
    model: &str,
    requested: ReasoningEffort,
) -> ReasoningEffort {
    let Some(preset) = find_preset_for_model(model) else {
        return requested;
    };

    if preset
        .supported_reasoning_efforts
        .iter()
        .any(|opt| opt.effort == requested)
    {
        return requested;
    }

    let requested_rank = reasoning_effort_rank(requested);

    preset
        .supported_reasoning_efforts
        .iter()
        .min_by_key(|opt| {
            let rank = reasoning_effort_rank(opt.effort);
            (requested_rank.abs_diff(rank), u8::MAX - rank)
        })
        .map(|opt| opt.effort)
        .unwrap_or(requested)
}

pub fn allowed_text_verbosity_for_model(model: &str) -> &'static [TextVerbosityConfig] {
    find_preset_for_model(model)
        .map(|preset| preset.supported_text_verbosity)
        .unwrap_or(ALL_TEXT_VERBOSITY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_default_model_is_configured() {
        assert_eq!(PRESETS.iter().filter(|preset| preset.is_default).count(), 1);
    }

    #[test]
    fn gpt_5_codex_hidden_for_api_key_auth() {
        let presets = builtin_model_presets(Some(AuthMode::ApiKey), false);
        assert!(presets.iter().all(|preset| {
            preset.id != "gpt-5.2-codex"
                && preset.id != "gpt-5.3-codex"
                && preset.id != "gpt-5.3-codex-spark"
        }));
    }

    #[test]
    fn spark_hidden_for_non_pro_chatgpt_auth() {
        let presets = builtin_model_presets(Some(AuthMode::Chatgpt), false);
        assert!(
            !presets
                .iter()
                .any(|preset| preset.id == "gpt-5.3-codex-spark")
        );
    }

    #[test]
    fn spark_available_for_pro_chatgpt_auth() {
        let presets = builtin_model_presets(Some(AuthMode::Chatgpt), true);
        assert!(
            presets
                .iter()
                .any(|preset| preset.id == "gpt-5.3-codex-spark")
        );
    }

    #[test]
    fn gpt_5_4_available_for_api_key_auth() {
        let presets = builtin_model_presets(Some(AuthMode::ApiKey), false);
        assert!(presets.iter().any(|preset| preset.id == "gpt-5.4"));
        assert!(presets.iter().any(|preset| preset.id == "gpt-5.4-mini"));
    }

    #[test]
    fn minimax_available_for_api_key_auth() {
        let presets = builtin_model_presets(Some(AuthMode::ApiKey), false);
        assert!(presets.iter().any(|preset| preset.id == "MiniMax-M3"));
        assert!(presets.iter().any(|preset| preset.id == "MiniMax-M2.7"));
    }

    #[test]
    fn opencode_go_models_available_for_api_key_auth() {
        let presets = builtin_model_presets(Some(AuthMode::ApiKey), false);
        for model in [
            "opencode-go/glm-5.1",
            "opencode-go/glm-5.2",
            "opencode-go/glm-5",
            "opencode-go/kimi-k2.7-code",
            "opencode-go/kimi-k2.6",
            "opencode-go/minimax-m3",
            "opencode-go/qwen3.7-plus",
            "opencode-go/deepseek-v4-flash",
        ] {
            assert!(
                presets.iter().any(|preset| preset.id == model),
                "missing preset {model}"
            );
        }
    }

    #[test]
    fn xiaomi_models_available_for_api_key_auth() {
        let presets = builtin_model_presets(Some(AuthMode::ApiKey), false);
        assert!(
            presets
                .iter()
                .any(|preset| preset.id == "xiaomi/mimo-v2.5-pro")
        );
        assert!(
            presets
                .iter()
                .any(|preset| preset.id == "xiaomi/mimo-v2.5")
        );
    }

    #[test]
    fn third_party_presets_do_not_advertise_openai_reasoning_effort() {
        let presets = builtin_model_presets(Some(AuthMode::ApiKey), false);
        for model in [
            "MiniMax-M3",
            "MiniMax-M2.7",
            "xiaomi/mimo-v2.5-pro",
            "xiaomi/mimo-v2.5",
            "opencode-go/glm-5.1",
            "opencode-go/kimi-k2.6",
            "opencode-go/deepseek-v4-flash",
        ] {
            let preset = presets
                .iter()
                .find(|preset| preset.id == model)
                .unwrap_or_else(|| panic!("missing preset {model}"));
            assert!(
                preset.supported_reasoning_efforts.is_empty(),
                "{model} should not show a reasoning-effort selector"
            );
        }
    }

    #[test]
    fn gpt_5_5_available_for_chatgpt_auth() {
        let presets = builtin_model_presets(Some(AuthMode::Chatgpt), true);
        assert!(presets.iter().any(|preset| preset.id == "gpt-5.5"));
    }

    #[test]
    fn clamp_reasoning_effort_downgrades_to_supported_level() {
        let clamped = clamp_reasoning_effort_for_model(
            "gpt-5.1-codex",
            ReasoningEffort::XHigh,
        );
        assert_eq!(clamped, ReasoningEffort::High);

        let clamped_minimal =
            clamp_reasoning_effort_for_model("gpt-5.1-codex-mini", ReasoningEffort::Minimal);
        assert_eq!(clamped_minimal, ReasoningEffort::Medium);
    }
}
