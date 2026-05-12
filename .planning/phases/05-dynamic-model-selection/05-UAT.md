---
status: complete
phase: 05-dynamic-model-selection
source: [05-01-SUMMARY.md, 05-02-SUMMARY.md]
started: 2026-05-12T14:21:45Z
updated: 2026-05-12T14:21:45Z
---

## Current Test

[testing complete]

## Tests

### 1. Credentialed `/model` picker shows unlocked provider buckets
expected: |
  With credentials configured for OpenCode Go, MiniMax, and OpenAI, the live `/model` picker shows the unlocked provider buckets in the fixed order OpenCode Go, MiniMax, OpenAI. The visible rows are provider-gated rather than showing every preset unconditionally.
result: pass

### 2. Picker preserves reasoning-effort rows inside each provider bucket
expected: |
  The provider-grouped picker still renders the reasoning-effort rows and model detail text for each visible provider bucket, so the user can choose the same model/effect combinations as before inside each unlocked group.
result: pass

### 3. Empty credentials state shows onboarding hint
expected: |
  When no provider credentials are configured, the live `/model` picker shows a clear onboarding or empty-state hint instead of a blank or misleading model list.
result: pass

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none yet]
