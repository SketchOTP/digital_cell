//! D-005 accessible active-attractor and viability-basin mapping.

use chemistry_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn d005_output_root() -> PathBuf {
    PathBuf::from("experiments/generated/d005")
}

pub fn d004_root() -> PathBuf {
    PathBuf::from("experiments/generated/d004")
}

fn git_commit_hash() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn git_dirty() -> bool {
    std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(true)
}

pub fn load_d004_identities() -> Result<Vec<CandidateIdentity>, Box<dyn std::error::Error>> {
    let commit = git_commit_hash();
    let mut ids = Vec::new();
    for k_phi in [0.5, 1.0, 2.0] {
        let params = crate::d003::load_final_calibrated_params(k_phi)?;
        ids.push(build_candidate_identity(
            params,
            &commit,
            Some(&format!("kphi_{k_phi}")),
            Some(5),
            "D-004 final calibrated candidate",
            None,
            None,
        ));
    }
    Ok(ids)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub candidate_hash: String,
    pub candidate_id: String,
    pub configuration_hash: String,
    pub state_class: String,
    pub seed: u64,
    pub seed_recipe: Option<String>,
    pub r0: Option<f64>,
    pub c0: Option<f64>,
    pub noise_amplitude: Option<f64>,
    pub starting_m_phi: f64,
    pub starting_m_c: f64,
    pub final_m_phi: f64,
    pub final_m_c: f64,
    pub q_phi: f64,
    pub q_c: f64,
    pub slope_phi: f64,
    pub slope_c: f64,
    pub retention: f64,
    pub equivalent_radius: f64,
    pub connected_fraction: f64,
    pub substeps: u64,
    pub sim_time: f64,
    pub classification: String,
    pub basin_outcome: Option<String>,
    pub turnover_ratios: TurnoverRatios,
    pub consecutive_stable_windows: u32,
    pub trajectory: Vec<TrajectoryPoint>,
}

pub struct SimRunConfig {
    pub substeps: u64,
    pub record_every: u64,
    pub checkpoint_every: u64,
    pub trajectory_sample_every: u64,
}

impl Default for SimRunConfig {
    fn default() -> Self {
        Self {
            substeps: 20_000,
            record_every: 500,
            checkpoint_every: 5_000,
            trajectory_sample_every: 500,
        }
    }
}

pub fn advance_simulation(
    sim: &mut Simulation,
    cfg: &SimRunConfig,
) -> (BalanceDiagnostics, Vec<TrajectoryPoint>, Vec<(f64, f64, BalanceDiagnostics, f64, f64)>) {
    let start_sub = sim.substep;
    let mut trajectory = Vec::new();
    let mut window_samples: Vec<BalanceWindowSample> = Vec::new();
    let mut d005_windows: Vec<(f64, f64, BalanceDiagnostics, f64, f64)> = Vec::new();
    let mut window_start_time = sim.sim_time;

    for _ in 0..cfg.substeps {
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

        if sim.substep % cfg.trajectory_sample_every == 0 {
            let diag = sim.current_diagnostics();
            let win = if window_samples.len() >= cfg.record_every as usize {
                compute_balance(&window_samples[window_samples.len() - cfg.record_every as usize..])
            } else {
                compute_balance(&window_samples)
            };
            let radius = (diag.protocell_area.max(1) as f64 / std::f64::consts::PI).sqrt();
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
            if window_samples.len() >= cfg.record_every as usize {
                let conn = diag.largest_component as f64 / diag.protocell_area.max(1) as f64;
                d005_windows.push((
                    window_start_time,
                    sim.sim_time,
                    win,
                    diag.catalyst_retention,
                    conn,
                ));
                window_start_time = sim.sim_time;
            }
        }
        let _ = start_sub;
    }

    let final_balance = compute_balance(&window_samples);
    (final_balance, trajectory, d005_windows)
}

