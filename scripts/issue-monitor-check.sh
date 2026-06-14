#!/usr/bin/env bash
# Detect open GitHub issues and post-close comment activity for the Kay issue monitor.
set -euo pipefail

REPO="${KAY_ISSUE_MONITOR_REPO:-alo-labs/kay}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STATE_FILE="$ROOT/.kay/issue-monitor-state.json"
LOG_FILE="$ROOT/.kay/issue-monitor.log"
NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

mkdir -p "$ROOT/.kay"
if [[ ! -f "$STATE_FILE" ]]; then
  printf '%s\n' '{"last_check_at":null,"last_seen_issue_numbers":[],"handled_issue_numbers":[],"acked_closed_activity_numbers":[]}' >"$STATE_FILE"
fi

LAST_CHECK="$(jq -r '.last_check_at // "1970-01-01T00:00:00Z"' "$STATE_FILE")"
SINCE_DATE="$(printf '%s' "$LAST_CHECK" | cut -dT -f1)"

OPEN_JSON="$(gh issue list --repo "$REPO" --state open --limit 30 --json number,title,createdAt,labels,state,updatedAt)"
HANDLED="$(jq -r '.handled_issue_numbers // [] | map(tostring) | join(",")' "$STATE_FILE")"
ACKED_CLOSED="$(jq -r '.acked_closed_activity_numbers // [] | map(tostring) | join(",")' "$STATE_FILE")"
OPEN_NUMBERS="$(echo "$OPEN_JSON" | jq -r '.[].number | tostring' | paste -sd, - || true)"

PENDING='[]'
if [[ -n "$OPEN_NUMBERS" ]]; then
  PENDING="$(echo "$OPEN_JSON" | jq --arg handled "$HANDLED" '
    ($handled | if length == 0 then [] else split(",") end) as $handled_set |
    [.[] | select((.number | tostring) as $n | ($handled_set | index($n) | not))]
  ')"
fi

CLOSED_ACTIVITY='[]'
if [[ -n "$SINCE_DATE" ]]; then
  RECENT_CLOSED_JSON="$(gh issue list --repo "$REPO" --state closed --search "updated:>=$SINCE_DATE" --limit 30 --json number,title,labels,state,updatedAt 2>/dev/null || printf '%s' '[]')"
  CLOSED_ACTIVITY="$(echo "$RECENT_CLOSED_JSON" | jq --arg last "$LAST_CHECK" --arg acked "$ACKED_CLOSED" '
    ($acked | if length == 0 then [] else split(",") end) as $acked_set |
    [.[] | select(.updatedAt > $last and ((.number | tostring) as $n | ($acked_set | index($n) | not)))]
  ')"
fi

PENDING_COUNT="$(echo "$PENDING" | jq 'length')"
CLOSED_ACTIVITY_COUNT="$(echo "$CLOSED_ACTIVITY" | jq 'length')"
OPEN_COUNT="$(echo "$OPEN_JSON" | jq 'length')"

jq --arg now "$NOW" --argjson open_numbers "$(echo "$OPEN_JSON" | jq '[.[].number]')" '
  .last_check_at = $now |
  .last_seen_issue_numbers = $open_numbers
' "$STATE_FILE" >"$STATE_FILE.tmp" && mv "$STATE_FILE.tmp" "$STATE_FILE"

{
  printf '%s cycle open_issues=%s pending_issues=%s closed_activity=%s handled=%s acked_closed=%s pending=%s closed_activity_issues=%s\n' \
    "$NOW" "$OPEN_COUNT" "$PENDING_COUNT" "$CLOSED_ACTIVITY_COUNT" "$HANDLED" "$ACKED_CLOSED" \
    "$(echo "$PENDING" | jq -c '[.[].number]')" \
    "$(echo "$CLOSED_ACTIVITY" | jq -c '[.[].number]')"
} >>"$LOG_FILE"

jq -n \
  --argjson open_pending "$PENDING" \
  --argjson closed_activity "$CLOSED_ACTIVITY" \
  '{
    open_pending: $open_pending,
    closed_activity: $closed_activity,
    pending_count: ($open_pending | length),
    closed_activity_count: ($closed_activity | length)
  }'
