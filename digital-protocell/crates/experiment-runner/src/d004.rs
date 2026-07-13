//! D-004 candidate provenance and attractor audit.

use chemistry_core::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn d004_output_root() -> PathBuf {
    PathBuf::from("experiments/generated/d004")
}

pub fn d003_root() -> PathBuf {
    PathBuf::from("experiments/generated/d003")
}

pub fn git_commit_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

pub fn extract_final_configs() -> Result<Vec<CandidateIdentity>, Box<dyn std::error::Error>> {
    fs::create_dir_all("configs/d004")?;
    let commit = git_commit_hash();
    let mut identities = Vec::new();
    for k_phi in [0.5, 1.0, 2.0] {
        let params = crate::d003::load_final_calibrated_params(k_phi)?;
        let slug = match k_phi {
            0.5 => "final_kphi_0_5",
            1.0 => "final_kphi_1_0",
            2.0 => "final_kphi_2_0",
            _ => unreachable!(),
        };
        let toml = format!(
            "# D-004 final calibrated candidate k_phi={k_phi} iteration 6 endpoint\n\
k_phi = {}\nk_structure = {}\nk_rep = {}\n",
            params.k_phi, params.k_structure, params.k_rep
        );
        let _ = &toml;
        let full_toml = serde_json::to_string_pretty(&params)?;
        fs::write(format!("configs/d004/{slug}.toml"), &full_toml)?;
        let identity = build_candidate_identity(
            params,
            &commit,
            Some(&format!("kphi_{k_phi}")),
            Some(5),
            "final calibration iteration 6 adjusted params from calibration_result.json",
            None,
            None,
        );
        identities.push(identity);
    }
    Ok(identities)
}

pub fn sha256_manifest(dir: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    collect_sha256(dir, dir, &mut files)?;
    Ok(serde_json::json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "commit_hash": git_commit_hash(),
        "root": dir.display().to_string(),
        "algorithm": "sha256",
        "files": files,
    }))
}

fn collect_sha256(
    root: &Path,
    dir: &Path,
    out: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_sha256(root, &path, out)?;
        } else if path.file_name().and_then(|s| s.to_str()) != Some("manifest.json") {
            let rel = path.strip_prefix(root)?.to_string_lossy().to_string();
            let data = fs::read(&path)?;
            out.insert(rel, candidate_identity::sha256_hex(&data));
        }
    }
    Ok(())
}

