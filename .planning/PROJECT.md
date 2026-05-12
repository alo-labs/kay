# code-monorepo

## What This Is

A Rust/Node.js monorepo for the Kay CLI and its supporting workflows, docs, and release tooling. This is a brownfield repository, so milestone changes must preserve the existing Kay stack, docs, and build expectations rather than replacing them.

## Core Value

Keep the CLI buildable, understandable, and safe to evolve without disturbing existing workflows.

## Current Milestone: v0.9.0 Test Notes App and Kay OCG Validation

**Goal:** Establish a Kay-local runtime under `~/.kay`, remove automatic inheritance from local Codex/Every Code environments, and bootstrap a real `projects/test-notes-app` validation project that Kay can drive with the supported OpenCode Go models.

**Target features:**
- Kay defaults to an isolated `~/.kay` home for end-user installs and no longer auto-inherits a local Codex environment.
- A real `alo-exp/test-notes-app` repository can be bootstrapped and driven by Kay from a local checkout under `projects/test-notes-app`.
- Kay exposes a lightweight transcript viewer for JSONL provenance and future analysis.
- Every supported OpenCode Go model can be exercised on meaningful note-app work before release.

## Requirements

### Validated

- ✓ Kay already has a working multi-provider architecture and provider-aware model selection from the prior milestone — shipped milestone
- ✓ Kay already records session transcripts as JSONL and exposes transcript-oriented UI surfaces — current codebase
- ✓ The current Kay workspace still has a multi-provider auth/config stack that can be isolated per home directory — current codebase

### Active

- [ ] Make Kay default to `~/.kay` for end-user state and stop inheriting a local Codex environment by default.
- [ ] Bootstrap a real `alo-exp/test-notes-app` repository that Kay can develop from a local `projects/test-notes-app` checkout.
- [ ] Add a lightweight transcript viewer that reads Kay session JSONL and renders a modern chat-like review experience.
- [ ] Run meaningful note-app work through the supported OpenCode Go models so each model proves more than a smoke test.
- [ ] Add tests and docs for the isolated Kay home, the transcript viewer, and the note-app validation workflow.
- [ ] Cut the next Kay release only after the isolated Kay install and note-app validation flow are verified.

## Out of Scope

- Reintroducing automatic inheritance from local Codex or Every Code state into the end-user Kay install.
- Building a generic multi-tenant agent platform beyond the specific Kay + note-app validation workflow.
- Expanding the supported provider set beyond the current OpenCode Go, MiniMax, and OpenAI stack in this milestone.
- Secret or API-key leakage into repo history, docs, transcripts, or release artifacts.

## Context

- Kay now has provider CRUD and provider-aware model selection from the prior milestone.
- The new milestone is about isolating Kay’s end-user runtime and turning Kay into its own validation harness for a real small project.
- The note-taking app is intended to be the live target repository for repeated Kay model validation, not a throwaway demo.
- Transcript JSONL is the provenance source of truth; the viewer must stay lightweight and readable.
- The supported OCG models must be exercised on realistic work, not only one-line smoke prompts.

## Constraints

- **Build**: `./build-fast.sh` remains the required validation gate for this repo.
- **Isolation**: Kay should use its own home under `~/.kay` for the end-user install and not silently read local Codex/Every Code state.
- **Provenance**: Keep session JSONL transcripts accessible for analysis and viewer output.
- **Release**: No release until isolated Kay state, transcript access, and note-app validation have been verified.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| `~/.kay` is the default writable home for the end-user Kay install | Keeps the Kay install isolated from local Codex/Every Code state | Pending |
| Kay should not auto-inherit local Codex environment state by default | Prevents cross-product environment leakage and surprises | Pending |
| `projects/test-notes-app` should be a real GitHub-backed repo under `alo-exp` | Gives Kay a stable live target for realistic model validation | Pending |
| Transcript JSONL should remain directly inspectable through a lightweight viewer | Makes provenance easy to inspect without heavyweight tooling | Pending |

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
*Last updated: 2026-05-13 after v0.9.0 milestone switch*
