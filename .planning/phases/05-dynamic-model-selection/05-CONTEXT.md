---
phase: 05-dynamic-model-selection
gathered: 2026-05-12
status: Ready for planning
---

# Phase 5: Dynamic Model Selection - Context

**Goal:** Make `/model` show only the models enabled by configured provider credentials, while keeping provider plugins and model plugins orthogonal.

<domain>
## Phase Boundary

This phase takes the provider-credential surface from Phase 4 and uses it to filter the model picker. The user should only see models for providers whose API keys or equivalent auth are available, and the model list should stay aligned with the provider-specific model sets already supported in Kay.

</domain>

<decisions>
## Implementation Decisions

### Provider-aware model visibility
- **D-01:** `/model` remains the user-facing model picker, but it should show only models for providers that currently have usable credentials.
- **D-02:** A provider counts as configured when Kay can already resolve its auth through the existing auth/env logic. That includes stored credentials and the existing OpenAI auth path.
- **D-03:** The visible provider order for model grouping and/or sectioning must remain OpenCode Go, MiniMax, OpenAI.

### Provider-specific model sets
- **D-04:** OpenCode Go should expose the OpenCode Go model list Kay already supports and live-tested.
- **D-05:** MiniMax should expose only `MiniMax-M2.7` for now.
- **D-06:** OpenAI should expose the OpenAI model set already supported by the upstream OSS Codex line, not a new bespoke list.

### Architecture seam
- **D-07:** Provider registration and model compatibility profiles must stay orthogonal. Do not encode provider-specific model filtering as a pile of per-SKU exceptions in the UI.
- **D-08:** Model-selection filtering should be driven from a reusable helper or catalog so future UI and API surfaces can share the same visibility rules.

### UI behavior
- **D-09:** If no provider credentials are configured, the model picker should present a clear empty-state / onboarding hint rather than a broken list.
- **D-10:** The picker should keep its current reasoning-effort behavior and model detail rendering; only the available model list should become provider-aware.

### agent's discretion
- Exact grouping chrome in the picker, as long as the provider order is stable and the user can clearly see which provider unlocks which models.
- Whether to keep the picker as a flat list with provider labels or as explicit provider sections.
- Exact empty-state wording when no configured provider is available.

</decisions>

<specifics>
## Specific Ideas

- The user explicitly wants `/model` to show only the models for which API key has been set or provided.
- OpenCode Go should continue to use the current supported model matrix rather than inventing a new one.
- MiniMax should remain narrow for now: only `MiniMax-M2.7`.
- OpenAI should continue to reflect the existing upstream-supported Codex model set.
- The same provider/model seam should be reusable for future provider families, but those families are deferred beyond this milestone.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Provider, auth, and model data
- `code-rs/core/src/auth.rs` — provider credential storage and auth resolution
- `code-rs/core/src/model_provider_info.rs` — built-in provider registry and auth metadata
- `code-rs/common/src/model_presets.rs` — current model preset inventory

### Model picker and command UX
- `code-rs/tui/src/bottom_pane/model_selection_view.rs` — current picker behavior
- `code-rs/tui/src/slash_command.rs` — command naming conventions
- `code-rs/tui/src/chatwidget.rs` — TUI command dispatch and picker entry points

### Strategy and docs
- `docs/ARCHITECTURE.md` — provider/model abstraction assessment and next-iteration sketch
- `docs/TESTING.md` — provider-model testing strategy and acceptance/regression split
- `docs/slash-commands.md` — existing command documentation and ordering

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `code-rs/core/src/auth.rs` already knows how to resolve provider credentials from `auth.json` and the existing OpenAI auth path.
- `code-rs/core/src/model_provider_info.rs` already stores the built-in provider metadata and provider credential ids.
- `code-rs/common/src/model_presets.rs` already contains the current OpenAI model preset inventory.
- `code-rs/tui/src/bottom_pane/model_selection_view.rs` already owns the picker UI and reasoning-effort rendering.

### Established Patterns
- Phase 4 kept `/provider` separate from `/login` and `/model`; that separation should continue here.
- The model picker is currently provider-agnostic, so this phase should introduce provider-aware filtering without rewriting the selection UX from scratch.
- The reusable compatibility-profile seam from the model-family work should stay in the core layer instead of being duplicated in the TUI.

### Integration Points
- `code-rs/core/src/config.rs` — provider selection and config resolution
- `code-rs/core/src/model_family.rs` — compatibility-profile / family-level behavior
- `code-rs/tui/src/app.rs` / `code-rs/tui/src/chatwidget.rs` — slash-command dispatch and picker entry points
- `code-rs/tui/src/bottom_pane/model_selection_view.rs` — actual model list rendering

</code_context>

<deferred>
## Deferred Ideas

- Broader provider families beyond OpenCode Go, MiniMax, and OpenAI stay out of this milestone.
- Deeper UI redesign of the model picker is out of scope; keep the current picker affordances unless a small grouping change is needed to make provider filtering understandable.
- Provider CRUD itself is complete in Phase 4 and should not be reopened here.

</deferred>

---

*Phase: 05-dynamic-model-selection*
*Context gathered: 2026-05-12*
