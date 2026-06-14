#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
BASE_CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$ROOT_DIR/target"}
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR

LOG_DIR=$(mktemp -d "${TMPDIR:-/tmp}/kay-pre-release.XXXXXX")
pids=()
names=()
logs=()

unset_live_provider_test_env() {
  unset OPENCODE_GO_API_KEY
  unset OPENCODE_GO_LIVE_API_KEY
  unset XIAOMI_API_KEY
  unset XIAOMI_LIVE_API_KEY
  unset MINIMAX_API_KEY
  unset MINIMAX_LIVE_API_KEY
  unset TEST_NOTES_APP_MODEL_FILTER
  unset KAY_ONBOARDING_LIVE_SMOKE
  unset KAY_ONBOARDING_LIVE_SMOKE_MODEL_FILTER
}

cleanup() {
  local code=$?
  if (( code != 0 )); then
    for pid in "${pids[@]:-}"; do
      kill "$pid" 2>/dev/null || true
    done
  fi
  if [[ "${KEEP_PRE_RELEASE_LOGS:-0}" == "1" ]]; then
    echo "[pre-release] logs kept at $LOG_DIR"
  else
    rm -rf "$LOG_DIR"
  fi
}
trap cleanup EXIT

start_job() {
  local name=$1
  shift
  local log="$LOG_DIR/${name}.log"

  echo "[pre-release] starting $name"
  (
    set -euo pipefail
    "$@"
  ) >"$log" 2>&1 &
  pids+=("$!")
  names+=("$name")
  logs+=("$log")
}

run_dev_fast_and_cli_smokes() {
  local target_dir="$BASE_CARGO_TARGET_DIR/pre-release-dev-fast"

  echo "[pre-release] building Kay CLI (dev-fast)"
  cd "$ROOT_DIR/kay-rs"
  CARGO_TARGET_DIR="$target_dir" cargo build --locked --profile dev-fast --bin kay --bin code

  echo "[pre-release] running Kay CLI smokes (skip cargo tests)"
  SKIP_CARGO_TESTS=1 \
    SKIP_POST_RELEASE_CLEANUP=1 \
    CI_CLI_BIN="$target_dir/dev-fast/kay" \
    bash "$ROOT_DIR/scripts/ci-tests.sh"
}

run_workspace_nextest() {
  echo "[pre-release] running workspace tests (nextest)"
  cd "$ROOT_DIR/kay-rs"
  unset_live_provider_test_env
  CARGO_TARGET_DIR="$BASE_CARGO_TARGET_DIR/pre-release-nextest" \
    cargo +stable nextest run --no-fail-fast --locked
}

run_post_release_cleanup_policy() {
  echo "[pre-release] running post-release cleanup policy"
  bash "$ROOT_DIR/scripts/test-post-release-cleanup.sh"
}

run_live_provider_gate() {
  if [[ "${KAY_PRE_RELEASE_SKIP_LIVE_PROVIDER_GATE:-0}" == "1" ]]; then
    echo "[pre-release] skipping live provider/model release gate by KAY_PRE_RELEASE_SKIP_LIVE_PROVIDER_GATE=1"
    return
  fi

  echo "[pre-release] running live provider/model release gate (KAY_PRE_RELEASE_LIVE_PROVIDER_GATE=${KAY_PRE_RELEASE_LIVE_PROVIDER_GATE:-default})"
  CARGO_TARGET_DIR="$BASE_CARGO_TARGET_DIR/pre-release-live-provider" \
    bash "$ROOT_DIR/scripts/pre-release-live-provider-gate.sh"
}

start_job dev-fast-cli run_dev_fast_and_cli_smokes
start_job workspace-nextest run_workspace_nextest
start_job post-release-cleanup run_post_release_cleanup_policy
start_job live-provider-gate run_live_provider_gate

failed=0
for idx in "${!pids[@]}"; do
  pid=${pids[$idx]}
  name=${names[$idx]}
  log=${logs[$idx]}

  if wait "$pid"; then
    echo "[pre-release] passed $name"
  else
    status=$?
    echo "[pre-release] failed $name (exit $status)" >&2
    echo "[pre-release] --- $name log ---" >&2
    cat "$log" >&2 || true
    echo "[pre-release] --- end $name log ---" >&2
    failed=1
  fi
done

if (( failed != 0 )); then
  exit 1
fi

echo "[pre-release] all checks passed"
