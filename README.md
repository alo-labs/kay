# Kay

Kay is a terminal coding agent built around local, scriptable, multi-provider workflows.

Its defining difference is first-class provider architecture: credentials, provider selection, and model routing are built into the product instead of being bolted on later.

It carries forward the ergonomics of the Codex CLI lineage and the multi-provider direction from Every Code, but it is its own project with its own release line, UI decisions, and isolated home directory under `~/.kay`.

## Why Kay exists separately

- The main reason Kay exists separately is that provider selection, credential management, and model routing are first-class architecture here, not add-ons.
- Codex gave this project its original CLI and agent workflow shape.
- Every Code introduced the multi-provider direction and the idea that provider/model support should be a core capability.
- Kay exists so that architecture can evolve independently without being forced into a rename-only or compatibility-only release model.
- Kay keeps the upstream lineage visible, but it is not the same project as Codex or Every Code.

## What Kay does well

- Runs as a local coding agent in your terminal.
- Coordinates multi-step tasks with Auto Drive.
- Supports browser-driven workflows through internal browser mode or CDP/Chrome.
- Provides multi-agent commands such as `/plan`, `/kay`, `/solve`, and `/auto`.
- Exposes a provider workflow for adding, updating, and removing credentials.
- Keeps its own state under `~/.kay` instead of inheriting a local Codex or Every Code environment.
- Integrates with MCP tools, custom agents, and safety controls directly in the TUI.
- Records transcript JSONL so sessions remain inspectable and attributable.

## Install

Kay is distributed through GitHub Releases. It is not currently published on npm, so `npm install -g @alo-labs/kay` will return a 404.

1. Open the latest release: [alo-labs/kay releases](https://github.com/alo-labs/kay/releases/latest)
2. Download the asset for your platform:
   - macOS arm64: `code-aarch64-apple-darwin.tar.gz` or `code-aarch64-apple-darwin.zst`
   - macOS x64: `code-x86_64-apple-darwin.tar.gz` or `code-x86_64-apple-darwin.zst`
   - Linux arm64 musl: `code-aarch64-unknown-linux-musl.tar.gz` or `code-aarch64-unknown-linux-musl.zst`
   - Linux x64 musl: `code-x86_64-unknown-linux-musl.tar.gz` or `code-x86_64-unknown-linux-musl.zst`
   - Windows x64: `code-x86_64-pc-windows-msvc.exe.zip`
3. Extract the archive and run the `code` binary.
4. If `code` is already taken on your machine, use the `coder` alias instead.

Example for macOS or Linux:

```bash
tar -xzf code-x86_64-apple-darwin.tar.gz
./code
```

Example for Windows PowerShell:

```powershell
Expand-Archive .\code-x86_64-pc-windows-msvc.exe.zip
.\code.exe
```

## Getting Started

1. Launch Kay:

   ```bash
   code
   ```

2. Set up a provider from inside the TUI with `/provider`.
   - Add the provider API key for OpenCode Go, MiniMax, or OpenAI.
   - If you prefer to avoid the TUI flow, you can provide the key from the CLI instead:

   ```bash
   code login --provider opencode-go --api-key <KEY>
   code login --provider minimax --api-key <KEY>
   code login --provider openai --api-key <KEY>
   ```

   If you want stdin-safe entry:

   ```bash
   printenv OPENCODE_GO_API_KEY | code login --provider opencode-go --with-api-key
   printenv MINIMAX_API_KEY | code login --provider minimax --with-api-key
   printenv OPENAI_API_KEY | code login --provider openai --with-api-key
   ```

3. Pick a model with `/model`.
   - Kay shows the models available for the providers you have configured.
   - For OpenCode Go, that is the OpenCode Go model list we already support.
   - For MiniMax, that is MiniMax M2.7.
   - For OpenAI, that is the upstream OpenAI model set supported by Codex.

4. Start a task:
   - Type a prompt directly into the TUI, for example: `refactor this module`
   - Or run a one-shot command with `code exec "..."`.
   - Use `/kay`, `/plan`, `/solve`, or `/auto` when you want a specialized workflow.

5. Review the transcript later if you need provenance or debugging context. Kay stores JSONL transcripts under `~/.kay/`, and the transcript viewer makes them easy to inspect.

## Build from source

```bash
git clone https://github.com/alo-labs/kay.git
cd kay

# Install Rust if you do not already have it.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Build everything the same way CI validates it.
./build-fast.sh

# Launch the TUI.
./target/debug/code -- "explain this codebase to me"
```

## Provider support

Kay currently supports these provider credentials in the UI and CLI, in this order:

1. OpenCode Go
2. MiniMax
3. OpenAI

Manage provider credentials from the TUI with:

- `/provider` for provider credential CRUD
- `/model` for model selection from the providers you have configured
- `/login` for the existing account flow

You can also set a provider key directly from the CLI:

```bash
code login --provider opencode-go --api-key <KEY>
code login --provider minimax --api-key <KEY>
code login --provider openai --api-key <KEY>
```

If you prefer stdin-safe entry:

```bash
printenv OPENCODE_GO_API_KEY | code login --provider opencode-go --with-api-key
printenv MINIMAX_API_KEY | code login --provider minimax --with-api-key
printenv OPENAI_API_KEY | code login --provider openai --with-api-key
```

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

### Transcripts and provenance

- Conversation history is recorded as JSONL under `~/.kay/`.
- The transcript viewer provides a lightweight way to inspect sessions after the fact.
- This makes it easier to debug model behavior, inspect provenance, and review what happened in a previous Kay session.

## Documentation

- [Getting Started](docs/getting-started.md)
- [Configuration](docs/config.md)
- [Authentication](docs/authentication.md)
- [Slash commands](docs/slash-commands.md)
- [Testing](docs/TESTING.md)
- [FAQ](docs/faq.md)
- [Install](docs/install.md)
- [Homebrew](docs/homebrew.md)

## Attribution and licenses

Kay is distributed under the repository license in [`LICENSE`](LICENSE).

This project acknowledges and preserves the lineage of both Codex and Every Code. Any upstream code, concepts, or notices that Kay inherits remain governed by their original terms and attributions. This README is an overview of the project, not a replacement for the applicable license files or notices.

If you are redistributing or extending Kay, please review the included license files and any upstream notices before shipping changes.
