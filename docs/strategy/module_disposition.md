# Digital Cell Module Disposition

This is a reconciliation inventory, not a deletion plan. No module is deleted or archived by DC-SR-001.

| Path / module family | Purpose and authority | External analogue | Disposition | Reason / safety |
|---|---|---|---|---|
| chemistry-core/src/material_mesh.rs | Conserved organism body; certified substrate | None equivalent established | KEEP | Scientific differentiator; protect with D-086/D-087 evidence |
| mesh_mechanics.rs, mesh_reactions.rs, mesh_transport.rs | Local mechanics, material turnover, boundary/resource coupling | ALIEN/Ribossome references | KEEP | Core causal mechanisms |
| metabolic_reserve.rs, activated_metabolism.rs, reactions.rs | Resource and activated chemistry | Evo2Sim benchmark | KEEP / BENCHMARK | Preserve qualified chemistry |
| mesh_growth.rs, mesh_topology.rs, mesh_fission.rs, mesh_population.rs | Surplus growth, local pinch/fission, state partition | DISHTINY/Avida | KEEP / BENCHMARK | D-088-qualified reproduction |
| phase1-certifier/ | Independent certification | Avida/Aevol methods | KEEP | Independent authority boundary |
| experiment-runner/ and d0xx analysis | Governed historical experiments | MABE2/Evochora | ADAPT | Modularize behind Layer 1 APIs |
| population_selection.rs, spatial_shared_dish.rs, d090_* | Selection/ecology trials | Evo2Sim, Avida, MABE2 | BENCHMARK / ADAPT | Selection not established |
| catalyst_composition.rs, template_*, template_network*, d094_* | Historical heredity candidates | Stringmol, Aevol, Avida | ARCHIVE / BENCHMARK | Preserve evidence; freeze escalation |
| d091_*, d092_*, d093_* | Qualified reserve/heredity studies | Evo2Sim/Stringmol | KEEP / BENCHMARK | Historical evidence remains reproducible |
| godot/, godot-bridge/ | Microscope shell and native interface | ALIEN/Ribossome | KEEP / REVIEW | Display/control boundary |
| configs/, snapshots, identities, accounting, provenance | Reproducibility and evidence | Evochora/Avida | KEEP / ADAPT | Protect hashes and accepted-step semantics |
| experiments/generated/ | Historical generated evidence | Evochora/Avida | ARCHIVE | Storage-backed where applicable; never delete blindly |
| .agent/, .cursor/, .serena/, .cocoindex_code/ | Governance and navigation | N/A | KEEP / REVIEW | Required continuity |
| Future GPU/world/sensor/neural/multicellular layers | Not implemented | Ribossome, ALIEN, DISHTINY, Polyworld, CAX/Lenia | REVIEW | Not authorized here |

For every future candidate record path, purpose, phase, scientific authority, runtime dependency, historical status, external analogue, license, disposition, isolation/removal safety, tests, and dependent artifacts. REPLACE is not assigned until SR-002 establishes a legal and technical boundary.

## DC-SR-002 external audit reconciliation

The following is an additive strategy decision; the pre-audit rows above remain historical context.

| Path / module family | DC-SR-002 decision | Evidence and boundary |
|---|---|---|
| material_mesh.rs, mesh_mechanics.rs, mesh_reactions.rs, mesh_transport.rs | KEEP | No audited source provides conserved material-mesh authority or an equivalent causal boundary. |
| mesh_growth.rs, mesh_topology.rs, mesh_fission.rs, mesh_population.rs | KEEP / BENCHMARK | Physical growth/fission remains Digital Cell-owned; DISHTINY/Avida are method benchmarks only. |
| template_*, template_network*, d089-d094 | KEEP / BENCHMARK | Stringmol/Aevol overlap heredity concepts but do not replace material template/fission causality; D-094 remains frozen. |
| population_selection.rs, spatial_shared_dish.rs, experiment-runner | ADAPT | Add a thin manifest/protocol/lineage/neutral-control harness; preserve observer-only selection and causal birth boundaries. |
| phase1-certifier, godot/, godot-bridge/ | KEEP | Certified evidence and current visualization/control bridge remain authoritative. |
| external source code and dependencies | REJECT_INTEGRATION | No external source or dependency is authorized by DC-SR-002; unknown licenses remain no-code-reuse. |
| future GPU/world layer | DEFER; ADOPT_WGPU_ARCHITECTURE_LATER | Only after measured need and CPU/certifier parity; use isolated wgpu patterns, not ALIEN/Ribossome organism code. |
| discovery sidecar | DEFER; ASAL_SIDECAR_RECOMMENDED | Future adapter may propose candidates, but governed Digital Cell artifacts determine scientific outcomes. |
