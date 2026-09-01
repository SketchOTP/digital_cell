# Project Learning Ledger Template

After adoption, this append-only ledger records durable, verified project knowledge only.

## Entry guidance after adoption

Use live learning headings only after adoption. The following schema is instructional and is not a live entry:

Allowed confidence values: `VERIFIED`, `SUPPORTED`, `INFERRED`, `UNRESOLVED`. Do not rewrite earlier entries; append corrections referencing the original.

## L-DCDEV001A-001

- Learning ID: L-DCDEV001A-001
- Date: 2026-08-15
- Fact or lesson: External ALife implementations are useful as local sensor, effector, developmental, and signaling pattern references, but none is a safe drop-in organism authority for Digital Cell.
- Evidence location: digital-protocell/docs/strategy/developmental_sensorimotor/external_landscape.md and generated external_sources.json.
- Confidence: VERIFIED
- Scope: DC-DEV-001A architecture selection.
- Supersedes learning: none

## L-DCDEV021-ENTRY014-001

- Learning ID: L-DCDEV021-ENTRY014-001
- Date: 2026-08-31
- Fact or lesson: A mathematical reimplementation of the published deterministic M2071 active/inactive-GTPase plus delayed F-actin negative-feedback equations produces a stationary non-homogeneous Polar state and a moving dominant mode-2 Traveling-Wave state. Using the versioned supplementary XML Traveling-Wave `b=0.00067` value, both qualitative regimes survive unchanged-parameter transfer to 24 periodic sites; `u+v` reaction exchange and periodic diffusion sums conserve to numerical precision. This is an isolated reference-transfer feasibility result only; no Digital Cell production mechanism or autonomous resource acquisition is established.
- Evidence location: `digital-protocell/examples/dcdev021_m2_entry014.rs`, `digital-protocell/experiments/generated/dcdev021m2entry014/`, and `.github/workflows/dc-dev-021-m2-entry014.yml`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-014 isolated excitable-polarity reference-transfer audit
- Supersedes learning: none

## L-DCDEV021-ENTRY014-002

- Learning ID: L-DCDEV021-ENTRY014-002
- Date: 2026-08-31
- Fact or lesson: Exact-head Linux CI `33458598747` independently validated the isolated ENTRY-014 result at `32f7380eedbfca063ba23fed2609dee0680d4294`: the versioned M2071 Polar and Traveling-Wave regimes reproduced and both transferred to 24 periodic sites with no parameter search, stochastic forcing, or Digital Cell runtime coupling. The result is a transferable reference substrate only; Architect acceptance and any production integration remain pending.
- Evidence location: `digital-protocell/experiments/generated/dcdev021m2entry014/`, `.github/workflows/dc-dev-021-m2-entry014.yml`, and GitHub Actions run `33458598747`.
- Confidence: VERIFIED
- Scope: DC-DEV-021 ENTRY-014 isolated excitable-polarity reference-transfer audit
- Supersedes learning: L-DCDEV021-ENTRY014-001

## L-DCDEV021-ENTRY010-001

