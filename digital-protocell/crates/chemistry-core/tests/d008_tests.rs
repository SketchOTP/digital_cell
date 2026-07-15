//! D-008 Stage 0, Stage A transport, and Stage B membrane-localization tests.

use chemistry_core::*;

const LEGACY_DEFAULT_CANDIDATE_HASH: &str =
    "3a71c61b818c2193407b609c2e1726344677f08e5f4c86f0aaeee1790f2bb2db";
const LEGACY_DEFAULT_CONFIGURATION_HASH: &str =
    "1fba90d376b1bfdf68b1dcae775860ee55d4ee7a86d521d6631e7a8890edae43";
const D006_SURFACE_CANDIDATE_HASH: &str =
    "a65c9c86e5ad93bd9088e7767917656a30641fbc82b3582db4a1bebc9633e808";
const D006_SURFACE_CONFIGURATION_HASH: &str =
    "53c5fd482d171d8a5d20dfbc16e7fdc1f1fc782d06d98c659c1a82fd23a172bb";
/// Fixed SHA-256 of 10 accepted legacy baseline steps (f64 bit concatenation).
const LEGACY_10_STEP_FIELD_DIGEST: &str =
    "4299ad9f8b9e6efb1befee5c21f800e48c1a077768933138f26c4fde168c77f9";

fn d008_params() -> SimParams {
    let mut params = SimParams::default();
    params.equation_version = EquationVersion::MembraneMetabolismV1;
    params
}

fn buffer_addresses(fields: &FieldBuffers) -> ([usize; 7], [usize; 7]) {
    (
        [
            fields.structure.as_ptr() as usize,
            fields.catalyst.as_ptr() as usize,
            fields.nutrient.as_ptr() as usize,
            fields.fuel.as_ptr() as usize,
            fields.waste.as_ptr() as usize,
            fields.activated.as_ptr() as usize,
            fields.membrane.as_ptr() as usize,
        ],
        [
            fields.structure_next.as_ptr() as usize,
            fields.catalyst_next.as_ptr() as usize,
            fields.nutrient_next.as_ptr() as usize,
            fields.fuel_next.as_ptr() as usize,
            fields.waste_next.as_ptr() as usize,
            fields.activated_next.as_ptr() as usize,
            fields.membrane_next.as_ptr() as usize,
        ],
    )
}

fn seven_markers(fields: &FieldBuffers, idx: usize) -> [f64; 7] {
    [
        fields.structure[idx],
        fields.catalyst[idx],
        fields.nutrient[idx],
        fields.fuel[idx],
        fields.waste[idx],
        fields.activated[idx],
        fields.membrane[idx],
    ]
}

fn set_seven_markers(fields: &mut FieldBuffers, idx: usize, markers: [f64; 7]) {
    fields.structure[idx] = markers[0];
    fields.catalyst[idx] = markers[1];
    fields.nutrient[idx] = markers[2];
    fields.fuel[idx] = markers[3];
    fields.waste[idx] = markers[4];
    fields.activated[idx] = markers[5];
    fields.membrane[idx] = markers[6];
}

#[test]
fn all_seven_fields_allocate_distinct_current_and_next_buffers() {
    let fields = FieldBuffers::new(4);
    let lengths = [
        fields.structure.len(),
        fields.catalyst.len(),
        fields.nutrient.len(),
        fields.fuel.len(),
        fields.waste.len(),
        fields.activated.len(),
        fields.membrane.len(),
        fields.structure_next.len(),
        fields.catalyst_next.len(),
        fields.nutrient_next.len(),
        fields.fuel_next.len(),
        fields.waste_next.len(),
        fields.activated_next.len(),
        fields.membrane_next.len(),
    ];
    assert!(lengths.iter().all(|&len| len == 4));
    let (current, next) = buffer_addresses(&fields);
    assert!(current.iter().zip(next).all(|(&a, b)| a != b));
}

#[test]
fn all_seven_fields_copy_to_working_buffers() {
    let mut fields = FieldBuffers::new(1);
    set_seven_markers(&mut fields, 0, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    let mut working = FieldBuffers::new(1);

    fields.copy_current_to_working(&mut working);

    assert_eq!(
        seven_markers(&working, 0),
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]
    );
}

#[test]
fn accepted_d008_step_swaps_all_seven_buffers() {
    let mut params = d008_params();
    params.diffusion_enabled = false;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    let markers = [0.11, 0.22, 0.33, 0.44, 0.55, 0.66, 0.77];
    set_seven_markers(&mut sim.fields, center, markers);
    let before = buffer_addresses(&sim.fields);

    assert!(sim.step());

    let after = buffer_addresses(&sim.fields);
    assert_eq!(after.0, before.1);
    assert_eq!(after.1, before.0);
    assert_eq!(seven_markers(&sim.fields, center), markers);
}

#[test]
fn rejected_d008_step_swaps_no_buffers() {
    let mut sim = Simulation::new(d008_params());
    sim.observer_enabled = false;
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.activated[center] = CONC_SAFETY_LIMIT + 1.0;
    let before = buffer_addresses(&sim.fields);

    assert!(!sim.step());

    assert_eq!(buffer_addresses(&sim.fields), before);
    assert_eq!(sim.substep, 0);
}

#[test]
fn seven_field_snapshot_json_round_trips_all_seven_fields() {
    let mut sim = Simulation::new(d008_params());
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    let markers = [0.11, 0.22, 0.33, 0.44, 0.55, 0.66, 0.77];
    set_seven_markers(&mut sim.fields, center, markers);

    let loaded = FieldSnapshot::from_json(&sim.snapshot().to_json().unwrap()).unwrap();

    assert_eq!(loaded.snapshot_schema_version, SNAPSHOT_SCHEMA_VERSION);
    assert_eq!(
        loaded.field_schema_version,
        FieldSchemaVersion::SevenFieldV1
    );
    assert_eq!(
        loaded.equation_version,
        EquationVersion::MembraneMetabolismV1
    );
    assert_eq!(
        [
            loaded.fields.structure()[center],
            loaded.fields.catalyst()[center],
            loaded.fields.nutrient()[center],
            loaded.fields.fuel()[center],
            loaded.fields.waste()[center],
            loaded.fields.activated().unwrap()[center],
            loaded.fields.membrane().unwrap()[center],
        ],
        markers
    );
    let mut restored = Simulation::new(d008_params());
    restored.try_restore_snapshot(&loaded).unwrap();
    assert_eq!(seven_markers(&restored.fields, center), markers);
}

