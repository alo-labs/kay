# Kay

Kay is a Codex-style terminal coding agent built for developers who want the same local agent workflow with more control over model cost.

The core idea is simple: keep OpenAI available when it is the right choice, but make cost-effective, high-performing model providers first-class options instead of one-off proxy hacks. Kay ships with built-in routing for Xiaomi MiMo, OpenCode Go, MiniMax, and OpenAI, so you can choose the model/provider mix that fits the task and budget.

Kay keeps its own state under `~/.kay`, supports interactive and headless workflows, and preserves the local terminal ergonomics of the Codex CLI lineage while moving provider selection, credential management, and model routing into the product.

## Why Use Kay

- Lower routine coding costs by routing everyday agent work to built-in non-OpenAI providers.
- Keep OpenAI in the same toolchain for tasks where you still want OpenAI models.
- Use Xiaomi MiMo directly with `xiaomi/mimo-v2.5-pro` and `xiaomi/mimo-v2.5`.
- Switch providers and models from `/provider` and `/model` instead of editing wrapper scripts.
- Run the same workflows interactively in the TUI or non-interactively through `kay exec`.
- Keep auth, transcripts, logs, config, and sessions isolated under `~/.kay`.

## Built-In Providers

Kay can use custom OpenAI-compatible providers, but these providers are available out of the box:

| Provider | Built-in models | Why it matters |
| --- | --- | --- |
| Xiaomi | `xiaomi/mimo-v2.5-pro`, `xiaomi/mimo-v2.5` | Direct Xiaomi MiMo support for cost-effective coding turns. |
| OpenCode Go | `opencode-go/glm-5.1`, `opencode-go/kimi-k2.6`, `opencode-go/mimo-v2.5-pro`, `opencode-go/mimo-v2.5`, `opencode-go/minimax-m2.7`, `opencode-go/qwen3.6-plus`, `opencode-go/deepseek-v4-pro`, `opencode-go/deepseek-v4-flash` | A curated set of high-performing coding models behind one provider. |
| MiniMax | `MiniMax-M2.7` | A focused built-in option for MiniMax workflows. |
| OpenAI | The upstream OpenAI model list supported by Kay's Codex lineage. | Keep OpenAI available without making it the only path. |

Provider availability, pricing, quotas, and model names can change. Use `/model` inside Kay to see the exact choices available for your configured credentials.

## Install

Install from npm:

```bash
npm install -g @alo-labs/kay
```

Or run it without a global install:

```bash
npx -y @alo-labs/kay
```

The main command is `kay`. The package also installs `codex` and `coder` compatibility aliases, and installs the legacy `code` alias when doing so will not override another `code` command already on `PATH`.

