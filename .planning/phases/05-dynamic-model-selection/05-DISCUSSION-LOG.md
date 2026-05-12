# Phase 5 Discussion Log

## Discussed and Locked

- `/model` should only show models for providers that currently have usable credentials.
- Configured providers are determined by the existing auth resolution paths, including stored provider credentials and the OpenAI auth path.
- The visible provider order must stay OpenCode Go, MiniMax, OpenAI.
- OpenCode Go uses the currently supported OpenCode Go model matrix.
- MiniMax is limited to `MiniMax-M2.7`.
- OpenAI uses the upstream OSS Codex-supported OpenAI model set.
- Provider plugins and model compatibility profiles remain orthogonal.
- The model list should come from a reusable helper so UI and future surfaces can share the same visibility rules.
- The picker should keep the existing reasoning-effort behavior and only change which models are visible.

## Deferred

- Any provider families beyond OpenCode Go, MiniMax, and OpenAI.
- Large-scale redesign of the picker chrome.
- Re-opening provider CRUD behavior, which was completed in Phase 4.

