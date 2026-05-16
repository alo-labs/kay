#!/usr/bin/env bash
set -euo pipefail

# Unified, fast verification for upstream-merge runs.
# - Runs build-fast.sh (treat warnings as failures via repo policy)
# - Compiles API-surface tests for code-core (no test execution)
# - Emits a JSON summary to .github/auto/VERIFY.json

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." >/dev/null 2>&1 && pwd)"
cd "$ROOT_DIR"

mkdir -p .github/auto

status_build="ok"
status_api="ok"
status_guards="ok"
status_branding="ok"
status_upstream_sync="ok"
oss_ahead_by=0
oss_behind_by=0

{
  echo "[verify] START $(date -u +%FT%TZ)"
  echo "[verify] repo: $ROOT_DIR"
  echo "[verify] STEP 1: build-fast.sh"
}

# Use the same environment as the job (including sccache) for consistency
export KEEP_ENV=1
# If running outside a fully-provisioned GitHub Actions runner, sccache's GHA backend
# can fail to start. In that case, disable sccache to allow local verification.
if [[ -z "${ACTIONS_CACHE_URL:-}" || -z "${ACTIONS_RUNTIME_TOKEN:-}" ]]; then
  export SCCACHE_DISABLE=1
  unset RUSTC_WRAPPER CARGO_BUILD_RUSTC_WRAPPER SCCACHE SCCACHE_BIN
fi
if ! ./build-fast.sh 2>&1 | tee .github/auto/VERIFY_build-fast.log; then
  status_build="fail"
fi

{
  echo "[verify] STEP 2: cargo check (core tests compile)"
}
# Respect pre-set CARGO_HOME/TARGET_DIR to share caches across steps
CODE_TARGET_DIR="$ROOT_DIR/code-rs/target"
export CARGO_HOME="${CARGO_HOME:-$ROOT_DIR/.cargo-home}"
if [ -z "${CARGO_TARGET_DIR:-}" ]; then
  export CARGO_TARGET_DIR="$CODE_TARGET_DIR"
fi
# Ensure rustup also uses a repo-local, writable directory to avoid HOME permission issues on CI
export RUSTUP_HOME="${RUSTUP_HOME:-${CARGO_HOME%/}/rustup}"
mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR" "$RUSTUP_HOME" >/dev/null 2>&1 || true
if ! (CARGO_TARGET_DIR="$CODE_TARGET_DIR" cd code-rs && cargo test -p code-core --test opencode_go_provider --no-run --quiet) 2>&1 | tee .github/auto/VERIFY_api-check.log; then
  status_api="fail"
fi

#
# STEP 3: Static guards for fork-specific functionality
# - Ensure browser/agent tools are still registered (not just handlers present)
# - Ensure version handling remains via codex_version in default_client
# - Ensure browser fetch action (legacy web_fetch) and web_search tool presence is consistent with fork policy
{
  echo "[verify] STEP 3: static guards (tools + UA/version)"
}
guards_log=.github/auto/VERIFY_guards.log
: > "$guards_log"

# Guard A: Browser and agent tool wiring must still be advertised.
if ! rg -n 'create_browser_tool|name:[[:space:]]*"browser"' code-rs/core/src/openai_tools.rs >/dev/null 2>&1; then
  printf "[guards] no 'browser' tool references found in openai_tools.rs - tool family likely dropped\n" | tee -a "$guards_log"
  status_guards="fail"
fi
if ! rg -n 'create_agent_tool|name:[[:space:]]*"agent"' code-rs/core/src/openai_tools.rs >/dev/null 2>&1; then
  printf "[guards] no 'agent' tool references found in openai_tools.rs - tool family likely dropped\n" | tee -a "$guards_log"
  status_guards="fail"
fi

# Guard B: default_client should reference code_version::wire_compatible_version for UA
if ! rg -n 'code_version::wire_compatible_version' code-rs/core/src/default_client.rs >/dev/null 2>&1; then
  printf "[guards] code_version::wire_compatible_version not referenced in core/default_client.rs\n" | tee -a "$guards_log"
  status_guards="fail"
