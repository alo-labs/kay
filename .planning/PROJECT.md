# code-monorepo

## Current State

- v0.9.14 is the current shipped release.
- v0.9.15 Kay Rust Workspace Rename is active.
- The milestone is a path-only migration from `code-rs/` to `kay-rs/` while
  preserving current crate names, imports, binaries, protocol names, and
  compatibility command aliases.

## Core Value

Keep the CLI buildable, understandable, and safe to evolve without disturbing
existing workflows.

## Current Milestone: v0.9.15 Kay Rust Workspace Rename

**Goal:** Rename the active Kay Rust workspace directory from `code-rs/` to
`kay-rs/` without changing Rust crate identities or shipped binary
compatibility.

**Target features:**
- Move the active Rust workspace to `kay-rs/` with no `code-rs` filesystem
  symlink.
- Update first-party build, CI, release, cleanup, upstream-sync, source-test,
  and documentation references to the new workspace path.
- Preserve `code-*` crate names, Rust imports, generated protocol names, and
  compatibility binaries for a later phased rename.

## Key Decisions

- Kay-first branding is the default for first-party surfaces.
- Legacy names remain only where compatibility or upstream comparison requires
  them.
- Rename decisions are tracked in a living inventory.
- The `code-rs/` to `kay-rs/` migration is path-only; crate/package/import
  renames are deferred to later milestones.
- No tracked or generated `code-rs -> kay-rs` compatibility symlink will remain
  after the workspace path migration.
- Daily upstream reconciliation is part of the migration process.
- `KAY_HOME` is the canonical root for Kay-owned writable state when it is
  set.
- When `KAY_HOME` is unset, Kay uses its normal default home layout.
- Session, transcript, auth, skills, worktree, and debug-log paths should all
  resolve under the resolved Kay home tree.
- The milestone archive and requirements archive are stored under
  `.planning/milestones/`.

## Next Milestone Goals

- Complete Phase 12 and validate the renamed workspace with the required
  build, focused script checks, TypeScript SDK path tests, and pre-release
  gate before pushing to `main`.
