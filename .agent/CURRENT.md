# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-29T15:42:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260829-dcdev021-m2-entry001-r1-d087-preservation-harness-repair`
- External directive ID: `DC-DEV-021-M2-ENTRY-001-R1-D087-PRESERVATION-HARNESS-REPAIR-001`
- Objective: `Repair only the D-087 selector harness, replay the canonical V2/V3/V4 boundary on baseline and M2 heads, and finalize existing ENTRY-001 evidence.`
- Current status: `COMPLETE`
- Acceptance: `M1 remains Architect-closed and frozen. ENTRY-001-R1 exact-head Linux validation passed at 39901a0 with canonical baseline/M2 D-087 parity, actuator qualification, and all required downstream preservation.`
- Current phase: `M2 ENTRY-001-R1 preservation-harness repair; autonomous resource acquisition is not established.`
- Expected or actual touched areas: `scoped workflow, compact evidence, and current governance only`
- Immediate next action: `Await Architect acceptance; do not begin resource-contact sensing/acquisition or successor M2 work.`

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
- The baseline CI `33271509819` and ENTRY-001 CI `33278000813` D-087 sub-runs are invalid preservation-harness evidence: both passed `MaturationCoupledV4` to the chemistry selector, which accepts only HistoricalV1, ConservativeV2, or ConservativeV3 and otherwise resolves to HistoricalV1. Canonical V4 selection is `DCDEV020R9R3_CONTRACT=ConservativeV3`, `DCDEV020R9R3_RESERVE=0`, and `DCDEV020M1REPLAN002R1_V4=1`.
- Autonomous resource acquisition, chemotaxis, target/gradient logic, reserve enablement, and M2 successor work remain unauthorized.

## Last validation after adoption

- Command or check: `Exact-head Linux post-M1 clean baseline workflow 33271104939 at 5e28762d0757bfa23b91820115b5893d0ef6d82a`
- Result: `PENDING`
- Current command: `Exact-head Linux ENTRY-001-R1 workflow 33282801415 — SUCCESS at 39901a01ce17f42826351278a4321e65b1a99780; artifact sha256:74a537d379b5ebd9d50c72daa1e09fd604ff3e1c8c4b11c2f61138d01e22d72f`

## Risks

- Linux is the target runtime; canonical dense evidence is stored under `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/`.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.

## Blockers

- No scientific blocker is known. ENTRY-001 actuator qualification is complete pending Architect acceptance; autonomous resource acquisition remains unestablished.

## Pending decisions

- The V4 equations, D-087 boundary, conservation contracts, and accepted M1 qualification are frozen; only baseline integration is in scope.
- M2 autonomous spatial resource acquisition is pending Architect baseline acceptance; no parameter search, reserve, recycling, salvage, or scientific change is authorized.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
