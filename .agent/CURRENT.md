# Current State

## Lifecycle

- Status: `ADOPTED`
- Last updated: `2026-08-30T00:40:00-04:00`

## Active state after adoption

- Local directive ID: `D-20260830-dcdev021-m2-entry005-refractory-motor-decoupling`
- External directive ID: `DC-DEV-021-M2-ENTRY-005-REFRACTORY-MOTOR-DECOUPLING-FEASIBILITY-001`
- Objective: `Test one opt-in composition where frozen local adaptation remains refractory inside ENTRY-003 intrinsic dynamics while raw intrinsic activity drives the accepted A-funded motor.`
- Current status: `COMPLETE`
- Acceptance: `Exact-head Linux workflow 33292817570 passed at e8349df8c1ef839b23e97d4bfe7c5b75b00b0b5a with M2_REFRACTORY_MOTOR_DECOUPLING_EXPLORATION_QUALIFIED. The uploaded compact artifact has sha256:654af0ac349ca13662416593bb152f581672de370eca75fcbdc382a458c80194. Architect review remains pending.`
- Current phase: `M2 intrinsic regulatory dynamics and target-free retained exploration are qualified; autonomous resource acquisition remains not established.`
- Expected or actual touched areas: `new opt-in refractory-only motor composition in intrinsic_exploration, focused tests/example, compact evidence, scoped workflow, and governance only`
- Immediate next action: `Awaiting reset and Architect review; do not add resource sensing, chemotaxis, targets, gradients, tuning, or successor work.`

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
- M2 ENTRY-001 starts from `baseline/m1-v4-closed` at `d76481c785e9eec361df3fa0cd03c512b521639c`; ENTRY-002 starts at accepted ENTRY-001 head `54f3af09804a9accd845dfcae2dfce13d1918b7c`; ENTRY-003 starts at accepted ENTRY-002 head `2ed0f6159b0169f1f7bd9c2c10e89a6b67d12167`.
- The opt-in `ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1` path is V4-only, reuses frozen DC-DEV-004 constants, spends absolute A into W after accepted mechanics, and leaves R-funded APIs unchanged.
- The baseline CI `33271509819` and ENTRY-001 CI `33278000813` D-087 sub-runs are invalid preservation-harness evidence: both passed `MaturationCoupledV4` to the chemistry selector, which accepts only HistoricalV1, ConservativeV2, or ConservativeV3 and otherwise resolves to HistoricalV1. Canonical V4 selection is `DCDEV020R9R3_CONTRACT=ConservativeV3`, `DCDEV020R9R3_RESERVE=0`, and `DCDEV020M1REPLAN002R1_V4=1`.
- ENTRY-002 exact-head evidence classifies the direct A-funded instantaneous-contact route as still negative and resource-independent exploration as not established. This is an audit result only; it does not authorize a memory, sensor, or navigation implementation.
- ENTRY-003 adds only the explicit opt-in `INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1`. It uses one seed of exactly `FROZEN_K_STIMULUS * FROZEN_DT`, frozen neighbor/self-excitation/decay dynamics, and existing local adaptation; it reads no resource, world, target, gradient, observer, or viability state.
- ENTRY-003 is Architect accepted as `M2_INTRINSIC_EXPLORATION_MECHANICALLY_INSUFFICIENT`: it records intrinsic activity switching and exact A-to-W closure with R unchanged, but no retained material-centroid displacement beyond frozen controls.
- ENTRY-004's uninterrupted clone reconstruction has exact clutch-ledger parity. It identifies adaptation attenuation, not a traction-law mismatch, as the cause of zero intrinsic slip events; no actuation amplification or adaptation change is authorized.
- ENTRY-005 is opt-in `INTRINSIC_EXPLORATION_REFRACTORY_MOTOR_SCHEMA_V1`: it retains the exact ENTRY-003 adaptation-inhibited excitation/update equations, but sends raw `activity_after` rather than `activity_after * (1 - adaptation)` to the already-qualified A-funded motor. It adds no numerical parameter, sensor, target, gradient, resource input, or reserve funding.
- Autonomous resource acquisition, chemotaxis, target/gradient logic, reserve enablement, and M2 successor work remain unauthorized.

## Last validation after adoption

- Command or check: `Exact-head ENTRY-004 Linux workflow 33288018734 plus local focused regulatory-core and canonical D-087 preservation replay`
- Result: `PASSED`

## Risks

- Linux is the target runtime; canonical dense evidence is stored under `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/`.
- The frozen substrate remains local, isotropic, passive, and reaction-only; no DC-DEV-010 directional substrate code may enter this branch.
- DC-DEV-012 is closed as valid negative evidence and is not imported.

## Blockers

- Architect review is required before any successor directive.

## Pending decisions

- The V4 equations, D-087 boundary, conservation contracts, accepted M1 qualification, and ENTRY-001 actuator are frozen.
- M2 autonomous spatial resource acquisition remains pending; no parameter search, reserve, recycling, salvage, sensor, target, gradient, resource seeking, or successor execution is authorized pending Architect review.

## Status vocabulary

Allowed adopted-project statuses: `IDLE`, `PLANNING`, `IN_PROGRESS`, `VALIDATING`, `BLOCKED`, `COMPLETE`. `CURRENT.md` is mutable and never replaces historical ledgers. Reset it to `IDLE` when an adopted task closes.
