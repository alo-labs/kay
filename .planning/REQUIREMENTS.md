# Requirements: code-monorepo

**Defined:** 2026-05-23
**Core Value:** Keep the CLI buildable, understandable, and safe to evolve
without disturbing existing workflows.

## v1 Requirements

### Workspace Path

- [ ] **PATH-01**: Kay's active Rust workspace lives at `kay-rs/` and no
  tracked or generated `code-rs` filesystem compatibility path remains.
- [ ] **PATH-02**: `./build-fast.sh` defaults to the Kay workspace at
  `kay-rs/`, accepts `kay` and `kay-rs` as primary workspace selectors, and
  keeps `code` / `code-rs` as selector aliases only.
- [ ] **PATH-03**: Root scripts, cleanup scripts, release scripts, local
  smokes, and package scripts resolve the active Kay workspace through
  `kay-rs/`.

### Compatibility

- [ ] **COMPAT-01**: Rust crate package names such as `code-core`,
  `code-cli`, and imports such as `code_core` remain unchanged.
- [ ] **COMPAT-02**: Shipped binary compatibility remains unchanged: `kay`
  stays primary and existing `code`, `code-tui`, and `code-exec`
  compatibility binaries remain where already supported.
- [ ] **COMPAT-03**: The `codex-rs/` mirror remains read-only and untouched by
  this milestone.

### Automation and Documentation

- [ ] **AUTO-01**: GitHub build, preview, issue, release, and upstream-merge
  workflows reference `kay-rs/` for Kay-owned workspace operations while
  preserving true upstream `codex-rs/` references.
- [ ] **AUTO-02**: Upstream comparison tooling compares `codex-rs/` against
  `kay-rs/` and keeps the existing branding-normalized diff behavior.
- [ ] **DOC-01**: Active docs and repo instructions describe `kay-rs/` as the
  Kay Rust workspace and record remaining legacy `code-*` names as
  compatibility or future-phase boundaries.
- [ ] **TEST-01**: The migration passes the required build gate, path audit,
  selector checks, focused script checks, TypeScript SDK path tests, and full
  pre-release gate before pushing to `main`.

## v2 Requirements

### Rust Identity Rename

- **IDENT-01**: Rename Rust crate packages, imports, generated protocol
  identifiers, and compatibility binaries from `code-*` toward `kay-*` in a
  later explicitly planned milestone.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Removing `codex-rs/` | Separate mirror-removal plan; this milestone only renames the active Kay workspace path. |
| `code-*` crate/package/import rename | Larger public/API migration deferred to a phased full-rename milestone. |
| Release cut | The rename is release-sensitive, but tagging/publishing remains an explicit separate request. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PATH-01 | Phase 12 | Pending |
| PATH-02 | Phase 12 | Pending |
| PATH-03 | Phase 12 | Pending |
| COMPAT-01 | Phase 12 | Pending |
| COMPAT-02 | Phase 12 | Pending |
| COMPAT-03 | Phase 12 | Pending |
| AUTO-01 | Phase 12 | Pending |
| AUTO-02 | Phase 12 | Pending |
| DOC-01 | Phase 12 | Pending |
| TEST-01 | Phase 12 | Pending |

**Coverage:**
- v1 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0

---
*Requirements defined: 2026-05-23*
*Last updated: 2026-05-23 after milestone initialization*
