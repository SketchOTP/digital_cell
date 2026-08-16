# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-15T20:33:20-04:00`

## Active state after adoption

- Local directive ID: `D-20260815-dcdev005-local-experience-dependent-plasticity`
- External directive ID: `DC-DEV-005`
- Objective: `Make prior local experience change a later local physical response through exactly one slow local adaptation trace.`
- Current status: `VALIDATING`
- Acceptance: `DC-DEV-005 is authorized from accepted DC-DEV-004 head; qualification and architect review remain pending.`
- Current phase: `DC-DEV-005 local history-dependent plasticity from entry edf517e6b802a7cd9cf141980061127dbb697b21.`
- Expected or actual touched areas: `regulatory-core local plasticity adapter, DC-DEV-005 assay/artifacts/docs/workflow, current governance state`
- Immediate next action: `Publish the locally passing assay and scoped regression evidence, obtain exact-head remote CI, and await architect review; do not begin DC-DEV-006.`

## Temporary task-relevant facts

- The exact scientific base is `0d2c404c0874d5430dd5d01dbdcc059a842dd689`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- DC-DEV-003 continuity remains authoritative; DC-DEV-004 adds one local contractile tension path and does not add sensors, commands, memory, learning, or evolution.
- The frozen funding quantity is existing D-091 metabolic reserve `R` in `MaterialMesh.interior.r`; expenditure enters existing `W`.
- Entry authority is `edf517e6b802a7cd9cf141980061127dbb697b21`.
- Implementation work is on `strategy/dc-dev-005-local-plasticity`.

## Last validation after adoption

- Command or check: `DC-DEV-005 gate assay, scoped regressions, governance validation, and exact-head GitHub Actions run 31917550450 at 9fe97069185ac48d4e979fe358b12d32433eb6d7`
- Result: `PASSED`

## Risks

- The full workspace test remains unavailable because of the pre-existing missing D-008 fixture; R1 workflow deliberately does not invoke it.
- Architect exact-head review remains required before any later directive.

## Blockers

- Architect review remains required before any later directive; exact-head GitHub Actions run 31917550450 passed.

## Pending decisions

- None; DC-DEV-005 is authorized only for one slow local plasticity trace. DC-DEV-006, additional traces, sensors, actuators, reward, fitness, learning optimizers, identity, and evolution remain outside scope.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
