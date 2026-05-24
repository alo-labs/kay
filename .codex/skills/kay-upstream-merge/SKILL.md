---
name: kay-upstream-merge
description: Use when merging Kay's upstream changes. Lists latest unmerged changes from Every Code and OSS Codex, then merges/ports only when the user explicitly asks for merge work.
metadata:
  short-description: List Kay upstream changes; merge only on explicit request
---

# Kay Upstream Merge

Use this skill for Kay upstream syncs in `/Users/shafqat/projects/codex-cli/kay`.
It follows `/Users/shafqat/projects/codex-cli/kay/docs/upstream-merge-strategy.md`.
The goal is not merely to produce a clean Git merge. The goal is to leave Kay
working, with Kay-only behavior deliberately preserved and every upstream
adoption decision auditable.

Hard rules:

- Do not use Superpowers.
- Do not rebase.
- Treat `upstream/main` as Every Code and `oss/main` as OSS Codex.
- Merge Every Code before OSS Codex.
- List unmerged changes before merging or porting them.
- Stop before changing merge state if the listing reveals a policy blocker,
  dirty worktree ambiguity, missing remotes, or unavailable required gates.
- Keep `kay-rs/` as Kay's active Rust workspace.
- Treat `codex-rs/` as a read-only OSS mirror unless the task explicitly says to refresh the mirror.
- Preserve Kay identity, provider architecture, release packaging, `~/.kay` isolation, and OpenCode Go model policy.
- Do not run `./build-fast.sh` for documentation-only changes.
- Never run rustfmt.
- Do not push or release if any required gate fails, any warning remains, or
  live-provider credentials needed for the touched area are unavailable.

## Start

Before doing any merge work, read the strategy doc enough to confirm current
policy and commands:

```bash
sed -n '1,260p' /Users/shafqat/projects/codex-cli/kay/docs/upstream-merge-strategy.md
sed -n '261,620p' /Users/shafqat/projects/codex-cli/kay/docs/upstream-merge-strategy.md
```

Then inspect state:

```bash
cd /Users/shafqat/projects/codex-cli/kay
git status --short --branch
git remote -v
```

If the worktree is dirty, stop and resolve or ask how to handle it unless the
user explicitly included dirty-worktree instructions. Do not stash, discard, or
merge across unrelated local work without explicit approval.

If the invocation is implicit, ambiguous, or the user did not explicitly ask to
merge, treat it as list/status-only. Use existing local refs or side-effect-free
remote checks and do not run `git fetch`, `git branch`, `git switch`, or
`git merge`:

```bash
git ls-remote --heads origin main
git ls-remote --heads upstream main
git ls-remote --heads oss main
```

For list/status-only output, clearly label local-ref data as possibly stale
unless it was confirmed with `git ls-remote`. Stop after the listing/status
summary and do not create safety refs, branches, or merge state.

If the user asked to merge, fetch without rebasing before the listing:

```bash
git fetch origin --prune
git fetch upstream --prune
git fetch oss --prune
listed_head="$(git rev-parse --verify HEAD^{commit})"
listed_every_code="$(git rev-parse --verify upstream/main^{commit})"
listed_oss="$(git rev-parse --verify oss/main^{commit})"
```

Use `listed_every_code` and `listed_oss` for the pre-merge listing and for the
later Every Code merge. Do not merge a newer remote-tracking ref that was not
included in the listing shown to the user.

If `origin/main` moved, record that in the listing. Treat stale local `main`, or
a current non-`main` branch that does not include the updated local `main`, as
blockers. Do not merge `origin/main` or switch branches before the listing.

Record refs and merge bases:

```bash
git show -s --format='%h %ci %s' "$listed_head" "$listed_every_code" "$listed_oss"
git rev-list --left-right --count "$listed_head"..."$listed_every_code"
git rev-list --left-right --count "$listed_head"..."$listed_oss"
git rev-list --left-right --count "$listed_every_code"..."$listed_oss"
git rev-list --left-right --count main...origin/main
git merge-base "$listed_head" "$listed_every_code"
git merge-base "$listed_every_code" "$listed_oss"
git log --oneline --decorate "$(git merge-base "$listed_head" "$listed_every_code")".."$listed_every_code"
```

## List First

Always provide the user a concise pre-merge listing before changing merge state:

