# Upstream Merge Strategy

Kay is not a direct fork of OSS Codex. Kay's practical upstream is Every Code
(`upstream`, `https://github.com/just-every/code.git`), which itself carries
merge-backs from OSS Codex (`oss`, `https://github.com/openai/codex.git`).

This strategy keeps that lineage explicit:

1. Merge Every Code first.
2. Preserve Kay-only architecture while resolving conflicts.
3. Run a separate OSS Codex review afterward, even when Every Code appears to
   have already merged OSS Codex.

The OSS pass is not optional. It is the check that prevents Kay from depending
blindly on Every Code's merge cadence, conflict choices, or feature priorities.

## Current Upstream Layout

- `main` is Kay's release branch.
- `upstream/main` is Every Code.
- `oss/main` is OSS Codex.
- `kay-rs/` is Kay's active Rust workspace.
- `codex-rs/` is currently a read-only OSS Codex mirror used for local
  comparison. Do not implement Kay changes there. Direct OSS review is only as
  current as this mirror, so refresh or verify it before relying on local
  crate-diff tooling.

As of the May 2026 upstream work:

- Kay merged Every Code through v0.6.98 in
  `d7c5b4166` (`Merge Every Code v0.6.98: preserve Kay providers and packaging`).
- That merge was intentionally conflict-resolved around Kay's provider stack,
  release packaging, branding, `~/.kay` isolation, and active workspace layout.
- After the later `kay-rs/` path migration, upstream comparison tooling compares
  `codex-rs/` against `kay-rs/`.

After refreshing remotes on 2026-05-24 Australia/Sydney:

- `upstream/main` was at `e9c6280194`
  (`docs(changelog): update for v0.6.99 [skip ci]`).
- `oss/main` was at `7d47056ea4`
  (`fix: plugin bundle archive handling for upload and install (#23983)`).
- `upstream/main` contained the fetched `oss/main` history at that moment
  (`git rev-list --left-right --count upstream/main...oss/main` returned
  `4462 0`).

That last point is useful context, not a rule. It only proves ancestry for the
fetched refs; it does not prove Kay has adopted the same behavior after conflict
resolution. Every upstream cycle still needs an explicit OSS review pass.

## Why Every Code Comes First

Every Code is the closest branch to Kay's current architecture. It has already
made many of the same high-level product bets Kay builds on:

- multi-provider support
- model/provider UX beyond the single upstream OpenAI path
- UX and app-server changes that diverge from the OSS baseline
- ongoing OSS Codex merge-backs

Merging Every Code first usually lowers conflict cost because Kay shares more
surface area with Every Code than with raw OSS Codex. It also lets Kay inherit
Every Code's OSS reconciliation work where that work is compatible.

The May 2026 Every Code v0.6.98 merge validated this order. The successful path
was not a blind merge; it was a preservation merge:

- adopt upstream fixes and structural changes where compatible
- keep Kay provider registration and model-routing behavior
- keep Kay release and packaging scripts
- keep Kay branding and installation boundaries
- keep `codex-rs/` as read-only upstream mirror
- validate with build, focused checks, live provider gate, release workflow, and
  local install verification

## Why OSS Codex Still Gets A Separate Pass

Every Code may contain OSS Codex, but Kay should not treat that as sufficient.
The direct OSS pass is required because:

- Every Code can lag OSS Codex between its own merge-backs.
- Every Code may intentionally skip or alter an OSS change.
- Every Code's conflict resolutions may not match Kay's architecture.
- Security, sandboxing, data-loss, protocol, auth, and release-critical fixes
  should be reviewed from the original OSS source.
- Kay needs a clear audit trail for "covered by Every Code", "ported directly",
  and "deferred with rationale".

The direct OSS pass is not necessarily a full merge from OSS Codex into Kay.
The default posture is triage and selective porting unless we intentionally
start a broader OSS merge branch.

## Priority Order

Use this order for each upstream cycle:

1. Prepare Kay.
2. Merge Every Code to its latest acceptable commit.
3. Stabilize Kay.
4. Review OSS Codex directly.
5. Port or cherry-pick critical/high-priority OSS changes not safely inherited.
6. Release only after Kay gates and release automation pass.

Do not start the OSS pass from a dirty or half-stabilized Every Code merge.
The OSS pass should compare against a known-good Kay state so failures are
attributable.

## Phase 1: Prepare Kay

Start from a clean worktree:

```bash
git status --short --branch
git fetch origin --prune
git fetch upstream --prune
git fetch oss --prune
git status --short --branch
```

If `origin/main` has moved, follow the repository merge-only policy. Do not
rebase Kay work.

Record the current refs:

