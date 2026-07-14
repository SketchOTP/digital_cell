//! D-008 deterministic staged runners.

use chemistry_core::{
    build_candidate_identity, field_sha256, CandidateIdentity, EquationVersion, FieldBuffers,
    FieldSchemaVersion, SimParams, Simulation, SpeciesTransportAccounting, TransportSpecies,
    SNAPSHOT_SCHEMA_VERSION,
};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command;

const MEMBRANE_LEVELS: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

#[derive(Debug, Deserialize)]
struct StageAReference {
    equation_version: EquationVersion,
    d_a: f64,
    beta_c: f64,
    beta_a: f64,
    beta_n: f64,
    beta_f: f64,
    beta_w: f64,
}

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn binary_hash() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| fs::read(path).ok())
        .map(|bytes| chemistry_core::sha256_hex(&bytes))
        .unwrap_or_else(|| "unknown".into())
}

fn reference_params() -> Result<SimParams, Box<dyn std::error::Error>> {
    let reference: StageAReference =
        toml::from_str(&fs::read_to_string("configs/d008/reference.toml")?)?;
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

fn seven_field_hashes(fields: &FieldBuffers) -> serde_json::Value {
    json!({
        "structure": field_sha256(&fields.structure),
        "catalyst": field_sha256(&fields.catalyst),
        "nutrient": field_sha256(&fields.nutrient),
        "fuel": field_sha256(&fields.fuel),
        "waste": field_sha256(&fields.waste),
        "activated": field_sha256(&fields.activated),
        "membrane": field_sha256(&fields.membrane),
    })
}

/// Pure Stage A artifact document builder (provenance keys must match Stage 0).
fn build_stage_a_result(
    identity: &CandidateIdentity,
    source_commit: &str,
    binary_sha256: &str,
    seed_recipe: &str,
    field_hashes: serde_json::Value,
    accepted_substeps: u64,
    simulated_time: f64,
    transport_accounting: Vec<serde_json::Value>,
    clean_termination: bool,
    stage_classification: &str,
) -> serde_json::Value {
    json!({
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
        "transport_accounting": transport_accounting,
        "clean_termination": clean_termination,
        "stage_classification": stage_classification,
    })
}

pub fn run_stage_a(output: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if output.exists() {
        return Err(format!(
            "refusing to overwrite Stage A attempt: {}",
            output.display()
        )
        .into());
    }
    fs::create_dir_all(output)?;

    let params = reference_params()?;
    let source_commit = git_commit_hash();
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
    let mut accepted_substeps = 0_u64;
    let mut simulated_time = 0.0;
    let mut clean_termination = true;
    let mut stage_pass = true;
    let mut field_hashes = json!({});

    for species in species() {
        let mut baseline_flux = 0.0;
        let mut previous_flux = f64::INFINITY;
        for membrane in MEMBRANE_LEVELS {
            let mut sim = Simulation::new(params.clone());
            prepare_planar(&mut sim, species, membrane);
            let initial_field_hash = field_sha256(species_field(&sim.fields, species));
            let accepted = sim.step();
            clean_termination &= accepted;
            accepted_substeps += sim.substep;
            simulated_time += sim.sim_time;
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
            field_hashes = seven_field_hashes(&sim.fields);
            points.push(json!({
                "species": species,
                "membrane": membrane,
                "effective_flux": effective_flux,
                "crossed_mass": effective_flux * sim.sim_time,
                "normalized_to_zero_membrane": normalized_flux,
                "net_change_rate": transport.net_change_rate,
                "initial_field_hash": initial_field_hash,
                "final_field_hash": field_sha256(species_field(&sim.fields, species)),
                "accepted_substeps": sim.substep,
                "simulated_time": sim.sim_time,
            }));
        }
    }

    stage_pass &= clean_termination;
    let result = build_stage_a_result(
        &identity,
        &source_commit,
        &binary_hash(),
        "planar_phi_0.5_fixed_membrane_left_unit_right_zero_each_species_independent",
        field_hashes,
        accepted_substeps,
        simulated_time,
        points,
        clean_termination,
        if stage_pass {
            "D008_STAGE_A_TRANSPORT_PASS"
        } else {
            "D008_STAGE_A_TRANSPORT_FAIL"
        },
    );
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
        "seed_recipe",
        "field_hashes",
        "transport_accounting",
        "clean_termination",
        "stage_classification",
    ];

    fn missing_stage_a_provenance_keys(result: &serde_json::Value) -> Vec<&'static str> {
        STAGE_A_REQUIRED_TOP_LEVEL_KEYS
            .iter()
            .copied()
            .filter(|key| result.get(key).is_none())
            .collect()
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
        let result = build_stage_a_result(
            &identity,
            "deadbeef",
            "abc123",
            "planar_unit_test",
            json!({"structure": "0"}),
            25,
            0.0625,
            vec![json!({"species": "catalyst", "membrane": 0.0})],
            true,
            "D008_STAGE_A_TRANSPORT_PASS",
        );

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
        assert!(result.get("binary_hash").is_none());
        assert!(result.get("candidate_identity").is_none());
        assert!(result.get("field_schema").is_none());
    }
}