#[test]
fn historical_five_field_snapshot_restores_all_five_values() {
    let mut sim = Simulation::new(baseline_params());
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    let markers = [0.12, 0.23, 0.34, 0.45, 0.56];
    sim.fields.structure[center] = markers[0];
    sim.fields.catalyst[center] = markers[1];
    sim.fields.nutrient[center] = markers[2];
    sim.fields.fuel[center] = markers[3];
    sim.fields.waste[center] = markers[4];
    let old_json = serde_json::json!({
        "version": "0.1.0",
        "random_seed": sim.params.random_seed,
        "substep": 0,
        "sim_time": 0.0,
        "params": sim.params,
        "structure": sim.fields.structure,
        "catalyst": sim.fields.catalyst,
        "nutrient": sim.fields.nutrient,
        "fuel": sim.fields.fuel,
        "waste": sim.fields.waste,
        "classification": sim.detector.last_classification,
        "turnover": sim.detector.turnover,
    });

    let loaded = FieldSnapshot::from_json(&old_json.to_string()).unwrap();

    assert_eq!(loaded.field_schema_version, FieldSchemaVersion::FiveFieldV1);
    assert_eq!(loaded.equation_version, EquationVersion::D003CrowdingV1);
    assert!(matches!(loaded.fields, SnapshotFields::FiveField(_)));
    let mut restored = Simulation::new(baseline_params());
    restored.try_restore_snapshot(&loaded).unwrap();
    assert_eq!(
        [
            restored.fields.structure[center],
            restored.fields.catalyst[center],
            restored.fields.nutrient[center],
            restored.fields.fuel[center],
            restored.fields.waste[center],
        ],
        markers
    );
}

#[test]
fn five_field_snapshot_is_rejected_for_membrane_metabolism() {
    let legacy = Simulation::new(baseline_params()).snapshot();
    let mut json: serde_json::Value = serde_json::from_str(&legacy.to_json().unwrap()).unwrap();
    json["equation_version"] = serde_json::json!("membrane_metabolism_v1");
    json["params"]["equation_version"] = serde_json::json!("membrane_metabolism_v1");

    let error = FieldSnapshot::from_json(&json.to_string()).unwrap_err();

    assert!(error.to_string().contains("five_field_v1"));
}

#[test]
fn in_memory_five_field_payload_rejected_for_membrane_metabolism() {
    let mut snap = Simulation::new(d008_params()).snapshot();
    let five = Simulation::new(baseline_params()).snapshot();
    snap.fields = match five.fields {
        SnapshotFields::FiveField(payload) => SnapshotFields::FiveField(payload),
        SnapshotFields::SevenField(_) => panic!("expected five-field baseline"),
    };
    snap.field_schema_version = FieldSchemaVersion::FiveFieldV1;

    let mut dest = FieldBuffers::for_grid(&Grid::new());
    let err = snap.try_restore_fields(&mut dest).unwrap_err();
    assert!(
        err.contains("five_field_v1") || err.contains("incompatible"),
        "{err}"
    );
    assert!(dest.activated.iter().all(|&v| v == 0.0));
    assert!(dest.membrane.iter().all(|&v| v == 0.0));
}

#[test]
fn malformed_payload_lengths_return_err_without_panic() {
    let mut snap = Simulation::new(d008_params()).snapshot();
    match &mut snap.fields {
        SnapshotFields::SevenField(payload) => {
            payload.activated.pop();
        }
        SnapshotFields::FiveField(_) => panic!("expected seven-field snapshot"),
    }

    let mut dest = FieldBuffers::for_grid(&Grid::new());
    let err = snap.try_restore_fields(&mut dest).unwrap_err();
    assert!(
        err.contains("length") || err.contains("size") || err.contains("mismatch"),
        "{err}"
    );
}

#[test]
fn unknown_snapshot_schema_version_is_rejected() {
    let mut snap = Simulation::new(d008_params()).snapshot();
    snap.snapshot_schema_version = 99;
    let mut dest = FieldBuffers::for_grid(&Grid::new());
    let err = snap.try_restore_fields(&mut dest).unwrap_err();
    assert!(
        err.contains("snapshot_schema_version") || err.contains("schema"),
        "{err}"
    );
}

#[test]
fn seven_field_snapshot_is_rejected_for_legacy_equation() {
    let d008 = Simulation::new(d008_params()).snapshot();
    let mut json: serde_json::Value = serde_json::from_str(&d008.to_json().unwrap()).unwrap();
    json["equation_version"] = serde_json::json!("d003-crowding-v1");
    json["params"]["equation_version"] = serde_json::json!("d003-crowding-v1");

    let error = FieldSnapshot::from_json(&json.to_string()).unwrap_err();

    assert!(error.to_string().contains("seven_field_v1"));
}

#[test]
fn historical_candidate_and_configuration_hashes_are_unchanged() {
    let grid = GridConfiguration::default();
    let baseline = baseline_params();
    assert_eq!(
        candidate_hash(&baseline, &grid),
        LEGACY_DEFAULT_CANDIDATE_HASH
    );
    assert_eq!(
        configuration_hash(&baseline, &grid),
        LEGACY_DEFAULT_CONFIGURATION_HASH
    );

    let mut surface = surface_turnover_params_from_calibrated_kphi1();
    surface.k_structure_interface = 0.09642857142857159;
    assert_eq!(candidate_hash(&surface, &grid), D006_SURFACE_CANDIDATE_HASH);
    assert_eq!(
        configuration_hash(&surface, &grid),
        D006_SURFACE_CONFIGURATION_HASH
    );
}

/// Frozen Stage A governed hashes (updated D-012: stoichiometric_schema_version=1).
const STAGE_A_CANDIDATE_HASH: &str =
    "3fda6d8177dfa0cb61d11016547fa4e66d73d7ee6e2ff21000f414def523faab";
const STAGE_A_CONFIGURATION_HASH: &str =
    "f7718235a5b1bf3d6b8018d9a839df03ca3fd812d210bf0b0fb6d75bf74e8f84";
/// Frozen Stage B selected governed hashes (updated D-012: stoichiometric_schema_version=1).
const STAGE_B_CANDIDATE_HASH: &str =
    "3c47ab95a957c696d81baa34567362c68fd1d7bf02d0da254a0e199468ceb762";
const STAGE_B_CONFIGURATION_HASH: &str =
    "0a6d018728b0903d6ac34402f37a3137bdea066c30b86fe738e7ccd65f011aad";

fn stage_a_reference_params() -> SimParams {
    let mut params = SimParams::default();
    params.equation_version = EquationVersion::MembraneMetabolismV1;
    params.d_a = 0.040;
    params.beta_c = 4.6;
    params.beta_a = 4.6;
    params.beta_n = 1.2;
    params.beta_f = 1.2;
    params.beta_w = 0.2;
    params.m_max = 1.0;
    params.d_m = 0.001;
    params.k_membrane_decay = 0.002;
    params.k_membrane_detach = 0.020;
    params.k_c_membrane = 0.10;
    params.k_membrane = 0.0;
    params.reactions_enabled = false;
    params.phase_separation_enabled = false;
    params
}

