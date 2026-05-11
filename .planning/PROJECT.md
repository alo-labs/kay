# code-monorepo

## What This Is

A Rust/Node.js monorepo for the Codex CLI and its supporting workflows, docs, and release tooling. This is a brownfield repository, so milestone changes must preserve the existing provider stack, docs, and build expectations rather than replacing them.

## Core Value

Keep the CLI buildable, understandable, and safe to evolve without disturbing existing workflows.

## Current Milestone: v1.0 OpenCode Go Provider

**Goal:** Configure Kay to work end-to-end with OpenCode Go, verify it live with the supplied API key, and cut a new Kay release after the integration is proven.

**Target features:**
- Built-in OpenCode Go provider support with secure API-key handling.
- `opencode-go/<model>` routing and model-family recognition for a representative OpenCode Go model.
- Live verification plus release notes/version bump for the follow-on Kay release.

## Requirements

### Validated

- ✓ OpenAI remains the default provider and continues to work with existing auth/config flows — current codebase
- ✓ MiniMax remains available as the additional built-in provider and uses its own auth path — current codebase
- ✓ The multi-provider config surface already supports adding or overriding providers without rewriting the CLI — current codebase

### Active

- [ ] Add first-class OpenCode Go provider support to Kay.
- [ ] Recognize and route `opencode-go/<model>` slugs for at least one representative OpenCode Go model.
- [ ] Validate the provider end-to-end with the supplied OpenCode Go API key before release.
- [ ] Document how to configure and use OpenCode Go in Kay, including the verified model namespace.
- [ ] Cut a new Kay release after the OpenCode Go integration is verified.

### Out of Scope

- Support for every OpenCode Go model family on day one — the first milestone only needs one verified path.
- Replacing the existing OpenAI or MiniMax provider behavior — the new provider must coexist with current support.
- Broader provider-architecture rewrites — the existing multi-provider stack is already the intended foundation.
- Committing or documenting the supplied API key — secrets must stay out of git history and docs.

## Context

- Kay already has a multi-provider architecture from the MiniMax work.
- Official OpenCode docs say OpenCode Go uses model IDs in the `opencode-go/<model-id>` format, with examples such as Kimi K2.6, and that those models are reachable via OpenCode Go API endpoints.
- The OpenCode Go docs also show mixed wire APIs across models, so the first milestone should validate a representative OpenAI-compatible path and keep any non-matching model families explicit.
- The user has supplied a live OpenCode Go API key for verification; do not commit it anywhere.

## Constraints

- **Build**: `./build-fast.sh` is still the required validation gate.
- **Security**: API keys must stay out of git history and docs; use the normal auth/env flow only.
- **Compatibility**: Existing OpenAI and MiniMax support must remain intact while OpenCode Go is added.
- **Release**: No release until OpenCode Go has been validated live end-to-end.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| OpenCode Go will be treated as a first-class provider integration, not a one-off config snippet | Keeps docs, tests, and release flow aligned | Pending |
| The milestone will validate a representative `opencode-go/<model>` path before release | OpenCode docs show multiple model formats; we should prove one working path first | Pending |
| The supplied API key will be used only for live validation, never committed | Protects secrets while still enabling end-to-end proof | Pending |

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
*Last updated: 2026-05-11 after OpenCode Go milestone start*
