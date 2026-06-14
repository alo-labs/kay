---
phase: 13-exec-status-issue-monitor-patch
type: patch-release
last_updated: "2026-06-14T04:05:00Z"
base_release: v0.9.30
target_release: v0.9.31
---

# SILVER BULLET > CONTEXT

## Scope

Post-v0.9.30 patch delivering two fixes already committed locally on `main`:

1. **Exec STATUS prompt detection** (`ef2ae84e5c`) — extend
   `prompt_requires_final_status` in `kay-rs/exec/src/lib.rs` to match compact
   Sidekick final-message contracts (e.g. bare `STATUS: SUCCESS` / `STATUS:
   BLOCKED`) when the prompt omits explicit "include status" wording. Closes
   gap behind #42 / #49 telemetry and supports #54 / #55 triage follow-ups.
2. **Issue monitor post-close activity** (`5b98a8b3e9`) — extend
   `scripts/issue-monitor-check.sh` to detect closed issues with activity after
   the last check and emit structured `open_pending` / `closed_activity` JSON for
   the Kay issue monitor automation.

## Out of Scope

- New feature work beyond these two fixes.
- Milestone v0.9.15 workspace-rename phase execution (deferred).
- Committing `.kay/issue-monitor-state.json` or `.kay/issue-monitor.log` (local
  runtime state; remain untracked).

## Decisions

- Patch release ships both commits together as v0.9.31.
- Pre-release gate runs with MiniMax-M3 live provider check per user request.
- Merge-and-push policy: merge-only, no rebase; push to `origin/main` after
  pre-release passes.

## Assumptions

| Assumption | Owner | Status |
| --- | --- | --- |
| v0.9.30 is the latest published GitHub release | Agent | Accepted |
| Local `main` is 2 commits ahead of `origin/main` with clean working tree except untracked `.kay/` | Agent | Accepted |
| `./pre-release.sh` is the required push gate (includes MiniMax live provider gate unless skipped) | Project policy | Accepted |
| Patch-release worker agent committed but did not finish push/release | Agent | Accepted |
| #42 and #46 telemetry items are addressed in shipped v0.9.30 docs/changelog; this patch is STATUS-detection + monitor only | Agent | Accepted |

## Constraints

- `./build-fast.sh` required for code validation before push (project rule).
- `./pre-release.sh` required before push to `main` (project rule).
- All compiler warnings are failures.
- Do not run `rustfmt`.

## Risks

- Release workflow failure after push (rollback: revert tag commit or hotfix).
- MiniMax live provider gate flake (mitigation: retry; `KAY_PRE_RELEASE_SKIP_LIVE_PROVIDER_GATE=1` only if user approves).

## Dependencies

- GitHub CLI (`gh`) authenticated for push and release monitoring.
- `scripts/wait-for-gh-run.sh --workflow Release --branch main` for post-push watch.

## Unresolved Questions

None blocking planning or execution.

## Planning Handoff

**In scope for `silver:plan`:**

- Wave 1: Verify local commits and targeted exec tests.
- Wave 2: Run `./pre-release.sh` (MiniMax-M3 live gate).
- Wave 3: Push `main`, monitor Release workflow, confirm v0.9.31 tag and Google Chat announcement.

**Acceptance criteria references:**

- Exec compact STATUS prompts detected and enforced (unit tests in `kay-rs/exec`).
- Issue monitor script returns `closed_activity` array for post-close comment events.
- GitHub release v0.9.31 published successfully.

**Blockers:** None — push and release remain pending execution.
