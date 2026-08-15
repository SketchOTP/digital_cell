# CURRENT.md

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-15T00:00:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260814-digital-cell-d096-repair-gain-scope-bounded-repair`
- External directive ID: `DC-SR-004C-R4`
- Objective: `Implement exactly the R3-preregistered D-096 finite-allocation structural-build repair and requalify the impacted Gate 5 without executing Gate 7`
- Current status: `VALIDATING`
- Acceptance: `One production D-096 structural-build repair, immutable R3 shadow equivalence, impacted Gate 5 requalification, and no Gate 7/Gate 8 execution`
- Current phase: `DC-SR-004C-R4 bounded repair and impacted Gate 5 requalification`
- Expected or actual touched areas: `D-096 structural-build production branch, focused chemistry regressions, R4 verifier, sr004cr4 artifacts, repair_gain_scope_fix docs, scoped CI, .agent records`
- Immediate next action: `Push stacked draft PR and obtain architect review of SR004CR4_REPAIR_INVALIDATES_GATE5; do not execute Gate 7 or Gate 8`

## Temporary task-relevant facts

- The canonical Authority repository is reference-only and must not be modified.
- Existing Digital Cell governance snapshots are preserved under `.agent/legacy/pre-authority-migration-20260814/`.
- The original Gate 7 artifacts under `digital-protocell/experiments/generated/sr004c/` are immutable evidence for this audit.

## Last validation after adoption

- Command or check: `Atlas release focused suites; GitHub Actions run 31845154445 on b258126fb2ac1373515a09711d7dcaa07022550f`
- Result: `R4 shadow equivalence PASSED 72/72 with max residual 2.84217094304040074e-14; H requalified; original B Gate 5 criterion failed in all eight seeds; evolution-harness 46/46`

## Risks

- Existing project records use historical formats that must remain preserved while the active interface adopts the canonical schema.

## Blockers

- The single R4 repair reproduces the immutable R3 shadow, but corrected production invalidates the original Gate 5 B criterion. Do not tune, add another physiology repair, or execute Gate 7.

## Pending decisions

- Gate 7 and Gate 8 remain blocked; architect review is required after the stacked R4 PR and exact remote CI.

## Status vocabulary

Allowed adopted-project statuses are `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, and `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
