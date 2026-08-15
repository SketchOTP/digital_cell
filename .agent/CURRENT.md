# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-15T19:17:08-04:00`

## Active state after adoption

- Local directive ID: `D-20260815-dcdev004-energy-coupled-local-contractility`
- External directive ID: `DC-DEV-004`
- Objective: `Give the accepted distributed regulatory state exactly one causal, local, energetically funded physical influence on the material body.`
- Current status: `IN_PROGRESS`
- Acceptance: `DC-DEV-004 remains in bounded implementation and validation; architect review is required before any later directive.`
- Current phase: `DC-DEV-004 local contractility package from entry e4cdb8a4fd9316e51e6490fd0f833097f02be6bb.`
- Expected or actual touched areas: `bounded chemistry-core mechanics hook, regulatory-core contractility adapter, DC-DEV-004 assay/artifacts/docs/workflow, current governance state`
- Immediate next action: `Complete scoped regressions and remote draft PR validation; do not begin DC-DEV-005 or any additional actuator/sensor capability.`

## Temporary task-relevant facts

- The exact scientific base is `0d2c404c0874d5430dd5d01dbdcc059a842dd689`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- DC-DEV-003 continuity remains authoritative; DC-DEV-004 adds one local contractile tension path and does not add sensors, commands, memory, learning, or evolution.
- The frozen funding quantity is existing D-091 metabolic reserve `R` in `MaterialMesh.interior.r`; expenditure enters existing `W`.
- Entry authority is `e4cdb8a4fd9316e51e6490fd0f833097f02be6bb`.
- Implementation work is on `strategy/dc-dev-004-local-contractility`.

## Last validation after adoption

- Command or check: `Local regulatory-core suite and DC-DEV-004 gate assay from e4cdb8a4fd9316e51e6490fd0f833097f02be6bb`
- Result: `PASSED`

## Risks

- The full workspace test remains unavailable because of the pre-existing missing D-008 fixture; R1 workflow deliberately does not invoke it.
- Architect exact-head review remains required before any later directive.

## Blockers

- Exact-head remote CI and architect re-review are required before any later directive.

## Pending decisions

- None; DC-DEV-004 is authorized only for one energy-coupled local actuator. DC-DEV-005, additional actuators/sensors, learning, memory, identity, and evolution remain outside scope.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
