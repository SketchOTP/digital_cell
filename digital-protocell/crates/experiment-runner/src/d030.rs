//! D-030 orthogonal reversible-exchange identification runner.

use crate::d013::{atomic_write_json, load_governed_checkpoint, restore_governed_simulation};
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d027_analysis::{surface_balance_q, WindowLocalSurfaceRates};
use chemistry_core::d029_analysis::{apply_exchange_candidate, ExchangeCandidate};
use chemistry_core::d030_analysis::{
    adsorption_matrix_specs, desorption_matrix_specs, recover_exchange_parameters,
    run_orthogonal_assay, OrthogonalAssaySpec, D030_MIXED_FLUX_REL_MAX,
};
use chemistry_core::surface_density::{
    compute_interface_geometry, surface_localization, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGENT_MEMORY_ID: &str = "D-20260718-d030-orthogonal-reversible-exchange-identification";
/// Equilibrium constant chosen so K > typical B/A (~10) keeps net adsorption possible.
const SEED_K_EQ: f64 = 50.0;
/// Initial k bracket for renewal-compatible seed.
const SEED_K_LOW: f64 = 0.0002;
const SEED_K_HIGH: f64 = 0.02;
const ISOLATED_STEPS: u64 = 12_000;
const SEED_SCREEN_STEPS: u64 = 6_000;
const PORTABILITY_MEASURE: u64 = 2_000;

/// Runtime-selected planted candidate (filled by seed screen).
fn planted_candidate() -> ExchangeCandidate {
    SEED_CANDIDATE
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or(ExchangeCandidate {
            identity: "d030_fallback_seed".into(),
            k_exchange: 0.005,
            k_exchange_eq: SEED_K_EQ,
        })
}

use std::sync::Mutex;
static SEED_CANDIDATE: Mutex<Option<ExchangeCandidate>> = Mutex::new(None);

fn set_planted_candidate(c: ExchangeCandidate) {
    if let Ok(mut g) = SEED_CANDIDATE.lock() {
        *g = Some(c);
    }
}

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

fn binary_hash() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| fs::read(p).ok())
        .map(|b| chemistry_core::sha256_hex(&b))
        .unwrap_or_else(|| "unknown".into())
}

fn source_hash(rel: &str) -> String {
    let p = resolve_path(Path::new(rel));
    fs::read(&p)
        .ok()
        .map(|b| chemistry_core::sha256_hex(&b))
        .unwrap_or_else(|| "missing".into())
}

fn disk_status() -> Value {
    // Compact: parse `df -B1 .` free bytes when available.
    let out = Command::new("df")
        .args(["-B1", "."])
        .output()
        .ok();
    if let Some(o) = out {
        if let Ok(text) = String::from_utf8(o.stdout) {
            if let Some(line) = text.lines().nth(1) {
                let cols: Vec<&str> = line.split_whitespace().collect();
                if cols.len() >= 4 {
                    let total: u64 = cols[1].parse().unwrap_or(0);
                    let used: u64 = cols[2].parse().unwrap_or(0);
                    let free: u64 = cols[3].parse().unwrap_or(0);
                    return json!({
                        "total_bytes": total,
                        "used_bytes": used,
                        "free_bytes": free,
                        "free_gib": free as f64 / (1024.0 * 1024.0 * 1024.0),
                        "used_pct": if total > 0 { 100.0 * used as f64 / total as f64 } else { 0.0 },
                    });
                }
            }
        }
    }
    json!({ "error": "df_unavailable" })
}

fn tag_commit(tag: &str) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", tag])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

fn compact_assay(r: &chemistry_core::d030_analysis::OrthogonalAssayResult) -> Value {
    json!({
        "label": r.spec.label,
        "theta0": r.spec.theta0,
        "precursor0": r.spec.precursor0,
        "catalyst0": r.spec.catalyst0,
        "q_c": r.q_c,
        "first": r.first,
        "n_traj": r.trajectory_p.len(),
        "p_final": r.trajectory_p.last(),
        "s_final": r.trajectory_s.last(),
        "theta_final": r.trajectory_theta.last(),
        "pass_gates": r.pass_gates,
        "notes": r.notes,
    })
}

