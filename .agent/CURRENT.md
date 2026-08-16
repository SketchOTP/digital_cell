# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-16T13:43:48-04:00`

## Active state after adoption

- Local directive ID: `D-20260816-dcdev010-passive-directional-substrate-traction`
- External directive ID: `DC-DEV-010`
- Objective: `Test whether one fixed passive direction-dependent substrate reaction can convert existing reserve-funded deformation into lawful body translation.`
- Current status: `VALIDATING`
- Acceptance: `DC-DEV-009 is architect-accepted at 8d6fe59397cabfa47bc1d8103acd68f544acc190; DC-DEV-010 first qualification execution failed closed at Gate 1 and architect review remains required.`
- Current phase: `DC-DEV-010 passive directional substrate coupling audit from entry 8d6fe59397cabfa47bc1d8103acd68f544acc190.`
- Expected or actual touched areas: `regulatory-core substrate traction production module, read-only contractility force audit, DC-DEV-010 assay/artifacts/docs/workflow, current governance state`
- Immediate next action: `Architect review of draft PR #19; do not tune the frozen parameters, add another traction architecture, or begin DC-DEV-011.`

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
- Exact-head remote CI run `31938648765` passed at `79751bed5ad78d367b7409f0ec677e32a3b9d527` for DC-DEV-008-R1.
- Local DC-DEV-009 audit passed with zero contractile force sum within `6.804363002006077e-16` and no valid contractility-only translation; architect accepted it at `8d6fe59397cabfa47bc1d8103acd68f544acc190`.
- DC-DEV-010 first execution used one frozen substrate parameter set. Substrate work remained non-positive, but the motor-off directional arm translated `0.013504913541228361` above tolerance `2.220446049250313e-13`.
- DC-DEV-010 scientific conclusion is `DCDEV010_DIRECTIONAL_SUBSTRATE_TRANSLATION_NOT_ESTABLISHED`; no parameter repair or second architecture is authorized.
- DC-DEV-011 remains blocked.
- DC-DEV-010 branch `strategy/dc-dev-010-directional-substrate-traction` is pushed at `83d1cd747e5ad750b2b6b2ae145c7ae4ff3444b`; draft PR #19 is open against `strategy/dc-dev-009-motility-feasibility-audit`.
- Exact-head remote CI run `31962293477` passed at `83d1cd747e5ad750b2b6b2ae145c7ae4ff3444b`; architect review remains pending.

## Blockers

- Architect review remains required for DC-DEV-010; Gate 1 failure is the active blocker.

## Pending decisions

- DC-DEV-010 may contain exactly one passive local directional substrate law. No parameter tuning, second traction architecture, new sensor, actuator, planner, reward, fitness, or evolution is authorized. DC-DEV-011 is blocked.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