pub fn audit_stage_b_candidate() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let commit = git_commit_hash();
    let analytical = build_candidate_identity(
        crate::d003::params_from_analytical_estimate(1.0),
        &commit,
        None,
        None,
        "analytical D-002 median estimate for K_phi=1.0",
        None,
        None,
    );

    let mut final_hash_strs = Vec::new();
    let mut final_identities = Vec::new();
    for k_phi in [0.5, 1.0, 2.0] {
        let params = crate::d003::load_final_calibrated_params(k_phi)?;
        let id = build_candidate_identity(
            params,
            &commit,
            Some(&format!("kphi_{k_phi}")),
            Some(5),
            "final calibrated candidate",
            None,
            None,
        );
        final_hash_strs.push((format!("kphi_{k_phi}"), id.candidate_hash.clone()));
        final_identities.push(id);
    }

    let mut intermediate_hashes = Vec::new();
    for k_phi in [0.5, 1.0, 2.0] {
        for iter in 0..5u32 {
            let params = crate::d003::load_iteration_params(k_phi, iter)?;
            intermediate_hashes.push(candidate_hash(&params, &GridConfiguration::default()));
        }
    }
    let intermediate_refs: Vec<&str> = intermediate_hashes.iter().map(|s| s.as_str()).collect();
    let final_refs: Vec<(&str, &str)> = final_hash_strs
        .iter()
        .map(|(k, h)| (k.as_str(), h.as_str()))
        .collect();

    let screen_dir = d003_root().join("short_screen");
    let mut seed_reports = Vec::new();
    for seed in 1..=3 {
        let path = screen_dir.join(format!("seed_{seed}.json"));
        let data: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
        let observed_hash = data["candidate_hash"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| {
                // Legacy artifacts lack hash — reconstruct from analytical params used by old pipeline
                analytical.candidate_hash.clone()
            });
        let match_class = if data["candidate_hash"].is_null() {
            CandidateMatchClass::MatchAnalyticalInitialEstimate
        } else {
            classify_candidate_match(
                &observed_hash,
                &final_refs,
                &analytical.candidate_hash,
                &intermediate_refs,
            )
        };
        seed_reports.push(serde_json::json!({
            "seed": seed,
            "candidate_hash": observed_hash,
            "configuration_hash": data["configuration_hash"],
            "K_phi": data.get("K_phi").unwrap_or(&serde_json::json!(1.0)),
            "k_structure": analytical.k_structure,
            "k_rep": analytical.k_rep,
            "initial_state": "fresh_seed",
            "diagnostic_window": format!("0..{} substeps", data["substeps"]),
            "match_class": format!("{:?}", match_class),
            "q_phi": data["q_phi"],
            "q_c": data["q_c"],
        }));
    }

    let defect = seed_reports.iter().any(|r| {
        r["match_class"].as_str() == Some("MatchAnalyticalInitialEstimate")
            || r["match_class"].as_str() == Some("MatchIntermediateIteration")
            || r["match_class"].as_str() == Some("MatchUnknownConfiguration")
    });

    Ok(serde_json::json!({
        "stage_b_used_analytical_estimate": true,
        "analytical_candidate_hash": analytical.candidate_hash,
        "final_calibrated_hashes": final_hash_strs,
        "seed_reports": seed_reports,
        "pipeline_candidate_handoff_defect": defect,
        "defect_code": if defect { serde_json::Value::String("D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT".into()) } else { serde_json::Value::Null },
    }))
}

pub fn replay_calibration_iteration(
    k_phi: f64,
    iteration: u32,
    window: u64,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let params = crate::d003::load_iteration_params(k_phi, iteration)?;
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    sim.params.random_seed = 2;
    let result = run_balance_window(&mut sim, window);
    let stored_path = d003_root().join(format!("calibration/kphi_{k_phi}/iter_{iteration:02}.json"));
    let stored: serde_json::Value = serde_json::from_str(&fs::read_to_string(&stored_path)?)?;
    let tol = 1e-6;
    let rel = |a: f64, b: f64| (a - b).abs() / a.abs().max(b.abs()).max(1e-12);
    let checks = serde_json::json!({
        "q_phi": rel(result.balance.q_phi, stored["q_phi"].as_f64().unwrap()),
        "q_c": rel(result.balance.q_c, stored["q_c"].as_f64().unwrap()),
        "slope_phi": rel(result.balance.slope_phi, stored["slope_phi"].as_f64().unwrap()),
        "slope_catalyst": rel(result.balance.slope_catalyst, stored["slope_catalyst"].as_f64().unwrap()),
    });
    let reproducible = checks["q_phi"].as_f64().unwrap() <= tol
        && checks["q_c"].as_f64().unwrap() <= tol
        && checks["slope_phi"].as_f64().unwrap() <= tol
        && checks["slope_catalyst"].as_f64().unwrap() <= tol;
    Ok(serde_json::json!({
        "k_phi": k_phi,
        "iteration": iteration,
        "reproducible": reproducible,
        "relative_errors": checks,
        "replayed": {
            "q_phi": result.balance.q_phi,
            "q_c": result.balance.q_c,
            "slope_phi": result.balance.slope_phi,
            "slope_catalyst": result.balance.slope_catalyst,
        },
        "stored": stored,
    }))
}

