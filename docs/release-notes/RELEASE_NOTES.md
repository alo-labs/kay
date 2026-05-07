## @just-every/code v0.6.99

This release packages Kay as a standalone Every Code build, keeps MiniMax as the active provider, and hardens the release path around host-environment sharing and installer behavior.

### Changes

- Kay: make `code` the canonical command, keep MiniMax provider support active, and treat the host `~/.codex` tree as a read-only environment overlay while keeping Kay auth/history local.
- Installer: repoint Sidekick consumption to `alo-labs/kay`, keep the local `code` launcher canonical, and harden startup against upstream merge and publish-path surprises.
- Release/CI: skip npm publish when the token is absent, skip Homebrew tap publishing when the PAT is absent, and keep the release checks from failing on missing external credentials.
- Prompts/config: preserve local `~/.code` precedence while borrowing host prompts, instructions, skills, MCPs, and plugin roots by reference when available.

### Install

```bash
npm install -g @just-every/code@latest
code
```

### Thanks

Thanks to @owenlin for contributions!

Compare: https://github.com/just-every/code/compare/v0.6.98...v0.6.99
