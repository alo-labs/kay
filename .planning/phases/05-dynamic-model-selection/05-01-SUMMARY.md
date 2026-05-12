---
phase: 05-dynamic-model-selection
plan: 1
subsystem: auth
tags: [auth, model-visibility, regression-tests, rust]

# Dependency graph
requires:
  - phase: 04-provider-credential-crud
    provides: provider credential CRUD and auth resolution used to decide which providers are visible
provides:
  - reusable provider-aware visibility helper in `code-core`
  - public export for downstream picker consumers
  - unit and integration regressions for provider visibility and ordering
affects:
  - phase-05-dynamic-model-selection-plan-2
  - code-rs/tui/src/chatwidget.rs
  - code-rs/tui/src/bottom_pane/model_selection_view.rs

# Tech tracking
tech-stack:
  added: []
  patterns:
    - core-local visibility catalog with auth-gated provider buckets
    - exact namespace and exact model-id classification for provider families
    - unit plus integration regressions for auth-driven visibility rules

key-files:
  created:
    - code-rs/core/src/model_visibility.rs
    - code-rs/core/tests/model_visibility.rs
  modified:
    - code-rs/core/src/lib.rs

key-decisions:
  - "Defined a core-local `VisibleModelPreset` trait and generic bucket catalog so `code-core` can expose the helper without depending on `code-common`."
  - "Used `AuthManager::auth` for OpenAI visibility and `AuthManager::provider_api_key` for OpenCode Go / MiniMax gating."
  - "Locked provider order as OpenCode Go -> MiniMax -> OpenAI and kept matching strict: exact `opencode-go/...` namespace and exact `MiniMax-M2.7` model-id checks."
  - "Added integration coverage for save/remove key transitions so the helper stays reusable for future picker and API consumers."

patterns-established:
  - "Pattern 1: provider bucket catalogs flatten into a stable visible preset list in fixed provider order."
  - "Pattern 2: visibility should be driven from saved auth state, not provider CRUD state or ad hoc allowlists."
  - "Pattern 3: regression tests should prove both the filtered output and the classification path."

requirements-completed: [MODEL-01, PLUG-01]

# Metrics
duration: 33m
completed: 2026-05-12
---

# Phase 5: Dynamic Model Selection Summary

**Reusable provider-aware model visibility catalog in `code-core`, with exact OpenCode Go and MiniMax matching plus auth-gated OpenAI filtering**

## Performance

- **Duration:** 33m
- **Started:** 2026-05-12T12:57:30Z
- **Completed:** 2026-05-12T13:30:33Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added `code_core::model_visibility` as a reusable provider bucket catalog with stable OpenCode Go, MiniMax, OpenAI ordering.
- Wired visibility through existing auth resolution so OpenAI follows `AuthManager::auth`, while OpenCode Go and MiniMax follow stored provider API keys.
- Added regression coverage for exact namespace matching, exact MiniMax model matching, provider-key gating, and flattened provider order.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the reusable provider-aware visibility helper** - `6277a75ea` (`feat`)
2. **Task 2: Add regression tests that lock provider visibility and ordering** - `0c6b05ba3` (`test`)

## Files Created/Modified
- `code-rs/core/src/model_visibility.rs` - Generic provider visibility catalog and helper logic.
- `code-rs/core/src/lib.rs` - Public export for the new helper module.
- `code-rs/core/tests/model_visibility.rs` - Integration regressions for credential-driven visibility and ordering.

## Decisions Made
- Kept the helper core-local and generic over a small visibility trait so `code-core` does not need a cyclic dependency on `code-common`.
- Treated provider visibility as credential-driven and not CRUD-driven, using saved auth state only.
- Preserved a locked provider order of OpenCode Go, MiniMax, then OpenAI.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Avoided a core/common dependency cycle by using a core-local visibility trait**
- **Found during:** Task 1 (Add the reusable provider-aware visibility helper)
- **Issue:** The plan context referenced `code_common::model_presets::ModelPreset`, but `code-common` already depends on `code-core`, so importing `ModelPreset` directly from core would create a dependency cycle.
- **Fix:** Implemented `VisibleModelPreset` and generic provider bucket helpers in `code_core::model_visibility` instead of a direct `ModelPreset` dependency.
- **Files modified:** `code-rs/core/src/model_visibility.rs`
- **Verification:** `cargo +1.90.0 test -p code-core model_visibility -- --nocapture` and `./build-fast.sh`
- **Committed in:** `6277a75ea` (part of Task 1 commit)

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** No scope creep. The helper remains reusable and downstream consumers can adapt their preset type to the trait.

## Issues Encountered
- The shell did not have a default Rust toolchain selected, so verification used the repo-pinned `1.90.0` toolchain explicitly via `cargo +1.90.0`.
- The plan text mentioned `AuthManager::from_code_home`, but the current `AuthManager` API uses `shared_with_mode_and_originator`; the tests used the available constructor without changing behavior.

## Next Phase Readiness
- The shared helper and regressions are in place for the TUI picker wiring in plan 2.
- No blockers remain in `code-core`; downstream consumers can now build on the exported helper.

## Self-Check: PASSED

 - Summary file exists at `.planning/phases/05-dynamic-model-selection/05-01-SUMMARY.md`.
 - Task commit `6277a75ea` is present in git history.
 - Task commit `0c6b05ba3` is present in git history.

---
*Phase: 05-dynamic-model-selection*
*Completed: 2026-05-12*
