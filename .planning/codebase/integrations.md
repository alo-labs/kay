# Integrations

## Summary

This repo integrates with GitHub, GitHub Actions, local plugin caches, and the GSD / Silver Bullet workflow layers.

## External Surfaces

- GitHub remote: `https://github.com/alo-labs/kay.git`
- GitHub Actions workflows under `.github/workflows/`
- GSD runtime state and planning files under `.planning/`
- Claude / Silver Bullet user settings under `~/.claude/`

## Notes

- Hooks and settings changes should be applied idempotently
- Init should preserve already-installed local plugin caches