pub fn run_fresh_seed(
    identity: &CandidateIdentity,
    recipe: &FreshSeedRecipe,
    cfg: &SimRunConfig,
) -> RunRecord {
    let mut sim = spawn_fresh_simulation(identity.params.clone(), recipe);
    sim.observer_enabled = true;
    let start_m_phi = total_mass(&sim.grid, &sim.fields.structure);
    let start_m_c = total_mass(&sim.grid, &sim.fields.catalyst);
    let (balance, trajectory, d005_windows) = advance_simulation(&mut sim, cfg);
    let diag = sim.current_diagnostics();
    let radius = (diag.protocell_area.max(1) as f64 / std::f64::consts::PI).sqrt();
    let conn = diag.largest_component as f64 / diag.protocell_area.max(1) as f64;
    let stable = count_consecutive_d005_windows(&d005_windows);
    let outcome = classify_basin_outcome(
        recipe.r0,
        radius,
        balance.q_phi,
        balance.slope_phi,
        diag.catalyst_retention,
        conn,
        sim.rejection_count,
    );
    let classification = if matches!(sim.detector.last_classification, ViabilityClass::Dead) {
        "DEAD".to_string()
    } else if stable >= 3 {
        "STABLE_ACTIVE".to_string()
    } else if balance.slope_phi.abs() <= 1e-3 {
        "NEAR_BALANCE".to_string()
    } else if balance.slope_phi < -1e-3 {
        "CONTINUED_DECLINE".to_string()
    } else {
        format!("{:?}", sim.detector.last_classification)
    };

    RunRecord {
        candidate_hash: identity.candidate_hash.clone(),
        candidate_id: identity.candidate_id.clone(),
        configuration_hash: identity.configuration_hash.clone(),
        state_class: "Fresh".into(),
        seed: recipe.noise_seed,
        seed_recipe: Some(recipe.identity_key()),
        r0: Some(recipe.r0),
        c0: Some(recipe.c0),
        noise_amplitude: Some(recipe.noise_amplitude),
        starting_m_phi: start_m_phi,
        starting_m_c: start_m_c,
        final_m_phi: diag.structural_mass,
        final_m_c: diag.catalyst_mass,
        q_phi: balance.q_phi,
        q_c: balance.q_c,
        slope_phi: balance.slope_phi,
        slope_c: balance.slope_catalyst,
        retention: diag.catalyst_retention,
        equivalent_radius: radius,
        connected_fraction: conn,
        substeps: sim.substep,
        sim_time: sim.sim_time,
        classification,
        basin_outcome: Some(basin_outcome_name(outcome)),
        turnover_ratios: sim.detector.turnover_ratios(),
        consecutive_stable_windows: stable,
        trajectory,
    }
}

fn basin_outcome_name(o: BasinOutcome) -> String {
    match o {
        BasinOutcome::RapidCollapse => "rapid_collapse",
        BasinOutcome::SlowDecline => "slow_decline",
        BasinOutcome::NearBalance => "near_balance",
        BasinOutcome::Growth => "growth",
        BasinOutcome::UnboundedGrowth => "unbounded_growth",
        BasinOutcome::Fragmentation => "fragmentation",
        BasinOutcome::NumericalFailure => "numerical_failure",
    }
    .into()
}

pub fn aggregate_d004_cross_state() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let root = d005_output_root().join("d004_aggregate");
    fs::create_dir_all(&root)?;
    let mut rows = Vec::new();

    for k_phi in [0.5, 1.0, 2.0] {
        let k_slug = k_phi.to_string().replace('.', "_");
        for (state_dir, state_label) in [
            ("fresh", "fresh_seed"),
            ("aged", "D-002_aged_state"),
            ("calibration_endpoint", "calibration_endpoint"),
        ] {
            let base = d004_root().join(format!("cross_state/kphi_{k_slug}/{state_dir}"));
            if !base.exists() {
                continue;
            }
            for entry in fs::read_dir(&base)? {
                let entry = entry?;
                let seed_dir = entry.path();
                if !seed_dir.is_dir() {
                    continue;
                }
                let result_path = seed_dir.join("result.json");
                if !result_path.exists() {
                    continue;
                }
                let data: serde_json::Value =
                    serde_json::from_str(&fs::read_to_string(&result_path)?)?;
                let traj_path = seed_dir.join("trajectory.json");
                let trajectory: Vec<TrajectoryPoint> = if traj_path.exists() {
                    serde_json::from_str(&fs::read_to_string(&traj_path)?)?
                } else {
                    vec![]
                };
                let behavior = classify_run_behavior(&data, &trajectory);
                rows.push(serde_json::json!({
                    "k_phi": k_phi,
                    "candidate_hash": data["candidate_hash"],
                    "state_class": state_label,
                    "seed": data["seed"],
                    "starting_structural_mass": data.get("starting_m_phi").unwrap_or(&serde_json::Value::Null),
                    "starting_catalyst_mass": data.get("starting_m_c").unwrap_or(&serde_json::Value::Null),
                    "final_structural_mass": data["final_m_phi"],
                    "final_catalyst_mass": data["final_m_c"],
                    "q_phi": data["q_phi"],
                    "q_c": data["q_c"],
                    "slope_phi": data["slope_phi"],
                    "slope_c": data["slope_catalyst"],
                    "retention": data["retention"],
                    "equivalent_radius": data["equivalent_radius"],
                    "connected_fraction": data.get("connected_fraction").unwrap_or(&serde_json::json!(null)),
                    "simulated_time": data["sim_time"],
                    "classification": data["attractor_classification"],
                    "behavior": behavior,
                }));
            }
        }
    }

    let summary = serde_json::json!({
        "run_count": rows.len(),
        "rows": rows,
    });
    fs::write(root.join("aggregate.json"), serde_json::to_string_pretty(&summary)?)?;
    Ok(summary)
}

