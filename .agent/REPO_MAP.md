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
- `digital-protocell/crates/regulatory-core/src/finite_world.rs` — reusable opt-in `FiniteWorldV1` finite shared-resource exchange with pre-step request collection, common proportional allocation, transfer disablement, and focused conservation/order tests.
- `digital-protocell/examples/dcdev021_m2_closure001.rs` — bounded M2 closure runtime entry point.
- `digital-protocell/examples/dcdev021_m2_closure001_impl.rs` — closure evidence generator for finite-world transfer, metabolism, inherited polarity, fission, existing-motor and assay-only local-protrusion arms.
- `digital-protocell/experiments/generated/dcdev021m2closure001/` — closure-level finite-world, feeding sanity, ecology, reproduction, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure001.yml` — exact-head Linux validation for the closure runtime, finite-world tests, historical preservation, D-087, downstream tests, governance, and artifact upload.
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
- `digital-protocell/examples/dcdev021_m2_entry026.rs` — isolated ENTRY-026 post-fission continued-development polarity-maintenance audit; no actuator, resource, production polarity, or second-fission execution.
- `digital-protocell/experiments/generated/dcdev021m2entry026/` — compact ENTRY-026 authority, growth controls, polarity chronology, conservation, remesh, rotation/index, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-entry026.yml` — exact-head Linux validation for ENTRY-026 authority, assay evidence, historical preservation, D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_entry026r1.rs` — isolated R1 requalification of the accepted MeshPopulation growth/fission gate and continued inherited-polarity maintenance; it preserves sealed ENTRY-026 and does not execute second fission or call behavior/resource paths.
- `digital-protocell/experiments/generated/dcdev021m2entry026r1/` — compact R1 authority, daughter birth-mass thresholds, corrected lifecycle-gate evidence, material closure, controls, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-entry026r1.yml` — exact-head Linux validation for R1 authority, sealed ENTRY-026 preservation, corrected assay, D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_entry027.rs` — isolated ENTRY-027 growth-on inter-fission inherited-polarity locomotion assay with spatial, same-mean, motor-off, lifecycle, closure, rotation/index, and preservation evidence; no second fission or production change.
- `digital-protocell/experiments/generated/dcdev021m2entry027/` — compact ENTRY-027 authority, daughter-arm, lifecycle, locomotion, closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-entry027.yml` — exact-head Linux validation for ENTRY-027 authority, assay evidence, historical preservation, D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_entry028.rs` — isolated ENTRY-028 balanced separated-resource ecological coupling entry point.
- `digital-protocell/examples/dcdev021_m2_entry028_impl.rs` — ENTRY-028 assay implementation reusing the accepted ENTRY-027 daughter/fission path; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2entry028/` — compact ENTRY-028 geometry, 24 daughter/bearing arms, contact/acquisition, metabolism, lifecycle, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-entry028.yml` — exact-head Linux validation for ENTRY-028 authority, separated ecology, historical preservation, D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure001_impl.rs` — additive R1 finite-world closure assay with corrected transport, polygon geometry, lineage accounting, passive traction semantics, and conditional local polarity clutch; no production default change.
- `digital-protocell/experiments/generated/dcdev021m2closure001r1/` — compact R1 authority, boundary, geometry, lineage, acquisition, clutch, closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure001-r1.yml` — exact-head Linux validation for R1 boundary corrections, conditional clutch ecology, preservation, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure002.rs` — isolated CLOSURE-002 entry point for corrected clutch controls and resource-dependent lifecycle causality.
- `digital-protocell/examples/dcdev021_m2_closure002_impl.rs` — CLOSURE-002 assay implementation with same-mean/frozen-traction controls, finite-world lifecycle, persistence/development/reproduction evidence, and no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure002/` — compact CLOSURE-002 authority, control, lifecycle, material/energy closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure002.yml` — exact-head Linux validation for CLOSURE-002 controls, lifecycle causality, preservation, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure003.rs` — isolated CLOSURE-003 entry point for reproductive resource-budget calibration and conditional lifecycle evidence.
- `digital-protocell/examples/dcdev021_m2_closure003_impl.rs` — CLOSURE-003 assay implementation with one permitted direct-contact calibration per daughter authority and a hard stop when unforced fission does not establish a resource unit; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure003/` — compact CLOSURE-003 authority, calibration, conditional lifecycle, closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure003.yml` — exact-head Linux validation for CLOSURE-003 budget authority, calibration stop boundary, preservation, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure003r1.rs` — isolated CLOSURE-003-R1 entry point for finite whole-membrane reproductive calibration and conditional spatial ecology.
- `digital-protocell/examples/dcdev021_m2_closure003r1_impl.rs` — assay-only finite membrane-wide transport/debit adapter, fixed-capacity daughter calibration, empirical resource-unit derivation, and conditional lifecycle boundary; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure003r1/` — compact CLOSURE-003-R1 authority, whole-membrane calibration, finite-unit, spatial stop, closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure003r1.yml` — exact-head Linux validation for CLOSURE-003-R1 calibration, conditional ecology, preservation, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure004.rs` — isolated CLOSURE-004 entry point for native material-consistent spatial resource geometry and reproductive ecology.
- `digital-protocell/examples/dcdev021_m2_closure004_impl.rs` — derived V1 radius, non-overlap audit, finite spatial lifecycle arms, closure evidence, and bounded access-insufficient classification; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure004/` — compact CLOSURE-004 geometry, lifecycle, closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure004.yml` — exact-head Linux validation for CLOSURE-004 native geometry, finite access, preservation, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure005.rs` — isolated CLOSURE-005 entry point for per-lineage finite-world attribution and fixed-coordinate solo/paired reproductive ecology.
- `digital-protocell/examples/dcdev021_m2_closure005_impl.rs` — observer-only organism/resource delivery attribution, reproductive-demand landmarks, solo/pair controls, and conditional descendant boundary; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure005/` — compact CLOSURE-005 authority, per-lineage ledgers, solo/pair lifecycle evidence, closure, preservation, and qualification artifacts.
- `.github/workflows/dc-dev-021-m2-closure005.yml` — exact-head Linux validation for CLOSURE-005 authority, per-lineage ecology, preservation, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure006.rs` — isolated CLOSURE-006 entry point for local resource-contact quiescence and reproductive ecology.
- `digital-protocell/examples/dcdev021_m2_closure006_impl.rs` — assay-only local contact/regulator composition, paired/solo controls, finite transfer, lifecycle, heredity boundary, and compact qualification evidence; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure006/` — compact CLOSURE-006 authority, contact provenance, controls, material/energy closure, lifecycle boundary, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure006.yml` — exact-head Linux validation for CLOSURE-006 local contact quiescence, controls, preservation, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure008.rs` — isolated CLOSURE-008 entry point for lineage-local and paired replay of the accepted post-ingestive material-work composition.
- `digital-protocell/examples/dcdev021_m2_closure008_impl.rs` — CLOSURE-008 assay implementation with Daughter A, Daughter B, paired, transfer-disabled, zero-resource, no-material-feedback, and motor-off controls; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure008/` — compact CLOSURE-008 lineage comparison, acquisition/work, reproduction, closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure008.yml` — exact-head Linux validation for CLOSURE-008 authority, lineage controls, preservation, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure007.rs` — isolated CLOSURE-007 entry point for assay-only post-ingestive material-to-work composition.
- `digital-protocell/examples/dcdev021_m2_closure007_impl.rs` — CLOSURE-007 implementation reusing existing internal N/F/A/W state, local regulator composition, finite-world transfer, frozen metabolism, growth, fission, and heredity accounting; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure007/` — compact CLOSURE-007 authority, material-signal definition, candidate/control arms, acquisition/work, closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure007.yml` — exact-head Linux validation for CLOSURE-007 authority, assay classification, preservation, D-087, downstream tests, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure009.rs` — isolated CLOSURE-009 entry point for direct post-ingestive material-to-motor allocation requalification.
- `digital-protocell/examples/dcdev021_m2_closure009_impl.rs` — assay-only direct `motor_i=base_i*(1-S)` composition and fixed lineage/paired controls; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure009/` — compact CLOSURE-009 authority, direct-allocation comparison, controls, closure, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure009.yml` — exact-head Linux validation for CLOSURE-009 authority, direct material allocation, preservation, governance, and artifact upload.
- `digital-protocell/examples/dcdev021_m2_closure010.rs` — isolated CLOSURE-010 entry point composing accepted local physical contact regulation with accepted internal material motor allocation.
- `digital-protocell/examples/dcdev021_m2_closure010_impl.rs` — CLOSURE-010 paired/solo finite-world assay with direct-material, contact-local, no-feedback, transfer-disabled, zero-resource, and motor-off controls; no production behavior change.
- `digital-protocell/experiments/generated/dcdev021m2closure010/` — compact CLOSURE-010 composition, arm comparisons, material/energy closure, reproduction boundary, preservation, and qualification evidence.
- `.github/workflows/dc-dev-021-m2-closure010.yml` — exact-head Linux validation for CLOSURE-010 authority, combined composition, controls, preservation, governance, and artifact upload.
- `digital-protocell/crates/chemistry-core/src/mesh_self_contact.rs` — DC-FINAL-001 opt-in geometry-only local frictionless nonpenetration and simple-polygon observers.
- `digital-protocell/crates/chemistry-core/src/planar_ring_topology.rs` — DC-FINAL-001 opt-in planar half-edge material-ring fallback preserving simple topology through frozen remesh operations.
- `digital-protocell/crates/chemistry-core/src/mesh_fission.rs` — retains the historical vertex-pinch API and adds opt-in conservative segment-apposition scission used only by DC-FINAL-001.
- `digital-protocell/examples/dcfinal001_reproduction.rs` — terminal Work Package 1 ten-arm reproduction ladder: vertex pinch, segment apposition, and authorized planar half-edge fallback.
- `digital-protocell/examples/dcfinal001_acquisition.rs` — terminal Work Package 2 finite conservative N/F field and source-derived local metabolism-to-polarity assay with three seeds, four bearings, and sensor/motor/field controls.
- `digital-protocell/experiments/dcfinal001_evidence.py` — compact terminal evidence generator sealing WP1 reproduction success and `NOT_EXECUTED_HARD_STOP_WP2` downstream boundaries.
- `digital-protocell/experiments/generated/dcfinal001/` — DC-FINAL-001 authority, valid reproduction/daughters, finite-field acquisition negative, hard-stop matrix, preservation, qualification, and manifest.
- `.github/workflows/dc-final-001.yml` — exact-head Linux validation for Work Packages 1-2, historical preservation, terminal qualification, and artifact upload.
- `digital-protocell/crates/regulatory-core/src/adaptive_chemotaxis.rs` — R1 zero-free-parameter perimeter-weighted material comparator producing local front/rear drives with exact uniform-field silence.
- `digital-protocell/examples/dcfinal001_r1_acquisition.rs` — R1-E three-seed/four-bearing adaptive front/rear acquisition matrix and six causal controls.
- `digital-protocell/experiments/generated/dcfinal001r1/` — R1 compact authority, acquisition, continuation, preservation, and final qualification evidence.
- `.github/workflows/dc-final-001-r1.yml` — R1 exact-head Linux validation and evidence artifact upload.
- `digital-protocell/crates/regulatory-core/src/low_level_sensory.rs` — non-semantic RGB luminance/motion and PCM amplitude/band environmental transduction.
- `digital-protocell/examples/dcfinal001_r1_life_history.rs` — shared-development, divergent-history, common-environment, and geometry-valid fission persistence assay.
- `digital-protocell/examples/dcfinal001_r1_evolution.rs` — finite physical mutation-on/off H-to-B campaign; its sealed result is negative and invalid fissions are excluded.
- `digital-protocell/crates/m2-lifeform-runtime/src/bin/digital-cell-final-lifeform.rs` — standalone adaptive organism/world runtime with atomic checkpointing, local experiential plasticity, inherited allocation, valid-fission handling, and observer-independent execution.
- `digital-protocell/crates/godot-bridge/src/lib.rs` — retains the historical simulator and adds a read-only `FinalLifeformObserver` for standalone runtime reports.
- `digital-protocell/examples/dcfinal001_r2_evolution.rs` — R2 assay-only powered lawful-birth mutation, fixed open-medium population, mutation-off, H/B, and fixed H-to-B reversal harness with exact cohort-exchangeability controls.
- `digital-protocell/experiments/dcfinal001_r2_evidence.py` — deterministic compact R2 evidence generator preserving R1 authority and separating physical death from post-birth runtime invalidation.
- `digital-protocell/experiments/generated/dcfinal001r2/` — R2 mutation source/power/frequency, genotype endpoint, population material, selection/reversal boundary, preservation, and provisional qualification evidence.
- `.github/workflows/dc-final-001-r2.yml` — exact-head Linux scope, mutation/population, preservation, D-087, PR #44, reproducibility, and artifact validation.
- `digital-protocell/crates/chemistry-core/tests/r4_contract_topology_tests.rs` — R4 contract-ownership, non-V4 isolation, V4 rupture/subpool closure, rebond-maturation, and inert-field preservation tests.
- `digital-protocell/examples/dcfinal001_r4_evolution.rs` — R4 contract matrix, exact R2 regression, V4 newborn closing-edge trace, canonical production-V4 reproduction campaign, and Gate-9 evolution stop harness.
- `digital-protocell/experiments/dcfinal001_r4_evidence.py` — deterministic compact R4 evidence generator preserving R1-R3 authority and separating production-V4 reproduction from HistoricalV1 regression.
- `digital-protocell/experiments/generated/dcfinal001r4/` — R4 authority, ownership, topology bookkeeping, newborn continuation, V4 reproduction, preservation, stopped-evolution, final matrix, and qualification evidence.
- `.github/workflows/dc-final-001-r4.yml` — R4 exact-head Linux scope, reproduction/historical regression replay, preservation, D-087, reproducibility, and artifact validation.
- `digital-protocell/examples/dcfinal001_r5_v4_neck.rs` — R5 exact V4 failure taxonomy, source-timescale horizon audit, clone-only reference-length counterfactual, frozen strain-regulator/A-funded contractility composition, required controls, and Gate-9 stop.
- `digital-protocell/experiments/dcfinal001_r5_evidence.py` — deterministic compact R5 evidence generator preserving R4 authority and explicitly marking evolution/final integration not reached.
- `digital-protocell/experiments/generated/dcfinal001r5/` — R5 authority, maturation, failure taxonomy, mechanochemical attribution/controls, preservation, stopped-evolution, final matrix, and provisional qualification evidence.
- `.github/workflows/dc-final-001-r5.yml` — exact-head Linux scope, deterministic R5 replay, preservation, D-087, PR #44, evidence reproducibility, and artifact upload.
- `digital-protocell/examples/dcfinal001_r5r1_strain_contrast.rs` — R5R1 entry point that reuses the exact R5 fixture and executes the sole zero-parameter mean-relative strain contrast fallback with matched controls.
- `digital-protocell/experiments/dcfinal001_r5r1_evidence.py` — deterministic compact R5R1 evidence generator preserving R5 authority and sealing the Gate-5 terminal stop.
- `digital-protocell/experiments/generated/dcfinal001r5r1/` — R5R1 authority, functional localization, contrast/control, reproduction, preservation, stopped-evolution, and provisional terminal qualification evidence.
- `.github/workflows/dc-final-001-r5r1.yml` — exact-head Linux R5R1 scope, deterministic contrast replay, preservation, D-087, PR #44, reproducibility, and artifact upload.
- `digital-protocell/examples/dcfinal001_r6_curvature_normal.rs` — R6 entry point reusing the exact R5/R5R1 fixture for tangential-force geometry audit and zero-parameter curvature-gated inward-normal mechanics.
- `digital-protocell/experiments/dcfinal001_r6_evidence.py` — deterministic compact R6 evidence generator preserving R5R1 authority and sealing the Gate-8 reproduction stop.
- `digital-protocell/experiments/generated/dcfinal001r6/` — R6 owner override, actuator geometry, curvature/force, controls, reproduction, preservation, stopped-evolution, and provisional qualification evidence.
- `.github/workflows/dc-final-001-r6.yml` — exact-head Linux R6 scope, deterministic campaign replay, preservation, D-087, PR #44, reproducibility, and artifact upload.
- `digital-protocell/examples/dcfinal001_r7_strain_contrast_normal.rs` — R7 entry point composing the exact R5R1 strain contrast with the exact R6 funded inward-normal actuator.
- `digital-protocell/experiments/dcfinal001_r7_evidence.py` — deterministic compact R7 evidence generator preserving R6 authority and sealing the Gate-7 reproduction stop.
- `digital-protocell/experiments/generated/dcfinal001r7/` — R7 authority, exact composition, controls, per-fission daughter diagnostics, preservation, stopped-evolution, and provisional qualification evidence.
- `.github/workflows/dc-final-001-r7.yml` — exact-head Linux R7 scope, deterministic campaign replay, preservation, D-087, PR #44, evidence reproducibility, and artifact upload.
- `digital-protocell/crates/chemistry-core/tests/r8_v4_closure_consistency_tests.rs` — R8 V4 young/mixed/mature load-bearing, rupture, rebond, two-edge yield accounting, and non-V4 parity contract matrix.
- `digital-protocell/experiments/fixtures/dcfinal001r8/` — exact pre-R8 R6/R7 newborn meshes used only for corrected daughter-continuation diagnostics, with compact governed hashes.
- `digital-protocell/examples/dcfinal001_r8_v4_closure.rs` — R8 entry point for historical daughter replay and frozen corrected passive/R5R1/R6 four-arm V4 reproduction qualification.
- `digital-protocell/experiments/dcfinal001_r8_evidence.py` — deterministic compact R8 evidence generator separating confirmed contract repair from failed robust reproduction.
- `digital-protocell/experiments/generated/dcfinal001r8/` — R8 root cause, structural/load-bearing contracts, daughter replay, corrected campaigns, preservation, stopped-evolution, and provisional qualification evidence.
- `.github/workflows/dc-final-001-r8.yml` — exact-head Linux R8 scope, contract tests, deterministic corrected campaign, preservation, D-087, PR #44, evidence reproducibility, and artifact upload.
- `digital-protocell/examples/dcfinal001_r9_gate1.rs` — R9 observer-only replay entry point for the sealed R8R1 static-attractor audit.
- `digital-protocell/examples/dcfinal001_r9_refractory_curvature.rs` — R9 entry point composing the frozen R6 curvature-normal actuator with existing accepted-step `PlasticityStateV1` and its authorized same-activity conditional control.
- `digital-protocell/experiments/dcfinal001_r9_evidence.py` — deterministic compact R9 evidence generator sealing static-attractor, refractory-remap, energy/control, reproduction, and stopped-downstream results.
- `digital-protocell/experiments/generated/dcfinal001r9/` — R9 authority, attractor audit, refractory contract, controls, campaign, daughter continuation, preservation, stopped-evolution, and provisional qualification evidence.
- `.github/workflows/dc-final-001-r9.yml` — exact-head Linux R9 scope, deterministic campaign replay, preservation, D-087, PR #44, evidence reproducibility, and artifact upload.
- `digital-protocell/crates/chemistry-core/src/mesh_fission.rs` — R10 V4 effective signed-load observer/helper, unchanged-threshold magnitude-based segment-apposition readiness, clone-only audit path, and daughter parent-source correspondence.
- `digital-protocell/examples/dcfinal001_r10_signed_stress_audit.rs` — R10 observer-only R9 replay and compression-candidate daughter counterfactual entry point.
- `digital-protocell/examples/dcfinal001_r10_signed_stress.rs` — R10 exact R9 control, signed-stress ten-arm reproduction, inherited-state daughter continuation, and energy closure entry point.
- `digital-protocell/examples/dcfinal001_r10_evolution.rs` — R10 production-V4 powered mutation, Resource/Damage, mutation-off, and fixed reversal population entry point.
- `digital-protocell/experiments/dcfinal001_r10_evidence.py` — deterministic compact R10 evidence generator separating robust reproduction and lawful variation from absent selection/reversal.
- `digital-protocell/experiments/generated/dcfinal001r10/` — R10 authority, signed-stress, reproduction, full-state daughters, powered mutation, selection boundary, preservation, final matrix, and qualification evidence.
- `.github/workflows/dc-final-001-r10.yml` — exact-head Linux R10 authority, deterministic replay, preservation, D-087, PR #44, evidence reproducibility, and artifact upload.
- `digital-protocell/examples/dcfinal001_r10r2_evolution.rs` — dedicated fixed-14,778-step R10 production-population entry point preserving the historical 2,500-step R10 executable.
- `digital-protocell/examples/dcfinal001_r4_evolution.rs` — shared R10/R10R2 production-population implementation with parameterized assay horizon and observer-only prefix, lineage, phenotype, turnover, and physical-blocker ledgers.
- `digital-protocell/experiments/dcfinal001_r10r2_evidence.py` — deterministic R10R2 evidence generator for horizon-only scope, exact prefix parity, Gate-4 turnover stop, closure, preservation, and qualification.
- `digital-protocell/experiments/generated/dcfinal001r10r2/` — R10R2 authority, horizon/prefix, turnover, D096 phenotype, closure, preservation, stopped-selection/reversal, final matrix, qualification, and manifest evidence.
- `.github/workflows/dc-final-001-r10r2.yml` — exact-head Linux R10 authority/scope, short-prefix and long-horizon replay, reproduction/M1-M3 preservation, D-087, PR #44, evidence reproducibility, and artifact upload.
- `digital-protocell/crates/chemistry-core/src/d096_allocation.rs` — preserves historical D096-v1 and adds explicitly versioned D096-v2 activated-material catalyst synthesis with precursor/activation/maintenance/turnover ledgers.
- `digital-protocell/crates/chemistry-core/src/material_mesh.rs` — explicit opt-in stamping for the D096-v2 equation/schema identity; historical allocation stamping remains unchanged.
- `digital-protocell/examples/dcfinal001_r10r3_d096v1_reproduction.rs` — exact ten-arm R10 production campaign with continuous neutral D096-v1 expression and mutation off.
- `digital-protocell/examples/dcfinal001_r10r3_budget_diagnostics.rs` — matched Resource/Damage generation-1 structural and activated-energy budget decomposition for D096-v1, D096-off, and the observer-only activated-material candidate.
- `digital-protocell/examples/dcfinal001_r10r3_d096v2_reproduction.rs` — decisive ten-arm R10 production campaign with continuous versioned D096-v2 expression and mutation off.
- `digital-protocell/experiments/dcfinal001_r10r3_evidence.py` — deterministic R10R3 evidence generator sealing integration, structural attribution, versioned chemistry, failed robust reproduction, and downstream Gate-10 stop.
- `digital-protocell/experiments/generated/dcfinal001r10r3/` — compact R10R3 authority, structural budget, v1/v2 reproduction, closure, preservation, stopped evolution, final matrix, and qualification evidence.
- `.github/workflows/dc-final-001-r10r3.yml` — exact-head Linux R10R3 authority, deterministic v1/budget/v2 replay, R10/D096 preservation, D-087, PR #44, evidence reproducibility, and artifact upload.
- `digital-protocell/crates/chemistry-core/src/d096_allocation.rs` — additionally owns explicitly versioned D096-v3, preserving v2 activated-material expression while computing functional gain from catalyst concentration.
- `digital-protocell/crates/chemistry-core/src/material_mesh.rs` — additionally provides opt-in D096-v3 equation/schema stamping without migrating v1 or v2.
- `digital-protocell/examples/dcfinal001_r10r4_gain_audit.rs` — R10R4 observer-only scale, fission, cost-only, gain-channel, and intensive counterfactual entry point.
- `digital-protocell/examples/dcfinal001_r10r4_d096v3_reproduction.rs` — exact ten-arm R10 campaign with continuously active neutral D096-v3 and mutation off.
- `digital-protocell/experiments/dcfinal001_r10r4_evidence.py` — deterministic R10R4 dimensional, causal, versioned-contract, reproduction, and Gate-10 stop evidence generator.
- `digital-protocell/experiments/generated/dcfinal001r10r4/` — compact R10R4 authority, gain audits, v3 conservation/reproduction, preserved upstream state, stopped evolution, qualification, and manifest evidence.
- `.github/workflows/dc-final-001-r10r4.yml` — exact-head Linux R10R4 authority, deterministic gain/reproduction replay, historical preservation, D-087, PR #44, evidence reproducibility, and artifact upload.
## R10R9R5 active files

- `digital-protocell/examples/dcfinal001_r4_evolution.rs` — shared R10/R10R9R5 population implementation with typed lifecycle boundaries, retained legal depletion/rejections, and raw lifecycle events; legacy control remains available.
- `digital-protocell/examples/dcfinal001_r10r9r5_evolution.rs` — canonical R5 population entry point.
- `digital-protocell/experiments/dcfinal001_r10r9r5_evidence.py` — independent raw event/census verifier and evidence generator.
- `.github/workflows/dc-final-001-r10r9r5.yml` — exact-head R5 validation and artifact workflow.

## DC-M4 reproductive-attractor architecture gate

- `digital-protocell/examples/dcm4_reproductive_attractor.rs` — observer-only entry point for the ten-arm Resource/fixture material-geometry and frozen-mechanics stability gate; no production biology changes.
- `digital-protocell/experiments/dcm4_reproductive_attractor_verify.py` — independent verifier for fission authority, observer invariance, causal ledger, stability records, external-prior-art classifications, route specification, and fail-closed scope checks.
- `.github/workflows/dc-m4-reproductive-attractor.yml` — exact-head Linux observer compilation, preserved chemistry/topology tests, ten-arm diagnostic run, verifier, and artifact sealing.
- `experiments/generated/dcm4reproductiveattractor/` — CI-sealed architecture-gate evidence root containing protocol, raw trajectories, material-geometry ledger, frozen stability records, fission-authority audit, route decision, qualification, and manifest. The large generated payload is retained in the immutable CI artifact rather than committed to source.

## DC-M4 R1 local conservative growth coupling

- `digital-protocell/crates/chemistry-core/src/mesh_growth.rs` — owns the frozen D-088 growth calculation and the opt-in V4 `GrowthPlacementMode::LocalCompressionNeighborV1` conservative one-hop routing helper.
- `digital-protocell/examples/dcfinal001_r4_evolution.rs` — shares the explicit growth-placement mode through the current R10/R5 transition; existing callers remain `FrozenD088` and the R1 runner opts in only for its matched Resource comparison.
- `digital-protocell/examples/dcm4_r1_local_conservative_growth_coupling.rs` — bounded ten-arm Route-A producer; it does not launch population selection or a successor.
- `digital-protocell/experiments/dcm4_r1_local_conservative_growth_coupling_verify.py` — independent conservation/locality, source-contract, route-off parity, and E2/E3 verifier.
- `.github/workflows/dc-m4-r1-local-conservative-growth-coupling.yml` — exact-head Linux build, focused tests, bounded Route-A run, fail-closed verifier, and artifact upload.
- `experiments/generated/dcm4r1localconservativegrowthcoupling/` — CI-sealed raw Route-A comparison and bounded qualification evidence.

- Final R1 evidence: hosted workflow `34773172149` at governed head `cd6cb020ff95b69fdf3d65cea79a611cc9d40036`; artifact ZIP digest `sha256:c6483571177e3e4081962d0187c6eaa21186ceb5d8a9f0280849e5b0523641a8`; Route-A E2 classification `RESOURCE_LOCAL_GROWTH_COUPLING_FAILS_TO_AMPLIFY_MODES`.

## DC-M4 R2 endogenous polarity substrate gate

- `digital-protocell/experiments/dcm4_r2_endogenous_polarity_substrate_verify.py` — diagnostic-only R1 reconciliation, current causal-state ownership inventory, standalone mass-conserving reaction-diffusion benchmark, and fail-closed Route-B architecture decision.
- `digital-protocell/experiments/fixtures/dcm4r2/r1_e2_compact.json` — compact immutable R1 E2 paired summary used for independent CI recomputation without carrying the large ignored raw trajectory.
- `.github/workflows/dc-m4-r2-endogenous-polarity-substrate.yml` — exact-R1-descendant scope check, Python verifier, manifest verification, and diagnostic artifact upload; it does not run production Rust or experiments.
- `experiments/generated/dcm4r2endogenouspolaritysubstrate/` — CI-sealed R2 authority, state inventory, benchmark, symbolic substrate contract, architecture decision, and manifest evidence.

## DC-M4 R3 conserved polarity substrate

- `digital-protocell/crates/regulatory-core/src/polarity_mass.rs` — opt-in amount-based `PolarityMassStateV1` substrate with local conservative reaction/transport, source/A/W ledgers, and remesh/restart/fission partition helpers; not wired to production transitions.
- `digital-protocell/examples/dcm4_r3_conserved_polarity_substrate.rs` — standalone R3 seal, isolated benchmark, and held-out Resource pre-fission diagnostic runner.
- `digital-protocell/experiments/dcm4_r3_conserved_polarity_substrate_verify.py` — independent fail-closed verifier for source scope, parameter sealing, mode recomputation, conservation, and matched Route-ON/null qualification.
- `digital-protocell/experiments/fixtures/dcm4r3/held_out_resource_histories.json` — compact immutable projection of ten accepted R1 Resource snapshots used to seed matched diagnostic histories.
- `.github/workflows/dc-m4-r3-conserved-polarity-substrate.yml` — exact-head hosted R3 source-scope, test, benchmark, held-out verifier, manifest, and artifact workflow.

## DC-M4 R4 polarity-to-paid-actuation

- `digital-protocell/crates/regulatory-core/src/polarity_actuation.rs` — opt-in local adapter from accepted R3 active-polarity amounts to the existing bounded actuator activity domain; no force or energy law.
- `digital-protocell/crates/regulatory-core/src/polarity_mass.rs` — equation-derived homogeneous R3 active reference used by the sealed R4 adapter.
- `digital-protocell/examples/dcfinal001_r5_v4_neck.rs` — shared R9/R10 mechanics path with the narrow optional polarity activity input; existing disconnected callers delegate unchanged.
- `digital-protocell/examples/dcfinal001_r4_evolution.rs` — shared current population lifecycle with transactional polarity sidecar, conservative remesh/fission partition, separate chemistry/mechanical ledgers, and the R4 ten-arm runner.
- `digital-protocell/examples/dcm4_r4_polarity_paid_actuation.rs` — R4 seal and matched disconnected/connected coherent-Resource diagnostic entry point; no E3/E4 launch after E2 failure.
- `digital-protocell/experiments/dcm4_r4_polarity_paid_actuation_verify.py` — independent fail-closed verifier for actuator contract, raw vertices/modes, paired E2 predicates, material and energy ledgers, and terminal classification.
- `.github/workflows/dc-m4-r4-polarity-to-paid-actuation.yml` — exact-head hosted R4 authority, focused tests, prospective seal, bounded matched execution, independent verification and artifact upload.
- `experiments/generated/dcm4r4polaritypaidactuation/` — R4 actuator audit, coupling seal, raw matched arms, independent verification, preservation, E2 bounded negative, and explicit unreached E3/E4 evidence.

R4 hosted seal: workflow `34801587951` on `c778e2a4fc858354b7c661a091837e6cc1dab5ee`; artifact ZIP `07b3c062065a58d832ac2c93a5b296b654bfaeeeab6f89081f086f97a007e78a`; binary `fbd0607c1867ca2c661da17d51ca36980e1eed87e63e1f70ff54154335d7fb82`.

## DC-M4 R5 mechanochemical-instability attribution

- `digital-protocell/experiments/dcm4_r5_mechanochemical_instability_verify.py` — observer-only independent recomputation of sealed R4 polarity, activity, funding, shape, frozen-stability and modal-projection evidence; no production transition.
- `.github/workflows/dc-m4-r5-mechanochemical-instability.yml` — exact-R4-descendant source-scope check, sealed R4 artifact download/hash verification, attribution run, fail-closed terminal decision and artifact upload.
- `experiments/generated/dcm4r5mechanochemicalinstability/` — R4 authority, causal graph, source/trajectory audit, matched checkpoint ledger, group comparison, frozen stability/modal projection, geometry feedback, prior art, verifier, preservation, decision and qualification evidence.
- Terminal classification is `R4_ATTRIBUTION_INCONCLUSIVE`: all valid frozen passive modes decay, but the sealed R4 schema does not identify a full coupled local Jacobian or exact pre-mechanics polarity-force projection. No successor mechanism or reproduction run was started.

## DC-M4 R6 atomic coupled-state replay

- `digital-protocell/examples/dcfinal001_r4_evolution.rs` — unchanged R4 current-kernel lifecycle plus R6 observer-only complete pre-mechanics snapshots, exact clone replay, and local central-difference boundary response summaries.
- `digital-protocell/examples/dcm4_r6_atomic_coupled_state.rs` — R6 fixed-checkpoint seal and ten-arm connected/disconnected diagnostic runner; no production coupling or reproduction stage.
- `digital-protocell/experiments/dcm4_r6_atomic_coupled_state_verify.py` — independent state-completeness, replay-identity, same-state response, finite-difference convergence and fail-closed terminal verifier.
- `.github/workflows/dc-m4-r6-atomic-coupled-state.yml` — exact-head authority/source-scope, R4 artifact input hash, prospective checkpoint seal, observer run, independent verification, manifest and upload workflow.
- `experiments/generated/dcm4r6atomiccoupledstate/` — R6 raw snapshots, authority, state contract, replay identity, response operator, causal comparison, preservation, verifier, decision, qualification and manifest evidence.
- R6 terminal classification is `R4_COUPLED_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE`; full closed-loop Jacobian, reproduction, selection, reversal and final integration remain unestablished.

## DC-M4 R7 full-cycle same-boundary coupled response identification

- `digital-protocell/examples/dcfinal001_r4_evolution.rs` — current R4 lifecycle plus the observer-only pre-polarity boundary snapshot, complete-cycle replay, R6-contract response probes and fail-closed non-smooth/incomplete operator summaries.
- `digital-protocell/examples/dcm4_r7_full_cycle_same_boundary.rs` — R7 fixed-checkpoint connected/disconnected diagnostic runner; no production coupling or reproduction stage.
- `digital-protocell/experiments/dcm4_r7_full_cycle_same_boundary_verify.py` — independent R7 state-completeness, exact same-phase replay, response-contract and terminal-classification verifier.
- `.github/workflows/dc-m4-r7-full-cycle-same-boundary.yml` — exact-head source-scope, focused checks, seal, full-cycle observer run, independent verification, manifest and artifact workflow.
- `experiments/generated/dcm4r7fullcyclesameboundary/` — R7 raw qualification, same-phase replay identity, response operator, connected/disconnected comparison, preservation, decision and manifest evidence. Hosted R7 terminal classification is `R4_FULL_CYCLE_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE`; no downstream reproduction, selection or reversal was executed.

- R7 authority/readback: final hosted governance tip `e1c8d5dd63fcdce3d19edbc12bb9e53674e3864c`, workflow `34854828443`, artifact `sha256:1225f713fd2eeb645e7d8cbe5ae8451e1edf1e2ffae0d49544c2eee3818d6795`; Notion R7 and Digital Cell SOT updated/read back. No successor started.

## DC-M4 R8 nonlinear mechanochemical causal-edge decomposition

- `digital-protocell/examples/dcfinal001_r4_evolution.rs` — current lifecycle plus observer-only clone-local P_TO_M and G_TO_P edge cuts and matched modular response records; the point-vector norm uses the independently verified flattened Euclidean convention.
- `digital-protocell/examples/dcm4_r8_nonlinear_mechanochemical_causal_edge.rs` — R8 authority/seal and ten-history, three-checkpoint diagnostic runner; it does not launch reproduction, population, selection or reversal.
- `digital-protocell/experiments/dcm4_r8_nonlinear_mechanochemical_causal_edge_verify.py` — independent fail-closed verifier for R7 authority, clone equality, edge cuts, sign symmetry, conservation, locality/coordinates, branch handling, artifact manifest and the modular terminal predicate.
- `.github/workflows/dc-m4-r8-nonlinear-mechanochemical-causal-edge.yml` — exact-head focused checks, contract seal, observer-only execution, independent verification, downstream-stop assertion and artifact upload.
- `experiments/generated/dcm4r8nonlinearmechanochemicalcausaledge/` — hosted R8 authority, graph, coordinates, perturbation contract, P_TO_M/G_TO_P raw summaries, modular indicators, preservation, decision, qualification and manifest evidence.
- R8 hosted seal: workflow `34865117387` on implementation head `023248e30a454d51a2a6f6c3450b0221659abf65`; artifact ZIP `sha256:384b92b4c1dcd5609e608f7b8bf8c86ce2e95b74c016bb18ae6b42dbd9b6dbf9`; binary `ec852cd9110577406103c40003f5f39292aad0d0f58d1927864211cf2b920812`; terminal classification `R4_GEOMETRY_TO_POLARITY_FEEDBACK_DAMPING`.

## DC-M4 R9 local mechanosensitive polarity activation

- `digital-protocell/crates/regulatory-core/src/polarity_mass.rs` — opt-in strain-aware extension of the existing R3 polarity advance; legacy `advance` remains the Route-OFF path.
- `digital-protocell/examples/dcfinal001_r4_evolution.rs` — opt-in production wiring and clone-local R9 G_TO_P diagnostic path; no default mechanics output or reproduction stage is enabled.
- `digital-protocell/examples/dcm4_r9_local_mechanosensitive_polarity_activation.rs` — R9 contract seal and fixed R8 checkpoint runner.
- `digital-protocell/experiments/dcm4_r9_local_mechanosensitive_polarity_activation_verify.py` — independent source, ledger, sign-symmetry, conservation and terminal-predicate verifier.
- `.github/workflows/dc-m4-r9-local-mechanosensitive-polarity-activation.yml` — exact-head authority, focused checks, contract seal, fixed diagnostic execution, independent verification and artifact upload.
- `experiments/generated/dcm4r9localmechanosensitivepolarityactivation/` — reserved hosted R9 contract, raw diagnostic, verifier and manifest evidence root. E2 and downstream statuses remain pending; PR #44 is untouched.
