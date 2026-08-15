---
name: mimir
description: Mimir-specific task lifecycle workflow used only when the Mimir integration is configured
---

# Mimir Skill

## Purpose

Provide the operational sequence for using a configured Mimir integration. Repository policy remains in `AGENTS.md`; project history remains in local `.agent/` files.

## Inputs

- Verified repository root and project identity from `.agent/PROJECT_PROFILE.md`.
- Active Mimir tool catalog or documented adapter.
- Task objective, scope, acceptance condition, and any external directive ID.

## Ordered steps

1. Resolve an existing project binding or register the current repository when no binding exists.
2. Begin the task and retain the returned task ID and version.
3. Compile project-scoped context before overlapping work and retain retrieval/session identifiers.
4. Observe only durable decisions, constraints, hypotheses, or root causes that Git cannot prove; retain each returned version.
5. Run completion-critical validation through the configured Mimir validation mechanism when required by the project contract.
6. Inspect task evidence and exclude failed or timed-out checks from passing evidence.
7. Submit retrieval feedback using only memory IDs that actually helped.
8. Close the task with verified changed files, tests, lessons, status, and the latest version.

## Outputs

- Task ID and latest version.
- Retrieval session and genuinely useful memory IDs, when applicable.
- Evidence-backed validation result.
- Mimir close result, when the full lifecycle succeeds.
- Local `.agent/OUTCOMES.md` entry regardless of Mimir availability.

## Failure handling

If any required Mimir step is unavailable, stop claiming Mimir progress at that step, record the exact limitation locally, and continue only when the work is safe and locally verifiable. Never store secrets, credentials, environment files, raw logs, full source files, full diffs, or unsupported claims.

## Verification

Confirm the active tools expose the required operations before starting. Confirm each successful call’s returned identifier/version before using it later. Inspect evidence before close. Report unavailable or unrun steps as `NOT RUN` or `BLOCKED`, never as passed.
