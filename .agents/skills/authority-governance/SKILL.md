---
name: authority-governance
description: Enforce the Authority repository governance contract, including the complete .agent preflight and append-only lifecycle records.
---

# Authority Governance Skill

## Purpose

Use this skill for repository work governed by the Authority package. Codex is the primary coding agent; `AGENTS.md` is always-on, and this skill supplies the individually applicable workflow.

## Mandatory inputs

Read the repository root `AGENTS.md` and every contract file in `.agent/` before planning or editing:

- `PROJECT_GOAL.md`
- `PROJECT_PROFILE.md`
- `CURRENT.md`
- `DIRECTIVES.md`
- `OUTCOMES.md`
- `LEARNINGS.md`
- `RECORD.md`
- `REPO_MAP.md`

Do not treat a subset as sufficient. If any required file is missing, unreadable, stale, or contradictory, stop and report the limitation before changing code.

## Ordered workflow

1. Confirm the repository root and locate the nearest applicable `AGENTS.md` files.
2. Read the complete `.agent/` contract and reconcile the active directive, acceptance condition, current state, and relevant historical records.
3. Inspect existing implementations, callers, tests, and integration boundaries before proposing changes.
4. Make the smallest authorized change and preserve unrelated work.
5. Run the smallest sufficient validation plus any acceptance-critical checks.
6. Record the result in `.agent/OUTCOMES.md`, update `.agent/CURRENT.md`, and append learnings or decisions only when durable evidence warrants them.
7. Report passed, failed, unavailable, and unrun checks distinctly. Never claim completion from prose, configuration, or an app-level success signal alone.

## `.agent/` contract

`.agent/` is a required project interface consumed by multiple processes. Do not rename, remove, flatten, replace, or selectively bypass its files. `CURRENT.md` is mutable state; `DIRECTIVES.md`, `OUTCOMES.md`, `LEARNINGS.md`, and `RECORD.md` are append-only ledgers after adoption. Local files remain authoritative unless an explicitly adopted external contract says otherwise.

## Precedence

Apply instructions in this order: runtime safety and platform restrictions; the user’s current request; the active project directive and acceptance condition; verified adopted external contracts; existing repository behavior, tests, interfaces, and compatibility commitments; `AGENTS.md`; this skill; then tool defaults. Report unresolved material conflicts instead of silently choosing the convenient interpretation.

## Scope and change discipline

Make the smallest authorized change. Understand affected callers, inputs, outputs, state, errors, and contracts before changing shared behavior. Preserve unrelated work, reuse established code and dependencies, fix root causes, and do not change application behavior, deployment, or external systems outside the authorized scope.

## Validation

Use the smallest sufficient check and report each result as `PASSED`, `FAILED`, `NOT RUN`, `NOT APPLICABLE`, or `BLOCKED`. Failed, skipped, unavailable, and timed-out checks are not passes. Governance and documentation changes require structural/content validation even without runtime tests.

## Safety and destructive actions

Do not delete, overwrite, rewrite history, force-push, migrate data, alter infrastructure, or deploy without exact authorization. Resolve exact targets, preserve uncommitted work, protect secrets and personal data, and verify recovery or rollback paths where relevant. Written guidance is not a substitute for deterministic enforcement.

## External integrations

Use a named integration only when its configured server, executable, or documented binding is present and verified. Follow its current tool contract, retain returned identifiers where applicable, and never claim unavailable lifecycle steps succeeded. Keep credentials, raw logs, full source files, and unsupported claims out of external memory.

## Completion and reporting

Before completion, reconcile acceptance, inspect changed files and the final diff when available, review validation states, update local records, and identify unresolved risks or conflicts. Report changes, affected files, checks, manual verification, deviations, blockers, and deployment status. Do not claim completion when acceptance is partial or unverified.

## Failure handling

If the governance files conflict, follow the precedence in `AGENTS.md`, surface the conflict, and do not silently choose the convenient interpretation. If validation fails or cannot run, record that exact state and stop short of claiming completion.
