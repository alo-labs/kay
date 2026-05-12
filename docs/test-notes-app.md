# Test Notes App

This is the real validation target for Kay's OCG-based release workflow.

## Locations

- Local checkout: `/Users/shafqat/projects/test-notes-app`
- GitHub repo: `alo-exp/test-notes-app`
- Kay home for the end-user simulation: `~/.kay`

## Purpose

The notes app is intentionally separate from Kay itself. It gives Kay a real
project to modify, review, and validate before releases without depending on
the developer's Codex or Every Code environment.

The project is small, but it is not a toy fixture:

- persistent SQLite-backed notes
- searchable list and tag filters
- note creation, editing, archiving, and deletion
- a lightweight browser UI that can be exercised by live model runs

## Bootstrap

Use the repository bootstrap script from the Kay checkout:

```bash
scripts/bootstrap-test-notes-app.sh
```

That script:

1. Ensures `~/.kay` exists.
2. Clones `alo-exp/test-notes-app` into `~/projects/test-notes-app` if needed.
3. Keeps the checkout pointed at the expected GitHub remote.
4. Installs the notes app dependencies unless `--skip-install` is used.

## Validation Workflow

The notes-app live harness runs Kay against the real checkout and copies the
session transcript JSONL into a repo-local transcripts directory for review.
That transcript is the provenance source for later analysis and for the
human-readable transcript viewer in Kay.

The harness is intended to be run before every release going forward, using the
supported OpenCode Go model matrix.

