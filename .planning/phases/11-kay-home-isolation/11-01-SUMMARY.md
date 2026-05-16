# 11-01 Summary

## Outcome

Completed the KAY_HOME isolation foundation for Kay-owned writable state and
updated the live test harnesses so they can run with `KAY_HOME` as the sole
isolated root.

## What Changed

- `KAY_HOME` is now the canonical root in the config builder, config fallback
  loader, prompt discovery, review lock storage, debug logging, cloud-task log
  resolution, dotenv bootstrap, and auth/session-related helpers.
- Session and worktree storage now resolve under the Kay home tree, with the
  session registry and branch metadata written under `KAY_HOME/working/...`.
- Live provider and notes-app tests now isolate themselves with `KAY_HOME`
  instead of depending on caller-managed `HOME` or `KAY_HOME` redirection.
- VT100 snapshot coverage was updated to seed `KAY_HOME` and clear the legacy
  env aliases while keeping the compatibility cleanup in place.

## Verification

- `./build-fast.sh`

## Remaining Assumptions

- When `KAY_HOME` is unset, Kay uses its normal default home layout.
- The tests now exercise `KAY_HOME` directly and no longer rely on legacy
  home aliases.
