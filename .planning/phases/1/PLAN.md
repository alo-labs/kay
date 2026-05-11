# Phase 1 Plan: OpenCode Provider Foundation

## Goal
Add the OpenCode Go provider foundation by building the provider/model interfaces and the minimal internal wiring needed to support them. Keep docs, live validation, and release work out of this phase.

## Requirements Covered
- `PROV-01`
- `PROV-02`
- `MODEL-01`

## Phase Boundaries
- In scope: provider metadata, model-family normalization, core config/constructor wiring, CLI auth entrypoints, and focused unit tests
- Out of scope: docs, live API-key validation, release/versioning, changelog work, and broad architecture rewrites

## Task Breakdown
1. Define the provider metadata and model-family seam first.
   - Files: `code-rs/core/src/model_provider_info.rs`, `code-rs/core/src/model_family.rs`
   - Change: Add the narrow interfaces for provider identity, provider-prefixed slug normalization, and any per-model request-shaping hook needed by later wiring.
   - Verify: A provider-prefixed slug maps through the same family behavior as the bare model when the provider metadata says it should.

2. Thread the new provider through core config and exports.
   - Files: `code-rs/core/src/config.rs`, `code-rs/core/src/lib.rs`
   - Change: Register `opencode-go` in the built-in provider path and expose the constructor or entry point needed by the rest of the app.
   - Verify: Built-in provider selection resolves without custom config and existing providers remain untouched.

3. Wire the CLI auth entrypoint to the new provider.
   - Files: `code-rs/cli/src/login.rs`, `code-rs/cli/src/main.rs`
   - Change: Add the OpenCode Go login path and CLI routing necessary to reach it, without adding docs or release text.
   - Verify: The CLI surfaces the new auth flow and still routes the existing providers as before.

4. Add focused unit coverage for provider selection and slug handling.
   - Files: `code-rs/core/tests/opencode_go_provider.rs`
   - Change: Cover the built-in provider selection, namespace handling, and representative `opencode-go/<model>` behavior.
   - Verify: The new core tests pass without live credentials.

## Verification Steps
1. Run `./build-fast.sh` from the repo root and fix every error and warning.
2. Run the focused unit tests for the touched core and CLI crates.
3. Confirm the phase is still foundation-only, with docs/live/release deferred to later phases.

## Threat Model
- API keys must remain local and out of git history.
- Provider/model selection must preserve existing OpenAI and MiniMax behavior.
- Phase 1 must not absorb docs, live validation, or release work.

## Rollback
- Revert the provider metadata, config, CLI auth, and test changes as a single unit if verification fails.
