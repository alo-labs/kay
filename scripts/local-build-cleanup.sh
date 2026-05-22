#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "${SCRIPT_DIR}/.." && pwd)}"

# Keep this list narrow: only transient build/cache artifacts created by local
# build-fast runs. Preserved binaries live under ./code-rs/bin and ./bin.
cleanup_paths=(
  "${TARGET_CACHE_DIR_ABS:-}"
  "code-rs/target"
  "codex-rs/target"
  "target"
)

removed=()
for rel in "${cleanup_paths[@]}"; do
  if [ -z "${rel}" ]; then
    continue
  fi

  path="${rel}"
  if [[ "${path}" != /* ]]; then
    path="${REPO_ROOT}/${path}"
  fi

  if [ -e "${path}" ]; then
    rm -rf -- "${path}"
    removed+=("${path#${REPO_ROOT}/}")
  fi
done

if [ "${#removed[@]}" -eq 0 ]; then
  echo "local build cleanup: no transient artifacts found"
else
  echo "local build cleanup removed: ${removed[*]}"
fi
