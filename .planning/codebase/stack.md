# Stack

## Summary

This repo is a Rust-first CLI monorepo with root-level Node.js tooling and a large markdown docs surface.

## Primary Technologies

- Rust workspace under `code-rs/`
- Node.js package at the repo root
- Markdown-based docs and workflow scaffolding
- GitHub Actions for CI / release automation

## Practical Notes

- The build gate is `./build-fast.sh`
- `rustfmt` is intentionally avoided during init and completion
- Existing docs and workflows are part of the repo contract, not disposable scaffolding
