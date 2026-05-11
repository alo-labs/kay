# Architecture

## Summary

The product architecture centers on a CLI surface backed by a Rust workspace, with workflow and docs layers wrapped around it for maintainability and release safety.

## Observed Layers

1. CLI / command entrypoints
2. Rust workspace implementation
3. Docs and workflow guidance
4. GitHub Actions and release automation
5. Local agent / plugin state under the user home directory

## Data Flow

- Commands enter through the CLI
- Build / release checks operate on the Rust workspace
- Workflow docs and planning artifacts capture higher-level decisions

## Notes

- Existing repo instructions live in `CLAUDE.md`
- Silver Bullet instructions live in `silver-bullet.md`