- listed `HEAD`, `upstream/main`, and `oss/main` commit SHAs
- count of Kay-only vs Every Code commits
- count of Kay-only vs OSS Codex commits
- count showing whether fetched Every Code contains fetched OSS Codex
- Every Code commits pending from the merge-base to `upstream/main`
- OSS commits not contained by Every Code from `upstream/main..oss/main`
- high-risk paths changed by the pending ranges
- whether `codex-rs/` is current enough for local OSS comparisons
- whether local `main` is behind `origin/main`, and if starting from another
  branch, whether that branch already includes the updated local `main`

Use commands like:

```bash
git diff --name-only "$listed_head"..."$listed_every_code"
git log --oneline --decorate "$listed_every_code".."$listed_oss"
git diff --name-only "$listed_every_code".."$listed_oss"
git log -1 --format='%h %ci %s' -- codex-rs
git rev-parse HEAD:codex-rs
git rev-parse "$listed_oss":codex-rs
git rev-parse "$listed_oss"
```

Treat `git log -1 -- codex-rs` as context only. It does not prove mirror
provenance. If `git rev-parse HEAD:codex-rs` differs from
`git rev-parse "$listed_oss":codex-rs`, or either tree lookup fails, report the
mirror mismatch in the listing and use an approved temporary `oss/main`
checkout for OSS comparisons.

Also extract high-risk touched paths for both pending ranges:

```bash
git diff --name-only "$listed_head"..."$listed_every_code" -- kay-rs codex-rs scripts .github package.json pnpm-lock.yaml codex-cli sdk
git diff --name-only "$listed_every_code".."$listed_oss" -- codex-rs kay-rs scripts .github package.json pnpm-lock.yaml codex-cli sdk
```

If these return anything, call it out in the pre-merge listing and state the
planned manual review.

If the user asked to merge, continue after the listing unless the listing
reveals a blocker.

Only after the listing is complete, the user asked to merge, and no blocker is
present, create a local safety ref before the first merge operation:

```bash
every_code_to_merge="$listed_every_code"
git show -s --format='%h %ci %s' "$every_code_to_merge"
git branch "backup/pre-upstream-sync-$(date +%Y%m%d-%H%M%S)"
```

Treat these as blockers until resolved:

- dirty worktree without explicit handling instructions
- missing `upstream` or `oss` remotes
- local `main` behind `origin/main`, or a non-`main` branch that does not
  include the updated local `main`
- `codex-rs/` too stale for the OSS audit and no approved temporary OSS checkout
- required credentials unavailable for a provider area touched by the merge
- high-risk upstream changes whose Kay preservation strategy is unclear

## Merge Every Code

Create a branch unless the user explicitly requested direct `main` work:

```bash
git switch -c "codex/every-code-sync-$(date +%Y%m%d)"
```

Merge Every Code:

```bash
git merge --no-ff --no-commit "$every_code_to_merge"
```

If the merge goes wrong before committing, prefer `git merge --abort`. If a
merge commit has already been created, do not rewrite history unless the user
explicitly approves it; make a forward fix or ask.

Resolve conflicts manually. Default to preserving Kay-only behavior in:

- provider registration and model routing
- provider configuration, auth, model aliases, and reasoning-level mapping
- release, package, Homebrew, npm, and install paths
- `kay-rs/` workspace paths
- `~/.kay` state isolation
- workflows and scripts that publish or upgrade Kay
- Kay branding and compatibility aliases
- `build-fast.sh`, `pre-release.sh`, and live-provider gates

Manually review these areas even without textual conflicts:

- `kay-rs/core/src/default_client.rs`
- `kay-rs/core/src/openai_tools.rs`
- `kay-rs/core/src/agent_tool.rs`
- `kay-rs/core/src/chat_completions.rs`
- `kay-rs/core/src/client.rs`
- `kay-rs/core/src/codex.rs`
- `kay-rs/core/src/config*.rs`
- `kay-rs/core/src/model*.rs`
- `kay-rs/core/src/provider*.rs`
- `kay-rs/core/src/auth*.rs`
- `kay-rs/protocol/**`
- `kay-rs/app-server-protocol/**`
- `kay-rs/tui/**`
- `kay-rs/exec/**`
- `kay-rs/apply-patch/**`
- `kay-rs/cli/**`
- `codex-cli/**`
- `sdk/typescript/**`
- release, install, Homebrew, npm, and GitHub workflow files

