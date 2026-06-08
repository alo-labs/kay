# Changelog

Rolling task log for the documentation and workflow surface.

## 2026-06-08

- Closed obsolete release-asset issue #13 after confirming the current latest
  release publishes installable platform assets.
- Hardened `kay exec` compatibility with `--ask-for-approval`, keeping headless
  defaults intact while allowing explicit approval-policy overrides.
- Repaired MiniMax-M3's observed malformed `ls && -la && <path>` shell probe at
  the shared command-normalization boundary so allowed edit workflows do not
  waste turns on invalid `-la` command execution.
- Strengthened skill instructions so exact named workflow requests such as
  `silver:init` must execute that named skill instead of drifting into adjacent
  scan/discovery paths.

## 2026-06-07

- Added the Hermes-compatible provider profile import path, built-in
  OpenRouter registration, and Bedrock Converse provider-profile recognition
  so Kay can ingest future Hermes Agent provider definitions while keeping the
  normal runtime Rust-only.

## 2026-05-31

- Wired `kay exec --output-schema` into the user-turn schema path and taught
  OpenAI-compatible Chat Completions requests to use native `response_format`
  only for model families that support it. MiMo now uses shared schema guidance
  across OpenCode Go and direct Xiaomi so direct Xiaomi does not stall on native
  structured-output transport and tool-capable turns still perform required
  edits before final JSON.
- Added a hard per-call timeout to live provider acceptance checks so repeated
  provider stream disconnect retries fail the release gate instead of wedging
  pre-release indefinitely.
- Kept recovered stream-retry notifications from making `kay exec` exit
  nonzero after the turn successfully completes, preserving fatal exit behavior
  for non-retry errors.
- Increased the OpenCode Go onboarding live-smoke turn budget so curated MiMo
  release coverage can survive normal multi-window SSE retry recovery.
- Taught the shared shell-tool parser to recover MiMo's observed concatenated
  JSON tool-argument objects, including string-form commands, by executing the
  commands as one quoted shell script instead of trapping the turn in repeated
  parse-error retries.
- Gave the direct Xiaomi provider the standard five-minute streaming idle window
  so slower first-token MiMo turns are not reset every 60 seconds.
- Added MiMo model-family recovery guidance for repeated `apply_patch` context
  failures so models switch to a smaller exact patch or bounded file rewrite
  instead of stalling on the same failed hunk.
- Normalized MiMo-style `apply_patch` hunk labels with trailing `@@` markers in
  the shared shell-tool path and made the MiMo family instructions explicitly
  reject that malformed header shape.
- Added MiMo-family final-output schema repair so an early non-JSON assistant
  progress message during a tool workflow is treated as a recoverable turn
  error instead of completing `kay exec` with an invalid final message.
- Hardened shared MiMo model-family tool guidance so OpenCode Go and direct
  Xiaomi MiMo models use the same apply-patch grammar, tool-call, and final
  output contract behavior.
- Extended the live notes-app harness for direct Xiaomi MiMo runs to validate
  real tracked file edits, duplicate-note behavior markers, and JavaScript
  syntax instead of accepting deterministic fallback patches, including common
  inline typing guards for the duplicate keyboard shortcut.

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
