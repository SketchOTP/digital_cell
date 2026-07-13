//! D-003 calibration and screening CLI logic.

use chemistry_core::*;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_diagnosis(output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let mut seed_audits = Vec::new();
    for seed in 1..=5 {
        let mut p = baseline_params();
        p.random_seed = seed;
        let sim = Simulation::new(p);
        seed_audits.push(audit_initial_seed(
            &sim.grid,
            &sim.fields,
            seed,
            sim.params.seed_r0,
        ));
    }
    fs::write(
        output.join("seed_audit.json"),
        serde_json::to_string_pretty(&seed_audits)?,
    )?;

    let mut d002_summary = Vec::new();
    for seed in 1..=5 {
        let path = format!("experiments/generated/phase1_acceptance/baseline_seed_{seed}/summary.json");
        if let Ok(data) = fs::read_to_string(path) {
            d002_summary.push(serde_json::from_str::<serde_json::Value>(&data)?);
        }
    }
    fs::write(
        output.join("d002_reference.json"),
        serde_json::to_string_pretty(&d002_summary)?,
    )?;
    Ok(())
}

pub fn analytical_estimates_from_d002(k_phi: f64) -> serde_json::Value {
    let mut per_seed = Vec::new();
    for seed in 1..=5 {
        let summary_path = format!("experiments/generated/phase1_acceptance/baseline_seed_{seed}/summary.json");
        let snapshot_path = format!("experiments/generated/phase1_acceptance/baseline_seed_{seed}/snapshot.json");
        if let Ok(data) = fs::read_to_string(&summary_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                let d_struct = v["turnover"]["structural_decay"].as_f64().unwrap_or(0.0);
                let d_cat = v["turnover"]["catalyst_decay"].as_f64().unwrap_or(0.0);
                let sim_time = fs::read_to_string(&snapshot_path)
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|snap| snap["sim_time"].as_f64())
                    .unwrap_or(39.0625);
                let mut p = legacy_d002_params();
                p.k_phi = k_phi;
                p.use_legacy_structure_kinetics = false;
                let mut sim = Simulation::new(p);
                sim.params.random_seed = seed;
                initialize_seed(&sim.grid, &sim.params, &mut sim.fields);
                let b_struct = integrated_structure_prefactor(
                    &sim.fields.structure,
                    &sim.fields.catalyst,
                    &sim.fields.nutrient,
                    &sim.fields.fuel,
                    &sim.grid.dish_mask,
                    &sim.params,
                );
                let b_rep = integrated_rep_prefactor(
                    &sim.fields.structure,
                    &sim.fields.catalyst,
                    &sim.fields.nutrient,
                    &sim.fields.fuel,
                    &sim.grid.dish_mask,
                    sim.params.c_max,
                );
                let k_struct_req = d_struct / (b_struct * sim_time).max(1e-12);
                let k_rep_req = d_cat / (b_rep * sim_time).max(1e-12);
                per_seed.push(serde_json::json!({
                    "seed": seed,
                    "d_structure": d_struct,
                    "d_catalyst": d_cat,
                    "b_structure_snapshot": b_struct,
                    "b_rep_snapshot": b_rep,
                    "k_structure_required": k_struct_req,
                    "k_rep_required": k_rep_req,
                }));
            }
        }
    }
    let ks: Vec<f64> = per_seed
        .iter()
        .filter_map(|v| v["k_structure_required"].as_f64())
        .collect();
    let kr: Vec<f64> = per_seed
        .iter()
        .filter_map(|v| v["k_rep_required"].as_f64())
        .collect();
    let median = |xs: &[f64]| {
        if xs.is_empty() {
            return 0.0;
        }
        let mut v = xs.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    serde_json::json!({
        "k_phi": k_phi,
        "per_seed": per_seed,
        "k_structure_median": median(&ks),
        "k_rep_median": median(&kr),
    })
}

pub fn params_from_analytical_estimate(k_phi: f64) -> SimParams {
    let est = analytical_estimates_from_d002(k_phi);
    let mut params = baseline_params();
    params.k_phi = k_phi;
    params.k_structure = est["k_structure_median"].as_f64().unwrap_or(0.03);
    params.k_rep = est["k_rep_median"].as_f64().unwrap_or(0.012);
    params
}

pub fn load_final_calibrated_params(k_phi: f64) -> Result<SimParams, Box<dyn std::error::Error>> {
    let path = d003_output_root().join(format!("calibration/kphi_{k_phi}/calibration_result.json"));
    let data = fs::read_to_string(&path)?;
    let v: serde_json::Value = serde_json::from_str(&data)?;
    let params: SimParams = serde_json::from_value(v["final_params"].clone())?;
    Ok(params)
}

