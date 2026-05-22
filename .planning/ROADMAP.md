# Roadmap: code-monorepo

## Current Milestone: v0.9.15 Kay Rust Workspace Rename

This milestone completes a path-only rename of the active Kay Rust workspace
from `code-rs/` to `kay-rs/`.

## Phases

### Phase 12: Kay Rust workspace path migration

**Goal:** Move the active Kay Rust workspace to `kay-rs/`, update first-party
tooling and docs to that path, and preserve current crate, import, protocol,
and binary compatibility.
**Depends on:** v0.9.14 shipped release metadata
**Plans:** 1 plan

Plans:

- [ ] 12-01: Rename active Rust workspace path from `code-rs/` to `kay-rs/`

**Requirements:** PATH-01, PATH-02, PATH-03, COMPAT-01, COMPAT-02, COMPAT-03,
AUTO-01, AUTO-02, DOC-01, TEST-01

**Success criteria:**
1. `git ls-files` shows the active Rust workspace under `kay-rs/` and no
   tracked `code-rs/` workspace path remains.
2. `./build-fast.sh`, `./build-fast.sh --workspace kay-rs`, and
   `./build-fast.sh --workspace code-rs` all build the Kay workspace.
3. First-party scripts, GitHub workflows, upstream-sync tooling, docs, and
   source tests use `kay-rs/` for Kay-owned paths.
4. `code-*` crate/package/import identities and compatibility binaries remain
   unchanged.
5. The path audit, focused checks, TypeScript SDK tests, and `./pre-release.sh`
   pass before pushing to `main`.

## Completed Milestones

- [v0.9.6 Kay Home Isolation](.planning/milestones/v0.9.6-ROADMAP.md) shipped
  on 2026-05-16.
- [v0.9.5 Kay Brand Renaming](.planning/milestones/v0.9.5-ROADMAP.md) shipped
  on 2026-05-16.
