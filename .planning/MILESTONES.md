# Project Milestones: code-monorepo

[Entries in reverse chronological order - newest first]

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
