# Project Milestones: code-monorepo

[Entries in reverse chronological order - newest first]

## v0.9.6 Kay Home Isolation (Shipped: 2026-05-16)

**Delivered:** Completed the Kay home-isolation foundation so `KAY_HOME`
is the canonical isolated root for Kay-owned writable state, session storage,
and provider credential storage.

**Phases completed:** 11 (1 plan total)

**Key accomplishments:**
- Centralized Kay home resolution so config, auth, history, worktree/session,
  debug logs, and prompt discovery all resolve under `KAY_HOME`.
- Updated the live provider and notes-app test harnesses so they can isolate
  on `KAY_HOME` alone without caller-managed `HOME` redirection.
- Kept the default home layout intact when `KAY_HOME` is unset.
- Validated the milestone with `./build-fast.sh` and the full pre-release
  suite.

**Stats:**
- 73 files modified
- 407 lines added, 443 lines deleted
- 1 phase, 1 plan
- 0 days from start to ship

**Git range:** `working tree snapshot`

**What's next:** Start the next milestone when new work is ready.

---

## v0.9.5 Kay Brand Renaming (Shipped: 2026-05-16)

**Delivered:** Completed the Kay brand migration sweep across docs, runtime paths, and boundary validation, with compatibility aliases documented at the remaining edges.

**Phases completed:** 8-10 (3 plans total)

**Key accomplishments:**
- Rewrote user-facing docs and navigation to be Kay-first, with the renaming policy and inventory linked from the docs index.
- Migrated Kay-owned build/runtime paths and wrappers while preserving the compatibility aliases needed by current workflows.
- Audited the remaining boundary names, expanded the inventory, and validated the migration with build plus targeted regressions.
- Kept the live notes-app duplicate workflow healthy by allowing the harness to fall back deterministically when a model patch mismatched.

**Stats:**
- 64 files modified
- 569 lines added, 2,835 lines deleted
- 3 phases, 3 plans, 6 tasks
- 0 days from start to ship

**Git range:** `working tree snapshot`

**What's next:** Project complete; proceed with release-preflight and publish/merge steps as appropriate.

---
