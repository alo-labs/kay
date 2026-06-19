use crate::config_types::Personality;
use crate::config_types::ContextMode;
use crate::config_types::ReasoningEffort;
use crate::config_types::ReasoningSummary;
use crate::MINIMAX_PROVIDER_ID;
use crate::OPENCODE_GO_PROVIDER_ID;
use crate::XIAOMI_PROVIDER_ID;
use crate::tool_apply_patch::ApplyPatchToolType;
use code_protocol::openai_models::ConfigShellToolType;
use code_protocol::openai_models::InputModality;
use code_protocol::openai_models::ModelInfo;
use code_protocol::openai_models::ModelsResponse;
use code_protocol::openai_models::TruncationMode;
use code_protocol::openai_models::WebSearchToolType;
use code_protocol::protocol::TruncationPolicy;
use once_cell::sync::Lazy;
use std::borrow::Cow;

/// The `instructions` field in the payload sent to a model should always start
/// with this content.
const BASE_INSTRUCTIONS: &str = include_str!("../prompt.md");
const BASE_INSTRUCTIONS_WITH_APPLY_PATCH: &str =
    include_str!("../prompt_with_apply_patch_instructions.md");
const GPT_5_CODEX_INSTRUCTIONS: &str = include_str!("../gpt_5_codex_prompt.md");
const GPT_5_1_INSTRUCTIONS: &str = include_str!("../gpt_5_1_prompt.md");
const GPT_5_2_INSTRUCTIONS: &str = include_str!("../gpt_5_2_prompt.md");
const GPT_5_1_CODEX_MAX_INSTRUCTIONS: &str = include_str!("../gpt-5.1-codex-max_prompt.md");
const GPT_5_2_CODEX_INSTRUCTIONS: &str = include_str!("../gpt-5.2-codex_prompt.md");
const MIMO_SYNTHESIS_CHECKPOINT_INSTRUCTIONS: &str = r#"MiMo investigation discipline:
- Do not stay in private reasoning when the next step is obvious. If the user names files to inspect, call the shell tool immediately and read those files in one bounded command.
- When calling a tool, emit exactly one JSON object for that tool call. Do not concatenate multiple JSON objects into a single tool arguments string.
- For `apply_patch`, use either a bare `@@` hunk marker or a single descriptive hunk header without a trailing `@@`. Never write hunk labels like `@@ hint paragraph @@`; the trailing `@@` becomes bad context. Include exact unchanged context lines from the file around every edit.
- If two `apply_patch` attempts fail on the same file because context does not match, stop retrying that patch shape. Re-read the target lines once, then use a smaller patch with exact nearby context or a bounded script that rewrites only the required file. After a third patch failure on the same file, do not call `apply_patch` again for that file.
- When the user gives a strict output contract or JSON schema, use assistant messages only for the final contracted output after tool work is complete.
- In tool workflows, do not use assistant messages for progress updates; continue with tool calls until you are ready to provide the final answer.
- Keep the first investigation action small and concrete; gather the minimum evidence needed before thinking further.
- Consolidate findings before ending an investigation turn.
- If you have read or searched the same files repeatedly, stop rereading and summarize what is already known.
- Before taking another exploratory tool action, state the current hypothesis, the evidence for it, and the single next observation that would change it.
- When enough evidence has been gathered, provide the diagnosis or next concrete code change instead of another preamble.
- In `scripts/verify-*.sh`, default the port with `PORT="${PORT:-3458}"` and export it before starting the app; never use `PORT="${PORT:?PORT is required}"`.
- For source files and verify scripts, use `apply_patch` Add/Update File hunks — never `printf`, `cat >`, or `&&`-chained shell to write files.
- When the user prompt names local verify scripts or a final STATUS block, create or update files with `apply_patch`, run named verification commands with any required environment exported, then honor that STATUS contract in the final reply."#;
const QWEN_TOOL_DISCIPLINE_INSTRUCTIONS: &str = r#"Qwen tool discipline:
- Call `apply_patch` with exactly one argument: the full patch string from `*** Begin Patch` through `*** End Patch`.
- Do not pass patch lines as separate shell arguments and do not insert `&&` between argv tokens.
- Pass shell work as one `bash -lc '...'` string; Kay joins argv arrays with `&&`, which breaks `apply_patch` and pipes.
- Prefer `bash -lc "apply_patch <<'PATCH'\n*** Begin Patch\n...\n*** End Patch\nPATCH"` over `cat file | apply_patch`.
- On macOS, use `cat -n`, not `cat -An` or `cat -A`.
- Prefer the `apply_patch` tool or a heredoc for file edits instead of empty redirections like `cat > /tmp/file` without content.
- In `scripts/verify-*.sh`, default the port with `PORT="${PORT:-3458}"` and export it before starting the app; never use `PORT="${PORT:?PORT is required}"`.
- For source files and verify scripts, use `apply_patch` Add/Update File hunks — never `printf`, `cat >`, or `&&`-chained shell to write files.
- When the user prompt names local verify scripts or a final STATUS block, create or update files with `apply_patch`, run named verification commands with any required environment exported, then honor that STATUS contract in the final reply."#;
const MINIMAX_TOOL_DISCIPLINE_INSTRUCTIONS: &str = r#"MiniMax tool discipline:
- Call `apply_patch` with exactly one argument: the full patch string from `*** Begin Patch` through `*** End Patch`.
- Do not pass patch lines as separate shell arguments and do not insert `&&` between argv tokens.
- Pass shell work as one `bash -lc '...'` string; Kay joins argv arrays with `&&`, which breaks `apply_patch` and pipes.
- Prefer `bash -lc "apply_patch <<'PATCH'\n*** Begin Patch\n...\n*** End Patch\nPATCH"` over `cat file | apply_patch`.
- On macOS, use `cat -n`, not `cat -An` or `cat -A`.
- Prefer the `apply_patch` tool or a heredoc for file edits instead of empty redirections like `cat > /tmp/file` without content.
- In `scripts/verify-*.sh`, default the port with `PORT="${PORT:-3458}"` and export it before starting the app; never use `PORT="${PORT:?PORT is required}"`.
- For source files and verify scripts, use `apply_patch` Add/Update File hunks — never `printf`, `cat >`, or `&&`-chained shell to write files.
- When the user prompt names local verify scripts or a final STATUS block, create or update files with `apply_patch`, run named verification commands with any required environment exported, then honor that STATUS contract in the final reply."#;
const DEFAULT_PERSONALITY_HEADER: &str = "You are Codex, a coding agent based on GPT-5. You and the user share the same workspace and collaborate to achieve the user's goals.";
const LOCAL_FRIENDLY_TEMPLATE: &str =
    "You optimize for team morale and being a supportive teammate as much as code quality.";
const LOCAL_PRAGMATIC_TEMPLATE: &str = "You are a deeply pragmatic, effective software engineer.";

const CONTEXT_WINDOW_272K: u64 = 272_000;
const CONTEXT_WINDOW_200K: u64 = 200_000;
const CONTEXT_WINDOW_128K: u64 = 128_000;
const CONTEXT_WINDOW_96K: u64 = 96_000;
const CONTEXT_WINDOW_16K: u64 = 16_385;
const CONTEXT_WINDOW_1M: u64 = 1_047_576;
/// Round 1M context limits used by several third-party models on OpenCode Go.
const CONTEXT_WINDOW_1M_ROUND: u64 = 1_000_000;
const CONTEXT_WINDOW_202_752: u64 = 202_752;
const CONTEXT_WINDOW_262_144: u64 = 262_144;
const CONTEXT_WINDOW_MIMO_PRO: u64 = 1_048_576;
const CONTEXT_WINDOW_MINIMAX_M2_7: u64 = 204_800;
const MAX_OUTPUT_DEFAULT: u64 = 128_000;