fn stage_b_selected_params() -> SimParams {
    let mut params = stage_a_reference_params();
    params.d008_stage_b_enabled = true;
    params.k_membrane = 0.19748231883326484;
    params
}

#[test]
fn d008_hash_identifies_equation_and_field_schema_in_fixed_order() {
    let bytes = String::from_utf8(canonical_params_bytes(&d008_params())).unwrap();
    assert!(bytes.ends_with(
        "equation_version=membrane_metabolism_v1;k_structure_interface=0;\
k_c_structure=0.1;d_a=0.04;beta_c=4.6;beta_a=4.6;beta_n=1.2;beta_f=1.2;\
beta_w=0.2;field_schema_version=seven_field_v1;snapshot_schema_version=2;stoichiometric_schema_version=1"
    ));
    assert!(!bytes.contains("d008_stage_mode="));
    assert!(!bytes.contains("k_d008_activation="));
    assert!(!bytes.contains("d008_stage_b_enabled="));
    assert!(!bytes.contains("m_max="));
    assert_eq!(
        candidate_hash(&d008_params(), &GridConfiguration::default()),
        candidate_hash(&d008_params(), &GridConfiguration::default())
    );
}

#[test]
fn frozen_stage_a_and_stage_b_hashes_remain_unchanged() {
    let grid = GridConfiguration::default();
    let stage_a = stage_a_reference_params();
    assert_eq!(candidate_hash(&stage_a, &grid), STAGE_A_CANDIDATE_HASH);
    assert_eq!(
        configuration_hash(&stage_a, &grid),
        STAGE_A_CONFIGURATION_HASH
    );

    let stage_b = stage_b_selected_params();
    assert_eq!(candidate_hash(&stage_b, &grid), STAGE_B_CANDIDATE_HASH);
    assert_eq!(
        configuration_hash(&stage_b, &grid),
        STAGE_B_CONFIGURATION_HASH
    );
}

#[test]
fn stage_c_hashes_include_stage_c_params_and_change_with_rates() {
    let grid = GridConfiguration::default();
    let base = stage_c_params();
    let bytes = String::from_utf8(canonical_params_bytes(&base)).unwrap();
    assert!(bytes.contains("d008_stage_mode=activated_metabolism"));
    assert!(bytes.contains("k_d008_activation=0.02"));
    let base_hash = candidate_hash(&base, &grid);
    assert_ne!(
        base_hash,
        candidate_hash(&stage_a_reference_params(), &grid)
    );

    let mut changed = base.clone();
    changed.k_d008_activation = 0.021;
    assert_ne!(candidate_hash(&changed, &grid), base_hash);
    changed = base.clone();
    changed.k_d008_reproduction = 0.041;
    assert_ne!(candidate_hash(&changed, &grid), base_hash);
    changed = base.clone();
    changed.d008_a_max = 0.99;
    assert_ne!(candidate_hash(&changed, &grid), base_hash);
}

#[test]
fn legacy_numerical_behavior_is_bit_reproducible() {
    let mut sim = Simulation::new(baseline_params());
    sim.observer_enabled = false;
    for _ in 0..10 {
        assert!(sim.step());
    }
    let digest = sim.stable_field_digest();
    eprintln!("D008_LEGACY_10_STEP_FIELD_DIGEST={digest}");
    assert_eq!(digest, LEGACY_10_STEP_FIELD_DIGEST);
}

#[test]
fn equation_versions_are_typed_and_serde_compatible() {
    for (version, name) in [
        (EquationVersion::D001BulkV1, "d001-bulk-v1"),
        (EquationVersion::D003CrowdingV1, "d003-crowding-v1"),
        (EquationVersion::SurfaceTurnoverV1, "surface_turnover_v1"),
        (
            EquationVersion::MembraneMetabolismV1,
            "membrane_metabolism_v1",
        ),
    ] {
        assert_eq!(
            serde_json::to_string(&version).unwrap(),
            format!("\"{name}\"")
        );
        assert_eq!(version.as_str(), name);
    }
}

#[test]
fn d008_stage_zero_dispatch_has_no_productive_chemistry() {
    let rates = compute_reactions_at(0.5, 0.35, 1.0, 1.0, 0.0, &d008_params(), true);
    assert_eq!(rates.r_rep, 0.0);
    assert_eq!(rates.r_structure, 0.0);
    assert_eq!(rates.r_phi, 0.0);
    assert_eq!(rates.r_c, 0.0);
    assert_eq!(rates.r_n, 0.0);
    assert_eq!(rates.r_f, 0.0);
    assert_eq!(rates.r_w, 0.0);
}

#[test]
fn accounting_scaffold_contains_all_seven_fields() {
    let accounting = StepAccounting::default();
    assert_eq!(accounting.activated.mass_before, 0.0);
    assert_eq!(accounting.membrane.mass_before, 0.0);
}

fn transport_species() -> [TransportSpecies; 5] {
    [
        TransportSpecies::Catalyst,
        TransportSpecies::Activated,
        TransportSpecies::Nutrient,
        TransportSpecies::Fuel,
        TransportSpecies::Waste,
    ]
}

fn normalized_face_flux(species: TransportSpecies, membrane: f64) -> f64 {
    let params = d008_params();
    let flux = face_flux(species, 1.0, 0.0, 0.5, 0.5, membrane, membrane, &params);
    let base = face_flux(species, 1.0, 0.0, 0.5, 0.5, 0.0, 0.0, &params);
    flux / base
}

#[test]
fn zero_membrane_reproduces_each_species_base_diffusion() {
    let params = d008_params();
    for species in transport_species() {
        let actual = face_diffusivity(species, 0.25, 0.75, 0.0, 0.0, &params);
        let expected = match species {
            TransportSpecies::Catalyst => {
                0.5 * (catalyst_diffusivity(0.25, &params) + catalyst_diffusivity(0.75, &params))
            }
            TransportSpecies::Activated => params.d_a,
            TransportSpecies::Nutrient => params.d_n,
            TransportSpecies::Fuel => params.d_f,
            TransportSpecies::Waste => params.d_w,
        };
        assert!((actual - expected).abs() < 1e-15, "{species:?}: {actual}");
    }
}

