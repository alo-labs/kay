## @alo-labs/kay v0.9.44

Patch release: adds first-class OpenCode Go **Qwen3.7 Max** (`qwen3.7-max`) support
with Qwen tool-discipline guidance and apply_patch repairs matching MiMo/MiniMax
OCG profiles.

### Model Families

- Register `opencode-go/qwen3.7-max` preset, visibility whitelist, and provider
  acceptance lists (`qwen3.7-max` wire slug).
- Extend the `qwen` model family with apply_patch instructions, malformed
  tool-call repairs, and Sidekick verify-script closeout discipline.

### Release And Verification

- Sidekick Kay live matrix **5/5 PASS** for `ocg-qwen` (`qwen3.7-max`) ×
  `e2e`, `task7`–`task10` (`prefix: qwen-ocg-v2`).
- Verified locally with `./build-fast.sh` and
  `KAY_PRE_RELEASE_LIVE_PROVIDER_GATE=minimax-m3 ./pre-release.sh`.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay --version
```

Compare: https://github.com/alo-labs/kay/compare/v0.9.43...v0.9.44

## @alo-labs/kay v0.9.41

Patch release after v0.9.40: fixes OpenCode Go `minimax-m3` live acceptance for
e2e and task8 by separating generic turn-continue nudges from task8 verify-script
closeout guidance and repairing malformed `bash -lc` argv splits.

### Exec Reliability

- Use context-aware turn-continue nudges so e2e prompts are not steered toward
  task8 verify-script closeout when the model narrates without tool calls
  (`836cc9bf`).
- Repair `bash -lc` argv splits and over-quoted `cat|apply_patch` shell
  invocations that reproduce on OpenCode Go `minimax-m3` (`836cc9bf`).

### Release And Verification

- Closes [#79](https://github.com/alo-labs/kay/issues/79); Sidekick Kay live
  matrix **task8 `ocg-minimax-m3` PASS**.
- Verified locally with `./build-fast.sh` and
  `KAY_PRE_RELEASE_LIVE_PROVIDER_GATE=minimax-m3 ./pre-release.sh`.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay --version
```

Compare: https://github.com/alo-labs/kay/compare/v0.9.40...v0.9.41

## @alo-labs/kay v0.9.40

Sidekick live matrix release: closes the MiMo/MiniMax task8–9 closeout gap with
STATUS-contract nudges, verify-script narration recovery, and model-family
guidance for exact `bulkArchiveButton` / `sortSelect` element ids.

### Exec Reliability

- Nudge premature `STATUS: BLOCKED` when verify scripts are still required so
  task8/9 can finish `notes-ui.js` wiring instead of stopping early
  (`2536c884`, `c2944f6a`, `e7488abd`, `e9981614`).
- Normalize trailing punctuation on `STATUS:` heads and keep repair-attempt 2 from
  forbidding tool calls while verify work remains (`c2944f6a`).
- Extend turn-continue nudges to e2e and other non-STATUS prompts when the model
  narrates mid-task without tool calls (`e9981614`).

### Model Families

- MiMo and MiniMax profiles include Sidekick verify-script grep contracts for
  `bulkArchiveButton`, `note-checkbox`, `sortSelect`, and `params.set('sort'`
  (`43b16e26`).

### Release And Verification

- Sidekick Kay live matrix **20/20 PASS** (`local-fix-r8` + `local-fix-r9`
  retest): profiles `ocg-minimax-m3`, `ocg-mimo-pro`, `ocg-mimo`, `minimax-m3`
  × tasks `e2e`, `task7`–`task10`.
- Verified locally with `./build-fast.sh` and
  `KAY_PRE_RELEASE_LIVE_PROVIDER_GATE=minimax-m3 ./pre-release.sh`.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay --version
```

Compare: https://github.com/alo-labs/kay/compare/v0.9.39...v0.9.40

## @alo-labs/kay v0.9.31

Patch release after v0.9.30: improves Sidekick `STATUS:` detection for compact
final prompts and extends the GitHub issue monitor so post-close activity on
already-handled issues is tracked instead of silently ignored.

### Exec Reliability

- Detect compact Sidekick `STATUS:` prompts in final assistant messages so
  `kay exec` honors `STATUS: SUCCESS` / `STATUS: BLOCKED` contracts when the
  model omits the usual multi-line framing (`ef2ae84e`).

### Operations

- `scripts/issue-monitor-check.sh` tracks `closed_activity` on issues that were
  already in `handled_issue_numbers`, so follow-up comments after a fix lands do
  not leave the monitor blind (`5b98a8b3`).

### Release And Verification

- Doc-scheme gate: `release-notes-body` remediation keeps categorized bodies in this file aligned with slash-command release guidance.
- Published package metadata for `@alo-labs/kay` 0.9.31 (`f554de1c`).
- Verified locally with `./build-fast.sh` and `./pre-release.sh` (MiniMax-M3 live
  provider gate).
- Doc-scheme gate: `release-notes-body` (granularity 2) keeps this file, `docs/slash-commands.md`, and `docs/task-doc-checklist.json` fresh in the same session.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay --version
```

