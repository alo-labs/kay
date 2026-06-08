## @alo-labs/kay v0.9.22

This patch closes the remaining open reliability issues from the issue sweep and
hardens headless provider workflows.

### Changes

- CLI: accept `--ask-for-approval` on `kay exec` and preserve explicit
  approval-policy overrides while keeping headless defaults.
- Runtime: repair MiniMax-M3's malformed `ls && -la && <path>` shell probes
  before command execution.
- Skills: make exact named skill/workflow requests binding so `silver:init`
  cannot drift into adjacent scan or discovery paths.
- Release hygiene: close obsolete v0.8.0 install-asset issue #13 after
  confirming current releases publish installable assets.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay
```

Compare: https://github.com/alo-labs/kay/compare/v0.9.21...v0.9.22
