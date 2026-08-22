# Repository Map

## Entry points

- `AGENTS.md` — canonical repository governance router.
- `.agent/` — adopted project contract, current state, ledgers, and historical handoffs.
- `digital-protocell/Cargo.toml` — Rust workspace entry point.

## Core modules

- `digital-protocell/crates/chemistry-core/` — certified material chemistry/equations remain frozen; DC-DEV-004 adds only the bounded post-Phase-1 edge-tension mechanics hook.
- `digital-protocell/crates/phase1-certifier/` — Phase 1 certification and regression checks.
- `digital-protocell/crates/experiment-runner/` — historical experiment orchestration; not changed by architecture selection.
- `digital-protocell/crates/evolution-harness/` — observer-only reusable evolution infrastructure; not extended by this directive.
- `digital-protocell/crates/regulatory-core/` — accepted regulatory state, DC-DEV-003 remesh continuity, the single DC-DEV-004 local contractility adapter, DC-DEV-005 plasticity, the bounded DC-DEV-006 spatial contact adapter, and the DC-DEV-008 finite-resource boundary with the DC-DEV-013 local contact observation.
- `digital-protocell/examples/dcdev009_gate_assay.rs` — observer-only fixed-topology free-space motility audit; it does not add production locomotion.
- `digital-protocell/examples/dcdev013_gate_assay.rs` — fixed-horizon local resource-contact feeding assay; it composes production resource observation, regulation, funded contractility, stick-slip, and uptake without implementing a second sensor.
- `digital-protocell/examples/dcdev015_metabolic_restoration_assay.rs` — observer-only 5,000-step settlement, 480-step deprivation, and matched metabolic intake-to-restoration audit; it reuses existing uptake and reaction/reserve ledgers without changing biology.
- `digital-protocell/examples/dcdev016_metabolic_break_even.rs` — observer-only one-shot derived-resource sufficiency challenge; it reproduces DC-DEV-015 baseline arms, tests one derived N/F inventory, and reports supply sufficiency versus stored activation restoration without changing biology.
- `digital-protocell/examples/dcdev020r2_allosteric_requalification.rs` — observer-only DC-DEV-020-R2 source-actuation, sequencing, August DC-DEV-017 replay, and A-only sufficiency audit; it stops before any derived law or downstream assay when Gate 4 fails.
- `digital-protocell/examples/dcdev020r3_two_substrate_saturating_activation.rs` — observer-only DC-DEV-020-R3 bilinear attribution and bounded symmetric two-substrate identifiability audit; it changes no production chemistry and stops fail-closed at Gate 4.
- `digital-protocell/examples/dcdev020r4_asymmetric_two_substrate_identifiability.rs` — observer-only DC-DEV-020-R4 five-probe independent-axis audit; it changes no production chemistry and stops fail-closed on reciprocal family mismatch.
- `digital-protocell/examples/dcdev020r5_local_zero_drift_source_audit.rs` — observer-only DC-DEV-020-R5 exact R4 replay, statewise physical source-response, zero-drift root, surrogate, and existing-coordinate audit; it changes no production chemistry.
- `digital-protocell/examples/dcdev020r6_nf_power_law_source.rs` — observer-only DC-DEV-020-R6 closed-form N/F power-law identification, held-out local-root validation, and fail-closed selected finite-feed counterfactual; it changes no production chemistry.
- `digital-protocell/examples/dcdev020r7_on_policy_zero_drift_audit.rs` — observer-only DC-DEV-020-R7 exact R6 replay, on-policy physical root audit, frozen NF/NFA observer replay, support-distance analysis, and exact-root oracle; it changes no production chemistry.
- `digital-protocell/examples/dcdev020r8_nfa_restorative_attractor.rs` — observer-only DC-DEV-020-R8 product-feedback topology feasibility audit using frozen N/F support, reciprocal constraints, and fail-closed training-gate classification; it changes no production chemistry.
- `digital-protocell/examples/dcdev020r8r1_causal_a_demand_elasticity.rs` — observer-only DC-DEV-020-R8-R1 within-state A perturbation, physical zero-drift demand decomposition, and R8 pair-confounding audit; it changes no production chemistry or behavior.
- `digital-protocell/examples/dcdev020r8r2_catalyst_investment_payback.rs` — observer-only DC-DEV-020-R8-R2 physical root, catalyst-production shadow, checkpoint payback, and whole-window R6 comparison; it changes no production chemistry or behavior.
- `digital-protocell/examples/dcdev020r8r4_shared_affinity_autogenous_cprod.rs` — preserved exact R8-R4 machinery with explicit R9-R1 ConservativeV2 compatibility replay; historical default behavior remains unchanged.
- `digital-protocell/examples/dcdev020r8r3_shared_affinity_helper.rs` — shared R8-R4 replay helper with explicit R9-R1 ConservativeV2 compatibility mode; historical default behavior remains unchanged.

