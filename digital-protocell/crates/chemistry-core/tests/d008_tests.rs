//! D-008 Stage 0 schema and Stage A static membrane-transport tests.

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

#[test]
fn d008_hash_identifies_equation_and_field_schema_in_fixed_order() {
    let bytes = String::from_utf8(canonical_params_bytes(&d008_params())).unwrap();
    assert!(bytes.ends_with(
        "equation_version=membrane_metabolism_v1;k_structure_interface=0;\
k_c_structure=0.1;d_a=0.04;beta_c=4.6;beta_a=4.6;beta_n=1.2;beta_f=1.2;\
beta_w=0.2;field_schema_version=seven_field_v1;snapshot_schema_version=2"
    ));
    assert_eq!(
        candidate_hash(&d008_params(), &GridConfiguration::default()),
        candidate_hash(&d008_params(), &GridConfiguration::default())
    );
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
    assert_eq!(sim.accounting.last_step.activated.mass_before, ledger_before.mass_before);
    assert_eq!(sim.accounting.last_step.activated.mass_after, ledger_before.mass_after);
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
    assert!(rate[center] < 0.0, "center must outflow, rate={}", rate[center]);
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