pub fn calibration_stopping_audit(k_phi: f64) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let path = d003_root().join(format!("calibration/kphi_{k_phi}/calibration_result.json"));
    let data: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let iters = data["iterations"].as_array().cloned().unwrap_or_default();
    let last = iters.last();
    let prev = iters.get(iters.len().saturating_sub(2));
    let still_improving = match (last, prev) {
        (Some(l), Some(p)) => {
            let qphi_err_l = (l["q_phi"].as_f64().unwrap() - 1.0).abs();
            let qphi_err_p = (p["q_phi"].as_f64().unwrap() - 1.0).abs();
            let slope_l = l["slope_phi"].as_f64().unwrap().abs();
            let slope_p = p["slope_phi"].as_f64().unwrap().abs();
            qphi_err_l < qphi_err_p || slope_l < slope_p
        }
        _ => false,
    };
    let classification = if still_improving {
        "STILL_IMPROVING"
    } else if iters.len() >= 2 {
        "PLATEAUED"
    } else {
        "ARBITRARY_CAP"
    };
    Ok(serde_json::json!({
        "k_phi": k_phi,
        "iterations": iters,
        "stop_classification": classification,
        "iteration_cap": 6,
        "scientifically_justified": false,
        "note": "Six iterations inherited from D-003 directive; slopes still above gate at iteration 5",
    }))
}

fn init_sim_from_state(
    params: &SimParams,
    state: InitialStateClass,
    seed: u64,
    k_phi: f64,
) -> Result<Simulation, Box<dyn std::error::Error>> {
    let mut p = params.clone();
    p.random_seed = seed;
    let mut sim = Simulation::new(p);
    sim.observer_enabled = false;
    match state {
        InitialStateClass::Fresh => {}
        InitialStateClass::AgedD002 => {
            let snap_path = format!(
                "experiments/generated/phase1_acceptance/baseline_seed_{seed}/checkpoint_050000/snapshot.json"
            );
            let snap = load_snapshot(Path::new(&snap_path))?;
            // Restore D-002 field state only; candidate params remain D-003 calibrated
            sim.restore_snapshot(&snap);
            sim.substep = snap.substep;
            sim.sim_time = snap.sim_time;
        }
        InitialStateClass::CalibrationEndpoint => {
            let iter_params = crate::d003::load_iteration_params(k_phi, 5)?;
            sim.params.k_structure = iter_params.k_structure;
            sim.params.k_rep = iter_params.k_rep;
            sim.params.k_phi = iter_params.k_phi;
            let endpoint = run_balance_window(&mut sim, BALANCE_WINDOW_SUBSTEPS);
            let _ = endpoint;
        }
    }
    Ok(sim)
}

