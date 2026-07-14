//! D-007 joint structural–catalyst fixed-point search runner.

use chemistry_core::*;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub fn d007_root() -> PathBuf {
    PathBuf::from("experiments/generated/d007")
}

fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn binary_hash() -> String {
    if let Ok(path) = std::env::current_exe() {
        if let Ok(bytes) = fs::read(&path) {
            return sha256_hex(&bytes);
        }
    }
    "unknown".into()
}

fn field_masses(sim: &Simulation) -> serde_json::Value {
    json!({
        "structure": total_mass(&sim.grid, &sim.fields.structure),
        "catalyst": total_mass(&sim.grid, &sim.fields.catalyst),
        "nutrient": total_mass(&sim.grid, &sim.fields.nutrient),
        "fuel": total_mass(&sim.grid, &sim.fields.fuel),
        "waste": total_mass(&sim.grid, &sim.fields.waste),
    })
}

fn field_hashes(sim: &Simulation) -> serde_json::Value {
    json!({
        "structure": field_sha256(&sim.fields.structure),
        "catalyst": field_sha256(&sim.fields.catalyst),
        "nutrient": field_sha256(&sim.fields.nutrient),
        "fuel": field_sha256(&sim.fields.fuel),
        "waste": field_sha256(&sim.fields.waste),
    })
}

fn mean_c_inside(sim: &Simulation) -> f64 {
    let mut retained = 0.0;
    let mut area = 0.0;
    for idx in 0..sim.grid.width * sim.grid.height {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        if interior_weight(sim.fields.structure[idx]) > 0.5 {
            retained += sim.fields.catalyst[idx];
            area += 1.0;
        }
    }
    if area < 1.0 {
        let r = (total_mass(&sim.grid, &sim.fields.structure).max(1e-9) / std::f64::consts::PI)
            .sqrt();
        return total_mass(&sim.grid, &sim.fields.catalyst)
            / (std::f64::consts::PI * r * r).max(1e-12);
    }
    retained / area
}

fn resource_stats(sim: &Simulation) -> serde_json::Value {
    let mut n_in = 0.0f64;
    let mut f_in = 0.0f64;
    let mut w_in = 0.0f64;
    let mut cells = 0.0f64;
    for idx in 0..sim.grid.width * sim.grid.height {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        if interior_weight(sim.fields.structure[idx]) > 0.5 {
            n_in += sim.fields.nutrient[idx];
            f_in += sim.fields.fuel[idx];
            w_in += sim.fields.waste[idx];
            cells += 1.0;
        }
    }
    let den = cells.max(1.0);
    json!({
        "mean_nutrient_inside": n_in / den,
        "mean_fuel_inside": f_in / den,
        "mean_waste_inside": w_in / den,
        "interior_cells": cells,
        "nutrient_exhausted": (n_in / den) < 1e-4,
        "fuel_exhausted": (f_in / den) < 1e-4,
    })
}

/// Build immutable D-007 reference params matching D-006 1.0× survivor.
pub fn reference_d006_params() -> SimParams {
    let mut p = surface_turnover_params_from_calibrated_kphi1();
    p.k_structure_interface = D006_K_STRUCTURE_INTERFACE;
    p.k_rep = D006_K_REP;
    p
}

pub fn write_reference_config() -> Result<CandidateIdentity, Box<dyn std::error::Error>> {
    let root = PathBuf::from("configs/d007");
    fs::create_dir_all(&root)?;
    let commit = git_commit_hash();
    let p = reference_d006_params();
    let id = build_candidate_identity(
        p,
        &commit,
        Some("surface_turnover_v1"),
        None,
        "D-007 reference replay of D-006 1.0× planar-derived rate",
        None,
        None,
    );
    if id.configuration_hash != D006_REFERENCE_CONFIGURATION_HASH {
        eprintln!(
            "WARN: reference configuration_hash {} != expected {}",
            id.configuration_hash, D006_REFERENCE_CONFIGURATION_HASH
        );
    }
    fs::write(
        root.join("reference_d006.toml"),
        serde_json::to_string_pretty(&id.params)?,
    )?;
    fs::write(
        root.join("reference_d006_identity.json"),
        serde_json::to_string_pretty(&id)?,
    )?;
    let art = d007_root().join("reference_replay");
    fs::create_dir_all(&art)?;
    fs::write(
        art.join("reference_identity.json"),
        serde_json::to_string_pretty(&id)?,
    )?;
    Ok(id)
}

