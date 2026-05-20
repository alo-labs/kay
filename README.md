# Kay

Kay is a terminal coding agent built for local, scriptable, multi-provider workflows.

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
- Runs Auto Review in a separate worktree so quality checks can happen without blocking the main flow.
- Keeps long sessions responsive with bounded queues, caches, and history compaction.
- Supports headless `kay exec` for JSONL streams, structured output, and CI automation.

## Current focus

- Auto Drive and Auto Review are decoupled so review finalization does not block the command flow; `Esc` returns control immediately while typing continues.
- Review metadata, worktree context, and history remain queryable after completion.
- Terminal agents are compacted and archived so heavy payloads stay smaller while review linkage is preserved.
- Coordinator and TUI caches are bounded, and background review notes are added as non-blocking history entries.
- Stress tests cover heavy agent churn plus concurrent review and typing responsiveness.
- Use `/model` to see the exact provider-specific model choices available in your current setup.
- The release philosophy is quality-first: the point is not only "can the model write this file" but "did we verify it works".

## Install

Install Kay from npm:

```bash
npm install -g @alo-labs/kay
```

If you want a one-shot launch without a global install:

```bash
npx -y @alo-labs/kay
```

The primary command is `kay`. The package also installs `codex` and `coder` aliases, and installs the legacy `code` alias when doing so would not override another `code` command already on PATH.

Provider credentials you save with `kay login` are stored in `$KAY_HOME/auth.json` (defaults to `~/.kay/auth.json`), so once configured they work from any directory.

GitHub Releases also provide standalone archives.

