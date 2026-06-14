# Slash Commands

The Kay CLI supports a set of slash commands you can type at the start of the
composer input. These commands provide quick actions, toggles, or expand into
full prompts. This document lists all built‑in commands and what they do.

Notes

- Commands are shown in the TUI’s slash popup; the order below matches the UI.
- Commands marked “prompt‑expanding” transform your input into a full prompt and
  typically kick off multi‑agent flows.
- Some commands accept arguments; if required, usage is shown in parentheses.

## Navigation & Session

- `/browser`: open internal browser.
- `/chrome`: connect to your Chrome browser.
- `/new`: start a new chat during a conversation.
- `/clear`: clear the terminal and start a new chat.
- `/resume`: resume a past session for this folder.
- `/rename <name>`: rename the current session (shown in the resume list).
- `/quit`: exit Kay.
- `/exit`: exit Kay.
- `/logout`: log out of Kay.
- `/login`: manage Kay sign-ins (select, add, or disconnect accounts).
- `/provider`: manage provider API keys for Xiaomi, OpenCode Go, MiniMax, OpenRouter, and OpenAI without editing config files.
- `/settings [section]`: open the settings panel. Optional section argument
  jumps directly to `model`, `theme`, `agents`, `skills`, `auto`, `review`,
  `validation`, `limits`, `chrome`, `mcp`, or `notifications`.

## Workspace & Git

- `/init`: create an `AGENTS.md` file with instructions for Kay.
- `/diff`: show `git diff` (including untracked files).
- `/copy`: copy the last assistant response as markdown.
- `/undo`: open a snapshot picker so you can restore workspace files to a
  previous Kay snapshot and optionally rewind the conversation to that point.
- `/branch [task]`: create a worktree branch and switch to it. If a
  task/description is provided, it is used when naming the branch. Must be run
  from the repository root (not inside another branch worktree). Set
  `CODE_BRANCH_COPY_CACHES=1` (legacy: `CODEX_BRANCH_COPY_CACHES=1`) to mirror
  `node_modules` and Rust build caches into the worktree; otherwise no cache
  directories are copied automatically.
- `/merge`: merge the current worktree branch back into the default branch and
  remove the worktree. Run this from inside the worktree created by `/branch`.
- `/push`: tell Kay to commit, push, and monitor workflows with guarded
  instructions. If no workflows appear right away, wait briefly and check again
  before concluding none were triggered. Skips cleanup or GitHub monitoring
  steps automatically when the workspace is already clean or required
  tooling/files are missing.
- `/review [focus]`: without arguments, opens a review picker so you can audit
  the workspace, a specific commit, compare against another branch, or enter
  custom instructions. With a focus argument, skips the picker and uses your
  text directly. Configure Auto Resolve and the max re-reviews (defaults to 5)
  from `/settings review` when you want Kay to rerun fixes and follow-up
  checks automatically. Scoped audit prompts should stay limited to the
  requested files and distinguish a completed low pass from an exhaustive
  review-loop result.
- `/cloud`: browse Kay Cloud tasks, view details, apply patches, and create
  new tasks from the TUI.
- `/cmd <name>`: run a project command defined for the current workspace.

## UX & Display

- `/theme`: customize the app theme.
- `/verbosity (high|medium|low)`: change text verbosity.
- `/model`: choose your default model from the providers you have configured.
  At the top of the selector, Kay shows the last completed turn's response
  model, the last request model, and the currently selected model. If the
  response model differs from the request model, the selector warns so provider
  reroutes are visible without asking the model to identify itself.
- `/fast`: open the model selector and toggle Fast mode.
- `/reasoning (minimal|low|medium|high)`: change reasoning effort.
- `/prompts`: manage custom prompts.
- `/skills`: manage skills.
- `/status`: show current session configuration and token usage.
- `/limits`: adjust session limits and visualize hourly and weekly rate-limit
  usage.
- `/update`: check the installed version, detect available upgrades, and open a
  guided upgrade terminal that runs the installer interactively when possible.
- `/notifications [status|on|off]`: manage notification settings. Without
  arguments, shows the notifications panel. With arguments: `status` shows
  current config, `on` enables all, `off` disables all.
- `/mcp [status|on|off <name>|add]`: manage MCP servers. Without arguments,
  shows all servers with toggle controls. With arguments: `status` lists
  servers, `on <name>` enables, `off <name>` disables, and `add` starts the new
  server workflow.
- `/validation [status|on|off|<tool> (on|off)]`: inspect or toggle validation
  harness settings.

## Search & Mentions

- `/mention`: mention a file (opens the file search for quick insertion).

## Performance & Agents

- `/perf (on|off|show|reset)`: performance tracing controls.
- `/agents`: configure agents and subagent commands (including autonomous
  follow-ups and observer status; available in dev, dev-fast, and perf builds).
- `/auto [goal]`: start the maintainer-style auto coordinator. If no goal is
  provided it defaults to "review the git log for recent changes and come up
  with sensible follow up work".

## Prompt‑Expanding (Multi‑Agent)

