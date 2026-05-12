---
spec-version: 1
status: Draft
feature: Provider Credential CRUD
project: code-monorepo
created: 2026-05-12
last-updated: 2026-05-12
source-artifacts:
  - docs/config.md
  - docs/authentication.md
  - docs/slash-commands.md
  - docs/ARCHITECTURE.md
  - docs/TESTING.md
---

# Provider Credential CRUD

## Overview

Kay needs a first-class way for users to manage provider credentials without editing config files. The initial supported provider set is OpenCode Go, MiniMax, and OpenAI, in that order. This spec covers provider creation, inspection, update, and deletion, plus a direct CLI API-key entry path that works alongside the existing stdin-based onboarding flow.

This spec intentionally stops before provider-aware model filtering. Showing only the models enabled by configured credentials is Phase 5 work.

## User Stories

- As a user, I can open `/provider` and see the supported providers in the required order.
- As a user, I can add a provider API key from inside Kay without hand-editing config files.
- As a user, I can update or remove an existing provider credential from inside Kay.
- As a script or power user, I can pass a provider API key directly on the CLI.
- As an existing user, I can keep using the stdin-based key flow I already have today.

## UX Flows

1. User runs `/provider`.
2. Kay shows the supported provider list in this order: OpenCode Go, MiniMax, OpenAI.
3. User selects a provider and either adds a key, updates a key, or deletes the stored credential.
4. For delete, Kay asks for confirmation before removing the credential.
5. For non-interactive onboarding, the user can run the login command with a direct API-key argument and an optional provider id.
6. The existing stdin-based `--with-api-key` path remains available for compatibility.

## Acceptance Criteria

- `/provider` exposes create, read/list, update, and delete actions for supported providers.
- The provider list appears in the required order: OpenCode Go, MiniMax, OpenAI.
- Direct CLI API-key entry works without needing config-file edits.
- Existing stdin-based API-key entry still works.
- Existing OpenAI, MiniMax, and OpenCode Go auth behavior remains intact while provider CRUD is added.
- Provider credentials are persisted in the existing auth flow, not in a separate ad hoc store.

## Out of Scope

- Provider-aware `/model` filtering and provider-specific model lists.
- Additional provider families beyond OpenCode Go, MiniMax, and OpenAI.
- Replacing the existing provider auth behaviors instead of extending them.
- Broad model compatibility refactors that belong to the later model-selection phase.

## Assumptions

- [ASSUMPTION: `/provider` can reuse the existing modal/bottom-pane shell, with provider-aware refactoring where needed | Status: Accepted | Owner: Kay]
- [ASSUMPTION: The direct CLI key entry will use a `--api-key <KEY>` style argument while `--with-api-key` stays stdin-based for compatibility | Status: Accepted | Owner: Kay]
- [ASSUMPTION: Provider credentials will continue to persist in the current auth storage path rather than a separate credentials file | Status: Accepted | Owner: Kay]

## Open Questions

- None at the spec level. The remaining choices are implementation details for discuss/planning.

