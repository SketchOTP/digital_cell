//! D-008 deterministic staged runners.

use chemistry_core::{
    build_candidate_identity, field_sha256_stable, interface_weight, membrane_calibration,
    membrane_candidates, membrane_partition, CandidateIdentity, EquationVersion, FieldBuffers,
    FieldSchemaVersion, SimParams, Simulation, SpeciesTransportAccounting, TransportSpecies,
    MEMBRANE_CANDIDATE_FACTORS, SNAPSHOT_SCHEMA_VERSION,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MEMBRANE_LEVELS: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
const STAGE_B_TRANSIENT_STEPS: u64 = 15_000;
const STAGE_B_EVALUATION_STEPS: u64 = 1_000;
const STAGE_B_STEPS: u64 = STAGE_B_TRANSIENT_STEPS + STAGE_B_EVALUATION_STEPS;
const STAGE_B_INITIAL_LEVELS: [f64; 3] = [0.25, 0.50, 0.75];

#[derive(Debug, Deserialize)]
struct StageAReference {
    equation_version: EquationVersion,
    d_a: f64,
    beta_c: f64,
    beta_a: f64,
    beta_n: f64,
    beta_f: f64,
    beta_w: f64,
    m_max: f64,
    d_m: f64,
    k_membrane_decay: f64,
    k_membrane_detach: f64,
    k_c_membrane: f64,
    k_membrane: f64,
}

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|err| format!("git_commit_hash failed: {err}"))?;
    if !output.status.success() {
        return Err(format!("git_commit_hash failed: git exited {}", output.status).into());
    }
    let value = String::from_utf8(output.stdout)
        .map_err(|err| format!("git_commit_hash failed: {err}"))?
        .trim()
        .to_string();
    require_provenance_token("source_commit", &value)?;
    Ok(value)
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    let path = std::env::current_exe().map_err(|err| format!("binary_sha256 failed: {err}"))?;
    let bytes = fs::read(&path).map_err(|err| format!("binary_sha256 failed: {err}"))?;
    let value = chemistry_core::sha256_hex(&bytes);
    require_provenance_token("binary_sha256", &value)?;
    Ok(value)
}

fn require_provenance_token(label: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unknown") {
        return Err(format!("{label} provenance unavailable: {value:?}").into());
    }
    Ok(())
}

fn reference_toml_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/d008/reference.toml")
}

fn reference_params() -> Result<SimParams, Box<dyn std::error::Error>> {
    let reference: StageAReference = toml::from_str(&fs::read_to_string(reference_toml_path())?)?;
    if reference.equation_version != EquationVersion::MembraneMetabolismV1 {
        return Err("D-008 Stage A reference must use membrane_metabolism_v1".into());
    }
    let mut params = SimParams::default();
    params.equation_version = reference.equation_version;
    params.d_a = reference.d_a;
    params.beta_c = reference.beta_c;
    params.beta_a = reference.beta_a;
    params.beta_n = reference.beta_n;
    params.beta_f = reference.beta_f;
    params.beta_w = reference.beta_w;
    params.m_max = reference.m_max;
    params.d_m = reference.d_m;
    params.k_membrane_decay = reference.k_membrane_decay;
    params.k_membrane_detach = reference.k_membrane_detach;
    params.k_c_membrane = reference.k_c_membrane;
    params.k_membrane = reference.k_membrane;
    params.reactions_enabled = false;
    params.phase_separation_enabled = false;
    Ok(params)
}

fn species() -> [TransportSpecies; 5] {
    [
        TransportSpecies::Catalyst,
        TransportSpecies::Activated,
        TransportSpecies::Nutrient,
        TransportSpecies::Fuel,
        TransportSpecies::Waste,
    ]
}

fn species_slug(species: TransportSpecies) -> &'static str {
    match species {
        TransportSpecies::Catalyst => "catalyst",
        TransportSpecies::Activated => "activated",
        TransportSpecies::Nutrient => "nutrient",
        TransportSpecies::Fuel => "fuel",
        TransportSpecies::Waste => "waste",
    }
}

fn case_id(species: TransportSpecies, membrane: f64) -> String {
    format!("{}@{}", species_slug(species), membrane)
}