pub fn result_has_strict_schema(v: &serde_json::Value) -> bool {
    let keys: Vec<&str> = v
        .as_object()
        .map(|o| o.keys().map(|k| k.as_str()).collect())
        .unwrap_or_default();
    result_schema_is_complete(&keys)
}

/// Strict-schema coupled run used for every D-007 scientific result.
pub fn run_strict(
    id: &CandidateIdentity,
    r0: f64,
    c0: f64,
    seed: u64,
    substeps: u64,
    out_dir: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(out_dir)?;
    let result_path = out_dir.join("result.json");
    if result_path.exists() {
        let existing: serde_json::Value = serde_json::from_str(&fs::read_to_string(&result_path)?)?;
        if result_has_strict_schema(&existing) && existing["clean_termination"] == true {
            return Ok(existing);
        }
        let _ = fs::remove_file(&result_path);
    }

    let recipe = FreshSeedRecipe {
        r0,
        c0,
        noise_seed: seed,
        noise_amplitude: 0.005,
    };
    let cfg = crate::d005::SimRunConfig {
        substeps,
        record_every: 500,
        checkpoint_every: 5_000,
        trajectory_sample_every: 500,
    };
    let t0 = Instant::now();
    let mut sim = spawn_fresh_simulation(id.params.clone(), &recipe);
    sim.observer_enabled = true;

    let initial_hashes = field_hashes(&sim);
    let initial_masses = field_masses(&sim);
    let start_m_phi = total_mass(&sim.grid, &sim.fields.structure);
    let start_r = (start_m_phi.max(1e-9) / std::f64::consts::PI).sqrt();
    let start_c_in = mean_c_inside(&sim);
    let start_sub = sim.substep;

    let (balance, _traj, _wins) = crate::d005::advance_simulation(&mut sim, &cfg);
    let clean = (sim.substep - start_sub) >= substeps;
    let diag = sim.current_diagnostics();
    let end_r = (diag.protocell_area.max(1) as f64 / std::f64::consts::PI).sqrt();
    let end_c_in = mean_c_inside(&sim);
    let dt = sim.sim_time.max(1e-12);
    let conn = diag.largest_component as f64 / diag.protocell_area.max(1) as f64;
    let cum = &sim.accounting.cumulative;
    let res = resource_stats(&sim);

    let reaction_accounting = json!({
        "structural_synthesis": cum.structural_synthesis,
        "structural_decay": cum.structural_decay,
        "catalyst_reproduction": cum.catalyst_reproduction,
        "catalyst_decay": cum.catalyst_decay,
        "nutrient_consumed_r1": cum.nutrient_consumed_r1,
        "nutrient_consumed_r2": cum.nutrient_consumed_r2,
        "fuel_consumed_r1": cum.fuel_consumed_r1,
        "fuel_consumed_r2": cum.fuel_consumed_r2,
        "waste_from_r1": cum.waste_from_r1,
        "waste_from_r2": cum.waste_from_r2,
    });
    let diffusion_accounting = json!({
        "last_step_structure_diffusion": sim.accounting.last_step.structure.diffusion_delta,
        "last_step_catalyst_diffusion": sim.accounting.last_step.catalyst.diffusion_delta,
        "last_step_nutrient_diffusion": sim.accounting.last_step.nutrient.diffusion_delta,
        "last_step_fuel_diffusion": sim.accounting.last_step.fuel.diffusion_delta,
        "last_step_waste_diffusion": sim.accounting.last_step.waste.diffusion_delta,
    });
    let reservoir_accounting = json!({
        "nutrient_supplied": cum.nutrient_supplied_reservoir,
        "fuel_supplied": cum.fuel_supplied_reservoir,
        "waste_removed": cum.waste_removed_reservoir,
    });
    let numerical_corrections = json!({
        "clamp_corrections": cum.clamp_corrections,
        "rejected_steps": cum.rejected_steps,
        "last_step_structure": sim.accounting.last_step.structure.numerical_correction_delta,
        "last_step_catalyst": sim.accounting.last_step.catalyst.numerical_correction_delta,
    });
    let waste_statistics = json!({
        "mean_waste_inside": res["mean_waste_inside"],
        "waste_production_cumulative": cum.waste_from_r1 + cum.waste_from_r2 + cum.waste_from_decay,
    });
    let mut out = serde_json::Map::new();
    out.insert("equation_version".into(), json!(id.equation_version));
    out.insert("candidate_id".into(), json!(id.candidate_id));
    out.insert("candidate_hash".into(), json!(id.candidate_hash));
    out.insert("configuration_hash".into(), json!(id.configuration_hash));
    out.insert("k_structure_interface".into(), json!(id.params.k_structure_interface));
    out.insert("k_rep".into(), json!(id.params.k_rep));
    out.insert("R0".into(), json!(r0));
    out.insert("C0".into(), json!(c0));
    out.insert("noise_seed".into(), json!(seed));
    out.insert("noise_amplitude".into(), json!(recipe.noise_amplitude));
    out.insert("source_commit".into(), json!(id.source_commit));
    out.insert("binary_hash".into(), json!(binary_hash()));
    out.insert("accepted_substeps".into(), json!(sim.substep - start_sub));
    out.insert("simulated_time".into(), json!(sim.sim_time));
    out.insert("initial_field_hashes".into(), initial_hashes);
    out.insert("final_field_hashes".into(), field_hashes(&sim));
    out.insert("initial_field_masses".into(), initial_masses);
    out.insert("final_field_masses".into(), field_masses(&sim));
    out.insert("reaction_accounting".into(), reaction_accounting);
    out.insert("diffusion_accounting".into(), diffusion_accounting);
    out.insert("reservoir_accounting".into(), reservoir_accounting);
    out.insert("numerical_corrections".into(), numerical_corrections);
    out.insert("accounting_residual".into(), json!(cum.cumulative_unexplained_residual));
    out.insert(
        "termination_status".into(),
        json!(if clean { "COMPLETED" } else { "ABORTED" }),
    );
    out.insert("clean_termination".into(), json!(clean));
    out.insert("Q_phi".into(), json!(balance.q_phi));
    out.insert("Q_C".into(), json!(balance.q_c));
    out.insert("slope_phi".into(), json!(balance.slope_phi));
    out.insert("slope_C".into(), json!(balance.slope_catalyst));
    out.insert("equivalent_radius".into(), json!(end_r));
    out.insert("v_R".into(), json!((end_r - start_r) / dt));
    out.insert("mean_C_inside".into(), json!(end_c_in));
    out.insert("v_C_inside".into(), json!((end_c_in - start_c_in) / dt));
    out.insert("retention".into(), json!(diag.catalyst_retention));
    out.insert("connected_component_fraction".into(), json!(conn));
    out.insert("turnover_ratios".into(), json!(diag.turnover_ratios));
    out.insert("resource_statistics".into(), res);
    out.insert("waste_statistics".into(), waste_statistics);
    out.insert(
        "classification".into(),
        json!(format!("{:?}", sim.detector.last_classification)),
    );
    out.insert("wall_seconds".into(), json!(t0.elapsed().as_secs_f64()));
    out.insert("initial_equivalent_radius".into(), json!(start_r));
    out.insert("initial_mean_C_inside".into(), json!(start_c_in));
    let out = serde_json::Value::Object(out);

    if !result_has_strict_schema(&out) {
        return Err("D-007 strict schema incomplete after run".into());
    }
    // Always write; selection gates refuse unclean rows separately.
    fs::write(&result_path, serde_json::to_string_pretty(&out)?)?;
    Ok(out)
}

