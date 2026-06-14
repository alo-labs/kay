#!/usr/bin/env bash
# Live provider release gate profiles (KAY_PRE_RELEASE_LIVE_PROVIDER_GATE):
#   default | release — OpenCode Go onboarding notes-app smoke + Xiaomi acceptance
#   minimax | minimax-m3 — MiniMax.io MiniMax-M3 only (onboarding smoke, exec E2E, acceptance)
#
# MiniMax-M3 live tests always use the built-in `minimax` provider (api.minimax.io),
# never OpenCode Go (`opencode-go`).
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
AUTH_HOME=${KAY_HOME:-"$HOME/.kay"}
AUTH_FILE="$AUTH_HOME/auth.json"

use_alias_if_present() {
  local target_env=$1
  local alias_env=$2

  if [[ -n "${!target_env:-}" || -z "${!alias_env:-}" ]]; then
    return
  fi

  printf -v "$target_env" '%s' "${!alias_env}"
  export "$target_env"
}

load_provider_key_from_auth_json() {
  local target_env=$1
  local provider=$2

  if [[ -n "${!target_env:-}" || ! -f "$AUTH_FILE" ]]; then
    return
  fi

  local value
  value=$(
    python3 - "$AUTH_FILE" "$provider" <<'PY'
import json
import sys

auth_file, provider = sys.argv[1], sys.argv[2]

try:
    with open(auth_file, "r", encoding="utf-8") as f:
        data = json.load(f)
except Exception:
    sys.exit(0)

key = (
    data.get("provider_credentials", {})
    .get(provider, {})
    .get("api_key", "")
)

if isinstance(key, str):
    print(key.strip(), end="")
PY
  )

  if [[ -n "$value" ]]; then
    printf -v "$target_env" '%s' "$value"
    export "$target_env"
  fi
}

use_alias_if_present OPENCODE_GO_LIVE_API_KEY OPENCODE_GO_API_KEY
use_alias_if_present XIAOMI_LIVE_API_KEY XIAOMI_API_KEY
use_alias_if_present MINIMAX_LIVE_API_KEY MINIMAX_API_KEY

load_provider_key_from_auth_json OPENCODE_GO_LIVE_API_KEY opencode-go
load_provider_key_from_auth_json XIAOMI_LIVE_API_KEY xiaomi
load_provider_key_from_auth_json MINIMAX_LIVE_API_KEY minimax

missing=()

require_opencode_go_key() {
  if [[ -z "${OPENCODE_GO_LIVE_API_KEY:-}" ]]; then
    missing+=("OPENCODE_GO_LIVE_API_KEY, OPENCODE_GO_API_KEY, or provider_credentials.opencode-go.api_key in $AUTH_FILE")
  fi
}

require_xiaomi_key() {
  if [[ -z "${XIAOMI_LIVE_API_KEY:-}" ]]; then
    missing+=("XIAOMI_LIVE_API_KEY, XIAOMI_API_KEY, or provider_credentials.xiaomi.api_key in $AUTH_FILE")
  fi
}

require_minimax_key() {
  if [[ -z "${MINIMAX_LIVE_API_KEY:-}" ]]; then
    missing+=("MINIMAX_LIVE_API_KEY, MINIMAX_API_KEY, or provider_credentials.minimax.api_key in $AUTH_FILE")
  fi
}

fail_if_missing_keys() {
  if (( ${#missing[@]} > 0 )); then
    echo "[pre-release/live-provider-gate] missing required live provider credentials:" >&2
    printf '  - %s\n' "${missing[@]}" >&2
    exit 2
  fi
}

run_default_release_gate() {
  require_opencode_go_key
  require_xiaomi_key
  fail_if_missing_keys

  echo "[pre-release/live-provider-gate] running curated OpenCode Go onboarding live smoke"
  cd "$ROOT_DIR/kay-rs"

  live_env=(
    "KAY_ONBOARDING_LIVE_SMOKE=1"
    "KAY_ONBOARDING_LIVE_SMOKE_TURN_TIMEOUT_SECS=${KAY_ONBOARDING_LIVE_SMOKE_TURN_TIMEOUT_SECS:-1800}"
    "KAY_ONBOARDING_LIVE_SMOKE_MODEL_FILTER=opencode-go/mimo-v2.5,opencode-go/mimo-v2.5-pro,opencode-go/deepseek-v4-flash,opencode-go/minimax-m2.7"
  )

  env "${live_env[@]}" cargo test -p code-cli --test onboarding_provider_notes_app_live_smoke -- --nocapture

  echo "[pre-release/live-provider-gate] running Xiaomi provider acceptance"
  env XIAOMI_LIVE_API_KEY="$XIAOMI_LIVE_API_KEY" \
    cargo test -p code-cli --test provider_model_acceptance xiaomi_provider_model_acceptance_matrix -- --nocapture
}

run_minimax_m3_gate() {
  require_minimax_key
  fail_if_missing_keys

  echo "[pre-release/live-provider-gate] running MiniMax.io MiniMax-M3 onboarding notes-app live smoke"
  cd "$ROOT_DIR/kay-rs"

  minimax_live_env=(
    "KAY_ONBOARDING_LIVE_SMOKE=1"
    "MINIMAX_LIVE_API_KEY=$MINIMAX_LIVE_API_KEY"
    "KAY_ONBOARDING_LIVE_SMOKE_TURN_TIMEOUT_SECS=${KAY_ONBOARDING_LIVE_SMOKE_TURN_TIMEOUT_SECS:-1800}"
    "KAY_ONBOARDING_LIVE_SMOKE_MODEL_FILTER=MiniMax-M3"
  )

  env "${minimax_live_env[@]}" cargo test -p code-cli --test onboarding_provider_notes_app_live_smoke -- --nocapture

  echo "[pre-release/live-provider-gate] running MiniMax.io MiniMax-M3 live exec E2E"
  env MINIMAX_LIVE_API_KEY="$MINIMAX_LIVE_API_KEY" \
    cargo test -p code-cli --test minimax_live_e2e minimax_m3_live_exec_edge_cases -- --nocapture

  echo "[pre-release/live-provider-gate] running MiniMax.io provider acceptance"
  env MINIMAX_LIVE_API_KEY="$MINIMAX_LIVE_API_KEY" \
    cargo test -p code-cli --test provider_model_acceptance minimax_provider_model_acceptance_matrix -- --nocapture
}

case "${KAY_PRE_RELEASE_LIVE_PROVIDER_GATE:-default}" in
  default | release)
    run_default_release_gate
    ;;
  minimax | minimax-m3)
    run_minimax_m3_gate
    ;;
  *)
    echo "[pre-release/live-provider-gate] unknown gate profile: ${KAY_PRE_RELEASE_LIVE_PROVIDER_GATE}" >&2
    echo "[pre-release/live-provider-gate] expected one of: default, release, minimax, minimax-m3" >&2
    exit 64
    ;;
esac
