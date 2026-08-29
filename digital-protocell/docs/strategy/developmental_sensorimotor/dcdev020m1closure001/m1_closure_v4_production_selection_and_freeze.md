# DC-DEV-020 M1 closure: V4 production selection and freeze

Directive: `DC-DEV-020-M1-CLOSURE-001-V4-PRODUCTION-SELECTION-AND-FREEZE-001`

This document records the bounded selector authority for the M1 closure
candidate. It does not alter any V4 equation, coefficient, observer rule, or
D-087 threshold.

## Authority

- Closure starting head: `d011543f58fa684d04c97c0cccba9f7202957546`
- Accepted R5-R4 head: `c56cf3791fc17e85073f6b1ed13cf827353ca3da`
- Selector change commit: `7720546ccc75a7d5becb1dc6eac8a063e0d75b8c`
- Adopted project goal: `.agent/PROJECT_GOAL.md`
- Project-goal blob SHA at closure execution head: `94864614625a8f3388a51b33b79ad203b2f66588`
- Target production contract: `MaturationCoupledV4`
- Reserve: `OFF`
- PR #44: preserve as open/draft/unmerged development provenance
- Formal Architect M1 closure: pending this exact-head candidate package

## Selector inventory

| Location | Role | Closure disposition |
| --- | --- | --- |
| `crates/phase1-certifier/src/sim.rs::seed_mesh` | historical experiment/certifier seed | preserved; existing explicit selectors remain available |
| `crates/phase1-certifier/src/sim.rs::seed_production_mesh` | single fresh production selector | default now stamps `MaturationCoupledV4`; explicit V1/V2/V3/V4 overrides remain supported |
| `crates/phase1-certifier/src/bin/phase1_runtime.rs` | packaged Linux runtime entrypoint | uses `seed_production_mesh`; reports the actual mesh contract |
| `crates/phase1-certifier/src/sim.rs::selected_mesh_schema` | explicit experiment/config selector | preserved for historical replay and explicit runtime requests |
| `crates/phase1-certifier/src/sim.rs::reserve_enabled` | reserve selector | default remains false; only explicit opt-in enables reserve |
| `crates/phase1-certifier/src/bin/phase1_certification.rs` | D-087 certifier | preserved; uses explicit contract environment for V2/V3/V4 qualification |
| existing R6–R5-R4 examples | historical/qualification harnesses | preserved; no production-default migration |
| serialized `MaterialMesh::contract_version` | save/load authority | preserved; historical snapshots retain their stored identity |

## Explicit contract behavior

The standalone production runtime has one default path: with no contract or
reserve environment override it creates `MaturationCoupledV4` with reserve
disabled. Explicit `DCDEV020R9R3_CONTRACT=ConservativeV2` and
`DCDEV020R9R3_CONTRACT=ConservativeV3` continue to select the historical
contracts for controlled replay; the V4 opt-in remains explicit for existing
qualification workflows. `seed_mesh` is intentionally not redefined, so
historical certifiers and evidence do not silently migrate.

## Evidence boundary

Compact closure evidence is under
`experiments/generated/dcdev020m1closure001/`. Dense ledgers remain on the
canonical Atlas evidence root:
`/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1closure001/`.

V4's historical D-087 result remains the frozen `[true, true, false, true,
true, true, true, true]` / `7/8` vector. Contract-aware V4 preservation is
recorded separately and is not a threshold change or a relabeling of the
historical certifier result.

## Final downstream preservation

The finalization run explicitly executes the existing canonical runners. All
six results pass and each runner reports `next_execution_started: false`:

| Capability | Directive | Runner | Result |
| --- | --- | --- | --- |
| Distributed regulator | DC-DEV-002 | `regulatory-core::dcdev002_gate_assay` | `DCDEV002_LOCAL_REGULATORY_SUBSTRATE_QUALIFIED` |
| History-dependent plasticity | DC-DEV-005 | `regulatory-core::dcdev005_gate_assay` | `DCDEV005_LOCAL_HISTORY_DEPENDENT_PLASTICITY_QUALIFIED` |
| Spatial contact environment | DC-DEV-006 | `regulatory-core::dcdev006_gate_assay` | `DCDEV006_SPATIAL_CONTACT_ENVIRONMENT_QUALIFIED` |
| External contact regulation | DC-DEV-007 | `regulatory-core::dcdev007_gate_assay` | `DCDEV007_ACTIVE_EXTERNAL_CONTACT_REGULATION_QUALIFIED` |
| Finite spatial resource infrastructure | DC-DEV-008 | `regulatory-core::dcdev008_gate_assay` | `DCDEV008_SPATIAL_RESOURCE_ACQUISITION_QUALIFIED` |
| Passive isotropic traction | DC-DEV-011 | `regulatory-core::dcdev011_gate_assay` | `DCDEV011_PASSIVE_ISOTROPIC_STICK_SLIP_TRANSLATION_QUALIFIED` |

D-088, D-091, evolution-harness, geometry/material conservation, lifecycle,
and forbidden-controller checks remain explicit parts of the closure workflow.

## R1 finalization boundary

M1 science is satisfied and `MaturationCoupledV4 / reserve OFF` is selected.
The closure evidence is complete pending Architect acceptance. The formal
acceptance flag remains pending; M2 remains unauthorized and PR #44 remains
open, draft, and unmerged.

## Freeze boundary

If the exact-head closure candidate passes, the M1 physiology is
`MaturationCoupledV4 / reserve OFF`, frozen pending Architect acceptance.
M2 remains unauthorized; no resource seeking, ecology, reproduction, or
additional M1 search is started by this directive.