fn species_field(fields: &FieldBuffers, species: TransportSpecies) -> &[f64] {
    match species {
        TransportSpecies::Catalyst => &fields.catalyst,
        TransportSpecies::Activated => &fields.activated,
        TransportSpecies::Nutrient => &fields.nutrient,
        TransportSpecies::Fuel => &fields.fuel,
        TransportSpecies::Waste => &fields.waste,
    }
}

fn species_field_mut(fields: &mut FieldBuffers, species: TransportSpecies) -> &mut [f64] {
    match species {
        TransportSpecies::Catalyst => &mut fields.catalyst,
        TransportSpecies::Activated => &mut fields.activated,
        TransportSpecies::Nutrient => &mut fields.nutrient,
        TransportSpecies::Fuel => &mut fields.fuel,
        TransportSpecies::Waste => &mut fields.waste,
    }
}

fn species_accounting(sim: &Simulation, species: TransportSpecies) -> SpeciesTransportAccounting {
    match species {
        TransportSpecies::Catalyst => sim.transport_accounting.last_step.catalyst,
        TransportSpecies::Activated => sim.transport_accounting.last_step.activated,
        TransportSpecies::Nutrient => sim.transport_accounting.last_step.nutrient,
        TransportSpecies::Fuel => sim.transport_accounting.last_step.fuel,
        TransportSpecies::Waste => sim.transport_accounting.last_step.waste,
    }
}

fn prepare_planar(sim: &mut Simulation, species: TransportSpecies, membrane: f64) {
    sim.observer_enabled = false;
    sim.fields.catalyst.fill(0.0);
    sim.fields.activated.fill(0.0);
    sim.fields.nutrient.fill(0.0);
    sim.fields.fuel.fill(0.0);
    sim.fields.waste.fill(0.0);
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) {
            sim.fields.structure[idx] = 0.5;
            sim.fields.membrane[idx] = membrane;
        }
    }
    let width = sim.grid.width;
    let center_x = sim.grid.cx;
    let dish_mask = sim.grid.dish_mask.clone();
    let field = species_field_mut(&mut sim.fields, species);
    for (idx, value) in field.iter_mut().enumerate() {
        if dish_mask[idx] && (idx % width) as f64 <= center_x {
            *value = 1.0;
        }
    }
}

fn target_met(species: TransportSpecies, normalized: f64) -> bool {
    match species {
        TransportSpecies::Catalyst | TransportSpecies::Activated => normalized <= 0.05,
        TransportSpecies::Nutrient | TransportSpecies::Fuel => (0.20..=0.50).contains(&normalized),
        TransportSpecies::Waste => normalized >= 0.70,
    }
}

fn seven_field_hashes(fields: &FieldBuffers) -> Value {
    json!({
        "structure": field_sha256_stable(&fields.structure),
        "catalyst": field_sha256_stable(&fields.catalyst),
        "nutrient": field_sha256_stable(&fields.nutrient),
        "fuel": field_sha256_stable(&fields.fuel),
        "waste": field_sha256_stable(&fields.waste),
        "activated": field_sha256_stable(&fields.activated),
        "membrane": field_sha256_stable(&fields.membrane),
    })
}

fn validate_stage_a_provenance(
    source_commit: &str,
    binary_sha256: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    require_provenance_token("source_commit", source_commit)?;
    require_provenance_token("binary_sha256", binary_sha256)?;
    Ok(())
}

/// Pure Stage A artifact document builder (provenance keys must match Stage 0).
fn build_stage_a_result(
    identity: &CandidateIdentity,
    source_commit: &str,
    binary_sha256: &str,
    seed_recipe: &str,
    field_hashes: Value,
    accepted_substeps: u64,
    simulated_time: f64,
    aggregate_accepted_substeps: u64,
    aggregate_simulated_time: f64,
    run_count: u64,
    transport_accounting: Vec<Value>,
    clean_termination: bool,
    stage_classification: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    validate_stage_a_provenance(source_commit, binary_sha256)?;
    Ok(json!({
        "snapshot_schema_version": SNAPSHOT_SCHEMA_VERSION,
        "field_schema_version": FieldSchemaVersion::SevenFieldV1,
        "equation_version": EquationVersion::MembraneMetabolismV1,
        "candidate_id": identity.candidate_id,
        "candidate_hash": identity.candidate_hash,
        "configuration_hash": identity.configuration_hash,
        "source_commit": source_commit,
        "binary_sha256": binary_sha256,
        "seed_recipe": seed_recipe,
        "membrane_levels": MEMBRANE_LEVELS,
        "field_hashes": field_hashes,
        "accepted_substeps": accepted_substeps,
        "simulated_time": simulated_time,
        "aggregate_accepted_substeps": aggregate_accepted_substeps,
        "aggregate_simulated_time": aggregate_simulated_time,
        "run_count": run_count,
        "transport_accounting": transport_accounting,
        "clean_termination": clean_termination,
        "stage_classification": stage_classification,
    }))
}

