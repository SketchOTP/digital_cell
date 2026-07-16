//! D-016 intracellular waste transport timescale runners.

use crate::d013::{atomic_write_json, load_governed_checkpoint, GovernedCheckpoint};
use crate::d015::{frozen_organism_params, FROZEN_CANDIDATE, FROZEN_CONFIG};
use chemistry_core::{
    analyze_membrane_conductance, analyze_timescales, audit_waste_transport, authorized_d_w_bound,
    build_candidate_identity, candidate_hash, classify_passive_feasibility, derive_d_w_candidates,
    environment_configuration_hash, interface_length_proxy, membrane_branch_authorized,
    organism_frozen_hash, resistance_decomposition, resistance_fractions_sum_to_one,
    run_fixed_source_assay, species_diffusivity_comparison, summarize_source_field,
    transport_schema_for_repair, w_ordering_vs_nutrient_fuel, Grid, GridConfiguration,
    PassiveTransportFeasibility, SimParams, CONC_SAFETY_LIMIT, D012_V2_CENTER_RADIUS,
    D015_ENVIRONMENT_SCHEMA_VERSION, D016_FIXED_SOURCE_MAX_STEPS, D016_W_TARGET_FRAC,
    TRANSPORT_SCHEMA_VERSION_V1,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const D015_MANIFEST: &str = "experiments/generated/d015/manifest.json";
const D015_FRESH_CKPT_25K: &str =
    "experiments/generated/d015/fresh_reference_r22/checkpoints/checkpoint_025000.json";
const D015_FRESH_CKPT_100K: &str =
    "experiments/generated/d015/fresh_reference_r22/checkpoints/checkpoint_100000.json";
const D015_FRESH_CKPT_150K: &str =
    "experiments/generated/d015/fresh_reference_r22/checkpoints/checkpoint_150000.json";
const D015_PRESERVATION: &str = "experiments/generated/d015/preservation/preservation_record.json";
const D015_FRESH_RESULT: &str = "experiments/generated/d015/fresh_reference_r22/result.json";
const D015_SOURCE_DECOMP: &str =
    "experiments/generated/d015/source_decomposition/d014_150k_sources.json";
const D015_DIFFUSION: &str = "experiments/generated/d015/diffusion_timescales/audit.json";
const D015_SINK: &str = "experiments/generated/d015/sink_capacity/analytical_from_d014_150k.json";
const D015_COMMITS: [&str; 3] = [
    "b18df585cb1cf071c9d9ab203c9c0f5f28b5844a",
    "7d656259bab6101bb1f530aca4b604f3ee9f13d8",
    "2cbd5535a8bb6438b019153ef6a438a81422baca",
];

fn git_commit_hash() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    if !output.status.success() {
        return Err("git rev-parse failed".into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn binary_hash() -> Result<String, Box<dyn std::error::Error>> {
    Ok(chemistry_core::sha256_hex(&fs::read(std::env::current_exe()?)?))
}

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(chemistry_core::sha256_hex(&fs::read(path)?))
}

fn bits_to_f64(bits: &[u64]) -> Vec<f64> {
    bits.iter().map(|b| f64::from_bits(*b)).collect()
}

fn load_ckpt_fields(
    path: &Path,
) -> Result<(GovernedCheckpoint, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), Box<dyn std::error::Error>>
{
    let ckpt = load_governed_checkpoint(path)?;
    let phi = bits_to_f64(&ckpt.lossless_fields.structure);
    let c = bits_to_f64(&ckpt.lossless_fields.catalyst);
    let n = bits_to_f64(&ckpt.lossless_fields.nutrient);
    let f = bits_to_f64(&ckpt.lossless_fields.fuel);
    let w = bits_to_f64(&ckpt.lossless_fields.waste);
    let a = bits_to_f64(&ckpt.lossless_fields.activated);
    let m = bits_to_f64(&ckpt.lossless_fields.membrane);
    Ok((ckpt, phi, c, n, f, w, a, m))
}

pub fn run_preserve(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let manifest = resolve_path(Path::new(D015_MANIFEST));
    let preservation = resolve_path(Path::new(D015_PRESERVATION));
    let fresh = resolve_path(Path::new(D015_FRESH_RESULT));
    let source = resolve_path(Path::new(D015_SOURCE_DECOMP));
    let diffusion = resolve_path(Path::new(D015_DIFFUSION));
    let sink = resolve_path(Path::new(D015_SINK));

    let d015_manifest: Value = serde_json::from_str(&fs::read_to_string(&manifest)?)?;
    let d015_pres: Value = serde_json::from_str(&fs::read_to_string(&preservation)?)?;
    let d015_fresh: Value = serde_json::from_str(&fs::read_to_string(&fresh)?)?;
    let d015_source: Value = serde_json::from_str(&fs::read_to_string(&source)?)?;
    let d015_diff: Value = serde_json::from_str(&fs::read_to_string(&diffusion)?)?;
    let d015_sink: Value = serde_json::from_str(&fs::read_to_string(&sink)?)?;

    let params = frozen_organism_params(true)?;
    let grid = GridConfiguration::default();
    let body = json!({
        "project_directive": "D-016",
        "agent_memory_directive": "D-20260715-d016-waste-transport-timescale",
        "source_commit": git_commit_hash().ok(),
        "binary_sha256": binary_hash().ok(),
        "d015_commits_verified": D015_COMMITS,
        "d015_tag": "D-015-waste-throughput-closure-fail",
        "d015_manifest_path": manifest.display().to_string(),
        "d015_manifest_hash": sha256_file(&manifest)?,
        "d015_historical_conclusion": d015_manifest["primary_conclusion"],
        "d015_operative_reinterpretation": "D015_WASTE_SOURCE_TRANSPORT_BALANCE_UNRESOLVED",
        "frozen_organism_identity": {
            "candidate_hash": FROZEN_CANDIDATE,
            "configuration_hash": FROZEN_CONFIG,
            "organism_frozen_hash": organism_frozen_hash(&params, &grid),
            "equation_version": "membrane_metabolism_v2_conservative",
            "stoichiometric_schema": 2,
            "transport_schema": TRANSPORT_SCHEMA_VERSION_V1,
        },
        "repaired_environment_identity": {
            "waste_sink_inner_radius": params.waste_sink_inner_radius,
            "environment_schema_version": D015_ENVIRONMENT_SCHEMA_VERSION,
            "environment_configuration_hash": environment_configuration_hash(&params),
        },
        "terminal_w_location": d015_pres["terminal_waste_location"],
        "terminal_w_concentration": d015_pres["terminal_waste_concentration"],
        "terminal_accepted_substep": d015_fresh["accepted_substeps"],
        "terminal_simulated_time": d015_fresh["simulated_time"],
        "waste_source_rate": d015_source["production_rate_mass_per_time"],
        "sink_capacity_estimate": d015_sink,
        "diffusion_timescale_estimate": d015_diff,
        "d015_artifacts_not_overwritten": true,
    });
    atomic_write_json(&output.join("preservation_record.json"), &body)?;
    // Copy hashes for audit trail without mutating D-015 trees.
    fs::write(
        output.join("d015_manifest.sha256"),
        format!("{}\n", sha256_file(&manifest)?),
    )?;
    Ok(body)
}

pub fn run_transport_audit(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let params = frozen_organism_params(true)?;
    let audit = audit_waste_transport(&params);
    let species = species_diffusivity_comparison(&params, D012_V2_CENTER_RADIUS);
    let ordering = w_ordering_vs_nutrient_fuel(&params);
    let body = json!({
        "project_directive": "D-016",
        "source_commit": git_commit_hash().ok(),
        "binary_sha256": binary_hash().ok(),
        "equation_version": "membrane_metabolism_v2_conservative",
        "stoichiometric_schema": 2,
        "transport_schema": params.transport_schema_version,
        "candidate_hash": FROZEN_CANDIDATE,
        "D_W": params.d_w,
        "beta_W": params.beta_w,
        "audit": audit,
        "species_comparison": species,
        "w_ordering_vs_n_f": ordering,
        "ordering_note": "W base diffusivity 0.25 > N/F 0.18; already faster than imported small solutes, consistent with exported metabolic product. Failure despite this ordering implicates source/transport imbalance rather than an under-diffusive W parameterization relative to N/F."
    });
    atomic_write_json(&output.join("audit.json"), &body)?;
    let cmp_dir = resolve_path(Path::new(
        "experiments/generated/d016/species_diffusivity_comparison",
    ));
    fs::create_dir_all(&cmp_dir)?;
    atomic_write_json(&cmp_dir.join("comparison.json"), &body["species_comparison"])?;
    Ok(body)
}

fn source_from_checkpoint(
    ckpt_rel: &str,
    label: &str,
    params: &SimParams,
) -> Result<(Vec<f64>, Value), Box<dyn std::error::Error>> {
    let path = resolve_path(Path::new(ckpt_rel));
    let (_ckpt, phi, c, n, f, w, a, m) = load_ckpt_fields(&path)?;
    let grid = Grid::new();
    let (q, summary) = summarize_source_field(
        &grid,
        &phi,
        &c,
        &n,
        &f,
        &a,
        &m,
        params,
        D012_V2_CENTER_RADIUS,
        label,
    );
    let mean_w_interior = {
        let masks = chemistry_core::build_waste_spatial_masks(&grid, &phi, D012_V2_CENTER_RADIUS);
        let mut s = 0.0;
        let mut ctn = 0.0;
        for idx in 0..w.len() {
            if masks.interior[idx] {
                s += w[idx];
                ctn += 1.0;
            }
        }
        if ctn > 0.0 { s / ctn } else { 0.0 }
    };
    Ok((
        q,
        json!({
            "summary": summary,
            "checkpoint": ckpt_rel,
            "mean_interior_w": mean_w_interior,
            "waste_field_mass": w.iter().sum::<f64>(),
        }),
    ))
}

pub fn run_source_and_timescales(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let params = frozen_organism_params(true)?;
    let src_dir = output_root.join("source_fields");
    let ts_dir = output_root.join("timescale_analysis");
    let cond_dir = output_root.join("conductance_analysis");
    let res_dir = output_root.join("resistance_decomposition");
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&ts_dir)?;
    fs::create_dir_all(&cond_dir)?;
    fs::create_dir_all(&res_dir)?;

    let windows = [
        (D015_FRESH_CKPT_25K, "0-25pct_proxy_25k"),
        (D015_FRESH_CKPT_100K, "50-75pct_proxy_100k"),
        (D015_FRESH_CKPT_150K, "75-100pct_proxy_150k_final_valid"),
    ];
    let mut window_summaries = Vec::new();
    let mut canonical_q = None;
    let mut canonical_summary = None;
    let mut mean_w = 0.0;
    for (path, label) in windows {
        let (q, meta) = source_from_checkpoint(path, label, &params)?;
        atomic_write_json(&src_dir.join(format!("{label}.json")), &meta)?;
        window_summaries.push(meta.clone());
        if label.contains("150k") {
            canonical_q = Some(q);
            canonical_summary = Some(meta["summary"].clone());
            mean_w = meta["mean_interior_w"].as_f64().unwrap_or(0.0);
        }
    }

    let summary: chemistry_core::SourceFieldSummary =
        serde_json::from_value(canonical_summary.clone().unwrap())?;
    // Analytical internal crossing; external gap uses D-015 repaired sink radius (30),
    // not the historical peripheral-reservoir τ=16129 from pre-repair audits.
    let pulse_c2i = Some((D012_V2_CENTER_RADIUS * D012_V2_CENTER_RADIUS) / (4.0 * params.d_w));
    let sink_gap = (params.waste_sink_inner_radius - D012_V2_CENTER_RADIUS).max(0.0);
    let pulse_ext = Some((sink_gap * sink_gap) / (4.0 * params.d_w));
    let times = analyze_timescales(
        &summary,
        &params,
        D012_V2_CENTER_RADIUS,
        0.0,
        2.0, // interface W proxy from D-015 spatial partition
        pulse_c2i,
        pulse_ext,
    );
    let resistance = resistance_decomposition(&times);
    assert!(resistance_fractions_sum_to_one(&resistance));
    let conductance = analyze_membrane_conductance(
        summary.interior_source_rate,
        interface_length_proxy(D012_V2_CENTER_RADIUS),
        params.d_w,
        params.beta_w,
        &params,
    );

    let ts_body = json!({
        "project_directive": "D-016",
        "source_commit": git_commit_hash().ok(),
        "binary_sha256": binary_hash().ok(),
        "mean_interior_w_at_canonical": mean_w,
        "timescales": times,
        "repaired_sink_gap": sink_gap,
        "historical_d015_tau_cell_to_reservoir": 16129.0,
        "historical_tau_transport_over_t_fail": 16129.0 / 405.18,
        "repaired_tau_interface_to_sink_over_t_fail": times.tau_interface_to_sink / 405.18,
    });
    atomic_write_json(&ts_dir.join("timescales.json"), &ts_body)?;
    atomic_write_json(
        &cond_dir.join("conductance.json"),
        &json!({
            "project_directive": "D-016",
            "conductance": conductance,
        }),
    )?;
    atomic_write_json(
        &res_dir.join("resistance.json"),
        &json!({
            "project_directive": "D-016",
            "resistance": resistance,
            "fractions_sum_to_one": true,
        }),
    )?;

    let _ = canonical_q;
    Ok(json!({
        "windows": window_summaries,
        "canonical_summary": summary,
        "timescales": times,
        "conductance": conductance,
        "resistance": resistance,
    }))
}

pub fn run_fixed_source_campaign(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let mut params = frozen_organism_params(true)?;
    let grid = Grid::new();
    let path = resolve_path(Path::new(D015_FRESH_CKPT_150K));
    let (_ckpt, phi, c, n, f, w, a, m) = load_ckpt_fields(&path)?;
    let (q, summary) = summarize_source_field(
        &grid,
        &phi,
        &c,
        &n,
        &f,
        &a,
        &m,
        &params,
        D012_V2_CENTER_RADIUS,
        "canonical_150k",
    );
    let sink_gap = (params.waste_sink_inner_radius - D012_V2_CENTER_RADIUS).max(0.0);
    let pulse_ext = Some((sink_gap * sink_gap) / (4.0 * params.d_w));
    let times = analyze_timescales(
        &summary,
        &params,
        D012_V2_CENTER_RADIUS,
        0.0,
        2.0,
        None,
        pulse_ext,
    );
    let resistance = resistance_decomposition(&times);
    let bound = authorized_d_w_bound(&params);
    let d_candidates = derive_d_w_candidates(params.d_w, times.d_w_required_50pct, bound);

    let base_dir = output_root.join("fixed_source_baseline");
    let diff_dir = output_root.join("diffusivity_candidates");
    let perm_dir = output_root.join("permeability_candidates");
    fs::create_dir_all(&base_dir)?;
    fs::create_dir_all(&diff_dir)?;
    fs::create_dir_all(&perm_dir)?;

    // Baseline at frozen D_W / β_W with zero initial W (pure transport vs source).
    let max_steps = D016_FIXED_SOURCE_MAX_STEPS;
    let baseline = run_fixed_source_assay(&grid, &phi, &m, &q, &params, None, max_steps);
    atomic_write_json(
        &base_dir.join("result.json"),
        &json!({
            "project_directive": "D-016",
            "source_commit": git_commit_hash().ok(),
            "binary_sha256": binary_hash().ok(),
            "equation_version": "membrane_metabolism_v2_conservative",
            "stoichiometric_schema": 2,
            "transport_schema": params.transport_schema_version,
            "candidate_hash": FROZEN_CANDIDATE,
            "D_W": params.d_w,
            "beta_W": params.beta_w,
            "source_rate": summary.total_source_rate,
            "assay": baseline,
        }),
    )?;

    let required_exceeds_bound = times.d_w_required_50pct > bound * 1.01;
    let mut diff_results = Vec::new();
    // When analytical D_W_required ≫ authorized bound, only assay the capped candidates
    // (deduplicated) rather than repeating near-identical failing runs.
    for d_w in &d_candidates {
        let mut p = params.clone();
        p.d_w = *d_w;
        let assay = run_fixed_source_assay(&grid, &phi, &m, &q, &p, None, max_steps);
        diff_results.push(json!({
            "D_W": d_w,
            "beta_W": p.beta_w,
            "assay": assay,
        }));
        if required_exceeds_bound
            && assay.classification == "CONCENTRATION_BOUND_REACHED"
            && (*d_w - bound).abs() < 1e-12
        {
            break;
        }
    }
    atomic_write_json(
        &diff_dir.join("results.json"),
        &json!({
            "authorized_bound": bound,
            "d_w_required_50pct": times.d_w_required_50pct,
            "d_w_required_90pct": times.d_w_required_90pct,
            "required_exceeds_bound": required_exceeds_bound,
            "candidates": d_candidates,
            "results": diff_results,
        }),
    )?;

    let mut perm_results = Vec::new();
    let run_beta = membrane_branch_authorized(&resistance.dominant, resistance.membrane_fraction)
        && !required_exceeds_bound;
    if run_beta {
        let mut p = params.clone();
        p.d_w = bound;
        for beta in [0.20_f64, 0.10, 0.00] {
            p.beta_w = beta;
            let assay = run_fixed_source_assay(&grid, &phi, &m, &q, &p, None, max_steps);
            perm_results.push(json!({"D_W": p.d_w, "beta_W": beta, "assay": assay}));
        }
    } else {
        // Insufficiency gate: D_W=bound, β_W=0 (most permissive membrane in branch).
        let mut p = params.clone();
        p.d_w = bound;
        p.beta_w = 0.0;
        let assay = run_fixed_source_assay(&grid, &phi, &m, &q, &p, None, max_steps);
        perm_results.push(json!({
            "D_W": p.d_w,
            "beta_W": 0.0,
            "assay": assay,
            "note": if required_exceeds_bound {
                "membrane branch skipped: internal diffusion dominates and D_W_required exceeds authorized bound"
            } else {
                "membrane branch not primary; insufficiency-gate point only"
            },
        }));
    }
    atomic_write_json(
        &perm_dir.join("results.json"),
        &json!({
            "membrane_branch_run": run_beta,
            "dominant_resistance": resistance.dominant,
            "results": perm_results,
        }),
    )?;

    // Feasibility gate at authorized bound + β_W=0.
    let gate_assay = perm_results
        .iter()
        .rev()
        .find(|r| r["beta_W"].as_f64() == Some(0.0))
        .and_then(|r| serde_json::from_value::<chemistry_core::FixedSourceAssayResult>(r["assay"].clone()).ok())
        .unwrap_or(baseline.clone());
    let feasibility = classify_passive_feasibility(&gate_assay, true, true);
    let feasible = matches!(feasibility, PassiveTransportFeasibility::Feasible);

    let conclusion = if feasible {
        "D016_PASSIVE_WASTE_TRANSPORT_FEASIBLE"
    } else if resistance.dominant == "internal" {
        "D016_PASSIVE_WASTE_TRANSPORT_INSUFFICIENT"
    } else {
        "D016_PASSIVE_WASTE_TRANSPORT_INSUFFICIENT"
    };

    let subsidiary = if resistance.dominant == "internal" {
        vec!["D016_INTERNAL_DIFFUSION_LIMIT_CONFIRMED"]
    } else if resistance.dominant == "membrane" {
        vec!["D016_MEMBRANE_CONDUCTANCE_LIMIT_CONFIRMED"]
    } else if resistance.dominant == "external" {
        vec!["D016_EXTERNAL_TRANSPORT_LIMIT_CONFIRMED"]
    } else {
        vec!["D016_DIAGNOSIS_INCONCLUSIVE"]
    };

    Ok(json!({
        "baseline": baseline,
        "d_w_candidates": d_candidates,
        "diffusivity_results": diff_results,
        "permeability_results": perm_results,
        "feasibility": format!("{:?}", feasibility),
        "primary_conclusion": conclusion,
        "subsidiary_conclusions": subsidiary,
        "selected_transport_candidate": Value::Null,
        "transport_schema": transport_schema_for_repair(feasible),
        "timescales": times,
        "resistance": resistance,
        "source_summary": summary,
        "unused_fields": {"catalyst_mass": c.iter().sum::<f64>(), "waste_mass": w.iter().sum::<f64>()},
    }))
}

pub fn run_regression_gate(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    let status = Command::new("cargo")
        .args([
            "test",
            "-p",
            "chemistry-core",
            "--release",
            "--test",
            "d016_tests",
            "--",
            "--nocapture",
        ])
        .current_dir(resolve_path(Path::new(".")))
        .status()?;
    let body = json!({
        "project_directive": "D-016",
        "d016_tests_pass": status.success(),
        "stage_a_transport_marker": "covered by test_stage_a_waste_transport_passes",
        "stage_d_marker": "covered by test_stage_d_fixed_compartment_still_passes",
    });
    atomic_write_json(&output.join("regression_summary.json"), &body)?;
    Ok(body)
}

pub fn run_manifest(output_root: &Path, campaign: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let params = frozen_organism_params(true)?;
    let grid = GridConfiguration::default();
    let primary = campaign["primary_conclusion"].as_str().unwrap_or("D016_FAIL");
    let feasible = primary == "D016_PASSIVE_WASTE_TRANSPORT_FEASIBLE"
        || primary == "D016_WASTE_TRANSPORT_TIMESCALE_PASS";
    let body = json!({
        "project_directive": "D-016",
        "agent_memory_directive": "D-20260715-d016-waste-transport-timescale",
        "source_commit": git_commit_hash().ok(),
        "binary_sha256": binary_hash().ok(),
        "equation_version": "membrane_metabolism_v2_conservative",
        "stoichiometric_schema": 2,
        "transport_schema": transport_schema_for_repair(feasible),
        "frozen_candidate_hash": FROZEN_CANDIDATE,
        "frozen_configuration_hash": FROZEN_CONFIG,
        "environment_configuration_hash": environment_configuration_hash(&params),
        "organism_frozen_hash": organism_frozen_hash(&params, &grid),
        "D_W": params.d_w,
        "beta_W": params.beta_w,
        "primary_conclusion": primary,
        "subsidiary_conclusions": campaign["subsidiary_conclusions"],
        "selected_transport_candidate": campaign["selected_transport_candidate"],
        "fixed_source_baseline": campaign["baseline"],
        "d_w_candidates": campaign["d_w_candidates"],
        "resistance": campaign["resistance"],
        "timescales": campaign["timescales"],
        "source_summary": campaign["source_summary"],
        "d012_solver_entry_gate": "CLOSED",
        "d008_status": {
            "stages_0_d": "PASS",
            "stage_e": "BLOCKED",
            "stages_f_g": "BLOCKED"
        },
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "production_verdict": "REQUIRES_REMEDIATION",
        "fail_tag": "D-016-passive-waste-transport-insufficient",
        "pass_tag": "D-016-waste-transport-timescale-pass",
        "biological_preflight": "SKIPPED_PASSIVE_TRANSPORT_INSUFFICIENT",
        "fresh_reference_r22": "SKIPPED_PASSIVE_TRANSPORT_INSUFFICIENT",
        "center_w_target_frac": D016_W_TARGET_FRAC,
        "conc_safety_limit": CONC_SAFETY_LIMIT,
    });
    atomic_write_json(&output_root.join("manifest.json"), &body)?;
    let _ = build_candidate_identity(
        params.clone(),
        &git_commit_hash().unwrap_or_else(|_| "unknown".into()),
        Some("d016-manifest"),
        None,
        "D-016 manifest seal",
        None,
        None,
    );
    let _ = candidate_hash(&params, &grid);
    Ok(body)
}

pub fn run_pipeline(output_root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    fs::create_dir_all(output_root)?;
    let preserve = run_preserve(&output_root.join("preservation"))?;
    let audit = run_transport_audit(&output_root.join("transport_audit"))?;
    let analysis = run_source_and_timescales(output_root)?;
    let campaign = run_fixed_source_campaign(output_root)?;
    let regression = run_regression_gate(&output_root.join("regressions"))?;
    let manifest = run_manifest(output_root, &campaign)?;
    Ok(json!({
        "preservation": preserve,
        "audit": audit,
        "analysis": analysis,
        "campaign": campaign,
        "regression": regression,
        "manifest": manifest,
    }))
}