fn classify_run_behavior(data: &serde_json::Value, trajectory: &[TrajectoryPoint]) -> &'static str {
    let slope = data["slope_phi"].as_f64().unwrap_or(0.0);
    let q = data["q_phi"].as_f64().unwrap_or(0.0);
    let final_m = data["final_m_phi"].as_f64().unwrap_or(0.0);
    if final_m < 5.0 {
        return "continued_collapse";
    }
    if slope > 1e-3 {
        return "continued_growth";
    }
    if !data["transient"]["t_settle"].is_null() {
        return "temporary_balance";
    }
    if trajectory.len() >= 2 {
        let r0 = trajectory.first().map(|p| p.equivalent_radius).unwrap_or(0.0);
        let r1 = trajectory.last().map(|p| p.equivalent_radius).unwrap_or(0.0);
        if (r1 - r0).abs() / r0.max(1.0) > 0.15 && slope.abs() < 1e-3 {
            return "state_dependent_balance";
        }
    }
    if slope.abs() <= 1e-4 && (0.98..=1.02).contains(&q) {
        return "cross_state_convergence";
    }
    if trajectory.len() > 100 && slope.abs() > 1e-4 {
        return "unresolved_long_transient";
    }
    "continued_drift"
}

pub fn run_continuation(
    identity: &CandidateIdentity,
    snapshot_dir: &Path,
    additional_substeps: u64,
    output: &Path,
) -> Result<RunRecord, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let snap_path = snapshot_dir.join("snapshot.json");
    let prov_path = snapshot_dir.join("provenance.json");
    let (mut sim, verification) = continue_from_snapshot(
        &snap_path,
        Some(&prov_path),
        identity,
        true,
    )?;
    fs::write(
        output.join("continuation_verification.json"),
        serde_json::to_string_pretty(&verification)?,
    )?;

    let start_m_phi = total_mass(&sim.grid, &sim.fields.structure);
    let start_m_c = total_mass(&sim.grid, &sim.fields.catalyst);
    let cfg = SimRunConfig {
        substeps: additional_substeps,
        record_every: 500,
        checkpoint_every: 5_000,
        trajectory_sample_every: 500,
    };
    let (balance, trajectory, d005_windows) = advance_simulation(&mut sim, &cfg);
    let diag = sim.current_diagnostics();
    let radius = (diag.protocell_area.max(1) as f64 / std::f64::consts::PI).sqrt();
    let conn = diag.largest_component as f64 / diag.protocell_area.max(1) as f64;
    let stable = count_consecutive_d005_windows(&d005_windows);

    let record = RunRecord {
        candidate_hash: identity.candidate_hash.clone(),
        candidate_id: identity.candidate_id.clone(),
        configuration_hash: identity.configuration_hash.clone(),
        state_class: "FreshContinuation".into(),
        seed: sim.params.random_seed,
        seed_recipe: None,
        r0: None,
        c0: None,
        noise_amplitude: None,
        starting_m_phi: start_m_phi,
        starting_m_c: start_m_c,
        final_m_phi: diag.structural_mass,
        final_m_c: diag.catalyst_mass,
        q_phi: balance.q_phi,
        q_c: balance.q_c,
        slope_phi: balance.slope_phi,
        slope_c: balance.slope_catalyst,
        retention: diag.catalyst_retention,
        equivalent_radius: radius,
        connected_fraction: conn,
        substeps: sim.substep,
        sim_time: sim.sim_time,
        classification: if stable >= 3 {
            "STABLE_ACTIVE".into()
        } else if matches!(sim.detector.last_classification, ViabilityClass::Dead) {
            "DEAD".into()
        } else {
            "CONTINUED_DRIFT".into()
        },
        basin_outcome: None,
        turnover_ratios: sim.detector.turnover_ratios(),
        consecutive_stable_windows: stable,
        trajectory,
    };
    fs::write(output.join("result.json"), serde_json::to_string_pretty(&record)?)?;
    Ok(record)
}