pub fn run_stage_a(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    if output.exists() {
        return Err(format!(
            "refusing to overwrite Stage A attempt: {}",
            output.display()
        )
        .into());
    }
    fs::create_dir_all(output)?;

    let params = reference_params()?;
    let source_commit = git_commit_hash()?;
    let binary_sha256 = binary_hash()?;
    let identity = build_candidate_identity(
        params.clone(),
        &source_commit,
        Some("d008-membrane-metabolic-closure"),
        None,
        "approved D-008 Stage A selective transport constants",
        None,
        None,
    );
    let mut points = Vec::new();
    let mut aggregate_accepted_substeps = 0_u64;
    let mut aggregate_simulated_time = 0.0;
    let mut clean_termination = true;
    let mut stage_pass = true;
    let mut field_hashes = Map::new();
    let mut run_count = 0_u64;

    for species in species() {
        let mut baseline_flux = 0.0;
        let mut previous_flux = f64::INFINITY;
        for membrane in MEMBRANE_LEVELS {
            let mut sim = Simulation::new(params.clone());
            prepare_planar(&mut sim, species, membrane);
            let initial_field_hash = field_sha256_stable(species_field(&sim.fields, species));
            let accepted = sim.step();
            clean_termination &= accepted;
            aggregate_accepted_substeps += sim.substep;
            aggregate_simulated_time += sim.sim_time;
            run_count += 1;
            let transport = species_accounting(&sim, species);
            let effective_flux = transport.absolute_crossed_face_flux;
            if membrane == 0.0 {
                baseline_flux = effective_flux;
            }
            let normalized_flux = effective_flux / baseline_flux.max(f64::MIN_POSITIVE);
            let monotonic = membrane == 0.0 || effective_flux < previous_flux;
            if membrane == 1.0 {
                stage_pass &= target_met(species, normalized_flux);
            }
            stage_pass &= accepted && monotonic && transport.net_change_rate.abs() < 1e-10;
            previous_flux = effective_flux;
            let id = case_id(species, membrane);
            field_hashes.insert(id.clone(), seven_field_hashes(&sim.fields));
            points.push(json!({
                "case_id": id,
                "species": species,
                "membrane": membrane,
                "effective_flux": effective_flux,
                "crossed_mass": effective_flux * sim.sim_time,
                "normalized_to_zero_membrane": normalized_flux,
                "net_change_rate": transport.net_change_rate,
                "initial_field_hash": initial_field_hash,
                "final_field_hash": field_sha256_stable(species_field(&sim.fields, species)),
                "field_hashes": seven_field_hashes(&sim.fields),
                "accepted_substeps": sim.substep,
                "simulated_time": sim.sim_time,
            }));
        }
    }

    stage_pass &= clean_termination;
    let result = build_stage_a_result(
        &identity,
        &source_commit,
        &binary_sha256,
        "planar_phi_0.5_fixed_membrane_left_unit_right_zero_each_species_independent",
        Value::Object(field_hashes),
        aggregate_accepted_substeps,
        aggregate_simulated_time,
        aggregate_accepted_substeps,
        aggregate_simulated_time,
        run_count,
        points,
        clean_termination,
        if stage_pass {
            "D008_STAGE_A_TRANSPORT_PASS"
        } else {
            "D008_STAGE_A_TRANSPORT_FAIL"
        },
    )?;
    fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(result)
}

