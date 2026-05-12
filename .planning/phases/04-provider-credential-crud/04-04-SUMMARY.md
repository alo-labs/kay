---
phase: 04-provider-credential-crud
plan: 4
subsystem: ui
tags: [rust, tui, provider-credentials, auth, snapshots, testing]

# Dependency graph
requires:
  - phase: 04-provider-credential-crud
    provides: provider-pane shell and add/update CRUD flow from plan 3 plus provider auth helper context from plan 1
provides:
  - provider delete-confirmation state and execution path in the provider pane
  - OpenAI-specific credential clearing that preserves existing login behavior
  - VT100 regression coverage for provider list, add/update, and delete-confirmation states
  - a small test harness helper to open the provider overlay directly for snapshot tests
affects: [phase 5 dynamic model selection, future provider credential UI polish, provider-auth regression coverage]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - explicit destructive confirmation state for provider deletion
    - provider-specific credential removal with an OpenAI preservation branch
    - deterministic VT100 snapshot coverage seeded from tempdir-backed auth state
    - test-only harness helper for opening the provider overlay without slash-command dispatch

key-files:
  created:
    - code-rs/tui/tests/snapshots/vt100_chatwidget_snapshot__provider_management_states_cover_list_update_add_and_delete.snap
    - .planning/phases/04-provider-credential-crud/04-04-SUMMARY.md
  modified:
    - code-rs/tui/src/bottom_pane/provider_credentials_view.rs
    - code-rs/tui/src/chatwidget/smoke_helpers.rs
    - code-rs/tui/tests/vt100_chatwidget_snapshot.rs

key-decisions:
  - "Keep provider deletion inside the provider pane with an explicit confirm/cancel state instead of routing it through `/login`."
  - "Treat OpenAI deletion as clearing the provider credential state only, so the existing login behavior remains intact."
  - "Use a small test-only harness helper to open the provider overlay directly because the VT100 harness does not dispatch slash commands the same way the live app does."

patterns-established:
  - "Pattern 1: destructive provider actions should require an explicit UI confirmation before mutation."
  - "Pattern 2: provider-credential snapshots should exercise visible UI states only and avoid raw secrets."
  - "Pattern 3: deterministic provider-credential tests should seed tempdir-backed auth state so list ordering and key presence stay stable."

requirements-completed: [PROVIDER-01]

# Metrics
duration: 18m
completed: 2026-05-12
---

# Phase 4: Provider Credential CRUD Summary

**Provider delete confirmation and VT100 regression coverage for `/provider`, with OpenAI login behavior preserved and redacted snapshots**

## Performance

- **Duration:** 18 min
- **Started:** 2026-05-12T12:32:59Z
- **Completed:** 2026-05-12T12:50:59Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added a delete-confirmation flow in the provider pane so destructive actions are explicit and only remove the selected provider credential.
- Preserved OpenAI login behavior by clearing only the provider credential state while leaving the existing auth path intact.
- Added VT100 snapshots for the ordered provider list, add/update state, and delete-confirmation state without exposing raw keys.
- Added a small test-only harness helper so the snapshot test can open the provider overlay directly and remain deterministic.
- Verified the targeted VT100 regression command and the repo build gate with `./build-fast.sh`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add provider deletion to the provider pane** - `5a39cb0c2` (feat)
2. **Task 2: Snapshot the final provider-management UX** - `c824f8259` (feat)

**Plan metadata:** pending final docs/state commit

## Files Created/Modified
- `code-rs/tui/src/bottom_pane/provider_credentials_view.rs` - Added provider delete confirmation, delete execution, and OpenAI-specific credential clearing.
- `code-rs/tui/src/chatwidget/smoke_helpers.rs` - Added a test-only helper to open the provider overlay directly.
- `code-rs/tui/tests/vt100_chatwidget_snapshot.rs` - Added provider-management VT100 snapshot coverage.
- `code-rs/tui/tests/snapshots/vt100_chatwidget_snapshot__provider_management_states_cover_list_update_add_and_delete.snap` - Captured the final provider-management frames.

## Decisions Made
- Kept provider deletion inside the provider pane so it stays separate from `/login` and `/model`.
- Preserved OpenAI login behavior by clearing only the provider credential state on delete.
- Added a test-only overlay opener instead of trying to drive slash-command dispatch through the VT100 harness.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added a test-only overlay opener for the VT100 harness**
- **Found during:** Task 2 (Snapshot the final provider-management UX)
- **Issue:** The VT100 harness did not route slash-command dispatch, so the new provider pane could not be opened reliably from the snapshot test.
- **Fix:** Added `ChatWidgetHarness::open_provider_credentials_overlay()` in `code-rs/tui/src/chatwidget/smoke_helpers.rs` and used it in the provider-management snapshot test.
- **Files modified:** `code-rs/tui/src/chatwidget/smoke_helpers.rs`, `code-rs/tui/tests/vt100_chatwidget_snapshot.rs`
- **Verification:** `cargo test -p code-tui --test vt100_chatwidget_snapshot --features test-helpers -- --nocapture` passed with the new provider snapshot.
- **Committed in:** `c824f8259`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to make the VT100 coverage deterministic and executable without changing the live `/provider` behavior.

## Issues Encountered
- The provider overlay could not be exercised through slash-command dispatch inside the VT100 harness, so a small test-only helper was added to reach the view directly.
- The first snapshot run produced a `.snap.new` file as expected; promoting it to the committed `.snap` file resolved the final verification mismatch.

## Next Phase Readiness
- Provider CRUD now has explicit delete behavior and regression snapshots, so phase 5 can continue with model-selection work without revisiting provider deletion.
- The existing OpenAI login path remains intact, and the provider list order stays stable for downstream UI work.

## Self-Check: PASSED

- Summary file exists at `.planning/phases/04-provider-credential-crud/04-04-SUMMARY.md`.
- Task commit hashes `5a39cb0c2` and `c824f8259` both exist in git history.

---
*Phase: 04-provider-credential-crud*
*Completed: 2026-05-12*
