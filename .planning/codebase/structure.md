# Structure

## Summary

The repo is organized as a brownfield CLI workspace with Rust sources, root-level docs, and workflow metadata.

## Top-Level Layout

- `code-rs/` - Rust workspace and binaries
- `docs/` - user-facing docs plus Silver Bullet governance docs
- `.planning/` - GSD project state, roadmap, and transient phase artifacts
- `.claude/` - Claude / Silver Bullet local settings
- `.github/workflows/` - CI and release automation
- `package.json` - root Node.js metadata and helper scripts

## Notes

- Preserve existing documentation structure when adding new guidance
- Do not assume a single source tree layout across the entire repo