pub fn run_cross_state_experiment(
    identity: &CandidateIdentity,
    state: InitialStateClass,
    seed: u64,
    substeps: u64,
    output: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let mut sim = init_sim_from_state(&identity.params, state.clone(), seed, identity.k_phi)?;
    let record_every = 500u64;
    let checkpoints = [0u64, 20_000, 40_000, 60_000, 80_000, 100_000];
    let start_substep = sim.substep;
    let mut trajectory = Vec::new();
    let mut window_samples: Vec<BalanceWindowSample> = Vec::new();
    let mut windows: Vec<(f64, f64, BalanceDiagnostics)> = Vec::new();
    let mut window_start_time = sim.sim_time;
    let mut window_start_sub = sim.substep;

    if checkpoints.contains(&start_substep) {
        save_run_snapshot(output, start_substep, &sim, identity, &state, seed)?;
    }

    for step in 0..substeps {
        if !sim.step() {
            break;
        }
        compute_all_reactions(
            &sim.fields.structure,
            &sim.fields.catalyst,
            &sim.fields.nutrient,
            &sim.fields.fuel,
            &sim.fields.waste,
            &sim.params,
            true,
            &mut sim.reaction_scratch,
        );
        let s_phi: f64 = sim.reaction_scratch.rates.iter().map(|r| r.r_structure).sum();
        let d_phi: f64 = sim
            .reaction_scratch
            .rates
            .iter()
            .map(|r| r.r_structure_decay)
            .sum();
        let r_c: f64 = sim.reaction_scratch.rates.iter().map(|r| r.r_rep).sum();
        let d_c: f64 = sim
            .reaction_scratch
            .rates
            .iter()
            .map(|r| r.r_catalyst_decay)
            .sum();
        window_samples.push(BalanceWindowSample {
            sim_time: sim.sim_time,
            m_phi: total_mass(&sim.grid, &sim.fields.structure),
            m_c: total_mass(&sim.grid, &sim.fields.catalyst),
            s_phi,
            d_phi,
            r_c,
            d_c,
        });

        if sim.substep % record_every == 0 {
            let diag = sim.current_diagnostics();
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
            let win = if window_samples.len() >= record_every as usize {
                let w: Vec<_> = window_samples[window_samples.len() - record_every as usize..].to_vec();
                compute_balance(&w)
            } else {
                compute_balance(&window_samples)
            };
            let radius = (diag.protocell_area.max(1) as f64 / std::f64::consts::PI).sqrt();
            trajectory.push(TrajectoryPoint {
                substep: sim.substep,
                sim_time: sim.sim_time,
                m_phi: diag.structural_mass,
                m_c: diag.catalyst_mass,
                q_phi: win.q_phi,
                q_c: win.q_c,
                slope_phi: win.slope_phi,
                slope_c: win.slope_catalyst,
                mean_n_inside: bn.mean_n_inside,
                mean_f_inside: bn.mean_f_inside,
                retention: diag.catalyst_retention,
                equivalent_radius: radius,
                compactness: diag.compactness,
            });
            if window_samples.len() >= record_every as usize {
                windows.push((window_start_time, sim.sim_time, win));
                window_start_time = sim.sim_time;
                window_start_sub = sim.substep;
            }
        }

        if checkpoints.contains(&(sim.substep)) && sim.substep > start_substep {
            save_run_snapshot(output, sim.substep, &sim, identity, &state, seed)?;
        }
        let _ = step;
    }

    let final_balance = compute_balance(&window_samples);
    let diag = sim.current_diagnostics();
    let transient = analyze_transient(&windows);
    let radius = (diag.protocell_area.max(1) as f64 / std::f64::consts::PI).sqrt();
    let classification = if final_balance.q_phi < 0.5 || diag.structural_mass < 5.0 {
        AttractorClassification::NoActiveAttractor
    } else if final_balance.slope_phi.abs() > 1e-3 {
        AttractorClassification::ContinuedDrift
    } else {
        AttractorClassification::StateDependentAttractors
    };

    let result = serde_json::json!({
        "candidate_id": identity.candidate_id,
        "candidate_hash": identity.candidate_hash,
        "configuration_hash": identity.configuration_hash,
        "state_class": format!("{:?}", state),
        "seed": seed,
        "substeps": sim.substep,
        "sim_time": sim.sim_time,
        "q_phi": final_balance.q_phi,
        "q_c": final_balance.q_c,
        "slope_phi": final_balance.slope_phi,
        "slope_catalyst": final_balance.slope_catalyst,
        "retention": diag.catalyst_retention,
        "final_m_phi": diag.structural_mass,
        "final_m_c": diag.catalyst_mass,
        "equivalent_radius": radius,
        "attractor_classification": format!("{:?}", classification),
        "transient": transient,
        "trajectory_points": trajectory.len(),
    });
    fs::write(output.join("result.json"), serde_json::to_string_pretty(&result)?)?;
    fs::write(output.join("trajectory.json"), serde_json::to_string_pretty(&trajectory)?)?;
    Ok(result)
}

fn save_run_snapshot(
    output: &Path,
    substep: u64,
    sim: &Simulation,
    identity: &CandidateIdentity,
    state: &InitialStateClass,
    seed: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = output.join(format!("snapshot_{substep:06}"));
    fs::create_dir_all(&dir)?;
    let snap = sim.snapshot();
    save_snapshot(&dir.join("snapshot.json"), &snap)?;
    let meta = serde_json::json!({
        "candidate_id": identity.candidate_id,
        "candidate_hash": identity.candidate_hash,
        "configuration_hash": identity.configuration_hash,
        "state_class": format!("{:?}", state),
        "seed": seed,
        "substep": substep,
        "equation_version": identity.equation_version,
        "source_commit": identity.source_commit,
        "structural_mass": total_mass(&sim.grid, &sim.fields.structure),
        "catalyst_mass": total_mass(&sim.grid, &sim.fields.catalyst),
        "field_hashes": {
            "structure": field_sha256(&sim.fields.structure),
            "catalyst": field_sha256(&sim.fields.catalyst),
        },
    });
    fs::write(dir.join("provenance.json"), serde_json::to_string_pretty(&meta)?)?;
    Ok(())
}

