# CURRENT.md

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-14T20:30:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260814-d096-repair-gain-specificity-audit`
- External directive ID: `DC-SR-004C-R3`
- Objective: `Classify the D-096 repair-gain scope defect using observer attribution and one fixed shadow counterfactual without repairing or executing Gate 7`
- Current status: `VALIDATING`
- Acceptance: `Observer-only current-path attribution and fixed non-authoritative shadow complete; no production repair; Gate 7 and Gate 8 remain blocked`
- Current phase: `DC-SR-004C-R3 repair-gain specificity audit`
- Expected or actual touched areas: `observer-only structural-build ledgers, isolated shadow mode, R3 audit example, sr004cr3 artifacts, repair_gain_specificity docs, scoped CI, .agent records`
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

- R2 mechanics/topology compatibility erased the required H reciprocal effect; R3 classifies broad repair-gain application as a bounded implementation-defect candidate. Do not repair production biology or execute Gate 7.

## Pending decisions

- Gate 7 and Gate 8 remain blocked; architect review is required after the stacked R3 PR and exact remote CI.

## Status vocabulary

Allowed adopted-project statuses are `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, and `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
