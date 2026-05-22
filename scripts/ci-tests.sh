#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

echo "[ci-tests] Running curated integration tests..."
if [[ "${SKIP_CARGO_TESTS:-0}" != "1" ]]; then
  pushd "$ROOT_DIR/kay-rs" >/dev/null

  cargo test -p code-login --tests -q
  cargo test -p code-chatgpt --tests -q
  cargo test -p code-apply-patch --tests -q
  cargo test -p code-execpolicy --tests -q
  cargo test -p mcp-types --tests -q

  popd >/dev/null
fi


echo "[ci-tests] CLI smokes with host Kay binary..."
BIN="${CI_CLI_BIN:-}"
if [[ -z "${BIN}" ]]; then
  if [[ -x "$ROOT_DIR/kay-rs/target/dev-fast/kay" ]]; then
    BIN="$ROOT_DIR/kay-rs/target/dev-fast/kay"
  elif [[ -x "$ROOT_DIR/kay-rs/target/dev-fast/code" ]]; then
    BIN="$ROOT_DIR/kay-rs/target/dev-fast/code"
  elif [[ -x "$ROOT_DIR/kay-rs/target/debug/kay" ]]; then
    BIN="$ROOT_DIR/kay-rs/target/debug/kay"
  elif [[ -x "$ROOT_DIR/kay-rs/target/debug/code" ]]; then
    BIN="$ROOT_DIR/kay-rs/target/debug/code"
  fi
fi

if [[ -z "${BIN}" || ! -x "${BIN}" ]]; then
  echo "[ci-tests] CLI binary not found; building debug binary..."
  pushd "$ROOT_DIR/kay-rs" >/dev/null
  cargo build --bin kay -q
  popd >/dev/null
  BIN="$ROOT_DIR/kay-rs/target/debug/kay"
fi

"${BIN}" --version >/dev/null
"${BIN}" completion bash >/dev/null
"${BIN}" doctor >/dev/null || true

if [[ "${SKIP_POST_RELEASE_CLEANUP:-0}" != "1" ]]; then
  echo "[ci-tests] Post-release cleanup policy..."
  bash "$ROOT_DIR/scripts/test-post-release-cleanup.sh"
fi

echo "[ci-tests] Done."
