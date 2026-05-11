# code-monorepo

## What This Is

A Rust/Node.js monorepo for the Codex CLI and its supporting workflows, docs, and release tooling. This is a brownfield repository, so initialization must preserve the existing docs, hooks, and build expectations rather than replacing them.

## Core Value

Keep the CLI buildable, understandable, and safe to evolve without disturbing existing workflows.

## Requirements

### Validated

(None yet - bootstrap in progress)

### Active

- Restore Silver Bullet scaffolding without deleting user content.
- Keep the Rust workspace buildable with `./build-fast.sh`.
- Maintain compatibility between GSD, Silver Bullet, and the repo's existing docs.

### Out of Scope

- Large-scale architectural rewrites - not part of init.
- Deleting or renaming existing docs - preserve them.

## Context

- Git repo: https://github.com/alo-labs/kay.git
- Existing code lives under `code-rs/` with root-level docs and supporting workflow files.
- `CLAUDE.md` predated Silver Bullet and now has the SB reference line added.
- GSD is installed, and the shared model catalog compatibility file has been restored.
- A brownfield codebase map now exists under `.planning/codebase/`.

## Constraints

- **Build**: `./build-fast.sh` is the required validation gate.
- **Safety**: No destructive git or docs operations during init.
- **Docs**: Existing docs stay intact; SB adds new canonical docs alongside them.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| `issue_tracker = gsd` | Matches the init default for this repo | ✓ Current |
| `active_workflow = full-dev-cycle` | Default developer workflow | ✓ Current |
| `permissions.defaultMode = auto` | Recommended for normal development | ✓ Current |

---
*Last updated: 2026-05-11 after Silver Bullet init*