/// Context window for supported third-party slugs, aligned with models.dev OpenCode Go
/// entries and MiniMax direct API specs (M3 = 1M).
fn context_window_for_third_party_slug(slug_lower: &str) -> Option<u64> {
    if slug_lower.contains("mimo-v2.5-pro") {
        return Some(CONTEXT_WINDOW_MIMO_PRO);
    }
    if slug_lower.contains("minimax-m3") {
        return Some(CONTEXT_WINDOW_1M_ROUND);
    }
    if slug_lower.contains("minimax-m2.7") {
        return Some(CONTEXT_WINDOW_MINIMAX_M2_7);
    }
    if slug_lower.contains("glm-5.2") {
        return Some(CONTEXT_WINDOW_1M_ROUND);
    }
    if slug_lower.contains("glm-5.1")
        || slug_lower.ends_with("glm-5")
        || slug_lower.ends_with("/glm-5")
    {
        return Some(CONTEXT_WINDOW_202_752);
    }
    if slug_lower.starts_with("kimi") {
        return Some(CONTEXT_WINDOW_262_144);
    }
    if slug_lower.contains("mimo-v2.5") {
        return Some(CONTEXT_WINDOW_1M_ROUND);
    }
    if slug_lower.starts_with("deepseek-v4") {
        return Some(CONTEXT_WINDOW_1M_ROUND);
    }
    if slug_lower.starts_with("qwen3.7") || slug_lower.contains("qwen3.6-plus") {
        return Some(CONTEXT_WINDOW_1M_ROUND);
    }
    if slug_lower.starts_with("qwen") {
        return Some(CONTEXT_WINDOW_262_144);
    }
    None
}

/// How chat-completions requests should serialize instruction-bearing roles
/// for a given model family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChatCompletionsRoleStrategy {
    /// Preserve the standard OpenAI developer/system/user/tool role mix.
    #[default]
    OpenAi,

    /// Normalize developer-like roles to `system` so providers that reject
    /// the `developer` role can still receive the same instructions.
    CollapseNonChatRolesToSystem,
}

/// How chat-completions requests should preserve reasoning content for a
/// given model family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChatCompletionsReasoningStrategy {
    /// Preserve the standard OpenAI behavior.
    #[default]
    OpenAi,

    /// Include reasoning content explicitly on assistant/tool-call messages so
    /// providers that require it can replay it across turns.
    PreserveReasoningContent,
}

static UPSTREAM_MODELS: Lazy<Vec<ModelInfo>> = Lazy::new(|| {
    serde_json::from_str::<ModelsResponse>(include_str!("../../../codex-rs/models-manager/models.json"))
        .map(|response| response.models)
        .unwrap_or_else(|err| panic!("failed to parse upstream models.json: {err}"))
});

fn namespaced_model_suffix(model: &str) -> Option<&str> {
    let (namespace, suffix) = model.split_once('/')?;
    if namespace.is_empty() || suffix.is_empty() {
        return None;
    }
    if suffix.contains('/') {
        return None;
    }
    Some(suffix)
}

pub fn provider_model_slug<'a>(provider_id: &str, model_slug: &'a str) -> Cow<'a, str> {
    if let Some((namespace, suffix)) = model_slug.split_once('/')
        && namespace.eq_ignore_ascii_case(provider_id)
        && !suffix.is_empty()
        && !suffix.contains('/')
    {
        return Cow::Borrowed(suffix);
    }

    Cow::Borrowed(model_slug)
}

/// Canonicalize provider-facing model slugs for wire requests.
pub fn wire_model_slug(provider_id: &str, model_slug: &str) -> String {
    let stripped = provider_model_slug(provider_id, model_slug);
    let canonical = stripped.as_ref();
    if provider_id == OPENCODE_GO_PROVIDER_ID || provider_id == MINIMAX_PROVIDER_ID {
        if canonical.eq_ignore_ascii_case("MiniMax-M3") {
            return "minimax-m3".to_string();
        }
    }
    canonical.to_string()
}

/// Infer the provider id from a model slug when the slug clearly belongs to a
/// non-OpenAI provider.
///
/// OpenAI-compatible GPT model slugs still normalize to `openai`, but callers
/// that only care about third-party providers can filter that value out.
pub fn infer_model_provider_id(model: &str) -> Option<&'static str> {
    let model = model.trim();
    if model.is_empty() {
        return None;
    }

    if model.eq_ignore_ascii_case("MiniMax-M2.7")
        || model.eq_ignore_ascii_case("MiniMax-M3")
        || provider_model_slug(MINIMAX_PROVIDER_ID, model).as_ref() != model
    {
        return Some(MINIMAX_PROVIDER_ID);
    }

    if provider_model_slug(OPENCODE_GO_PROVIDER_ID, model).as_ref() != model {
        return Some(OPENCODE_GO_PROVIDER_ID);
    }

    if provider_model_slug(XIAOMI_PROVIDER_ID, model).as_ref() != model {
        return Some(XIAOMI_PROVIDER_ID);
    }

    let normalized = provider_model_slug("openai", model)
        .as_ref()
        .trim()
        .to_ascii_lowercase();
    if normalized.starts_with("gpt-") && !normalized.starts_with("gpt-oss") {
        return Some("openai");
    }

    None
}

fn normalized_model_matches_request(requested_model: &str, response_model: &str) -> bool {
    if response_model == requested_model {
        return true;
    }

    response_model.strip_prefix(requested_model).is_some_and(|suffix| {
        suffix
            .strip_prefix('-')
            .and_then(|version| version.chars().next())
            .is_some_and(|first| first.is_ascii_digit())
    })
}

fn owner_prefixed_model_slug(model: &str) -> Option<&str> {
    let (_, suffix) = model.split_once('/')?;
    if suffix.is_empty() || suffix.contains('/') {
        return None;
    }
    Some(suffix)
}

/// OpenCode Go sometimes returns Fireworks-hosted model paths such as
/// `accounts/fireworks/models/glm-5p2` where dotted versions use `p`.
fn opencode_go_fireworks_slug_matches(requested_slug: &str, response_model: &str) -> bool {
    let response = response_model.trim().to_ascii_lowercase();
    if !response.contains("fireworks/") {
        return false;
    }
    let Some(tail) = response.rsplit('/').next() else {
        return false;
    };
    let requested = requested_slug.trim().to_ascii_lowercase();
    if requested == tail {
        return true;
    }
    requested.replace('.', "p") == tail || tail.replace('p', ".") == requested
}

/// Returns true when the provider response model is equivalent to the requested
/// model slug.
///
/// Some OpenAI-compatible third-party providers return the upstream model slug
/// (`glm-5.1`) even when Kay requested the namespaced provider slug
/// (`opencode-go/glm-5.1`). That is expected normalization, not model
/// rerouting.
pub fn response_model_matches_request(requested_model: &str, response_model: &str) -> bool {
    let requested = requested_model.trim().to_ascii_lowercase();
    let response = response_model.trim().to_ascii_lowercase();

    if normalized_model_matches_request(&requested, &response) {
        return true;
    }

    let provider_ids = [
        infer_model_provider_id(&requested),
        infer_model_provider_id(&response),
    ];

    for provider_id in provider_ids.into_iter().flatten() {
        let requested_slug = provider_model_slug(provider_id, &requested);
        let response_slug = provider_model_slug(provider_id, &response);
        if normalized_model_matches_request(requested_slug.as_ref(), response_slug.as_ref()) {
            return true;
        }

        if owner_prefixed_model_slug(response_slug.as_ref()).is_some_and(|response_slug| {
            normalized_model_matches_request(requested_slug.as_ref(), response_slug)
        }) {
            return true;
        }

        if opencode_go_fireworks_slug_matches(requested_slug.as_ref(), &response) {
            return true;
        }
    }

    if let Some(response_slug) = owner_prefixed_model_slug(&response)
        && normalized_model_matches_request(&requested, response_slug)
    {
        return true;
    }

    if let Some(requested_slug) = owner_prefixed_model_slug(&requested)
        && normalized_model_matches_request(requested_slug, &response)
    {
        return true;
    }

    false
}

pub const STANDARD_CONTEXT_WINDOW_272K: u64 = CONTEXT_WINDOW_272K;
pub const EXTENDED_CONTEXT_WINDOW_1M: u64 = CONTEXT_WINDOW_1M;

