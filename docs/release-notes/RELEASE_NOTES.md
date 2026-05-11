## @just-every/code v0.6.100

This release adds OpenCode Go as a built-in provider, normalizes provider-prefixed model slugs across request paths, and broadens model-adaptation seams so provider-specific plugins can stay configuration-driven.

### Changes

- Core: add the built-in OpenCode Go provider, support `opencode-go/<model-id>` selection, and normalize provider-prefixed slugs before chat/completions, responses, and compaction requests are built.
- Tests: add request-body coverage proving OpenCode Go sends the bare model slug over chat completions, plus live coverage for the OpenCode Go model matrix.
- CLI/TUI: update login guidance for `OPENCODE_GO_API_KEY` and skip onboarding for non-OpenAI OpenCode Go sessions.
- Docs: document OpenCode Go provider configuration and examples.

### Install

```bash
npm install -g @just-every/code@latest
code
```

Compare: https://github.com/just-every/code/compare/v0.6.99...v0.6.100
