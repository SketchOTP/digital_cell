# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-15T23:34:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260816-dcdev008-finite-spatial-resource-acquisition`
- External directive ID: `DC-DEV-008`
- Objective: `Give the existing organism finite, local, mass-conservative access to existing N/F material and demonstrate coupling into existing metabolism.`
- Current status: `IN_PROGRESS`
- Acceptance: `DC-DEV-007 is architect-accepted at 2968882769991f48c987ceb40c719fd351b2e046; DC-DEV-008 Gates 0-8, exact-head remote CI, and architect review remain pending.`
- Current phase: `DC-DEV-008 finite spatial resource acquisition from entry 2968882769991f48c987ceb40c719fd351b2e046.`
- Expected or actual touched areas: `chemistry-core local resource-region transport, DC-DEV-008 assay/artifacts/docs/workflow, current governance state`
- Immediate next action: `Implement and validate the bounded DC-DEV-008 assay; do not start DC-DEV-009.`

## Temporary task-relevant facts

- The exact scientific base is `0d2c404c0874d5430dd5d01dbdcc059a842dd689`.
- The DC-DEV-008 entry authority is `2968882769991f48c987ceb40c719fd351b2e046`.
- R4/D-096 source remains isolated in the other worktree and is not an input to this branch.
- Later append-only governance snapshots are preserved under `.agent/legacy/`.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- DC-DEV-003 continuity remains authoritative; DC-DEV-004 adds one local contractile tension path and does not add sensors, commands, memory, learning, or evolution.
- The frozen funding quantity is existing D-091 metabolic reserve `R` in `MaterialMesh.interior.r`; expenditure enters existing `W`.
- Entry authority is `3a5971be332f94848250196e8148b722464066f2`.
- Implementation work is on `strategy/dc-dev-007-active-contact-regulation` stacked on `strategy/dc-dev-006-spatial-contact-environment`.

## Last validation after adoption

- Command or check: `DC-DEV-007 local Gates 0-8 assay`
- Result: `PASSED`

## Risks

- The full workspace test remains unavailable because of the pre-existing missing D-008 fixture; R1 workflow deliberately does not invoke it.
- Architect exact-head review remains required before any later directive; remote run 31924373883 passed at the exact PR head.

## Blockers

- Mimir V2 tools are unavailable in this session; lifecycle evidence must be reported as blocked, not fabricated.

## Pending decisions

- None; DC-DEV-008 is authorized only as a finite local N/F resource-region extension over the accepted organism and existing transport/metabolism. DC-DEV-009 remains prohibited.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
