#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SANDBOX="$(mktemp -d)"
trap 'rm -rf "${SANDBOX}"' EXIT

cleanup_paths=(
  "kay-rs/target"
  "codex-rs/target"
  "target"
)
preserve_paths=(
  "kay-rs/bin"
  "docs/specs"
  "docs/design"
)

for rel in "${cleanup_paths[@]}"; do
  mkdir -p "${SANDBOX}/${rel}"
done
for rel in "${preserve_paths[@]}"; do
  mkdir -p "${SANDBOX}/${rel}"
done
mkdir -p "${SANDBOX}/keep"
touch "${SANDBOX}/keep/keep.txt"
touch "${SANDBOX}/kay-rs/bin/kay"
touch "${SANDBOX}/kay-rs/bin/code"

OUTPUT="$(REPO_ROOT="${SANDBOX}" TARGET_CACHE_DIR_ABS="${SANDBOX}/.kay/working/_target-cache/kay/example/kay-rs" bash "${SCRIPT_DIR}/local-build-cleanup.sh")"

for rel in "${cleanup_paths[@]}"; do
  if [ -e "${SANDBOX}/${rel}" ]; then
    echo "FAIL: expected cleanup to remove ${rel}" >&2
    exit 1
  fi
done

for rel in "${preserve_paths[@]}"; do
  if [ ! -d "${SANDBOX}/${rel}" ]; then
    echo "FAIL: expected cleanup to preserve ${rel}" >&2
    exit 1
  fi
done

if [ ! -f "${SANDBOX}/keep/keep.txt" ]; then
  echo "FAIL: cleanup removed non-transient file" >&2
  exit 1
fi

if [ ! -f "${SANDBOX}/kay-rs/bin/kay" ] || [ ! -f "${SANDBOX}/kay-rs/bin/code" ]; then
  echo "FAIL: cleanup removed preserved bin outputs" >&2
  exit 1
fi

if ! printf '%s\n' "${OUTPUT}" | grep -q 'local build cleanup removed'; then
  echo "FAIL: cleanup output missing removal summary" >&2
  exit 1
fi

SECOND_OUTPUT="$(REPO_ROOT="${SANDBOX}" TARGET_CACHE_DIR_ABS="${SANDBOX}/.kay/working/_target-cache/kay/example/kay-rs" bash "${SCRIPT_DIR}/local-build-cleanup.sh")"
if ! printf '%s\n' "${SECOND_OUTPUT}" | grep -q 'no transient artifacts found'; then
  echo "FAIL: cleanup should be idempotent" >&2
  exit 1
fi

echo "PASS: local build cleanup removes only transient build trees and is idempotent"
