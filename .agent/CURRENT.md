# CURRENT.md

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-14T19:20:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260814-d096-gate7-parity-correct-executor-preflight`
- External directive ID: `DC-SR-004C-R2`
- Objective: `Construct and verify a Gate-5-parity-correct Gate 7 execution path without executing reproductive Gate 7`
- Current status: `VALIDATING`
- Acceptance: `Exact Gate 5 parity preflight and mechanics/topology compatibility preflight are executed without fission; future Gate 7 protocol is frozen but not executed; R1 execution history is corrected`
- Current phase: `DC-SR-004C-R2 parity-correct executor preflight`
- Expected or actual touched areas: `chemistry-core D-096 shared constructors, evolution-harness mesh adapter and R2 preflight example, sr004cr2 artifacts, parity_correct_preflight docs, scoped CI, .agent records`
- Immediate next action: `Push stacked draft PR and verify exact remote CI; do not execute Gate 7 or Gate 8`

## Temporary task-relevant facts

- The canonical Authority repository is reference-only and must not be modified.
- Existing Digital Cell governance snapshots are preserved under `.agent/legacy/pre-authority-migration-20260814/`.
- The original Gate 7 artifacts under `digital-protocell/experiments/generated/sr004c/` are immutable evidence for this audit.

## Last validation after adoption

- Command or check: `Atlas release focused suites; GitHub Actions run 31845154445 on b258126fb2ac1373515a09711d7dcaa07022550f`
- Result: `PASSED: 14 D-096 tests, 46 evolution-harness tests, R1 shadow audit, original sr004c immutability assertion`

## Risks

- Existing project records use historical formats that must remain preserved while the active interface adopts the canonical schema.

## Blockers

- R2 mechanics/topology compatibility preflight currently erases the required H reciprocal effect; do not repair mechanics autonomously or execute Gate 7.

## Pending decisions

- Gate 7 and Gate 8 remain blocked; architect review is required after the stacked R2 PR and exact remote CI.

## Status vocabulary

Allowed adopted-project statuses are `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, and `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