fn prepare_stage_b(sim: &mut Simulation, initial_level: f64) {
    sim.observer_enabled = false;
    sim.params.d008_stage_b_enabled = true;
    for idx in 0..sim.fields.membrane.len() {
        sim.fields.membrane[idx] = if sim.grid.in_dish(idx) {
            initial_level * interface_weight(sim.fields.structure[idx])
        } else {
            0.0
        };
    }
}

fn run_stage_b_case(mut params: SimParams, initial_level: f64) -> (Simulation, f64) {
    params.d008_stage_b_enabled = true;
    let mut sim = Simulation::new(params);
    prepare_stage_b(&mut sim, initial_level);
    let mut minimum_after_transient = f64::INFINITY;
    for _ in 0..STAGE_B_STEPS {
        if !sim.step() {
            break;
        }
        if sim.substep > STAGE_B_TRANSIENT_STEPS {
            minimum_after_transient = minimum_after_transient.min(
                membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane)
                    .localization_fraction,
            );
        }
    }
    (sim, minimum_after_transient)
}

fn stage_b_case_pass(sim: &Simulation, minimum_after_transient: f64) -> bool {
    let partition = membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane);
    sim.substep == STAGE_B_STEPS
        && sim.rejection_count == 0
        && sim
            .fields
            .membrane
            .iter()
            .all(|&value| value.is_finite() && value >= 0.0 && value <= sim.params.m_max)
        && partition.total_mass > f64::EPSILON
        && minimum_after_transient >= 0.90
        && sim.membrane_accounting.cumulative.synthesis > 0.0
        && sim.membrane_accounting.cumulative.decay > 0.0
        && sim.membrane_accounting.cumulative.detachment > 0.0
        && sim.membrane_accounting.cumulative.residual.abs() < 1e-8
}

fn stage_b_run_count(candidate_runs: usize, initial_state_runs: usize) -> usize {
    candidate_runs + initial_state_runs
}

