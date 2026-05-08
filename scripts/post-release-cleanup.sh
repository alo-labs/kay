#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "${SCRIPT_DIR}/.." && pwd)}"

# Keep this list narrow: only transient build/cache artifacts.
# Planning, spec, and design folders are intentionally preserved.
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

removed=()
for rel in "${cleanup_paths[@]}"; do
  path="${REPO_ROOT}/${rel}"
  if [ -e "${path}" ]; then
    rm -rf -- "${path}"
    removed+=("${rel}")
  fi
done

if [ "${#removed[@]}" -eq 0 ]; then
  echo "post-release cleanup: no transient artifacts found"
else
  echo "post-release cleanup removed: ${removed[*]}"
fi
