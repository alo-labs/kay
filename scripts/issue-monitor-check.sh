#!/usr/bin/env bash
# Detect open GitHub issues that still need handling for the Kay issue monitor.
set -euo pipefail

REPO="${KAY_ISSUE_MONITOR_REPO:-alo-labs/kay}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STATE_FILE="$ROOT/.kay/issue-monitor-state.json"
LOG_FILE="$ROOT/.kay/issue-monitor.log"
NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

mkdir -p "$ROOT/.kay"
if [[ ! -f "$STATE_FILE" ]]; then
  printf '%s\n' '{"last_check_at":null,"last_seen_issue_numbers":[],"handled_issue_numbers":[]}' >"$STATE_FILE"
fi

OPEN_JSON="$(gh issue list --repo "$REPO" --state open --limit 30 --json number,title,createdAt,labels)"
HANDLED="$(jq -r '.handled_issue_numbers // [] | map(tostring) | join(",")' "$STATE_FILE")"
OPEN_NUMBERS="$(echo "$OPEN_JSON" | jq -r '.[].number | tostring' | paste -sd, - || true)"

PENDING='[]'
if [[ -n "$OPEN_NUMBERS" ]]; then
  PENDING="$(echo "$OPEN_JSON" | jq --arg handled "$HANDLED" '
    ($handled | if length == 0 then [] else split(",") end) as $handled_set |
    [.[] | select((.number | tostring) as $n | ($handled_set | index($n) | not))]
  ')"
fi

PENDING_COUNT="$(echo "$PENDING" | jq 'length')"
OPEN_COUNT="$(echo "$OPEN_JSON" | jq 'length')"

jq --arg now "$NOW" --argjson open_numbers "$(echo "$OPEN_JSON" | jq '[.[].number]')" '
  .last_check_at = $now |
  .last_seen_issue_numbers = $open_numbers
' "$STATE_FILE" >"$STATE_FILE.tmp" && mv "$STATE_FILE.tmp" "$STATE_FILE"

{
  printf '%s cycle open_issues=%s pending_issues=%s pending=%s\n' \
    "$NOW" "$OPEN_COUNT" "$PENDING_COUNT" "$(echo "$PENDING" | jq -c '.[].number')"
} >>"$LOG_FILE"

echo "$PENDING"