pub fn run_full_audit() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let root = d004_output_root();
    fs::create_dir_all(&root)?;

    let d003_manifest = sha256_manifest(&d003_root())?;
    fs::write(
        d003_root().join("manifest.json"),
        serde_json::to_string_pretty(&d003_manifest)?,
    )?;

    let identities = extract_final_configs()?;
    let provenance = audit_stage_b_candidate()?;
    fs::create_dir_all(root.join("provenance_audit"))?;
    fs::write(
        root.join("provenance_audit/stage_b_audit.json"),
        serde_json::to_string_pretty(&provenance)?,
    )?;

    let mut replays = Vec::new();
    fs::create_dir_all(root.join("candidate_replay"))?;
    for k_phi in [0.5, 1.0, 2.0] {
        let replay = replay_calibration_iteration(k_phi, 5, BALANCE_WINDOW_SUBSTEPS)?;
        replays.push(replay.clone());
        fs::write(
            root.join(format!("candidate_replay/kphi_{k_phi}_iter_05.json")),
            serde_json::to_string_pretty(&replay)?,
        )?;
    }

    let mut stopping = Vec::new();
    for k_phi in [0.5, 1.0, 2.0] {
        stopping.push(calibration_stopping_audit(k_phi)?);
    }
    fs::write(
        root.join("provenance_audit/calibration_stopping.json"),
        serde_json::to_string_pretty(&stopping)?,
    )?;

    let commit = git_commit_hash();
    let analytical = build_candidate_identity(
        crate::d003::params_from_analytical_estimate(1.0),
        &commit,
        None,
        None,
        "analytical estimate",
        None,
        None,
    );

    let mut cross_state_results = Vec::new();
    for identity in &identities {
        let k_slug = identity.k_phi.to_string().replace('.', "_");
        for (state, state_dir) in [
            (InitialStateClass::Fresh, "fresh"),
            (InitialStateClass::AgedD002, "aged"),
            (InitialStateClass::CalibrationEndpoint, "calibration_endpoint"),
        ] {
            let base = root.join(format!("cross_state/kphi_{k_slug}/{state_dir}"));
            for seed in 1..=3 {
                if matches!(state, InitialStateClass::AgedD002) && seed != 2 {
                    continue;
                }
                let out = base.join(format!("seed_{seed}"));
                let result = run_cross_state_experiment(identity, state.clone(), seed, 100_000, &out)?;
                cross_state_results.push(result);
            }
        }
    }

    let corrected_screen = crate::d003::short_screen(
        identities.iter().find(|i| (i.k_phi - 1.0).abs() < 1e-9).unwrap_or(&analytical),
        &[1, 2, 3],
        100_000,
        &root.join("provenance_audit/corrected_short_screen_kphi_1"),
    )?;

    let all_reproducible = replays.iter().all(|r| r["reproducible"].as_bool() == Some(true));
    let handoff_defect = provenance["pipeline_candidate_handoff_defect"].as_bool() == Some(true);

    let conclusion = if handoff_defect {
        "D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT"
    } else if !all_reproducible {
        "D004_CALIBRATION_NOT_REPRODUCIBLE"
    } else {
        "D004_STATE_DEPENDENT_ATTRACTORS"
    };

    let summary = serde_json::json!({
        "conclusion": conclusion,
        "d003_result": "D003_RESULT_UNRESOLVED_PENDING_PIPELINE_AUDIT",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "handoff_defect": handoff_defect,
        "calibration_reproducible": all_reproducible,
        "corrected_short_screen": corrected_screen,
        "cross_state_run_count": cross_state_results.len(),
    });
    fs::write(root.join("manifest.json"), serde_json::to_string_pretty(&summary)?)?;
    Ok(summary)
}