/// Gate 0 — preservation and executable baseline.
pub fn run_gate0_preservation(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let tags = [
        "D-021-retention-localization-not-recovered",
        "D-022-localization-not-recovered",
        "D-023-precursor-assembly-fail",
        "D-024-surface-density-pass",
        "D-024-surface-density-pass-provenance-sealed",
        "D-025-surface-density-recovery-fail",
        "D-026-stage-e-recovery-fail",
        "D-027-surface-renewal-fail",
        "D-028-bracketed-renewal-fail",
        "D-029-reversible-exchange-fail",
    ];
    let tag_rows: Vec<Value> = tags
        .iter()
        .map(|t| {
            let c = tag_commit(t);
            json!({ "tag": t, "present": c.is_some(), "commit": c })
        })
        .collect();
    let all_tags = tag_rows.iter().all(|r| r["present"] == true);
    let d029_commit = tag_commit("D-029-reversible-exchange-fail");
    let starting = "9c4a4ea";
    let starting_full = tag_commit("9c4a4ea").or_else(|| {
        Command::new("git")
            .args(["rev-parse", starting])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    });
    let artifacts = [
        (
            "d029_preservation",
            "experiments/generated/d029/preservation/preservation_manifest.json",
        ),
        (
            "d029_gate1",
            "experiments/generated/d029/exchange_unit/gate1_unit.json",
        ),
        (
            "d029_identification",
            "experiments/generated/d029/parameter_identification/parameter_identification.json",
        ),
        (
            "d027_adsorption_basis",
            "experiments/generated/d027/adsorption_basis/adsorption_basis.json",
        ),
        (
            "d028_manifest",
            "experiments/generated/d028/manifest.json",
        ),
    ];
    let mut preserved = serde_json::Map::new();
    for (name, rel) in artifacts {
        let p = resolve_path(Path::new(rel));
        preserved.insert(
            name.to_string(),
            json!({
                "path": format!("digital-protocell/{rel}"),
                "exists": p.is_file(),
                "sha256": source_hash(rel),
                "bytes": fs::metadata(&p).map(|m| m.len()).unwrap_or(0),
            }),
        );
    }
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    let pass = all_tags
        && d029_commit.is_some()
        && starting_full.is_some()
        && preserved.values().all(|v| v["exists"] == true);
    let body = json!({
        "project_directive": "D-030",
        "agent_memory_directive": AGENT_MEMORY_ID,
        "gate": 0,
        "pass": pass,
        "conclusion": if pass { "D030_PRESERVATION_PASS" } else { "D030_PRESERVATION_FAILURE" },
        "branch": branch,
        "starting_result_commit": starting,
        "starting_result_commit_full": starting_full,
        "d029_tag_commit": d029_commit,
        "d029_operative_reinterpretation": "REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE_FROM_NATURAL_BALANCE_STATES",
        "d029_historical_conclusion_unchanged": "D029_REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE",
        "record_preserved": "IRREVERSIBLE_ADSORPTION_LAW_REJECTED",
        "equation_version": EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange.as_str(),
        "surface_exchange_schema_version": 2,
        "disk": disk_status(),
        "tags": tag_rows,
        "preserved_artifacts": preserved,
        "source_hashes": {
            "surface_density.rs": source_hash("crates/chemistry-core/src/surface_density.rs"),
            "d029_analysis.rs": source_hash("crates/chemistry-core/src/d029_analysis.rs"),
            "d030_analysis.rs": source_hash("crates/chemistry-core/src/d030_analysis.rs"),
        },
        "binary_hash": binary_hash(),
        "source_commit": git_commit_hash(),
        "artifact_policy": "compact_transient_only",
    });
    atomic_write_json(&output.join("preservation_manifest.json"), &body)?;
    Ok(body)
}

