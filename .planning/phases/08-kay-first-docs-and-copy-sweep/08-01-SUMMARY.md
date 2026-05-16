# Phase 08 Summary: Kay-first docs and copy sweep

Completed the first Kay brand migration sweep for the user-facing docs surface.

## What changed

- Rewrote the top-level onboarding and packaging docs to present Kay as the
  primary product brand:
  - `docs/getting-started.md`
  - `docs/homebrew.md`
  - `docs/faq.md`
  - `docs/install.md`
  - `docs/exec.md`
  - `docs/execpolicy.md`
  - `docs/skills.md`
  - `docs/integration-zed.md`
  - `docs/ARCHITECTURE.md`
  - `docs/tui-alternate-screen.md`
- Refreshed the rename inventory in `docs/kay-brand-renaming-inventory.md` to
  record the docs wave and keep the remaining compatibility-bound references
  visible.
- Synced `docs/task-doc-checklist.json` with the docs sweep.

## What stayed compatible

- Compatibility aliases and legacy paths remain where they are still part of
  the shipped interface or an upstream comparison boundary.
- Upstream and generated names such as `openai/codex`, `.codexpolicy`, and
  retained model slugs remain in place where they are intentionally
  compatibility-bound.

## Verification

- `./build-fast.sh` passed.
- `jq empty docs/task-doc-checklist.json` passed.

## Next wave

- Sweep the remaining user-facing config and agent docs where compatibility
  names still need manual review.
- Continue with the runtime paths, scripts, and binary rename phase after the
  docs-first surface is stable.
