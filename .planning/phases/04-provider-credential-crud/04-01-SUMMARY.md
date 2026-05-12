---
phase: 04-provider-credential-crud
plan: 01
subsystem: auth
tags: [rust, auth.json, provider-credentials, unit-tests]
requires: []
provides:
  - provider-key deletion helper in `code-rs/core/src/auth.rs`
  - regression tests for save/update/delete round trips
  - OpenAI auth preservation while provider credentials mutate
affects:
  - phase-4 provider CRUD follow-up plans
  - phase-5 dynamic model selection
tech-stack:
  added: []
  patterns:
    - atomic auth.json mutation for provider credential CRUD
    - idempotent delete helper for missing provider entries
key-files:
  created:
    - .planning/phases/04-provider-credential-crud/04-01-SUMMARY.md
  modified:
    - code-rs/core/src/auth.rs
    - .planning/phases/04-provider-credential-crud/deferred-items.md
    - .planning/STATE.md
    - .planning/ROADMAP.md
    - .planning/REQUIREMENTS.md
key-decisions:
  - "Delete provider credentials only from auth.json.provider_credentials and leave the dedicated OPENAI_API_KEY field untouched."
  - "Normalize provider ids before mutation and treat missing auth files or missing provider entries as no-op deletes."
patterns-established:
  - "Pattern 1: route provider credential writes through the existing atomic auth.json writer."
  - "Pattern 2: use focused auth.rs unit tests to prove save/update/delete round trips and unrelated credential preservation."
requirements-completed: [PROVIDER-01]
duration: 30m
completed: 2026-05-12
---

# Phase 4: Provider Credential CRUD Summary

**Atomic provider-key deletion in auth.rs with round-trip tests that preserve OpenAI auth and other provider credentials**

## Performance

- **Duration:** 30 min
- **Started:** 2026-05-12T11:52:38Z
- **Completed:** 2026-05-12T12:22:01Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Added `remove_provider_api_key(...)` so provider credential deletion only mutates `auth.json.provider_credentials` through the existing atomic write path.
- Kept OpenAI auth on the dedicated `OPENAI_API_KEY` field and preserved unrelated provider credentials during save, update, and delete cycles.
- Added regression tests covering save/update/delete round trips, provider normalization, and OpenAI preservation, then verified them with the focused auth suite and `./build-fast.sh`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add provider-key deletion to the auth layer** - `768fb8428` (feat)
2. **Task 2: Lock provider-credential CRUD invariants with unit tests** - `860e54b28` (test)

**Plan metadata:** pending final docs commit

## Files Created/Modified
- `code-rs/core/src/auth.rs` - Added the deletion helper and regression tests.
- `.planning/phases/04-provider-credential-crud/deferred-items.md` - Logged unrelated build warnings from sibling provider-credentials UI work.
- `.planning/STATE.md` - Session state will be refreshed to reflect completion.
- `.planning/ROADMAP.md` - Phase 4 progress will be advanced.
- `.planning/REQUIREMENTS.md` - PROVIDER-01 will be marked complete.
- `.planning/phases/04-provider-credential-crud/04-01-SUMMARY.md` - Phase summary artifact.

## Decisions Made
- Kept delete scope limited to `auth.json.provider_credentials` so the existing OpenAI auth path stays untouched.
- Normalized provider ids before mutation to keep save/delete behavior consistent with the rest of the auth helpers.
- Treated missing auth files and missing provider entries as no-op deletes so provider CRUD stays idempotent.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `cargo test -p code-core auth -- --nocapture` needed an explicit `cargo +1.90.0` invocation from `code-rs/` because the shell had no default Rust toolchain and the workspace root lives under `code-rs/`.
- `./build-fast.sh` surfaced warnings in sibling `code-rs/tui/src/bottom_pane/provider_credentials_view.rs`; those are outside this plan scope and were logged to [deferred-items.md](./deferred-items.md).

## Next Phase Readiness
- The auth-layer deletion helper is ready for the remaining phase-4 provider UI and CLI work.
- Sibling phase-4 work can now consume the new delete helper without changing auth storage semantics.

## Self-Check: PASSED

- Summary file exists at `.planning/phases/04-provider-credential-crud/04-01-SUMMARY.md`.
- Task commit hashes `768fb8428` and `860e54b28` both exist in git history.

---
*Phase: 04-provider-credential-crud*
*Completed: 2026-05-12*