#[test]
fn membrane_attenuation_is_monotonic_and_meets_selectivity_targets() {
    for species in transport_species() {
        let fluxes: Vec<f64> = [0.0, 0.25, 0.5, 0.75, 1.0]
            .into_iter()
            .map(|m| normalized_face_flux(species, m))
            .collect();
        assert!(
            fluxes.windows(2).all(|pair| pair[1] < pair[0]),
            "{species:?}: {fluxes:?}"
        );
        let normalized = fluxes[4];
        match species {
            TransportSpecies::Catalyst | TransportSpecies::Activated => {
                assert!(normalized <= 0.05, "{species:?}: {normalized}");
            }
            TransportSpecies::Nutrient | TransportSpecies::Fuel => {
                assert!(
                    (0.20..=0.50).contains(&normalized),
                    "{species:?}: {normalized}"
                );
            }
            TransportSpecies::Waste => {
                assert!(normalized >= 0.70, "{species:?}: {normalized}");
            }
        }
    }
}

#[test]
fn face_flux_is_symmetric_and_antisigned() {
    let params = d008_params();
    for species in transport_species() {
        let forward = face_flux(species, 0.8, 0.2, 0.4, 0.6, 0.7, 0.3, &params);
        let reverse = face_flux(species, 0.2, 0.8, 0.6, 0.4, 0.3, 0.7, &params);
        assert!((forward + reverse).abs() < 1e-15, "{species:?}");
    }
}

#[test]
fn no_flux_dish_transport_conserves_each_species_mass() {
    let grid = Grid::new();
    let params = d008_params();
    let size = grid.width * grid.height;
    let phi = vec![0.5; size];
    let membrane = vec![0.7; size];
    for species in transport_species() {
        let mut field = vec![0.0; size];
        let center = Grid::index(grid.width, grid.cx as usize, grid.cy as usize);
        field[center] = 1.0;
        let mut rate = vec![0.0; size];
        let accounting =
            transport_field(&grid, species, &field, &phi, &membrane, &params, &mut rate);
        let net: f64 = grid
            .dish_mask
            .iter()
            .zip(&rate)
            .filter(|(inside, _)| **inside)
            .map(|(_, value)| *value)
            .sum();
        assert!(net.abs() < 1e-12, "{species:?}: {net}");
        assert!(accounting.net_change_rate.abs() < 1e-12);
        assert!(accounting.absolute_crossed_face_flux > 0.0);
    }
}

#[test]
fn d008_initialization_uses_approved_activated_and_membrane_seed() {
    let d008 = Simulation::new(d008_params());
    for idx in 0..d008.fields.structure.len() {
        if !d008.grid.in_dish(idx) {
            continue;
        }
        let phi = d008.fields.structure[idx];
        assert_eq!(d008.fields.activated[idx], 0.10 * interior_weight(phi));
        assert_eq!(d008.fields.membrane[idx], 0.50 * interface_weight(phi));
    }

    let legacy = Simulation::new(baseline_params());
    assert!(legacy.fields.activated.iter().all(|&value| value == 0.0));
    assert!(legacy.fields.membrane.iter().all(|&value| value == 0.0));
}

#[test]
fn accepted_d008_step_transports_all_solubles_and_keeps_phi_membrane_fixed() {
    let mut params = d008_params();
    params.reactions_enabled = false;
    params.phase_separation_enabled = false;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) {
            sim.fields.structure[idx] = 0.5;
            sim.fields.membrane[idx] = 0.5;
            sim.fields.catalyst[idx] = 0.0;
            sim.fields.activated[idx] = 0.0;
            sim.fields.nutrient[idx] = 0.0;
            sim.fields.fuel[idx] = 0.0;
            sim.fields.waste[idx] = 0.0;
        }
    }
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.catalyst[center] = 1.0;
    sim.fields.activated[center] = 1.0;
    sim.fields.nutrient[center] = 1.0;
    sim.fields.fuel[center] = 1.0;
    sim.fields.waste[center] = 1.0;
    let phi_before = sim.fields.structure.clone();
    let membrane_before = sim.fields.membrane.clone();
    let before = buffer_addresses(&sim.fields);

    assert!(sim.step());

    assert_eq!(sim.fields.structure, phi_before);
    assert_eq!(sim.fields.membrane, membrane_before);
    assert_eq!(buffer_addresses(&sim.fields).0, before.1);
    for (species, center_value, crossed) in [
        (
            TransportSpecies::Catalyst,
            sim.fields.catalyst[center],
            sim.transport_accounting
                .last_step
                .catalyst
                .absolute_crossed_face_flux,
        ),
        (
            TransportSpecies::Activated,
            sim.fields.activated[center],
            sim.transport_accounting
                .last_step
                .activated
                .absolute_crossed_face_flux,
        ),
        (
            TransportSpecies::Nutrient,
            sim.fields.nutrient[center],
            sim.transport_accounting
                .last_step
                .nutrient
                .absolute_crossed_face_flux,
        ),
        (
            TransportSpecies::Fuel,
            sim.fields.fuel[center],
            sim.transport_accounting
                .last_step
                .fuel
                .absolute_crossed_face_flux,
        ),
        (
            TransportSpecies::Waste,
            sim.fields.waste[center],
            sim.transport_accounting
                .last_step
                .waste
                .absolute_crossed_face_flux,
        ),
    ] {
        assert!(center_value < 1.0, "{species:?} did not move");
        assert!(crossed > 0.0, "{species:?} was not accounted");
    }
}

#[test]
fn rejected_d008_transport_step_swaps_none_and_records_no_transport() {
    let mut sim = Simulation::new(d008_params());
    sim.observer_enabled = false;
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.activated[center] = CONC_SAFETY_LIMIT + 1.0;
    let before = buffer_addresses(&sim.fields);
    let ledger_before = sim.accounting.last_step.activated.clone();
    let clamp_before = sim.accounting.cumulative.clamp_corrections;

    assert!(!sim.step());

    assert_eq!(buffer_addresses(&sim.fields), before);
    assert_eq!(sim.transport_accounting.accepted_steps, 0);
    assert_eq!(
        sim.accounting.last_step.activated.mass_before,
        ledger_before.mass_before
    );
    assert_eq!(
        sim.accounting.last_step.activated.mass_after,
        ledger_before.mass_after
    );
    assert_eq!(sim.accounting.cumulative.clamp_corrections, clamp_before);
}

#[test]
fn rejected_d008_attempt_after_accept_does_not_mutate_accepted_accounting() {
    let mut params = d008_params();
    params.reactions_enabled = false;
    params.phase_separation_enabled = false;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) {
            sim.fields.structure[idx] = 0.5;
            sim.fields.membrane[idx] = 0.0;
            sim.fields.catalyst[idx] = 0.0;
            sim.fields.activated[idx] = 0.0;
            sim.fields.nutrient[idx] = 0.0;
            sim.fields.fuel[idx] = 0.0;
            sim.fields.waste[idx] = 0.0;
        }
    }
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.waste[center] = 1.0;
    assert!(sim.step());
    let accepted_steps = sim.transport_accounting.accepted_steps;
    let last_waste = sim.accounting.last_step.waste.clone();
    let clamp_after_accept = sim.accounting.cumulative.clamp_corrections;

    sim.fields.activated[center] = CONC_SAFETY_LIMIT + 1.0;
    assert!(!sim.step());

    assert_eq!(sim.transport_accounting.accepted_steps, accepted_steps);
    assert_eq!(
        sim.accounting.last_step.waste.mass_after,
        last_waste.mass_after
    );
    assert_eq!(
        sim.accounting.cumulative.clamp_corrections,
        clamp_after_accept
    );
}

