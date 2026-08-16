# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-16T01:15:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260816-dcdev008-finite-spatial-resource-acquisition`
- External directive ID: `DC-DEV-008`
- Objective: `Prove finite local environmental N/F material enters the existing metabolic pathway and supports internal A/R state.`
- Current status: `IN_PROGRESS`
- Acceptance: `DC-DEV-007 is architect-accepted at 2968882769991f48c987ceb40c719fd351b2e046; DC-DEV-008-R1 production boundary is complete at 9872d4e251817177989a980760796a8ba767d037 and exact-head remote CI run 31938214782 passed; architect review is required.`
- Current phase: `DC-DEV-008 finite spatial resource acquisition from entry 2968882769991f48c987ceb40c719fd351b2e046.`
- Expected or actual touched areas: `regulatory-core spatial_resource production module/tests, DC-DEV-008 assay/artifacts/docs/workflow, current governance state`
- Immediate next action: `Await independent architect review of PR #17 at exact head 9872d4e251817177989a980760796a8ba767d037; do not begin DC-DEV-009.`

## Temporary task-relevant facts

- The exact scientific base is `0d2c404c0874d5430dd5d01dbdcc059a842dd689`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- DC-DEV-003 continuity remains authoritative; DC-DEV-004 adds one local contractile tension path and does not add sensors, commands, memory, learning, or evolution.
- The frozen funding quantity is existing D-091 metabolic reserve `R` in `MaterialMesh.interior.r`; expenditure enters existing `W`.
- Entry authority is `2968882769991f48c987ceb40c719fd351b2e046`.
- DC-DEV-007 is architect-accepted; its active contact chain remains preserved and is exercised by the DC-DEV-008 preservation workflow.
- Implementation work is on `strategy/dc-dev-008-spatial-resource-acquisition` stacked on `strategy/dc-dev-007-active-contact-regulation`.

## Last validation after adoption

- Command or check: `DC-DEV-008-R1 exact-head scoped preservation workflow`
- Result: `PASSED`

## Risks

- The full workspace test remains unavailable because of the pre-existing missing D-008 fixture; the scoped workflow deliberately does not invoke it.
- DC-DEV-008-R1 retains the accepted evidence values and closes the runtime-boundary defect by making regulatory-core the sole resource implementation.
- Exact-head remote CI run `31938214782` passed at `9872d4e251817177989a980760796a8ba767d037`.
- Architect exact-head review remains required before any later directive; DC-DEV-009 remains blocked.

## Blockers

- Architect review remains required; DC-DEV-009 is not authorized.

## Pending decisions

- None; DC-DEV-008 is authorized only as a finite local N/F environment adapter over existing permeability, metabolism, A, and reserve R. New species, global transport changes, sensors, actuators, traces, planner, reward, fitness, and evolution remain outside scope.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
