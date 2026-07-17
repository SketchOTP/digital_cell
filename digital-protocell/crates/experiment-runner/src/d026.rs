//! D-026 Gates 0–6: parity, observability, reference history, causal controls, classification.

use crate::d013::{atomic_write_json, load_governed_checkpoint, restore_governed_simulation};
use crate::d025::{seed_v7_compartment, v7_base_params, D025_FROZEN_K_ADS};
use chemistry_core::config::EquationVersion;
use chemistry_core::d026_analysis::{
    classify_chronology_earliest, classify_mechanism_from_evidence, linear_slope,
    run_runner_parity, sample_stage_e_observability, settle_constrained, total_a_demand_from_sample,
    CausalControlMetrics, CausalControlsReport, D026_GATE5_DIAGNOSTIC_STEPS,
    D026_GATE5_SLOPE_WINDOW, D026_REFERENCE_CHECKPOINTS, D026_SETTLE_STEPS,
    ReferenceHistoryPoint, ReferenceHistoryReport,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const D026_GATE1_DEMO_STEPS: u64 = 200;
const D026_GATE1_SAMPLE_EVERY: u64 = 20;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
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

fn build_v7_stage_e_seed() -> Result<Simulation, Box<dyn std::error::Error>> {
    let params = v7_base_params()?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    settle_constrained(&mut sim, D026_SETTLE_STEPS);
    Ok(sim)
}

fn history_point_from_sim(sim: &Simulation, step: u64, source: &str) -> ReferenceHistoryPoint {
    let sample = sample_stage_e_observability(sim);
    let metab = &sim.metabolism_accounting.cumulative;
    let mem = &sim.membrane_accounting.cumulative;
    ReferenceHistoryPoint {
        checkpoint_step: step,
        source: source.to_string(),
        total_a_demand: total_a_demand_from_sample(&sample),
        q_activated: Some(metab.activation / metab.activated_decay.max(f64::EPSILON)),
        q_membrane: Some(mem.synthesis / mem.decay.max(f64::EPSILON)),
        sample,
    }
}

fn run_diagnostic_reference_points(max_steps: u64) -> Result<Vec<ReferenceHistoryPoint>, Box<dyn std::error::Error>> {
    let mut sim = build_v7_stage_e_seed()?;
    sim.enforce_structure_constraint = true;
    let sample_every = (max_steps / 6).max(100);
    let mut points = Vec::new();
    points.push(history_point_from_sim(&sim, sim.substep, "analytic_seed"));
    for _ in 0..max_steps {
        if !sim.step() {
            break;
        }
        if sim.substep % sample_every == 0 || sim.substep == max_steps {
            points.push(history_point_from_sim(
                &sim,
                sim.substep,
                "diagnostic_fallback",
            ));
        }
    }
    Ok(points)
}

pub fn run_gate0_parity(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let base = build_v7_stage_e_seed()?;
    let report = run_runner_parity(&base);
    let body = json!({
        "project_directive": "D-026",
        "gate": 0,
        "source_commit": git_commit_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "k_ads_frozen": D025_FROZEN_K_ADS,
        "settle_steps": D026_SETTLE_STEPS,
        "report": report,
        "gate0_pass": report.gate0_pass,
        "max_abs_diff": report.max_abs_diff,
        "max_abs_diff_metric": report.max_abs_diff_metric,
        "conclusion": if report.gate0_pass { "D026_GATE0_PARITY_PASS" } else { "D026_GATE0_PARITY_FAIL" },
    });
    atomic_write_json(&output.join("gate0_parity.json"), &body)?;
    Ok(body)
}

pub fn run_gate1_observability_demo(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let surface_dir = output.join("surface_coverage");
    let budget_dir = output.join("a_budget");
    fs::create_dir_all(&surface_dir)?;
    fs::create_dir_all(&budget_dir)?;

    let mut sim = build_v7_stage_e_seed()?;
    sim.enforce_structure_constraint = true;
    let mut surface_samples = Vec::new();
    let mut budget_samples = Vec::new();

    for step in 1..=D026_GATE1_DEMO_STEPS {
        if !sim.step() {
            break;
        }
        if step % D026_GATE1_SAMPLE_EVERY != 0 {
            continue;
        }
        let sample = sample_stage_e_observability(&sim);
        surface_samples.push(json!({
            "step": sample.step,
            "sim_time": sample.sim_time,
            "surface": sample.surface,
            "interface_measure": sample.interface_measure,
            "mass_s": sample.mass_s,
        }));
        budget_samples.push(json!({
            "step": sample.step,
            "sim_time": sample.sim_time,
            "mass_a": sample.mass_a,
            "a_production_activation": sample.a_production_activation,
            "a_consumption_catalyst_reproduction": sample.a_consumption_catalyst_reproduction,
            "a_consumption_precursor_production": sample.a_consumption_precursor_production,
            "a_consumption_virtual_structural": sample.a_consumption_virtual_structural,
            "a_consumption_decay": sample.a_consumption_decay,
            "a_retention": sample.a_retention,
            "outward_leakage_per_interface": sample.outward_leakage_per_interface,
            "activation_to_demand": sample.activation_to_demand,
            "activation_to_leakage": sample.activation_to_leakage,
        }));
    }

    atomic_write_json(
        &surface_dir.join("samples.json"),
        &json!({ "samples": surface_samples }),
    )?;
    atomic_write_json(
        &budget_dir.join("samples.json"),
        &json!({ "samples": budget_samples }),
    )?;

    let body = json!({
        "project_directive": "D-026",
        "gate": 1,
        "source_commit": git_commit_hash(),
        "equation_version": EquationVersion::MembraneMetabolismV7SurfaceDensity.as_str(),
        "demo_steps": D026_GATE1_DEMO_STEPS,
        "sample_every": D026_GATE1_SAMPLE_EVERY,
        "accepted_substeps": sim.substep,
        "surface_sample_count": surface_samples.len(),
        "budget_sample_count": budget_samples.len(),
        "gate1_pass": !surface_samples.is_empty() && !budget_samples.is_empty(),
        "conclusion": "D026_GATE1_OBSERVABILITY_READY",
    });
    atomic_write_json(&output.join("gate1_observability.json"), &body)?;
    Ok(body)
}

pub fn run_gate2_reference_history(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let hist_dir = output.join("reference_history");
    fs::create_dir_all(&hist_dir)?;

    let ckpt_root = resolve_path(Path::new(
        "experiments/generated/d025/stage_e_reference/checkpoints",
    ));
    let result_path = resolve_path(Path::new(
        "experiments/generated/d025/stage_e_reference/result.json",
    ));

    let mut points = Vec::new();
    let mut notes = Vec::new();
    let mut checkpoints_available = true;

    for &step in &D026_REFERENCE_CHECKPOINTS {
        let path = ckpt_root.join(format!("checkpoint_{step:06}.json"));
        if !path.is_file() {
            checkpoints_available = false;
            notes.push(format!("missing checkpoint: {}", path.display()));
            continue;
        }
        let ckpt = load_governed_checkpoint(&path)?;
        let mut sim = Simulation::new(v7_base_params()?);
        restore_governed_simulation(&mut sim, &ckpt)?;
        sim.enforce_structure_constraint = true;
        points.push(history_point_from_sim(&sim, step, "d025_checkpoint"));
    }

    let fallback_diagnostic = !checkpoints_available;
    if fallback_diagnostic {
        notes.push("D-025 checkpoints unavailable; using 3000-step diagnostic fallback".into());
        points = run_diagnostic_reference_points(3_000)?;
    }

    let rolling_window_slopes = if result_path.is_file() {
        let raw = fs::read_to_string(&result_path)?;
        let result: Value = serde_json::from_str(&raw)?;
        Some(json!({
            "rolling_windows": result.get("rolling_windows").cloned().unwrap_or(Value::Null),
            "metrics": result.get("metrics").cloned().unwrap_or(Value::Null),
        }))
    } else {
        notes.push(format!("missing result.json: {}", result_path.display()));
        None
    };

    let earliest = classify_chronology_earliest(&points);
    let report = ReferenceHistoryReport {
        checkpoints_available,
        fallback_diagnostic,
        earliest_divergence: earliest,
        points: points.clone(),
        rolling_window_slopes,
        notes: notes.clone(),
    };

    let chronology_json = json!({
        "project_directive": "D-026",
        "gate": 2,
        "source_commit": git_commit_hash(),
        "checkpoints_available": checkpoints_available,
        "fallback_diagnostic": fallback_diagnostic,
        "earliest_divergence": earliest.as_str(),
        "notes": notes,
        "points": points.iter().map(|p| json!({
            "checkpoint_step": p.checkpoint_step,
            "source": p.source,
            "a_retention": p.sample.a_retention,
            "mean_theta_gamma": p.sample.surface.mean_theta_gamma,
            "mass_s": p.sample.mass_s,
            "mass_a": p.sample.mass_a,
            "low_coverage_frac_0_50": p.sample.surface.fraction_below_0_50_gamma_ref,
            "a_demand_split": {
                "activation": p.sample.a_production_activation,
                "virtual_structural": p.sample.a_consumption_virtual_structural,
                "catalyst_reproduction": p.sample.a_consumption_catalyst_reproduction,
                "precursor_synthesis": p.sample.a_consumption_precursor_production,
                "decay": p.sample.a_consumption_decay,
            },
            "q_activated": p.q_activated,
            "q_membrane": p.q_membrane,
            "total_a_demand": p.total_a_demand,
        })).collect::<Vec<_>>(),
        "rolling_window_slopes": report.rolling_window_slopes,
        "conclusion": "D026_GATE2_HISTORY_READY",
    });
    atomic_write_json(&hist_dir.join("chronology.json"), &chronology_json)?;
    Ok(chronology_json)
}

fn run_causal_horizon(sim: &mut Simulation, steps: u64) -> CausalControlMetrics {
    let mut a_series = Vec::new();
    let mut s_series = Vec::new();
    let mut c_series = Vec::new();
    let mut p_series = Vec::new();
    let mut activation_sum = 0.0;
    let mut demand_sum = 0.0;
    let mut accepted = 0u64;
    for _ in 0..steps {
        if !sim.step() {
            break;
        }
        accepted += 1;
        let sample = sample_stage_e_observability(sim);
        a_series.push(sample.mass_a);
        s_series.push(sample.mass_s);
        c_series.push(sample.mass_c);
        p_series.push(sample.mass_p);
        activation_sum += sample.a_production_activation;
        demand_sum += total_a_demand_from_sample(&sample);
    }
    let window = D026_GATE5_SLOPE_WINDOW.min(a_series.len().max(1) as u64) as usize;
    let end_sample = sample_stage_e_observability(sim);
    CausalControlMetrics {
        label: String::new(),
        accepted_steps: accepted,
        a_slope: linear_slope(&a_series[a_series.len().saturating_sub(window)..]),
        a_retention_end: end_sample.a_retention,
        a_leakage_end: end_sample.outward_leakage_per_interface,
        activation_mean: activation_sum / accepted.max(1) as f64,
        total_a_demand_mean: demand_sum / accepted.max(1) as f64,
        theta_gamma_end: end_sample.surface.mean_theta_gamma,
        mass_s_slope: linear_slope(&s_series[s_series.len().saturating_sub(window)..]),
        mass_c_slope: linear_slope(&c_series[c_series.len().saturating_sub(window)..]),
        mass_p_slope: linear_slope(&p_series[p_series.len().saturating_sub(window)..]),
    }
}

fn seed_for_causal_controls() -> Result<Simulation, Box<dyn std::error::Error>> {
    let ckpt_path = resolve_path(Path::new(
        "experiments/generated/d025/stage_e_reference/checkpoints/checkpoint_100000.json",
    ));
    if ckpt_path.is_file() {
        let ckpt = load_governed_checkpoint(&ckpt_path)?;
        let mut sim = Simulation::new(v7_base_params()?);
        restore_governed_simulation(&mut sim, &ckpt)?;
        sim.enforce_structure_constraint = true;
        return Ok(sim);
    }
    build_v7_stage_e_seed()
}

pub fn run_gate5_causal_controls(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let ctrl_dir = output.join("causal_controls");
    fs::create_dir_all(&ctrl_dir)?;

    let horizon = D026_GATE5_DIAGNOSTIC_STEPS;
    let mut notes = vec![
        "diagnostic_only: not biological acceptance".into(),
        format!("horizon_steps={horizon}"),
    ];

    let mut baseline_sim = seed_for_causal_controls()?;
    baseline_sim.d026_disable_a_normal_transport = false;
    baseline_sim.d026_freeze_surface = false;
    baseline_sim.d026_disable_virtual_structure = false;
    baseline_sim.d026_disable_catalyst_reproduction = false;
    baseline_sim.d026_disable_precursor_synthesis = false;
    let baseline = {
        let mut m = run_causal_horizon(&mut baseline_sim, horizon);
        m.label = "baseline".into();
        m
    };
    atomic_write_json(&ctrl_dir.join("baseline.json"), &json!(baseline))?;

    let controls_spec: [(&str, fn(&mut Simulation)); 5] = [
        (
            "control_a_no_a_transport",
            |s| {
                s.d026_disable_a_normal_transport = true;
            },
        ),
        (
            "control_b_freeze_surface",
            |s| {
                s.d026_freeze_surface = true;
            },
        ),
        (
            "control_c_no_virtual_structure",
            |s| {
                s.d026_disable_virtual_structure = true;
            },
        ),
        (
            "control_d_no_catalyst_reproduction",
            |s| {
                s.d026_disable_catalyst_reproduction = true;
            },
        ),
        (
            "control_e_no_precursor_synthesis",
            |s| {
                s.d026_freeze_surface = true;
                s.d026_disable_precursor_synthesis = true;
            },
        ),
    ];

    let mut controls = Vec::new();
    for (name, apply) in controls_spec {
        let mut sim = seed_for_causal_controls()?;
        apply(&mut sim);
        let mut metrics = run_causal_horizon(&mut sim, horizon);
        metrics.label = name.to_string();
        atomic_write_json(&ctrl_dir.join(format!("{name}.json")), &json!(metrics))?;
        controls.push(metrics);
    }

    let report = CausalControlsReport {
        diagnostic_only: true,
        horizon_steps: horizon,
        baseline: baseline.clone(),
        controls: controls.clone(),
        notes: notes.clone(),
    };

    let summary = json!({
        "project_directive": "D-026",
        "gate": 5,
        "diagnostic_only": true,
        "horizon_steps": horizon,
        "baseline": baseline,
        "controls": controls.iter().map(|c| json!({
            "label": c.label,
            "delta_a_retention_vs_baseline": c.a_retention_end - baseline.a_retention_end,
            "delta_theta_vs_baseline": c.theta_gamma_end - baseline.theta_gamma_end,
            "delta_a_leakage_vs_baseline": c.a_leakage_end - baseline.a_leakage_end,
            "delta_a_slope_vs_baseline": c.a_slope - baseline.a_slope,
            "delta_a_demand_vs_baseline": c.total_a_demand_mean - baseline.total_a_demand_mean,
            "mass_s_slope": c.mass_s_slope,
        })).collect::<Vec<_>>(),
        "notes": notes,
        "conclusion": "D026_GATE5_CONTROLS_READY",
    });
    atomic_write_json(&ctrl_dir.join("summary.json"), &summary)?;
    Ok(summary)
}

pub fn run_gate6_classification(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    let hist_path = output.join("reference_history/chronology.json");
    let ctrl_path = output.join("causal_controls/summary.json");

    let gate2_body = if hist_path.is_file() {
        serde_json::from_str::<Value>(&fs::read_to_string(&hist_path)?)?
    } else {
        run_gate2_reference_history(&output)?
    };

    let gate5_body = if ctrl_path.is_file() {
        serde_json::from_str::<Value>(&fs::read_to_string(&ctrl_path)?)?
    } else {
        run_gate5_causal_controls(&output)?
    };

    let history = ReferenceHistoryReport {
        checkpoints_available: gate2_body["checkpoints_available"].as_bool().unwrap_or(false),
        fallback_diagnostic: gate2_body["fallback_diagnostic"].as_bool().unwrap_or(true),
        earliest_divergence: parse_chronology_label(
            gate2_body["earliest_divergence"].as_str().unwrap_or("UNKNOWN"),
        ),
        points: Vec::new(),
        rolling_window_slopes: gate2_body.get("rolling_window_slopes").cloned(),
        notes: gate2_body["notes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
    };

    let baseline: CausalControlMetrics = serde_json::from_value(gate5_body["baseline"].clone())?;
    let controls: CausalControlsReport = CausalControlsReport {
        diagnostic_only: true,
        horizon_steps: gate5_body["horizon_steps"].as_u64().unwrap_or(D026_GATE5_DIAGNOSTIC_STEPS),
        baseline: baseline.clone(),
        controls: gate5_body["controls"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| {
                        Some(CausalControlMetrics {
                            label: x["label"].as_str()?.to_string(),
                            accepted_steps: 0,
                            a_slope: baseline.a_slope + x["delta_a_slope_vs_baseline"].as_f64()?,
                            a_retention_end: baseline.a_retention_end
                                + x["delta_a_retention_vs_baseline"].as_f64()?,
                            a_leakage_end: baseline.a_leakage_end
                                + x["delta_a_leakage_vs_baseline"].as_f64()?,
                            activation_mean: 0.0,
                            total_a_demand_mean: baseline.total_a_demand_mean
                                + x["delta_a_demand_vs_baseline"].as_f64()?,
                            theta_gamma_end: baseline.theta_gamma_end
                                + x["delta_theta_vs_baseline"].as_f64()?,
                            mass_s_slope: x["mass_s_slope"].as_f64().unwrap_or(0.0),
                            mass_c_slope: 0.0,
                            mass_p_slope: 0.0,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        notes: gate5_body["notes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
    };

    let classification = classify_mechanism_from_evidence(&history, &controls);

    let class_dir = output.join("late_time_classification");
    let budget_dir = output.join("a_budget");
    fs::create_dir_all(&class_dir)?;
    fs::create_dir_all(&budget_dir)?;

    let class_json = json!({
        "project_directive": "D-026",
        "gate": 6,
        "gate6_mechanism": classification.gate6_mechanism.as_str(),
        "chronology": classification.chronology.as_str(),
        "evidence": classification.evidence,
        "gate7_continuation_warranted": classification.gate7_continuation_warranted,
        "gate8_rate_correction_warranted": classification.gate8_rate_correction_warranted,
        "suggested_rate": classification.suggested_rate,
    });
    atomic_write_json(&class_dir.join("classification.json"), &class_json)?;
    atomic_write_json(
        &budget_dir.join("mechanism.json"),
        &json!({
            "mechanism": classification.gate6_mechanism.as_str(),
            "evidence": classification.evidence,
            "suggested_rate": classification.suggested_rate,
        }),
    )?;
    Ok(class_json)
}

fn parse_chronology_label(label: &str) -> chemistry_core::d026_analysis::D026ChronologyLabel {
    use chemistry_core::d026_analysis::D026ChronologyLabel as L;
    match label {
        "SURFACE_COVERAGE_DECLINE" => L::SurfaceCoverageDecline,
        "ACTIVATION_CAPACITY_DECLINE" => L::ActivationCapacityDecline,
        "STRUCTURAL_DEMAND_EXCESS" => L::StructuralDemandExcess,
        "CATALYST_DEMAND_EXCESS" => L::CatalystDemandExcess,
        "PRECURSOR_DEMAND_EXCESS" => L::PrecursorDemandExcess,
        "A_LEAKAGE_INCREASE" => L::ALeakageIncrease,
        "INITIAL_STATE_DIVERGENCE" => L::InitialStateDivergence,
        "OSCILLATORY_ONSET" => L::OscillatoryOnset,
        "MONOTONIC_SLOW_DRIFT" => L::MonotonicSlowDrift,
        _ => L::Unknown,
    }
}
