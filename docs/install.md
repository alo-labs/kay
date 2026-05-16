## Installing & building

### System requirements

| Requirement                 | Details                                                         |
| --------------------------- | --------------------------------------------------------------- |
| Operating systems           | macOS 12+, Ubuntu 20.04+/Debian 10+, or Windows 11 **via WSL2** |
| Git (optional, recommended) | 2.23+ for built-in PR helpers                                   |
| RAM                         | 4-GB minimum (8-GB recommended)                                 |

### npm

```bash
npm install -g @alo-labs/kay
```

Use `kay` to launch the CLI. The npm package also provides compatibility aliases `codex` and `coder`, plus a `code` compatibility alias when no existing `code` command would be shadowed.

### Release Archives

GitHub Releases contain `kay-*` assets for each supported platform. Legacy `code-*` assets are published during the migration so existing scripts can keep working.

### Build from source

```bash
# Clone the repository and navigate to the workspace root.
git clone https://github.com/alo-labs/kay.git
cd kay

# Install the Rust toolchain, if necessary.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Build everything (CLI, TUI, MCP servers). This is the same check CI runs.
./build-fast.sh

# Launch the TUI with a sample prompt.
./target/debug/kay -- "explain this codebase to me"
```

> [!NOTE]
> The project treats compiler warnings as errors. The only required local check is `./build-fast.sh`; skip `rustfmt`/`clippy` unless asked.
