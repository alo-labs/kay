---
gsd_state_version: 1.0
milestone: v0.8.0
milestone_name: Provider CRUD and Dynamic Model Selection
current_plan: 2
status: executing
stopped_at: Completed 05-01-PLAN.md
last_updated: "2026-05-12T13:40:15.538Z"
last_activity: 2026-05-12
progress:
  total_phases: 3
  completed_phases: 1
  total_plans: 6
  completed_plans: 5
  percent: 83
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-12)

**Core value:** Keep the CLI buildable, understandable, and safe to evolve without disturbing existing workflows.
**Current focus:** Phase 05 — Dynamic Model Selection

## Current Position

Phase: 05 (Dynamic Model Selection) — EXECUTING
Plan: 2 of 2
Current Plan: 2
Total Plans in Phase: 2
Status: Ready to execute
Last activity: 2026-05-12

## Performance Metrics

**Velocity:**

- Total plans completed: 5
- Average duration: 38m
- Total execution time: 3.2 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 4. Provider Credential CRUD | 4/4 | Complete | 2026-05-12 |
| 5. Dynamic Model Selection | 1/2 | In Progress | - |
| 6. Verification, Docs, and Release | 0/TBD | Not started | - |

**Recent Trend:**

- Last 5 plans: 30m, 1h 35m, 14m, 18m, 33m
- Trend: Stable

*Updated after each plan completion*
| Phase 04-provider-credential-crud P1 | 30m | 2 tasks | 6 files |
| Phase 04-provider-credential-crud P2 | 1h 35m | 2 tasks | 5 files |
| Phase 04-provider-credential-crud P3 | 14m | 2 tasks | 6 files |
| Phase 04-provider-credential-crud P4 | 18m | 2 tasks | 4 files |
| Phase 05-dynamic-model-selection P1 | 33m | 2 tasks | 3 files |

## Accumulated Context

### Decisions

- [Init] issue_tracker=gsd, active_workflow=full-dev-cycle, permissions.auto enabled
- [Milestone v1.0] OpenCode Go was treated as a first-class provider integration and validated with a representative `opencode-go/<model>` path before release.
- [Milestone v0.8.0] `/provider` is the canonical provider CRUD surface; `/model` should filter by configured provider credentials; provider plugins and model plugins stay orthogonal.
- [Phase 04-provider-credential-crud]: Restore --api-key as the direct-argument login path while keeping --with-api-key stdin compatibility
- [Phase 04-provider-credential-crud]: Trim and reject empty direct API-key input before dispatching to auth helpers
- [Phase ?]: Delete provider credentials only from auth.json.provider_credentials and leave the dedicated OPENAI_API_KEY field untouched.
- [Phase ?]: Normalize provider ids before mutation and treat missing auth files or missing provider entries as no-op deletes.
- [Phase 4]: Delete provider credentials only from auth.json.provider_credentials and leave the dedicated OPENAI_API_KEY field untouched.
- [Phase 4]: Normalize provider ids before mutation and treat missing auth files or missing provider entries as no-op deletes.
- [Phase 4]: Route provider credential writes through the existing atomic auth.json writer.
- [Phase 04-provider-credential-crud]: /provider stays separate from /login and /model — Keeping provider CRUD orthogonal avoids cross-flow regressions in existing account and model behavior.
- [Phase 04-provider-credential-crud]: Provider order is explicit: OpenCode Go, MiniMax, OpenAI — A fixed order preserves the user-facing contract and avoids hash/map iteration drift.
- [Phase 04-provider-credential-crud]: Provider add/update reuses existing auth save helpers and metadata hints — Reusing the existing auth path keeps config mutation consistent and avoids duplicate parsing or persistence logic.
- [Phase 04-provider-credential-crud]: Keep provider deletion inside the provider pane with an explicit confirm/cancel state instead of routing it through /login. — Destructive actions stay explicit and localized to provider CRUD.
- [Phase 04-provider-credential-crud]: Use a test-only harness helper to open the provider overlay directly because the VT100 harness does not dispatch slash commands the same way the live app does. — The snapshot test needs a deterministic way to reach the provider pane without relying on slash-command routing.
- [Phase 05-dynamic-model-selection]: Defined a core-local VisibleModelPreset trait and generic provider bucket catalog so code-core can expose the helper without depending on code-common — Avoids a core/common dependency cycle while keeping the visibility seam reusable.
- [Phase 05-dynamic-model-selection]: Used AuthManager::auth for OpenAI visibility and AuthManager::provider_api_key for OpenCode Go and MiniMax gating — Matches the existing auth resolution paths for each provider family.
- [Phase 05-dynamic-model-selection]: Locked provider order as OpenCode Go -> MiniMax -> OpenAI and kept matching strict — Prevents provider ordering drift and enforces exact namespace and model-id classification.
- [Phase 05-dynamic-model-selection]: Added integration coverage for save/remove key transitions so the helper stays reusable for future picker and API consumers — Proves the helper stays credential-driven as auth state changes.

### Pending Todos

None yet.

### Blockers/Concerns

None.

## Deferred Items

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| *(none)* | | | |

## Session Continuity

Last session: 2026-05-12T13:40:09.472Z
Stopped at: Completed 05-01-PLAN.md
Resume file: 05-02-PLAN.md
