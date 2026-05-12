# Milestone Requirements: v0.9.0 Test Notes App and Kay OCG Validation

## Requirement Set

### KAY-01 - Isolated Kay Home
Kay must default to its own writable home under `~/.kay` for end-user installs and not auto-inherit local Codex/Every Code state unless the user explicitly overrides the path.

### KAY-02 - No Silent Codex Inheritance
Kay must stop silently reading local `~/.code` or `~/.codex` state as the default source of truth for the end-user install.

### NOTE-01 - Real Validation Repo
A real `alo-exp/test-notes-app` repository must exist and be usable from a local checkout under `projects/test-notes-app` so Kay can drive meaningful note-taking work there.

### VIEW-01 - Transcript Viewer
Kay session transcripts must be inspectable through a lightweight, modern, chat-like viewer that reads the JSONL transcript/provenance stream.

### TEST-01 - OCG Model Coverage
The supported OpenCode Go models must be exercised on substantial note-app work, not just one-line smoke prompts.

### DOCS-01 - Workflow Documentation
The isolated Kay install, transcript viewer, and note-app validation workflow must be documented clearly enough for repeatable use before releases.

### REL-01 - Release Gate
The next release may only happen after the isolated Kay runtime and note-app validation workflow have been verified.

## Validation Notes

- `~/.kay` is the target home for the end-user install.
- `projects/test-notes-app` is the live project target for the OCG model work.
- Session JSONL is the provenance source of truth for analysis and the viewer.
- The OCG model matrix must prove useful note-app work, not a trivial smoke result.
