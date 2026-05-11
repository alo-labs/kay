# Documentation Scheme

Silver Bullet uses this file as the human-readable policy companion to `docs/doc-scheme.json`.

## Current Shape

- Canonical docs for this repo live under `docs/`
- Brownfield docs that already exist are preserved
- New SB-scaffolded docs are added alongside the existing docs tree instead of replacing it

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