pub fn run_all_continuations(max_substeps: u64) -> Result<Vec<RunRecord>, Box<dyn std::error::Error>> {
    let identities = load_d004_identities()?;
    let mut results = Vec::new();
    for identity in &identities {
        let k_slug = identity.k_phi.to_string().replace('.', "_");
        let cont_root = d005_output_root().join(format!("continuations/kphi_{k_slug}"));
        for seed in 1..=3u64 {
            let snap_dir = d004_root().join(format!(
                "cross_state/kphi_{k_slug}/fresh/seed_{seed}/snapshot_100000"
            ));
            if !snap_dir.exists() {
                continue;
            }
            let out = cont_root.join(format!("fresh_seed_{seed}"));
            // ponytail: resume gate — skip finished runs after OOM/kill; incomplete dirs keep verification.json only
            let result_path = out.join("result.json");
            if result_path.exists() {
                let existing: RunRecord = serde_json::from_str(&fs::read_to_string(&result_path)?)?;
                results.push(existing);
                continue;
            }
            let current = {
                let snap = load_snapshot(&snap_dir.join("snapshot.json"))?;
                snap.substep
            };
            let additional = max_substeps.saturating_sub(current);
            if additional == 0 {
                continue;
            }
            eprintln!(
                "D-005 continuation k_phi={} seed={} +{} substeps -> {}",
                identity.k_phi,
                seed,
                additional,
                out.display()
            );
            results.push(run_continuation(identity, &snap_dir, additional, &out)?);
        }
    }
    Ok(results)
}

pub fn run_coarse_basin(identity: &CandidateIdentity, substeps: u64) -> Result<Vec<RunRecord>, Box<dyn std::error::Error>> {
    let root = d005_output_root().join("coarse_basin");
    fs::create_dir_all(&root)?;
    let cfg = SimRunConfig {
        substeps,
        ..Default::default()
    };
    let mut records = Vec::new();
    for r0 in coarse_grid_r0() {
        for c0 in coarse_grid_c0() {
            let slug = format!("R{}_C{}", r0 as u32, (c0 * 1000.0) as u32);
            let out = root.join(&slug);
            let result_path = out.join("result.json");
            if result_path.exists() {
                let existing: RunRecord = serde_json::from_str(&fs::read_to_string(&result_path)?)?;
                records.push(existing);
                continue;
            }
            let recipe = FreshSeedRecipe {
                r0,
                c0,
                noise_seed: 1,
                noise_amplitude: 0.005,
            };
            let record = run_fresh_seed(identity, &recipe, &cfg);
            fs::create_dir_all(&out)?;
            fs::write(out.join("result.json"), serde_json::to_string_pretty(&record)?)?;
            records.push(record);
        }
    }
    Ok(records)
}

pub fn build_macrostate_flow(records: &[RunRecord]) -> serde_json::Value {
    let points: Vec<FlowGridPoint> = records
        .iter()
        .filter_map(|r| {
            let r0 = r.r0?;
            let c0 = r.c0?;
            macrostate_velocity_from_trajectory(&r.trajectory, 0.25).map(|velocity| FlowGridPoint {
                r0,
                c0,
                velocity,
            })
        })
        .collect();
    serde_json::json!({
        "point_count": points.len(),
        "points": points,
    })
}

