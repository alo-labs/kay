# Phase 4: Provider Credential CRUD - Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Add the `/provider` command and CLI API-key entry so users can CRUD supported provider credentials without editing config files. This phase covers provider registration, credential entry, updates, and deletion for the supported provider set only. Provider-aware model filtering is a separate phase.

</domain>

<decisions>
## Implementation Decisions

### Provider CRUD surface
- **D-01:** `/provider` is the canonical TUI command for provider credential CRUD.
- **D-02:** The supported providers appear in this order: OpenCode Go, MiniMax, OpenAI.
- **D-03:** CRUD should cover create, read/list, update, and delete for provider credentials, not just add-account flows.

### Credential entry
- **D-04:** Provider API keys should be accepted directly as a CLI argument so scripted onboarding does not require config-file edits.
- **D-05:** Existing stdin-based key entry remains available for compatibility where the current login flow already uses it.

### Plugin seam
- **D-06:** Provider registration must stay orthogonal to model compatibility logic; provider CRUD should not hard-code model-family adaptations.
- **D-07:** Existing provider auth behavior for OpenAI, MiniMax, and OpenCode Go must remain intact while the new CRUD surface is added.

### the agent's Discretion
- Exact `/provider` screen layout and copy.
- Whether provider state is presented inline in the current account pane or in a dedicated provider pane.
- Confirmation prompt wording for destructive delete operations.

</decisions>

<specifics>
## Specific Ideas

- The user explicitly does not want to edit config files for API-key onboarding.
- The supported provider order matters: OpenCode Go first, then MiniMax, then OpenAI.
- The current CLI already supports provider-specific key storage, so the new work should reuse that path instead of inventing a second auth model.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Provider and auth
- `docs/config.md` — Existing provider/auth configuration surface and current key-handling conventions
- `docs/authentication.md` — Auth and API-key handling expectations for Kay

### Command and UX conventions
- `docs/slash-commands.md` — Slash-command naming and interaction conventions
- `docs/ARCHITECTURE.md` — Provider-model abstraction assessment and next-iteration sketch

### Test strategy
- `docs/TESTING.md` — Two-layer acceptance/regression testing strategy for provider-model work

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `code-rs/core/src/auth.rs` — Already has `save_provider_api_key` and provider-key storage helpers.
- `code-rs/cli/src/login.rs` — Already routes provider-specific login and stdin API-key intake.
- `code-rs/core/src/model_provider_info.rs` — Built-in provider registry and auth metadata.

### Established Patterns
- `code-rs/tui/src/bottom_pane/login_accounts_view.rs` is the existing account-management surface, but it is OpenAI-centric and will need provider-aware refactoring or a new command surface.
- `code-rs/tui/src/bottom_pane/model_selection_view.rs` and `code-rs/common/src/model_presets.rs` show that `/model` is currently model-only and provider-agnostic.

### Integration Points
- `code-rs/core/src/config.rs` — Built-in provider selection and reserved provider IDs
- `code-rs/core/src/auth_accounts.rs` — Account storage / provider-key persistence
- `code-rs/tui/src/app.rs` / `code-rs/tui/src/chatwidget.rs` — Slash-command dispatch and TUI command handling

</code_context>

<deferred>
## Deferred Ideas

- Provider-aware model filtering and provider-specific model lists belong in Phase 5.
- Broader provider families beyond OpenCode Go, MiniMax, and OpenAI belong in a later milestone.
- A broader settings UX redesign for auth flows is out of scope for this phase.

</deferred>

---

*Phase: 04-provider-credential-crud*
*Context gathered: 2026-05-12*
