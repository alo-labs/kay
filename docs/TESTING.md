# Testing

This file is the canonical home for test strategy and verification guidance.

## Required Gate

- Run `./build-fast.sh` from the repo root before finishing work
- Treat warnings as failures and fix them before completion
- For provider-registration and resume-compatibility changes, add a focused regression check for the built-in provider path before closing

## Recommended Checks

- Focused Rust tests when touching specific crates
- End-to-end smoke checks for user-visible CLI flows
- Doc and workflow verification when SB or GSD scaffolding changes
- Use the dedicated `opencode-go` config, login, and live E2E coverage when touching provider ids, login help text, or namespaced model handling

## Notes

- Do not use `rustfmt` as part of init or completion unless explicitly requested
- If a test or build step fails, capture the concrete failing command and fix the underlying issue rather than papering over it
- When a task only refreshes planning or docs scaffolding, still refresh `docs/task-doc-checklist.json` in the same session before closing out
- Provider compatibility changes should be verified against both config loading and resumed-session replay, not just the happy-path CLI invocation