pub fn run_nullcline_analysis(records: &[RunRecord]) -> serde_json::Value {
    let points: Vec<FlowGridPoint> = records
        .iter()
        .filter_map(|r| {
            let r0 = r.r0?;
            let c0 = r.c0?;
            macrostate_velocity_from_trajectory(&r.trajectory, 0.25).map(|velocity| FlowGridPoint {
                r0,
                c0,
                velocity,
            })
        })
        .collect();
    let intersections = find_nullcline_intersections(&points);
    serde_json::to_value(&intersections).unwrap_or(serde_json::json!([]))
}

pub fn select_chemistry(
    continuations: &[RunRecord],
    coarse_by_kphi: &HashMap<String, Vec<RunRecord>>,
) -> Option<CandidateIdentity> {
    let identities = load_d004_identities().ok()?;
    let mut best: Option<(CandidateIdentity, i32, f64)> = None;

    for id in identities {
        let k_slug = id.k_phi.to_string().replace('.', "_");
        let cont_pass = continuations
            .iter()
            .filter(|r| r.candidate_hash == id.candidate_hash)
            .filter(|r| r.consecutive_stable_windows >= 3)
            .count();
        let coarse = coarse_by_kphi.get(&k_slug).map(|v| v.as_slice()).unwrap_or(&[]);
        let near_balance = coarse
            .iter()
            .filter(|r| r.basin_outcome.as_deref() == Some("near_balance"))
            .count();
        let drift = continuations
            .iter()
            .filter(|r| r.candidate_hash == id.candidate_hash)
            .map(|r| r.slope_phi.abs())
            .sum::<f64>();
        let score = (cont_pass as i32 * 10 + near_balance as i32) as i32;
        let tie = -drift;
        if best.as_ref().map_or(true, |(_, s, d)| score > *s || (score == *s && tie > *d)) {
            best = Some((id, score, tie));
        }
    }
    best.map(|(id, _, _)| id)
}

pub fn decide_d005_conclusion(
    continuations: &[RunRecord],
    coarse: &[RunRecord],
    intersections: &[NullclineIntersection],
    full_pass_count: u32,
) -> &'static str {
    let any_stable = continuations.iter().any(|r| r.consecutive_stable_windows >= 3);
    let near_count = coarse
        .iter()
        .filter(|r| r.basin_outcome.as_deref() == Some("near_balance"))
        .count();
    let all_decline = coarse.iter().all(|r| {
        matches!(
            r.basin_outcome.as_deref(),
            Some("slow_decline") | Some("rapid_collapse")
        )
    });
    let stable_fp = intersections.iter().any(|i| i.classification == FixedPointClass::Stable);

    if full_pass_count >= 4 {
        return "D005_ACCESSIBLE_ACTIVE_ATTRACTOR_PASS";
    }
    if any_stable && near_count >= 3 && basin_requires_neighboring_points(&pass_grid_from_coarse(coarse)) {
        return "D005_ACTIVE_ATTRACTOR_FOUND_BUT_BASIN_NARROW";
    }
    if any_stable && !stable_fp {
        return "D005_UNSTABLE_FIXED_POINT";
    }
    if all_decline && !stable_fp {
        return "D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR";
    }
    if continuations.iter().any(|r| r.classification == "CONTINUED_DRIFT") && !any_stable {
        return "D005_LONG_TRANSIENT_UNRESOLVED";
    }
    "D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR"
}

fn pass_grid_from_coarse(coarse: &[RunRecord]) -> Vec<Vec<bool>> {
    let rs = coarse_grid_r0();
    let cs = coarse_grid_c0();
    let mut grid = vec![vec![false; cs.len()]; rs.len()];
    for r in coarse {
        if let (Some(r0), Some(out)) = (r.r0, &r.basin_outcome) {
            if out != "near_balance" {
                continue;
            }
            for (i, rv) in rs.iter().enumerate() {
                for (j, cv) in cs.iter().enumerate() {
                    if (rv - r0).abs() < 0.1 {
                        if let Some(c0) = r.c0 {
                            if (cv - c0).abs() < 0.01 {
                                grid[i][j] = true;
                            }
                        }
                    }
                }
            }
        }
    }
    grid
}

