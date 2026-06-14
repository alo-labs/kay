# PLAN — Exec runtime hardening patch (v0.9.31)

## Dependencies

- Local commits on `main`: `ef2ae84e5c` (STATUS prompts), `5b98a8b3e9` (issue monitor).
- MiniMax live API key in `~/.kay/auth.json` or `MINIMAX_LIVE_API_KEY`.
- `silver:context` and `silver:plan` invoked (hook receipts).

## Wave 1 — Verify and push

| Task | Files | Verification |
|------|-------|--------------|
| Confirm `cargo test -p code-exec final_status_contract` green | `kay-rs/exec/src/lib.rs` | test output |
| Run `./build-fast.sh` | workspace | exit 0 |
| `git push origin main` | — | remote updated |

## Wave 2 — Pre-release

| Task | Verification |
|------|--------------|
| `KAY_PRE_RELEASE_LIVE_PROVIDER_GATE=minimax-m3 ./pre-release.sh` | all four jobs pass |

## Wave 3 — Release

| Task | Verification |
|------|--------------|
| Monitor `scripts/wait-for-gh-run.sh --workflow Release --branch main` | success |
| Confirm npm tag + GitHub release + Google Chat job | gh release view |

## TDD policy

- STATUS detection: unit test added in wave 0 (done).
- Monitor script: shell check via `issue-monitor-check.sh` dry run.

## Assumptions / open questions

- None blocking release.
