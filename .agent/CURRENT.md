# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-16T14:15:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260816-dcdev010r1-mechanical-rest-causal-isolation`
- External directive ID: `DC-DEV-010-R1`
- Objective: `Determine whether the accepted DC-DEV-010 Gate 1 motor-off translation was passive rectification of unrelaxed seeded-body mechanics, then retest only from a preregistered settled common state.`
- Current status: `VALIDATING`
- Acceptance: `DC-DEV-010 negative result is architect-accepted; DC-DEV-010-R1 failed closed at baseline mechanical-rest Gate 1 and architect review remains required.`
- Current phase: `DC-DEV-010-R1 mechanical-rest causal isolation from entry b4178417e30907835183c7f9c16a639bdd8d31db.`
- Expected or actual touched areas: `observer-only DC-DEV-010-R1 settlement assay, separate evidence, scoped workflow, documentation, and governance; no production behavior change`
- Immediate next action: `Architect review of draft PR #19 at the pushed R1 head; do not tune the frozen parameters, add another substrate, or begin DC-DEV-011.`

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
- DC-DEV-010-R1 baseline rest did not satisfy the preregistered local/global convergence contract within 5,000 accepted steps; chemistry/resource state hash remained unchanged.
- DC-DEV-010-R1 conclusion is `DCDEV010R1_BASELINE_MECHANICAL_REST_NOT_ESTABLISHED`; no matched R1 arms executed.
- DC-DEV-010 branch `strategy/dc-dev-010-directional-substrate-traction` is pushed at `89a57f68253af431a80f8b66dc7e626a0846b5de`; draft PR #19 is open against `strategy/dc-dev-009-motility-feasibility-audit`.
- Exact-head remote CI for the R1 head is pending; local governance validation and R1 assay execution passed/failed closed as specified.

## Blockers

- Architect review remains required for DC-DEV-010-R1; baseline mechanical rest is the active blocker.

## Pending decisions

- DC-DEV-010-R1 may only isolate startup mechanical relaxation using the existing law and parameters. No parameter tuning, second substrate, adhesion, anchoring, sensor, planner, reward, fitness, evolution, or DC-DEV-011 is authorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
