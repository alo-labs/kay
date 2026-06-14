# CONTEXT — Exec runtime hardening patch (v0.9.31)

**Scope:** Ship post-v0.9.30 Kay exec/runtime fixes, issue-monitor improvements, and close triage follow-ups.

## Locked decisions

- **STATUS contract:** Sidekick compact prompts (`Final message STATUS: SUCCESS with FILES_CHANGED`) must trigger `prompt_requires_final_status` — fixed in `ef2ae84e5c`.
- **Issue monitor:** Poll open issues plus `closed_activity` since `last_check_at`; do not treat post-close Sidekick telemetry on #42/#46 as reopen signals.
- **Live pre-release:** MiniMax.io / `MiniMax-M3` gate (`KAY_PRE_RELEASE_LIVE_PROVIDER_GATE=minimax-m3`) before patch release.
- **Kay-only fixes:** Edit `kay-rs/`; Sidekick registry issues out of scope (close as not planned).

## Assumptions

| Assumption | Owner | Status |
|------------|-------|--------|
| v0.9.30 already ships e1786691 triage blockers (#39–#53) | Release CI | Accepted |
| #54/#55 fixes need v0.9.31+ for users on older builds | Planner | Accepted |
| `silver-bullet` CLI at Cursor plugin cache path records hook receipts | Agent | Accepted |

## Constraints

- Merge-only push to `main`; no rebase.
- `./build-fast.sh` required for code changes; docs-only exempt.
- Never run `rustfmt`.

## Planning handoff

**In scope:** Push commits `ef2ae84`, `5b98a8b`; pre-release; patch release; verify Google Chat announcement.

**Out of scope:** Sidekick provider normalization; reopening closed #42/#46 for telemetry-only comments.

**Acceptance:** Release workflow green; `kay` npm tag incremented; open issues remain 0 unless new regressions filed.

**Blockers:** None.
