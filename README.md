# Kay

Kay is a terminal coding agent for people who want a local, scriptable, multi-provider workflow without giving up the ergonomics of the original Codex CLI lineage.

It is a separate project with its own release line, provider surface, and UI decisions. Kay keeps the familiar agent workflow, but it is not just a rename of Codex and it is not just Every Code under a different badge.

## Why Kay exists separately

- **Codex** gave this project its core CLI and agent workflow heritage.
- **Every Code** contributed the multi-provider direction and the idea that provider/model support should be first-class rather than bolted on.
- **Kay** exists so those ideas can evolve on their own schedule, with their own UX and release cadence, while still staying compatible with the upstream ecosystem where it makes sense.

## What Kay does well

- Runs as a local coding agent in your terminal.
- Coordinates multi-step tasks with Auto Drive.
- Supports browser-driven workflows through internal browser mode or CDP/Chrome.
- Gives you multi-agent commands such as `/plan`, `/kay`, `/solve`, and `/auto`.
- Provides themes, approvals, and safety controls directly in the TUI.
- Integrates with MCP tools and custom provider wiring.
- Tracks provider credentials without forcing config-file edits.

## Provider support

Kay currently supports these provider credentials in the UI and CLI:

1. OpenCode Go
2. MiniMax
3. OpenAI

Manage them from the TUI with:

- `/provider` for provider credential CRUD
- `/login` for the existing account flow
- `/model` for model selection

You can also set a key directly from the CLI:

```bash
code login --api-key <KEY>
```

If you prefer stdin-safe entry:

```bash
printenv OPENAI_API_KEY | code login --with-api-key
printenv MINIMAX_API_KEY | code login --provider minimax --with-api-key
printenv OPENCODE_GO_API_KEY | code login --provider opencode-go --with-api-key
```

## Quickstart

Install and run:

```bash
npm install -g @alo-labs/kay
code
```

If `code` conflicts with another program on your system, use the `coder` alias instead.

For an interactive sign-in flow, launch Kay and choose the provider path that matches your account or API key.

## Core capabilities

### Agents and orchestration

- `/auto` hands a task to Auto Drive for multi-step coordination.
- `/plan` is for collaborative planning before implementation.
- `/solve` is for fast multi-agent problem solving.
- `/kay` is the main coding workflow.

### Browser workflows

- `/chrome` connects to an external Chrome session.
- `/browser` opens the internal browser experience.

### UI and safety

- `/themes` switches the visual theme.
- `/reasoning` adjusts reasoning effort.
- `/approvals` controls when Kay can proceed automatically.
- `/new` starts a fresh conversation.

## Documentation

- [Authentication](docs/authentication.md)
- [Slash commands](docs/slash-commands.md)
- [Testing](docs/TESTING.md)
- [FAQ](docs/faq.md)

## Attribution and licenses

Kay is distributed under the repository license in [`LICENSE`](LICENSE).

This project acknowledges and preserves the lineage of both Codex and Every Code. Any upstream code, concepts, or notices that Kay inherits remain governed by their original terms and attributions. This README is an overview of the project, not a replacement for the applicable license files or notices.

If you are redistributing or extending Kay, please review the included license files and any upstream notices before shipping changes.
