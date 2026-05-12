# Phase 7: Bootstrap isolated test-notes-app note-taking project and Kay OCG live-testing harness - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Source:** User request and milestone switch to v0.9.0

<domain>
## Phase Boundary

This phase establishes the Kay-local end-user runtime under `~/.kay`, removes automatic inheritance from local Codex/Every Code state, and sets up a real `projects/test-notes-app` validation project with a transcript viewer and live OCG validation harness.

The phase is intentionally bootstrap-first: it should create the isolation foundation, the transcript provenance surface, and the note-app validation scaffolding that later phases can extend into deeper app features.

</domain>

<decisions>
## Implementation Decisions

### Kay home isolation
- The end-user Kay install must default to `~/.kay` and not silently inherit local `~/.code` or `~/.codex` state.

### Transcript provenance
- Kay session JSONL remains the source of truth for later analysis, so the viewer should read that data directly rather than invent a second transcript format.

### Real validation target
- `alo-exp/test-notes-app` must be a real GitHub-backed project, not a toy mock, and the local checkout under `projects/test-notes-app` should be the target Kay works on.

### OCG model validation
- The supported OpenCode Go models should be exercised on meaningful note-app work so the harness proves more than a smoke test.

### the agent's Discretion
- Exact repo bootstrap mechanics, viewer implementation details, and the division between Kay code changes versus external repo setup are left to the agent as long as the isolated Kay install, transcript access, and validation harness all land.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Kay home / config paths
- `code-rs/core/src/config/sources.rs` — Defines Kay home resolution and legacy-read behavior.
- `code-rs/arg0/src/lib.rs` — Loads global dotenv files and bridges environment variables during startup.
- `code-rs/core/src/config.rs` — Re-exports config helpers used throughout the workspace.
- `code-rs/core/src/message_history.rs` — Persists transcript/history JSONL under the resolved home directory.
- `code-rs/tui/src/session_log.rs` — Writes per-session JSONL transcript logs for UI analysis.

### Login / auth entry points
- `code-rs/cli/src/login.rs` — CLI key-entry flow that should remain compatible with the isolated Kay install.
- `code-rs/cli/tests/provider_api_key_entry.rs` — Current coverage for direct API-key entry modes.

### Transcript and live validation references
- `code-rs/cli/tests/provider_model_acceptance.rs` — Existing live provider/model acceptance matrix.
- `code-rs/cli/tests/opencode_go_live_e2e.rs` — Focused OCG live regression smoke.
- `code-rs/cli/tests/minimax_live_e2e.rs` — Focused MiniMax live regression smoke.
- `docs/auto-drive.md` — Describes transcript/session-log handling for Kay.
- `docs/config.md` — Describes current home-dir and history/config behavior that must be updated for the new default.
- `docs/authentication.md` — Current auth and home-dir guidance that must be updated with the Kay-isolated install story.

</canonical_refs>

<specifics>
## Specific Ideas

- The note-taking app repo should live at `/Users/shafqat/projects/test-notes-app` locally and `alo-exp/test-notes-app` on GitHub.
- The transcript viewer should feel like a lightweight chat app: readable message bubbles/rows, chronological transcript browsing, and easy provenance access from JSONL.
- The Kay install path must stay separate from the current dev-test workspace so the end-user simulation does not accidentally reuse local Codex state.
- Later milestone work can deepen the note-taking app itself, but this phase must first make the isolated Kay runtime and validation target real.

</specifics>

<deferred>
## Deferred Ideas

- Detailed note-app product features beyond the bootstrap skeleton.
- Cross-provider model experiments outside the supported OpenCode Go matrix.
- Any optional backward-compatibility shim for local Codex state inheritance.

</deferred>

---

*Phase: 07-bootstrap-isolated-test-notes-app-note-taking-project-and-ka*
*Context gathered: 2026-05-13 via milestone switch and repo review*
