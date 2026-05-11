<!-- This file is managed by Silver Bullet. Do not edit manually. -->
<!-- To update: run /silver:init in your project. -->

# code-monorepo — Silver Bullet Enforcement Instructions

> **Always adhere strictly to this file — it overrides all defaults.**

---

## 0. Session Startup (Automatic)

At the very start of a new session:

1. Read `CLAUDE.md` and the markdown files in `docs/` for project context.
2. Run `/compact` before starting work.
3. Check for updates:
   - Silver Bullet version from `~/.claude/plugins/installed_plugins.json`
   - GSD version from `~/.claude/get-shit-done/VERSION`
   - Optional plugins: Superpowers, Design, Engineering, MultAI, Product Management
4. If a newer version is available, ask before upgrading. If the check is offline or unreadable, continue.
5. If bypass-permissions is detected, write `~/.claude/.silver-bullet/mode` as `autonomous` and continue with defaults.

## 1. Automated Enforcement

- `~/.claude/.silver-bullet/session-init` records that session startup already ran.
- `~/.claude/.silver-bullet/mode` stores the current session mode.
- `.planning/` holds authoritative planning state; do not treat SB state as the source of truth for phases.
- Never overwrite or delete existing project docs during init.
- Use the repo's existing build gate: `./build-fast.sh` from the repo root.
- Treat workflow and docs edits as part of the initialization surface, not incidental clutter.

## 2. Active Workflow

The active workflow is read from `docs/workflows/`.

- Default dev workflow: `docs/workflows/full-dev-cycle.md`
- DevOps / infra workflow: `docs/workflows/devops-cycle.md`

Read the active workflow before any non-trivial task. If the task is infrastructure-heavy, switch to the DevOps workflow.

## 3. Workflow Transitions

- Dev -> DevOps: after shipping an app release or when the task is infrastructure-focused.
- DevOps -> Dev: after finishing infrastructure work and returning to feature work.
- Preserve `.planning/`, `.silver-bullet.json`, and git history when switching workflows.

## 4. GSD Command Awareness

Core commands you should expect to use:

| Command | Purpose |
|---------|---------|
| `/gsd:new-project` | Create project context, requirements, and roadmap |
| `/gsd:map-codebase` | Build brownfield codebase intelligence |
| `/gsd:discuss-phase` | Capture decisions before planning |
| `/gsd:plan-phase` | Decompose work into executable plans |
| `/gsd:execute-phase` | Implement plan waves and write summaries |
| `/gsd:verify-work` | Check the work against the phase requirements |
| `/gsd:ship` | Finalize and prepare release output |
| `/gsd:next` | Auto-advance to the next logical step |
| `/gsd:resume-work` | Restore the current project state |
| `/gsd:debug` | Diagnose workflow or implementation issues |
| `/gsd:review` | Cross-model review before merge or release |

If a required GSD or SB skill is unavailable, stop and say so rather than silently substituting a different path.

## 5. State Awareness

- Use `.planning/STATE.md` and `.planning/ROADMAP.md` to determine where the project actually is.
- Use `.planning/PROJECT.md` for the durable project definition and current decisions.
- Use `docs/doc-scheme.md` and `docs/doc-scheme.json` for documentation governance.

## 6. Docs Governance

- Create or update docs non-destructively.
- When docs are added, update the docs scheme and checklist together.
- Keep `docs/knowledge/` and `docs/lessons/` monthly files append-only.

## 7. Interactive Flow

When working interactively:

- Show a short progress banner at phase transitions.
- Ask targeted questions only when the choice materially changes the work.
- Prefer clear defaults when the repo context makes the safe option obvious.

## 8. Autonomous Flow

When the session mode is autonomous:

- Proceed without asking for routine confirmations.
- Log decisions in project state instead of pausing for every step.
- Surface blockers only when they are real blockers.

## 9. Non-Destructive Rules

- No `git reset --hard`, `git checkout --`, or `git clean`.
- Do not delete user docs or overwrite existing markdown without a clear migration path.
- Preserve existing repo instructions and append SB guidance rather than replacing it.