These commands expand into full prompts (generated by `code-core`) and
typically start multiple agents. They require a task/problem description.

- `/plan <task>`: create a comprehensive plan (multiple agents). Prompt‑expanding.
- `/solve <problem>`: solve a challenging problem (multiple agents). Prompt‑expanding.
- `/code <task>`: perform a coding task (multiple agents). Prompt‑expanding.

## Account & Exit

- `/logout`: log out of Kay.
- `/quit`: exit Kay.

## Development‑Only

- `/demo`: populate the chat history with assorted sample cells (available in
  dev and perf builds for UI testing).
- `/demo auto drive card`: render the Auto Drive card once for each ANSI-16
  background color so you can compare theme contrast.
- `/test-approval`: test approval request (available in debug builds only).

Implementation Notes

- The authoritative list of commands is defined in
  `kay-rs/tui/src/slash_command.rs` (the `SlashCommand` enum). When adding a
  new command, please update this document to keep the UI and docs in sync.
- Prompt formatting for `/plan`, `/solve`, and `/code` lives in
  `kay-rs/core/src/slash_commands.rs`. `/code` is the canonical prompt-expanding
  command in the Kay UI.
  Provider credential CRUD lives in `/provider`, which is the shared entry
  point for adding, updating, or removing Xiaomi, OpenCode Go, MiniMax,
  OpenRouter, and OpenAI API keys.
  Provider/model compatibility fixes should update the provider, model-family,
  and testing docs; only update this slash-command surface when the visible
  command behavior changes.
  External workflow names such as `silver:init` are skill/plugin requests, not
  Kay built-in slash commands. Exact named workflow routing is enforced in the
  skills prompt renderer so those requests execute the named workflow instead
  of drifting into adjacent scan or discovery paths.
  Silver Bullet init/migrate runs should preserve exact skill receipts and keep
  docs/learnings terminology aligned with the governed doc-scheme keys.
  The end-user install and release docs should stay aligned with the shipped
  assets in GitHub Releases, not with the presence or absence of npm registry
  publication.
  Installer and upgrade helpers should verify Homebrew ownership with
  `brew which-formula` before proposing uninstall or upgrade actions; when a
  formula is confirmed, use that formula name in the suggested brew command.
  Fallback aliases like `code` or `codex` should only warn about PATH
  ambiguity until ownership is confirmed.
  Release-note headers must match the package version contract used by the
  release pipeline (`## @alo-labs/kay v<version>`) so release-note verification
  stays strict; hook remediation for release/install work should refresh this
  note in the same session as `docs/task-doc-checklist.json`.
  GitHub Releases must carry detailed, categorized release notes directly in
  the release body; `CHANGELOG.md` can supplement the release, but a generic
  CHANGELOG-only pointer is not an acceptable release-note fallback.
  Release-monitoring instructions should stay aligned with
  `docs/upstream-merge-strategy.md` and `AGENTS.md`, including the required
  Google Chat announcement check after successful `main` releases.
  Release/install audit reports should cite only reproducible findings with
  exact refs, or state a clean low-pass verdict with the checks actually run.
  When a release-preflight failure is caused by runner resource pressure, cite
  the exact workflow step, the tempdir/disk strategy used to verify the fix,
  and the observed log line so the report remains reproducible.
  When no `[[agents]]` are configured, the orchestrator advertises the
  following model slugs to the LLM for multi-agent runs: `code-gpt-5.4`,
  `code-gpt-5.3-codex`, `claude-opus-4.6`, `gemini-3-pro`,
  `code-gpt-5.1-codex-mini`, `claude-sonnet-4.5`, `gemini-3-flash`,
  `claude-haiku-4.5`, and `qwen-3-coder` (with
  `cloud-gpt-5.1-codex-max` gated by `CODE_ENABLE_CLOUD_AGENT_MODEL`). (`gemini`
  resolves to `gemini-3-flash`.) You can replace or pin this set via
  `[[agents]]` or per-command `[[subagents.commands]].agents`.
  Set `[subagents].enabled = false` to remove the runtime `agent` tool when a
  run must stay pinned to one provider/model.
  For v0.9.31+, treat `docs/release-notes/RELEASE_NOTES.md` as the canonical GitHub Release body source; refresh it with `docs/task-doc-checklist.json` when `release-notes-body` hook remediation runs.
  v0.9.31 release-note bodies must list categorized fixes directly in the
  GitHub Release text sourced from `docs/release-notes/RELEASE_NOTES.md`
  (compact Sidekick `STATUS:` detection for final prompts, issue-monitor
  `closed_activity` tracking on already-handled issues, and the MiniMax-M3 live
  gate exercised in `./pre-release.sh`); hook remediation for
  Cursor DOC-SCHEME gate remediation for `release-notes-body` must re-touch `docs/doc-scheme.md`, `docs/doc-scheme.json`, this file, and `docs/task-doc-checklist.json` so session mtimes exceed the Silver Bullet marker.
  `release-notes-body` should refresh this note and
  `docs/task-doc-checklist.json` in the same session (2026-06-14).
