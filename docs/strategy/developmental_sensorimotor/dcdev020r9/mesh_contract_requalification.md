# DC-DEV-020-R9 — Mesh contract requalification

Entry authority: `600bc8bef735a6be4b019a65263b023b2bada48a`
Clean production base: `1e242f28152797b512e25cd56c7b718e45d6ca97`
Branch: `strategy/dc-dev-020r9-mesh-contract-requalification`

## Result

The current historical mesh schema returns `NO_POSITIVE_CONSERVATION_VECTOR`. The versioned `material_mesh_stoichiometry_v2_conservative` schema returns `POSITIVE_CONSERVATION_VECTOR_EXISTS` with unit positive material weights across the audited pools. Descriptor/runtime parity is asserted for all fourteen audited reaction columns.

The v2 runtime closes the strict organism-material ledger to approximately `2.84e-14` in the internal no-boundary run. The E2 activation store (`F+A+R`) falls by approximately `0.62447` while organized retained material rises by approximately `0.15502`; these are separate ledgers, so `E_AR` is not treated as organism-level homeostasis.

The v2 death classification is observer-only. A physically ruptured ring remains `alive=true` as a legacy-compatible field, reports nonviability, and still accepts transport. No `alive=false` injection is used for the no-respawn causal check.

The bounded conservative requalification exercised reaction closure, physical damage, observer-only death, and conservative fission partition accounting. Historical D-087/D-088 evidence and the Phase 1 source remain preserved; the legacy runtime preservation smoke is emitted separately and is not rewritten as v2 evidence.

## E5 boundary

The six E5 rows are compact v2 contract replays using the existing mesh kernels and finite N/F boundary exchange. They are explicitly labeled contract replays; they do not overwrite or claim to reproduce the historical D-015/D-016/R8 protocol artifacts. Each row reports strict material, activation, organized-retained, boundary, and closure residuals.

Primary classification:

`DCDEV020R9_METRIC_CONFOUNDING_DOMINANT`

This classification is limited to the demonstrated coordinate result: strict material closes, organized retained material can remain nonnegative, and `E_AR` can decline independently. It authorizes no new metabolic law, salvage pathway, controller, behavior, evolution, or DC-DEV-021 work.

Production chemistry change: versioned conservation repair only. Production behavior and historical evidence are preserved.

Artifacts: `digital-protocell/experiments/generated/dcdev020r9/`.
