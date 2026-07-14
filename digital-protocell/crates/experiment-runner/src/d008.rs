//! D-008 deterministic staged runners.

use chemistry_core::{
    build_candidate_identity, field_sha256, EquationVersion, FieldBuffers, SimParams, Simulation,
    SpeciesTransportAccounting, TransportSpecies,
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
    let result = json!({
        "result_schema": "d008_stage_a_result_v1",
        "equation_version": EquationVersion::MembraneMetabolismV1,
        "field_schema": "seven_field_v1",
        "source_commit": source_commit,
        "binary_hash": binary_hash(),
        "candidate_identity": {
            "candidate_id": identity.candidate_id,
            "candidate_hash": identity.candidate_hash,
            "configuration_hash": identity.configuration_hash,
        },
        "seed_recipe": "planar_phi_0.5_fixed_membrane_left_unit_right_zero_each_species_independent",
        "membrane_levels": MEMBRANE_LEVELS,
        "accepted_substeps": accepted_substeps,
        "simulated_time": simulated_time,
        "transport_accounting": points,
        "clean_termination": clean_termination,
        "stage_classification": if stage_pass {
            "D008_STAGE_A_TRANSPORT_PASS"
        } else {
            "D008_STAGE_A_TRANSPORT_FAIL"
        },
    });
    fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(result)
}
