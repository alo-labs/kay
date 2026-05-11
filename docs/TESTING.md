# Testing

This file is the canonical home for test strategy and verification guidance.

## Required Gate

- Run `./build-fast.sh` from the repo root before finishing work
- Treat warnings as failures and fix them before completion
- For provider-registration and resume-compatibility changes, add a focused regression check for the built-in provider path before closing

## Recommended Checks

- Focused Rust tests when touching specific crates
- End-to-end smoke checks for user-visible CLI flows
- Doc and workflow verification when SB or GSD scaffolding changes
- Use the dedicated `opencode-go` config, login, and live E2E coverage when touching provider ids, login help text, or namespaced model handling

## Provider / Model Strategy

When a provider needs to support one or more prioritized models, treat validation as a three-layer contract:

1. **Configuration layer**
   - Verify the provider is registered as a built-in provider.
   - Verify config loading accepts the provider id without extra custom wiring.
   - Verify every prioritized model slug can be selected through `model_provider=<provider>` plus `model=<provider>/<model-id>`.

2. **Wire layer**
   - Use a mock server to confirm provider-prefixed model slugs are normalized only where the target wire format expects a provider-local slug.
   - Keep one canonical model test per provider for request-body inspection, but make the assertion generic across the provider-local naming rule rather than a single model quirk.
   - When a model family needs a compatibility profile, assert the profile itself at the model-family layer before checking the wire payload.

3. **Live capability layer**
   - Run a live login + exec smoke against the provider key.
   - Exercise the full prioritized model matrix with the least opinionated prompt the whole set can satisfy.
   - Treat CLI warnings, reroute notices, and provider metadata fallbacks as noise unless they prevent the final assistant response from being parsed and validated.

For OpenCode Go specifically, the live matrix should prove:
- the provider key works for login
- the CLI can select each prioritized model
- each model can complete a simple exact-response prompt successfully
- focused model-specific tests cover the stricter JSON/role edge cases
- shared compatibility profiles like the qwen/deepseek collapsed-role profile are asserted at the model-family and payload layers

The live matrix is a capability smoke, not a ranking benchmark. If a provider adapts model behavior internally, prefer assertion styles that validate the CLI contract and final response shape instead of a brittle exact wording check.

## Release Smoke Checklist

This remains smoke testing, not exhaustive certification. For every provider/model combo that is meant to behave like a supported Codex path, run the smallest realistic set that still exercises both developer and non-developer traffic:

1. **Non-dev completion**
   - A user-only prompt that asks for a short exact answer.
   - Catches basic routing, auth, and final-answer rendering issues.

2. **Dev-instructed completion**
   - A developer message plus a user prompt.
   - Catches role handling, instruction stacking, and provider/model compatibility profiles.

3. **Structured output**
   - A JSON-only or `json_schema` response request.
   - Catches serialization, quoting, and parser/renderer edge cases.

4. **Markdown / rendering**
   - A multiline response that includes a list, code fence, or other formatting-sensitive content.
   - Catches whitespace, fencing, wrapping, and terminal rendering regressions.

5. **Tool round-trip**
   - A single tool-use or tool-return turn for providers/models that advertise tool support.
   - Catches request/response shape mismatches that only show up once tools are in play.

6. **Carry-forward turn**
   - One follow-up turn that depends on earlier context.
   - Catches history replay, role retention, and multi-turn continuity issues.

Acceptance target:

- The observable contract should match the equivalent official OpenAI model path for Codex core flows: same prompt classes accepted, same developer/non-developer role handling, same structured-output and tool semantics, and the same rendered final-answer shape.
- This is intentionally smoke-level. Use it to catch regressions early, not to claim full behavioral parity or benchmark equivalence.
- If a provider/model family needs a compatibility profile, keep the profile-specific assertions in unit or wire tests and keep the release smoke matrix minimal and realistic.

## Notes

- Do not use `rustfmt` as part of init or completion unless explicitly requested
- If a test or build step fails, capture the concrete failing command and fix the underlying issue rather than papering over it
- When a task only refreshes planning or docs scaffolding, still refresh `docs/task-doc-checklist.json` in the same session before closing out
- Provider compatibility changes should be verified against both config loading and resumed-session replay, not just the happy-path CLI invocation
