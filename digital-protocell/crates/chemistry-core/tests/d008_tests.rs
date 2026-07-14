//! D-008 Stage 0 schema and engine scaffold tests.

use chemistry_core::*;

const LEGACY_10_STEP_FIELD_HASH: u64 = 9_108_120_965_361_457_156;
const LEGACY_DEFAULT_CANDIDATE_HASH: &str =
    "3a71c61b818c2193407b609c2e1726344677f08e5f4c86f0aaeee1790f2bb2db";
const LEGACY_DEFAULT_CONFIGURATION_HASH: &str =
    "1fba90d376b1bfdf68b1dcae775860ee55d4ee7a86d521d6631e7a8890edae43";
const D006_SURFACE_CANDIDATE_HASH: &str =
    "a65c9c86e5ad93bd9088e7767917656a30641fbc82b3582db4a1bebc9633e808";
const D006_SURFACE_CONFIGURATION_HASH: &str =
    "53c5fd482d171d8a5d20dfbc16e7fdc1f1fc782d06d98c659c1a82fd23a172bb";

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
    fields.structure[0] = 1.0;
    fields.catalyst[0] = 2.0;
    fields.nutrient[0] = 3.0;
    fields.fuel[0] = 4.0;
    fields.waste[0] = 5.0;
    fields.activated[0] = 6.0;
    fields.membrane[0] = 7.0;
    let mut working = FieldBuffers::new(1);

    fields.copy_current_to_working(&mut working);

    assert_eq!(
        [
            working.structure[0],
            working.catalyst[0],
            working.nutrient[0],
            working.fuel[0],
            working.waste[0],
            working.activated[0],
            working.membrane[0],
        ],
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]
    );
}

#[test]
fn accepted_d008_step_swaps_all_seven_buffers() {
    let mut sim = Simulation::new(d008_params());
    sim.observer_enabled = false;
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.activated[center] = 0.25;
    sim.fields.membrane[center] = 0.5;
    let before = buffer_addresses(&sim.fields);

    assert!(sim.step());

    let after = buffer_addresses(&sim.fields);
    assert_eq!(after.0, before.1);
    assert_eq!(after.1, before.0);
    assert_eq!(sim.fields.activated[center], 0.25);
    assert_eq!(sim.fields.membrane[center], 0.5);
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
fn seven_field_snapshot_json_round_trips_activated_and_membrane() {
    let mut sim = Simulation::new(d008_params());
    let center = Grid::index(sim.grid.width, sim.grid.cx as usize, sim.grid.cy as usize);
    sim.fields.activated[center] = 0.125;
    sim.fields.membrane[center] = 0.75;

    let loaded = FieldSnapshot::from_json(&sim.snapshot().to_json().unwrap()).unwrap();

    assert_eq!(loaded.snapshot_schema_version, SNAPSHOT_SCHEMA_VERSION);
    assert_eq!(loaded.field_schema_version, FieldSchemaVersion::SevenFieldV1);
    assert_eq!(loaded.equation_version, EquationVersion::MembraneMetabolismV1);
    assert_eq!(loaded.fields.activated().unwrap()[center], 0.125);
    assert_eq!(loaded.fields.membrane().unwrap()[center], 0.75);
    let mut restored = Simulation::new(d008_params());
    restored.restore_snapshot(&loaded);
    assert_eq!(restored.fields.activated[center], 0.125);
    assert_eq!(restored.fields.membrane[center], 0.75);
}

#[test]
fn historical_five_field_snapshot_remains_readable_for_legacy_equation() {
    let sim = Simulation::new(baseline_params());
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
    assert_eq!(candidate_hash(&baseline, &grid), LEGACY_DEFAULT_CANDIDATE_HASH);
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
k_c_structure=0.1;field_schema_version=seven_field_v1;snapshot_schema_version=2"
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
    assert_eq!(sim.field_hash(), LEGACY_10_STEP_FIELD_HASH);
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
        assert_eq!(serde_json::to_string(&version).unwrap(), format!("\"{name}\""));
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
