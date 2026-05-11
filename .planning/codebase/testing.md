# Testing

## Summary

Testing is centered on build verification and focused regressions for the Rust workspace and workflow surface.

## Current Checks

- `./build-fast.sh` from the repo root
- Focused Rust tests when changing specific crates
- Workflow / docs validation when init scaffolding changes

## Notes

- Treat warnings as failures
- Use the smallest targeted check that proves the fix
- If a test fails, capture the concrete failure and fix the root cause
