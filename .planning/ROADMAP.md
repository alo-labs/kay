# Roadmap: code-monorepo

## Overview

This milestone bootstraps an isolated Kay end-user install under `~/.kay`, removes automatic inheritance from local Codex/Every Code state, and turns Kay into the live validation harness for a real `test-notes-app` project. The work adds a lightweight transcript viewer, creates a real GitHub-backed note app target, and exercises the supported OpenCode Go models on meaningful work before release.

## Phases

- [ ] **Phase 7: Bootstrap isolated test-notes-app note-taking project and Kay OCG live-testing harness** - Establish the isolated Kay home, transcript viewer, and live note-app validation project that future OCG model runs will drive.

## Phase Details

### Phase 7: Bootstrap isolated test-notes-app note-taking project and Kay OCG live-testing harness
**Goal**: Establish Kay's isolated end-user runtime, add a modern transcript viewer, and bootstrap the real `projects/test-notes-app` validation repo that the supported OpenCode Go models will build against.
**Depends on**: Nothing
**Requirements**: KAY-01, KAY-02, NOTE-01, VIEW-01, TEST-01, DOCS-01, REL-01
**Plans**: 3 plans
**Plan list**:
- [ ] 07-01-PLAN.md — Make Kay default to `~/.kay` and cut the Codex inheritance path
- [ ] 07-02-PLAN.md — Add a lightweight transcript viewer for JSONL provenance
- [ ] 07-03-PLAN.md — Bootstrap `alo-exp/test-notes-app` and wire the OCG validation harness
**Success Criteria** (what must be TRUE):
  1. The end-user Kay install uses its own isolated home under `~/.kay` instead of silently inheriting local Codex/Every Code state.
  2. Kay transcripts remain accessible as JSONL and a lightweight chat-like viewer can inspect them.
  3. A real `alo-exp/test-notes-app` repo exists and the supported OCG models can do meaningful validation work against it before release.

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 7. Bootstrap isolated test-notes-app note-taking project and Kay OCG live-testing harness | 0/3 | Not started | - |