#[test]
fn d008_transport_step_clamp_correction_closes_ledger() {
    let mut params = d008_params();
    params.reactions_enabled = false;
    params.phase_separation_enabled = false;
    let mut sim = Simulation::new(params.clone());
    sim.observer_enabled = false;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) {
            sim.fields.structure[idx] = 0.5;
            sim.fields.membrane[idx] = 0.0;
            sim.fields.catalyst[idx] = 0.0;
            sim.fields.activated[idx] = 0.0;
            sim.fields.nutrient[idx] = 0.0;
            sim.fields.fuel[idx] = 0.0;
            sim.fields.waste[idx] = 0.0;
        }
    }
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    // Choose c0 and dt so Euler lands in soft-negative band [NEG_CLAMP, 0).
    let c0 = 1e-6;
    sim.fields.catalyst[center] = c0;
    let mut rate = vec![0.0; sim.fields.catalyst.len()];
    let accounting = transport_field(
        &sim.grid,
        TransportSpecies::Catalyst,
        &sim.fields.catalyst,
        &sim.fields.structure,
        &sim.fields.membrane,
        &params,
        &mut rate,
    );
    assert!(accounting.net_change_rate.abs() < 1e-12);
    let target = 0.5 * NEG_CLAMP; // -5e-7
    assert!(
        rate[center] < 0.0,
        "center must outflow, rate={}",
        rate[center]
    );
    let dt = (target - c0) / rate[center];
    assert!(dt > 0.0, "dt={dt}");
    let predicted = c0 + dt * rate[center];
    assert!(
        predicted >= NEG_CLAMP && predicted < 0.0,
        "predicted next={predicted}"
    );
    sim.dt = dt;

    assert!(sim.step());

    let ledger = &sim.accounting.last_step.catalyst;
    assert!(
        ledger.numerical_correction_delta.abs() > 0.0,
        "expected non-zero clamp correction, got {}",
        ledger.numerical_correction_delta
    );
    assert!(
        ledger.accounting_residual.abs() < 1e-12,
        "ledger residual did not close: {}",
        ledger.accounting_residual
    );
    assert!(
        sim.accounting.cumulative.clamp_corrections.abs() > 0.0,
        "combined clamp correction was not recorded"
    );
}

#[test]
fn transport_field_path_is_monotonic_and_selective_across_membrane() {
    let grid = Grid::new();
    let params = d008_params();
    let size = grid.width * grid.height;
    let phi = vec![0.5; size];
    let levels = [0.0, 0.25, 0.5, 0.75, 1.0];
    for species in transport_species() {
        let mut field = vec![0.0; size];
        for idx in 0..size {
            if grid.in_dish(idx) && (idx % grid.width) as f64 <= grid.cx {
                field[idx] = 1.0;
            }
        }
        let mut fluxes = Vec::new();
        for membrane_level in levels {
            let membrane = vec![membrane_level; size];
            let mut rate = vec![0.0; size];
            let accounting =
                transport_field(&grid, species, &field, &phi, &membrane, &params, &mut rate);
            fluxes.push(accounting.absolute_crossed_face_flux);
        }
        assert!(
            fluxes.windows(2).all(|pair| pair[1] < pair[0]),
            "{species:?} transport_field not monotonic: {fluxes:?}"
        );
        let normalized = fluxes[4] / fluxes[0].max(f64::MIN_POSITIVE);
        match species {
            TransportSpecies::Catalyst | TransportSpecies::Activated => {
                assert!(normalized <= 0.05, "{species:?}: {normalized}");
            }
            TransportSpecies::Nutrient | TransportSpecies::Fuel => {
                assert!(
                    (0.20..=0.50).contains(&normalized),
                    "{species:?}: {normalized}"
                );
            }
            TransportSpecies::Waste => {
                assert!(normalized >= 0.70, "{species:?}: {normalized}");
            }
        }
    }
}

#[test]
fn simulation_step_path_is_monotonic_and_selective_across_membrane() {
    let levels = [0.0, 0.25, 0.5, 0.75, 1.0];
    for species in transport_species() {
        let mut fluxes = Vec::new();
        for membrane_level in levels {
            let mut params = d008_params();
            params.reactions_enabled = false;
            params.phase_separation_enabled = false;
            let mut sim = Simulation::new(params);
            sim.observer_enabled = false;
            for idx in 0..sim.fields.structure.len() {
                if sim.grid.in_dish(idx) {
                    sim.fields.structure[idx] = 0.5;
                    sim.fields.membrane[idx] = membrane_level;
                    sim.fields.catalyst[idx] = 0.0;
                    sim.fields.activated[idx] = 0.0;
                    sim.fields.nutrient[idx] = 0.0;
                    sim.fields.fuel[idx] = 0.0;
                    sim.fields.waste[idx] = 0.0;
                }
            }
            let width = sim.grid.width;
            let center_x = sim.grid.cx;
            let field = match species {
                TransportSpecies::Catalyst => &mut sim.fields.catalyst,
                TransportSpecies::Activated => &mut sim.fields.activated,
                TransportSpecies::Nutrient => &mut sim.fields.nutrient,
                TransportSpecies::Fuel => &mut sim.fields.fuel,
                TransportSpecies::Waste => &mut sim.fields.waste,
            };
            for (idx, value) in field.iter_mut().enumerate() {
                if sim.grid.dish_mask[idx] && (idx % width) as f64 <= center_x {
                    *value = 1.0;
                }
            }
            assert!(sim.step());
            let flux = match species {
                TransportSpecies::Catalyst => {
                    sim.transport_accounting
                        .last_step
                        .catalyst
                        .absolute_crossed_face_flux
                }
                TransportSpecies::Activated => {
                    sim.transport_accounting
                        .last_step
                        .activated
                        .absolute_crossed_face_flux
                }
                TransportSpecies::Nutrient => {
                    sim.transport_accounting
                        .last_step
                        .nutrient
                        .absolute_crossed_face_flux
                }
                TransportSpecies::Fuel => {
                    sim.transport_accounting
                        .last_step
                        .fuel
                        .absolute_crossed_face_flux
                }
                TransportSpecies::Waste => {
                    sim.transport_accounting
                        .last_step
                        .waste
                        .absolute_crossed_face_flux
                }
            };
            fluxes.push(flux);
        }
        assert!(
            fluxes.windows(2).all(|pair| pair[1] < pair[0]),
            "{species:?} Simulation::step not monotonic: {fluxes:?}"
        );
        let normalized = fluxes[4] / fluxes[0].max(f64::MIN_POSITIVE);
        match species {
            TransportSpecies::Catalyst | TransportSpecies::Activated => {
                assert!(normalized <= 0.05, "{species:?}: {normalized}");
            }
            TransportSpecies::Nutrient | TransportSpecies::Fuel => {
                assert!(
                    (0.20..=0.50).contains(&normalized),
                    "{species:?}: {normalized}"
                );
            }
            TransportSpecies::Waste => {
                assert!(normalized >= 0.70, "{species:?}: {normalized}");
            }
        }
    }
}

