---
phase: 05-dynamic-model-selection
plan: 2
subsystem: ui
tags: [ui, model-selection, vt100, snapshots]

# Dependency graph
requires:
  - phase: 05-01
    provides: shared provider visibility helper and auth-gated preset filtering
provides:
  - Shared `/model` wiring that consumes the reusable visibility helper
  - Provider-grouped picker rendering with stable bucket order and empty state
  - Live VT100 coverage for credentialed and no-credential picker states
affects:
  - model picker UX
  - provider credentials flow
  - snapshot coverage

# Tech tracking
tech-stack:
  added: [code_core::model_visibility, insta snapshots, test-only smoke helper]
  patterns: [provider-aware picker rendering, live VT100 smoke coverage, empty-state onboarding hint]

key-files:
  created:
    - code-rs/tui/tests/snapshots/vt100_chatwidget_snapshot__model_selection_visibility__credentialed_provider_list.snap
    - code-rs/tui/tests/snapshots/vt100_chatwidget_snapshot__model_selection_visibility__empty_credentials_hint.snap
  modified:
    - code-rs/tui/src/chatwidget.rs
    - code-rs/tui/src/bottom_pane/model_selection_view.rs
    - code-rs/tui/src/chatwidget/smoke_helpers.rs
    - code-rs/tui/tests/vt100_chatwidget_snapshot.rs

key-decisions:
  - Use the shared `code_core::model_visibility` helper as the source of truth for picker visibility.
  - Keep the OpenAI shortlist curation inside the OpenAI bucket only, after provider grouping.
  - Render provider headers in the fixed OpenCode Go, MiniMax, OpenAI order and show an explicit empty state when nothing is unlocked.

patterns-established:
  - "Pattern 1: shared auth-driven visibility now feeds every /model entry point."
  - "Pattern 2: provider metadata stays attached through rendering so bucket headers and model rows can be ordered independently."
  - "Pattern 3: live VT100 smoke tests open the real picker, not a fake render path."

requirements-completed: [MODEL-01, MODEL-02]

# Metrics
duration: 2h 30m
completed: 2026-05-12
---

# Phase 05: Dynamic Model Selection Summary

**Shared provider visibility wiring for `/model`, provider-grouped picker rendering, and live VT100 coverage**

## Performance

- **Duration:** 2h 30m
- **Started:** 2026-05-12T11:47:00Z
- **Completed:** 2026-05-12T14:17:04Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- `ChatWidget` now routes both preset refresh and `/model` selection through the shared visibility helper.
- The picker renders locked provider buckets in stable order, preserves reasoning-effort rows, and shows a clear empty state.
- A test-only smoke helper opens the real `/model` overlay, and VT100 snapshots cover both credentialed and empty-credentials states.

## Task Commits

Each task was committed atomically:

1. **Task 1: Route ChatWidget preset loading through the shared visibility helper** - `3a5387ef5` (feat)
2. **Task 2: Render provider sections and empty state in the model picker** - `74635dfb0` (feat)
3. **Task 3: Add smoke-helper and VT100 coverage for the live `/model` picker** - `8878e7cd0` (test)

**Plan metadata:** pending final docs/state commit

## Files Created/Modified
- `code-rs/tui/src/chatwidget.rs` - Shared visibility wiring for picker loading and `/model` command handling.
- `code-rs/tui/src/bottom_pane/model_selection_view.rs` - Provider-aware rendering, ordering, and empty-state behavior.
- `code-rs/tui/src/chatwidget/smoke_helpers.rs` - Test-only helper that opens the real model picker with seeded presets.
- `code-rs/tui/tests/vt100_chatwidget_snapshot.rs` - Live VT100 coverage for credentialed and empty picker states.
- `code-rs/tui/tests/snapshots/vt100_chatwidget_snapshot__model_selection_visibility__credentialed_provider_list.snap` - Credentialed picker snapshot baseline.
- `code-rs/tui/tests/snapshots/vt100_chatwidget_snapshot__model_selection_visibility__empty_credentials_hint.snap` - Empty-state picker snapshot baseline.

## Decisions Made
- Kept provider visibility centralized in `code_core::model_visibility` so the TUI and future entry points share the same auth rules.
- Preserved the OpenAI shortlist behavior inside the OpenAI provider bucket instead of spreading per-SKU exceptions into the picker.
- Used explicit empty-state copy so the picker does not appear broken when no provider credentials are available.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- The live VT100 frame needed a taller credentialed viewport so all provider buckets could appear in one snapshot.
- The empty-state snapshot needed to be verified against the actual rendered copy, which only exposed the top onboarding line at the chosen viewport height.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- The shared visibility helper is now wired into the TUI picker path.
- Provider grouping and empty-state rendering are covered by both unit tests and live snapshots.
- Later model-selection work can reuse the same helper without re-implementing provider gating.

---
*Phase: 05-dynamic-model-selection*
*Completed: 2026-05-12*

## Self-Check: PASSED