Before committing, produce a preservation checklist in the working notes or
merge log covering:

- Kay binary remains `kay`; compatibility aliases remain only where intended.
- Active Rust workspace is `kay-rs/`; no generated or tracked `code-rs` path is
  reintroduced.
- `codex-rs/` stayed read-only except for an explicit mirror update.
- Kay provider registry still includes OpenCode Go paths and Kay-supported
  models.
- Current OpenCode Go live-test model set remains: MiMo V2.5, MiMo V2.5 Pro,
  DeepSeek V4 Flash with max reasoning, and MiniMax M2.7.
- Direct MiniMax.io provider tests remain excluded unless the user explicitly
  changes that policy.
- `~/.kay` isolation is not replaced by upstream `~/.codex` behavior.
- Release workflow does not publish under upstream identities.
- User-facing docs and examples do not regress to stale `code-rs/` paths.

After resolving conflicts, run the strategy's required code gates for upstream
merge work:

```bash
git diff --check
bash scripts/check-kay-path-deps.sh
bash scripts/upstream-merge/diff-crates.sh --all
bash scripts/upstream-merge/verify.sh
./build-fast.sh
```

If the merge changes provider/model behavior, also run the relevant live
provider gate. Exclude direct MiniMax.io provider tests; validate MiniMax M2.7
through OpenCode Go.

```bash
bash scripts/pre-release-live-provider-gate.sh
```

This gate uses `OPENCODE_GO_LIVE_API_KEY`, falls back to
`OPENCODE_GO_API_KEY`, and can also read
`provider_credentials.opencode-go.api_key` from `~/.kay/auth.json`.

Commit with a merge subject that names Every Code and the preserved Kay areas,
for example:

```text
Merge Every Code v0.6.99: preserve Kay providers and packaging
```

## OSS Codex Pass

Do this only after the Every Code merge is stable. Before starting the OSS pass,
confirm:

```bash
git status --short
./build-fast.sh
bash scripts/upstream-merge/verify.sh
```

If either command fails, or the worktree contains unresolved or unstaged merge
work, stop. Do not begin OSS classification or porting from a half-stabilized
Every Code merge.

Record the exact Every Code commit that was merged into Kay before refreshing
refs. Use the earlier `every_code_to_merge` value if this run captured it before
the merge. For resumed runs, first inspect candidate merge commits or recover the
commit from the audit log; do not derive a parent from any merge commit until
you have verified that it is the Every Code sync:

```bash
git rev-list --first-parent --merges -n 10 --format='%H %ci %s' HEAD
```

Then explicitly set `every_code_merge_commit` to the verified Every Code merge
commit, or set `every_code_to_merge` directly from the audit log. Canonicalize
the pinned commit before continuing so resumed or multi-shell runs use one full
SHA everywhere. Use this pinned commit, not a later-moving `upstream/main`, when
deciding whether OSS commits were covered by the Every Code merge Kay actually
adopted:

```bash
test -n "${every_code_to_merge:-${every_code_merge_commit:-}}"
if test -z "${every_code_to_merge:-}"; then
  test "$(git rev-list --parents -n 1 "$every_code_merge_commit" | wc -w | tr -d ' ')" = 3
  git show -s --format='%h %ci %s' "$every_code_merge_commit"
  every_code_to_merge="$(git rev-parse "$every_code_merge_commit^2")"
fi
merged_every_code="$(git rev-parse --verify "${every_code_to_merge}^{commit}")"
every_code_to_merge="$merged_every_code"
if test -n "${every_code_merge_commit:-}"; then
  git show -s --format='%h %ci %s' "$every_code_merge_commit"
fi
git show -s --format='%h %ci %s' "$merged_every_code"
git merge-base --is-ancestor "$merged_every_code" HEAD
git merge-base --is-ancestor "$merged_every_code" upstream/main
```

If you cannot verify the Every Code merge commit, do not guess. Recover
`every_code_merge_commit` or `every_code_to_merge` from the audit log or stop.

If either `git merge-base --is-ancestor` guard fails, stop. Kay has not actually
adopted the Every Code commit being used for OSS coverage classification, or the
recovered commit is not from Every Code.