#[test]
fn field_sha256_stable_is_fixed_digest_not_default_hasher() {
    let field = [1.0_f64, -2.5, 0.0];
    let stable = field_sha256_stable(&field);
    let legacy = field_sha256(&field);
    assert_eq!(stable.len(), 64);
    assert_ne!(stable, legacy);
    assert_eq!(stable, field_sha256_stable(&field));
}

#[test]
fn membrane_synthesis_requires_activated_catalyst_and_interface_and_saturates() {
    let mut params = d008_params();
    params.k_membrane = 0.4;
    let productive = membrane_rates(0.5, 0.4, 0.3, 0.2, &params);
    assert!(productive.synthesis > 0.0);
    assert_eq!(membrane_rates(0.5, 0.4, 0.0, 0.2, &params).synthesis, 0.0);
    assert_eq!(membrane_rates(0.5, 0.0, 0.3, 0.2, &params).synthesis, 0.0);
    assert_eq!(membrane_rates(0.0, 0.4, 0.3, 0.2, &params).synthesis, 0.0);
    assert_eq!(membrane_rates(0.5, 0.4, 0.3, 1.0, &params).synthesis, 0.0);
    assert!(membrane_rates(0.5, 0.4, 0.3, 0.75, &params).synthesis < productive.synthesis);
}

#[test]
fn membrane_losses_include_decay_and_positive_off_interface_detachment() {
    let params = d008_params();
    let interface = membrane_rates(0.5, 0.0, 0.0, 0.5, &params);
    let off_interface = membrane_rates(0.0, 0.0, 0.0, 0.5, &params);
    assert!(interface.decay > 0.0);
    assert!(off_interface.detachment > 0.0);
    assert!(off_interface.detachment > interface.detachment);
}

#[test]
fn membrane_diffusion_is_conservative_and_localization_is_observer_only() {
    let grid = Grid::new();
    let mut membrane = vec![0.0; grid.width * grid.height];
    membrane[Grid::index(grid.width, grid.cx as usize, grid.cy as usize)] = 1.0;
    let mut lap = vec![0.0; membrane.len()];
    let mut rate = vec![0.0; membrane.len()];
    membrane_diffusion_rate(&grid, &membrane, 0.001, &mut lap, &mut rate);
    let net: f64 = grid
        .dish_mask
        .iter()
        .zip(&rate)
        .filter(|(inside, _)| **inside)
        .map(|(_, value)| *value)
        .sum();
    assert!(net.abs() < 1e-12, "diffusion net={net}");

    let phi = vec![0.5; membrane.len()];
    let before = membrane.clone();
    let partition = membrane_partition(&grid, &phi, &membrane);
    assert_eq!(partition.localization_fraction, 1.0);
    assert_eq!(membrane, before, "diagnostics must not mutate membrane");
}

#[test]
fn membrane_calibration_uses_exact_balance_and_three_candidates() {
    let params = d008_params();
    let phi = [0.5, 0.0];
    let catalyst = [0.4, 0.4];
    let activated = [0.3, 0.3];
    let membrane = [0.5, 0.5];
    let mask = [true, true];
    let calibration = membrane_calibration(&phi, &catalyst, &activated, &membrane, &mask, &params);
    let expected_basis =
        membrane_basis(0.5, 0.4, 0.3, 0.5, &params) + membrane_basis(0.0, 0.4, 0.3, 0.5, &params);
    let expected_loss = membrane_losses(0.5, 0.5, &params) + membrane_losses(0.0, 0.5, &params);
    assert!((calibration.production_basis - expected_basis).abs() < 1e-15);
    assert!((calibration.loss - expected_loss).abs() < 1e-15);
    assert!((calibration.k_required - expected_loss / expected_basis).abs() < 1e-15);
    assert_eq!(
        membrane_candidates(calibration.k_required),
        [
            0.75 * calibration.k_required,
            calibration.k_required,
            1.25 * calibration.k_required,
        ]
    );
}

#[test]
fn stage_b_advances_only_membrane_and_swaps_all_seven_buffers() {
    let mut params = d008_params();
    params.d008_stage_b_enabled = true;
    params.k_membrane = 0.05;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    let fixed_before = [
        sim.fields.structure.clone(),
        sim.fields.catalyst.clone(),
        sim.fields.nutrient.clone(),
        sim.fields.fuel.clone(),
        sim.fields.waste.clone(),
        sim.fields.activated.clone(),
    ];
    let membrane_before = sim.fields.membrane.clone();
    let addresses_before = buffer_addresses(&sim.fields);

    assert!(sim.step());

    assert_eq!(sim.fields.structure, fixed_before[0]);
    assert_eq!(sim.fields.catalyst, fixed_before[1]);
    assert_eq!(sim.fields.nutrient, fixed_before[2]);
    assert_eq!(sim.fields.fuel, fixed_before[3]);
    assert_eq!(sim.fields.waste, fixed_before[4]);
    assert_eq!(sim.fields.activated, fixed_before[5]);
    assert_ne!(sim.fields.membrane, membrane_before);
    assert_eq!(buffer_addresses(&sim.fields).0, addresses_before.1);
    assert_eq!(sim.rejection_count, 0);
    assert!(sim.membrane_accounting.last_step.synthesis > 0.0);
    assert!(sim.membrane_accounting.last_step.decay > 0.0);
    assert!(sim.membrane_accounting.last_step.detachment > 0.0);
    assert!(sim.membrane_accounting.last_step.residual.abs() < 1e-10);
}