- Learning ID: L-DCDEV021-ENTRY010-001
- Date: 2026-08-31
- Fact or lesson: In the exact ENTRY-009 frozen M2 fixture, unchanged DC-DEV-008 transfer produces `0.1474334734486514` N and F per species. The first successful transfer and first divergence from a matched contact-without-transfer replay occur at step `0`. Existing mechanics can subsequently alter area because its local pressure reads N/F, so raw concentrations are geometry-confounded; reconstructing concentration multiplied by actual V4 mesh area yields existing internal N, F, and combined N+F material amounts that remain distinguishable from no-transfer for all `480` accepted steps. The fixture does not call `reactions_step`, so A/W/C are not causally changed by uptake within this assay.
- Evidence location: `digital-protocell/examples/dcdev021_m2_entry010.rs` and `digital-protocell/experiments/generated/dcdev021m2entry010/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-010 observer-only post-ingestive material-signal substrate audit; exact-head remote CI and Architect review pending.
- Supersedes learning: none

## L-DCDEV021-ENTRY010-002

- Learning ID: L-DCDEV021-ENTRY010-002
- Date: 2026-08-31
- Fact or lesson: Exact-head Linux CI independently confirms that unchanged finite N/F transfer creates a reusable existing internal material signal in the frozen ENTRY-009 fixture: actual-area reconstructed N, F, and combined N+F material amounts first diverge from matched contact-without-transfer at step `0` and remain distinguishable for all `480` accepted steps. Raw concentrations remain geometry-confounded because existing mechanics reads N/F. Activated metabolism is not advanced in this fixture, so A/W/C are not transfer-causal here. This result establishes signal-substrate availability only, not exploitation or autonomous acquisition.
- Evidence location: `digital-protocell/experiments/generated/dcdev021m2entry010/`, GitHub Actions run `33413197919`, and artifact digest `sha256:ac3b34585a38078043a8830e5e0a5664461229f72870abda7636c6c1cfce8491`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-010 observer-only post-ingestive material-signal substrate audit; Architect review pending.
- Supersedes learning: L-DCDEV021-ENTRY010-001

## L-DCDEV020-POST-M1-BASELINE-001

- Learning ID: L-DCDEV020-POST-M1-BASELINE-001
- Date: 2026-08-29
- Fact or lesson: The accepted M1 implementation can be separated from its development history by retaining the V4 scientific/runtime closure and reusable foundation crates while omitting generated experiment outputs and superseded audit workflows from the active baseline.
- Evidence location: `digital-protocell/experiments/generated/dcdev020postm1baseline001/capability_inventory.json` and the accepted source closure at `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1closure001/`.
- Confidence: PROVISIONAL
- Scope: Post-M1 clean capability baseline extraction.
- Supersedes learning: none

## L-DCDEV001A-002

- Learning ID: L-DCDEV001A-002
- Date: 2026-08-15
- Fact or lesson: The clean scientific base is preserved when later R4/D-096 source and evidence are excluded and only governance/operations are carried forward or reconstructed.
- Evidence location: digital-protocell/docs/strategy/developmental_sensorimotor/clean_scientific_base.md and governance_carryforward.json.
- Confidence: VERIFIED
- Scope: DC-DEV-001A Gate 0.
- Supersedes learning: none

## L-DCDEV001A-R1

- Learning ID: L-DCDEV001A-R1
- Date: 2026-08-15
- Fact or lesson: First-slice contracts must expose only observer-coupled regulatory state, neighbor signal, local transduced input, and provenance; effector and motor outputs remain deferred.
- Evidence location: digital-protocell/docs/strategy/developmental_sensorimotor/first_implementation_contract.md and generated first_implementation_contract.json.
- Confidence: VERIFIED
- Scope: DC-DEV-001A-R1.
- Supersedes learning: none

## L-DCDEV003-001

- Learning ID: L-DCDEV003-001
- Date: 2026-08-15
- Fact or lesson: Regulatory continuity can be kept observer-only by deriving independent nearest-local old-to-new mappings from immutable material frames; explicit fission and unknown topology events must fail closed.
- Evidence location: `digital-protocell/crates/regulatory-core/src/continuity.rs`, `digital-protocell/experiments/generated/dcdev003/`, and `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev003/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-003 bounded continuity assay.
- Supersedes learning: none

## L-DCDEV004-001

- Learning ID: L-DCDEV004-001
- Date: 2026-08-15
- Fact or lesson: A single local contractile edge-tension rule can be funded by existing D-091 reserve R, spend R into existing W, preserve exact zero-activity mechanics parity, and close a local tensile sensorimotor loop without target geometry or a central action selector.
- Evidence location: `digital-protocell/crates/regulatory-core/src/contractility.rs`, `digital-protocell/crates/chemistry-core/src/mesh_mechanics.rs`, `digital-protocell/experiments/generated/dcdev004/`, and `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev004/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-004 bounded local contractility assay.
- Supersedes learning: none

## L-DCDEV006-001

- Learning ID: L-DCDEV006-001
- Date: 2026-08-15
- Fact or lesson: A deterministic static geometric obstacle can provide one bounded local contact-force vector and one penetration-normalized external signal while preserving exact zero-contact DC-DEV-005 trajectory parity; existing mechanics remains the movement authority.
- Evidence location: `digital-protocell/crates/regulatory-core/src/spatial.rs`, `digital-protocell/crates/chemistry-core/src/mesh_mechanics.rs`, and `digital-protocell/experiments/generated/dcdev006/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-006 local Gates 0-6 assay; architect review and exact-head remote CI pending.
- Supersedes learning: none

## L-DCDEV009-001

- Learning ID: L-DCDEV009-001
- Date: 2026-08-16
- Fact or lesson: In the accepted fixed-topology free-space arm, local contractility produces large shape change but its equal-and-opposite edge forces have zero net force. The measured active-minus-control centroid drift is reproduced by the changed baseline force field after deformation and is not sufficient evidence of locomotion.
- Evidence location: `digital-protocell/examples/dcdev009_gate_assay.rs`, `digital-protocell/experiments/generated/dcdev009/force_accounting.json`, `digital-protocell/experiments/generated/dcdev009/matched_arms.json`, and `digital-protocell/experiments/generated/dcdev009/artifact_analysis.json`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-009 fixed-topology free-space audit; architect review pending.
- Supersedes learning: none

## L-DCDEV011-001