Refresh refs again:

```bash
git fetch upstream --prune
git fetch oss --prune
refreshed_upstream_main="$(git rev-parse --verify upstream/main^{commit})"
refreshed_oss="$(git rev-parse --verify oss/main^{commit})"
oss_under_review="$refreshed_oss"
git show -s --format='%h %ci %s' "$refreshed_upstream_main" "$refreshed_oss"
git rev-list --left-right --count "$refreshed_upstream_main"..."$refreshed_oss"
```

If `upstream/main` changed after the Every Code merge, record the drift and do
not classify OSS changes against the newer `upstream/main` unless the user
explicitly chooses to merge the newer Every Code first:

```bash
test "$refreshed_upstream_main" = "$merged_every_code" || {
  scripts/upstream-merge/log-merge.sh init "$merged_every_code"
  scripts/upstream-merge/log-merge.sh note refs "HEAD=$(git rev-parse HEAD) every_code_to_merge=$every_code_to_merge merged_every_code=$merged_every_code refreshed_upstream_main=$refreshed_upstream_main oss_under_review=$oss_under_review merge-base=$(git merge-base "$merged_every_code" "$oss_under_review")"
  scripts/upstream-merge/log-merge.sh note provenance "codex-rs-tree=$(git rev-parse HEAD:codex-rs 2>/dev/null || echo unavailable); oss-codex-rs-tree=$(git rev-parse "$oss_under_review":codex-rs 2>/dev/null || echo unavailable)"
  scripts/upstream-merge/log-merge.sh note deviation "upstream/main moved after the Every Code merge; stopped before OSS classification"
  scripts/upstream-merge/log-merge.sh finalize
  echo "upstream/main moved after the Every Code merge; classify against $merged_every_code or merge the newer Every Code first"
  exit 1
}
```

If `log-merge.sh init` would block on an interactive prompt in this drift path,
record the same refs and deviation manually in
`docs/maintenance/upstream-merge-logs/` before stopping.

List direct OSS candidates missing from Every Code:

```bash
git log --oneline --decorate "$merged_every_code".."$oss_under_review"
git diff --stat "$merged_every_code".."$oss_under_review"
git diff --name-only "$merged_every_code".."$oss_under_review"
git log -1 --format='%h %ci %s' -- codex-rs
git rev-parse HEAD:codex-rs
git rev-parse "$oss_under_review":codex-rs
git rev-parse "$oss_under_review"
```

An empty `"$merged_every_code".."$oss_under_review"` does not end the OSS pass. It only
means the fetched OSS commits are reachable from the Every Code commit Kay
actually merged. Still audit critical OSS areas against Kay's final resolved
tree because Kay conflict resolution can drop or alter behavior that Every Code
already carried.

Generate fresh crate diffs before highlighting:

```bash
bash scripts/upstream-merge/diff-crates.sh --all
bash scripts/upstream-merge/highlight-critical-changes.sh --all
```

Also inspect critical OSS-vs-Kay areas directly. Use `codex-rs/` only if its
tree exactly matches the OSS commit under review. A last-touch commit from
`git log -1 -- codex-rs` is not sufficient provenance. If the `HEAD:codex-rs`
tree differs from `"$oss_under_review":codex-rs`, use a temporary checkout of
the exact OSS commit under review and `git diff --no-index` for targeted paths.
Log the temporary checkout commit.

Default OSS behavior is direct review and selective porting, not a blind full
merge. For each relevant OSS change classify it as:

- `covered-by-every-code`
- `port-directly`
- `defer`
- `reject`

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

Port critical or high-priority OSS fixes/features that are missing, altered
incompatibly by Every Code, or worth validating directly in Kay. Prefer one
commit per coherent OSS fix or small related set. Include source OSS commit or
PR, Every Code coverage status, Kay conflict notes, and tests run in the commit
body.

If the user explicitly asks for a full OSS merge, create a separate branch
unless they explicitly requested direct `main` work, list first, and use only
merge workflow:

```bash
oss_to_merge="$(git rev-parse --verify "${oss_under_review:-$refreshed_oss}^{commit}")"
git show -s --format='%h %ci %s' "$oss_to_merge"
scripts/upstream-merge/log-merge.sh init "$merged_every_code"
scripts/upstream-merge/log-merge.sh note refs "oss_to_merge=$oss_to_merge"
git switch -c "codex/oss-sync-$(date +%Y%m%d)"
git merge --no-ff --no-commit "$oss_to_merge"
```

