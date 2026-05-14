## @alo-labs/kay v0.9.3

This patch hardens Kay's MiniMax and autonomous runtime paths after the `kay`
binary rollout.

### Changes

- Core: repair invalid MiniMax tool-call argument strings before sending chat
  completion history back to the provider.
- Core: keep autonomous runs from finishing with a null final assistant message
  when the provider/runtime aborts before producing a response.
- Core: normalize single-string shell argv payloads and quoted absolute
  workdirs emitted by provider adapters.
- Runtime: hide user-input approval tools when approval policy is `never`, and
  add `[subagents].enabled = false` as a hard switch for disabling delegation.
- Docs: document the new runtime delegation isolation setting.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay
```

Compare: https://github.com/alo-labs/kay/compare/v0.9.2...v0.9.3

## @alo-labs/kay v0.6.100

This release adds OpenCode Go as a built-in provider, normalizes provider-prefixed model slugs across request paths, and broadens model-adaptation seams so provider-specific plugins can stay configuration-driven.

### Changes

- Core: add the built-in OpenCode Go provider, support `opencode-go/<model-id>` selection, and normalize provider-prefixed slugs before chat/completions, responses, and compaction requests are built.
- Tests: add request-body coverage proving OpenCode Go sends the bare model slug over chat completions, plus live coverage for the OpenCode Go model matrix.
- CLI/TUI: update login guidance for `OPENCODE_GO_API_KEY` and skip onboarding for non-OpenAI OpenCode Go sessions.
- Docs: document OpenCode Go provider configuration and examples.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay
```

Compare: https://github.com/alo-labs/kay/compare/v0.6.99...v0.6.100