- Learning ID: L-DCDEV011-001
- Date: 2026-08-16
- Fact or lesson: A single frozen local isotropic stick-slip reaction law, coupled only through existing local attempted velocity and force, retained active reserve-funded displacement through relaxation in the preregistered fixed-topology assay while motor-off and zero-reserve controls remained stationary. This is evidence for the bounded substrate mechanism, not autonomous gait, steering, navigation, resource seeking, learning, or evolution.
- Evidence location: `digital-protocell/crates/regulatory-core/src/stick_slip_traction.rs`, `digital-protocell/examples/dcdev011_gate_assay.rs`, `digital-protocell/experiments/generated/dcdev011/`, and `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev011/qualification_results.md`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-011 local qualification; exact-head remote CI passed, architect review pending.
- Supersedes learning: none

## L-DCDEV013-001

- Learning ID: L-DCDEV013-001
- Date: 2026-08-16
- Fact or lesson: The production local resource-contact signal is physically local and causal through the accepted regulator and funded motor, but under the frozen DC-DEV-013 resource geometry and 480-step horizon the active arm acquired less finite N/F material than both sensor-off and motor-off controls; local sensing and movement therefore did not establish a feeding benefit.
- Evidence location: `digital-protocell/crates/regulatory-core/src/spatial_resource.rs`, `digital-protocell/examples/dcdev013_gate_assay.rs`, `digital-protocell/experiments/generated/dcdev013/`, and `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev013/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-013 frozen local resource-contact feeding assay; exact-head remote CI and architect review pending.
- Supersedes learning: none

## L-DCDEV015-001

- Learning ID: L-DCDEV015-001
- Date: 2026-08-17
- Fact or lesson: Existing finite N/F uptake is conserved and delivers measurable precursor material; existing reactions convert a small fraction to A within 480 steps, but A, R, E_stored, and E_available all move farther from their replete references despite feeding outperforming no-delivery. Intake-to-internal restoration is therefore not established on the behavioral window.
- Evidence location: `digital-protocell/examples/dcdev015_metabolic_restoration_assay.rs`, `digital-protocell/experiments/generated/dcdev015/results.json`, and `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev015/qualification_results.md`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-015 observer-only metabolic intake-to-restoration audit.
- Supersedes learning: none

## L-DCDEV016-001

- Learning ID: L-DCDEV016-001
- Date: 2026-08-17
- Fact or lesson: A single resource inventory derived from the DC-DEV-015 activated-store decline delivered `11.401893960861464` matched N/F units against the `11.387290380605897` target and raised E_available from `60.82781514212436` to `64.13760842349555`, but A, R, and E_stored remained below the deprived starting state. Existing uptake and conversion therefore passed supply sufficiency while stored activated-material restoration remained unsupported.
- Evidence location: `digital-protocell/examples/dcdev016_metabolic_break_even.rs`, `digital-protocell/experiments/generated/dcdev016/results.json`, and `digital-protocell/docs/strategy/developmental_sensorimotor/dcdev016/qualification_results.md`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-016 observer-only metabolic break-even resource sufficiency challenge.
- Supersedes learning: none

## L-DCDEV021-ENTRY003-001

- Learning ID: L-DCDEV021-ENTRY003-001
- Date: 2026-08-29
- Fact or lesson: Under one explicit fixed intrinsic local activity dynamic using only the established regulator/plasticity constants, activity changes and A-to-W funding closes exactly, but the accepted A-funded actuator plus frozen stick-slip traction produces no retained material-centroid displacement above matched controls. Intrinsic local dynamics is therefore not by itself a qualified exploratory-movement substrate.
- Evidence location: `digital-protocell/crates/regulatory-core/src/intrinsic_exploration.rs`, `digital-protocell/examples/dcdev021_m2_entry003.rs`, and `digital-protocell/experiments/generated/dcdev021m2entry003/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-003 preregistered feasibility assay; exact-head remote CI and Architect review pending.
- Supersedes learning: none

## L-DCDEV021-ENTRY004-001

- Learning ID: L-DCDEV021-ENTRY004-001
- Date: 2026-08-29
- Fact or lesson: On the accepted ENTRY-003 uninterrupted trajectory, adaptation-limited effective activity never creates a local free-step required force above the frozen `0.45` stick-slip limit, whereas the raw intrinsic activity would create threshold crossings on the same clone state. The observed zero-slip regime is therefore explained by existing adaptation attenuation rather than a clutch-ledger parity discrepancy.
- Evidence location: `digital-protocell/examples/dcdev021_m2_entry004.rs` and `digital-protocell/experiments/generated/dcdev021m2entry004/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-004 observer-only traction-transfer audit; exact-head Linux CI and Architect review pending.
- Supersedes learning: none

## L-DCDEV021-ENTRY005-001

- Learning ID: L-DCDEV021-ENTRY005-001
- Date: 2026-08-30
- Fact or lesson: In the explicit opt-in ENTRY-005 composition, retaining the existing adaptation trace inside intrinsic excitation while coupling raw intrinsic activity directly to the accepted A-funded motor crosses the unchanged frozen clutch regime and yields retained target-free substrate-mediated exploration. Applying adaptation again at the motor boundary is therefore not required for refractory intrinsic dynamics and suppresses this otherwise available physical transfer.
- Evidence location: `digital-protocell/crates/regulatory-core/src/intrinsic_exploration.rs`, `digital-protocell/examples/dcdev021_m2_entry005.rs`, and `digital-protocell/experiments/generated/dcdev021m2entry005/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-005 preregistered refractory-only motor feasibility assay; exact-head Linux CI passed and Architect review is pending.
- Supersedes learning: none