/// A model family is a group of models that share certain characteristics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelFamily {
    /// The full model slug used to derive this model family, e.g.
    /// "gpt-4.1-2025-04-14".
    pub slug: String,

    /// The model family name, e.g. "gpt-4.1".
    pub family: String,

    /// True if the model needs additional instructions on how to use the
    /// "virtual" `apply_patch` CLI.
    pub needs_special_apply_patch_instructions: bool,

    /// Maximum supported context window, if known.
    pub context_window: Option<u64>,

    /// Maximum number of output tokens that can be generated for the model.
    pub max_output_tokens: Option<u64>,

    /// Truncation policy to apply when recording tool outputs in the model context.
    pub truncation_policy: TruncationPolicy,

    /// Token threshold where we should automatically compact history.
    auto_compact_token_limit: Option<i64>,

    // Whether the `reasoning` field can be set when making a request to this
    // model family. Note it has `effort` and `summary` subfields (though
    // `summary` is optional).
    pub supports_reasoning_summaries: bool,

    /// The reasoning effort to use for this model family when none is explicitly chosen.
    pub default_reasoning_effort: Option<ReasoningEffort>,

    /// The reasoning summary setting to use when requests don't override it.
    pub default_reasoning_summary: ReasoningSummary,

    /// Whether this model supports parallel tool calls when using the
    /// Responses API.
    pub supports_parallel_tool_calls: bool,

    /// Additional speed tiers advertised by the backend for this model.
    pub additional_speed_tiers: Vec<String>,

    /// Whether the backend says this model supports the native search tool.
    pub supports_search_tool: bool,

    /// Prefer websocket transport for this model when supported by the provider.
    pub prefer_websockets: bool,

    // This should be set to true when the model expects a tool named
    // "local_shell" to be provided. Its contract must be understood natively by
    // the model such that its description can be omitted.
    // See https://platform.openai.com/docs/guides/tools-local-shell
    pub uses_local_shell_tool: bool,

    /// Present if the model performs better when `apply_patch` is provided as
    /// a tool call instead of just a bash command
    pub apply_patch_tool_type: Option<ApplyPatchToolType>,

    /// Route malformed `apply_patch` FunctionCall / CustomToolCall items through
    /// patch normalization instead of rejecting them as unsupported tool calls.
    pub repairs_malformed_apply_patch_tool_calls: bool,

    /// Retry turns when the final assistant message does not satisfy
    /// `final_output_json_schema` on the turn context.
    pub repairs_final_output_json_schema: bool,

    /// This should be set when the model expects a `shell_command` tool that
    /// accepts a shell script string instead of argv-style arguments.
    pub uses_shell_command_tool: bool,

    /// Whether web_search should request text-only or multimodal results.
    pub web_search_tool_type: WebSearchToolType,

    /// Whether responses can use `detail: "original"` for tool-returned images.
    pub supports_image_detail_original: bool,

    /// Whether this model supports image generation via the native Responses tool.
    pub supports_image_generation: bool,

    /// Chat-completions role handling for model families that need a
    /// compatibility profile beyond the provider's default format.
    pub chat_completions_role_strategy: ChatCompletionsRoleStrategy,

    /// Chat-completions reasoning handling for model families that require
    /// reasoning content to be replayed explicitly.
    pub chat_completions_reasoning_strategy: ChatCompletionsReasoningStrategy,

    /// Whether Chat Completions requests may use native
    /// `response_format: json_schema` for structured final output.
    pub supports_chat_completions_response_format_json_schema: bool,

    // Instructions to use for querying the model
    pub base_instructions: String,
}

pub(crate) fn base_instructions_override_for_personality(
    model: &str,
    personality: Option<Personality>,
) -> Option<String> {
    if !(model.starts_with("gpt-5.2-codex")
        || model.starts_with("gpt-5.3-codex")
        || model.starts_with("bengalfox")
        || model.starts_with("exp-codex")
        || model.starts_with("codex-1p"))
    {
        return None;
    }
    let personality_message = match personality {
        Some(Personality::None) => "",
        Some(Personality::Friendly) => LOCAL_FRIENDLY_TEMPLATE,
        Some(Personality::Pragmatic) => LOCAL_PRAGMATIC_TEMPLATE,
        None => "",
    };
    Some(format!(
        "{DEFAULT_PERSONALITY_HEADER}\n\n{personality_message}\n\n{BASE_INSTRUCTIONS}"
    ))
}

macro_rules! model_family {
    (
        $slug:expr, $family:expr $(, $key:ident : $value:expr )* $(,)?
    ) => {{
        let slug_value = $slug;
        // defaults
        let mut mf = ModelFamily {
            slug: slug_value.to_string(),
            family: $family.to_string(),
            needs_special_apply_patch_instructions: false,
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Bytes(10_000),
            auto_compact_token_limit: None,
            supports_reasoning_summaries: false,
            default_reasoning_effort: None,
            default_reasoning_summary: ReasoningSummary::Auto,
            supports_parallel_tool_calls: false,
            additional_speed_tiers: Vec::new(),
            supports_search_tool: false,
            prefer_websockets: false,
            uses_local_shell_tool: false,
            apply_patch_tool_type: None,
            repairs_malformed_apply_patch_tool_calls: false,
            repairs_final_output_json_schema: false,
            uses_shell_command_tool: false,
            web_search_tool_type: WebSearchToolType::Text,
            supports_image_detail_original: false,
            supports_image_generation: false,
            chat_completions_role_strategy: ChatCompletionsRoleStrategy::OpenAi,
            chat_completions_reasoning_strategy: ChatCompletionsReasoningStrategy::OpenAi,
            supports_chat_completions_response_format_json_schema: true,
            base_instructions: BASE_INSTRUCTIONS.to_string(),
        };
        // apply overrides
        $(
            mf.$key = $value;
        )*
        Some(apply_upstream_model_overrides(mf))
    }};
}

fn apply_upstream_model_overrides(mut family: ModelFamily) -> ModelFamily {
    let model_slug = family
        .slug
        .strip_prefix("openai/")
        .or_else(|| namespaced_model_suffix(&family.slug))
        .unwrap_or(&family.slug);
    let Some(model_info) = UPSTREAM_MODELS.iter().find(|model| model.slug == model_slug) else {
        return family;
    };

    family.base_instructions = model_info.base_instructions.clone();
    family.context_window = model_info
        .resolved_context_window()
        .and_then(|limit| u64::try_from(limit).ok());
    family.default_reasoning_effort = model_info.default_reasoning_level.map(|effort| match effort {
        code_protocol::openai_models::ReasoningEffort::None
        | code_protocol::openai_models::ReasoningEffort::Minimal => ReasoningEffort::Minimal,
        code_protocol::openai_models::ReasoningEffort::Low => ReasoningEffort::Low,
        code_protocol::openai_models::ReasoningEffort::Medium => ReasoningEffort::Medium,
        code_protocol::openai_models::ReasoningEffort::High => ReasoningEffort::High,
        code_protocol::openai_models::ReasoningEffort::XHigh => ReasoningEffort::XHigh,
    });
    family.default_reasoning_summary = model_info.default_reasoning_summary.into();
    family.supports_reasoning_summaries = model_info.supports_reasoning_summaries;
    family.supports_parallel_tool_calls = model_info.supports_parallel_tool_calls;
    if let Some(tool_type) = model_info.apply_patch_tool_type.as_ref() {
        family.apply_patch_tool_type = Some(match tool_type {
            code_protocol::openai_models::ApplyPatchToolType::Freeform => {
                ApplyPatchToolType::Freeform
            }
            code_protocol::openai_models::ApplyPatchToolType::Function => ApplyPatchToolType::Function,
        });
    }
    family.web_search_tool_type = model_info.web_search_tool_type;
    family.supports_search_tool = model_info.supports_search_tool;
    family.additional_speed_tiers = model_info.additional_speed_tiers.clone();
    family.prefer_websockets = model_info.prefer_websockets;
    family.supports_image_detail_original = model_info.supports_image_detail_original;
    family.supports_image_generation = supports_image_generation(model_info);
    family.uses_local_shell_tool = matches!(model_info.shell_type, ConfigShellToolType::Local);
    family.uses_shell_command_tool =
        matches!(model_info.shell_type, ConfigShellToolType::ShellCommand);
    family.auto_compact_token_limit = model_info.auto_compact_token_limit();
    family.truncation_policy = match model_info.truncation_policy.mode {
        TruncationMode::Bytes => TruncationPolicy::Bytes(
            usize::try_from(model_info.truncation_policy.limit).unwrap_or(10_000),
        ),
        TruncationMode::Tokens => TruncationPolicy::Tokens(
            usize::try_from(model_info.truncation_policy.limit).unwrap_or(10_000),
        ),
    };

    family
}

