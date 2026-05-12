# Requirements: code-monorepo

**Defined:** 2026-05-12
**Core Value:** Keep the CLI buildable, understandable, and safe to evolve without disturbing existing workflows.

## v0.8.0 Requirements

Requirements for this milestone. Each maps to roadmap phases.

### Provider Management

- [x] **PROVIDER-01**: Kay can create, read, update, and delete supported providers from within the app via `/provider`.
- [x] **PROVIDER-02**: `/provider` supports the current provider order OpenCode Go, MiniMax, OpenAI and keeps existing provider settings intact.

### Credentials

- [ ] **AUTH-01**: Kay accepts provider API keys as a CLI argument so users can set credentials without editing config files.

### Model Routing

- [x] **MODEL-01**: Kay’s `/model` command lists only models for providers with configured credentials.
- [ ] **MODEL-02**: `/model` exposes the expected model sets for OpenCode Go, MiniMax, and OpenAI.

### Plugin Seams

- [x] **PLUG-01**: Provider plugins and model plugins remain orthogonal, with reusable model-compatibility profiles shared across multiple models instead of hard-coded per SKU.

### Verification

- [ ] **TEST-01**: Automated tests cover provider CRUD, CLI credential entry, provider-aware model filtering, and the reusable compatibility-profile seam.

### Docs

- [ ] **DOCS-01**: Kay documentation explains `/provider`, CLI API-key entry, and provider-aware `/model` behavior for the supported providers.

### Release

- [ ] **REL-01**: Kay release metadata and release notes are updated, and a new release is cut only after the provider-management milestone is verified.

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Provider Expansion

- **PROV-EXP-01**: Support additional provider families beyond OpenCode Go, MiniMax, and OpenAI without changing the provider CRUD surface.

### Model Expansion

- **MODEL-EXP-01**: Add more model-specific compatibility profiles without coupling them to provider registration.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Adding provider families beyond OpenCode Go, MiniMax, and OpenAI during this milestone | The current milestone only needs the supported provider set in the specified order. |
| Forcing config-file edits for provider onboarding | The user explicitly wants provider setup through the app or CLI arguments. |
| Hard-coding model adaptations per SKU instead of reusable profiles | The milestone should move toward reusable model plugins. |
| Replacing the existing OpenAI, MiniMax, or OpenCode Go provider behavior | Existing providers must keep working while the new UX is added. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PROVIDER-01 | Phase 4 | Complete |
| PROVIDER-02 | Phase 4 | Complete |
| AUTH-01 | Phase 4 | Pending |
| MODEL-01 | Phase 5 | Complete |
| MODEL-02 | Phase 5 | Pending |
| PLUG-01 | Phase 5 | Complete |
| TEST-01 | Phase 6 | Pending |
| DOCS-01 | Phase 6 | Pending |
| REL-01 | Phase 6 | Pending |

**Coverage:**
- v0.8.0 requirements: 9 total
- Mapped to phases: 9
- Unmapped: 0 ✓

---
*Requirements defined: 2026-05-12*
*Last updated: 2026-05-12 after v0.8.0 semver realignment*
