# Documentation Scheme

Silver Bullet uses this file as the human-readable policy companion to `docs/doc-scheme.json`.

## Current Shape

- Canonical docs for this repo live under `docs/`
- Brownfield docs that already exist are preserved
- New SB-scaffolded docs are added alongside the existing docs tree instead of replacing it
- `docs/task-doc-checklist.json` is the freshness ledger: `updated` means the file was touched in the current session, and the ledger itself must be refreshed in the same session as any governed docs it covers
- For narrow runtime work, the freshness ledger should reflect the docs that were actually refreshed in the current session, plus any directly affected governed docs
- Bootstrap inventory docs remain governed, but they should not block unrelated runtime work unless the change touches their documented behavior
- The docs scheme files are mandatory only when governance, inventory, or checklist policy changes; runtime tasks should not mark them `updated` unless they were actually edited

## Governed Documents

The initial SB docs set for this init includes:

- `docs/ARCHITECTURE.md`
- `docs/TESTING.md`
- `docs/CHANGELOG.md`
- `docs/knowledge/INDEX.md`
- `docs/knowledge/2026-05.md`
- `docs/lessons/2026-05.md`
- `docs/workflows/full-dev-cycle.md`
- `docs/workflows/devops-cycle.md`
- `docs/doc-scheme.md`
- `docs/doc-scheme.json`
- `docs/slash-commands.md`
- `docs/task-doc-checklist.json`

## Maintenance Rules

- Keep monthly knowledge and lessons files append-only
- Update the docs scheme and checklist together
- Do not delete user-authored docs during bootstrap or reconciliation
- Refresh the governed docs in the same session as the checklist so `updated` means "touched now," not "listed only"
- Do not mark `docs/doc-scheme.md` or `docs/doc-scheme.json` as updated for ordinary runtime work unless the governance contract itself changed
- When config or provider behavior changes, update the docs that describe the behavior and the docs that describe the workflow gate together
- When install or release packaging changes, keep [`README.md`](../README.md) and `docs/install.md` aligned with the actual shipped assets and install channels
- For live E2E harnesses, prefer a trusted clean clone of the seed repo so model behavior is measured against the repo's HEAD state rather than a dirty working tree
