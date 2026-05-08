#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SANDBOX="$(mktemp -d)"
trap 'rm -rf "${SANDBOX}"' EXIT

cleanup_paths=(
  "code-rs/target"
  "codex-rs/target"
  "target"
  ".tmp"
  ".cache"
  "build"
  "dist"
  "coverage"
  ".pytest_cache"
  "node_modules"
)
preserve_paths=(
  ".planning"
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

OUTPUT="$(REPO_ROOT="${SANDBOX}" bash "${SCRIPT_DIR}/post-release-cleanup.sh")"

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

if ! printf '%s\n' "${OUTPUT}" | grep -q 'post-release cleanup removed'; then
  echo "FAIL: cleanup output missing removal summary" >&2
  exit 1
fi

SECOND_OUTPUT="$(REPO_ROOT="${SANDBOX}" bash "${SCRIPT_DIR}/post-release-cleanup.sh")"
if ! printf '%s\n' "${SECOND_OUTPUT}" | grep -q 'no transient artifacts found'; then
  echo "FAIL: cleanup should be idempotent" >&2
  exit 1
fi

echo "PASS: post-release cleanup preserves planning/spec/design content and is idempotent"
