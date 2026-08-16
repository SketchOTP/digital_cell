# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-15T22:46:41-04:00`

## Active state after adoption

- Local directive ID: `D-20260815-dcdev007-active-external-contact-regulation`
- External directive ID: `DC-DEV-007`
- Objective: `Prove that existing local contact sensing, regulation, adaptation, reserve-funded contractility, and mechanics alter future external contact.`
- Current status: `IN_PROGRESS`
- Acceptance: `DC-DEV-006 is architect-accepted at 3a5971be332f94848250196e8148b722464066f2; DC-DEV-007 local Gates 0-8 pass and exact-head remote CI plus architect review remain pending.`
- Current phase: `DC-DEV-007 active external-contact regulation from entry 3a5971be332f94848250196e8148b722464066f2.`
- Expected or actual touched areas: `regulatory-core assay registration, DC-DEV-007 assay/artifacts/docs/workflow, current governance state`
- Immediate next action: `Run the scoped DC-DEV-007 preservation suite, push strategy/dc-dev-007-active-contact-regulation, open a stacked draft PR, and await exact-head remote CI and architect review; do not begin DC-DEV-008.`

## Temporary task-relevant facts

- The exact scientific base is `0d2c404c0874d5430dd5d01dbdcc059a842dd689`.
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
- Architect exact-head review remains required before any later directive; remote run 31923037384 passed at the exact PR head.

## Blockers

- Exact-head remote CI and architect review remain required; DC-DEV-008 is not authorized.

## Pending decisions

- None; DC-DEV-007 is authorized only as an integration qualification over the accepted static obstacle, contact signal, distributed regulator, existing adaptation trace, D-091 reserve-funded contractility, and mechanics. Additional sensors, actuators, traces, world primitives, reward, fitness, identity, evolution, and DC-DEV-008 remain outside scope.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
