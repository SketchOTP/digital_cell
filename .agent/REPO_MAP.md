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
- `digital-protocell/examples/dcdev017_metabolic_homeostasis_foraging.rs` — Phase 0 control-surface audit and Phase 1 intrinsic reserve-timescale precursor-clamp challenge; later phases are conditional and must remain gated.

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
- `digital-protocell/experiments/generated/dcdev017/` — DC-DEV-017 phase-gated audit, prior-art disposition, intrinsic-timescale results, and later compact evidence only if authorized by prior gates.
- `digital-protocell/docs/strategy/developmental_sensorimotor/` — DC-DEV-001A human-readable analysis.

## External integration points

- `atlas:/home/sketch/Projects/authority/` — reference-only Authority governance checkout.
- `git@github.com:SketchOTP/digital_cell.git` — verified repository remote.

## Areas that must not be edited manually

- `digital-protocell/crates/chemistry-core/` — certified biology and equations are frozen.
- `digital-protocell/experiments/generated/` — evidence is append-only and provenance-bound.
- `.git/` — Git metadata and object storage.
- `.agent/legacy/` — preserved historical governance snapshots.