fn with_mimo_synthesis_checkpoint(mut family: ModelFamily) -> ModelFamily {
    family.repairs_malformed_apply_patch_tool_calls = true;
    family.repairs_final_output_json_schema = true;
    if !family
        .base_instructions
        .contains("Consolidate findings before ending an investigation turn")
    {
        family.base_instructions.push_str("\n\n");
        family
            .base_instructions
            .push_str(MIMO_SYNTHESIS_CHECKPOINT_INSTRUCTIONS);
    }
    family
}

fn with_qwen_tool_discipline(mut family: ModelFamily) -> ModelFamily {
    family.repairs_malformed_apply_patch_tool_calls = true;
    if !family.base_instructions.contains("Qwen tool discipline") {
        family.base_instructions.push_str("\n\n");
        family
            .base_instructions
            .push_str(QWEN_TOOL_DISCIPLINE_INSTRUCTIONS);
    }
    family
}

fn with_minimax_tool_discipline(mut family: ModelFamily) -> ModelFamily {
    family.repairs_malformed_apply_patch_tool_calls = true;
    if !family
        .base_instructions
        .contains("MiniMax tool discipline")
    {
        family.base_instructions.push_str("\n\n");
        family
            .base_instructions
            .push_str(MINIMAX_TOOL_DISCIPLINE_INSTRUCTIONS);
    }
    family
}

/// Returns a `ModelFamily` for the given model slug, or `None` if the slug
/// does not match any known model family.
pub fn find_family_for_model(slug: &str) -> Option<ModelFamily> {
    if let Some(suffix) = namespaced_model_suffix(slug)
        && let Some(mut family) = find_family_for_model(suffix)
    {
        family.slug = slug.to_string();
        return Some(family);
    }

    let slug_lower = slug.to_ascii_lowercase();
    if matches!(
        slug_lower.as_str(),
        "minimax-m3" | "codex-minimax-m3"
    ) {
        model_family!(
            slug, "minimax-m3",
            needs_special_apply_patch_instructions: true,
            repairs_malformed_apply_patch_tool_calls: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: Some(CONTEXT_WINDOW_1M_ROUND),
            truncation_policy: TruncationPolicy::Tokens(10_000),
        )
        .map(with_minimax_tool_discipline)
    } else if matches!(
        slug_lower.as_str(),
        "minimax-m2.7" | "codex-minimax-m2.7"
    ) {
        model_family!(
            slug, "minimax-m2.7",
            needs_special_apply_patch_instructions: true,
            repairs_malformed_apply_patch_tool_calls: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: Some(CONTEXT_WINDOW_MINIMAX_M2_7),
            truncation_policy: TruncationPolicy::Tokens(10_000),
        )
        .map(with_minimax_tool_discipline)
    } else if slug.starts_with("o3") {
        model_family!(
            slug, "o3",
            supports_reasoning_summaries: true,
            needs_special_apply_patch_instructions: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: Some(CONTEXT_WINDOW_200K),
            max_output_tokens: Some(100_000),
        )
    } else if slug.starts_with("o4-mini") {
        model_family!(
            slug, "o4-mini",
            supports_reasoning_summaries: true,
            needs_special_apply_patch_instructions: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: Some(CONTEXT_WINDOW_200K),
            max_output_tokens: Some(100_000),
        )
    } else if slug.starts_with("codex-mini-latest") {
        model_family!(
            slug, "codex-mini-latest",
            supports_reasoning_summaries: true,
            uses_local_shell_tool: true,
            needs_special_apply_patch_instructions: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: Some(CONTEXT_WINDOW_200K),
            max_output_tokens: Some(100_000),
        )
    } else if slug.starts_with("gpt-4.1") {
        model_family!(
            slug, "gpt-4.1",
            needs_special_apply_patch_instructions: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: Some(CONTEXT_WINDOW_1M),
            max_output_tokens: Some(32_768),
        )
    } else if slug.starts_with("gpt-oss") || slug.starts_with("openai/gpt-oss") {
        model_family!(slug, "gpt-oss", apply_patch_tool_type: Some(ApplyPatchToolType::Function),
            uses_local_shell_tool: true,
            context_window: Some(CONTEXT_WINDOW_96K),
            max_output_tokens: Some(32_000))
    } else if slug.starts_with("gpt-4o") {
        model_family!(slug, "gpt-4o", needs_special_apply_patch_instructions: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: Some(CONTEXT_WINDOW_128K),
            max_output_tokens: Some(16_384))
    } else if slug.starts_with("gpt-3.5") {
        model_family!(slug, "gpt-3.5", needs_special_apply_patch_instructions: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: Some(CONTEXT_WINDOW_16K),
            max_output_tokens: Some(4_096))
    } else if slug.starts_with("test-gpt-5") {
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_CODEX_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            supports_parallel_tool_calls: true,
            default_reasoning_effort: Some(ReasoningEffort::Medium),
            truncation_policy: TruncationPolicy::Tokens(10_000),
        )
    } else if slug.starts_with("exp-codex") || slug.starts_with("codex-1p") {
        // Same defaults as gpt-5.2-codex.
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_2_CODEX_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            supports_parallel_tool_calls: true,
            truncation_policy: TruncationPolicy::Tokens(10_000),
        )
    } else if slug.starts_with("exp-") {
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            supports_parallel_tool_calls: true,
            default_reasoning_effort: Some(ReasoningEffort::Medium),
            truncation_policy: TruncationPolicy::Bytes(10_000),
        )
    } else if slug.starts_with("gpt-5.1-codex-max") {
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_1_CODEX_MAX_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Tokens(10_000),
        )
    } else if slug.starts_with("codex-")
        || slug.starts_with("gpt-5-codex")
        || slug.starts_with("gpt-5.1-codex")
    {
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_CODEX_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Tokens(10_000),
        )
    } else if slug.starts_with("gpt-5.2-codex") {
        // Same defaults as gpt-5.1-codex-max.
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_2_CODEX_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            supports_parallel_tool_calls: true,
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Tokens(10_000),
        )
    } else if slug.starts_with("gpt-5.3-codex") {
        // Same defaults as gpt-5.2-codex.
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_2_CODEX_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            supports_parallel_tool_calls: true,
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Tokens(10_000),
        )
    } else if slug.starts_with("bengalfox") {
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_2_CODEX_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            supports_parallel_tool_calls: true,
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Tokens(10_000),
        )
    } else if slug.starts_with("gpt-5.3") {
        model_family!(
            slug, "gpt-5.3",
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_2_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            default_reasoning_effort: Some(ReasoningEffort::Medium),
            supports_parallel_tool_calls: true,
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Bytes(10_000),
        )
    } else if slug.starts_with("gpt-5.2") {
        model_family!(
            slug, "gpt-5.2",
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_2_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            default_reasoning_effort: Some(ReasoningEffort::Medium),
            supports_parallel_tool_calls: true,
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Bytes(10_000),
        )
    } else if slug.starts_with("boomslang") {
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_2_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            default_reasoning_effort: Some(ReasoningEffort::Medium),
            supports_parallel_tool_calls: true,
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Bytes(10_000),
        )
    } else if slug.starts_with("gpt-5.1") {
        model_family!(
            slug, "gpt-5.1",
            supports_reasoning_summaries: true,
            base_instructions: GPT_5_1_INSTRUCTIONS.to_string(),
            apply_patch_tool_type: Some(ApplyPatchToolType::Freeform),
            default_reasoning_effort: Some(ReasoningEffort::Medium),
            supports_parallel_tool_calls: true,
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Bytes(10_000),
        )
    } else if slug.starts_with("gpt-5") {
        model_family!(
            slug, "gpt-5",
            supports_reasoning_summaries: true,
            base_instructions: BASE_INSTRUCTIONS.to_string(),
            context_window: Some(CONTEXT_WINDOW_272K),
            max_output_tokens: Some(MAX_OUTPUT_DEFAULT),
            truncation_policy: TruncationPolicy::Bytes(10_000),
        )
    } else if slug.starts_with("qwen") {
        model_family!(
            slug, "qwen",
            chat_completions_role_strategy: ChatCompletionsRoleStrategy::CollapseNonChatRolesToSystem,
            needs_special_apply_patch_instructions: true,
            repairs_malformed_apply_patch_tool_calls: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: context_window_for_third_party_slug(&slug_lower),
        )
        .map(with_qwen_tool_discipline)
    } else if slug.starts_with("kimi") {
        model_family!(
            slug, "kimi",
            chat_completions_reasoning_strategy:
                ChatCompletionsReasoningStrategy::PreserveReasoningContent,
            context_window: context_window_for_third_party_slug(&slug_lower),
        )
    } else if slug.starts_with("mimo") {
        model_family!(
            slug, "mimo",
            needs_special_apply_patch_instructions: true,
            supports_chat_completions_response_format_json_schema: false,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: context_window_for_third_party_slug(&slug_lower),
        )
        .map(with_mimo_synthesis_checkpoint)
    } else if slug.starts_with("minimax-m3") {
        model_family!(
            slug, "minimax-m3",
            context_window: Some(CONTEXT_WINDOW_1M_ROUND),
            needs_special_apply_patch_instructions: true,
            repairs_malformed_apply_patch_tool_calls: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
        )
        .map(with_minimax_tool_discipline)
    } else if slug.starts_with("minimax-m2.7") {
        model_family!(
            slug, "minimax-m2.7",
            context_window: Some(CONTEXT_WINDOW_MINIMAX_M2_7),
            needs_special_apply_patch_instructions: true,
            repairs_malformed_apply_patch_tool_calls: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
        )
        .map(with_minimax_tool_discipline)
    } else if slug_lower.contains("glm") {
        model_family!(
            slug, "glm",
            uses_local_shell_tool: true,
            needs_special_apply_patch_instructions: true,
            base_instructions: BASE_INSTRUCTIONS_WITH_APPLY_PATCH.to_string(),
            context_window: context_window_for_third_party_slug(&slug_lower),
        )
    } else if slug.starts_with("deepseek") {
        model_family!(
            slug, "deepseek",
            chat_completions_role_strategy: ChatCompletionsRoleStrategy::CollapseNonChatRolesToSystem,
            chat_completions_reasoning_strategy:
                ChatCompletionsReasoningStrategy::PreserveReasoningContent,
            context_window: context_window_for_third_party_slug(&slug_lower),
        )
    } else {
        None
    }
}

