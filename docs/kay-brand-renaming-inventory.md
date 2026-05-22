# Kay Rename Inventory

This is a living snapshot of the remaining brand-token work.

It is not a raw grep dump. The goal is to classify every remaining `codex` or
product-branded `code` reference so the rename can be done deliberately.

Phase 08 has already rewritten the highest-priority docs and copy surfaces:
`docs/getting-started.md`, `docs/homebrew.md`, `docs/faq.md`, `docs/install.md`,
`docs/exec.md`, `docs/execpolicy.md`, `docs/skills.md`,
`docs/integration-zed.md`, `docs/ARCHITECTURE.md`, and
`docs/tui-alternate-screen.md`. The remaining docs in this bucket still need
later sweeps where the compatibility boundary is more explicit or the path
reference is still shared with the current codebase.

The current release-style rename sweep completed
`docs/example-config.md`, `kay-rs/cli/src/main.rs`,
`kay-rs/core/src/agent_tool.rs`,
`kay-rs/tui/src/bottom_pane/prompts_settings_view.rs`,
`kay-rs/tui/src/bottom_pane/skills_settings_view.rs`,
`kay-rs/tui/src/chatwidget.rs`, `kay-rs/tui/src/lib.rs`,
`kay-rs/tui/src/model_migration.rs`, and
`kay-rs/login/src/assets/success.html`.

## Legend

- `rename-now` - safe to rewrite mechanically because it is first-party text
  with no compatibility consequence
- `rename-with-compat` - requires aliases, shims, or staged rollout
- `retain-for-compat` - external contract, generated artifact, or upstream
  mirror label that should stay until a versioned migration exists
- `historical` - provenance note that may keep the old name for context

## Current Inventory

| Bucket | Examples | Status | Notes |
| --- | --- | --- | --- |
| User-facing docs and help text | `docs/getting-started.md`, `docs/faq.md`, `docs/config.md`, `docs/homebrew.md`, `docs/agents.md`, `docs/exec.md`, `docs/execpolicy.md`, `docs/advanced.md`, `docs/authentication.md`, `docs/prompts.md`, `docs/settings.md`, `docs/tui-stream-chunking-validation.md`, `docs/integration-zed.md`, `docs/skills.md`, `docs/tui-alternate-screen.md`, `kay-rs/README.md` | rename-now | Replace product-branded `Codex` and `code` references with `kay` where the text is describing the Kay product. Leave ordinary prose like "source code" alone. Phase 10 completed the remaining config, agent, auth, prompt, settings, and validation-doc sweep; compatibility names stay only where the boundary requires them. |
| Docs navigation and governance | `docs/index.md`, `docs/ARCHITECTURE.md`, `docs/TESTING.md` | rename-now | Keep the docs landing page Kay-first and link the migration policy and inventory from there. |
| Repo plumbing and release scripts | `package.json`, `build-fast.sh`, `pre-release.sh`, `scripts/ci-tests.sh`, `scripts/generate-homebrew-formula.sh`, `scripts/post-release-cleanup.sh`, `scripts/test-post-release-cleanup.sh`, `scripts/check-kay-path-deps.sh`, `scripts/check-codex-path-deps.sh`, `scripts/start-kay-exec.sh`, `scripts/start-codex-exec.sh`, `kay-rs/justfile`, `kay-rs/protocol-ts/generate-ts` | rename-with-compat | Update command names, log text, and path assumptions carefully so current automation still works while the transition is in progress. `check-codex-path-deps.sh` and `start-codex-exec.sh` remain compatibility wrappers while the Kay-first entrypoints are used by current tooling. |
| Workspace path and binary roots | `kay-rs/`, legacy `code-rs` selector aliases, `kay-rs/bin/code`, `kay-rs/target`, `~/.code`, `KAY_HOME` | rename-with-compat | High-risk path migration. The active filesystem path is `kay-rs/`; keep command/selector compatibility where needed, but do not retain a `code-rs` workspace symlink. |
| Crate and package identifiers | `code-core`, `code-cli`, `code-login`, `code-version`, and the rest of the `code-*` crates | rename-with-compat | Rename crate and package names only when downstream compatibility is accounted for. Preserve published identifiers until the migration story is explicit. |
| Model slugs, telemetry prefixes, and built-in agent IDs | `code-gpt-5.4`, `code-gpt-5.3-codex`, `code-gpt-5.1-codex-mini`, `cloud-gpt-5.1-codex-max`, `codex.*`, `codex_tui::streaming::commit_tick`, `codex.preview/1` | retain-for-compat | These identifiers are external contracts, dashboard keys, or telemetry namespaces. Keep them until the downstream ecosystem has a versioned rename path. |
| Kay home roots and log/path labels | `KAY_HOME`, `CODE_BINARY_PATH`, `CODE_ENABLE_CLOUD_AGENT_MODEL`, `codex-tui.log`, replay paths under `~/.code/debug_logs/` | rename-with-compat | `KAY_HOME` is the supported root for Kay-owned writable state; the remaining labels are separate compatibility boundaries. |
| Runtime docs, CLI help, and setup text | `kay-rs/core/src/agent_tool.rs`, `kay-rs/core/src/config_types.rs`, `kay-rs/cli/src/main.rs`, `kay-rs/tui/src/chatwidget.rs`, `kay-rs/login/src/assets/success.html` | rename-with-compat | These strings appear in logs, prompts, setup, or error messages and need human review if the name changes affect user comprehension. Keep the user-facing wording Kay-first while retaining compatibility aliases where they are still part of the shipped interface. |
| Protocol and schema artifacts | `codex_app_server_protocol`, `CodexErrorInfo`, `codex_app_server_protocol.schemas.json`, generated TS files under `kay-rs/app-server-protocol/schema/typescript/` | retain-for-compat | These names are part of generated or external contracts. Rename only with versioning or compatibility aliases. |
| Upstream sync and comparison tooling | `scripts/upstream-merge/*.sh`, `kay-rs/MIGRATION_GUIDE.md`, `kay-rs/MIGRATION_QUICK_REFERENCE.md` | retain-for-compat | Keep the upstream project name when it identifies the upstream baseline. Translate fork-owned narrative to Kay. |
| Historical migration notes | `kay-rs/MIGRATION_GUIDE.md`, `kay-rs/MIGRATION_QUICK_REFERENCE.md` | historical | These files may mention old names for provenance. They can be rewritten later once the migration is complete. |

## Sweep Order

1. User-facing docs and UI strings.
2. Repo scripts and top-level docs.
3. Kay-owned runtime paths and environment variables.
4. Crate, binary, and package identifiers.
5. Protocol, schema, and generated contract names.
6. Historical migration notes.

## Review Rule

Do not land a bulk rename where a token also changes behavior, on-disk layout,
wire format, or package identity without a manual review entry in this
inventory.

Keep this file current as the migration progresses.
