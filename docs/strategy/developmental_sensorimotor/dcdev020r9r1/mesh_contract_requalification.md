# DC-DEV-020-R9-R1 — Mesh contract requalification

Entry authority: `22529ca0caa570e1603c28fe39b05786052b969e`
Clean production base: `1e242f28152797b512e25cd56c7b718e45d6ca97`
Branch: `strategy/dc-dev-020r9-mesh-contract-requalification`
Pull request: `#44` (draft, unmerged)

## Scope

This is an observer-only requalification. `MeshContractVersion::ConservativeV2`
selects strict physical/material accounting and observer-only death semantics;
it does not replace the biological equation identity. D-091 therefore retains
`autopoietic_material_mesh_metabolic_reserve_v1` and loads its reserve schema
under the v2 contract.

No new chemistry, source law, sink law, transport law, controller, behavior,
evolution, or DC-DEV-021 work is included.

## Evidence

- `experiments/generated/dcdev020r9r1/mesh_contract/manifest.json` contains the
  compact R9-R1 report and classification.
- `d087_gate_matrix.json` records reserve-bearing D-087 Gates 0–7. All eight
  gates pass locally; Gate 7 records observer nonviability while the physical
  `alive` field remains unchanged.
- `../exact_replays/manifest.json` and `results.json` contain seven exact
  D-015/D-016 rows. Strict material equals delivered N+F in every row, with
  zero closure residual and zero rejected reserve steps.
- `../r8r2_exact/` contains the exact R8-R2 machinery replayed in explicit
  ConservativeV2 compatibility mode. Its compact manifest records the v2
  contract and unchanged D-091 equation lineage; dense frames remain governed
  external evidence.
- `../r8r4_exact/` contains the exact R8-R4 shared-affinity/autogenous-Cprod
  machinery replayed under the same ConservativeV2 compatibility mode. The
  replay is independently qualified as a negative bounded result; its dense
  8,000-step traces remain governed external evidence.

The historical `dcdev020r9/` evidence is preserved and not overwritten. The
legacy R9 E5 rows remain proxy diagnostics and are not presented as exact
historical protocol replays.

## Status

Local sanctioned Rust 1.89.0 execution passes the focused contract, D-091,
R9-R1 gate, and exact replay checks. Exact-head remote CI and architect review
are pending. `DC-DEV-021` is not authorized and no next execution has started.
