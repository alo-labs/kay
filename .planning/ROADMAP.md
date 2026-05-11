# Roadmap: code-monorepo

## Overview

This milestone adds first-class OpenCode Go support to Kay, proves it with a live OpenCode Go API key, and finishes with a Kay release once the integration is verified. The work stays inside the existing multi-provider architecture and preserves OpenAI and MiniMax behavior.

## Phases

- [ ] **Phase 1: OpenCode Provider Foundation** - Add the built-in provider, auth plumbing, and representative model routing.
- [ ] **Phase 2: Live Validation and Docs** - Prove the provider with a live API key and document the supported setup.
- [ ] **Phase 3: Release Kay** - Bump release metadata, update notes, and cut the release after verification.

## Phase Details

### Phase 1: OpenCode Provider Foundation
**Goal**: Add OpenCode Go as a first-class provider and make a representative OpenCode Go model resolve correctly.
**Depends on**: Nothing
**Requirements**: PROV-01, PROV-02, MODEL-01
**Success Criteria** (what must be TRUE):
  1. Kay can load an OpenCode Go provider entry without breaking OpenAI or MiniMax.
  2. A representative `opencode-go/<model>` slug resolves to the expected provider/model-family behavior.
  3. Existing provider-related tests continue to pass alongside the new provider wiring.

### Phase 2: Live Validation and Docs
**Goal**: Prove the provider works with a real API key and make setup discoverable in docs.
**Depends on**: Phase 1
**Requirements**: TEST-01, DOCS-01
**Success Criteria** (what must be TRUE):
  1. A live test can authenticate with the supplied OpenCode Go API key and complete a prompt.
  2. The docs show how to configure OpenCode Go and which model namespace is verified.
  3. Verification output clearly distinguishes the tested OpenCode Go path from any unverified model families.

### Phase 3: Release Kay
**Goal**: Cut a new Kay release after the OpenCode Go integration has been verified.
**Depends on**: Phase 2
**Requirements**: REL-01
**Success Criteria** (what must be TRUE):
  1. Release notes and version metadata reflect the OpenCode Go milestone.
  2. `./build-fast.sh` passes on the release candidate.
  3. The release is ready to publish with the new provider support included.

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. OpenCode Provider Foundation | 0/TBD | Not started | - |
| 2. Live Validation and Docs | 0/TBD | Not started | - |
| 3. Release Kay | 0/TBD | Not started | - |
