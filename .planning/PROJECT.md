# code-monorepo

## What This Is

A Rust/Node.js monorepo for the Codex CLI and its supporting workflows, docs, and release tooling. This is a brownfield repository, so milestone changes must preserve the existing provider stack, docs, and build expectations rather than replacing them.

## Core Value

Keep the CLI buildable, understandable, and safe to evolve without disturbing existing workflows.

## Current Milestone: v0.8.0 Provider CRUD and Dynamic Model Selection

**Goal:** Let users manage supported providers from Kay itself, set provider API keys without editing config files, and make `/model` reflect only the models enabled by configured provider credentials.

**Target features:**
- `/provider` CRUD for the supported provider set, ordered OpenCode Go, MiniMax, OpenAI.
- CLI API-key entry for providers so keys can be supplied directly without config-file edits.
- Provider-aware `/model` lists plus reusable model-plugin-style compatibility profiles.

## Requirements

### Validated

- ✓ OpenAI remains the default provider and continues to work with existing auth/config flows — current codebase
- ✓ MiniMax remains available as the additional built-in provider and uses its own auth path — current codebase
- ✓ OpenCode Go provider support was added, live-verified, and released in v0.7.2 — shipped milestone
- ✓ The multi-provider config surface already supports adding or overriding providers without rewriting the CLI — current codebase

### Active

- [ ] Add a `/provider` CRUD command for supported providers in Kay.
- [ ] Allow provider API keys to be supplied directly via CLI argument, without editing config files.
- [ ] Make `/model` list only models enabled by configured provider credentials.
- [ ] Keep model-specific compatibility profiles reusable and orthogonal to provider plugins.
- [ ] Add tests and docs for provider CRUD and provider-aware model selection.
- [ ] Cut a new Kay release after the provider-management milestone is verified.

### Out of Scope

- Adding provider families beyond OpenCode Go, MiniMax, and OpenAI in this milestone.
- Replacing the existing OpenAI, MiniMax, or OpenCode Go provider behavior — the new CRUD surface must coexist with current support.
- Broad provider-architecture rewrites — the existing multi-provider stack is already the intended foundation.
- Hard-coding model adaptations per SKU instead of reusable model plugins.
- Committing or documenting the supplied API key — secrets must stay out of git history and docs.

## Context

- Kay already has a multi-provider architecture from the MiniMax work.
- The CLI already supports provider-specific key storage via `code login --provider <id> --with-api-key`, but the TUI still needs a `/provider` CRUD surface.
- `/model` currently selects model presets only; it does not yet filter by which provider keys are configured.
- The user wants provider plugins and model plugins to remain orthogonal, so reusable compatibility profiles should not be hard-coded per model SKU.
- OpenCode Go, MiniMax, and OpenAI are the supported providers for this milestone, in that order.

## Constraints

- **Build**: `./build-fast.sh` is still the required validation gate.
- **Security**: API keys must stay out of git history and docs; use the normal auth/env flow only.
- **Compatibility**: Existing OpenAI, MiniMax, and OpenCode Go support must remain intact while the new CRUD/model-selection UX is added.
- **Release**: No release until provider CRUD, provider-aware model selection, and the new provider/plugin seam have been verified.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| `/provider` becomes the canonical TUI surface for creating, reading, updating, and deleting provider credentials | Keeps provider management discoverable and avoids file edits | Pending |
| CLI provider API keys should be accepted directly as arguments in addition to any existing stdin flow | Lets users onboard providers without editing config files | Pending |
| `/model` should only list models for providers with configured credentials | Keeps the picker honest and avoids dead-end choices | Pending |
| Provider plugins and model plugins should remain orthogonal | Lets multiple models share the same compatibility profile cleanly | Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-12 after v0.8.0 semver realignment*
