# Requirements: code-monorepo

**Defined:** 2026-05-11
**Core Value:** Keep the CLI buildable, understandable, and safe to evolve without disturbing existing workflows.

## v1 Requirements

Requirements for this milestone. Each maps to roadmap phases.

### Provider Core

- [ ] **PROV-01**: Kay can select OpenCode Go as a built-in provider and authenticate with a secure API key path.
- [ ] **PROV-02**: Kay routes OpenCode Go requests to the documented OpenCode Go API endpoint and wire format.

### Model Routing

- [ ] **MODEL-01**: Kay recognizes `opencode-go/<model>` slugs for a representative OpenCode Go model and applies the expected model-family defaults.

### Verification

- [ ] **TEST-01**: A live end-to-end test can authenticate with the supplied OpenCode Go API key and complete a simple prompt successfully.

### Docs

- [ ] **DOCS-01**: Kay documentation explains how to configure OpenCode Go and which `opencode-go/<model>` namespace is verified in this milestone.

### Release

- [ ] **REL-01**: Kay release metadata and release notes are updated, and a new release is cut only after OpenCode Go verification passes.

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### OpenCode Go Expansion

- **OPCGO-02**: Support the remaining OpenCode Go model families that require non-OpenAI wire formats.
- **OPCGO-03**: Add broader provider-specific tuning once the first verified model path is stable.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Support for every OpenCode Go model family on day one | The first milestone only needs one verified OpenAI-compatible path. |
| Replacing OpenAI or MiniMax provider behavior | Existing providers must keep working while OpenCode Go is added. |
| Committing or documenting API keys | Secrets must stay out of git history and docs. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PROV-01 | Phase 1 | Pending |
| PROV-02 | Phase 1 | Pending |
| MODEL-01 | Phase 1 | Pending |
| TEST-01 | Phase 2 | Pending |
| DOCS-01 | Phase 2 | Pending |
| REL-01 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 6 total
- Mapped to phases: 6
- Unmapped: 0 ✓

---
*Requirements defined: 2026-05-11*
*Last updated: 2026-05-11 after OpenCode Go milestone start*