pub fn derive_default_model_family(model: &str) -> ModelFamily {
    apply_upstream_model_overrides(ModelFamily {
        slug: model.to_string(),
        family: model.to_string(),
        needs_special_apply_patch_instructions: false,
        context_window: None,
        max_output_tokens: None,
        truncation_policy: TruncationPolicy::Bytes(10_000),
        auto_compact_token_limit: None,
        supports_reasoning_summaries: false,
        default_reasoning_effort: None,
        default_reasoning_summary: ReasoningSummary::Auto,
        supports_parallel_tool_calls: false,
        additional_speed_tiers: Vec::new(),
        supports_search_tool: false,
        prefer_websockets: false,
        uses_local_shell_tool: false,
        apply_patch_tool_type: None,
        repairs_malformed_apply_patch_tool_calls: false,
        repairs_final_output_json_schema: false,
        uses_shell_command_tool: false,
        web_search_tool_type: WebSearchToolType::Text,
        supports_image_detail_original: false,
        supports_image_generation: false,
        chat_completions_role_strategy: ChatCompletionsRoleStrategy::OpenAi,
        chat_completions_reasoning_strategy: ChatCompletionsReasoningStrategy::OpenAi,
        supports_chat_completions_response_format_json_schema: true,
        base_instructions: BASE_INSTRUCTIONS.to_string(),
    })
}

fn supports_image_generation(model_info: &ModelInfo) -> bool {
    model_info.input_modalities.contains(&InputModality::Image)
}

/// Live compatibility matrix inputs derived from [`ModelFamily`] and provider
/// routing helpers. Keeps CLI acceptance tests aligned with runtime behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCompatibilityProfile {
    pub family: String,
    pub uses_local_shell_tool: bool,
    pub needs_apply_patch: bool,
    pub supports_json_schema_output: bool,
    pub repairs_malformed_apply_patch: bool,
    pub uses_anthropic_messages_wire: bool,
    pub expected_wire_slug: String,
}

pub fn compatibility_profile_for_model(provider_id: &str, model: &str) -> ModelCompatibilityProfile {
    let family =
        find_family_for_model(model).unwrap_or_else(|| derive_default_model_family(model));
    ModelCompatibilityProfile {
        family: family.family.clone(),
        uses_local_shell_tool: family.uses_local_shell_tool,
        needs_apply_patch: family.needs_special_apply_patch_instructions,
        supports_json_schema_output: family.supports_chat_completions_response_format_json_schema,
        repairs_malformed_apply_patch: family.repairs_malformed_apply_patch_tool_calls,
        uses_anthropic_messages_wire: uses_opencode_go_anthropic_messages(provider_id, model),
        expected_wire_slug: wire_model_slug(provider_id, model),
    }
}

#[cfg(test)]
mod tests {
    use crate::config_types::ReasoningEffort;
    use crate::tool_apply_patch::ApplyPatchToolType;

    use super::ChatCompletionsRoleStrategy;
    use super::ChatCompletionsReasoningStrategy;
    use super::CONTEXT_WINDOW_1M_ROUND;
    use super::CONTEXT_WINDOW_202_752;
    use super::CONTEXT_WINDOW_262_144;
    use super::CONTEXT_WINDOW_MIMO_PRO;
    use super::CONTEXT_WINDOW_MINIMAX_M2_7;
    use super::EXTENDED_CONTEXT_WINDOW_1M;
    use super::compatibility_profile_for_model;
    use super::default_auto_compact_limit_for_context_window;
    use super::find_family_for_model;
    use super::infer_model_provider_id;
    use super::model_supports_configurable_reasoning_effort;
    use super::model_supports_configurable_reasoning_effort_for_provider;
    use super::model_supports_fast_mode;
    use super::model_supports_fast_mode_for_provider;
    use super::provider_model_slug;
    use super::response_model_matches_request;
    use super::resolve_context_mode_limits;
    use super::supports_extended_context;
    use super::MINIMAX_PROVIDER_ID;
    use super::OPENCODE_GO_PROVIDER_ID;
    use super::XIAOMI_PROVIDER_ID;

    #[test]
    fn image_generation_support_tracks_image_input_modality() {
        let family = find_family_for_model("gpt-5.4").expect("known upstream model");

        assert!(family.supports_image_generation);
    }

    #[test]
    fn bundled_model_metadata_applies_upstream_tool_flags() {
        let family = find_family_for_model("gpt-5.5").expect("known upstream model");

        assert_eq!(
            family.apply_patch_tool_type,
            Some(ApplyPatchToolType::Freeform)
        );
        assert!(family.uses_shell_command_tool);
        assert!(family.supports_search_tool);
        assert!(family.prefer_websockets);
    }

    #[test]
    fn bundled_model_metadata_applies_upstream_reasoning_default() {
        let family = find_family_for_model("gpt-5.4").expect("known upstream model");

        assert_eq!(
            family.default_reasoning_effort,
            Some(ReasoningEffort::Medium)
        );
    }

    #[test]
    fn minimax_m27_has_first_class_model_family() {
        let family = find_family_for_model("MiniMax-M2.7").expect("known MiniMax model");

        assert_eq!(family.family, "minimax-m2.7");
        assert_eq!(family.context_window, Some(CONTEXT_WINDOW_MINIMAX_M2_7));
        assert!(family.needs_special_apply_patch_instructions);
        assert!(!model_supports_configurable_reasoning_effort("MiniMax-M2.7"));
    }

