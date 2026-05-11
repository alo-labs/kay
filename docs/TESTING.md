# Testing

This file is the canonical home for test strategy and verification guidance.

## Required Gate

- Run `./build-fast.sh` from the repo root before finishing work
- Treat warnings as failures and fix them before completion

## Recommended Checks

- Focused Rust tests when touching specific crates
- End-to-end smoke checks for user-visible CLI flows
- Doc and workflow verification when SB or GSD scaffolding changes

## Notes

- Do not use `rustfmt` as part of init or completion unless explicitly requested
- If a test or build step fails, capture the concrete failing command and fix the underlying issue rather than papering over it
