# Getting started with Kay

Kay is a terminal coding agent for local, scriptable, multi-provider workflows. This guide walks through the first session, provider setup, model selection, and a few everyday commands.

## 1. Install Kay

Install Kay from npm:

```bash
npm install -g @alo-labs/kay
```

The primary command is `kay`. The package also installs compatibility aliases `codex` and `coder`, and installs the legacy `code` alias when doing so would not override another `code` command already on PATH.

You can also download a standalone `kay-*` archive from the [latest GitHub release](https://github.com/alo-labs/kay/releases/latest).

If you are building from source:

```bash
git clone https://github.com/alo-labs/kay.git
cd kay
./build-fast.sh
```

## 2. Launch Kay

Start the TUI with:

```bash
kay
```

The legacy `code` command remains available as a compatibility alias where packaging can provide it.

## 3. Add a provider

Kay is multi-provider. The first thing most users should do is register the provider API key they want to use.

Inside the TUI, open:

```text
/provider
```

Use that screen to add, update, or remove credentials for the supported providers:

1. Xiaomi
2. OpenCode Go
3. MiniMax
4. OpenAI

You can also provide a key from the CLI instead of typing it into the TUI:

```bash
kay login --provider xiaomi --api-key <KEY>
kay login --provider opencode-go --api-key <KEY>
kay login --provider minimax --api-key <KEY>
kay login --provider openai --api-key <KEY>
```

If you prefer stdin-safe entry:

```bash
printenv XIAOMI_API_KEY | kay login --provider xiaomi --with-api-key
printenv OPENCODE_GO_API_KEY | kay login --provider opencode-go --with-api-key
printenv MINIMAX_API_KEY | kay login --provider minimax --with-api-key
printenv OPENAI_API_KEY | kay login --provider openai --with-api-key
```

## 4. Choose a model

After at least one provider is configured, open:

```text
/model
```

Kay shows only the models that are available for the providers you have configured.

Model availability currently looks like this:

| Provider | Models shown in `/model` |
| --- | --- |
| Xiaomi | `xiaomi/mimo-v2.5-pro`, `xiaomi/mimo-v2.5` |
| OpenCode Go | `opencode-go/glm-5.1`, `opencode-go/kimi-k2.6`, `opencode-go/mimo-v2.5-pro`, `opencode-go/mimo-v2.5`, `opencode-go/minimax-m2.7`, `opencode-go/qwen3.6-plus`, `opencode-go/deepseek-v4-pro`, `opencode-go/deepseek-v4-flash` |
| MiniMax | `minimax-m2.7` |
| OpenAI | The OpenAI models supported by the upstream model list |

Pick the model that matches the provider key you already added. Kay keeps provider selection and model selection separate so you can mix and match supported providers without editing config files by hand.

## 5. Run your first task

There are three common ways to use Kay:

### Interactive prompt

```bash
kay "refactor this module"
```

### Non-interactive automation

```bash
kay exec "run the test suite and summarize the failures"
```

### Multi-agent workflows

From the TUI, use the built-in commands when you want a more opinionated workflow:

- `/kay` for the main coding flow
- `/plan` for collaborative planning
- `/solve` for fast multi-agent problem solving
- `/auto` for longer, fully coordinated work

## 6. Review transcripts later

Kay records conversation history as JSONL under `~/.kay/`. That makes it easier to inspect a previous session, understand why a change was made, and review model behavior after the fact.

If you want to inspect a previous run, use the transcript viewer in the TUI or open the JSONL transcript directly.

## Helpful extras

### Image input

You can attach screenshots or other images to a prompt.

### Shell completions

Generate shell completions with:

```bash
kay completion bash
kay completion zsh
kay completion fish
```

### Working directory

If it is more convenient than changing directories first, use `kay --cd <path>` to start Kay in a specific working root.

## Where to go next

- [Slash commands](slash-commands.md)
- [Authentication](authentication.md)
- [Configuration](config.md)
- [Testing](TESTING.md)
- [Install](install.md)
