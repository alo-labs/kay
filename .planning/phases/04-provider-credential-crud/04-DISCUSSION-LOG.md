# Phase 4: Provider Credential CRUD - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 4-provider-credential-crud
**Areas discussed:** Provider CRUD surface, Credential entry, Plugin seam, Scope boundaries

---

## Provider CRUD surface

| Option | Description | Selected |
|--------|-------------|----------|
| `/provider` as canonical command | Add a dedicated provider CRUD command in the TUI | ✓ |
| Extend `/model` for provider CRUD | Overload the model picker with provider management | |
| Keep CLI-only provider setup | Leave provider management outside the TUI | |

**User's choice:** `/provider` as the canonical command
**Notes:** The user explicitly wants CRUD for new providers and wants Kay to avoid config-file editing.

---

## Credential entry

| Option | Description | Selected |
|--------|-------------|----------|
| CLI `--api-key <KEY>` | Accept the API key directly as a CLI argument | ✓ |
| Stdin-only flow | Keep using piped stdin for key entry only | |
| Config-file editing | Require manual edits for onboarding | |

**User's choice:** CLI API-key argument
**Notes:** Keep stdin support for compatibility where it already exists, but make direct CLI entry available.

---

## Plugin seam

| Option | Description | Selected |
|--------|-------------|----------|
| Orthogonal provider/model plugins | Keep provider registration separate from model compatibility profiles | ✓ |
| Provider-specific model hacks | Encode compatibility directly inside each provider | |
| Single global model table | Flatten provider/model differences into one registry | |

**User's choice:** Orthogonal provider/model plugins
**Notes:** The user explicitly wants reusable model plugins that can be shared across multiple models and kept separate from provider plugins.

---

## Scope boundaries

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 4 only | Provider CRUD + credential entry only | ✓ |
| Include model filtering too | Pull provider-aware `/model` into this phase | |
| Expand provider families | Add new provider families beyond OpenCode Go, MiniMax, OpenAI now | |

**User's choice:** Phase 4 only
**Notes:** Provider-aware `/model` filtering is deferred to Phase 5; additional provider families are deferred to later milestones.

---

## the agent's Discretion

- Exact provider-pane layout and copy.
- Confirmation phrasing for delete/update operations.
- Whether to reuse the existing account pane or introduce a dedicated provider pane.

## Deferred Ideas

- Provider-aware `/model` filtering and provider-specific model lists.
- Additional provider families beyond OpenCode Go, MiniMax, and OpenAI.
- A broader settings UX redesign for auth flows.

---

*Phase: 04-provider-credential-crud*
*Discussion log generated: 2026-05-12*
