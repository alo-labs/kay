# Changelog

Rolling task log for the documentation and workflow surface.

## 2026-05-24

- Added Xiaomi as a built-in provider with `xiaomi/mimo-v2.5-pro` and
  `xiaomi/mimo-v2.5`, including config, picker, preset, and live acceptance
  coverage.
- Added a MiMo-specific synthesis checkpoint so `opencode-go/mimo-v2.5-pro`
  gets explicit anti-loop investigation guidance during multi-file debugging
  turns.
- Reduced OpenCode Go stream-idle tolerance so dead provider streams retry
  inside the live-turn window instead of leaving MiMo stuck on `Thinking...`.
- Omitted the discoverable skills catalog from MiMo prompts to keep live
  investigation turns focused on project instructions and immediate evidence.

## [0.9.0] — 2026-05-13

- Added the isolated `~/.kay` runtime and stopped Kay from inheriting a local Codex environment by default.
- Added the real `projects/test-notes-app` live validation harness for OpenCode Go model runs.
- Added the transcript viewer and CLI transcript command for readable JSONL provenance review.
- Hardened the OpenCode Go provider and model-family behavior used by the live notes-app workflow.

---

## 2026-05-13

- Added the isolated test-notes-app live E2E harness for Kay provider-model acceptance runs
- Reconciled the Silver Bullet session-marker path used by the docs gate during the harness work

## 2026-05-11

- Rewrote the OpenCode Go Phase 1 plan to stay foundation-only and modular, with docs/live/release work deferred to later phases
- Refreshed the governed docs checklist after the Phase 1 plan rewrite so the docs-scheme gate sees current-session updates
- Initialized Silver Bullet enforcement scaffolding
- Restored the shared GSD model catalog compatibility file
- Added canonical docs placeholders, workflow docs, and the docs governance contract
- Added the built-in `opencode-go` provider, login guidance, and provider-slug normalization for matching namespaced model requests
- Completed the Kay slash-command rename path so `/kay` is the canonical prompt-expanding command across the UI, formatter, and docs
- Refreshed the docs-scheme ledger and governed docs together after the final Kay slash-command review so the completion gate sees current-session updates
