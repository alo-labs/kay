#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "${SCRIPT_DIR}/.." && pwd)}"
DRY_RUN=false

usage() {
  cat <<'EOF'
Usage: upgrade-local-kay-latest.sh [--dry-run]

Upgrades the local Kay command to the latest published release and verifies
that the visible `kay` command reports that version.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option '$1'" >&2
      usage >&2
      exit 1
      ;;
  esac
done

latest_version() {
  local tag=""
  if command -v gh >/dev/null 2>&1; then
    tag=$(gh release view --repo alo-labs/kay --json tagName -q .tagName 2>/dev/null || true)
  fi

  if [[ -z "$tag" ]]; then
    if command -v curl >/dev/null 2>&1; then
      tag=$(
        curl -fsSL https://api.github.com/repos/alo-labs/kay/releases/latest \
          | python3 -c 'import json,sys; print(json.load(sys.stdin).get("tag_name",""))'
      )
    elif command -v wget >/dev/null 2>&1; then
      tag=$(
        wget -q -O - https://api.github.com/repos/alo-labs/kay/releases/latest \
          | python3 -c 'import json,sys; print(json.load(sys.stdin).get("tag_name",""))'
      )
    fi
  fi

  tag="${tag#rust-v}"
  tag="${tag#v}"
  if [[ -z "$tag" ]]; then
    echo "error: unable to resolve latest Kay release version" >&2
    exit 1
  fi
  printf '%s\n' "$tag"
}

visible_kay_path() {
  command -v kay 2>/dev/null || true
}

classify_install() {
  local kay_path="$1"
  case "$kay_path" in
    *"/node_modules/@alo-labs/kay/"*|*"/node_modules/.bin/kay")
      printf 'npm\n'
      ;;
    *"/.bun/"*)
      printf 'bun\n'
      ;;
    *"/.local/share/pnpm/"*|*"/pnpm/"*)
      printf 'pnpm\n'
      ;;
    /opt/homebrew/*|/usr/local/*)
      printf 'brew\n'
      ;;
    *)
      printf 'standalone\n'
      ;;
  esac
}

upgrade_command_for() {
  local manager="$1"
  case "$manager" in
    npm)
      if ! command -v npm >/dev/null 2>&1; then
        echo "error: visible Kay appears npm-managed, but npm is not on PATH" >&2
        exit 1
      fi
      printf 'npm install -g @alo-labs/kay@latest\n'
      ;;
    bun)
      if ! command -v bun >/dev/null 2>&1; then
        echo "error: visible Kay appears bun-managed, but bun is not on PATH" >&2
        exit 1
      fi
      printf 'bun add -g @alo-labs/kay@latest\n'
      ;;
    pnpm)
      if ! command -v pnpm >/dev/null 2>&1; then
        echo "error: visible Kay appears pnpm-managed, but pnpm is not on PATH" >&2
        exit 1
      fi
      printf 'pnpm add -g @alo-labs/kay@latest\n'
      ;;
    brew)
      if ! command -v brew >/dev/null 2>&1; then
        echo "error: visible Kay appears Homebrew-managed, but brew is not on PATH" >&2
        exit 1
      fi
      printf 'brew upgrade kay || brew upgrade code\n'
      ;;
    standalone)
      printf 'bash %q --release latest\n' "$REPO_ROOT/scripts/install/install.sh"
      ;;
    *)
      echo "error: unknown Kay install manager: $manager" >&2
      exit 1
      ;;
  esac
}

reported_version() {
  local cmd="$1"
  "$cmd" --version 2>/dev/null | sed -nE 's/.* ([0-9][0-9A-Za-z.+-]*)$/\1/p' | head -n 1
}

latest="$(latest_version)"
kay_path="$(visible_kay_path)"
manager="$(classify_install "$kay_path")"
upgrade_cmd="$(upgrade_command_for "$manager")"

echo "[local-kay-upgrade] latest release: $latest"
if [[ -n "$kay_path" ]]; then
  echo "[local-kay-upgrade] visible kay: $kay_path ($manager)"
else
  echo "[local-kay-upgrade] visible kay: not found; installing standalone"
fi
echo "[local-kay-upgrade] command: $upgrade_cmd"

if [[ "$DRY_RUN" == true ]]; then
  exit 0
fi

eval "$upgrade_cmd"
hash -r 2>/dev/null || true
export PATH="$HOME/.local/bin:$PATH"

verify_path="$(visible_kay_path)"
if [[ -z "$verify_path" && -x "$HOME/.local/bin/kay" ]]; then
  verify_path="$HOME/.local/bin/kay"
fi

if [[ -z "$verify_path" ]]; then
  echo "error: kay is still not visible after upgrade" >&2
  exit 1
fi

installed="$(reported_version "$verify_path")"
if [[ "$installed" != "$latest" ]]; then
  echo "error: visible Kay is $installed at $verify_path, expected $latest" >&2
  echo "       Remove older Kay installs or adjust PATH so the latest installation is first." >&2
  exit 1
fi

echo "[local-kay-upgrade] verified kay $installed at $verify_path"
