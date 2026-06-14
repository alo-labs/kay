# Kay session prompt: Sidekick live test matrix

Copy everything under **SESSION PROMPT (start)** into a Kay host session (Cursor/Codex/Claude with Sidekick + Kay active). The session drives the reusable release matrix in the Sidekick repo and reports triage against open Kay parent issues.

---

## SESSION PROMPT (start)

### OBJECTIVE

Run Sidekick’s **Kay release live-test matrix** (4 model profiles × 5 tasks = 20 cells) against the **local Kay binary** under test, capture logs under `tests/.kay-live-logs/`, summarize pass/fail in the matrix TSVs, and classify failures to **parent Kay issues** (comment via `gh`, do not file duplicate matrix tickets when a bucket matches).

Deliver a final host message with matrix score, delta vs baseline, per-parent issue signal, and recommended next fix focus.

### CONTEXT

| Artifact | Repo | Path / note |
|----------|------|-------------|
| Sidekick (harness) | [alo-exp/sidekick](https://github.com/alo-exp/sidekick) | Clone or use existing checkout; `cd` to repo root |
| Matrix runner | sidekick | `tests/run_kay_release_matrix.bash` |
| Matrix docs | sidekick | `tests/kay-live-matrix.md` |
| Task prompts | sidekick | `tests/kay-live-prompts/` (`task7-retry2-closeout.txt`, `task8-bulk-archive.txt`, `task9-sort-ui.txt`, `task10-full-regression.txt`) |
| E2E driver | sidekick | `tests/run_live_codex_e2e.bash` |
| Per-task driver | sidekick | `tests/run_kay_live_task.bash` |
| Export/import seed | sidekick | `tests/test-notes-app-seeds/export-import` |
| Log output dir | sidekick | `tests/.kay-live-logs/` |
| Pre-release gate pointer | sidekick | `site/pre-release-quality-gate.md`, `site/TESTING.md` |
| Kay live QA (SB harness) | sidekick | `docs/workflows/kay-live-qa.md` |
| Kay product repo | [alo-labs/kay](https://github.com/alo-labs/kay) | Parent issues filed/commented via matrix `gh` hooks |

**Checkout:** If Sidekick is not present, clone `https://github.com/alo-exp/sidekick.git` and work from that root. This prompt does not require a specific branch name; use the branch that contains the matrix scripts you intend to run.

### CURRENT STATUS (headline — 2026-06-15)

Use this as the **baseline** when interpreting your run; update the headline in this doc after a confirmed retest.

| Signal | Value |
|--------|--------|
| Kay under test | **local build** (`kay 0.0.0` from `cargo build -p code-cli --release`) at `~/.local/bin/kay` |
| Matrix prefix (latest local retest) | `local-fix-r5` → **15/20 PASS** (prior: r4 **13/20**, r3 **11/20**, v0.9.31 **4/20**) |
| Baseline comparison | **Δ +11** vs v0.9.31; **Δ +2** vs r4 |
| **#42** (STATUS contract) | **Strong overall:** ocg-mimo-pro **5/5**; minimax-m3 direct **e2e/task7/task10 PASS**; remaining FAILs emit `STATUS: BLOCKED` (contract present) but miss SUCCESS — task8/task9 incomplete work |
| **#46**, **#49** | **#49 cleared**; **#46** narrowed to task8 bulk-archive incomplete patches / multi-arg apply_patch (minimax-m3 e2e now PASS) |
| **#39**, **#52**, **#54** | **#39** mitigated (`cat -An` → `cat -n` repair); **#52** PORT default guidance added to MiMo/MiniMax profiles |
| Parent issues (active) | [#42](https://github.com/alo-labs/kay/issues/42), [#46](https://github.com/alo-labs/kay/issues/46), [#49](https://github.com/alo-labs/kay/issues/49), [#39](https://github.com/alo-labs/kay/issues/39), [#52](https://github.com/alo-labs/kay/issues/52), [#54](https://github.com/alo-labs/kay/issues/54), [#56](https://github.com/alo-labs/kay/issues/56) (Vision Delegate enhancement) |
| Deduped closures | [#57](https://github.com/alo-labs/kay/issues/57)–[#71](https://github.com/alo-labs/kay/issues/71), [#72](https://github.com/alo-labs/kay/issues/72), [#73](https://github.com/alo-labs/kay/issues/73) closed as duplicates/mis-triage |

**Triage order** (first match wins per cell — see `tests/kay-live-matrix.md`): #52 → #46 → #49 → #42 → #55/#39 → #54 → others.

### Model profiles (default matrix)

| profile_id | provider | model |
|------------|----------|-------|
| `ocg-minimax-m3` | `opencode-go` | `minimax-m3` |
| `ocg-mimo-pro` | `opencode-go` | `mimo-v2.5-pro` |
| `ocg-mimo` | `opencode-go` | `mimo-v2.5` |
| `minimax-m3` | `minimax` | `minimax/MiniMax-M3` |

### Tasks (default)

| Task | Driver | Prompt / notes |
|------|--------|----------------|
| `e2e` | `tests/run_live_codex_e2e.bash` | Health-fix smoke on canonical `test-notes-app` |
| `task7` | `tests/run_kay_live_task.bash` | `kay-live-prompts/task7-retry2-closeout.txt`, export-import seed |
| `task8` | `tests/run_kay_live_task.bash` | `task8-bulk-archive.txt` |
| `task9` | `tests/run_kay_live_task.bash` | `task9-sort-ui.txt` |
| `task10` | `tests/run_kay_live_task.bash` | `task10-full-regression.txt`, export-import seed |

Per-job env set by matrix: `KAY_LIVE_MODEL_PROVIDER`, `KAY_LIVE_MODEL`.

### Credentials (no secrets in chat)

| Requirement | Location / action |
|-------------|-------------------|
| OpenCode Go | `OPENCODE_GO_API_KEY` in Sidekick repo **`.env.local`** (gitignored) |
| MiniMax (direct profile) | `MINIMAX_API_KEY` in `.env.local` when running `minimax-m3` profile |
| GitHub | `gh auth status` must succeed for issue comment hooks |
| Toolchain | `kay`, `node`, `npm` on `PATH`; ensure `~/.local/bin` precedes other installs |

Do **not** paste API keys into Kay transcripts or this prompt file.

### DESIRED STATE (commands)

1. **Confirm Kay binary (local fix — do not overwrite)**

   ```bash
   export PATH="${HOME}/.local/bin:${PATH}"
   kay --version   # expect 0.9.33 for local-fix testing
   which kay       # expect ~/.local/bin/kay
   ```

2. **Sidekick repo**

   ```bash
   # Example: use existing checkout
   cd /path/to/sidekick   # e.g. ~/projects/sidekick/repo
   git status
   test -f tests/run_kay_release_matrix.bash
   test -f .env.local || echo "WARN: create .env.local with required keys"
   ```

3. **Full matrix (sequential — recommended for clean signal)**

   ```bash
   cd /path/to/sidekick
   export PATH="${HOME}/.local/bin:${PATH}"
   KAY_MATRIX_SKIP_INSTALL=1 \
   KAY_MATRIX_PREFIX=local-fix-r2 \
   KAY_MATRIX_PARALLEL=0 \
     bash tests/run_kay_release_matrix.bash
   ```

   - **`KAY_MATRIX_SKIP_INSTALL=1`** — mandatory when testing a **local** `~/.local/bin/kay` build; do **not** run official `install.sh` over the standalone package.
   - Omit `KAY_RELEASE` when the installed binary is the candidate under test.
   - Optional parallel rerun (noisy): `KAY_MATRIX_PARALLEL=1` only after a clean sequential pass, or to reproduce rc 143 / e2e flake.

4. **Re-report from existing logs (no new jobs)**

   ```bash
   KAY_MATRIX_PREFIX=local-fix-r1 KAY_MATRIX_REREPORT=1 \
     bash tests/run_kay_release_matrix.bash
   ```

5. **Inspect outputs**

   ```bash
   column -t -s $'\t' tests/.kay-live-logs/<prefix>-matrix-summary.tsv | less
   column -t -s $'\t' tests/.kay-live-logs/<prefix>-issue-report.tsv | less
   ls -lt tests/.kay-live-logs/<profile>-<prefix>-<task>-*.log | head
   ```

6. **Optional smoke before full matrix**

   ```bash
   cd /path/to/sidekick
   export PATH="${HOME}/.local/bin:${PATH}"
   KAY_MATRIX_SKIP_INSTALL=1 KAY_MATRIX_PREFIX=smoke \
   KAY_MATRIX_TASKS=e2e KAY_MATRIX_PROFILES=$'ocg-mimo:opencode-go:mimo-v2.5\n' \
     bash tests/run_kay_release_matrix.bash
   ```

### SUCCESS CRITERIA

**Harness (matrix):**

- All 20 jobs reach a resolved status (`PASS` / `FAIL` / `UNKNOWN`) in `tests/.kay-live-logs/<prefix>-matrix-summary.tsv`.
- `tests/.kay-live-logs/<prefix>-issue-report.tsv` lists `gh` actions (`comment#<parent>` or new issue) for each FAIL.
- Pass count and parent-issue breakdown are compared to the **CURRENT STATUS** headline (5/20 `local-fix-r1` vs 4/20 `v0.9.31`).
- Improvement targets: more #42 cells PASS (STATUS contract in `*-last-message.txt`); #46/#49 cleared on task8/task9; no spurious rc 143 on e2e when `KAY_MATRIX_PARALLEL=0`.

**Kay delegation closeout (per live task prompts):** delegated Kay runs should end with the standard block when the task prompt requires it:

```text
STATUS: SUCCESS
FILES_CHANGED: [...]
TESTS_RUN: ...
ASSUMPTIONS: [...]
```

Harness enforces this when `SIDEKICK_KAY_REQUIRE_STATUS=1` (default in `run_kay_live_task.bash`). Missing block → FAIL `status_contract` → parent **#42**.

**Host final message MUST include this block** (fill with your run’s values):

```text
STATUS: SUCCESS
MATRIX_PREFIX: <e.g. local-fix-r2>
KAY_VERSION: <kay --version>
SCORE: <pass>/20 PASS (baseline: 5/20 local-fix-r1, 4/20 v0.9.31)
DELTA: <+N|-N|0> vs local-fix-r1 headline
PARENT_SIGNAL: #42=<cells>; #46=<cells>; #49=<cells>; #39/#52/#54=<observed|none>
REGRESSION_NOTES: <e.g. ocg-minimax-m3 e2e under parallel>
ARTIFACTS: tests/.kay-live-logs/<prefix>-matrix-summary.tsv, <prefix>-issue-report.tsv
NEXT_FOCUS: <1-3 concrete Kay or harness fixes>
ASSUMPTIONS: [...]
```

If the matrix did not complete, use `STATUS: FAIL` and list blocking cells and log paths.

### CONSTRAINTS

- **Do not** overwrite `~/.local/bin/kay` with GitHub `install.sh` while validating local Kay **0.9.33** fixes; use `KAY_MATRIX_SKIP_INSTALL=1`.
- **Do not** commit `.env.local` or print API keys.
- Prefer **`KAY_MATRIX_PARALLEL=0`** for release-quality signal; document parallel-only failures separately.
- Matrix `gh` hooks target **alo-labs/kay**; one parent issue per failed cell when classification matches.
- Do not file new Kay issues for signatures already mapped to parents (#42, #46, #49, #39, #52, #54, etc.) unless classification is genuinely new.
- Sidekick Silver Bullet: for harness-only work, see `docs/workflows/kay-live-qa.md` (planning floor exemption); restore full SB floor before unrelated product ship.

### ASSUMPTIONS

- Sidekick checkout contains current `tests/run_kay_release_matrix.bash` and `tests/kay-live-matrix.md` (or equivalent on your branch).
- Local Kay **0.9.33** at `~/.local/bin/kay` is the intentional candidate; version may differ if you explicitly retarget the prompt.
- Network access for model APIs and `gh` is available.
- Canonical `test-notes-app` seed and verify scripts are intact under `tests/test-notes-app-seeds/`.
- Long-running matrix jobs may take hours sequentially; parallel mode trades speed for flake risk (especially e2e).

### Quick smoke one-liner

```bash
cd /path/to/sidekick && export PATH="${HOME}/.local/bin:${PATH}" && \
KAY_MATRIX_SKIP_INSTALL=1 KAY_MATRIX_PARALLEL=0 KAY_MATRIX_PREFIX=smoke \
KAY_MATRIX_TASKS=e2e KAY_MATRIX_PROFILES=$'ocg-mimo:opencode-go:mimo-v2.5\n' \
  bash tests/run_kay_release_matrix.bash
```

### Full matrix one-liner (v0.9.31-style suite, local kay, sequential)

```bash
cd /path/to/sidekick && export PATH="${HOME}/.local/bin:${PATH}" && \
KAY_MATRIX_SKIP_INSTALL=1 KAY_MATRIX_PARALLEL=0 KAY_MATRIX_PREFIX=local-fix-r2 \
  bash tests/run_kay_release_matrix.bash
```

---

## SESSION PROMPT (end)

## Maintaining this doc

After a confirmed matrix retest, update **CURRENT STATUS (headline)** with new `KAY_MATRIX_PREFIX`, pass count, parent-issue notes, and date. Keep in sync with Sidekick `tests/kay-live-matrix.md` when profiles, tasks, or parent-issue tables change.
