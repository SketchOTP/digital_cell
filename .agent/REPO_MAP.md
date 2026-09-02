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
- `digital-protocell/crates/regulatory-core/src/contractility.rs` — historical R-funded DC-DEV-004 API plus opt-in V4 `ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1` A-to-W adapter.
- `digital-protocell/crates/regulatory-core/src/stick_slip_traction.rs` — historical DC-DEV-011 adapters plus the opt-in A-funded composition.
- `digital-protocell/examples/dcdev009_gate_assay.rs` — observer-only fixed-topology free-space motility audit; it does not add production locomotion.
- `digital-protocell/examples/dcdev013_gate_assay.rs` — fixed-horizon local resource-contact feeding assay; it composes production resource observation, regulation, funded contractility, stick-slip, and uptake without implementing a second sensor.
- `digital-protocell/examples/dcdev015_metabolic_restoration_assay.rs` — observer-only 5,000-step settlement, 480-step deprivation, and matched metabolic intake-to-restoration audit; it reuses existing uptake and reaction/reserve ledgers without changing biology.
- `digital-protocell/examples/dcdev016_metabolic_break_even.rs` — observer-only one-shot derived-resource sufficiency challenge; it reproduces DC-DEV-015 baseline arms, tests one derived N/F inventory, and reports supply sufficiency versus stored activation restoration without changing biology.
- `digital-protocell/examples/dcdev021_m2_entry001.rs` — bounded opt-in A-funded contractility and stick-slip feasibility assay; it does not implement resource acquisition.
- `digital-protocell/crates/regulatory-core/src/intrinsic_exploration.rs` — opt-in, versioned ENTRY-003 intrinsic local activity state that composes frozen local regulator/plasticity constants with the accepted A-funded actuator; it reads no resource, target, gradient, observer, or viability state.
- `digital-protocell/examples/dcdev021_m2_entry003.rs` — preregistered resource-free intrinsic-exploration feasibility assay; it records a negative mechanical result without installing production resource-seeking behavior.
- `digital-protocell/examples/dcdev021_m2_entry004.rs` — observer-only clone/free-proposal audit that compares ENTRY-001 clutch crossing to the adaptation-limited ENTRY-003 trajectory; it does not modify the explorer, actuator, traction, or production runtime.
- `digital-protocell/examples/dcdev021_m2_entry005.rs` — preregistered resource-free feasibility assay for the opt-in refractory-only motor composition; it preserves ENTRY-003 dynamics while supplying raw intrinsic activity to the accepted A-funded motor.
- `digital-protocell/examples/dcdev021_m2_entry006.rs` — preregistered observer-only composition of ENTRY-005 target-free exploration with the frozen DC-DEV-013 finite N/F ecology; resource contact is never an organism input.
- `digital-protocell/examples/dcdev021_m2_entry007.rs` — observer-only decomposition of unchanged DC-DEV-008 uptake per step and exposed edge across ENTRY-006 unguided, ENTRY-003 pinned, and motor-off arms.
- `digital-protocell/examples/dcdev021_m2_entry010.rs` — observer-only paired transfer/contact-without-transfer audit of existing internal N/F concentrations and V4 material amounts; it does not implement post-ingestive behavior.
- `digital-protocell/examples/dcdev021_m2_entry011.rs` — observer-only composition of ENTRY-005 locomotion, unchanged DC-DEV-008 uptake, and the exact frozen V4-compatible reaction kernel; it does not add behavior or change production defaults.
- `digital-protocell/examples/dcdev021_m2_entry012.rs` — observer-only separated-resource encounter assay using the exact ENTRY-011 composition; it preregisters one settled mean-edge-length gap and does not add resource-seeking behavior.
- `digital-protocell/examples/dcdev021_m2_entry013.rs` — observer-only intrinsic search-persistence audit using exact ENTRY-012 no-resource composition; it records ring modes/kinematics and runs non-production phase-locked/fixed-profile counterfactuals without changing scientific runtime.
- `digital-protocell/examples/dcdev021_m2_entry014.rs` — isolated mathematical reimplementation of the published Morpheus M2071 Polar and Traveling-Wave 1-D periodic regimes; it performs no Digital Cell runtime, actuator, resource, or observer-feedback calls.
- `digital-protocell/examples/dcdev021_m2_entry015.rs` — isolated resource-free assay composing the exact ENTRY-014 24-site polarity equations with the unchanged A-funded actuator through the parameter-free local `u/(u+v)` interface and same-mean/motor-off controls; it does not install production polarity.
- `digital-protocell/examples/dcdev021_m2_entry016.rs` — observer-only homogeneous-equilibrium/24-site linear-stability and settled-body local-field audit; it does not initialize, couple, or install production polarity.
- `digital-protocell/examples/dcdev021_m2_entry017.rs` — observer-only replay of the accepted D-088 physical growth/fission path with mother/daughter local-field spectra, conservative partition closure, topology compatibility, provenance, and post-fission persistence; it does not initialize or couple polarity.

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