## L-DCDEV021-ENTRY006-001

- Learning ID: L-DCDEV021-ENTRY006-001
- Date: 2026-08-30
- Fact or lesson: In the exact frozen DC-DEV-013 finite-resource geometry, qualified target-free ENTRY-005 movement remains materially and energetically conservative but lowers cumulative N/F capture below matched nonexploring controls without changing geometric exposure. Unguided locomotion alone therefore does not establish a resource-acquisition benefit on this fixed assay.
- Evidence location: `digital-protocell/examples/dcdev021_m2_entry006.rs` and `digital-protocell/experiments/generated/dcdev021m2entry006/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-006 preregistered unguided finite-resource acquisition assay; exact-head Linux CI and Architect review pending.
- Supersedes learning: none

## L-DCDEV021-ENTRY011-001

- Learning ID: L-DCDEV021-ENTRY011-001
- Date: 2026-08-31
- Fact or lesson: In the observer-only ENTRY-011 composition, the exact frozen M1/V4 reaction kernel is active after unchanged finite N/F uptake. Relative to the accepted no-metabolism ENTRY-006 arm, metabolism reduces internal N+F buildup at step 116, preserves the uptake driving force, and raises cumulative acquisition from `0.2948669468973028` to `0.8602206124447573` while retaining target-free movement. Local N/F/A/W accounting closes below `4e-13`; this is a composition result only, not autonomous resource acquisition or a behavioral post-ingestive coupling.
- Evidence location: `digital-protocell/examples/dcdev021_m2_entry011.rs`, `digital-protocell/experiments/generated/dcdev021m2entry011/`, and `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev021m2entry011/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-011 frozen uptake/metabolism composition feasibility; exact-head remote CI and Architect review pending.
- Supersedes learning: none

## L-DCDEV021-ENTRY012-001

- Learning ID: L-DCDEV021-ENTRY012-001
- Date: 2026-08-31
- Fact or lesson: Under the preregistered separated ecology, the accepted ENTRY-011 metabolic explorer remained active but did not encounter the unchanged finite resource within 1,500 accepted steps. The exact one-settled-mean-edge-length initial gap was `1.3036408078380952`, the closest midpoint gap remained the same, and the body recorded path `0.33538885163612836`, net displacement `0.03988968845502883`, `9196` slips, and `12` dominant-patch changes. Target-free in-contact exploitation therefore does not imply encounter from this separated placement and fixed horizon.
- Evidence location: `digital-protocell/examples/dcdev021_m2_entry012.rs`, `digital-protocell/experiments/generated/dcdev021m2entry012/`, and `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev021m2entry012/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-012 preregistered separated-resource encounter assay; exact-head Linux CI and Architect review pending.
- Supersedes learning: none

## L-DCDEV021-ENTRY013-001

- Learning ID: L-DCDEV021-ENTRY013-001
- Date: 2026-08-31
- Fact or lesson: In the exact resource-free ENTRY-012 metabolic explorer, the first-harmonic intrinsic/motor polarity is nonzero but decays from `0.243646444843049` maximum to `0.0020155616613880007` at step 1,499. Its unwrapped phase changes by one half-turn (`-3.141592653589797`) without a complete phase cycle; real motion accumulates high path (`0.33538885163612875`) but low net displacement (`0.039889688455029104`, ratio `0.11893564219691585`). The unchanged actuator/traction mechanics can translate a first observed persistent asymmetric profile in an assay-only fixed-profile clone (`0.09316990400571264` net displacement over 480 steps), so the bounded-search diagnosis is polarity decay/homogenization rather than an established mechanical impossibility. No production mechanism changed.
- Evidence location: `digital-protocell/examples/dcdev021_m2_entry013.rs`, `digital-protocell/experiments/generated/dcdev021m2entry013/`, and `/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev021m2entry013/`.
- Confidence: PROVISIONAL
- Scope: DC-DEV-021 ENTRY-013 observer-only intrinsic search-persistence/cancellation audit; exact-head Linux CI and Architect review pending.
- Supersedes learning: none
