---
phase: 13-exec-status-issue-monitor-patch
plan: 1
type: ship
wave: 1
depends_on: []
base_release: v0.9.30
target_release: v0.9.31
files_modified:
  - kay-rs/exec/src/lib.rs
  - scripts/issue-monitor-check.sh
autonomous: true
---

# SILVER BULLET > PLAN

## Goal

Ship v0.9.31 patch release with exec STATUS prompt detection fix and issue
monitor post-close activity tracking.

## Assumptions (from CONTEXT)

- Commits `ef2ae84e5c` and `5b98a8b3e9` are on local `main`, not yet pushed.
- Latest published release is v0.9.30.
- Pre-release MiniMax-M3 live gate is required before push.

## TDD Policy

- Implementation already landed in prior commits; no new application code edits
  planned in this phase.
- Verification: run existing exec unit tests covering STATUS contract detection;
  smoke `scripts/issue-monitor-check.sh` output shape.
- Docs-only changes not expected; skip application TDD for new code.

## Wave 1 — Verify local state

**Goal:** Confirm commits and targeted tests pass before release gate.

| Task | Files / commands | Evidence |
| --- | --- | --- |
| Confirm `git log -2` shows STATUS fix + monitor commits | `git log --oneline -3` | Two commits atop `chore(release): 0.9.30` |
| Run exec STATUS contract tests | `cargo test -p code-exec prompt_requires_final_status` (or full exec crate tests) | All pass, zero warnings |
| Smoke issue monitor script | `bash scripts/issue-monitor-check.sh \| jq .` | JSON with `open_pending`, `closed_activity`, counts |

**Risks:** None. **Rollback:** N/A.

## Wave 2 — Pre-release gate

**Goal:** Pass `./pre-release.sh` including MiniMax-M3 live provider gate.

| Task | Files / commands | Evidence |
| --- | --- | --- |
| Run full pre-release | `./pre-release.sh` from repo root | Exit 0; log shows live provider gate passed |
| On failure | Inspect `$TMPDIR/kay-pre-release.*` logs | Root cause documented before retry |

**Risks:** Live provider flake. **Rollback:** Fix or retry; do not push on failure.

## Wave 3 — Push and release

**Goal:** Publish v0.9.31 and confirm CI release workflow success.

| Task | Files / commands | Evidence |
| --- | --- | --- |
| Push to origin | `git push origin main` | Remote `main` includes both commits |
| Monitor Release workflow | `scripts/wait-for-gh-run.sh --workflow Release --branch main` | Exit 0 |
| Confirm tag | `gh release list --limit 1` | v0.9.31 listed as Latest |
| Confirm Google Chat job | Release workflow job log | `Announce release in Google Chat` succeeded |

**Risks:** Release workflow failure. **Rollback:** Revert or hotfix per incident process.

## Verification Plan (`silver:verify`)

1. `gh release view v0.9.31` shows both fix subjects in release notes or commit range.
2. `cargo test -p code-exec` passes on release tag checkout (optional spot-check).
3. Issue monitor script produces valid JSON on production repo (`alo-labs/kay`).

## Exit Gate

- [ ] v0.9.31 published on GitHub
- [ ] Release workflow green including Google Chat announcement
- [ ] Local `main` synced with `origin/main`
- [ ] No uncommitted code changes (`.kay/` untracked is OK)

## Blockers

| Blocker | Status |
| --- | --- |
| `silver:context` + `silver:plan` hook receipts | Resolved — invoked 2026-06-14 via `silver-bullet invoke-skill` |
| Push + release execution | **Pending** — awaiting Wave 2–3 |