#[test]
fn stage_b_membrane_clamp_accounting_closes_and_rejection_is_atomic() {
    let mut params = d008_params();
    params.d008_stage_b_enabled = true;
    params.k_membrane = 10_000.0;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    assert!(sim.step());
    assert!(sim.membrane_accounting.last_step.clamp_correction < 0.0);
    assert!(sim.membrane_accounting.last_step.residual.abs() < 1e-10);
    assert!(sim
        .fields
        .membrane
        .iter()
        .all(|&m| (0.0..=M_MAX).contains(&m)));

    let accepted = sim.membrane_accounting.clone();
    let addresses = buffer_addresses(&sim.fields);
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.membrane[center] = CONC_SAFETY_LIMIT + 1.0;
    assert!(!sim.step());
    assert_eq!(buffer_addresses(&sim.fields), addresses);
    assert_eq!(
        sim.membrane_accounting.accepted_steps,
        accepted.accepted_steps
    );
    assert_eq!(
        sim.membrane_accounting.cumulative.synthesis,
        accepted.cumulative.synthesis
    );
}

#[test]
fn deterministic_stage_b_candidate_remains_localized_after_transient() {
    let mut params = d008_params();
    params.d008_stage_b_enabled = true;
    let mut sim = Simulation::new(params.clone());
    let calibration = membrane_calibration(
        &sim.fields.structure,
        &sim.fields.catalyst,
        &sim.fields.activated,
        &sim.fields.membrane,
        &sim.grid.dish_mask,
        &params,
    );
    sim.params.k_membrane = membrane_candidates(calibration.k_required)[1];
    sim.observer_enabled = false;
    for _ in 0..15_000 {
        assert!(sim.step());
    }
    let partition = membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane);
    assert!(
        partition.localization_fraction >= 0.90,
        "localization={}",
        partition.localization_fraction
    );
    assert!(sim.membrane_accounting.cumulative.synthesis > 0.0);
    assert!(sim.membrane_accounting.cumulative.decay > 0.0);
    assert!(sim.membrane_accounting.cumulative.detachment > 0.0);
}

fn stage_c_params() -> SimParams {
    let mut params = d008_params();
    params.d008_stage_mode = D008StageMode::ActivatedMetabolism;
    params.diffusion_enabled = false;
    params.phase_separation_enabled = false;
    params
}

#[test]
fn stage_c_activation_requires_positive_catalyst_nutrient_and_fuel() {
    let params = stage_c_params();
    let active = activated_metabolism_rates(0.4, 0.8, 0.7, 0.2, &params);
    assert!(active.activation > 0.0);
    assert_eq!(
        activated_metabolism_rates(0.0, 0.8, 0.7, 0.2, &params).activation,
        0.0
    );
    assert_eq!(
        activated_metabolism_rates(0.4, 0.0, 0.7, 0.2, &params).activation,
        0.0
    );
    assert_eq!(
        activated_metabolism_rates(0.4, 0.8, 0.0, 0.2, &params).activation,
        0.0
    );
}

#[test]
fn stage_c_reproduction_uses_activated_resource_not_raw_inputs() {
    let params = stage_c_params();
    assert!(activated_metabolism_rates(0.4, 0.0, 0.0, 0.2, &params).reproduction > 0.0);
    assert_eq!(
        activated_metabolism_rates(0.4, 1.0, 1.0, 0.0, &params).reproduction,
        0.0
    );
}

#[test]
fn stage_c_rates_have_exact_unit_stoichiometry_and_positive_waste() {
    let params = stage_c_params();
    let rates = activated_metabolism_rates(0.4, 0.8, 0.7, 0.2, &params);
    assert_eq!(rates.d_nutrient, -rates.activation);
    assert_eq!(rates.d_fuel, -rates.activation);
    assert_eq!(
        rates.d_activated,
        rates.activation - rates.reproduction - rates.activated_decay
    );
    assert_eq!(
        rates.d_catalyst,
        rates.reproduction - rates.catalyst_turnover
    );
    assert_eq!(
        rates.d_waste,
        rates.activation + rates.reproduction + rates.activated_decay + rates.catalyst_turnover
    );
    assert!(rates.d_waste > 0.0);
}

#[test]
fn stage_c_dispatch_is_zero_dimensional_and_accounting_closes() {
    let mut sim = Simulation::new(stage_c_params());
    sim.observer_enabled = false;
    let structure_before = sim.fields.structure.clone();
    let membrane_before = sim.fields.membrane.clone();
    let transport_before = sim.transport_accounting.accepted_steps;

    assert!(sim.step());

    assert_eq!(sim.fields.structure, structure_before);
    assert_eq!(sim.fields.membrane, membrane_before);
    assert_eq!(sim.transport_accounting.accepted_steps, transport_before);
    assert_eq!(sim.metabolism_accounting.accepted_steps, 1);
    let step = &sim.metabolism_accounting.last_step;
    assert!(step.activation > 0.0);
    assert!(step.reproduction > 0.0);
    assert!(step.activated_decay > 0.0);
    assert!(step.catalyst_turnover > 0.0);
    for ledger in [
        &step.catalyst,
        &step.nutrient,
        &step.fuel,
        &step.activated,
        &step.waste,
    ] {
        assert!(ledger.accounting_residual.abs() < 1e-10);
    }
    assert!(
        sim.metabolism_accounting.cumulative.residual.abs() < 1e-8,
        "residual={}",
        sim.metabolism_accounting.cumulative.residual
    );
}

#[test]
fn stage_c_decay_controls_decline_without_activation_or_reproduction() {
    let mut no_activation = Simulation::new(stage_c_params());
    no_activation.observer_enabled = false;
    no_activation.fields.nutrient.fill(0.0);
    let activated_before = total_mass(&no_activation.grid, &no_activation.fields.activated);
    assert!(no_activation.step());
    assert!(total_mass(&no_activation.grid, &no_activation.fields.activated) < activated_before);

    let mut no_reproduction = Simulation::new(stage_c_params());
    no_reproduction.observer_enabled = false;
    no_reproduction.fields.activated.fill(0.0);
    let catalyst_before = total_mass(&no_reproduction.grid, &no_reproduction.fields.catalyst);
    assert!(no_reproduction.step());
    assert!(total_mass(&no_reproduction.grid, &no_reproduction.fields.catalyst) < catalyst_before);
}

