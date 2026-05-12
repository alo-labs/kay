# Roadmap: code-monorepo

## Overview

This milestone adds provider CRUD and provider-aware model selection to Kay. Users will be able to add API keys without editing config files, select from the supported provider set in the app, and see only the models enabled by configured provider credentials. The work preserves the existing multi-provider architecture and keeps provider plugins orthogonal to reusable model plugins.

## Phases

- [x] **Phase 4: Provider Credential CRUD** - Add the `/provider` command, provider ordering, and CLI API-key entry for the supported providers. (completed 2026-05-12)
- [ ] **Phase 5: Dynamic Model Selection** - Make `/model` list only models enabled by configured provider credentials and keep model compatibility profiles reusable.
- [ ] **Phase 6: Verification, Docs, and Release** - Prove the provider-management flow with tests, document it, and cut a release after verification.

## Phase Details

### Phase 4: Provider Credential CRUD
**Goal**: Let users create, read, update, and delete provider credentials from within Kay and through the CLI without editing config files.
**Depends on**: Nothing
**Requirements**: PROVIDER-01, PROVIDER-02, AUTH-01
**Plans**: 4 plans
**Plan list**:
- [x] 04-01-PLAN.md — Add provider auth CRUD helpers and tests
- [x] 04-02-PLAN.md — Restore direct CLI API-key entry
- [x] 04-03-PLAN.md — Add `/provider` command and provider pane shell
- [x] 04-04-PLAN.md — Finish delete action and TUI regressions
**Success Criteria** (what must be TRUE):
  1. `/provider` can manage the supported provider set in the required order: OpenCode Go, MiniMax, OpenAI.
  2. API keys can be supplied directly via CLI argument and saved without manual config-file edits.
  3. Existing provider auth behavior remains intact while the new CRUD surface is added.

### Phase 5: Dynamic Model Selection
**Goal**: Filter `/model` by configured provider credentials and keep model-specific compatibility profiles reusable across multiple models.
**Depends on**: Phase 4
**Requirements**: MODEL-01, MODEL-02, PLUG-01
**Success Criteria** (what must be TRUE):
  1. `/model` shows only models for providers with configured credentials.
  2. OpenCode Go shows the supported OpenCode Go list, MiniMax shows M2.7, and OpenAI shows the upstream-supported OpenAI models.
  3. Provider plugins and model plugins remain orthogonal, with shared compatibility profiles reused across model families.

### Phase 6: Verification, Docs, and Release
**Goal**: Prove the provider-management and model-selection flow with automated tests, document it, and release the result.
**Depends on**: Phase 5
**Requirements**: TEST-01, DOCS-01, REL-01
**Success Criteria** (what must be TRUE):
  1. Automated tests cover provider CRUD, CLI credential entry, provider-aware model filtering, and compatibility-profile reuse.
  2. Docs explain `/provider`, CLI API-key entry, and the provider-aware `/model` behavior.
  3. A Kay release is cut only after the new workflow has been verified.

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 4. Provider Credential CRUD | 4/4 | Complete   | 2026-05-12 |
| 5. Dynamic Model Selection | 0/TBD | Not started | - |
| 6. Verification, Docs, and Release | 0/TBD | Not started | - |