pub fn run_controls(identity: &CandidateIdentity, recipe: &FreshSeedRecipe) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let root = d005_output_root().join("controls");
    fs::create_dir_all(&root)?;
    let cfg = SimRunConfig {
        substeps: 20_000,
        ..Default::default()
    };
    let mut results = Vec::new();

    for (name, params) in control_param_sets(identity, recipe) {
        let mut sim = Simulation::new(params);
        sim.observer_enabled = true;
        let (_, _, _) = advance_simulation(&mut sim, &cfg);
        let diag = sim.current_diagnostics();
        results.push(serde_json::json!({
            "control": name,
            "classification": format!("{:?}", sim.detector.last_classification),
            "final_m_phi": diag.structural_mass,
            "final_m_c": diag.catalyst_mass,
        }));
    }
    let out = serde_json::json!({ "controls": results });
    fs::write(root.join("controls.json"), serde_json::to_string_pretty(&out)?)?;
    Ok(out)
}

fn control_param_sets(identity: &CandidateIdentity, recipe: &FreshSeedRecipe) -> Vec<(&'static str, SimParams)> {
    let base = |mut p: SimParams| {
        recipe.apply_to_params(&mut p);
        p
    };
    vec![
        ("A_no_catalyst", {
            let mut p = base(identity.params.clone());
            p.seed_catalyst_scale = 0.0;
            p
        }),
        ("B_no_nutrient", {
            let mut p = base(identity.params.clone());
            p.n_reservoir = 0.0;
            p
        }),
        ("C_no_fuel", {
            let mut p = base(identity.params.clone());
            p.f_reservoir = 0.0;
            p
        }),
        ("D_no_structure", {
            let mut p = base(identity.params.clone());
            p.k_structure = 0.0;
            p
        }),
        ("E_no_rep", {
            let mut p = base(identity.params.clone());
            p.k_rep = 0.0;
            p
        }),
    ]
}

pub fn run_full_d005(continuation_target: u64, coarse_steps: u64) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let root = d005_output_root();
    fs::create_dir_all(&root)?;

    let aggregate = aggregate_d004_cross_state()?;
    let continuations = run_all_continuations(continuation_target)?;

    let identities = load_d004_identities()?;
    let mut coarse_by_kphi: HashMap<String, Vec<RunRecord>> = HashMap::new();
    let mut all_coarse = Vec::new();
    for id in &identities {
        let records = run_coarse_basin(id, coarse_steps)?;
        let k_slug = id.k_phi.to_string().replace('.', "_");
        coarse_by_kphi.insert(k_slug, records.clone());
        all_coarse.extend(records);
    }

    let selected = select_chemistry(&continuations, &coarse_by_kphi);
    let flow = if let Some(ref id) = selected {
        let k_slug = id.k_phi.to_string().replace('.', "_");
        build_macrostate_flow(coarse_by_kphi.get(&k_slug).map(|v| v.as_slice()).unwrap_or(&[]))
    } else {
        build_macrostate_flow(&all_coarse)
    };
    fs::create_dir_all(root.join("macrostate_flow"))?;
    fs::write(
        root.join("macrostate_flow/flow.json"),
        serde_json::to_string_pretty(&flow)?,
    )?;

    let intersections: Vec<NullclineIntersection> = if let Some(ref id) = selected {
        let k_slug = id.k_phi.to_string().replace('.', "_");
        let recs = coarse_by_kphi.get(&k_slug).map(|v| v.as_slice()).unwrap_or(&[]);
        let v = run_nullcline_analysis(recs);
        serde_json::from_value(v)?
    } else {
        vec![]
    };
    fs::create_dir_all(root.join("nullclines"))?;
    fs::write(
        root.join("nullclines/intersections.json"),
        serde_json::to_string_pretty(&intersections)?,
    )?;

    let recipe = FreshSeedRecipe::default_production();
    if let Some(ref id) = selected {
        let _ = run_controls(id, &recipe);
    }

    let full_pass = 0u32;
    let conclusion = decide_d005_conclusion(&continuations, &all_coarse, &intersections, full_pass);

    let manifest = serde_json::json!({
        "directive": "D-005",
        "conclusion": conclusion,
        "d003_revised": if conclusion == "D005_ACCESSIBLE_ACTIVE_ATTRACTOR_PASS" {
            "D003_ACTIVE_STEADY_STATE_PASS"
        } else {
            "D003_RESULT_UNRESOLVED_PENDING_ACCESSIBLE_ATTRACTOR_SEARCH"
        },
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "d004_conclusion": "D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT",
        "commit_hash": git_commit_hash(),
        "dirty_working_tree": git_dirty(),
        "selected_chemistry_k_phi": selected.as_ref().map(|i| i.k_phi),
        "continuation_count": continuations.len(),
        "coarse_point_count": all_coarse.len(),
        "nullcline_intersections": intersections.len(),
    });
    fs::write(root.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
    Ok(manifest)
}

