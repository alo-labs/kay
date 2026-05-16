# Kay Brand Renaming Policy

This policy defines how Kay migrates the remaining `codex` and product-branded
`code` references to `kay`.

The rule is simple:

- use `kay` for first-party Kay surfaces by default
- keep upstream or compatibility names only where they are required
- treat ordinary English uses of the word `code` as prose, not branding
- reconcile daily so rename drift stays small instead of accumulating

## Scope

In scope:

- user-facing docs, help text, and UI copy
- Kay-owned scripts and release tooling
- Kay-owned binary, package, and workspace names
- Kay-owned logs, prompts, and setup text
- migration inventories and reconciliation notes

Out of scope until a compatibility plan exists:

- wire protocol and schema names consumed by external clients
- published package identifiers that downstream tooling already imports
- generated schema/type names that encode upstream contracts
- upstream mirror labels used to compare against OSS Codex

## Decision Rules

1. Rename brand references to `kay` by default.
2. Preserve compatibility names at boundaries that are visible to external
   clients, generated code, or published package consumers.
3. Keep upstream labels only when they identify the upstream baseline.
4. Do not mechanically rewrite files when the rename changes behavior, path
   layout, package identity, or wire compatibility.
5. If a file mixes rename-safe text with a compatibility boundary, split the
   change into a mechanical part and a manual reconciliation part.

## Reconciliation Workflow

1. Pull upstream daily.
2. Review the diff in three buckets:
   - rename-now
   - rename-with-compat
   - retain-for-compat
3. Apply mechanical rename changes only in rename-now files.
4. Hand-review any file that changes a path, package name, binary name, schema
   name, or external contract.
5. Update the rename inventory with new exceptions or completed migrations.
6. Run the required repo gates before merging.

## Acceptance Criteria

- No user-facing Kay surface uses Codex branding unless the name is part of a
  compatibility shim or an upstream reference.
- Every remaining `codex` or product-branded `code` token has a documented
  reason to exist.
- Daily upstream reconciliation keeps common files small and low-drift.
- New Kay work introduces `kay` naming first, not additional legacy tokens.

## Notes

- This policy is intentionally stricter than historical naming in the repo.
- It does not attempt to rename ordinary programming-language prose such as
  "source code" or "code review".
- The companion inventory in `docs/kay-brand-renaming-inventory.md` is the
  working list of current exceptions and migration targets.