pub fn run_stage_b(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    if output.exists() {
        return Err(format!(
            "refusing to overwrite Stage B attempt: {}",
            output.display()
        )
        .into());
    }
    fs::create_dir_all(output)?;

    let mut reference = reference_params()?;
    reference.d008_stage_b_enabled = true;
    let calibration_seed = {
        let mut sim = Simulation::new(reference.clone());
        prepare_stage_b(&mut sim, 0.50);
        sim
    };
    let calibration = membrane_calibration(
        &calibration_seed.fields.structure,
        &calibration_seed.fields.catalyst,
        &calibration_seed.fields.activated,
        &calibration_seed.fields.membrane,
        &calibration_seed.grid.dish_mask,
        &reference,
    );
    let candidate_rates = membrane_candidates(calibration.k_required);
    let source_commit = git_commit_hash()?;
    let binary_sha256 = binary_hash()?;
    validate_stage_a_provenance(&source_commit, &binary_sha256)?;

    let mut candidate_runs = Vec::new();
    for (factor, rate) in MEMBRANE_CANDIDATE_FACTORS.into_iter().zip(candidate_rates) {
        let mut params = reference.clone();
        params.k_membrane = rate;
        let identity = build_candidate_identity(
            params.clone(),
            &source_commit,
            Some("d008-membrane-metabolic-closure"),
            None,
            "D-008 Stage B prescribed-balance candidate",
            None,
            None,
        );
        let initial = {
            let mut sim = Simulation::new(params.clone());
            prepare_stage_b(&mut sim, 0.50);
            seven_field_hashes(&sim.fields)
        };
        let (sim, minimum_after_transient) = run_stage_b_case(params, 0.50);
        let partition = membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane);
        let pass = stage_b_case_pass(&sim, minimum_after_transient);
        candidate_runs.push((
            factor,
            identity,
            initial,
            sim,
            partition,
            minimum_after_transient,
            pass,
        ));
    }

    let selected_index = candidate_runs
        .iter()
        .enumerate()
        .filter(|(_, run)| run.6)
        .min_by(|(_, left), (_, right)| {
            (left.0 - 1.0)
                .abs()
                .total_cmp(&(right.0 - 1.0).abs())
                .then_with(|| {
                    right
                        .4
                        .localization_fraction
                        .total_cmp(&left.4.localization_fraction)
                })
        })
        .map(|(index, _)| index);

    let mut candidate_results = Vec::new();
    for (factor, identity, initial, sim, partition, minimum_after_transient, pass) in
        &candidate_runs
    {
        candidate_results.push(json!({
            "factor": factor,
            "k_membrane": identity.params.k_membrane,
            "candidate_id": identity.candidate_id,
            "candidate_hash": identity.candidate_hash,
            "configuration_hash": identity.configuration_hash,
            "initial_membrane_level": 0.50,
            "initial_field_hashes": initial,
            "final_field_hashes": seven_field_hashes(&sim.fields),
            "accepted_substeps": sim.substep,
            "simulated_time": sim.sim_time,
            "rejection_count": sim.rejection_count,
            "membrane_accounting": sim.membrane_accounting,
            "mass_partition": partition,
            "bounded": sim.fields.membrane.iter().all(|&m| m.is_finite() && m >= 0.0 && m <= sim.params.m_max),
            "positive_turnover": sim.membrane_accounting.cumulative.synthesis > 0.0
                && sim.membrane_accounting.cumulative.decay > 0.0
                && sim.membrane_accounting.cumulative.detachment > 0.0,
            "minimum_localization_after_transient": minimum_after_transient,
            "localized": *minimum_after_transient >= 0.90,
            "nonvanishing": partition.total_mass > f64::EPSILON,
            "result": if *pass { "pass" } else { "fail" },
        }));
    }

    let mut aggregate_accepted_substeps: u64 = candidate_runs.iter().map(|run| run.3.substep).sum();
    let mut aggregate_simulated_time: f64 = candidate_runs.iter().map(|run| run.3.sim_time).sum();
    let mut initial_state_results = Vec::new();
    let mut initial_states_pass = true;
    let mut initial_states_clean = true;
    let (
        selected_identity,
        selected_initial,
        selected_sim,
        selected_partition,
        selected_minimum_localization,
    ) = if let Some(index) = selected_index {
        let selected_params = candidate_runs[index].1.params.clone();
        for initial_level in [STAGE_B_INITIAL_LEVELS[0], STAGE_B_INITIAL_LEVELS[2]] {
            let (sim, minimum_after_transient) =
                run_stage_b_case(selected_params.clone(), initial_level);
            aggregate_accepted_substeps += sim.substep;
            aggregate_simulated_time += sim.sim_time;
            initial_states_pass &= stage_b_case_pass(&sim, minimum_after_transient);
            initial_states_clean &= sim.substep == STAGE_B_STEPS && sim.rejection_count == 0;
            initial_state_results.push(json!({
                    "initial_membrane_level": initial_level,
                    "accepted_substeps": sim.substep,
                    "simulated_time": sim.sim_time,
                    "clean_termination": sim.substep == STAGE_B_STEPS && sim.rejection_count == 0,
                    "membrane_accounting": sim.membrane_accounting,
                    "mass_partition": membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane),
                    "minimum_localization_after_transient": minimum_after_transient,
                    "field_hashes": seven_field_hashes(&sim.fields),
                }));
        }
        let run = &candidate_runs[index];
        (&run.1, &run.2, &run.3, run.4, run.5)
    } else {
        let run = &candidate_runs[1];
        (&run.1, &run.2, &run.3, run.4, run.5)
    };

    let stage_pass = selected_index.is_some() && initial_states_pass;
    let result = json!({
        "snapshot_schema_version": SNAPSHOT_SCHEMA_VERSION,
        "field_schema_version": FieldSchemaVersion::SevenFieldV1,
        "equation_version": EquationVersion::MembraneMetabolismV1,
        "candidate_id": selected_identity.candidate_id,
        "candidate_hash": selected_identity.candidate_hash,
        "configuration_hash": selected_identity.configuration_hash,
        "selected_candidate_hash": selected_identity.candidate_hash,
        "selected_configuration_hash": selected_identity.configuration_hash,
        "source_commit": source_commit,
        "binary_sha256": binary_sha256,
        "seed_recipe": "fixed_circular_phi_seed_with_fixed_C_A_and_M_level_times_interface_weight",
        "calibration": calibration,
        "candidate_factors": MEMBRANE_CANDIDATE_FACTORS,
        "candidate_results": candidate_results,
        "initial_state_results": initial_state_results,
        "fixed_field_hashes": {
            "initial": selected_initial,
            "final": seven_field_hashes(&selected_sim.fields),
        },
        "field_hashes": seven_field_hashes(&selected_sim.fields),
        "accepted_substeps": selected_sim.substep,
        "simulated_time": selected_sim.sim_time,
        "aggregate_accepted_substeps": aggregate_accepted_substeps,
        "aggregate_simulated_time": aggregate_simulated_time,
        "run_count": stage_b_run_count(candidate_runs.len(), initial_state_results.len()),
        "membrane_accounting": selected_sim.membrane_accounting,
        "localization": {
            "interface_threshold": 0.25,
            "transient_steps": STAGE_B_TRANSIENT_STEPS,
            "evaluation_steps": STAGE_B_EVALUATION_STEPS,
            "minimum_after_transient": selected_minimum_localization,
        },
        "mass_partitions": selected_partition,
        "clean_termination": candidate_runs.iter().all(|run| run.3.substep == STAGE_B_STEPS && run.3.rejection_count == 0)
            && initial_states_clean,
        "stage_classification": if stage_pass {
            "D008_STAGE_B_LOCALIZATION_PASS"
        } else {
            "D008_STAGE_B_LOCALIZATION_FAIL"
        },
    });
    fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chemistry_core::build_candidate_identity;
    use std::time::{SystemTime, UNIX_EPOCH};

    const FIELD_HASH_KEYS: [&str; 7] = [
        "structure",
        "catalyst",
        "nutrient",
        "fuel",
        "waste",
        "activated",
        "membrane",
    ];

    const STAGE_A_REQUIRED_TOP_LEVEL_KEYS: &[&str] = &[
        "snapshot_schema_version",
        "field_schema_version",
        "equation_version",
        "candidate_id",
        "candidate_hash",
        "configuration_hash",
        "source_commit",
        "binary_sha256",
        "accepted_substeps",
        "simulated_time",
        "aggregate_accepted_substeps",
        "aggregate_simulated_time",
        "run_count",
        "seed_recipe",
        "field_hashes",
        "transport_accounting",
        "clean_termination",
        "stage_classification",
    ];

    const STAGE_B_REQUIRED_TOP_LEVEL_KEYS: &[&str] = &[
        "snapshot_schema_version",
        "field_schema_version",
        "equation_version",
        "candidate_id",
        "candidate_hash",
        "configuration_hash",
        "source_commit",
        "binary_sha256",
        "accepted_substeps",
        "simulated_time",
        "aggregate_accepted_substeps",
        "aggregate_simulated_time",
        "run_count",
        "seed_recipe",
        "fixed_field_hashes",
        "selected_configuration_hash",
        "selected_candidate_hash",
        "calibration",
        "candidate_factors",
        "candidate_results",
        "membrane_accounting",
        "localization",
        "mass_partitions",
        "clean_termination",
        "stage_classification",
    ];

    fn missing_stage_a_provenance_keys(result: &Value) -> Vec<&'static str> {
        STAGE_A_REQUIRED_TOP_LEVEL_KEYS
            .iter()
            .copied()
            .filter(|key| result.get(key).is_none())
            .collect()
    }

    fn expected_case_ids() -> Vec<String> {
        let mut ids = Vec::new();
        for species in species() {
            for membrane in MEMBRANE_LEVELS {
                ids.push(case_id(species, membrane));
            }
        }
        ids
    }

    #[test]
    fn stage_a_result_json_has_exact_stage_zero_provenance_keys() {
        let mut params = SimParams::default();
        params.equation_version = EquationVersion::MembraneMetabolismV1;
        let identity = build_candidate_identity(
            params,
            "deadbeef",
            Some("d008-membrane-metabolic-closure"),
            None,
            "schema unit test",
            None,
            None,
        );
        let mut hashes = Map::new();
        for id in expected_case_ids() {
            hashes.insert(
                id,
                json!({
                    "structure": "0",
                    "catalyst": "0",
                    "nutrient": "0",
                    "fuel": "0",
                    "waste": "0",
                    "activated": "0",
                    "membrane": "0",
                }),
            );
        }
        let result = build_stage_a_result(
            &identity,
            "deadbeef",
            "abc123",
            "planar_unit_test",
            Value::Object(hashes),
            25,
            0.0625,
            25,
            0.0625,
            25,
            vec![json!({"species": "catalyst", "membrane": 0.0})],
            true,
            "D008_STAGE_A_TRANSPORT_PASS",
        )
        .expect("valid provenance");

        let missing = missing_stage_a_provenance_keys(&result);
        assert!(
            missing.is_empty(),
            "Stage A result missing required provenance keys: {missing:?}"
        );
        assert_eq!(result["snapshot_schema_version"], SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(
            result["field_schema_version"],
            json!(FieldSchemaVersion::SevenFieldV1)
        );
        assert_eq!(result["candidate_id"], json!(identity.candidate_id));
        assert_eq!(result["candidate_hash"], json!(identity.candidate_hash));
        assert_eq!(
            result["configuration_hash"],
            json!(identity.configuration_hash)
        );
        assert_eq!(result["binary_sha256"], json!("abc123"));
        assert_eq!(result["run_count"], json!(25));
        assert_eq!(result["aggregate_accepted_substeps"], json!(25));
        assert_eq!(result["accepted_substeps"], json!(25));
        assert!(result.get("binary_hash").is_none());
        assert!(result.get("candidate_identity").is_none());
        assert!(result.get("field_schema").is_none());
    }

    #[test]
    fn stage_a_result_builder_rejects_unknown_or_empty_provenance() {
        let mut params = SimParams::default();
        params.equation_version = EquationVersion::MembraneMetabolismV1;
        let identity = build_candidate_identity(
            params,
            "deadbeef",
            Some("d008-membrane-metabolic-closure"),
            None,
            "schema unit test",
            None,
            None,
        );
        let hashes = json!({"catalyst@0": seven_field_hashes(&FieldBuffers::new(1))});
        for (source, binary) in [
            ("unknown", "abc123"),
            ("", "abc123"),
            ("deadbeef", "unknown"),
            ("deadbeef", ""),
            ("UNKNOWN", "abc123"),
        ] {
            let err = build_stage_a_result(
                &identity,
                source,
                binary,
                "planar_unit_test",
                hashes.clone(),
                1,
                0.0,
                1,
                0.0,
                1,
                vec![],
                true,
                "D008_STAGE_A_TRANSPORT_PASS",
            )
            .expect_err("provenance must fail closed");
            assert!(
                err.to_string().contains("provenance unavailable"),
                "unexpected error: {err}"
            );
        }
    }

    #[test]
    fn run_stage_a_temp_artifact_covers_all_cases_and_refuses_rerun() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let output = std::env::temp_dir().join(format!("d008_stage_a_path_{stamp}"));
        let _ = fs::remove_dir_all(&output);

        let result = run_stage_a(&output).expect("stage a run");
        let on_disk: Value = serde_json::from_slice(
            &fs::read(output.join("result.json")).expect("result.json readable"),
        )
        .expect("result.json parses");
        assert_eq!(
            on_disk["stage_classification"],
            result["stage_classification"]
        );
        assert_eq!(on_disk["run_count"], result["run_count"]);
        assert_eq!(
            on_disk["field_hashes"].as_object().map(|o| o.len()),
            result["field_hashes"].as_object().map(|o| o.len())
        );

        let hashes = on_disk["field_hashes"]
            .as_object()
            .expect("field_hashes object");
        let expected = expected_case_ids();
        assert_eq!(hashes.len(), 25, "expected 25 independent case hashes");
        assert_eq!(on_disk["run_count"], json!(25));
        assert_eq!(
            on_disk["accepted_substeps"],
            on_disk["aggregate_accepted_substeps"]
        );
        assert_eq!(
            on_disk["simulated_time"],
            on_disk["aggregate_simulated_time"]
        );
        for id in &expected {
            let case = hashes
                .get(id)
                .unwrap_or_else(|| panic!("missing case hash {id}"));
            let case_obj = case.as_object().expect("seven-field hash object");
            assert_eq!(case_obj.len(), 7, "{id} missing fields");
            for key in FIELD_HASH_KEYS {
                let digest = case_obj[key].as_str().expect("hash string");
                assert_eq!(digest.len(), 64, "{id}.{key} not sha256");
                assert!(!digest.eq_ignore_ascii_case("unknown"));
            }
        }

        let source = on_disk["source_commit"].as_str().unwrap();
        let binary = on_disk["binary_sha256"].as_str().unwrap();
        assert!(!source.is_empty() && !source.eq_ignore_ascii_case("unknown"));
        assert!(!binary.is_empty() && !binary.eq_ignore_ascii_case("unknown"));
        assert_eq!(
            on_disk["stage_classification"],
            json!("D008_STAGE_A_TRANSPORT_PASS")
        );

        let accounting = on_disk["transport_accounting"]
            .as_array()
            .expect("transport_accounting array");
        assert_eq!(accounting.len(), 25);
        for species in species() {
            let mut baseline = None;
            for membrane in MEMBRANE_LEVELS {
                let id = case_id(species, membrane);
                let point = accounting
                    .iter()
                    .find(|row| row["case_id"] == id)
                    .unwrap_or_else(|| panic!("missing transport row {id}"));
                let flux = point["effective_flux"].as_f64().unwrap();
                if membrane == 0.0 {
                    baseline = Some(flux);
                } else {
                    assert!(flux < baseline.unwrap(), "{id} not attenuated vs M=0");
                }
                if membrane == 1.0 {
                    let normalized = point["normalized_to_zero_membrane"].as_f64().unwrap();
                    assert!(
                        target_met(species, normalized),
                        "{id} selectivity failed: {normalized}"
                    );
                }
            }
        }

        let err = run_stage_a(&output).expect_err("immutable rerun refusal");
        assert!(
            err.to_string().contains("refusing to overwrite"),
            "unexpected rerun error: {err}"
        );

        fs::remove_dir_all(&output).expect("cleanup temp stage-a artifact");
    }

    #[test]
    fn run_stage_b_has_exact_candidates_complete_schema_and_immutable_output() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let output = std::env::temp_dir().join(format!("d008_stage_b_path_{stamp}"));
        let _ = fs::remove_dir_all(&output);

        let result = run_stage_b(&output).expect("stage b run");
        let on_disk: Value = serde_json::from_slice(
            &fs::read(output.join("result.json")).expect("result.json readable"),
        )
        .expect("result.json parses");
        for key in STAGE_B_REQUIRED_TOP_LEVEL_KEYS {
            assert!(on_disk.get(key).is_some(), "missing Stage B key {key}");
        }
        assert_eq!(on_disk["candidate_factors"], json!([0.75, 1.0, 1.25]));
        assert_eq!(
            on_disk["candidate_results"]
                .as_array()
                .expect("candidate results")
                .len(),
            3
        );
        assert_eq!(
            on_disk["stage_classification"],
            json!("D008_STAGE_B_LOCALIZATION_PASS")
        );
        assert_eq!(
            on_disk["selected_candidate_hash"],
            result["selected_candidate_hash"]
        );
        assert!(on_disk["clean_termination"].as_bool().unwrap());
        assert!(on_disk["accepted_substeps"].as_u64().unwrap() > 1);
        assert!(
            on_disk["localization"]["minimum_after_transient"]
                .as_f64()
                .unwrap()
                >= 0.90
        );
        for key in [
            "structure",
            "catalyst",
            "nutrient",
            "fuel",
            "waste",
            "activated",
        ] {
            assert_eq!(
                on_disk["fixed_field_hashes"]["initial"][key],
                on_disk["fixed_field_hashes"]["final"][key],
                "fixed field changed: {key}"
            );
        }
        let accounting = &on_disk["membrane_accounting"]["cumulative"];
        assert!(accounting["synthesis"].as_f64().unwrap() > 0.0);
        assert!(accounting["decay"].as_f64().unwrap() > 0.0);
        assert!(accounting["detachment"].as_f64().unwrap() > 0.0);
        assert!(accounting["residual"].as_f64().unwrap().abs() < 1e-8);

        let err = run_stage_b(&output).expect_err("immutable rerun refusal");
        assert!(err.to_string().contains("refusing to overwrite"));
        fs::remove_dir_all(&output).expect("cleanup temp stage-b artifact");
    }

    #[test]
    fn stage_b_run_count_includes_executed_robustness_failures() {
        assert_eq!(stage_b_run_count(3, 2), 5);
    }
}
