# Documentation Scheme

Silver Bullet uses this file as the human-readable policy companion to `docs/doc-scheme.json`.

## Current Shape

- Canonical docs for this repo live under `docs/`
- Brownfield docs that already exist are preserved
- New SB-scaffolded docs are added alongside the existing docs tree instead of replacing it
- `docs/task-doc-checklist.json` is the freshness ledger: `updated` means the file was touched in the current session
- For task-driven code changes, the freshness ledger should reflect any governed doc that documents the affected surface, such as provider registration or resume compatibility

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
- `docs/task-doc-checklist.json`

## Maintenance Rules

- Keep monthly knowledge and lessons files append-only
- Update the docs scheme and checklist together
- Do not delete user-authored docs during bootstrap or reconciliation
- Refresh the governed docs in the same session as the checklist so `updated` means "touched now," not "listed only"
- When config or provider behavior changes, update the docs that describe the behavior and the docs that describe the workflow gate together
