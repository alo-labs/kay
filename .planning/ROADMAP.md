# Roadmap: code-monorepo

## Overview

This roadmap starts from a brownfield CLI monorepo. The near-term goal is to stabilize the project scaffolding, capture the current baseline, and keep the existing docs and build behavior intact so future phases can focus on product work rather than bootstrap drift.

## Phases

- [ ] **Phase 1: Baseline and Orientation** - Create and verify Silver Bullet and GSD scaffolding, capture the current project shape, and preserve existing docs.
- [ ] **Phase 2: Documentation Baseline** - Establish the canonical docs placeholders and record the initial monthly knowledge/lesson entries.
- [ ] **Phase 3: Workflow Hardening** - Verify the build, hooks, and release hygiene so normal development stays safe.

## Phase Details

### Phase 1: Baseline and Orientation
**Goal**: Establish a trustworthy init state and capture the brownfield baseline.
**Depends on**: Nothing
**Requirements**: init scaffolding, preserved docs, working GSD catalog
**Success Criteria** (what must be TRUE):
  1. Silver Bullet files exist and reference the active workflow.
  2. `.planning/PROJECT.md` and `.planning/STATE.md` describe the repo accurately.
  3. GSD can read the restored shared model catalog.
**Plans**: TBD

### Phase 2: Documentation Baseline
**Goal**: Create the canonical docs placeholders and record initial monthly knowledge.
**Depends on**: Phase 1
**Requirements**: docs scheme, workflow docs, knowledge/lessons files
**Success Criteria** (what must be TRUE):
  1. `docs/doc-scheme.*` and `docs/task-doc-checklist.json` exist.
  2. `docs/ARCHITECTURE.md`, `docs/TESTING.md`, and `docs/CHANGELOG.md` exist.
  3. `docs/knowledge/INDEX.md`, monthly knowledge, and lessons files exist.
**Plans**: TBD

### Phase 3: Workflow Hardening
**Goal**: Confirm the repo build and enforcement hooks are ready for normal work.
**Depends on**: Phase 2
**Requirements**: build-fast, hook registration, permission mode
**Success Criteria** (what must be TRUE):
  1. `./build-fast.sh` passes cleanly.
  2. SB hooks are registered in the user's global Claude settings.
  3. The repo can resume into normal development without init blockers.
**Plans**: TBD

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Baseline and Orientation | 0/TBD | Not started | - |
| 2. Documentation Baseline | 0/TBD | Not started | - |
| 3. Workflow Hardening | 0/TBD | Not started | - |
