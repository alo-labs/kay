# DevOps Cycle Workflow

This workflow covers infrastructure, release, and operations work.

## Start

1. Read `.planning/STATE.md` and the current roadmap
2. Identify the infrastructure scope and blast radius
3. Switch from feature mode to DevOps mode when the task is ops-heavy

## Core Loop

1. `discuss` - capture the operational decision points
2. `blast radius` - understand what the change can affect
3. `plan` - break the change into safe rollout steps
4. `execute` - implement the change in small, reversible pieces
5. `verify` - confirm the infra change works and is safe to promote

## Release Discipline

- Validate the change at the lowest safe environment first
- Preserve rollback paths and audit trails
- Use the same non-destructive init rules as the feature workflow
- Validate the release-notes header contract before push or tag; `scripts/check-release-notes-version.sh` fails the release if the header does not match `## @alo-labs/kay v<version>`, and hook remediation must refresh the checklist in the same session
- On GitHub Actions release preflights, keep linker-heavy Rust tests off the root filesystem by freeing stale cargo/sccache metadata before workspace nextest and setting `TMPDIR` to the data disk when one is mounted
- For release/install audits, report only reproducible findings with exact refs; if interrupted before a full review loop completes, label the result as a low pass and list the completed checks.

## Finish

- Update docs and project state
- Return to the feature workflow when the ops task is complete
- Keep the release notes and verification artifacts current
- If the change touches governed docs, refresh `docs/task-doc-checklist.json` in the same session before handing off
- Treat provider-registration and config-compatibility changes as release-sensitive and document the migration path before promotion