- `digital-protocell/experiments/generated/dcdev020m1closure001/m1_closure_manifest.json` — compact accepted M1 closure reference retained without the dense evidence archive.
- `digital-protocell/experiments/generated/dcdev020postm1baseline001/` — compact clean-baseline inventory, manifest, and validation evidence.
- `digital-protocell/experiments/generated/dcdev021m2entry001/` — compact ENTRY-001 feasibility evidence, populated by exact-head Linux validation.
- `digital-protocell/experiments/generated/dcdev021m2entry002/` — compact observer-only ENTRY-002 temporal-navigation substrate-audit evidence; dense per-step ledgers remain on Atlas.
- `digital-protocell/experiments/generated/dcdev021m2entry003/` — compact ENTRY-003 intrinsic-exploration feasibility evidence; dense trajectory ledgers remain on Atlas.
- `digital-protocell/experiments/generated/dcdev021m2entry004/` — compact ENTRY-004 intrinsic-to-traction force-transfer evidence; dense per-vertex/per-step ledgers remain on Atlas.
- `digital-protocell/experiments/generated/dcdev021m2entry005/` — compact ENTRY-005 refractory-only motor feasibility evidence; dense trajectories remain on Atlas.
- `digital-protocell/experiments/generated/dcdev021m2entry006/` — compact ENTRY-006 unguided finite-resource acquisition evidence; dense trajectories remain on Atlas.
- `digital-protocell/experiments/generated/dcdev021m2entry007/` — compact ENTRY-007 uptake-degradation audit evidence with per-step exposed-edge records embedded in the three arm artifacts.
- `digital-protocell/experiments/generated/dcdev021m2entry010/` — compact ENTRY-010 internal material-signal audit evidence; dense paired state records remain externalized by the assay interface.
- `digital-protocell/experiments/generated/dcdev021m2entry011/` — compact ENTRY-011 frozen uptake/metabolism composition evidence; dense trajectories remain externalized to Atlas.
- `digital-protocell/experiments/generated/dcdev021m2entry012/` — compact ENTRY-012 separated-resource encounter and reachability evidence; dense trajectories remain externalized to Atlas.
- `digital-protocell/experiments/generated/dcdev021m2entry013/` — compact ENTRY-013 ring-mode, polarity-persistence, kinematic, mechanical-counterfactual, preservation, and qualification evidence; dense trajectories remain externalized to Atlas.
- `digital-protocell/experiments/generated/dcdev021m2entry014/` — compact ENTRY-014 external-provenance, equation, conservation, reference-reproduction, exact-24-site-transfer, compatibility, preservation, and qualification evidence; dense numerical trajectories remain externalized.
- `digital-protocell/experiments/generated/dcdev021m2entry015/` — compact ENTRY-015 polarity-to-actuator interface, equal-drive controls, translation/reorientation, energetic closure, semantic boundary, preservation, and qualification evidence; dense trajectories remain externalized.
- `digital-protocell/experiments/generated/dcdev021m2entry016/` — compact ENTRY-016 homogeneous equilibria, discrete-mode stability, homogeneous replay, settled local-field inventory, asymmetry provenance, mapping boundary, preservation, and qualification evidence.
- `digital-protocell/experiments/generated/dcdev021m2entry017/` — compact ENTRY-017 mother/daughter physical-state snapshots, local asymmetry spectra, partition closure, topology boundary, life-history provenance, rotation, persistence, preservation, and qualification evidence.
- `digital-protocell/examples/dcdev021_m2_entry018.rs` — isolated conservative finite-volume transfer of the accepted M2071 polarity PDE onto normalized physical-arclength material rings, including regular-grid regression, native stability, homogeneous replay, and ENTRY-017 geometry replay; it never initializes polarity or calls behavior/runtime coupling.
- `digital-protocell/experiments/generated/dcdev021m2entry018/` — compact ENTRY-018 native coordinate, operator, conservation, topology, stability, replay, projection, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-entry018.yml` — exact-head Linux validation for ENTRY-018 authority, historical preservation, native numerical audit, D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_entry019.rs` — isolated observer-only replay of accepted D-088 pre-fission physical history with conservative native-ring amount transport, homogeneous controls, and unchanged ENTRY-018 reaction-diffusion seed/amplification attribution; it does not initialize production polarity or call behavior.
- `digital-protocell/experiments/generated/dcdev021m2entry019/` — compact ENTRY-019 authority, physical-history, conservative-remesh, control, causal-attribution, conservation, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-entry019.yml` — exact-head Linux validation for ENTRY-019 authority, isolated assay, historical preservation, D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_entry020.rs` — isolated ENTRY-020 live autonomous-Polar composition with homogeneous initialization, conservative native-ring continuity, exact `u/(u+v)` interface, unchanged A-funded mechanics, matched controls, and no resource behavior.
- `digital-protocell/examples/dcdev021_m2_entry024.rs` — isolated ENTRY-024 direct-versus-complementary effector-orientation audit through unchanged A-funded contractility/stick-slip; it does not change production polarity or resource behavior.
- `digital-protocell/experiments/generated/dcdev021m2entry024/` — compact ENTRY-024 direct-parity, complement-identity, inherited/reference controls, spatial-leverage, closure, rotation/index, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-entry024.yml` — exact-head Linux validation for ENTRY-024 authority, orientation controls, historical preservation, production/D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/experiments/generated/dcdev021m2entry020/` — compact ENTRY-020 authority, live causal-order, initiation, locomotion, control, closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-entry020.yml` — exact-head Linux validation for ENTRY-020 authority, autonomous polarity/locomotion gates, historical preservation, D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_entry025.rs` — isolated ENTRY-025 live post-fission antagonistic inherited-polarity assay with strict eligibility, direct-live parity, unchanged mechanics/remesh, matched mean/off controls, closure, and preservation evidence.
- `digital-protocell/experiments/generated/dcdev021m2entry025/` — compact ENTRY-025 authority-correction, fission, live causal-order, daughter robustness, closure, rotation/index, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-entry025.yml` — exact-head Linux validation for ENTRY-025 authority, live controls, preservation, D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/experiments/generated/dcdev001/` — DC-DEV-001A machine-readable decision artifacts.
- `digital-protocell/experiments/generated/dcdev006/` — DC-DEV-006 local spatial-contact evidence artifacts.
- `digital-protocell/experiments/generated/dcdev009/` — DC-DEV-009 force, displacement, coupling, and audit evidence artifacts.
- `digital-protocell/experiments/generated/dcdev013/` — frozen local resource-contact feeding protocol, settled body, matched-arm results, gate results, and final manifest.
- `digital-protocell/experiments/generated/dcdev015/` — frozen metabolic intake/restoration protocol, settlement, deprivation, matched-arm snapshots, ledgers, destination reconciliation, gate results, and final manifest.
- `digital-protocell/experiments/generated/dcdev016/` — frozen derived-resource break-even protocol, settlement, deprivation, matched-arm results, existing ledgers, gate results, and final manifest.
- `digital-protocell/docs/strategy/developmental_sensorimotor/` — DC-DEV-001A human-readable analysis.

## External integration points

- `atlas:/home/sketch/Projects/authority/` — reference-only Authority governance checkout.
- `git@github.com:SketchOTP/digital_cell.git` — verified repository remote.

## Areas that must not be edited manually

- `digital-protocell/crates/chemistry-core/` — certified biology and equations are frozen.
- `digital-protocell/experiments/generated/` — evidence is append-only and provenance-bound.
- `.git/` — Git metadata and object storage.
- `PR #44 and /srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1closure001/` — historical M1 provenance and dense evidence; do not rewrite from the baseline branch.