pub fn load_iteration_params(k_phi: f64, iteration: u32) -> Result<SimParams, Box<dyn std::error::Error>> {
    let path = d003_output_root().join(format!("calibration/kphi_{k_phi}/iter_{iteration:02}.json"));
    let data = fs::read_to_string(&path)?;
    let v: serde_json::Value = serde_json::from_str(&data)?;
    let mut params = baseline_params();
    params.k_phi = k_phi;
    params.k_structure = v["k_structure"].as_f64().unwrap_or(0.03);
    params.k_rep = v["k_rep"].as_f64().unwrap_or(0.012);
    Ok(params)
}

pub fn calibrate_kphi(k_phi: f64, output: &Path, window: u64) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let estimates = analytical_estimates_from_d002(k_phi);
    let mut params = params_from_analytical_estimate(k_phi);

    let mut iterations = Vec::new();
    for iter in 0..6 {
        let mut sim = Simulation::new(params.clone());
        sim.observer_enabled = false;
        sim.params.random_seed = 2;
        let result = run_balance_window(&mut sim, window);
        let balance = result.balance;
        let bn = compute_bottleneck(
            &sim.grid,
            &sim.fields.structure,
            &sim.fields.catalyst,
            &sim.fields.nutrient,
            &sim.fields.fuel,
            &sim.reaction_scratch,
            sim.params.c_max,
            sim.params.n_reservoir,
            sim.params.f_reservoir,
        );
        let entry = serde_json::json!({
            "iteration": iter,
            "k_structure": params.k_structure,
            "k_rep": params.k_rep,
            "q_phi": balance.q_phi,
            "q_c": balance.q_c,
            "slope_phi": balance.slope_phi,
            "slope_catalyst": balance.slope_catalyst,
            "retention_limited": bn.retention_limited,
            "transport_limited": bn.transport_limited,
            "passes": balance_window_passes(&balance),
        });
        iterations.push(entry.clone());
        fs::write(output.join(format!("iter_{iter:02}.json")), serde_json::to_string_pretty(&entry)?)?;

        if balance_window_passes(&balance) && iter > 0 {
            if iterations.iter().rev().take(2).all(|e| e["passes"].as_bool() == Some(true)) {
                break;
            }
        }

        let adj_s = (1.0 / balance.q_phi.max(1e-6)).sqrt().clamp(0.5, 2.0);
        let adj_c = (1.0 / balance.q_c.max(1e-6)).sqrt().clamp(0.5, 2.0);
        params.k_structure *= adj_s;
        params.k_rep *= adj_c;
    }

    let result = serde_json::json!({
        "k_phi": k_phi,
        "final_params": params,
        "iterations": iterations,
    });
    fs::write(output.join("calibration_result.json"), serde_json::to_string_pretty(&result)?)?;
    Ok(result)
}

pub fn short_screen(
    identity: &CandidateIdentity,
    seeds: &[u64],
    substeps: u64,
    output: &Path,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let mut results = Vec::new();
    for &seed in seeds {
        let mut p = identity.params.clone();
        p.random_seed = seed;
        let mut sim = Simulation::new(p);
        sim.observer_enabled = false;
        let result = run_balance_window(&mut sim, substeps);
        let balance = result.balance;
        let ratios = sim.detector.turnover_ratios();
        let diag = sim.current_diagnostics();
        let pass = balance.q_phi >= 0.80
            && balance.q_phi <= 1.25
            && balance.q_c >= 0.80
            && balance.q_c <= 1.25
            && diag.catalyst_retention >= 0.75;
        let entry = serde_json::json!({
            "seed": seed,
            "substeps": result.substeps,
            "sim_time": result.sim_time,
            "q_phi": balance.q_phi,
            "q_c": balance.q_c,
            "slope_phi": balance.slope_phi,
            "slope_catalyst": balance.slope_catalyst,
            "classification": format!("{:?}", diag.classification),
            "retention": diag.catalyst_retention,
            "turnover_ratios": ratios,
            "pass": pass,
            "candidate_id": identity.candidate_id,
            "candidate_hash": identity.candidate_hash,
            "configuration_hash": identity.configuration_hash,
        });
        results.push(entry.clone());
        fs::write(
            output.join(format!("seed_{seed}.json")),
            serde_json::to_string_pretty(&entry)?,
        )?;
    }
    Ok(results)
}

pub fn d003_output_root() -> PathBuf {
    PathBuf::from("experiments/generated/d003")
}