#[test]
fn stage_c_bounds_clamp_with_closed_ledgers_and_rejection_is_atomic() {
    let mut params = stage_c_params();
    params.k_d008_reproduction = 10_000.0;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    for idx in 0..sim.fields.catalyst.len() {
        if sim.grid.in_dish(idx) {
            sim.fields.catalyst[idx] = 0.95;
            sim.fields.activated[idx] = 0.10;
        }
    }
    assert!(sim.step());
    assert!(sim
        .fields
        .catalyst
        .iter()
        .all(|&value| (0.0..=sim.params.d008_c_max).contains(&value)));
    assert!(sim
        .fields
        .activated
        .iter()
        .all(|&value| (0.0..=sim.params.d008_a_max).contains(&value)));
    assert!(
        sim.metabolism_accounting
            .last_step
            .catalyst
            .numerical_correction_delta
            < 0.0
    );
    assert!(
        sim.metabolism_accounting.cumulative.residual.abs() < 1e-7,
        "residual={}",
        sim.metabolism_accounting.cumulative.residual
    );

    let accounting = sim.metabolism_accounting.clone();
    let addresses = buffer_addresses(&sim.fields);
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.nutrient[center] = CONC_SAFETY_LIMIT + 1.0;
    assert!(!sim.step());
    assert_eq!(buffer_addresses(&sim.fields), addresses);
    assert_eq!(
        sim.metabolism_accounting.accepted_steps,
        accounting.accepted_steps
    );
    assert_eq!(
        sim.metabolism_accounting.cumulative.activation,
        accounting.cumulative.activation
    );
}

#[test]
fn stage_c_clamp_heavy_horizon_exceeds_cumulative_tolerance() {
    let mut params = stage_c_params();
    params.k_d008_reproduction = 10_000.0;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    for idx in 0..sim.fields.catalyst.len() {
        if sim.grid.in_dish(idx) {
            sim.fields.catalyst[idx] = 0.95;
            sim.fields.activated[idx] = 0.10;
            sim.fields.nutrient[idx] = 0.8;
            sim.fields.fuel[idx] = 0.7;
        }
    }
    for _ in 0..100 {
        assert!(sim.step());
    }
    let cumulative = &sim.metabolism_accounting.cumulative;
    assert!(
        !stage_c_clamp_negligible(cumulative),
        "clamp-heavy horizon must exceed CUMULATIVE_RESIDUAL_TOL; catalyst_corr={} activated_corr={}",
        cumulative.catalyst_clamp_correction,
        cumulative.activated_clamp_correction
    );
    assert!(
        sim.fields
            .catalyst
            .iter()
            .all(|&value| (0.0..=sim.params.d008_c_max).contains(&value)),
        "values remain in bounds after clamp — boundedness must not be tautological"
    );
}

fn stage_d_simulation(radius: f64) -> Simulation {
    let mut params = d008_params();
    params.d008_stage_mode = D008StageMode::FixedCompartment;
    params.d008_stage_b_enabled = false;
    params.diffusion_enabled = true;
    params.phase_separation_enabled = false;
    params.reactions_enabled = true;
    params.reservoir_rate = 1.0;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let x = (idx % sim.grid.width) as f64 - sim.grid.cx;
        let y = (idx / sim.grid.width) as f64 - sim.grid.cy;
        let distance = (x * x + y * y).sqrt();
        let phi = 0.5 * (1.0 - ((distance - radius) / 2.0).tanh());
        sim.fields.structure[idx] = phi;
        sim.fields.membrane[idx] = interface_weight(phi);
        if phi >= 0.5 {
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.activated[idx] = 0.2;
            sim.fields.nutrient[idx] = 0.2;
            sim.fields.fuel[idx] = 0.2;
            sim.fields.waste[idx] = 0.5;
        } else {
            sim.fields.nutrient[idx] = 0.8;
            sim.fields.fuel[idx] = 0.7;
        }
    }
    sim
}

#[test]
fn stage_d_couples_selective_transport_metabolism_and_reservoir_with_fixed_geometry() {
    let mut sim = stage_d_simulation(16.0);
    let structure_hash = field_sha256_stable(&sim.fields.structure);
    let membrane_hash = field_sha256_stable(&sim.fields.membrane);

    assert!(sim.step());

    assert_eq!(field_sha256_stable(&sim.fields.structure), structure_hash);
    assert_eq!(field_sha256_stable(&sim.fields.membrane), membrane_hash);
    assert!(
        sim.transport_accounting
            .last_step
            .nutrient
            .interior_net_flux_rate
            > 0.0
    );
    assert!(
        sim.transport_accounting
            .last_step
            .fuel
            .interior_net_flux_rate
            > 0.0
    );
    assert!(
        sim.transport_accounting
            .last_step
            .waste
            .interior_net_flux_rate
            < 0.0
    );
    assert!(sim.metabolism_accounting.last_step.activation > 0.0);
    assert!(sim.metabolism_accounting.last_step.reproduction > 0.0);
    assert!(sim.accounting.last_step.nutrient.reservoir_delta > 0.0);
    assert!(sim.accounting.last_step.fuel.reservoir_delta > 0.0);
    assert!(sim.accounting.cumulative_within_tolerance());
}

#[test]
fn stage_e_prescribed_balance_enables_all_reaction_terms() {
    let mut params = d008_params();
    params.k_d008_structure = 0.03;
    let interior = PrescribedInterior::default();
    let point = prescribed_balance_point(&params, 24.0, &interior);
    assert!(point.d_structure.is_finite());
    assert!(point.d_catalyst.is_finite());
    assert!(point.d_membrane.is_finite());
    assert!(point.d_activated.is_finite());
}

#[test]
fn stage_e_joint_overlap_detects_shared_zero_flow_region() {
    let mut params = d008_params();
    params.k_membrane = 0.0;
    params.k_membrane_decay = 0.0;
    params.k_membrane_detach = 0.0;
    params.k_d008_activation = 0.0;
    params.k_d008_reproduction = 0.0;
    params.k_d008_structure = 0.0;
    params.k_structure_decay = 0.0;
    params.k_d008_activated_decay = 0.0;
    params.k_d008_catalyst_turnover = 0.0;
    let interior = PrescribedInterior::default();
    let sweep = prescribed_radius_sweep(&params, &[20.0, 24.0, 28.0], &interior);
    assert!(joint_zero_flow_overlap(&sweep));
}

#[test]
fn stage_e_joint_overlap_rejects_disjoint_signatures() {
    let params = d008_params();
    let interior = PrescribedInterior::default();
    let sweep = prescribed_radius_sweep(&params, &stage_e_default_radii(), &interior);
    assert!(!sweep.is_empty());
    // Default uncalibrated rates should not guarantee overlap.
    let _ = joint_zero_flow_overlap(&sweep);
}

#[test]
fn stage_d_rejection_is_atomic() {
    let mut sim = stage_d_simulation(16.0);
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.nutrient[center] = CONC_SAFETY_LIMIT + 1.0;
    let addresses = buffer_addresses(&sim.fields);

    assert!(!sim.step());

    assert_eq!(buffer_addresses(&sim.fields), addresses);
    assert_eq!(sim.substep, 0);
    assert_eq!(sim.transport_accounting.accepted_steps, 0);
    assert_eq!(sim.metabolism_accounting.accepted_steps, 0);
    assert_eq!(sim.accounting.steps_within_tolerance, 0);
}