```bash
git show -s --format='%h %ci %s' HEAD upstream/main oss/main
git rev-list --left-right --count HEAD...upstream/main
git rev-list --left-right --count HEAD...oss/main
git rev-list --left-right --count upstream/main...oss/main
```

Interpretation:

- `HEAD...upstream/main` shows Kay-only commits and pending Every Code commits.
- `HEAD...oss/main` shows Kay-only commits and pending OSS Codex commits.
- `upstream/main...oss/main` shows whether Every Code currently contains the
  fetched OSS Codex mainline.

Also identify the already-integrated Every Code parent before merging:

```bash
git merge-base HEAD upstream/main
git log --oneline --decorate "$(git merge-base HEAD upstream/main)"..upstream/main
```

Use that range for review. Git still merges a ref, not a range.

Create a branch for the work unless the task explicitly says to work directly
on `main`:

```bash
git switch -c codex/every-code-sync-YYYYMMDD
```

## Phase 2: Merge Every Code

Merge Every Code as the primary upstream:

```bash
git merge --no-ff --no-commit upstream/main
```

Resolve conflicts with these defaults:

- Keep Kay identity, branding, install paths, release scripts, and package
  publishing rules.
- Keep `kay-rs/` as the active Rust workspace.
- Keep `codex-rs/` read-only unless the task is specifically refreshing the
  mirror.
- Preserve Kay's provider architecture and OpenCode Go release matrix.
- Preserve `~/.kay` state isolation.
- Adopt Every Code changes that are upstream bug fixes, compatibility fixes,
  protocol updates, test improvements, or architecture that Kay has not
  intentionally forked away from.
- Do not accept changes that silently revert Kay-only behavior. If a conflict
  touches Kay-owned behavior, resolve it manually and document the decision.

High-risk areas require manual review even without textual conflicts:

- `kay-rs/core/src/default_client.rs`
- `kay-rs/core/src/openai_tools.rs`
- `kay-rs/core/src/agent_tool.rs`
- `kay-rs/core/src/chat_completions.rs`
- `kay-rs/core/src/client.rs`
- `kay-rs/core/src/codex.rs`
- `kay-rs/core/src/config*.rs`
- `kay-rs/protocol/**`
- `kay-rs/app-server-protocol/**`
- `kay-rs/tui/**`
- `kay-rs/exec/**`
- `kay-rs/apply-patch/**`
- release, install, Homebrew, and npm packaging scripts
- GitHub Actions release/build workflows

After conflict resolution:

```bash
git diff --check
bash scripts/check-kay-path-deps.sh
bash scripts/upstream-merge/diff-crates.sh --all
bash scripts/upstream-merge/verify.sh
./build-fast.sh
```

For provider/model changes, also run the relevant live provider gate. Current
release policy excludes direct MiniMax.io provider tests and validates MiniMax
M2.7 through OpenCode Go.

Commit with a merge subject that says what was merged and what Kay preserved,
for example:

```text
Merge Every Code v0.6.99: preserve Kay providers and packaging
```

## Phase 3: Stabilize Kay Before OSS Review

Do not begin OSS Codex porting while the Every Code merge is still unstable.
First make Kay pass its local gates:

```bash
./build-fast.sh
bash scripts/upstream-merge/verify.sh
```

Before pushing to `main`, run:

```bash
./pre-release.sh
```

If live-provider credentials are unavailable, do not treat the run as ready for
`main`. The live gate is part of Kay's release confidence for upstream work.

## Phase 4: Direct OSS Codex Review

After the Every Code merge is stable, review OSS Codex directly regardless of
Every Code's merge-back status. This pass has two separate jobs:

1. Find OSS commits not contained by Every Code.
2. Audit critical OSS areas against Kay's final resolved behavior, even when
   Every Code already contains the OSS commits.

Refresh refs again:

```bash
git fetch upstream --prune
git fetch oss --prune
git show -s --format='%h %ci %s' upstream/main oss/main
git rev-list --left-right --count upstream/main...oss/main
```

First classify missing OSS commits from the original source:

```bash
git log --oneline --decorate upstream/main..oss/main
git diff --stat upstream/main..oss/main
git diff --name-only upstream/main..oss/main
```

If `upstream/main` already contains `oss/main`, these commands may be empty.
That means there are no OSS commits missing from Every Code at the fetched refs;
it does not mean Kay's conflict-resolved tree matches the intended OSS behavior.

Then audit Kay's resolved tree against the OSS baseline. The local crate-diff
tools compare the tracked `codex-rs/` directory against `kay-rs/`, not remote
refs. `codex-rs/` is not a nested Git checkout, so before relying on those
tools, record the tracked-tree provenance and decide whether it is current
enough for the OSS commit being audited:

