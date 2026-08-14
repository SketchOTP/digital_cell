# CURRENT.md

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-14T17:34:49-04:00`

## Active state after adoption

- Local directive ID: `D-20260814-1734-gate7-parity-horizon-audit`
- External directive ID: `DC-SR-004C-R1`
- Objective: `Audit Gate 5 physiology parity and D-088 horizon transfer for preserved Gate 7 execution without rerunning or tuning Gate 7`
- Current status: `COMPLETE`
- Acceptance: `Required shadow artifacts and documentation exist, original sr004c evidence is unchanged, exact Gate 7 metadata is corrected, and one allowed audit conclusion is supported by local and scoped remote validation`
- Current phase: `DC-SR-004C-R1 shadow audit`
- Expected or actual touched areas: `evolution-harness audit example or tests, sr004cr1 artifacts, d096_gate7 parity_audit docs, .agent records, scoped CI if required`
- Immediate next action: `Pause for architect review; do not rerun Gate 7 or begin Gate 8`

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

- Gate 5-to-Gate 7 physiology parity failed in the shadow audit; no adapter repair or Gate 7 rerun is authorized without a new directive.

## Pending decisions

- Do not rerun Gate 7 or begin Gate 8; architect review remains required after scoped CI and the exact R1 conclusion.

## Status vocabulary

Allowed adopted-project statuses are `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, and `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