Compare: https://github.com/alo-labs/kay/compare/v0.9.30...v0.9.31

## @alo-labs/kay v0.9.27

This release closes the open triage blocker sweep (#39–#53), hardens exec and
runtime guardrails, and ships categorized GitHub release notes for the 0.9.27
tag with the MiniMax-M3 live provider gate exercised during `./pre-release.sh`.

### Exec, Sandbox, And Runtime Reliability

- Provider env keys such as `OPENCODE_GO_API_KEY` now override stored
  `~/.kay/auth.json` credentials (#43).
- OpenCode Go wire requests canonicalize aliases such as `MiniMax-M3` →
  `minimax-m3` (#45).
- `kay exec --full-auto` enables workspace-write network access by default for
  package registries, GitHub APIs, and local test servers (#40, #44, #47).
- Malformed model shell commands (`git && -C && …`, `bash && -lc && …`) and
  `apply_patch` bodies are normalized before execution (#39, #46, #50).
- Express route handlers registered after `module.exports` in `routes/*.js`
  modules are rejected (#48).
- In-workspace `python3 -c` pathlib writes are allowed while writes outside the
  session cwd remain blocked (#51).
- Leading `KEY=value` argv prefixes are merged into the child environment before
  exec preflight (#52).
- Redundant leading `cd <workspace>` segments are auto-stripped instead of
  blocking (#53).
- Sidekick-style final `STATUS:` contracts are enforced and
  `STATUS: BLOCKED` is written on `--max-seconds` timeout (#42, #49).
- Heredoc workspace guards preserve `Path()` case instead of lowercasing it.

### Release And Verification

- Published package metadata for `@alo-labs/kay` 0.9.27 and platform optional
  dependencies.
- Verified locally with `./build-fast.sh`.
- Ran `./pre-release.sh` with the MiniMax-M3 live provider gate enabled.
- GitHub Releases carry detailed categorized notes directly in the release body;
  `CHANGELOG.md` supplements but does not replace the release body.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay
```

Compare: https://github.com/alo-labs/kay/compare/v0.9.26...v0.9.27

## @alo-labs/kay v0.9.25

This maintenance release refreshes Kay's Silver Bullet project governance
surface and completes the migration from the retired `docs/lessons`
terminology to the current `docs/learnings` convention. There are no Rust
runtime behavior changes in this release.

### Silver Bullet Governance

- Refreshed `silver-bullet.md` from the installed Silver Bullet 0.38.2 template
  while preserving the Kay project identity, active workflow, build gate, issue
  tracker mode, and multi-agent identity tags.
- Updated `.silver-bullet.json` to the current enforcement template and kept
  Kay-specific validation through `./build-fast.sh`.
- Registered the current Silver Bullet hook surface through the host runtime
  settings so doc-scheme enforcement remains active for future sessions.

### Documentation Migration

- Moved the monthly learnings file from `docs/lessons/2026-05.md` to
  `docs/learnings/2026-05.md`.
- Added current learnings frontmatter and retitled the document to the Learnings
  terminology.
- Updated live documentation, planning conventions, and historical session
  references that pointed to `lessons` so governed docs now consistently use
  `learnings`.

### Documentation Governance

- Updated `docs/doc-scheme.md` and `docs/doc-scheme.json` so the governed
  document inventory points at `docs/learnings/2026-05.md`.
- Refreshed `docs/task-doc-checklist.json` for the `silver-init-migrate` task
  with complete governed-doc coverage.
- Added slash-command implementation guidance that Silver Bullet init/migrate
  runs should preserve exact skill receipts and keep `docs/learnings`
  terminology aligned with governed doc-scheme keys.

### Release And Verification

- Published package metadata for `@alo-labs/kay` 0.9.25 and platform optional
  dependencies.
- Verified locally with `./build-fast.sh`.
- Ran `./pre-release.sh` with the MiniMax M3 live provider gate enabled.
- Completed the Release workflow successfully across preflight, platform
  artifact builds, GitHub Release creation, Homebrew formula publishing, and
  Google Chat announcement.
- Upgraded the visible local `kay` installation to 0.9.25 after the Release
  workflow completed.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay
```

Compare: https://github.com/alo-labs/kay/compare/v0.9.24...v0.9.25
