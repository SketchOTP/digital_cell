# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-29T12:00:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260829-post-m1-clean-capability-baseline`
- External directive ID: `DC-DEV-020-POST-M1-BASELINE-001-CLEAN-CAPABILITY-BASELINE-001`
- Objective: `Extract a clean post-M1 capability baseline from the accepted V4 closure while preserving the source provenance stack and changing no science.`
- Current status: `VALIDATING`
- Acceptance: `Baseline extraction is prepared from the pre-M1 base; exact-head Linux validation and Architect acceptance remain pending.`
- Current phase: `Post-M1 clean capability baseline validation; M1 remains closed and frozen.`
- Expected or actual touched areas: `baseline branch, retained V4 runtime/capabilities, compact manifests, current governance, scoped CI`
- Immediate next action: `Run local and exact-head Linux baseline validation, archive compact evidence, and return for Architect review; do not begin M2.`

## Temporary task-relevant facts

- M0 is closed; M1 is formally closed and frozen at `fb77f472b1519a9e0f713833efba5b1d403f4723`.
- Production selection is `MaturationCoupledV4` with reserve OFF; M2 is not yet authorized.
- PR #44 is the immutable M1 provenance stack and must remain open, draft, unmerged, and untouched.
- The clean baseline is derived from `1e242f28152797b512e25cd56c7b718e45d6ca97`; accepted M1 evidence remains on Atlas.
- The accepted first implementation contract is observer-coupled and exposes no effector or motor output.
- DC-DEV-003 continuity remains authoritative; DC-DEV-004 adds one local contractile tension path and does not add sensors, commands, memory, learning, or evolution.
- The frozen funding quantity is existing D-091 metabolic reserve `R` in `MaterialMesh.interior.r`; expenditure enters existing `W`.
- Entry authority is `8d6fe59397cabfa47bc1d8103acd68f544acc190`.
- DC-DEV-007 is architect-accepted; its active contact chain remains preserved and is exercised by the DC-DEV-008 preservation workflow.
- DC-DEV-010 / PR #19 is closed, unmerged negative evidence and must not be imported.
- Implementation work is on `strategy/dc-dev-016-metabolic-break-even` based on `strategy/dc-dev-015-metabolic-restoration-audit`.
- DC-DEV-015 starts exactly at `5a4e0a2d7314af411ec2283b0ffcf4950eb217db` from `strategy/dc-dev-013-resource-contact-feeding`; DC-DEV-014/PR #23 is closed, unmerged negative evidence and is not imported.

## Last validation after adoption

- Command or check: `Local Rust 1.89.0 DC-DEV-016 example check/run and evidence inspection`
- Result: `PASSED`

## Risks

- Linux is the target runtime; canonical dense evidence is stored under `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/`.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.

## Blockers

- Exact-head baseline CI and Architect acceptance remain open; no M2 implementation or M1 reopening is authorized.

## Pending decisions

- The V4 equations, D-087 boundary, conservation contracts, and accepted M1 qualification are frozen; only baseline integration is in scope.
- M2 autonomous spatial resource acquisition is pending Architect baseline acceptance; no parameter search, reserve, recycling, salvage, or scientific change is authorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