/// Load completed continuation/coarse artifacts and write flow, nullclines, final manifest.
pub fn finalize_from_artifacts() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let root = d005_output_root();
    let mut continuations = Vec::new();
    let cont_root = root.join("continuations");
    if cont_root.exists() {
        for entry in fs::read_dir(&cont_root)? {
            let kphi_dir = entry?.path();
            if !kphi_dir.is_dir() {
                continue;
            }
            for seed_entry in fs::read_dir(&kphi_dir)? {
                let seed_dir = seed_entry?.path();
                let result = seed_dir.join("result.json");
                if result.exists() {
                    continuations.push(serde_json::from_str::<RunRecord>(&fs::read_to_string(
                        result,
                    )?)?);
                }
            }
        }
    }

    let mut all_coarse = Vec::new();
    let coarse_root = root.join("coarse_basin");
    if coarse_root.exists() {
        for entry in fs::read_dir(&coarse_root)? {
            let dir = entry?.path();
            let result = dir.join("result.json");
            if result.exists() {
                all_coarse.push(serde_json::from_str::<RunRecord>(&fs::read_to_string(result)?)?);
            }
        }
    }

    let mut coarse_by_kphi: HashMap<String, Vec<RunRecord>> = HashMap::new();
    for r in &all_coarse {
        // candidate_id embeds kphi token e.g. kphi1
        let key = if r.candidate_id.contains("kphi1") {
            "1".into()
        } else if r.candidate_id.contains("kphi2") {
            "2".into()
        } else if r.candidate_id.contains("kphi0_5") || r.candidate_id.contains("kphi0.5") {
            "0_5".into()
        } else {
            "unknown".into()
        };
        coarse_by_kphi.entry(key).or_default().push(r.clone());
    }

    let selected = select_chemistry(&continuations, &coarse_by_kphi);
    let flow = build_macrostate_flow(&all_coarse);
    fs::create_dir_all(root.join("macrostate_flow"))?;
    fs::write(
        root.join("macrostate_flow/flow.json"),
        serde_json::to_string_pretty(&flow)?,
    )?;

    let intersections: Vec<NullclineIntersection> = {
        let v = run_nullcline_analysis(&all_coarse);
        serde_json::from_value(v)?
    };
    fs::create_dir_all(root.join("nullclines"))?;
    fs::write(
        root.join("nullclines/intersections.json"),
        serde_json::to_string_pretty(&intersections)?,
    )?;

    let full_pass = 0u32;
    let conclusion = decide_d005_conclusion(&continuations, &all_coarse, &intersections, full_pass);

    let incomplete: Vec<String> = {
        let mut missing = Vec::new();
        for k in ["0_5", "1", "2"] {
            for seed in 1..=3u64 {
                let p = cont_root.join(format!("kphi_{k}/fresh_seed_{seed}/result.json"));
                if !p.exists() {
                    missing.push(format!("kphi_{k}/fresh_seed_{seed}"));
                }
            }
        }
        missing
    };

    let selected_k = selected.as_ref().map(|i| i.k_phi);
    let stable_fp_count = intersections
        .iter()
        .filter(|i| i.classification == FixedPointClass::Stable)
        .count();

    let manifest = serde_json::json!({
        "directive": "D-005",
        "conclusion": conclusion,
        "d003_revised": "D003_RESULT_UNRESOLVED_PENDING_ACCESSIBLE_ATTRACTOR_SEARCH",
        "d004_conclusion": "D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "commit_hash": git_commit_hash(),
        "dirty_working_tree": git_dirty(),
        "selected_chemistry_k_phi": selected_k,
        "continuation_count": continuations.len(),
        "continuation_incomplete": incomplete,
        "coarse_point_count": all_coarse.len(),
        "nullcline_intersections": intersections.len(),
        "stable_fixed_points": stable_fp_count,
        "finalized_from_artifacts": true,
    });
    fs::write(root.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
    Ok(manifest)
}
