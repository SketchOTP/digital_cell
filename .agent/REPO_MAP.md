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
- `digital-protocell/docs/strategy/developmental_sensorimotor/` — DC-DEV-001A human-readable analysis.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r2/` — DC-DEV-020-R2 observer requalification and Gate 4 disposition.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r3/` — DC-DEV-020-R3 two-substrate identifiability audit and fail-closed disposition.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r4/` — DC-DEV-020-R4 asymmetric independent-axis identifiability audit and fail-closed disposition.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r5/` — DC-DEV-020-R5 local zero-drift source requirement, R3/R4 surrogate, and existing-coordinate audit.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r6/` — DC-DEV-020-R6 generalized N/F power-law identification and Gate 5 negative-result audit.
- `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev020r7/` — DC-DEV-020-R7 on-policy zero-drift attribution, frozen coordinate replay, and exact-root oracle audit.

## External integration points

- `atlas:/home/sketch/Projects/authority/` — reference-only Authority governance checkout.
- `git@github.com:SketchOTP/digital_cell.git` — verified repository remote.

## Areas that must not be edited manually

- `digital-protocell/crates/chemistry-core/` — certified biology and equations are frozen.
- `digital-protocell/experiments/generated/` — evidence is append-only and provenance-bound.
- `.git/` — Git metadata and object storage.
- `.agent/legacy/` — preserved historical governance snapshots.
