# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-29T15:42:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260829-dcdev021-m2-entry001-activated-energy-contractility`
- External directive ID: `DC-DEV-021-M2-ENTRY-001-ACTIVATED-ENERGY-CONTRACTILITY-FEASIBILITY-001`
- Objective: `Qualify one explicit V4 A-funded contractility path through the existing local activity and stick-slip traction APIs without changing production selection.`
- Current status: `IN_PROGRESS`
- Acceptance: `M1 is Architect-closed and frozen; M2 ENTRY-001 actuator feasibility is active and Architect acceptance remains pending.`
- Current phase: `M2 ENTRY-001 activated-energy contractility feasibility; autonomous resource acquisition is not established.`
- Expected or actual touched areas: `regulatory-core additive actuator APIs, focused assay/evidence, scoped Linux CI, current governance`
- Immediate next action: `Run and verify the exact-head ENTRY-001 assay; do not begin resource-contact sensing or acquisition.`

## Temporary task-relevant facts

- M0 is closed; M1 is formally closed and frozen at `fb77f472b1519a9e0f713833efba5b1d403f4723`.
- Production selection is `MaturationCoupledV4` with reserve OFF; M2 is active only under ENTRY-001.
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
- M2 ENTRY-001 starts from `baseline/m1-v4-closed` at `d76481c785e9eec361df3fa0cd03c512b521639c`.
- The opt-in `ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1` path is V4-only, reuses frozen DC-DEV-004 constants, spends absolute A into W after accepted mechanics, and leaves R-funded APIs unchanged.
- Autonomous resource acquisition, chemotaxis, target/gradient logic, reserve enablement, and M2 successor work remain unauthorized.

## Last validation after adoption

- Command or check: `Exact-head Linux post-M1 clean baseline workflow 33271104939 at 5e28762d0757bfa23b91820115b5893d0ef6d82a`
- Result: `PENDING`
- Current command: `M2 ENTRY-001 exact-head Linux workflow; no run yet`

## Risks

- Linux is the target runtime; canonical dense evidence is stored under `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/`.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.

## Blockers

- Architect acceptance of the finalized baseline remains open; no M2 implementation or M1 reopening is authorized.

## Pending decisions

- The V4 equations, D-087 boundary, conservation contracts, and accepted M1 qualification are frozen; only baseline integration is in scope.
- M2 autonomous spatial resource acquisition is pending Architect baseline acceptance; no parameter search, reserve, recycling, salvage, or scientific change is authorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
