#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "${SCRIPT_DIR}/.." && pwd)}"
INSTALL_SH="$REPO_ROOT/scripts/install/install.sh"
DRY_RUN=false
brew_formula=""
manager=""

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
  local kay_path

  kay_path="$(command -v kay 2>/dev/null || true)"
  if [[ -z "$kay_path" ]]; then
    return 0
  fi

  if command -v python3 >/dev/null 2>&1; then
    python3 - "$kay_path" <<'PY'
import os
import sys

print(os.path.realpath(sys.argv[1]))
PY
  else
    printf '%s\n' "$kay_path"
  fi
}

brew_formula_for_path() {
  local target_path="$1"
  local output=""

  if ! command -v brew >/dev/null 2>&1; then
    return 0
  fi

  output="$(brew which-formula "$target_path" 2>/dev/null || true)"
  if [[ -z "$output" ]]; then
    return 0
  fi

  printf '%s\n' "$output" | awk 'NR==1 {print $1}'
}

is_node_wrapper() {
  local target_path="$1"

  [[ -f "$target_path" ]] && grep -qF '#!/usr/bin/env node' "$target_path" 2>/dev/null
}

classify_install() {
  local kay_path="$1"
  brew_formula=""
  manager=""

  if [[ -z "$kay_path" ]]; then
    manager="standalone"
    return
  fi

  case "$kay_path" in
    *"/.kay/packages/standalone/"*)
      manager="standalone"
      return
      ;;
    *"/node_modules/@alo-labs/kay/"*|*"/node_modules/.bin/kay")
      manager="npm"
      return
      ;;
    *"/.bun/"*)
      manager="bun"
      return
      ;;
    *"/.local/share/pnpm/"*|*"/pnpm/"*)
      manager="pnpm"
      return
      ;;
  esac

  if is_node_wrapper "$kay_path"; then
    case "$kay_path" in
      *"/.bun/"*)
        manager="bun"
        ;;
      *)
        manager="npm"
        ;;
    esac
    return
  fi

  local detected_brew_formula
  detected_brew_formula="$(brew_formula_for_path "$kay_path")"
  if [[ -n "$detected_brew_formula" ]]; then
    manager="brew"
    brew_formula="$detected_brew_formula"
    return
  fi

  manager="standalone"
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
      printf 'brew upgrade %s\n' "${brew_formula:-kay}"
      ;;
    standalone)
      printf 'bash %q --release latest\n' "$INSTALL_SH"
      ;;
    *)
      printf 'bash %q --release latest\n' "$INSTALL_SH"
      ;;
  esac
}

reported_version() {
  local cmd="$1"
  "$cmd" --version 2>/dev/null | sed -nE 's/.* ([0-9][0-9A-Za-z.+-]*)$/\1/p' | head -n 1
}

run_upgrade() {
  local upgrade_cmd="$1"
  eval "$upgrade_cmd"
  hash -r 2>/dev/null || true
  export PATH="$HOME/.local/bin:$PATH"
}

verify_visible_kay() {
  local expected="$1"
  local verify_path installed

  verify_path="$(visible_kay_path)"
  if [[ -z "$verify_path" && -x "$HOME/.local/bin/kay" ]]; then
    verify_path="$HOME/.local/bin/kay"
  fi

  if [[ -z "$verify_path" ]]; then
    echo "error: kay is still not visible after upgrade" >&2
    return 1
  fi

  installed="$(reported_version "$verify_path")"
  if [[ "$installed" != "$expected" ]]; then
    echo "error: visible Kay is $installed at $verify_path, expected $expected" >&2
    return 1
  fi

  echo "[local-kay-upgrade] verified kay $installed at $verify_path"
}

latest="$(latest_version)"
kay_path="$(visible_kay_path)"
classify_install "$kay_path"
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

run_upgrade "$upgrade_cmd"
if ! verify_visible_kay "$latest"; then
  if [[ "$manager" != "standalone" ]]; then
    echo "[local-kay-upgrade] retrying with standalone installer" >&2
    fallback_cmd="$(upgrade_command_for standalone)"
    echo "[local-kay-upgrade] command: $fallback_cmd"
    run_upgrade "$fallback_cmd"
    verify_visible_kay "$latest"
  else
    echo "       Remove older Kay installs or adjust PATH so the latest installation is first." >&2
    exit 1
  fi
fi
