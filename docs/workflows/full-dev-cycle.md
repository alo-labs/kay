# Full Dev Cycle Workflow

This workflow covers feature work and normal product development.

## Start

1. Read `.planning/STATE.md` and `.planning/ROADMAP.md`
2. Confirm the active phase and plan
3. Read the active workflow section in `silver-bullet.md`

## Core Loop

1. `discuss` - capture decisions and close open questions
2. `plan` - decompose the phase into concrete executable work
3. `execute` - implement the plans and commit atomically
4. `verify` - run tests and validate the phase outcomes
5. `review` - inspect the work for regressions and gaps

## Supporting Commands

- `gsd:new-project`
- `gsd:map-codebase`
- `gsd:discuss-phase`
- `gsd:plan-phase`
- `gsd:execute-phase`
- `gsd:verify-work`
- `gsd:ship`
- `gsd:next`

## Finish

- Update project state and docs
- Keep the next phase ready to resume
- Do not delete prior context unless it is clearly superseded
