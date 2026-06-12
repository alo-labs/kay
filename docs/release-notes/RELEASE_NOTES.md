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
