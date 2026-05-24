# Testing

This file is the canonical home for test strategy and verification guidance.

## Required Gate

- Run `./build-fast.sh` from the repo root before finishing code, build, script, packaging, workflow, dependency, or generated-artifact changes; it now performs the mandatory transient-artifact cleanup step on success
- Documentation-only changes are exempt from `./build-fast.sh`; validate them with targeted checks such as `git diff --check`, structured-file syntax checks for edited JSON/YAML, and Markdown review where useful
- Treat warnings as failures and fix them before completion
- For provider-registration and resume-compatibility changes, add a focused regression check for the built-in provider path before closing
- For runtime model observability changes, add focused TUI coverage that proves `/model` shows the latest response model and warns when the response model differs from the request model
- For upstream merge strategy or release-monitoring documentation changes, validate the edited Markdown plus structured checklist files directly; only run the full build gate when code, scripts, packaging, workflows, dependencies, or generated artifacts changed

## Reliability Grade

Kay's reliability target is parity with the upstream OSS Codex release bar, plus the Kay-specific surfaces we own locally.

The practical scorecard is:

1. `./build-fast.sh` passes with no warnings.
2. Focused regressions cover the code path that changed.
3. Live end-to-end validation covers the supported OCG note-app workflow when provider, model, or orchestration behavior changes.
4. Transcript JSONL remains readable for triage and post-incident review.
5. Runtime model routing can be verified from Kay-owned metadata (`turn_context`, `/status`, and `/model`) rather than model self-reporting.
6. Upstream drift is triaged and merged deliberately instead of being allowed to accumulate silently.

Upstream high-risk changes are not a release blocker by themselves. They are a merge-review obligation: classify them, record the decision, and either adopt or defer them with an owner.

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

## Real Project Validation

Kay also has a real-world note-app target at `~/projects/test-notes-app`
(`alo-exp/test-notes-app` on GitHub). Use it to validate substantial model work
before releases, not just final-answer smoke:

1. Bootstrap the target checkout and isolated Kay home:
   - `scripts/bootstrap-test-notes-app.sh`
2. Run the live Kay harness against the supported OpenCode Go model set:
   - `cargo test -p code-cli --test test_notes_app_live_e2e -- --nocapture`
3. Review the copied session transcript JSONL in the notes-app repo when
   investigating model behavior or UX frictions.

This target is intended to expose real edit/review behavior, transcript
provenance, and UX rough edges that simple prompt smokes cannot catch.

For onboarding/provider regressions, run the opt-in live smoke that exercises
the first-start provider flow without `kay login`:

```bash
KAY_ONBOARDING_LIVE_SMOKE=1 \
OPENCODE_GO_LIVE_API_KEY=... \
cargo test -p code-cli --test onboarding_provider_notes_app_live_smoke -- --nocapture
```

That test starts Kay with an empty `KAY_HOME`, configures OpenCode Go through
onboarding's provider manager, then iterates the curated OpenCode Go release
matrix: MiMo V2.5, MiMo V2.5 Pro, DeepSeek V4 Flash at `xhigh` reasoning, and
MiniMax M2.7 through OpenCode Go. For each model it switches through the TUI
`/model` command, applies any model-specific `/reasoning` setting, asserts the
TUI header shows the selected model, runs one live notes-app inspection turn,
and verifies Kay's session log recorded the expected `model_provider_id` and
`model` in the outbound session configuration. OpenAI API-key live testing is
intentionally excluded from this smoke for now; OpenAI coverage should use a
separate OAuth-mode smoke. Direct MiniMax.io provider testing is also excluded
from the release gate; MiniMax M2.7 coverage runs through OpenCode Go. The model
response is only used to prove live work completed; model/provider identity is
verified from Kay-side metadata, not from model self-reporting.

This smoke is a required pre-release gate. `./pre-release.sh` runs it after the
dev-fast build, CLI smokes, and workspace nextest suite. The release gate uses
only the curated OpenCode Go matrix above. It accepts credentials from
`OPENCODE_GO_LIVE_API_KEY`, falls back to the normal `OPENCODE_GO_API_KEY` env
var, and finally falls back to `provider_credentials.opencode-go.api_key` in
`$KAY_HOME/auth.json`.

For focused debugging, set `KAY_ONBOARDING_LIVE_SMOKE_MODEL_FILTER` to a comma
separated subset of exact provider ids or model ids, such as
`opencode-go/minimax-m2.7` or `opencode-go/deepseek-v4-flash`. The default
per-model live turn timeout is 15 minutes because some third-party providers
can be slow after tool-use history accumulates.

## Upstream Sync Policy

The upstream merge workflow is the place to triage OSS Codex drift. Keep the following separation intact:

- release gating answers "is Kay safe to ship?"
- upstream triage answers "what should we pull forward next?"

That means high-risk upstream changes should be logged, classified, and resolved in the merge workflow, but they should not stop a release unless they also introduce a Kay-side regression or a missing required validation.

After a release workflow succeeds on `main`, release verification also includes
checking the Google Chat announcement job or running the manual announcement
workflow for the released tag before calling the release complete.

## Notes

- Do not use `rustfmt` as part of init or completion unless explicitly requested
- If a test or build step fails, capture the concrete failing command and fix the underlying issue rather than papering over it
- When a task only refreshes planning or docs scaffolding, still refresh `docs/task-doc-checklist.json` in the same session before closing out
- Provider compatibility changes should be verified against both config loading and resumed-session replay, not just the happy-path CLI invocation
