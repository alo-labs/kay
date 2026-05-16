---
phase: 09-runtime-paths-scripts-and-binary-renames
plan: 1
subsystem: infra
tags: [bash, justfile, docs, packaging, kay]
requires:
  - phase: 08-kay-first-docs-and-copy-sweep
    provides: Kay-first docs and inventory baseline
provides:
  - Kay-first build and release entrypoints with compatibility fallbacks
  - Updated justfile and generator helper to route through Kay names
  - Manual compatibility review notes for config, agents, advanced, and integration docs
  - Refreshed rename inventory and docs freshness ledger
affects: [phase 10 boundary audit and validation, release scripts, developer entrypoints, docs governance]
tech-stack:
  added: []
  patterns:
    - Kay-primary wrapper scripts with compatibility shims for legacy names
    - Inventory-first tracking of retained tokens in runtime and docs sweeps
key-files:
  created:
    - scripts/check-kay-path-deps.sh
    - scripts/start-kay-exec.sh
    - .planning/phases/09-runtime-paths-scripts-and-binary-renames/09-01-SUMMARY.md
  modified:
    - build-fast.sh
    - code-rs/justfile
    - code-rs/protocol-ts/generate-ts
    - docs/advanced.md
    - docs/integration-zed.md
    - docs/kay-brand-renaming-inventory.md
    - docs/task-doc-checklist.json
    - scripts/check-codex-path-deps.sh
requirements-completed: [COMP-01, SYNC-01, SYNC-02]
key-decisions:
  - "Kay is the primary entrypoint name for just recipes and build-time helpers; legacy codex/code entrypoints remain as wrappers."
  - "Compatibility boundaries stay explicit in docs and the rename inventory instead of being hidden behind a broad mechanical rewrite."
patterns-established:
  - "Primary Kay wrapper plus compatibility shim for legacy script names"
  - "Inventory-based tracking of retained tokens in runtime and docs sweeps"
duration: 18m
completed: 2026-05-16
---

# Phase 9: Runtime Paths, Scripts, and Binary Renames Summary

Kay-first developer entrypoints and runtime docs now lead the migration, while legacy names remain explicit compatibility boundaries.

## Performance

- **Duration:** 18m
- **Started:** 2026-05-16T14:15:05Z
- **Completed:** 2026-05-16T14:33:44Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- `build-fast.sh` now defaults to `kay`, recognizes `KAY_HOME`, and uses the Kay-named path guard while preserving compatibility aliases.
- `code-rs/justfile` now exposes `kay` as the primary recipe and retains `codex`/`code` wrappers.
- `code-rs/protocol-ts/generate-ts` now routes through `just kay`.
- `docs/advanced.md` and `docs/integration-zed.md` explicitly call out the remaining compatibility-named `code` and `code-mcp-server` boundaries.
- The rename inventory and docs freshness ledger now track the new wrappers and retained legacy tokens.

## Verification

- `./build-fast.sh` passed cleanly.
- `jq empty docs/task-doc-checklist.json` passed.

## Task Commits

None - no git commits were created in this session.

## Files Created/Modified

- `build-fast.sh` - Kay-first build gate and compatibility alias handling.
- `code-rs/justfile` - Primary `kay` recipes with legacy `codex`/`code` wrappers.
- `code-rs/protocol-ts/generate-ts` - Kay-first just invocation for TS generation.
- `scripts/check-kay-path-deps.sh` - Kay-named path dependency guard.
- `scripts/check-codex-path-deps.sh` - Compatibility wrapper to the Kay guard.
- `scripts/start-kay-exec.sh` - Kay-named wrapper for the legacy exec helper.
- `docs/advanced.md` - Clarified the legacy `code` MCP tool as a compatibility boundary.
- `docs/integration-zed.md` - Marked `code-mcp-server` as the compatibility-named server binary.
- `docs/kay-brand-renaming-inventory.md` - Added the new wrappers and kept the legacy entries explicit.
- `docs/task-doc-checklist.json` - Refreshed the docs freshness ledger.

## Decisions Made

- Use Kay-first names for the primary developer entrypoints, but keep explicit compatibility wrappers for legacy callers.
- Document the remaining `code` and `codex` tokens instead of hiding them, so phase 10 can review the boundary list deliberately.

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 10 can begin with the current compatibility inventory and Kay-first runtime entrypoints already in place.

---
*Phase: 09-runtime-paths-scripts-and-binary-renames*
*Completed: 2026-05-16*
