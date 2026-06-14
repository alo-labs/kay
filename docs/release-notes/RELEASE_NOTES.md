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