pub fn run_reference_replay(substeps: u64) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    ensure_artifact_dirs()?;
    let id = write_reference_config()?;
    let out = d007_root().join("reference_replay").join("run");
    let rec = run_strict(&id, 24.0, 0.35, 2, substeps, &out)?;
    let ok = reference_flow_direction_ok(
        rec["v_R"].as_f64().unwrap_or(0.0),
        rec["v_C_inside"].as_f64().unwrap_or(0.0),
    );
    let summary = json!({
        "candidate_id": id.candidate_id,
        "configuration_hash": id.configuration_hash,
        "expected_configuration_hash": D006_REFERENCE_CONFIGURATION_HASH,
        "hash_match": id.configuration_hash == D006_REFERENCE_CONFIGURATION_HASH,
        "v_R": rec["v_R"],
        "v_C_inside": rec["v_C_inside"],
        "flow_direction_ok": ok,
        "substeps": substeps,
        "clean_termination": rec["clean_termination"],
        "result_path": out.join("result.json").display().to_string(),
    });
    fs::write(
        d007_root().join("reference_replay/summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    if !ok {
        return Err("reference replay did not reproduce D-006 flow direction".into());
    }
    Ok(summary)
}

pub fn make_joint_candidate(
    structural_factor: f64,
    k_rep: f64,
    parent_id: &str,
    reason: &str,
) -> CandidateIdentity {
    let mut p = reference_d006_params();
    p.k_structure_interface = D006_K_STRUCTURE_INTERFACE * structural_factor;
    p.k_rep = k_rep;
    build_candidate_identity(
        p,
        &git_commit_hash(),
        Some("surface_turnover_v1"),
        None,
        &format!("{reason}; parent={parent_id}; struct_fac={structural_factor}; k_rep={k_rep}"),
        None,
        None,
    )
}