    #[test]
    fn minimax_m3_has_first_class_model_family() {
        let family = find_family_for_model("MiniMax-M3").expect("known MiniMax M3 model");

        assert_eq!(family.family, "minimax-m3");
        assert_eq!(family.context_window, Some(CONTEXT_WINDOW_1M_ROUND));
        assert!(family.needs_special_apply_patch_instructions);
        assert!(!model_supports_configurable_reasoning_effort("MiniMax-M3"));
    }

    #[test]
    fn third_party_context_windows_match_reference_limits() {
        use crate::config_types::ContextMode;

        let cases = [
            ("opencode-go/glm-5.1", CONTEXT_WINDOW_202_752, false),
            ("opencode-go/glm-5.2", CONTEXT_WINDOW_1M_ROUND, false),
            ("opencode-go/glm-5", CONTEXT_WINDOW_202_752, false),
            ("opencode-go/kimi-k2.7-code", CONTEXT_WINDOW_262_144, false),
            ("opencode-go/kimi-k2.6", CONTEXT_WINDOW_262_144, false),
            ("opencode-go/mimo-v2.5-pro", CONTEXT_WINDOW_MIMO_PRO, true),
            ("opencode-go/mimo-v2.5", CONTEXT_WINDOW_1M_ROUND, true),
            ("opencode-go/minimax-m3", CONTEXT_WINDOW_1M_ROUND, false),
            ("opencode-go/minimax-m2.7", CONTEXT_WINDOW_MINIMAX_M2_7, false),
            ("opencode-go/qwen3.7-max", CONTEXT_WINDOW_1M_ROUND, false),
            ("opencode-go/qwen3.7-plus", CONTEXT_WINDOW_1M_ROUND, false),
            ("opencode-go/qwen3.6-plus", CONTEXT_WINDOW_1M_ROUND, false),
            ("opencode-go/deepseek-v4-pro", CONTEXT_WINDOW_1M_ROUND, true),
            ("opencode-go/deepseek-v4-flash", CONTEXT_WINDOW_1M_ROUND, true),
            ("MiniMax-M3", CONTEXT_WINDOW_1M_ROUND, false),
            ("MiniMax-M2.7", CONTEXT_WINDOW_MINIMAX_M2_7, false),
            ("xiaomi/mimo-v2.5-pro", CONTEXT_WINDOW_MIMO_PRO, true),
            ("xiaomi/mimo-v2.5", CONTEXT_WINDOW_1M_ROUND, true),
        ];

        for (model, expected_family_window, extended_on_auto) in cases {
            let family = find_family_for_model(model).unwrap_or_else(|| panic!("{model}"));
            assert_eq!(
                family.context_window,
                Some(expected_family_window),
                "family context for {model}"
            );

            let (auto_window, auto_compact) =
                resolve_context_mode_limits(model, Some(ContextMode::Auto), &family);
            if extended_on_auto {
                assert_eq!(
                    auto_window,
                    Some(EXTENDED_CONTEXT_WINDOW_1M),
                    "auto context for {model}"
                );
                assert_eq!(
                    auto_compact,
                    Some(default_auto_compact_limit_for_context_window(
                        EXTENDED_CONTEXT_WINDOW_1M,
                    )),
                    "auto compact for {model}"
                );
            } else {
                assert_eq!(
                    auto_window,
                    Some(expected_family_window),
                    "auto context for {model}"
                );
                assert_eq!(
                    auto_compact,
                    Some(default_auto_compact_limit_for_context_window(
                        expected_family_window,
                    )),
                    "auto compact for {model}"
                );
            }
        }
    }

    #[test]
    fn namespaced_model_with_hyphenated_provider_id_resolves() {
        let family = find_family_for_model("opencode-go/gpt-5.1")
            .expect("hyphenated provider namespace should resolve");

        assert_eq!(family.slug, "opencode-go/gpt-5.1");
        assert_eq!(family.family, "gpt-5.1");
    }

    #[test]
    fn qwen_and_deepseek_families_collapse_developer_roles_to_system() {
        for (slug, family_name) in [
            ("opencode-go/qwen3.6-plus", "qwen"),
            ("opencode-go/qwen3.7-max", "qwen"),
            ("opencode-go/deepseek-v4-pro", "deepseek"),
        ] {
            let family = find_family_for_model(slug).expect("namespaced model should resolve");

            assert_eq!(family.slug, slug);
            assert_eq!(family.family, family_name);
            assert_eq!(
                family.chat_completions_role_strategy,
                ChatCompletionsRoleStrategy::CollapseNonChatRolesToSystem
            );
        }
    }

    #[test]
    fn qwen_family_repairs_malformed_apply_patch_tool_calls() {
        let family = find_family_for_model("opencode-go/qwen3.7-max")
            .expect("known OpenCode Go Qwen model");
        assert!(family.repairs_malformed_apply_patch_tool_calls);
        assert!(family.needs_special_apply_patch_instructions);
        assert!(
            family
                .base_instructions
                .contains("Kay joins argv arrays with `&&`"),
            "Qwen models need explicit shell argv guidance for OpenCode Go wire"
        );
        assert!(
            family
                .base_instructions
                .contains("never `printf`, `cat >`, or `&&`-chained shell"),
            "Qwen models need apply_patch file-edit guidance"
        );
    }

    #[test]
    fn kimi_family_preserves_reasoning_content_for_chat_completions() {
        let family = find_family_for_model("opencode-go/kimi-k2.6")
            .expect("namespaced model should resolve");

        assert_eq!(family.slug, "opencode-go/kimi-k2.6");
        assert_eq!(family.family, "kimi");
        assert_eq!(
            family.chat_completions_reasoning_strategy,
            ChatCompletionsReasoningStrategy::PreserveReasoningContent
        );
    }

    #[test]
    fn opencode_go_mimo_and_minimax_have_first_class_model_families() {
        let mimo = find_family_for_model("opencode-go/mimo-v2.5")
            .expect("namespaced model should resolve");
        assert_eq!(mimo.slug, "opencode-go/mimo-v2.5");
        assert_eq!(mimo.family, "mimo");

        let minimax = find_family_for_model("opencode-go/minimax-m2.7")
            .expect("namespaced model should resolve");
        assert_eq!(minimax.slug, "opencode-go/minimax-m2.7");
        assert_eq!(minimax.family, "minimax-m2.7");
        assert_eq!(minimax.context_window, Some(CONTEXT_WINDOW_MINIMAX_M2_7));
    }

    #[test]
    fn mimo_family_includes_synthesis_checkpoint_instructions() {
        let family = find_family_for_model("opencode-go/mimo-v2.5-pro")
            .expect("known MiMo model");

        assert!(
            family
                .base_instructions
                .contains("Consolidate findings before ending an investigation turn"),
            "MiMo models need explicit anti-loop synthesis guidance"
        );
        assert!(
            family
                .base_instructions
                .contains("call the shell tool immediately"),
            "MiMo models need explicit pre-tool stall guidance"
        );
        assert!(
            family
                .base_instructions
                .contains("If two `apply_patch` attempts fail"),
            "MiMo models need explicit patch failure recovery guidance"
        );
        assert!(
            family
                .base_instructions
                .contains("Never write hunk labels like `@@ hint paragraph @@`"),
            "MiMo models need explicit apply_patch hunk header guidance"
        );
        assert!(
            family
                .base_instructions
                .contains("never `printf`, `cat >`, or `&&`-chained shell"),
            "MiMo models need apply_patch file-edit guidance"
        );
        assert!(
            family
                .base_instructions
                .contains("honor that STATUS contract"),
            "MiMo models need prompt-driven STATUS closeout guidance"
        );
        assert!(family.repairs_malformed_apply_patch_tool_calls);
        assert!(family.repairs_final_output_json_schema);
        assert!(family.routes_apply_patch_function_call());
        assert!(family.routes_apply_patch_freeform_call());
    }