Standalone release archives are also available from [GitHub Releases](https://github.com/alo-labs/kay/releases/latest).

## Quick Start

Launch the TUI:

```bash
kay
```

Add a provider key from the TUI:

```text
/provider
```

Or add a provider key from the CLI:

```bash
kay login --provider xiaomi --api-key <KEY>
kay login --provider opencode-go --api-key <KEY>
kay login --provider minimax --api-key <KEY>
kay login --provider openai --api-key <KEY>
```

For shell-safe key entry, pipe the key through stdin:

```bash
printenv XIAOMI_API_KEY | kay login --provider xiaomi --with-api-key
printenv OPENCODE_GO_API_KEY | kay login --provider opencode-go --with-api-key
printenv MINIMAX_API_KEY | kay login --provider minimax --with-api-key
printenv OPENAI_API_KEY | kay login --provider openai --with-api-key
```

Choose a model:

```text
/model
```

Start coding:

```bash
kay "explain this repo and find the riskiest module"
kay exec "run the test suite and summarize the failures"
kay exec --full-auto --sandbox workspace-write "fix the failing tests"
```

Credentials saved with `kay login` are stored in `$KAY_HOME/auth.json`. `KAY_HOME` defaults to `~/.kay`, so the same provider credentials work from any project directory.

## Common Workflows

### Interactive Coding

Use Kay as a terminal coding agent:

```bash
kay "refactor this parser and explain the change"
```

Inside the TUI, use slash commands for focused workflows:

```text
/model
/provider
/plan "design the migration"
/code "implement the selected plan"
/solve "find the root cause of this flaky test"
/auto "finish the bug fix and verify it"
```

### Headless Automation

Use `kay exec` when you want scriptable output:

```bash
kay exec --json "inspect this change and list the risks"
kay exec --output-schema schema.json --output-last-message "extract the release summary"
kay exec --full-auto --sandbox workspace-write "update docs and run the relevant checks"
```

`kay exec` defaults to read-only. Add `--full-auto` and a writable sandbox only when you want Kay to edit files or run commands that change the workspace.

### Browser And Agent Workflows

Kay includes browser and multi-agent commands:

```text
/browser
/browser https://example.com
/chrome
/chrome 9222
/plan "decompose this feature"
/solve "debug this failure"
/auto "drive this task to completion"
```

## Configuration

Kay reads configuration from `~/.kay/config.toml` by default. If that file is absent, Kay can also read the older `~/.kay/kay.toml` provider-default format.

Minimal example:

```toml
model = "xiaomi/mimo-v2.5-pro"
model_provider = "xiaomi"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
model_reasoning_effort = "medium"

[tui.theme]
name = "light-photon"
```

Built-in Xiaomi provider configuration:

```toml
[model_providers.xiaomi]
name = "Xiaomi"
base_url = "https://token-plan-sgp.xiaomimimo.com/v1"
env_key = "XIAOMI_API_KEY"
wire_api = "chat"
requires_openai_auth = false
```

Kay also supports custom providers that expose OpenAI-compatible Chat Completions or Responses APIs:

```toml
model = "mistral"
model_provider = "local-ollama"

[model_providers.local-ollama]
name = "Local Ollama"
base_url = "http://localhost:11434/v1"
wire_api = "chat"
requires_openai_auth = false
```

Useful environment variables:

- `KAY_HOME`: Override Kay's config and state directory.
- `XIAOMI_API_KEY`: API key for the built-in Xiaomi provider.
- `OPENCODE_GO_API_KEY`: API key for the built-in OpenCode Go provider.
- `MINIMAX_API_KEY`: API key for the built-in MiniMax provider.
- `OPENAI_API_KEY`: Use OpenAI API key auth.
- `OPENAI_BASE_URL`: Override the built-in OpenAI base URL.
- `OPENAI_WIRE_API`: Force the built-in OpenAI provider to use `chat` or `responses`.

See [Configuration](docs/config.md) for the full config reference.

## Project Memory And Transcripts

Kay reads project instructions from files such as `AGENTS.md` and `CLAUDE.md` in your project root. Use those files to record local conventions, test commands, architecture notes, and safety rules.

Kay records transcripts and logs under `~/.kay/`, including JSONL history that can be inspected later for provenance, debugging, or review.

## Build From Source

```bash
git clone https://github.com/alo-labs/kay.git
cd kay

# Install Rust if needed.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Build the workspace using the same required local gate.
./build-fast.sh

# Launch the built binary.
./kay-rs/bin/kay
```

## FAQ

**How is Kay different from Codex?**

Kay is a community fork of the upstream `openai/codex` CLI focused on provider choice and model cost control. It keeps the terminal coding workflow, then adds built-in provider management for Xiaomi MiMo, OpenCode Go, MiniMax, and OpenAI.

**Is Kay only for cheaper models?**

No. Kay is for choosing the right model for each job. The point is to avoid making one provider the default for every task when lower-cost providers can handle much of the daily coding workload.

**Can I use OpenAI?**

Yes. OpenAI remains a built-in provider. Kay supports both OpenAI API key auth and ChatGPT sign-in where that mode is appropriate.

**Which model should I use?**

Use `/model` after configuring your providers. Kay shows only the model choices available for your current credentials. A practical default is to start with Xiaomi MiMo or another lower-cost provider for routine work, then switch to OpenAI when a task benefits from it.

**Does Kay proxy my traffic?**

No. Kay sends requests from your machine to the provider you configure. Your API keys are stored locally in `~/.kay/auth.json`.

**Can I migrate existing Codex settings?**

Yes. Copy the settings you still want into `~/.kay/config.toml`. Kay keeps some compatibility paths alive, but `~/.kay` is the canonical home for Kay state.

## Documentation

- [Getting Started](docs/getting-started.md)
- [Configuration](docs/config.md)
- [Authentication](docs/authentication.md)
- [Slash Commands](docs/slash-commands.md)
- [Agents](docs/agents.md)
- [Auto Drive](docs/auto-drive.md)
- [Exec Mode](docs/exec.md)
- [MCP](docs/config.md#mcp_servers)
- [Testing](docs/TESTING.md)
- [FAQ](docs/faq.md)
- [Install](docs/install.md)
- [Homebrew](docs/homebrew.md)
- [Advanced](docs/advanced.md)

## Contributing

Kay keeps the required local build gate simple:

```bash
./build-fast.sh
```

Run it from the repository root before sending code, build, packaging, workflow, dependency, or generated-artifact changes. Documentation-only changes can use targeted validation such as `git diff --check`.

To enable repository hooks locally:

```bash
git config core.hooksPath .githooks
```

The `pre-push` hook runs `./pre-release.sh` automatically when pushing to `main`.

## Legal And Use

Kay is distributed under the repository license in [LICENSE](LICENSE). It is a community fork of the Codex CLI lineage and preserves upstream license and notice files where applicable. Kay is not affiliated with, sponsored by, or endorsed by OpenAI.

Using OpenAI, Xiaomi, MiniMax, OpenCode Go, Anthropic, Google, or other provider services through Kay means you are responsible for the terms, limits, data policies, and account rules of the provider you choose.

Your auth file lives at `~/.kay/auth.json`. Inputs and outputs sent to AI providers are handled under those providers' terms and privacy policies.

## Need Help?

Open an issue on [GitHub](https://github.com/alo-labs/kay/issues) or start with the [Getting Started](docs/getting-started.md) guide.
