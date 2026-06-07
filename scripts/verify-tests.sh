#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
STATE_ROOT="${SB_RUNTIME_STATE_DIR:-$HOME/.codex/.silver-bullet}"
MARKER_FILE="${SILVER_BULLET_VERIFY_TESTS_STATE_FILE:-$STATE_ROOT/verify-tests-state}"

if [[ -L "$STATE_ROOT" || -L "$MARKER_FILE" ]]; then
  echo "refusing to write verify-tests marker through a symlink" >&2
  exit 1
fi

commands=()
while IFS= read -r command; do
  commands+=("$command")
done < <(jq -r '.verify_commands[]?' "$ROOT_DIR/.silver-bullet.json" 2>/dev/null || true)

if [[ ${#commands[@]} -eq 0 ]]; then
  commands=("./build-fast.sh")
fi

for command in "${commands[@]}"; do
  [[ -n "$command" ]] || continue
  echo "[verify-tests] running: $command"
  (
    cd "$ROOT_DIR"
    bash -lc "$command"
  )
done

mkdir -p "$(dirname "$MARKER_FILE")"
tmp=$(mktemp)
{
  printf 'timestamp=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  printf 'repo=%s\n' "$ROOT_DIR"
  printf 'commands=%s\n' "${commands[*]}"
} > "$tmp"
mv "$tmp" "$MARKER_FILE"
echo "[verify-tests] marker written: $MARKER_FILE"