    #[test]
    fn minimax_family_repairs_malformed_apply_patch_tool_calls() {
        let family = find_family_for_model("opencode-go/minimax-m2.7")
            .expect("known MiniMax model");
        assert!(family.repairs_malformed_apply_patch_tool_calls);
        assert!(!family.repairs_final_output_json_schema);
        assert!(family.routes_apply_patch_function_call());
        assert!(family.routes_apply_patch_freeform_call());
        assert!(
            family
                .base_instructions
                .contains("Kay joins argv arrays with `&&`"),
            "MiniMax models need explicit shell argv guidance for OpenCode Go wire"
        );
        assert!(
            family
                .base_instructions
                .contains("never `printf`, `cat >`, or `&&`-chained shell"),
            "MiniMax models need apply_patch file-edit guidance"
        );
        assert!(
            family
                .base_instructions
                .contains("honor that STATUS contract"),
            "MiniMax models need prompt-driven STATUS closeout guidance"
        );
    }

    #[test]
    fn gpt_oss_routes_native_apply_patch_function_tool() {
        let family = find_family_for_model("gpt-oss-120b").expect("gpt-oss family");
        assert!(family.routes_apply_patch_function_call());
        assert!(!family.routes_apply_patch_freeform_call());
    }

    #[test]
    fn mimo_family_profile_is_shared_across_provider_namespaces() {
        for suffix in ["mimo-v2.5", "mimo-v2.5-pro"] {
            let opencode = find_family_for_model(&format!("opencode-go/{suffix}"))
                .expect("known OpenCode Go MiMo model");
            let xiaomi = find_family_for_model(&format!("xiaomi/{suffix}"))
                .expect("known Xiaomi MiMo model");

            assert_eq!(opencode.family, "mimo");
            assert_eq!(xiaomi.family, "mimo");
            assert_eq!(
                opencode.chat_completions_role_strategy,
                xiaomi.chat_completions_role_strategy
            );
            assert_eq!(
                opencode.chat_completions_reasoning_strategy,
                xiaomi.chat_completions_reasoning_strategy
            );
            assert_eq!(
                opencode.supports_chat_completions_response_format_json_schema,
                xiaomi.supports_chat_completions_response_format_json_schema
            );
            assert!(
                !xiaomi.supports_chat_completions_response_format_json_schema,
                "MiMo models should use shared schema guidance instead of native response_format"
            );
            assert_eq!(
                opencode.needs_special_apply_patch_instructions,
                xiaomi.needs_special_apply_patch_instructions
            );
            assert!(
                xiaomi.needs_special_apply_patch_instructions,
                "MiMo models need the detailed apply_patch grammar"
            );
            assert!(
                xiaomi.base_instructions.contains("## `apply_patch`"),
                "MiMo models should share the detailed apply_patch instructions"
            );
            assert!(
                xiaomi
                    .base_instructions
                    .contains("MiMo investigation discipline")
            );
        }

        assert!(response_model_matches_request(
            "xiaomi/mimo-v2.5-pro",
            "xiaomi/mimo-v2.5-pro-20260422"
        ));
        assert!(response_model_matches_request(
            "xiaomi/mimo-v2.5",
            "xiaomi/mimo-v2.5-20260422"
        ));
        assert!(!response_model_matches_request(
            "xiaomi/mimo-v2.5",
            "xiaomi/mimo-v2.5-pro-20260422"
        ));
    }

    #[test]
    fn non_openai_models_do_not_expose_openai_fast_or_reasoning_effort() {
        assert!(model_supports_fast_mode("gpt-5.4"));
        assert!(model_supports_configurable_reasoning_effort("gpt-5.4"));
        assert!(model_supports_fast_mode_for_provider("openai", "gpt-5.4"));
        assert!(model_supports_configurable_reasoning_effort_for_provider(
            "openai", "gpt-5.4"
        ));

        for model in [
            "opencode-go/glm-5.1",
            "opencode-go/deepseek-v4-flash",
            "MiniMax-M2.7",
        ] {
            assert!(!model_supports_fast_mode(model));
            assert!(!model_supports_configurable_reasoning_effort(model));
        }

        assert!(!model_supports_fast_mode_for_provider(
            OPENCODE_GO_PROVIDER_ID,
            "glm-5.1"
        ));
        assert!(!model_supports_configurable_reasoning_effort_for_provider(
            OPENCODE_GO_PROVIDER_ID,
            "gpt-5.4"
        ));
    }

    #[test]
    fn opencode_go_extended_context_tracks_modelsdev_limits() {
        for model in [
            "opencode-go/deepseek-v4-pro",
            "opencode-go/deepseek-v4-flash",
            "opencode-go/mimo-v2.5",
            "opencode-go/mimo-v2.5-pro",
        ] {
            assert!(supports_extended_context(model), "{model}");
            let family = find_family_for_model(model).expect("known OpenCode Go model");
            let (window, compact) = resolve_context_mode_limits(
                model,
                Some(crate::config_types::ContextMode::OneM),
                &family,
            );
            assert_eq!(window, Some(EXTENDED_CONTEXT_WINDOW_1M));
            assert_eq!(
                compact,
                Some(default_auto_compact_limit_for_context_window(
                    EXTENDED_CONTEXT_WINDOW_1M,
                ))
            );
        }

        for model in ["opencode-go/glm-5.1", "opencode-go/qwen3.6-plus", "MiniMax-M2.7"] {
            assert!(!supports_extended_context(model), "{model}");
        }
    }

    #[test]
    fn deepseek_family_preserves_reasoning_content_for_chat_completions() {
        let family = find_family_for_model("opencode-go/deepseek-v4-pro")
            .expect("namespaced model should resolve");

        assert_eq!(
            family.chat_completions_reasoning_strategy,
            ChatCompletionsReasoningStrategy::PreserveReasoningContent
        );
    }

    #[test]
    fn glm_family_uses_local_shell_tool() {
        for model in ["opencode-go/glm-5.1", "opencode-go/glm-5.2"] {
            let family = find_family_for_model(model).expect("namespaced model should resolve");

            assert_eq!(family.slug, model);
            assert_eq!(family.family, "glm");
            assert!(family.needs_special_apply_patch_instructions);
            assert!(family.uses_local_shell_tool);
        }
    }

    #[test]
    fn glm_compatibility_profile_uses_oa_compat_wire() {
        for (model, wire_slug) in [
            ("opencode-go/glm-5.1", "glm-5.1"),
            ("opencode-go/glm-5.2", "glm-5.2"),
        ] {
            let profile = compatibility_profile_for_model(OPENCODE_GO_PROVIDER_ID, model);
            assert_eq!(profile.family, "glm", "{model}");
            assert!(profile.uses_local_shell_tool, "{model}");
            assert!(profile.needs_apply_patch, "{model}");
            assert!(!profile.uses_anthropic_messages_wire, "{model}");
            assert_eq!(profile.expected_wire_slug, wire_slug, "{model}");
        }
    }

    #[test]
    fn deepseek_compatibility_profile_uses_oa_compat_wire() {
        for (model, wire_slug) in [
            ("opencode-go/deepseek-v4-pro", "deepseek-v4-pro"),
            ("opencode-go/deepseek-v4-flash", "deepseek-v4-flash"),
        ] {
            let profile = compatibility_profile_for_model(OPENCODE_GO_PROVIDER_ID, model);
            assert_eq!(profile.family, "deepseek", "{model}");
            assert!(!profile.uses_local_shell_tool, "{model}");
            assert!(!profile.needs_apply_patch, "{model}");
            assert!(!profile.uses_anthropic_messages_wire, "{model}");
            assert_eq!(profile.expected_wire_slug, wire_slug, "{model}");
        }
    }

    #[test]
    fn qwen37_compatibility_profile_uses_anthropic_wire() {
        let profile = compatibility_profile_for_model(
            OPENCODE_GO_PROVIDER_ID,
            "opencode-go/qwen3.7-plus",
        );
        assert!(profile.uses_anthropic_messages_wire);
        assert_eq!(profile.expected_wire_slug, "qwen3.7-plus");
    }

