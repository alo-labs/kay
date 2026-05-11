# Architecture

This file captures the durable architecture view for the repo. The codebase is a brownfield Rust/Node.js monorepo, so the goal here is to explain the stable boundaries rather than redesign them.

## System Shape

- Rust workspace and CLI binaries live under `code-rs/`
- Root-level docs explain user-facing behavior and workflow guidance
- GSD and Silver Bullet metadata live outside the product code, under `.planning/` and `~/.claude/.silver-bullet/`

## Data Flow

1. User-facing commands enter through the CLI entrypoints
2. Project workflow decisions are captured in `.planning/`
3. Docs and workflow scaffolding document the stable operating model

## Current Notes

- Build validation is centered on `./build-fast.sh`
- Existing repo docs are preserved during init rather than being replaced
- Future architecture notes should be appended here when a real subsystem boundary changes