If `log-merge.sh init` would block on an interactive prompt in this full-OSS
path, record `oss_to_merge` manually in
`docs/maintenance/upstream-merge-logs/` before merging.

Apply the same Kay-preservation review list used for Every Code, plus an
explicit check for branding, home directory, package names, provider routing,
release metadata, and workspace path regressions. Do not rebase. Treat a full
OSS merge as higher risk than the default selective port path and require the
same preservation checklist plus the final verification suite before any push.

## Audit Log

For non-trivial cycles, create or update the upstream merge log with:

```bash
scripts/upstream-merge/log-merge.sh init "$merged_every_code"
scripts/upstream-merge/log-merge.sh note refs "HEAD=<head> every_code_to_merge=<sha> refreshed_upstream_main=<sha> oss/main=<sha> merge-base=<merge-base>"
scripts/upstream-merge/log-merge.sh note provenance "codex-rs-tree=<tree-or-mismatch>; oss-codex-rs-tree=<tree>; temp-oss-checkout=<commit-if-used>"
scripts/upstream-merge/log-merge.sh decision core preserve "Keep Kay provider routing"
scripts/upstream-merge/log-merge.sh decision oss-review port-directly "Port critical OSS fix <commit>"
scripts/upstream-merge/log-merge.sh note verify "Ran: git diff --check; check-kay-path-deps; diff-crates --all; verify.sh; build-fast.sh; pre-release.sh; sdk/typescript test=<ran-or-not-applicable>; live-provider-gate=<ran-or-skipped-with-reason>"
scripts/upstream-merge/log-merge.sh note deviation "<any failed/skipped gate, unavailable credential, warning, or follow-up>"
scripts/upstream-merge/log-merge.sh finalize
```

`log-merge.sh init` may prompt before overwriting a same-day log. If that would
block unattended work, record the same audit fields manually in a clearly named
merge log under `docs/maintenance/upstream-merge-logs/` before continuing.
Do not let the interactive prompt cause the audit trail to be skipped.

The log must capture fetched refs, the pinned `every_code_to_merge`, refreshed
`upstream/main`, merge base, `codex-rs/` tree provenance or temporary OSS
checkout commit, conflict decisions, Kay-only behavior preserved, OSS
classifications, every verification command run, and every failure, warning,
skipped gate, credential gap, or deviation.

## Final Verification

For code/build/script/packaging/workflow/dependency/generated-artifact upstream
merge work, run:

```bash
git diff --check
bash scripts/check-kay-path-deps.sh
bash scripts/upstream-merge/diff-crates.sh --all
bash scripts/upstream-merge/verify.sh
./build-fast.sh
./pre-release.sh
```

If TypeScript SDK paths or generated protocol bindings changed:

```bash
pnpm --dir sdk/typescript test
```

If provider/model behavior changed, run the provider gate for the current Kay
policy: OpenCode Go MiMo V2.5, MiMo V2.5 Pro, DeepSeek V4 Flash with max
reasoning, and MiniMax M2.7. Exclude direct MiniMax.io provider tests.

```bash
bash scripts/pre-release-live-provider-gate.sh
```

Before declaring the merge ready, verify:

- no compiler warnings or script warnings remain
- no unreviewed conflict markers remain
- no unintended `code-rs` path references were reintroduced outside deliberate
  compatibility or historical notes
- no tracked or generated `code-rs -> kay-rs` symlink exists
- no Kay provider, config, auth, packaging, or state-isolation behavior was
  silently reverted
- release is not cut unless explicitly requested or normal `main` automation is
  intentionally being used

If pushing `main`, follow the repo's merge-only push policy. After pushing,
monitor release workflow when relevant:

```bash
scripts/wait-for-gh-run.sh --workflow Release --branch main --failure-logs
```

## Report Back

Summarize:

- Every Code refs and commits merged
- OSS Codex refs reviewed
- OSS changes classified, ported, deferred, or rejected
- Kay-only behavior preserved
- verification commands and outcomes
- any warnings, skipped gates, unavailable credentials, or residual risks
- release workflow/local install status, if applicable