## Interfaces and contracts

- `.agents/skills/authority-governance/SKILL.md` — adopted governance workflow.
- `.agents/skills/external-discovery/SKILL.md` — source-level prior-art workflow.
- `governance_carryforward.json` — Gate 0 carry-forward classifications.
- `governance_carryforward_manifest.json` — Gate 0 source/base manifest.

## Tests and validation

- `scripts/validate_governance.py` — governance validator.
- `scripts/test_validate_governance.py` — governance validator fixture tests.
- `digital-protocell/` — Cargo scientific-base regression tests.

## Configuration

- `.cursor/` — compatibility adapters and verified repository-owned configuration.
- `STORAGE_MAP.md` — canonical external storage locator.
- `docs/storage_archive_policy.md` — storage handling policy.

## Generated areas

- `digital-protocell/experiments/generated/` — generated evidence and provenance; preserve historical evidence.
- `digital-protocell/experiments/generated/dcdev001/` — DC-DEV-001A machine-readable decision artifacts.
- `digital-protocell/experiments/generated/dcdev006/` — DC-DEV-006 local spatial-contact evidence artifacts.
- `digital-protocell/experiments/generated/dcdev009/` — DC-DEV-009 force, displacement, coupling, and audit evidence artifacts.
- `digital-protocell/experiments/generated/dcdev013/` — frozen local resource-contact feeding protocol, settled body, matched-arm results, gate results, and final manifest.
- `digital-protocell/experiments/generated/dcdev015/` — frozen metabolic intake/restoration protocol, settlement, deprivation, matched-arm snapshots, ledgers, destination reconciliation, gate results, and final manifest.
- `digital-protocell/experiments/generated/dcdev016/` — frozen derived-resource break-even protocol, settlement, deprivation, matched-arm results, existing ledgers, gate results, and final manifest.
- `digital-protocell/experiments/generated/dcdev020r2/` — immutable R2 protocol, compact qualification, results, source-actuation envelope, and literature classification; historical `dcdev020/` evidence is preserved.
- `digital-protocell/experiments/generated/dcdev020r3/` — append-only R3 protocol, compact result/qualification, sole dense per-step kinetic ledger, and primary-literature classification.
- `digital-protocell/experiments/generated/dcdev020r4/` — append-only R4 protocol, compact result/qualification, sole dense five-probe identification ledger, and primary-literature classification.
- `digital-protocell/experiments/generated/dcdev020r5/` — compact append-only R5 protocol, results, qualification, schema, representative diagnostics, literature classification, and external dense-ledger SHA-256 manifest.
- `digital-protocell/experiments/generated/dcdev020r6/` — compact append-only R6 protocol, identification, finite-feed physiology, qualification, literature disposition, and R5 dense-input manifest; no dense R6 trajectory package is committed.
- `digital-protocell/experiments/generated/dcdev020r7/` — compact append-only R7 protocol, on-policy root/source summary, frozen observer/support statistics, oracle result, qualification, literature disposition, and external dense-ledger manifest.
- `digital-protocell/experiments/generated/dcdev020r8/` — compact append-only R8 protocol, matched-pair summary, reciprocal constraint summary, qualification, literature disposition, and external dense-ledger manifest; dense pair/constraint records remain in governed external evidence storage.
- `digital-protocell/experiments/generated/dcdev020r8r1/` — compact append-only R8-R1 protocol, A-elasticity decomposition, pair-confounding summary, qualification, literature disposition, and external dense-ledger manifest; the dense demand ledger remains in governed external evidence storage.
- `digital-protocell/experiments/generated/dcdev020r8r2/` — compact append-only R8-R2 protocol, 480 paired-root summary, two-context payback, whole-window shadow, qualification, literature disposition, and external dense-ledger manifest; the dense ledger remains in governed external evidence storage.
- `digital-protocell/examples/dcdev020r8r3_catalyst_reserve_horizon.rs` — observer-only R8-R3 frozen catalyst half-life, sustained reserve trajectories, deterministic marginal payback, source-context comparison, and conditional delayed-resume audit; it changes no production chemistry or behavior.
- `digital-protocell/experiments/generated/dcdev020r8r3/` — compact append-only R8-R3 acute reproduction, timescale, sustained trajectory, marginal payback, delayed-resume disposition, qualification, literature, and external dense-ledger manifest.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r8r3/` — R8-R3 catalyst reserve horizon documentation.
- `digital-protocell/docs/strategy/developmental_sensorimotor/` — DC-DEV-001A human-readable analysis.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r2/` — DC-DEV-020-R2 observer requalification and Gate 4 disposition.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r3/` — DC-DEV-020-R3 two-substrate identifiability audit and fail-closed disposition.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r4/` — DC-DEV-020-R4 asymmetric independent-axis identifiability audit and fail-closed disposition.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r5/` — DC-DEV-020-R5 local zero-drift source requirement, R3/R4 surrogate, and existing-coordinate audit.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r6/` — DC-DEV-020-R6 generalized N/F power-law identification and Gate 5 negative-result audit.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r7/` — DC-DEV-020-R7 on-policy zero-drift attribution, frozen coordinate replay, and exact-root oracle audit.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r8/` — DC-DEV-020-R8 product-feedback topology feasibility audit and fail-closed Gate 3 disposition.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r8r1/` — DC-DEV-020-R8-R1 causal A-demand elasticity audit and independent R8 pair-confounding disposition.
- `digital-protocell/examples/dcdev020r8r4_shared_affinity_autogenous_cprod.rs` — DC-DEV-020-R8-R4 observer-only shared-affinity audit; includes the normalized R8-R3 helper and must not be imported into production chemistry.
- `digital-protocell/experiments/generated/dcdev020r8r4/` — compact R8-R4 evidence; dense ledger is externalized to governed Atlas storage.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r8r4/` — R8-R4 protocol and disposition.
- `digital-protocell/examples/dcdev020r8r5_ac_allocation_upper_bound.rs` — DC-DEV-020-R8-R5 observer-only conservative A↔C allocation capacity envelope; it changes no production chemistry or behavior.
- `digital-protocell/experiments/generated/dcdev020r8r5/` — compact R8-R5 protocol, economic envelope, R8-R4 reproduction, constant-allocation results, local late-state drift summaries, qualification, literature disposition, and external dense-ledger manifest.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r8r5/` — R8-R5 A↔C allocation upper-bound audit and mixed-envelope disposition.
- `digital-protocell/examples/dcdev020r8r5r1_net_allocation_drift.rs` — DC-DEV-020-R8-R5-R1 corrected incoming-state net-drift observer wrapper; it preserves the R8-R5 constant-allocation runner.
- `digital-protocell/experiments/generated/dcdev020r8r5r1/` — compact R8-R5-R1 checkpoint hashes, statewise reversible/forward-only envelopes, attribution, qualification, and external dense-ledger manifest.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r8r5r1/` — R8-R5-R1 corrected net allocation drift documentation.
- `digital-protocell/crates/chemistry-core/src/mesh_contracts.rs` — R9 exact historical/conservative mesh stoichiometric descriptors, runtime parity, and three-ledger accounting.
- `digital-protocell/crates/chemistry-core/src/d020r9_analysis.rs` — bounded R9 E0-E5 contract requalification and compact evidence writer.
- `digital-protocell/crates/chemistry-core/examples/dcdev020r9_mesh_contract_requalification.rs` — R9 reproducible observer/requalification runner.
- `digital-protocell/experiments/generated/dcdev020r9/` — compact R9 E0-E5 evidence and preserved legacy runtime smoke artifact.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r9/` — R9 mesh contract requalification documentation.
- `.github/workflows/dc-dev-020r9.yml` — scoped R9 governance, format, preservation, contract-test, and evidence-runner CI.
- `digital-protocell/crates/chemistry-core/src/material_mesh.rs` — orthogonal `MeshContractVersion` metadata and observer-only death selection.
- `digital-protocell/crates/chemistry-core/src/d020r9_analysis.rs` — R9-R1 reserve-bearing D-087 Gates 0–7 matrix and compact evidence writer.
- `digital-protocell/examples/dcdev020r9r1_exact_metabolic_replays.rs` — exact D-015/D-016 observer replay and three-ledger closure evidence.
- `digital-protocell/examples/dcdev020r8r2_catalyst_investment_payback.rs` — preserved R8-R2 machinery with explicit R9-R1 ConservativeV2 compatibility mode.
- `digital-protocell/experiments/generated/dcdev020r9r1/` — R9-R1 compact mesh-contract, exact metabolic replay, and exact R8 compatibility evidence.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r9r1/` — R9-R1 protocol, provenance, and pending-acceptance documentation.
- `digital-protocell/crates/chemistry-core/src/mesh_reactions.rs` — observer-only direct A-decay accounting used by the R9-R2 material-fate ledger.
- `digital-protocell/crates/phase1-certifier/src/bin/phase1_certification.rs` — direct actual D-087 Gates 0–7 launcher with ConservativeV2+D-091 mode selection.
- `digital-protocell/examples/dcdev020r9r2_material_fate_audit.rs` — exact D-015/D-016 material-fate and sustained-trajectory observer runner.
- `digital-protocell/crates/phase1-certifier/examples/dcdev020r9r3_conservation_reserve_decomposition.rs` — observer-only actual D-087 contract × reserve matrix with H0 historical hard stop and reserve-execution proof.
- `digital-protocell/crates/phase1-certifier/src/runtime.rs` — Gate-7 packaged-runtime build/copy/launch qualification with platform executable-path handling and fail-visible subprocess diagnostics.
- `digital-protocell/experiments/generated/dcdev020r9r2/` — compact R9-R2 certifier, replay, fate, qualification, protocol, and manifest evidence.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r9r2/` — R9-R2 material-fate and certifier requalification documentation.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r9r3/conservation_reserve_decomposition.md` — R9-R3 observer-only contract × reserve decomposition, provisional classification, and preservation boundary.
- `digital-protocell/experiments/generated/dcdev020r3r1/` — fresh R9-R3-R1 H0/V20/H1/V21 replay evidence and Gate-7 packaging diagnostics; prior R9-R3 evidence remains preserved.
- `digital-protocell/crates/chemistry-core/src/metabolic_reserve.rs` — bounded R9-R4 observer-only reserve flux controls; default production path remains Full.
- `digital-protocell/crates/chemistry-core/src/mesh_reactions.rs` — additive R9-R4 observer-only A/R/build/membrane interference ledger; default production path remains Full.
- `digital-protocell/crates/phase1-certifier/examples/dcdev020r9r4_reserve_interference_audit.rs` — exact 5,000-step V20/V21 control, four required reserve ablations, maintenance-priority shadow, and Gate-5 reserve preservation observer.
- `digital-protocell/experiments/generated/dcdev020r9r4/` — compact R9-R4 protocol, qualification, and dense-ledger manifest; dense JSONL is external/local and hash-bound.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r9r4/` — R9-R4 observer-only reserve interference audit and fail-closed classification.
- `digital-protocell/crates/chemistry-core/src/metabolic_reserve.rs` — R9-R5 observer-only frozen-store potential and capped-storage helpers; default D-091 reserve kernel remains unchanged.
- `digital-protocell/crates/chemistry-core/src/mesh_reactions.rs` — additive R9-R5 stock/flux/closure ledger and surplus/liquid diagnostic modes; production mode remains Full.
- `digital-protocell/crates/phase1-certifier/examples/dcdev020r9r5_charge_liquidity_audit.rs` — exact 5,000-step FULL/STORE_OFF, surplus-only, liquid upper-bound, combined observer arms and actual D-087 shadow summaries.
- `digital-protocell/experiments/generated/dcdev020r9r5/` — compact R9-R5 protocol, qualification, and report; dense rows remain local/external audit output.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r9r5/` — R9-R5 charge/liquidity protocol and provisional fail-closed disposition.

## External integration points

- `atlas:/home/sketch/Projects/authority/` — reference-only Authority governance checkout.
- `git@github.com:SketchOTP/digital_cell.git` — verified repository remote.

## Areas that must not be edited manually

- `digital-protocell/crates/chemistry-core/` — certified biology and equations are frozen.
- `digital-protocell/experiments/generated/` — evidence is append-only and provenance-bound.
- `.git/` — Git metadata and object storage.
- `.agent/legacy/` — preserved historical governance snapshots.