```bash
git log -1 --format='%h %ci %s' -- codex-rs
git rev-parse oss/main
```

If the tracked mirror is stale, refresh `codex-rs/` in a dedicated mirror-update
step or use a temporary checkout of `oss/main` for direct `git diff --no-index`
comparisons. Do not edit Kay product code under `codex-rs/`.

Generate fresh crate diffs before highlighting critical changes:

```bash
bash scripts/upstream-merge/diff-crates.sh --all
bash scripts/upstream-merge/highlight-critical-changes.sh --all
```

Do not run `highlight-critical-changes.sh --all` against old `.github/auto`
diff artifacts. It reads the existing diff files; `diff-crates.sh --all`
regenerates them.

For each relevant OSS change, record one of these outcomes:

- **covered-by-every-code**: the change is already present after the Every Code
  merge and behaves correctly in Kay.
- **port-directly**: the OSS change is critical/high-priority and either absent
  from Every Code or altered in a way Kay should not inherit.
- **defer**: the change is lower priority, incompatible with Kay's architecture,
  or needs a larger design pass.
- **reject**: the change conflicts with an intentional Kay decision and should
  not be brought forward.

Critical/high-priority categories include:

- security fixes
- sandbox escape or permission-boundary fixes
- auth/login fixes
- data-loss fixes
- crash, panic, or stuck-session fixes
- command execution safety fixes
- protocol compatibility fixes
- release, packaging, installer, or upgrade fixes
- high-value model/provider fixes that apply to Kay's supported matrix
- tests that catch a class of bugs Kay is likely to reintroduce

Direct OSS ports should be small and attributable. Prefer one commit per
coherent OSS fix or closely related set of fixes. In the commit body, include:

- source OSS commit or PR
- whether Every Code already had it
- Kay-specific conflict notes
- tests run

## Phase 5: Verify The Combined Result

After Every Code and OSS work are both complete:

```bash
git diff --check
bash scripts/check-kay-path-deps.sh
bash scripts/upstream-merge/diff-crates.sh --all
bash scripts/upstream-merge/verify.sh
./build-fast.sh
./pre-release.sh
```

If the merge touched TypeScript SDK paths or generated protocol bindings, run:

```bash
pnpm --dir sdk/typescript test
```

If the merge touched release workflows, installers, package metadata, or binary
layout, watch the GitHub Release workflow after pushing:

```bash
scripts/wait-for-gh-run.sh --workflow Release --branch main --failure-logs
```

## Logging And Audit Trail

Use the upstream merge log helper for non-trivial cycles:

```bash
scripts/upstream-merge/log-merge.sh init upstream/main
scripts/upstream-merge/log-merge.sh decision core preserve "Keep Kay provider routing"
scripts/upstream-merge/log-merge.sh decision protocol adopt "Adopt upstream protocol compatibility fix"
scripts/upstream-merge/log-merge.sh finalize
```

The log should capture:

- fetched Every Code and OSS refs
- `codex-rs/` tracked-tree provenance or temporary OSS checkout commit used for
  local OSS comparisons
- merge target
- conflicts and resolutions
- Kay-only behavior explicitly preserved
- OSS changes classified as covered, ported, deferred, or rejected
- verification commands and outcomes

## What Not To Do

- Do not rebase Kay on Every Code or OSS Codex.
- Do not wholesale replace `kay-rs/` with `codex-rs/`.
- Do not edit `codex-rs/` as if it were active Kay product code.
- Do not keep a physical compatibility path such as `code-rs -> kay-rs`.
- Do not allow upstream branding, home-directory behavior, package names, or
  release metadata to overwrite Kay identity accidentally.
- Do not skip the OSS review just because `upstream/main` currently contains
  `oss/main`.
- Do not cut a release merely because an upstream merge landed; release remains
  an explicit decision unless the normal `main` release automation is intended.

## Recommended Next Step

The next upstream cycle should:

1. Review Every Code changes from the last integrated commit (`861c9bab69`,
   v0.6.98 merge parent) to the latest `upstream/main`, then merge
   `upstream/main`.
2. Stabilize Kay and run the required gates.
3. Independently review OSS Codex from `oss/main`, regardless of Every Code's
   merge-back state, and verify the `codex-rs/` mirror commit before using
   local diff tooling.
4. Port any critical/high-priority OSS fixes or features that are missing,
   altered incompatibly, or worth validating directly in Kay.
5. Record covered OSS changes explicitly so future cycles know whether Kay got
   them through Every Code or through a direct OSS port.

This keeps Every Code as Kay's primary upstream while still treating OSS Codex
as the authoritative source for critical fixes.