pub fn write_joint_candidate(
    id: &CandidateIdentity,
    structural_factor: f64,
    catalyst_factor: f64,
    parent: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = PathBuf::from("configs/d007/joint_candidates").join(&id.candidate_id);
    if dir.join("identity.json").exists() {
        return Ok(dir);
    }
    fs::create_dir_all(&dir)?;
    let meta = json!({
        "candidate_id": id.candidate_id,
        "candidate_hash": id.candidate_hash,
        "configuration_hash": id.configuration_hash,
        "parent_d006_candidate": parent,
        "structural_factor": structural_factor,
        "k_structure_interface": id.params.k_structure_interface,
        "catalyst_factor": catalyst_factor,
        "k_rep": id.params.k_rep,
        "selection_provenance": id.selection_reason,
        "equation_version": id.equation_version,
    });
    fs::write(dir.join("identity.json"), serde_json::to_string_pretty(id)?)?;
    fs::write(dir.join("params.json"), serde_json::to_string_pretty(&id.params)?)?;
    fs::write(dir.join("meta.json"), serde_json::to_string_pretty(&meta)?)?;
    let art = d007_root().join("joint_candidates").join(&id.candidate_id);
    fs::create_dir_all(&art)?;
    fs::write(art.join("meta.json"), serde_json::to_string_pretty(&meta)?)?;
    Ok(dir)
}

pub fn write_structural_candidate(factor: f64) -> Result<CandidateIdentity, Box<dyn std::error::Error>> {
    let id = make_joint_candidate(
        factor,
        D006_K_REP,
        "d006-1.0x-reference",
        "D-007 structural bracket (frozen k_rep)",
    );
    let dir = d007_root()
        .join("structural_bracket")
        .join("candidates")
        .join(&id.candidate_id);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("identity.json"), serde_json::to_string_pretty(&id)?)?;
    fs::write(
        dir.join("meta.json"),
        serde_json::to_string_pretty(&json!({
            "structural_factor": factor,
            "k_structure_interface": id.params.k_structure_interface,
            "k_rep": id.params.k_rep,
            "candidate_id": id.candidate_id,
            "candidate_hash": id.candidate_hash,
            "configuration_hash": id.configuration_hash,
        }))?,
    )?;
    Ok(id)
}

pub fn ensure_artifact_dirs() -> Result<(), Box<dyn std::error::Error>> {
    for sub in [
        "reference_replay",
        "diagnosis",
        "structural_bracket",
        "catalyst_bracket",
        "joint_candidates",
        "joint_screen_j1",
        "joint_screen_j2",
        "nullclines",
        "fixed_points",
        "refined_basin",
        "puncture_mechanism",
        "controls",
        "full_acceptance",
    ] {
        fs::create_dir_all(d007_root().join(sub))?;
    }
    Ok(())
}