fi

# Summarize guards
echo "guards=${status_guards}" >> "$guards_log"

# STEP 4: Branding guard parity with CI (non-fixing check)
{
  echo "[verify] STEP 4: branding guard (TUI/CLI user-visible)"
}
DEFAULT_BRANCH_LOCAL=${DEFAULT_BRANCH:-main}
# Try to fetch origin to ensure refs exist; ignore failure for local runs
git fetch origin "$DEFAULT_BRANCH_LOCAL" >/dev/null 2>&1 || true
range_ref="origin/${DEFAULT_BRANCH_LOCAL}..HEAD"
changed_files=$(git diff --name-only $range_ref -- 'code-rs/tui/**' 'codex-cli/**' | tr '\n' ' ' || true)
branding_log=.github/auto/VERIFY_branding.log
: > "$branding_log"
if [ -n "${changed_files:-}" ]; then
  echo "[branding] scanning changed TUI/CLI files for user-visible 'Codex' strings relative to $range_ref" | tee -a "$branding_log"
  if git diff -U0 --no-color $range_ref -- $changed_files \
    | grep -E '^\+' \
    | grep -E '"[^"]*Codex[^"]*"|'\''[^'\''']*Codex[^'\''']*'\''|`[^`]*Codex[^`]*`' \
    | grep -Evi '(codex-rs|codex-[a-z0-9_-]+|https?://|Cargo|crate|package|workspace)' \
    | sed 's/^/+ /' | tee -a "$branding_log"; then
    echo "[branding] NOTE: guidance only; no changes applied." | tee -a "$branding_log"
    status_branding="notice"
  else
    echo "[branding] no user-visible 'Codex' strings detected in changed TUI/CLI files." | tee -a "$branding_log"
  fi
else
  echo "[branding] no relevant file changes vs $range_ref; skipping" | tee -a "$branding_log"
fi

# STEP 5: Upstream sync advisory
{
  echo "[verify] STEP 5: upstream sync advisory"
}
upstream_log=.github/auto/VERIFY_upstream.log
: > "$upstream_log"
if git remote get-url oss >/dev/null 2>&1; then
  git fetch --no-tags --prune oss main >/dev/null 2>&1 || true
  if git rev-parse --verify oss/main >/dev/null 2>&1; then
    oss_ahead_by=$(git rev-list --count oss/main..HEAD || echo 0)
    oss_behind_by=$(git rev-list --count HEAD..oss/main || echo 0)
    {
      echo "[upstream] oss/main ahead of HEAD: ${oss_ahead_by}";
      echo "[upstream] HEAD ahead of oss/main: ${oss_behind_by}";
      echo "[upstream] note: drift is advisory here; it must be triaged in merge review, not treated as a release blocker by itself.";
    } | tee -a "$upstream_log"
    if [ "${oss_behind_by}" -gt 0 ]; then
      status_upstream_sync="notice"
    fi
  else
    echo "[upstream] oss/main ref unavailable after fetch; skipping advisory" | tee -a "$upstream_log"
    status_upstream_sync="missing"
  fi
else
  echo "[upstream] remote 'oss' unavailable; skipping advisory" | tee -a "$upstream_log"
  status_upstream_sync="missing"
fi

rc=0
# Branding is guide-only and does not affect rc. Fail only on build/api/guards.
if [[ "$status_build" != ok || "$status_api" != ok || "$status_guards" != ok ]]; then
  rc=1
fi

cat > .github/auto/VERIFY.json <<JSON
{
  "build_fast": "$status_build",
  "api_check": "$status_api",
  "guards": "$status_guards",
  "branding": "$status_branding",
  "upstream_sync": "$status_upstream_sync",
  "oss_ahead_by": ${oss_ahead_by},
  "oss_behind_by": ${oss_behind_by}
}
JSON

echo "[verify] SUMMARY: build_fast=$status_build api_check=$status_api"
exit $rc
