#!/usr/bin/env bash
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
use_alias_if_present MINIMAX_LIVE_API_KEY MINIMAX_API_KEY

load_provider_key_from_auth_json OPENCODE_GO_LIVE_API_KEY opencode-go
load_provider_key_from_auth_json MINIMAX_LIVE_API_KEY minimax

missing=()
if [[ -z "${OPENCODE_GO_LIVE_API_KEY:-}" ]]; then
  missing+=("OPENCODE_GO_LIVE_API_KEY, OPENCODE_GO_API_KEY, or provider_credentials.opencode-go.api_key in $AUTH_FILE")
fi
if [[ -z "${MINIMAX_LIVE_API_KEY:-}" ]]; then
  missing+=("MINIMAX_LIVE_API_KEY, MINIMAX_API_KEY, or provider_credentials.minimax.api_key in $AUTH_FILE")
fi

if (( ${#missing[@]} > 0 )); then
  echo "[pre-release/live-provider-gate] missing required live provider credentials:" >&2
  printf '  - %s\n' "${missing[@]}" >&2
  exit 2
fi

echo "[pre-release/live-provider-gate] running OpenCode Go onboarding live smoke"
cd "$ROOT_DIR/code-rs"

KAY_ONBOARDING_LIVE_SMOKE=1 \
KAY_ONBOARDING_LIVE_SMOKE_MODEL_FILTER="OpenCode Go" \
KAY_ONBOARDING_LIVE_SMOKE_TURN_TIMEOUT_SECS="${KAY_ONBOARDING_LIVE_SMOKE_TURN_TIMEOUT_SECS:-900}" \
cargo test -p code-cli --test onboarding_provider_notes_app_live_smoke -- --nocapture
