#!/usr/bin/env bash

set -euo pipefail

PROJECT_DIR="${PROJECT_DIR:-$HOME/projects/test-notes-app}"
REPO_SLUG="${REPO_SLUG:-alo-exp/test-notes-app}"
KAY_HOME_DIR="${KAY_HOME:-$HOME/.kay}"
SKIP_INSTALL=0

usage() {
  cat <<'EOF'
Usage: bootstrap-test-notes-app.sh [--help] [--skip-install]

Provision the real test-notes-app checkout under ~/projects/test-notes-app and
ensure Kay's isolated home exists under ~/.kay.
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip-install)
      SKIP_INSTALL=1
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
  shift
done

step() {
  printf '==> %s\n' "$1"
}

ensure_kay_home() {
  step "Ensuring Kay home exists at $KAY_HOME_DIR"
  mkdir -p "$KAY_HOME_DIR"
}

ensure_repo_checkout() {
  if [ -d "$PROJECT_DIR/.git" ]; then
    step "Found existing checkout at $PROJECT_DIR"
    return
  fi

  step "Cloning $REPO_SLUG into $PROJECT_DIR"
  mkdir -p "$(dirname "$PROJECT_DIR")"
  gh repo clone "$REPO_SLUG" "$PROJECT_DIR"
}

ensure_git_remote() {
  local current_remote
  current_remote="$(git -C "$PROJECT_DIR" remote get-url origin 2>/dev/null || true)"
  if [ "$current_remote" != "https://github.com/$REPO_SLUG.git" ] && [ "$current_remote" != "git@github.com:$REPO_SLUG.git" ]; then
    if [ -n "$current_remote" ]; then
      step "Updating origin remote for $PROJECT_DIR"
      git -C "$PROJECT_DIR" remote set-url origin "https://github.com/$REPO_SLUG.git"
    else
      step "Adding origin remote for $PROJECT_DIR"
      git -C "$PROJECT_DIR" remote add origin "https://github.com/$REPO_SLUG.git"
    fi
  fi
}

ensure_dependencies() {
  if [ "$SKIP_INSTALL" -eq 1 ]; then
    step "Skipping dependency installation"
    return
  fi

  if [ -f "$PROJECT_DIR/package.json" ]; then
    step "Installing Node dependencies for test-notes-app"
    (cd "$PROJECT_DIR" && npm install)
  fi
}

main() {
  ensure_kay_home
  ensure_repo_checkout
  ensure_git_remote
  ensure_dependencies

  cat <<EOF
Ready.
- Notes app checkout: $PROJECT_DIR
- GitHub repo: https://github.com/$REPO_SLUG
- Kay home: $KAY_HOME_DIR
EOF
}

main

