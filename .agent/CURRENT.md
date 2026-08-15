# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-15T16:00:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260815-dcdev001a-architecture-selection`
- External directive ID: `DC-DEV-001A`
- Objective: `Establish the clean scientific base and select exactly one developmental/sensorimotor architecture without modifying production organism biology.`
- Current status: `VALIDATING`
- Acceptance: `Architecture package complete; final validation, commit, push, and draft PR are pending independent architect review.`
- Current phase: `DC-DEV-001A architecture selection package and final validation`
- Expected or actual touched areas: `governance carry-forward, developmental/sensorimotor strategy documentation, dcdev001 machine-readable artifacts, no production organism source`
- Immediate next action: `Complete validation, publish the branch, and request architect review.`

## Temporary task-relevant facts

- The exact scientific base is `0d2c404c0874d5430dd5d01dbdcc059a842dd689`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The selected conclusion is `DCDEV001_HYBRID_COMPOSITION_SELECTED`.
- The full workspace test is blocked by a pre-existing missing D-008 generated fixture; focused clean-base tests pass.

## Last validation after adoption

- Command or check: `python3 scripts/validate_governance.py --mode ADOPTED; cargo test -p chemistry-core --test d088_tests; cargo test -p phase1-certifier --test metrics_semantics`
- Result: `PASSED; 4 + 4 focused tests passed`

## Risks

- The full workspace test remains unavailable until the clean-base D-008 fixture gap is resolved outside this directive.
- External source licenses must be rechecked before any future source or dependency reuse.

## Blockers

- No architecture-selection blocker. Independent architect review is required before any future implementation directive.

## Pending decisions

- None for DC-DEV-001A; future implementation requires a separate directive.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