1. Open the latest release: [alo-labs/kay releases](https://github.com/alo-labs/kay/releases/latest)
2. Download the asset for your platform:
   - macOS arm64: `kay-aarch64-apple-darwin.tar.gz` or `kay-aarch64-apple-darwin.zst`
   - macOS x64: `kay-x86_64-apple-darwin.tar.gz` or `kay-x86_64-apple-darwin.zst`
   - Linux arm64 musl: `kay-aarch64-unknown-linux-musl.tar.gz` or `kay-aarch64-unknown-linux-musl.zst`
   - Linux x64 musl: `kay-x86_64-unknown-linux-musl.tar.gz` or `kay-x86_64-unknown-linux-musl.zst`
   - Windows x64: `kay-x86_64-pc-windows-msvc.exe.zip`
3. Extract the archive and run the `kay` binary. Legacy `code-*` compatibility archives are also published during the migration so existing scripts can keep working.

Example for macOS or Linux:

```bash
tar -xzf kay-x86_64-apple-darwin.tar.gz
./kay
```

Example for Windows PowerShell:

```powershell
Expand-Archive .\kay-x86_64-pc-windows-msvc.exe.zip
.\kay.exe
```

## Getting Started

1. Launch Kay:

   ```bash
   kay
   ```

2. Set up a provider from inside the TUI with `/provider`.
   - Add the provider API key for OpenCode Go, MiniMax, or OpenAI.
   - For OpenAI, `kay login` can also use ChatGPT sign-in when that is the auth mode you want.
   - If you prefer to avoid the TUI flow, you can provide the key from the CLI instead:

   ```bash
   kay login --provider opencode-go --api-key <KEY>
   kay login --provider minimax --api-key <KEY>
   kay login --provider openai --api-key <KEY>
   ```

   If you want stdin-safe entry:

   ```bash
   printenv OPENCODE_GO_API_KEY | kay login --provider opencode-go --with-api-key
   printenv MINIMAX_API_KEY | kay login --provider minimax --with-api-key
   printenv OPENAI_API_KEY | kay login --provider openai --with-api-key
   ```

   These commands save the credentials into `$KAY_HOME/auth.json` so Kay can
   reuse them the next time you launch the CLI, even from a different
   directory. If you plan to use external CLI agents through `[[agents]]`,
   install the binaries you reference there and keep them on `PATH`.

3. Pick a model with `/model`.
   - Kay shows the models available for the providers you have configured.
   - For OpenCode Go, that is the OpenCode Go model list we already support.
   - For MiniMax, that is MiniMax M2.7.
   - For OpenAI, that is the upstream OpenAI model set supported by Codex.

4. Start a task:
   - Type a prompt directly into the TUI, for example: `refactor this module`
   - Or run a one-shot command with `kay exec "..."`.
   - Use `/kay`, `/plan`, `/solve`, or `/auto` when you want a specialized workflow.

5. Review the transcript later if you need provenance or debugging context. Kay stores JSONL transcripts under `~/.kay/history.jsonl` and related logs under `~/.kay/debug_logs/`, and the transcript viewer makes them easy to inspect.

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
./target/debug/kay -- "explain this codebase to me"
```

## Demo Videos

The Every Code lineage also published a few workflow demos that map closely to Kay's browser, Auto Drive, and multi-agent flows:

- [Auto Review](https://www.youtube.com/watch?v=Ra3q8IVpIOc)
- [Auto Drive Overview](https://youtu.be/UOASHZPruQk)
- [Multi-Agent Promo](https://youtu.be/sV317OhiysQ)

## Commands

### Browser

```bash
# Connect Kay to an external Chrome browser running CDP
/chrome
/chrome 9222

# Switch to internal browser mode
/browser
/browser https://example.com
```

### Agents

```bash
# Plan code changes with multiple agents and a consolidated plan
/plan "Stop the AI from ordering pizza at 3AM"

# Solve a hard problem with multiple agents racing on the answer
/solve "Why does deleting one user drop the whole database?"

# Run the main coding workflow
/kay "Show dark mode when I feel cranky"
```

### Auto Drive

```bash
# Hand off a multi-step task; Auto Drive coordinates agents and approvals
/auto "Refactor the auth flow and add device login"

/auto status
```

### General

```bash
/themes
/reasoning low|medium|high
/model
/new
/settings
/provider
/login
/approvals
```

## CLI Reference

```shell
kay [options] [prompt]

Options:
  -m, --model <name>    Override the model for the active provider
  -s, --sandbox <mode>   Set sandbox level (read-only, workspace-write, etc.)
  -a, --ask-for-approval <mode>
                        Choose when commands need human approval
  --full-auto           Convenience alias for low-friction automatic execution
  --dangerously-bypass-approvals-and-sandbox
                        Skip approvals and sandboxing entirely
  -C, --cd <dir>        Use a different working root
  -i, --image <file>    Attach image(s) to the initial prompt
  -c, --config <key=val>
                        Override config values
  -d, --debug           Log API requests and responses to file
  --oss                 Use the local open source model provider
  --version             Show version number
```

Note: `--model` only changes the model name sent to the active provider. To use a different provider, set `model_provider` in `config.toml`. Providers must expose an OpenAI-compatible API (Chat Completions or Responses). Use `--full-auto` for the low-friction automatic execution preset, and reserve `--dangerously-bypass-approvals-and-sandbox` for externally sandboxed environments.

## Headless / CI Mode

For automation and CI/CD:

```shell
# Run a specific task and stream JSONL events
kay exec --json "run the test suite and summarize the failures"

# Generate structured output and keep only the final payload
kay exec --output-schema schema.json --output-last-message "extract the project summary"

# Let Kay edit files and run commands in a writable sandbox
kay exec --full-auto --sandbox workspace-write "run the test suite and fix any failures"
```

`kay exec` defaults to read-only. Use `--full-auto` together with a writable sandbox when you want edits. By default it uses the same authentication method as the TUI, and you can override that per process with `CODEX_API_KEY` if needed.

## Memory & project docs

Kay can remember project context across sessions:

1. Create an `AGENTS.md` or `CLAUDE.md` file in your project root:

   ```markdown
   # Project Context
   This is a React TypeScript application with:
   - Authentication via JWT
   - PostgreSQL database
   - Express.js backend

   ## Key files:
   - `/src/auth/` - Authentication logic
   - `/src/api/` - API client code
   - `/server/` - Backend services
   ```

2. Kay maintains conversation history and transcript files under `~/.kay/`.
3. Kay automatically understands project structure as long as your repo instructions are clear.

## Model Context Protocol (MCP)

Kay supports MCP for extended capabilities:

- File operations
- Database connections
- API integrations
- Custom tools

Configure MCP in `~/.kay/config.toml` under `[mcp_servers.<name>]`:

```toml
[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/project"]
```

## Configuration

Main config file: `~/.kay/config.toml`

> [!NOTE]
> Kay reads from `~/.kay/` by default and keeps its writable state there. If `~/.kay/config.toml` is absent, Kay also reads the older `~/.kay/kay.toml` provider-default format. If you are migrating from a Codex-based setup, copy the settings you want to keep into `~/.kay/config.toml` so Kay can use them explicitly.

```toml
# Model settings
model = "gpt-5.1"
model_provider = "openai"

# Behavior
approval_policy = "on-request"  # untrusted | on-failure | on-request | never
model_reasoning_effort = "medium" # low | medium | high
sandbox_mode = "workspace-write"

# UI preferences
[tui.theme]
name = "light-photon"

# Add config for specific models
[profiles.gpt-5]
model = "gpt-5.1"
model_provider = "openai"
approval_policy = "never"
model_reasoning_effort = "high"
model_reasoning_summary = "detailed"
```

Kay supports custom model providers that expose OpenAI-compatible Chat Completions or Responses APIs. The built-in provider set includes OpenAI, OpenCode Go, and MiniMax, and you can extend it in `config.toml` with additional providers if needed.

### Environment variables

- `KAY_HOME`: Override config directory location
- `OPENAI_API_KEY`: Use API key instead of ChatGPT auth
- `OPENAI_BASE_URL`: Use OpenAI-compatible API endpoints (chat or responses)
- `OPENAI_WIRE_API`: Force the built-in OpenAI provider to use `chat` or `responses` wiring
- `OPENCODE_GO_API_KEY`: API key for the built-in OpenCode Go provider
- `MINIMAX_API_KEY`: API key for the built-in MiniMax provider

## FAQ

**How is this different from the original?**

> Kay is a community fork of the upstream `openai/codex` CLI with browser integration, multi-agent commands, Auto Drive, theming, reasoning controls, MCP support, and a provider workflow built around `~/.kay`.

**Which models are supported?**

> The built-in model list changes over time. Configure your providers, then use `/model` to see the current options Kay can use in your environment.

**Can I use my existing Codex configuration?**

> You can migrate the settings you care about into `~/.kay/config.toml`. Kay keeps a few compatibility paths alive, but `~/.kay` is the directory to treat as canonical.

**Does this work with ChatGPT Plus?**

> Yes. Use the `kay login` flow and choose ChatGPT sign-in for the OpenAI provider when that is the account mode you want.

**Is my data secure?**

> Authentication stays on your machine, and Kay does not proxy your credentials or conversations beyond the provider you choose.

## Documentation

- [Getting Started](docs/getting-started.md)
- [Configuration](docs/config.md)
- [Authentication](docs/authentication.md)
- [Slash commands](docs/slash-commands.md)
- [Agents](docs/agents.md)
- [Auto Drive](docs/auto-drive.md)
- [Exec mode](docs/exec.md)
- [MCP](docs/config.md#mcp_servers)
- [Testing](docs/TESTING.md)
- [FAQ](docs/faq.md)
- [Install](docs/install.md)
- [Homebrew](docs/homebrew.md)
- [Advanced](docs/advanced.md)

## Contributing

We welcome contributions. Kay keeps the core build gate simple: run `./build-fast.sh` from the repository root and make sure it passes cleanly before you send changes.

### Development workflow

```bash
git clone https://github.com/alo-labs/kay.git
cd kay
npm install
./build-fast.sh
./target/debug/kay
```

If you want the repository hooks that ship with Kay, enable them locally:

```bash
git config core.hooksPath .githooks
```

The `pre-push` hook runs `./pre-release.sh` automatically when pushing to `main`.

## Legal & Use

### License & attribution

- Kay is distributed under the repository license in [`LICENSE`](LICENSE).
- Kay is a community fork of the original Codex CLI lineage and preserves upstream LICENSE and NOTICE files where applicable.
- Kay is not affiliated with, sponsored by, or endorsed by OpenAI.

### Your responsibilities

Using OpenAI, Anthropic, Google, MiniMax, OpenCode Go, or other provider services through Kay means you agree to their terms and policies. In particular:

- Do not programmatically scrape or extract content outside intended flows.
- Do not bypass or interfere with rate limits, quotas, or safety mitigations.
- Use your own account; do not share or rotate accounts to evade limits.
- If you configure other model providers, you are responsible for their terms.

### Privacy

- Your auth file lives at `~/.kay/auth.json`
- Inputs and outputs you send to AI providers are handled under their terms and privacy policies; consult those documents and any org-level data-sharing settings.

### Subject to change

AI providers can change eligibility, limits, models, or authentication flows. Kay supports both ChatGPT sign-in and API-key modes so you can pick what fits your workflow.

## Need help?

Open an issue on [GitHub](https://github.com/alo-labs/kay/issues) or check the documentation.
