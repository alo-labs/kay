# Deferred Items

## 2026-05-12

- `cargo test -p code-cli provider_api_key_entry -- --nocapture` is blocked by an unrelated workspace compile error in `code-rs/tui/src/bottom_pane/provider_credentials_view.rs`:
  - `error[E0603]: module model_provider_info is private`
  - This file was not part of the plan scope, so it was logged instead of being edited here.
- `./build-fast.sh` surfaced warnings in the sibling provider credentials UI file `code-rs/tui/src/bottom_pane/provider_credentials_view.rs`:
  - `variant 'Edit' is never constructed`
  - `fields 'tail_ticket' and 'mode' are never read`
  - The warnings were outside this plan's scope, so they were deferred instead of being fixed here.
