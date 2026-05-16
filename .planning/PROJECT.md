# code-monorepo

## Current State

- v0.9.6 Kay Home Isolation shipped on 2026-05-16.
- The KAY_HOME root isolation implementation is archived and the release gate
  passed.
- The next milestone has not been started yet.

## Core Value

Keep the CLI buildable, understandable, and safe to evolve without disturbing
existing workflows.

## Current Milestone

None. Start a new milestone when the next release cycle is defined.

## Key Decisions

- Kay-first branding is the default for first-party surfaces.
- Legacy names remain only where compatibility or upstream comparison requires
  them.
- Rename decisions are tracked in a living inventory.
- Daily upstream reconciliation is part of the migration process.
- `KAY_HOME` is the canonical root for Kay-owned writable state when it is
  set.
- When `KAY_HOME` is unset, Kay uses its normal default home layout.
- Session, transcript, auth, skills, worktree, and debug-log paths should all
  resolve under the resolved Kay home tree.
- The milestone archive and requirements archive are stored under
  `.planning/milestones/`.

## Next Milestone Goals

- Start the next GSD milestone when new work is ready.
