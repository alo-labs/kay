---
gsd_state_version: 1.0
milestone: v0.9.6
milestone_name: Kay Home Isolation
status: Shipped
last_updated: "2026-05-16T18:59:23Z"
last_activity: 2026-05-16 — v0.9.6 archived, tagged, and released after the pre-release gate passed
progress:
  total_phases: 1
  completed_phases: 1
  total_plans: 1
  completed_plans: 1
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-16)

**Core value:** Keep the CLI buildable, understandable, and safe to evolve
without disturbing existing workflows.
**Current focus:** Release complete; start the next milestone when ready

## Current Position

Phase: complete
Plan: complete
Status: shipped
Last activity: 2026-05-16 — v0.9.6 archived, tagged, and released after the pre-release gate passed

## Current Context

### Decisions

- [Init] issue_tracker=gsd, active_workflow=full-dev-cycle, permissions.auto enabled
- [Provider credential CRUD] Restore --api-key as the direct-argument login path while keeping --with-api-key stdin compatibility
- [Provider credential CRUD] Trim and reject empty direct API-key input before dispatching to auth helpers
- [Provider credential CRUD] Delete provider credentials only from auth.json.provider_credentials and leave the dedicated OPENAI_API_KEY field untouched.
- [Provider credential CRUD] Normalize provider ids before mutation and treat missing auth files or missing provider entries as no-op deletes.
- [Provider credential CRUD] Route provider credential writes through the existing atomic auth.json writer.
- [Provider CRUD] /provider stays separate from /login and /model — Keeping provider CRUD orthogonal avoids cross-flow regressions in existing account and model behavior.
- [Dynamic model selection] Defined a core-local VisibleModelPreset trait and generic provider bucket catalog so code-core can expose the helper without depending on code-common — Avoids a core/common dependency cycle while keeping the visibility seam reusable.
- [Dynamic model selection] Used AuthManager::auth for OpenAI visibility and AuthManager::provider_api_key for OpenCode Go and MiniMax gating — Matches the existing auth resolution paths for each provider family.
- [Dynamic model selection] Locked provider order as OpenCode Go -> MiniMax -> OpenAI and kept matching strict — Prevents provider ordering drift and enforces exact namespace and model-id classification.
- [Dynamic model selection] Added integration coverage for save/remove key transitions so the helper stays reusable for future picker and API consumers — Proves the helper stays credential-driven as auth state changes.
- [Dynamic model selection] Use the shared code_core::model_visibility helper as the source of truth for picker visibility.
- [Dynamic model selection] Keep the OpenAI shortlist curation inside the OpenAI bucket only, after provider grouping.
- [Dynamic model selection] Render provider headers in the fixed OpenCode Go, MiniMax, OpenAI order and show an explicit empty state when nothing is unlocked.
- [Brand migration] Kay-first branding is now the default for first-party surfaces; legacy names stay only where compatibility or upstream comparison requires them.
- [Brand migration] Daily upstream reconciliation is part of the migration process so common-file drift stays small and reviewable.
- [KAY_HOME isolation] KAY_HOME is the canonical isolated root for Kay-owned writable state when it is set.
- [KAY_HOME isolation] When KAY_HOME is unset, Kay uses its normal default home layout.
- [KAY_HOME isolation] Session, transcript, auth, skills, worktree, and debug-log paths should resolve beneath the resolved Kay home tree.
- [KAY_HOME isolation] The pre-release gate and archive docs confirm the KAY_HOME-only isolation path without caller-managed home redirection.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-05-16T18:59:23.000Z
Stopped at: Milestone archived and release tagged
Resume file: None