/// Gate 1 — transient exchange observability (diagnostics only).
pub fn run_gate1_observability(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let c = planted_candidate();
    let mut params = SimParams::default();
    apply_exchange_candidate(&mut params, &c);
    let ads = OrthogonalAssaySpec {
        label: "obs_ads".into(),
        theta0: 0.0,
        precursor0: 0.5,
        catalyst0: chemistry_core::d030_analysis::catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 12,
        theta_stop: 0.05,
    };
    let des = OrthogonalAssaySpec {
        label: "obs_des".into(),
        theta0: 0.5,
        precursor0: 0.0,
        catalyst0: chemistry_core::d030_analysis::catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 12,
        theta_stop: 1.0,
    };
    let a = run_orthogonal_assay(c.k_exchange, c.k_exchange_eq, &ads)?;
    let d = run_orthogonal_assay(c.k_exchange, c.k_exchange_eq, &des)?;
    let pass = a.first.forward_exchange > 0.0
        && a.first.reverse_exchange.abs() < 1e-12
        && d.first.reverse_exchange > 0.0
        && d.first.forward_exchange.abs() < 1e-12
        && a.first.accounting_residual < 1e-9
        && d.first.accounting_residual < 1e-9;
    let body = json!({
        "project_directive": "D-030",
        "gate": 1,
        "pass": pass,
        "adsorption": compact_assay(&a),
        "desorption": compact_assay(&d),
        "planted": c,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("exchange_observability.json"), &body)?;
    Ok(body)
}

/// Gate 2 — forward adsorption identification.
pub fn run_gate2_adsorption(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let c = planted_candidate();
    let mut params = SimParams::default();
    apply_exchange_candidate(&mut params, &c);
    let specs = adsorption_matrix_specs(&params);
    let mut rows = Vec::new();
    let mut alphas = Vec::new();
    let mut by_q = vec![Vec::new(); 3];
    let mut all_ok = true;
    for (i, spec) in specs.iter().enumerate() {
        let r = run_orthogonal_assay(c.k_exchange, c.k_exchange_eq, spec)?;
        let ok = r.first.net_exchange > 0.0
            && r.first.reverse_exchange.abs() < 1e-12
            && r.first.alpha_estimate.is_finite()
            && r.first.alpha_estimate > 0.0
            && r.first.accounting_residual < 1e-9
            && r.first.exchange_dissipation >= -1e-12
            && r.first.mean_theta <= 0.05 + 1e-6;
        all_ok &= ok;
        alphas.push(r.first.alpha_estimate);
        by_q[i / 3].push(r.first.alpha_estimate);
        let compact = compact_assay(&r);
        atomic_write_json(&output.join(format!("{}.json", spec.label)), &compact)?;
        rows.push(compact);
    }
    let alpha_direct = chemistry_core::d030_analysis::robust_median(&alphas);
    let spread = chemistry_core::d030_analysis::relative_spread(&alphas);
    let q_meds: Vec<f64> = by_q
        .iter()
        .map(|g| chemistry_core::d030_analysis::robust_median(g))
        .collect();
    let q_spread = chemistry_core::d030_analysis::relative_spread(&q_meds);
    let pass = all_ok
        && alpha_direct.is_finite()
        && alpha_direct > 0.0
        && spread <= 0.10
        && q_spread <= 0.10;
    let body = json!({
        "project_directive": "D-030",
        "gate": 2,
        "pass": pass,
        "conclusion": if pass { "D030_FORWARD_EXCHANGE_IDENTIFIED" } else { "D030_FORWARD_EXCHANGE_NOT_IDENTIFIABLE" },
        "alpha_estimates": alphas,
        "alpha_direct": alpha_direct,
        "alpha_spread": spread,
        "alpha_q_norm_spread": q_spread,
        "matrix": rows,
        "planted": c,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("adsorption_transients.json"), &body)?;
    Ok(body)
}

/// Gate 3 — reverse desorption identification.
pub fn run_gate3_desorption(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let c = planted_candidate();
    let mut params = SimParams::default();
    apply_exchange_candidate(&mut params, &c);
    let specs = desorption_matrix_specs(&params);
    let mut rows = Vec::new();
    let mut betas = Vec::new();
    let mut by_q = vec![Vec::new(); 3];
    let mut all_ok = true;
    for (i, spec) in specs.iter().enumerate() {
        let r = run_orthogonal_assay(c.k_exchange, c.k_exchange_eq, spec)?;
        let ok = r.first.net_exchange < 0.0
            && r.first.forward_exchange.abs() < 1e-12
            && r.first.beta_estimate.is_finite()
            && r.first.beta_estimate > 0.0
            && r.first.accounting_residual < 1e-9
            && r.first.exchange_dissipation >= -1e-12;
        all_ok &= ok;
        betas.push(r.first.beta_estimate);
        by_q[i / 3].push(r.first.beta_estimate);
        let compact = compact_assay(&r);
        atomic_write_json(&output.join(format!("{}.json", spec.label)), &compact)?;
        rows.push(compact);
    }
    let beta_direct = chemistry_core::d030_analysis::robust_median(&betas);
    let spread = chemistry_core::d030_analysis::relative_spread(&betas);
    let q_meds: Vec<f64> = by_q
        .iter()
        .map(|g| chemistry_core::d030_analysis::robust_median(g))
        .collect();
    let q_spread = chemistry_core::d030_analysis::relative_spread(&q_meds);
    let pass = all_ok
        && beta_direct.is_finite()
        && beta_direct > 0.0
        && spread <= 0.10
        && q_spread <= 0.10;
    let body = json!({
        "project_directive": "D-030",
        "gate": 3,
        "pass": pass,
        "conclusion": if pass { "D030_REVERSE_EXCHANGE_IDENTIFIED" } else { "D030_REVERSE_EXCHANGE_NOT_IDENTIFIABLE" },
        "beta_estimates": betas,
        "beta_direct": beta_direct,
        "beta_spread": spread,
        "beta_q_norm_spread": q_spread,
        "matrix": rows,
        "planted": c,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("desorption_transients.json"), &body)?;
    Ok(body)
}

/// Gate 4 — parameter recovery from Gate 2/3 matrices.
pub fn run_gate4_recovery(
    output: &Path,
    gate2: &Value,
    gate3: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let alphas: Vec<f64> = serde_json::from_value(gate2["alpha_estimates"].clone())?;
    let betas: Vec<f64> = serde_json::from_value(gate3["beta_estimates"].clone())?;
    let mut a_by_q = vec![Vec::new(); 3];
    let mut b_by_q = vec![Vec::new(); 3];
    for (i, a) in alphas.iter().enumerate() {
        a_by_q[i / 3].push(*a);
    }
    for (i, b) in betas.iter().enumerate() {
        b_by_q[i / 3].push(*b);
    }
    let rec = recover_exchange_parameters(&alphas, &betas, &a_by_q, &b_by_q);
    let body = json!({
        "project_directive": "D-030",
        "gate": 4,
        "pass": rec.identifiable,
        "recovery": rec,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("parameter_recovery.json"), &body)?;
    Ok(body)
}

fn candidate_from_recovery(rec: &Value) -> ExchangeCandidate {
    let fallback = planted_candidate();
    ExchangeCandidate {
        identity: "d030_identified".into(),
        k_exchange: rec["recovery"]["k_exchange"]
            .as_f64()
            .unwrap_or(fallback.k_exchange),
        k_exchange_eq: rec["recovery"]["k_exchange_eq"]
            .as_f64()
            .unwrap_or(fallback.k_exchange_eq),
    }
}

/// Gate 5 — mixed-state cross-validation.
pub fn run_gate5_mixed(
    output: &Path,
    c: &ExchangeCandidate,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let mut params = SimParams::default();
    apply_exchange_candidate(&mut params, c);
    let cat = chemistry_core::d030_analysis::catalyst_for_q(&params, 0.5);
    let alpha = c.k_exchange * c.k_exchange_eq;
    let beta = c.k_exchange;
    let mix_ads = [
        (0.25, 0.20),
        (0.50, 0.40),
        (1.00, 0.60),
    ];
    let mix_des_theta = [0.30, 0.50, 0.70];
    let mut rows = Vec::new();
    let mut all_ok = true;
    for &(p, th) in &mix_ads {
        let spec = OrthogonalAssaySpec {
            label: format!("mix_ads_p{p}_th{th}"),
            theta0: th,
            precursor0: p,
            catalyst0: cat,
            radius: 10.0,
            dt: 1e-3,
            max_steps: 30,
            theta_stop: 1.0,
        };
        let r = run_orthogonal_assay(c.k_exchange, c.k_exchange_eq, &spec)?;
        // Predicted initial net ~ (α p (1-θ) − β θ) * scale; direction check + flux within 15%.
        let pred_sign = alpha * p * (1.0 - th) - beta * th;
        let ok_dir = r.first.net_exchange.signum() == pred_sign.signum() || pred_sign.abs() < 1e-15;
        let model_rate = r.first.adsorption_basis * alpha - r.first.desorption_basis * beta;
        let pred_xfer = model_rate * r.first.dt;
        let rel = if pred_xfer.abs() > 1e-15 {
            ((r.first.net_exchange - pred_xfer) / pred_xfer).abs()
        } else {
            0.0
        };
        let ok = ok_dir
            && rel <= D030_MIXED_FLUX_REL_MAX
            && r.first.accounting_residual < 1e-9
            && r.first.exchange_dissipation >= -1e-12;
        all_ok &= ok;
        rows.push(json!({
            "kind": "mixed_ads",
            "result": compact_assay(&r),
            "pred_sign": pred_sign.signum(),
            "flux_rel_err": rel,
            "ok": ok,
        }));
    }
    for &th in &mix_des_theta {
        let p_eq = th / ((1.0 - th) * c.k_exchange_eq).max(1e-30);
        let p = (0.5 * p_eq).max(0.0);
        let spec = OrthogonalAssaySpec {
            label: format!("mix_des_th{th}_p{p:.4}"),
            theta0: th,
            precursor0: p,
            catalyst0: cat,
            radius: 10.0,
            dt: 1e-3,
            max_steps: 30,
            theta_stop: 1.0,
        };
        let r = run_orthogonal_assay(c.k_exchange, c.k_exchange_eq, &spec)?;
        let ok = r.first.net_exchange < 0.0
            && r.first.accounting_residual < 1e-9
            && r.first.exchange_dissipation >= -1e-12;
        all_ok &= ok;
        rows.push(json!({
            "kind": "mixed_des",
            "p_eq": p_eq,
            "p": p,
            "result": compact_assay(&r),
            "ok": ok,
        }));
    }
    let body = json!({
        "project_directive": "D-030",
        "gate": 5,
        "pass": all_ok,
        "conclusion": if all_ok { "D030_MIXED_CROSS_VALIDATION_PASS" } else { "D030_TRANSIENT_EXCHANGE_MODEL_MISMATCH" },
        "candidate": c,
        "rows": rows,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("mixed_cross_validation.json"), &body)?;
    Ok(body)
}

/// Gate 6 — equilibrium-family validation.
pub fn run_gate6_equilibrium(
    output: &Path,
    c: &ExchangeCandidate,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let mut params = SimParams::default();
    apply_exchange_candidate(&mut params, c);
    let cat = chemistry_core::d030_analysis::catalyst_for_q(&params, 0.5);
    // Use faster local kinetics for compact equilibrium families (same K; scale k).
    let k_fast = c.k_exchange.max(0.5);
    let inventories = [4.0, 8.0, 12.0, 16.0];
    let fracs = [0.15, 0.45, 0.75];
    let mut families = Vec::new();
    let mut all_ok = true;
    for &inv in &inventories {
        let mut finals = Vec::new();
        for &fs in &fracs {
            let (th, p_mass, t0, t1) =
                chemistry_core::d030_analysis::run_equilibrium_partition_assay(
                    k_fast,
                    c.k_exchange_eq,
                    10.0,
                    inv,
                    fs,
                    cat,
                    5e-3,
                    800,
                )?;
            let conserved = (t1 - t0).abs() < 1e-7;
            finals.push(json!({
                "surface_fraction0": fs,
                "theta": th,
                "p_mass": p_mass,
                "total0": t0,
                "total1": t1,
                "conserved": conserved,
            }));
            all_ok &= conserved && (0.0..=1.0).contains(&th);
        }
        let thetas: Vec<f64> = finals
            .iter()
            .filter_map(|v| v["theta"].as_f64())
            .collect();
        let med = chemistry_core::d030_analysis::robust_median(&thetas);
        let partition_indep = thetas.iter().all(|t| (t - med).abs() < 0.08);
        all_ok &= partition_indep;
        families.push(json!({
            "inventory": inv,
            "finals": finals,
            "median_theta": med,
            "partition_independent": partition_indep,
            "ok": partition_indep,
        }));
    }
    let body = json!({
        "project_directive": "D-030",
        "gate": 6,
        "pass": all_ok,
        "conclusion": if all_ok { "D030_EQUILIBRIUM_FAMILY_PASS" } else { "D030_EQUILIBRIUM_FAMILY_MISMATCH" },
        "candidate": c,
        "k_fast_for_equilibration": k_fast,
        "families": families,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
        "note": "Equilibration uses k_fast=max(k,0.5) at fixed K to keep artifacts compact; isotherm K unchanged",
    });
    atomic_write_json(&output.join("equilibrium_families.json"), &body)?;
    Ok(body)
}

fn v8_from_candidate(c: &ExchangeCandidate) -> Result<chemistry_core::SimParams, Box<dyn std::error::Error>> {
    let mut p = v7_base_params()?;
    apply_exchange_candidate(&mut p, c);
    Ok(p)
}

fn gamma_localization(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    surface_localization(
        &sim.grid,
        &geometry,
        &sim.fields.membrane,
        sim.params.delta_floor,
    )
}

/// Short isolated Q screen used to seed orthogonal identification.
fn screen_isolated_q(
    c: &ExchangeCandidate,
    steps: u64,
) -> Result<(f64, f64, f64, f64), Box<dyn std::error::Error>> {
    let params = v8_from_candidate(c)?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }
    let burn = steps * 2 / 3;
    let measure = steps - burn;
    for _ in 0..burn {
        if !sim.step() {
            break;
        }
    }
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut s_sum = 0.0;
    let mut n = 0u64;
    for _ in 0..measure.max(200) {
        if !sim.step() {
            break;
        }
        if sim.substep % 20 == 0 {
            s_sum += chemistry_core::surface_density::total_surface_mass(
                &sim.grid,
                &sim.fields.membrane,
            );
            n += 1;
        }
    }
    let rates = WindowLocalSurfaceRates::from_sim(&sim);
    let mean_s = if n > 0 {
        s_sum / n as f64
    } else {
        chemistry_core::surface_density::total_surface_mass(&sim.grid, &sim.fields.membrane)
    };
    let q = rates.adsorption / rates.gamma_turnover.max(f64::EPSILON);
    let g = (rates.adsorption - rates.gamma_turnover) / mean_s.max(f64::EPSILON);
    let fwd = sim.surface_accounting.window_local().exchange_forward;
    let rev = sim.surface_accounting.window_local().exchange_reverse;
    Ok((q, g, fwd, rev))
}

/// Select renewal-compatible (k, K) seed before orthogonal ID (not a biological six-state fit).
pub fn run_seed_screen(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let k_eq = SEED_K_EQ;
    let mut trials = Vec::new();
    let mut k_low = SEED_K_LOW;
    let mut k_high = SEED_K_HIGH;
    let c_low = ExchangeCandidate {
        identity: "seed_low".into(),
        k_exchange: k_low,
        k_exchange_eq: k_eq,
    };
    let c_high = ExchangeCandidate {
        identity: "seed_high".into(),
        k_exchange: k_high,
        k_exchange_eq: k_eq,
    };
    let (q_low, g_low, fwd_low, rev_low) = screen_isolated_q(&c_low, SEED_SCREEN_STEPS)?;
    let (q_high, g_high, fwd_high, rev_high) = screen_isolated_q(&c_high, SEED_SCREEN_STEPS)?;
    trials.push(json!({"k": k_low, "q": q_low, "g": g_low, "fwd": fwd_low, "rev": rev_low}));
    trials.push(json!({"k": k_high, "q": q_high, "g": g_high, "fwd": fwd_high, "rev": rev_high}));

    let mut selected = if (0.98..=1.02).contains(&q_low) {
        c_low.clone()
    } else if (0.98..=1.02).contains(&q_high) {
        c_high.clone()
    } else {
        // Regula-falsi toward Q=1 when bracket straddles (or nearest endpoint).
        let mut best = if (q_low - 1.0).abs() < (q_high - 1.0).abs() {
            c_low.clone()
        } else {
            c_high.clone()
        };
        let mut ql = q_low;
        let mut qh = q_high;
        for i in 0..4 {
            if (ql - 1.0) * (qh - 1.0) > 0.0 {
                break; // no sign change
            }
            let k_trial = chemistry_core::d028_analysis::regula_falsi_trial(k_low, ql, k_high, qh)
                .clamp(k_low * 1.001, k_high * 0.999);
            let c_trial = ExchangeCandidate {
                identity: format!("seed_rf_{i}"),
                k_exchange: k_trial,
                k_exchange_eq: k_eq,
            };
            let (q_t, g_t, fwd_t, rev_t) = screen_isolated_q(&c_trial, SEED_SCREEN_STEPS)?;
            trials.push(json!({"k": k_trial, "q": q_t, "g": g_t, "fwd": fwd_t, "rev": rev_t}));
            if (0.98..=1.02).contains(&q_t) {
                best = c_trial;
                break;
            }
            if (q_t - 1.0).abs() < (if best.k_exchange == k_low {
                (ql - 1.0).abs()
            } else {
                (qh - 1.0).abs()
            }) {
                best = c_trial.clone();
            }
            if q_t < 1.0 {
                k_low = k_trial;
                ql = q_t;
            } else {
                k_high = k_trial;
                qh = q_t;
            }
        }
        best
    };
    selected.identity = "d030_renewal_seed".into();
    set_planted_candidate(selected.clone());
    let (q_s, g_s, fwd_s, rev_s) = screen_isolated_q(&selected, SEED_SCREEN_STEPS)?;
    let body = json!({
        "project_directive": "D-030",
        "gate": "seed",
        "K_exchange": k_eq,
        "selected": selected,
        "q_seed": q_s,
        "g_seed": g_s,
        "forward": fwd_s,
        "reverse": rev_s,
        "trials": trials,
        "note": "Seed from isolated renewal screen at fixed K; orthogonal Gates 2–4 identify α,β of this seed",
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("seed_screen.json"), &body)?;
    Ok(body)
}

/// Gate 7 — biological turnover reconstruction (D-027 isolated config).
pub fn run_gate7_isolated_turnover(
    output: &Path,
    c: &ExchangeCandidate,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let params = v8_from_candidate(c)?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }
    let burn = ISOLATED_STEPS * 2 / 3;
    let measure = ISOLATED_STEPS - burn;
    for _ in 0..burn {
        if !sim.step() {
            break;
        }
    }
    let window = (measure / 3).max(200);
    let mut windows = Vec::new();
    let mut consecutive = 0usize;
    let mut steps_ok = true;
    for _ in 0..3 {
        sim.surface_accounting
            .begin_window_local(sim.substep, sim.sim_time);
        let mut s_sum = 0.0;
        let mut n = 0u64;
        let mut accepted = 0u64;
        for _ in 0..window {
            if !sim.step() {
                steps_ok = false;
                break;
            }
            accepted += 1;
            if sim.substep % 20 == 0 {
                s_sum += chemistry_core::surface_density::total_surface_mass(
                    &sim.grid,
                    &sim.fields.membrane,
                );
                n += 1;
            }
        }
        let rates = WindowLocalSurfaceRates::from_sim(&sim);
        let mean_s = if n > 0 {
            s_sum / n as f64
        } else {
            chemistry_core::surface_density::total_surface_mass(&sim.grid, &sim.fields.membrane)
        };
        let net = rates.adsorption;
        let turn = rates.gamma_turnover;
        let q = net / turn.max(f64::EPSILON);
        let g = (net - turn) / mean_s.max(f64::EPSILON);
        let loc = gamma_localization(&sim);
        let wl = sim.surface_accounting.window_local();
        let fwd = wl.exchange_forward;
        let rev = wl.exchange_reverse;
        let ok = steps_ok
            && accepted >= window / 2
            && (0.98..=1.02).contains(&q)
            && g.abs() <= 1e-4
            && loc >= 0.98
            && fwd > 0.0
            && rev > 0.0
            && turn > 0.0
            && sim.fields.precursor.iter().all(|v| v.is_finite() && *v >= 0.0)
            && sim.fields.membrane.iter().all(|v| v.is_finite() && *v >= 0.0);
        if ok {
            consecutive += 1;
        } else {
            consecutive = 0;
        }
        windows.push(json!({
            "q_renewal": q,
            "g_surface": g,
            "localization": loc,
            "forward": fwd,
            "reverse": rev,
            "net": net,
            "turnover": turn,
            "accepted_in_window": accepted,
            "last_reject": sim.last_reject_detail.clone(),
            "ok": ok,
        }));
    }
    let pass = consecutive >= 3;
    let body = json!({
        "project_directive": "D-030",
        "gate": 7,
        "pass": pass,
        "conclusion": if pass { "D030_TURNOVER_RECONSTRUCTION_PASS" } else { "D030_TURNOVER_EXCHANGE_INCOMPATIBILITY" },
        "candidate": c,
        "windows": windows,
        "consecutive_ok": consecutive,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("isolated_turnover.json"), &body)?;
    Ok(body)
}

fn stage_e_ckpt(step: u64) -> PathBuf {
    resolve_path(Path::new(&format!(
        "experiments/generated/d025/stage_e_reference/checkpoints/checkpoint_{:06}.json",
        step
    )))
}

fn restore_stage_e_checkpoint(step: u64) -> Result<Simulation, Box<dyn std::error::Error>> {
    let path = stage_e_ckpt(step);
    if !path.is_file() {
        return Err(format!("missing Stage E checkpoint {}", path.display()).into());
    }
    let ckpt = load_governed_checkpoint(&path)?;
    let mut sim = Simulation::new(v7_base_params()?);
    restore_governed_simulation(&mut sim, &ckpt)?;
    sim.enforce_structure_constraint = true;
    Ok(sim)
}

fn fixed_interface_r22_state() -> Result<Simulation, Box<dyn std::error::Error>> {
    let params = v7_base_params()?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }
    for _ in 0..2_000 {
        if !sim.step() {
            break;
        }
    }
    Ok(sim)
}

fn dynamic_r22_endpoint_state() -> Result<Simulation, Box<dyn std::error::Error>> {
    let params = v7_base_params()?;
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, 22.0, 0.6);
    for _ in 0..D026_SETTLE_STEPS {
        if !sim.step() {
            break;
        }
    }
    for _ in 0..4_000 {
        if !sim.step() {
            break;
        }
    }
    Ok(sim)
}

fn six_states() -> Result<Vec<(String, Simulation)>, Box<dyn std::error::Error>> {
    Ok(vec![
        ("d024_fixed_interface_r22".into(), fixed_interface_r22_state()?),
        ("d025_dynamic_r22_endpoint".into(), dynamic_r22_endpoint_state()?),
        ("d026_stage_e_10000".into(), restore_stage_e_checkpoint(10_000)?),
        ("d026_stage_e_25000".into(), restore_stage_e_checkpoint(25_000)?),
        ("d026_stage_e_100000".into(), restore_stage_e_checkpoint(100_000)?),
        ("d026_stage_e_200000".into(), restore_stage_e_checkpoint(200_000)?),
    ])
}

/// Gate 8 — six-state portability (validation only; no refit).
pub fn run_gate8_portability(
    output: &Path,
    c: &ExchangeCandidate,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = resolve_path(output);
    fs::create_dir_all(&output)?;
    let mut rows = Vec::new();
    let mut pass_count = 0usize;
    let mut hard_fail = false;
    for (label, mut base) in six_states()? {
        let mut params = base.params.clone();
        apply_exchange_candidate(&mut params, c);
        base.params = params;
        // Re-seed surface accounting window after param swap.
        for _ in 0..500 {
            if !base.step() {
                break;
            }
        }
        base.surface_accounting
            .begin_window_local(base.substep, base.sim_time);
        for _ in 0..PORTABILITY_MEASURE {
            if !base.step() {
                break;
            }
        }
        let rates = WindowLocalSurfaceRates::from_sim(&base);
        let q = surface_balance_q(rates.adsorption, rates.gamma_turnover);
        let g = rates.adsorption - rates.gamma_turnover;
        let loc = gamma_localization(&base);
        let p_ok = base.fields.precursor.iter().all(|v| v.is_finite() && *v >= 0.0);
        let s_ok = base.fields.membrane.iter().all(|v| v.is_finite() && *v >= 0.0);
        let sat_lock = rates.adsorption.abs() < 1e-15 && rates.gamma_turnover > 0.0;
        let state_pass = (0.90..=1.10).contains(&q)
            && p_ok
            && s_ok
            && !sat_lock
            && loc.is_finite();
        if state_pass {
            pass_count += 1;
        }
        if !p_ok || !s_ok {
            hard_fail = true;
        }
        let row = json!({
            "label": label,
            "q_renewal": q,
            "g_surface": g,
            "localization": loc,
            "adsorption": rates.adsorption,
            "turnover": rates.gamma_turnover,
            "pass": state_pass,
        });
        atomic_write_json(&output.join(format!("{label}.json")), &row)?;
        rows.push(row);
    }
    let pass = !hard_fail && pass_count >= 5;
    let body = json!({
        "project_directive": "D-030",
        "gate": 8,
        "pass": pass,
        "pass_count": pass_count,
        "conclusion": if pass { "D030_PORTABILITY_PASS" } else { "D030_REVERSIBLE_EXCHANGE_NOT_PORTABLE" },
        "candidate": c,
        "states": rows,
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&output.join("portability.json"), &body)?;
    Ok(body)
}

/// Full D-030 pipeline through available gates; stop at first failure.
pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let root = resolve_path(output);
    fs::create_dir_all(&root)?;
    for d in [
        "preservation",
        "exchange_observability",
        "adsorption_transients",
        "desorption_transients",
        "parameter_recovery",
        "mixed_cross_validation",
        "equilibrium_families",
        "isolated_turnover",
        "portability",
        "accounting",
        "dissipation",
    ] {
        fs::create_dir_all(root.join(d))?;
    }

    let g0 = run_gate0_preservation(&root.join("preservation"))?;
    if g0["pass"] != true {
        let body = json!({
            "project_directive": "D-030",
            "conclusion": "D030_PRESERVATION_FAILURE",
            "stopped_at": "gate0",
            "gate0": g0,
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let seed = run_seed_screen(&root.join("parameter_recovery"))?;
    // Seed always proceeds to orthogonal ID; Gate 7 is the renewal acceptance gate.

    let g1 = run_gate1_observability(&root.join("exchange_observability"))?;
    if g1["pass"] != true {
        let body = json!({
            "project_directive": "D-030",
            "conclusion": "D030_FAIL",
            "stopped_at": "gate1",
            "gate0": g0,
            "gate1": g1,
            "seed": seed,
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let g2 = run_gate2_adsorption(&root.join("adsorption_transients"))?;
    if g2["pass"] != true {
        let body = json!({
            "project_directive": "D-030",
            "conclusion": "D030_FORWARD_EXCHANGE_NOT_IDENTIFIABLE",
            "stopped_at": "gate2",
            "gate2": g2,
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let g3 = run_gate3_desorption(&root.join("desorption_transients"))?;
    if g3["pass"] != true {
        let body = json!({
            "project_directive": "D-030",
            "conclusion": "D030_REVERSE_EXCHANGE_NOT_IDENTIFIABLE",
            "stopped_at": "gate3",
            "gate2": g2,
            "gate3": g3,
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let g4 = run_gate4_recovery(&root.join("parameter_recovery"), &g2, &g3)?;
    if g4["pass"] != true {
        let conclusion = g4["recovery"]["conclusion"]
            .as_str()
            .unwrap_or("D030_EXCHANGE_PARAMETER_INCONSISTENCY");
        let body = json!({
            "project_directive": "D-030",
            "conclusion": conclusion,
            "stopped_at": "gate4",
            "gate4": g4,
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let identified = candidate_from_recovery(&g4);
    let g5 = run_gate5_mixed(&root.join("mixed_cross_validation"), &identified)?;
    if g5["pass"] != true {
        let body = json!({
            "project_directive": "D-030",
            "conclusion": "D030_TRANSIENT_EXCHANGE_MODEL_MISMATCH",
            "stopped_at": "gate5",
            "gate4": g4,
            "gate5": g5,
            "selected": identified,
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let g6 = run_gate6_equilibrium(&root.join("equilibrium_families"), &identified)?;
    if g6["pass"] != true {
        let body = json!({
            "project_directive": "D-030",
            "conclusion": "D030_EQUILIBRIUM_FAMILY_MISMATCH",
            "stopped_at": "gate6",
            "gate6": g6,
            "selected": identified,
            "pass": false,
            "source_commit": git_commit_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let g7 = run_gate7_isolated_turnover(&root.join("isolated_turnover"), &identified)?;
    if g7["pass"] != true {
        let body = json!({
            "project_directive": "D-030",
            "conclusion": "D030_TURNOVER_EXCHANGE_INCOMPATIBILITY",
            "stopped_at": "gate7",
            "gate4": g4,
            "gate7": g7,
            "selected": identified,
            "pass": false,
            "d029_operative_reinterpretation": "REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE_FROM_NATURAL_BALANCE_STATES",
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let g8 = run_gate8_portability(&root.join("portability"), &identified)?;
    if g8["pass"] != true {
        let body = json!({
            "project_directive": "D-030",
            "conclusion": "D030_REVERSIBLE_EXCHANGE_NOT_PORTABLE",
            "stopped_at": "gate8",
            "gate7": g7,
            "gate8": g8,
            "selected": identified,
            "pass": false,
            "source_commit": git_commit_hash(),
            "binary_hash": binary_hash(),
        });
        atomic_write_json(&root.join("manifest.json"), &body)?;
        return Ok(body);
    }

    let body = json!({
        "project_directive": "D-030",
        "conclusion": "D030_GATES_0_8_PASS_CONTINUE",
        "stopped_at": null,
        "pass": true,
        "selected": identified,
        "gate0": g0["pass"],
        "gate1": g1["pass"],
        "gate2": g2["pass"],
        "gate3": g3["pass"],
        "gate4": g4["pass"],
        "gate5": g5["pass"],
        "gate6": g6["pass"],
        "gate7": g7["pass"],
        "gate8": g8["pass"],
        "note": "Gates 9–17 (Stage B–E / productive / robustness) not yet executed in this revision",
        "source_commit": git_commit_hash(),
        "binary_hash": binary_hash(),
    });
    atomic_write_json(&root.join("manifest.json"), &body)?;
    Ok(body)
}