    #[test]
    fn provider_model_slug_strips_matching_namespace() {
        assert_eq!(
            provider_model_slug("opencode-go", "opencode-go/kimi-k2.6").as_ref(),
            "kimi-k2.6"
        );
        assert_eq!(
            provider_model_slug("opencode-go", "kimi-k2.6").as_ref(),
            "kimi-k2.6"
        );
        assert_eq!(
            provider_model_slug("opencode-go", "other-provider/kimi-k2.6").as_ref(),
            "other-provider/kimi-k2.6"
        );
    }

    #[test]
    fn response_model_match_accepts_third_party_provider_slug_normalization() {
        assert!(response_model_matches_request(
            "opencode-go/glm-5.1",
            "glm-5.1"
        ));
        assert!(response_model_matches_request(
            "opencode-go/glm-5.2",
            "glm-5.2"
        ));
        assert!(response_model_matches_request(
            "opencode-go/glm-5.2",
            "accounts/fireworks/models/glm-5p2"
        ));
        assert!(response_model_matches_request(
            "opencode-go/kimi-k2.6",
            "kimi-k2.6"
        ));
        assert!(response_model_matches_request(
            "minimax/MiniMax-M2.7",
            "MiniMax-M2.7"
        ));
        assert!(response_model_matches_request(
            "minimax/MiniMax-M3",
            "MiniMax-M3"
        ));
    }

    #[test]
    fn response_model_match_accepts_owner_prefixed_opencode_go_versions() {
        assert!(response_model_matches_request(
            "opencode-go/mimo-v2.5-pro",
            "xiaomi/mimo-v2.5-pro-20260422"
        ));
        assert!(response_model_matches_request(
            "opencode-go/mimo-v2.5",
            "xiaomi/mimo-v2.5-20260422"
        ));
        assert!(!response_model_matches_request(
            "opencode-go/mimo-v2.5",
            "xiaomi/mimo-v2.5-pro-20260422"
        ));
        assert!(!response_model_matches_request(
            "opencode-go/mimo-v2.5-pro",
            "xiaomi/mimo-v2.5-20260422"
        ));
    }

    #[test]
    fn response_model_match_accepts_bare_third_party_requests_against_namespaced_responses() {
        assert!(response_model_matches_request(
            "mimo-v2.5",
            "xiaomi/mimo-v2.5-20260422"
        ));
        assert!(response_model_matches_request("MiniMax-M2.7", "minimax/MiniMax-M2.7"));
    }

    #[test]
    fn response_model_match_keeps_openai_version_suffix_behavior() {
        assert!(response_model_matches_request(
            "gpt-5.4",
            "gpt-5.4-2026-04-01"
        ));
        assert!(!response_model_matches_request("gpt-5.4", "gpt-5.5"));
        assert!(!response_model_matches_request("gpt-5.4", "gpt-5.4-mini"));
    }

    #[test]
    fn infer_model_provider_id_prefers_third_party_models() {
        assert_eq!(infer_model_provider_id("MiniMax-M2.7"), Some(MINIMAX_PROVIDER_ID));
        assert_eq!(infer_model_provider_id("MiniMax-M3"), Some(MINIMAX_PROVIDER_ID));
        assert_eq!(
            infer_model_provider_id("opencode-go/kimi-k2.6"),
            Some(OPENCODE_GO_PROVIDER_ID)
        );
        assert_eq!(
            infer_model_provider_id("xiaomi/mimo-v2.5-pro"),
            Some(XIAOMI_PROVIDER_ID)
        );
        assert_eq!(infer_model_provider_id("gpt-5.4"), Some("openai"));
        assert_eq!(infer_model_provider_id("custom-model"), None);
    }
}

impl ModelFamily {
    /// Token limit at which we should automatically compact, if known.
    pub fn auto_compact_token_limit(&self) -> Option<i64> {
        self.auto_compact_token_limit
            .or(self.context_window.map(Self::default_auto_compact_limit))
    }

    /// Whether `apply_patch` FunctionCall items should be routed through patch
    /// normalization (native function tool or model-family recovery).
    pub fn routes_apply_patch_function_call(&self) -> bool {
        matches!(
            self.apply_patch_tool_type,
            Some(ApplyPatchToolType::Function)
        ) || self.repairs_malformed_apply_patch_tool_calls
    }

    /// Whether `apply_patch` CustomToolCall items should be routed through patch
    /// normalization (native freeform tool or model-family recovery).
    pub fn routes_apply_patch_freeform_call(&self) -> bool {
        matches!(
            self.apply_patch_tool_type,
            Some(ApplyPatchToolType::Freeform)
        ) || self.repairs_malformed_apply_patch_tool_calls
    }

    pub fn set_auto_compact_token_limit(&mut self, limit: Option<i64>) {
        self.auto_compact_token_limit = limit;
    }

    pub fn tool_output_max_bytes(&self) -> usize {
        match self.truncation_policy {
            TruncationPolicy::Bytes(limit) => limit,
            TruncationPolicy::Tokens(limit) => limit.saturating_mul(4),
        }
    }

    pub fn set_truncation_policy(&mut self, policy: TruncationPolicy) {
        self.truncation_policy = policy;
    }

    const fn default_auto_compact_limit(context_window: u64) -> i64 {
        // Match upstream behaviour: 90% of the context window.
        ((context_window as i64) * 9) / 10
    }
}

pub const fn default_auto_compact_limit_for_context_window(context_window: u64) -> i64 {
    ((context_window as i64) * 9) / 10
}

fn normalized_third_party_model<'a>(provider_id: &str, model: &'a str) -> Cow<'a, str> {
    provider_model_slug(provider_id, model)
}

pub fn model_supports_fast_mode(model: &str) -> bool {
    match infer_model_provider_id(model) {
        Some(provider) => provider.eq_ignore_ascii_case("openai"),
        None => !model.contains('/'),
    }
}

pub fn model_supports_fast_mode_for_provider(provider_id: &str, model: &str) -> bool {
    if !provider_id.eq_ignore_ascii_case("openai") {
        return false;
    }

    model_supports_fast_mode(model)
}

pub fn model_supports_configurable_reasoning_effort(model: &str) -> bool {
    if infer_model_provider_id(model).is_some_and(|provider| !provider.eq_ignore_ascii_case("openai")) {
        return false;
    }

    find_family_for_model(model)
        .map(|family| family.supports_reasoning_summaries)
        .unwrap_or(false)
}

pub fn model_supports_configurable_reasoning_effort_for_provider(
    provider_id: &str,
    model: &str,
) -> bool {
    if !provider_id.eq_ignore_ascii_case("openai") {
        return false;
    }

    model_supports_configurable_reasoning_effort(model)
}

pub fn uses_opencode_go_anthropic_messages(provider_id: &str, model_slug: &str) -> bool {
    if provider_id != OPENCODE_GO_PROVIDER_ID {
        return false;
    }
    let slug = provider_model_slug(provider_id, model_slug);
    slug.starts_with("qwen3.7") || slug.starts_with("minimax-m")
}

pub fn supports_extended_context(model: &str) -> bool {
    if model.eq_ignore_ascii_case("gpt-5.4") {
        return true;
    }

    let Some(provider_id @ (OPENCODE_GO_PROVIDER_ID | XIAOMI_PROVIDER_ID)) =
        infer_model_provider_id(model)
    else {
        return false;
    };

    matches!(
        normalized_third_party_model(provider_id, model)
            .as_ref()
            .to_ascii_lowercase()
            .as_str(),
        "deepseek-v4-pro" | "deepseek-v4-flash" | "mimo-v2.5" | "mimo-v2.5-pro"
    )
}

pub fn resolve_context_mode_limits(
    model: &str,
    mode: Option<ContextMode>,
    family: &ModelFamily,
) -> (Option<u64>, Option<i64>) {
    match mode {
        Some(ContextMode::OneM | ContextMode::Auto) if supports_extended_context(model) => (
            Some(EXTENDED_CONTEXT_WINDOW_1M),
            Some(default_auto_compact_limit_for_context_window(
                EXTENDED_CONTEXT_WINDOW_1M,
            )),
        ),
        Some(ContextMode::Disabled) => {
            (family.context_window, family.auto_compact_token_limit())
        }
        _ => (family.context_window, family.auto_compact_token_limit()),
    }
}
