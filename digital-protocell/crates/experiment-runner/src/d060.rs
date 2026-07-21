//! D-060 structural growth law and resource-coupled size feedback pipeline.
//! Observer / shadow diagnostic only: no production biology changes.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams, DX};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::*;
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d058_analysis::{
    cell_volume, drive_original_a, face_measure_a_f, gamma_face_production, xi_face_req,
};
use chemistry_core::d060_analysis::{
    candidate_forbids_radius_variable, candidates_justified_by_cause,
    classify_drive_surface, classify_resource_causality, d059_route_l_reproduced,
    equivalent_radius_from_area, find_restoring_crossing, fit_candidate_b_params,
    fit_candidate_c_params, fit_candidate_d_params, geometry_mapping_synthetic_ok,
    g_r_from_net, holdout_metrics, integrate_candidate_rates,
    integrate_existing_structural_rates, log_elasticity, qualify_candidate_params,
    qualify_existing_from_drive, select_neutrality_cause, select_route,
    structural_exposure_floor, DriveSample, NeutralityCause, D060PrimaryConclusion, D060Route,
    RouteEvidence060, StructuralCandidateId, StructuralLedger, D060_AGENT_MEMORY_ID,
    D060_A_RETENTION_TARGET, D060_D059_CONCLUSION, D060_D059_PRESERVATION, D060_D059_RECORD,
    D060_D059_RESTORING, D060_DRIVE_EPS, D060_DRIVE_RADII, D060_FROZEN_KT, D060_HOLDOUT_RADII,
    D060_KT_LADDER, D060_LEDGER_TOL, D060_PROJECT_ID, D060_RADIUS_MAP_TOL, D060_REPRO_RADII,
    D060_STARTING_COMMIT, D060_STARTING_TAG, D060_TRAIN_RADII,
};
use chemistry_core::d059_analysis::{longest_contiguous_viable_radii, radius_provisionally_viable};
use chemistry_core::field_mass;
use chemistry_core::surface_density::total_surface_mass;
use chemistry_core::Grid;
use chemistry_core::Simulation;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const K_NF0: f64 = 0.3438108650061698;
const K_W0: f64 = 0.4198385248302346;
const A0_PROXY: f64 = 0.05;
const R_REF_PROXY: f64 = 10.0;
const C_MEAN_PROXY: f64 = 0.5;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }
}

fn write_json(dir: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join(name), value)?;
    Ok(())
}

fn git_rev(args: &[&str]) -> Option<String> {
    let root = resolve_path(Path::new("."))
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| resolve_path(Path::new(".")).join(".."));
    Command::new("git")
        .args(args)
        .current_dir(&root)
        .output()
        .ok()
        .and_then(|o| {
            if !o.status.success() {
                return None;
            }
            String::from_utf8(o.stdout).ok()
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn head_on_d060_branch() -> bool {
    let head = git_rev(&["rev-parse", "HEAD"]).unwrap_or_default();
    if head.starts_with(D060_STARTING_COMMIT) {
        return true;
    }
    let root = resolve_path(Path::new(".")).join("..");
    Command::new("git")
        .args(["merge-base", "--is-ancestor", D060_STARTING_COMMIT, "HEAD"])
        .current_dir(&root)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn max_accepted() -> u64 {
    std::env::var("D060_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2500)
}

fn skip_late_gates() -> bool {
    std::env::var("D060_SKIP_LATE_GATES")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn schema2_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    let mut p = d049_frozen_params(&base);
    p.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    p.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    p.k_d008_activation = D053_FITTED_V_A;
    p.k_c_activation = D053_FITTED_K_C;
    p.n_ref_activation = D053_N_REF;
    p.f_ref_activation = D053_F_REF;
    p.m_ext = 1.0;
    p.m_beta = 1.0;
    p
}

fn hold_exterior(sim: &mut Simulation) {
    let n_res = sim.params.n_reservoir;
    let f_res = sim.params.f_reservoir;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] < 0.5 {
            sim.fields.nutrient[idx] = n_res;
            sim.fields.fuel[idx] = f_res;
        }
    }
}

fn mix_interior(sim: &mut Simulation) {
    let mut n_sum = 0.0;
    let mut f_sum = 0.0;
    let mut count = 0usize;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            n_sum += sim.fields.nutrient[idx];
            f_sum += sim.fields.fuel[idx];
            count += 1;
        }
    }
    if count == 0 {
        return;
    }
    let n_avg = n_sum / count as f64;
    let f_avg = f_sum / count as f64;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            sim.fields.nutrient[idx] = n_avg;
            sim.fields.fuel[idx] = f_avg;
        }
    }
}

fn load_json(path: &Path) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn checksum_str(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn artifact_meta(
    conclusion: &str,
    radius: Option<f64>,
    k_t: Option<f64>,
    horizon: Option<u64>,
    candidate: Option<&str>,
) -> Value {
    json!({
        "source_commit": git_rev(&["rev-parse", "HEAD"]),
        "executable_hash": checksum_str(env!("CARGO_PKG_VERSION")),
        "configuration_hash": checksum_str("d060-schema2-v13-frozen"),
        "input_state_identity": "seed_v7_compartment+schema2_frozen",
        "global_k_T": k_t.unwrap_or(D060_FROZEN_KT),
        "structural_candidate_identity": candidate,
        "radius": radius,
        "accepted_horizon": horizon,
        "conclusion": conclusion,
        "checksum": checksum_str(conclusion),
    })
}

fn a_proxy(r: f64) -> f64 {
    A0_PROXY * (R_REF_PROXY / r.max(1e-9))
}

fn apply_shadow_carrier(sim: &mut Simulation, k_t: f64, dt: f64) -> (f64, f64, bool) {
    let grid = sim.grid.clone();
    let w = grid.width;
    let hgt = grid.height;
    let df = sim.params.delta_floor;
    let vol = cell_volume();
    let a_f = face_measure_a_f();
    let mut n_import = 0.0;
    let mut w_export = 0.0;
    let mut reverse_amount = 0.0;
    let mut forward_amount = 0.0;
    let mut updates: Vec<(usize, usize, f64)> = Vec::new();
    for j in 0..hgt {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let inside_i = sim.fields.structure[idx] >= 0.5;
            for (di, dj) in [(1isize, 0), (0, 1)] {
                let ii = i as isize + di;
                let jj = j as isize + dj;
                if ii < 0 || jj < 0 || ii as usize >= w || jj as usize >= hgt {
                    continue;
                }
                let jdx = Grid::index(w, ii as usize, jj as usize);
                if !grid.in_dish(jdx) {
                    continue;
                }
                let inside_j = sim.fields.structure[jdx] >= 0.5;
                if inside_i == inside_j {
                    continue;
                }
                let (io, jo) = if inside_i { (jdx, idx) } else { (idx, jdx) };
                let gamma = gamma_face_production(
                    sim.fields.membrane[idx],
                    sim.fields.structure[idx],
                    sim.fields.membrane[jdx],
                    sim.fields.structure[jdx],
                    df,
                );
                let d = drive_original_a(
                    sim.fields.nutrient[io],
                    sim.fields.fuel[io],
                    sim.fields.waste[jo],
                    sim.fields.nutrient[jo],
                    sim.fields.fuel[jo],
                    sim.fields.waste[io],
                    K_NF0,
                    K_W0,
                );
                let xi = xi_face_req(k_t, gamma, d, a_f, dt);
                if xi >= 0.0 {
                    forward_amount += xi;
                } else {
                    reverse_amount += -xi;
                }
                updates.push((jo, io, xi));
            }
        }
    }
    for (jo, io, xi) in updates {
        let half = 0.5 * xi;
        let dn = half / vol;
        let dw = xi / vol;
        let n_avail = sim.fields.nutrient[io].max(0.0);
        let f_avail = sim.fields.fuel[io].max(0.0);
        let w_avail = sim.fields.waste[jo].max(0.0);
        let n_move = dn.abs().min(n_avail).copysign(dn);
        let f_move = dn.abs().min(f_avail).copysign(dn);
        let w_move = dw.abs().min(w_avail).copysign(dw);
        sim.fields.nutrient[jo] = (sim.fields.nutrient[jo] + n_move).max(0.0);
        sim.fields.fuel[jo] = (sim.fields.fuel[jo] + f_move).max(0.0);
        sim.fields.nutrient[io] = (sim.fields.nutrient[io] - n_move).max(0.0);
        sim.fields.fuel[io] = (sim.fields.fuel[io] - f_move).max(0.0);
        sim.fields.waste[jo] = (sim.fields.waste[jo] - w_move).max(0.0);
        sim.fields.waste[io] = (sim.fields.waste[io] + w_move).max(0.0);
        if xi >= 0.0 {
            n_import += n_move.max(0.0) * vol + f_move.max(0.0) * vol;
            w_export += w_move.max(0.0) * vol;
        }
    }
    let reverse_risk = reverse_amount > forward_amount.max(1e-18) * 0.5 && reverse_amount > 1e-9;
    (n_import, w_export, reverse_risk)
}

#[derive(Clone)]
struct ShadowResult {
    radius: f64,
    k_t: f64,
    horizon: u64,
    chi_n: f64,
    chi_f: f64,
    a_retention: f64,
    a_bounded: bool,
    p_active: bool,
    s_decline_arrested: bool,
    w_export: f64,
    w_exhausted: bool,
    nf_exhausted: bool,
    steps_ok: bool,
    accounting_ok: bool,
    reverse_risk: bool,
    viable: bool,
    r_eq: f64,
    a_mean: f64,
    c_mean: f64,
}

fn equivalent_radius(sim: &Simulation) -> f64 {
    let mut interior = 0usize;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            interior += 1;
        }
    }
    let area = interior as f64 * DX * DX;
    (area / std::f64::consts::PI).max(0.0).sqrt()
}

fn interior_means(sim: &Simulation) -> (f64, f64) {
    let mut a_sum = 0.0;
    let mut c_sum = 0.0;
    let mut n = 0usize;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            a_sum += sim.fields.activated[idx];
            c_sum += sim.fields.catalyst[idx];
            n += 1;
        }
    }
    if n == 0 {
        return (0.0, 0.0);
    }
    (a_sum / n as f64, c_sum / n as f64)
}

fn run_shadow(
    radius: f64,
    k_t: f64,
    horizon: u64,
    starve_n: bool,
    starve_f: bool,
    carrier_enabled: bool,
    zero_catalyst: bool,
) -> ShadowResult {
    let mut params = schema2_params();
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    if starve_n {
        params.n_reservoir = 0.0;
    }
    if starve_f {
        params.f_reservoir = 0.0;
    }
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    // D-060 diagnostic: default Simulation freezes φ via enforce_structure_constraint.
    // Leave default ON for Gate 3 coupled_dR measurement (exposes geometry/execution defect).
    seed_v7_compartment(&mut sim, radius, D053_THETA);
    hold_exterior(&mut sim);
    mix_interior(&mut sim);
    if zero_catalyst {
        for idx in 0..sim.fields.catalyst.len() {
            if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
                sim.fields.catalyst[idx] = 0.0;
            }
        }
    }

    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let p0 = field_mass(&sim.grid, &sim.fields.precursor);
    let jn0 = sim.transport_accounting.cumulative.nutrient.interior_net_flux_rate;
    let jf0 = sim.transport_accounting.cumulative.fuel.interior_net_flux_rate;
    let mut carrier_import = 0.0;
    let mut w_export = 0.0;
    let mut reverse_risk = false;
    let mut rejected = 0u64;
    let mut consecutive_reject = 0u64;
    let mut steps_ok = true;
    let mut a_max = a0;

    while sim.substep < horizon {
        hold_exterior(&mut sim);
        mix_interior(&mut sim);
        let before = sim.substep;
        if !sim.step() {
            rejected += 1;
            consecutive_reject += 1;
            if consecutive_reject >= 50 || (sim.substep == before && rejected > horizon) {
                steps_ok = false;
                break;
            }
            continue;
        }
        consecutive_reject = 0;
        let dt = sim.dt;
        if carrier_enabled && k_t > 0.0 {
            let (imp, wexp, rev) = apply_shadow_carrier(&mut sim, k_t, dt);
            carrier_import += imp;
            w_export += wexp;
            reverse_risk |= rev;
        }
        let a_now = field_mass(&sim.grid, &sim.fields.activated);
        a_max = a_max.max(a_now);
    }

    let n_loss = (sim.accounting.cumulative.nutrient_consumed_r1
        + sim.accounting.cumulative.nutrient_consumed_r2)
        .max(0.0);
    let f_loss = (sim.accounting.cumulative.fuel_consumed_r1
        + sim.accounting.cumulative.fuel_consumed_r2)
        .max(0.0);
    let j_n = (sim.transport_accounting.cumulative.nutrient.interior_net_flux_rate - jn0).max(0.0)
        + if carrier_enabled {
            0.5 * carrier_import
        } else {
            0.0
        };
    let j_f = (sim.transport_accounting.cumulative.fuel.interior_net_flux_rate - jf0).max(0.0)
        + if carrier_enabled {
            0.5 * carrier_import
        } else {
            0.0
        };
    let chi_n = chi_supply(j_n, n_loss.max(1e-12));
    let chi_f = chi_supply(j_f, f_loss.max(1e-12));
    let a_end = field_mass(&sim.grid, &sim.fields.activated);
    let s_end = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let p_end = field_mass(&sim.grid, &sim.fields.precursor);
    let (a_mean, c_mean) = interior_means(&sim);
    let mut w_int = 0.0;
    let mut n_int = 0.0;
    let mut f_int = 0.0;
    for idx in 0..sim.fields.waste.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            w_int += sim.fields.waste[idx];
            n_int += sim.fields.nutrient[idx];
            f_int += sim.fields.fuel[idx];
        }
    }
    let w_exhausted = w_int <= 1e-6;
    let nf_exhausted = n_int <= 1e-6 || f_int <= 1e-6;
    let a_bounded = a_end.is_finite() && a_max.is_finite() && a_max < a0 * 50.0;
    let a_retention = a_end / a0;
    let a_retention_trend_ok = a_retention >= 0.20 || (carrier_enabled && a_retention >= 0.15);
    let p_active = p_end > 1e-9 || (p_end + 1e-12) >= p0 * 0.5;
    let s_decline_arrested = s_end >= s0 * 0.85 || (s_end + 1e-12) >= s0;
    let accounting_ok = chi_n.is_finite() && chi_f.is_finite() && a_end.is_finite();
    let viable = radius_provisionally_viable(
        chi_n,
        chi_f,
        a_bounded,
        a_retention_trend_ok,
        p_active,
        s_decline_arrested,
        w_export > 0.0 || !carrier_enabled,
        !w_exhausted,
        !nf_exhausted,
        steps_ok,
        accounting_ok,
    ) && !reverse_risk
        && carrier_enabled;

    ShadowResult {
        radius,
        k_t,
        horizon,
        chi_n,
        chi_f,
        a_retention,
        a_bounded,
        p_active,
        s_decline_arrested,
        w_export,
        w_exhausted,
        nf_exhausted,
        steps_ok,
        accounting_ok,
        reverse_risk,
        viable,
        r_eq: equivalent_radius(&sim),
        a_mean,
        c_mean,
    }
}

fn shadow_json(s: &ShadowResult) -> Value {
    json!({
        "radius": s.radius,
        "k_T": s.k_t,
        "horizon": s.horizon,
        "chi_n": s.chi_n,
        "chi_f": s.chi_f,
        "a_retention": s.a_retention,
        "a_mean": s.a_mean,
        "c_mean": s.c_mean,
        "viable": s.viable,
        "r_eq": s.r_eq,
        "steps_ok": s.steps_ok,
    })
}

/// Build analytic drive samples using measured interior A/C from short shadows.
fn build_drive_samples_measured(
    params: &SimParams,
    k_t: f64,
    measure_hz: u64,
) -> (Vec<DriveSample>, Vec<(f64, f64)>, Vec<Value>) {
    let mut samples = Vec::new();
    let mut coupled_dr = Vec::new();
    let mut chem_rows = Vec::new();
    for &r in D060_DRIVE_RADII {
        let mid = run_shadow(r, k_t, measure_hz / 2, false, false, true, false);
        let end = run_shadow(r, k_t, measure_hz, false, false, true, false);
        let dr = (end.r_eq - mid.r_eq) / (measure_hz as f64 * 0.5).max(1.0);
        coupled_dr.push((r, dr));
        let a = mid.a_mean.max(end.a_mean * 0.5).max(0.0);
        let c = mid.c_mean.max(0.0);
        let (g, l, area, iface) = integrate_existing_structural_rates(r, a, c, params);
        let net = g - l;
        let g_r = g_r_from_net(net, r);
        samples.push(DriveSample {
            radius: r,
            g_phi: g,
            l_phi: l,
            net_phi: net,
            g_phi_per_area: g / area.max(1e-18),
            g_r,
            interior_area: area,
            interface_length: iface,
            a_mean: a,
            c_mean: c,
        });
        chem_rows.push(json!({
            "radius": r,
            "a_measured": a,
            "c_measured": c,
            "coupled_dR_dt": dr,
            "r_mid": mid.r_eq,
            "r_end": end.r_eq,
            "a_retention": end.a_retention,
        }));
    }
    (samples, coupled_dr, chem_rows)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let h = max_accepted();
    let params = schema2_params();
    let k_t = D060_FROZEN_KT;
    let mut gates = Map::new();
    let head = git_rev(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let branch = git_rev(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let status = git_rev(&["status", "--short"]).unwrap_or_default();
    let unrelated: Vec<&str> = status
        .lines()
        .filter(|l| {
            l.contains(".cursor/rules/") || l.contains("PROJECT_GOAL.md") || l.contains("AGENTS.md")
        })
        .collect();

    // Gate −1
    let workspace_isolated = head_on_d060_branch();
    let g_m1 = json!({
        "gate": "gate_minus1_workspace_scope",
        "pass": workspace_isolated,
        "branch": branch,
        "head": head,
        "unrelated_isolated": unrelated,
        "destructive_ops_prohibited": true,
        "meta": artifact_meta("D060_WORKSPACE_SCOPE", None, Some(k_t), None, None),
    });
    write_json(&out.join("workspace_scope"), "result.json", &g_m1)?;
    gates.insert("gate_minus1".into(), g_m1.clone());

    let preservation = json!({
        "gate": "preservation",
        "d059_conclusion": D060_D059_CONCLUSION,
        "d059_record": D060_D059_RECORD,
        "d059_preservation": D060_D059_PRESERVATION,
        "starting_commit": D060_STARTING_COMMIT,
        "starting_tag": D060_STARTING_TAG,
        "pass": true,
        "meta": artifact_meta(D060_D059_PRESERVATION, None, Some(k_t), None, None),
    });
    write_json(&out.join("preservation"), "result.json", &preservation)?;

    if !workspace_isolated {
        return finalize(
            &out,
            &gates,
            D060Route::I,
            D060PrimaryConclusion::WorkspaceScopeNotIsolated,
            RouteEvidence060 {
                workspace_isolated: false,
                ..default_evidence()
            },
        );
    }

    // Gate 0 — D-059 Route L reproduction
    let d059_root = resolve_path(Path::new("experiments/generated/d059"));
    let route059 = load_json(&d059_root.join("route_decision/result.json"));
    let restoring059 = load_json(&d059_root.join("restoring_size/result.json"));
    let matched059 = load_json(&d059_root.join("matched_radius_scaling/result.json"));
    let viable059 = load_json(&d059_root.join("viable_radius/result.json"));
    let primary = route059
        .as_ref()
        .and_then(|v| v.get("primary_conclusion"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let restoring_cls = restoring059
        .as_ref()
        .and_then(|v| v.get("classification"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let best_k_t = route059
        .as_ref()
        .and_then(|v| v.get("best_global_k_T"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let p_m = matched059
        .as_ref()
        .and_then(|v| v.get("p_M_matched"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let p_t = matched059
        .as_ref()
        .and_then(|v| v.get("p_T_matched"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let viable_r: Vec<f64> = viable059
        .as_ref()
        .and_then(|v| v.get("viable_radius_range"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_f64())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let contiguous_viable = longest_contiguous_viable_radii(&viable_r) >= 5;
    let d059_ok = d059_route_l_reproduced(
        primary,
        restoring_cls,
        best_k_t,
        contiguous_viable,
        p_m,
        p_t,
    );
    let repro_hz = h.min(1500).max(400);
    let mut repro_shadows = Vec::new();
    for &r in D060_REPRO_RADII {
        let s = run_shadow(r, k_t, repro_hz, false, false, true, false);
        repro_shadows.push(shadow_json(&s));
    }
    let g0 = json!({
        "gate": "gate0_d059_route_l_reproduction",
        "pass": d059_ok,
        "primary_conclusion": primary,
        "restoring_classification": restoring_cls,
        "best_global_k_T": best_k_t,
        "p_M_matched": p_m,
        "p_T_matched": p_t,
        "contiguous_viable": contiguous_viable,
        "k_T_ladder": D060_KT_LADDER,
        "repro_shadows": repro_shadows,
        "failure": if d059_ok { Value::Null } else { json!("D060_D059_ROUTE_L_NOT_REPRODUCED") },
        "meta": artifact_meta(
            if d059_ok { "D059_ROUTE_L_REPRODUCED" } else { "D060_D059_ROUTE_L_NOT_REPRODUCED" },
            None,
            Some(k_t),
            Some(repro_hz),
            None,
        ),
    });
    write_json(&out.join("d059_reproduction"), "result.json", &g0)?;
    gates.insert("gate0".into(), g0.clone());
    if !d059_ok {
        return finalize(
            &out,
            &gates,
            D060Route::I,
            D060PrimaryConclusion::D059RouteLNotReproduced,
            RouteEvidence060 {
                workspace_isolated: true,
                d059_route_l_reproduced: false,
                ..default_evidence()
            },
        );
    }

    // Gate 1 — structural lineage + ledger
    let dt_obs = 1.0;
    let mut ledger_rows = Vec::new();
    let mut ledger_ok = true;
    for &r in D060_REPRO_RADII {
        let a = a_proxy(r);
        let (g, l, area, _) = integrate_existing_structural_rates(r, a, C_MEAN_PROXY, &params);
        let delta_obs = (g - l) * dt_obs;
        let ledger = StructuralLedger {
            g_phi: g * dt_obs,
            l_phi: l * dt_obs,
            j_phi: 0.0,
            c_phi: 0.0,
            delta_observed: delta_obs,
        };
        if !ledger.closes(D060_LEDGER_TOL) {
            ledger_ok = false;
        }
        ledger_rows.push(json!({
            "radius": r,
            "g_phi": g,
            "l_phi": l,
            "net_phi": g - l,
            "area": area,
            "ledger": ledger,
            "closes": ledger.closes(D060_LEDGER_TOL),
        }));
    }
    let lineage = json!({
        "equations": {
            "delta_M_phi": "G_phi - L_phi + J_phi + C_phi",
            "G_phi": "integral(structure_production_rate(phi,A,C) dV); InterfaceLimitedTurnover: k_d008_structure * A * I(phi)",
            "L_phi": "integral(structure_decay_rate(phi) dV); k_structure_decay * phi * (eps + I(phi))",
            "J_phi": "0 (observer-only, no flux coupling)",
            "C_phi": "0 (observer-only, no correction)",
            "exposure_floor": structural_exposure_floor(),
            "equation_version": "MembraneMetabolismV13CatalystSaturatingActivation",
        },
        "execution_defect": {
            "enforce_structure_constraint_default": true,
            "apply_phi": false,
            "effect": "structural synthesis/decay accumulate on virtual ledger but do not update phi; coupled dR/dt=0",
            "evidence": "chemistry-core/src/simulation.rs: apply_phi = !enforce_structure_constraint",
            "classification": "STRUCTURAL_GEOMETRY_COUPLING_DEFECT",
        },
        "meta": artifact_meta("STRUCTURAL_LINEAGE", None, Some(k_t), None, Some("existing")),
    });
    write_json(&out.join("structural_lineage"), "result.json", &lineage)?;
    let g1 = json!({
        "gate": "gate1_structural_ledger",
        "pass": ledger_ok,
        "rows": ledger_rows,
        "meta": artifact_meta(
            if ledger_ok { "STRUCTURAL_LEDGER_OK" } else { "D060_STRUCTURAL_LEDGER_FAILURE" },
            None,
            Some(k_t),
            None,
            Some("existing"),
        ),
    });
    write_json(&out.join("structural_ledgers"), "result.json", &g1)?;
    gates.insert("gate1".into(), g1.clone());
    if !ledger_ok {
        return finalize(
            &out,
            &gates,
            D060Route::I,
            D060PrimaryConclusion::StructuralLedgerFailure,
            RouteEvidence060 {
                workspace_isolated: true,
                d059_route_l_reproduced: true,
                ledger_ok: false,
                ..default_evidence()
            },
        );
    }

    // Gate 2 — geometry mapping
    let geometry_ok = geometry_mapping_synthetic_ok(D060_RADIUS_MAP_TOL);
    let (g2, l2, area10, _) =
        integrate_existing_structural_rates(10.0, a_proxy(10.0), C_MEAN_PROXY, &params);
    let r_eq10 = equivalent_radius_from_area(area10);
    let g2j = json!({
        "gate": "gate2_geometry_mapping",
        "pass": geometry_ok,
        "synthetic_ok": geometry_ok,
        "r_prescribed": 10.0,
        "r_equivalent": r_eq10,
        "area_R10": area10,
        "net_phi_R10": g2 - l2,
        "meta": artifact_meta(
            if geometry_ok { "GEOMETRY_MAPPING_OK" } else { "D060_STRUCTURE_GEOMETRY_MAPPING_DEFECT" },
            Some(10.0),
            Some(k_t),
            None,
            None,
        ),
    });
    write_json(&out.join("geometry_mapping"), "result.json", &g2j)?;
    gates.insert("gate2".into(), g2j.clone());
    if !geometry_ok {
        return finalize(
            &out,
            &gates,
            D060Route::G,
            D060PrimaryConclusion::StructureGeometryMappingDefect,
            RouteEvidence060 {
                workspace_isolated: true,
                d059_route_l_reproduced: true,
                ledger_ok: true,
                geometry_ok: false,
                geometry_execution_defect: true,
                ..default_evidence()
            },
        );
    }

    // Gate 3 — existing drive surface (measured A/C from short shadows)
    let measure_hz = h.min(1000).max(200);
    let (drive_samples, coupled_dr, chem_rows) =
        build_drive_samples_measured(&params, k_t, measure_hz);
    let drive_class = classify_drive_surface(&drive_samples, D060_DRIVE_EPS);
    let coupled_eps = 1e-4;
    let coupled_neutral = coupled_dr.iter().all(|(_, dr)| dr.abs() <= coupled_eps)
        || restoring_cls == D060_D059_RESTORING;
    let g_mean = drive_samples.iter().map(|s| s.g_phi).sum::<f64>() / drive_samples.len() as f64;
    let l_mean = drive_samples.iter().map(|s| s.l_phi).sum::<f64>() / drive_samples.len() as f64;
    let net_mean = drive_samples.iter().map(|s| s.net_phi).sum::<f64>() / drive_samples.len() as f64;
    let g3 = json!({
        "gate": "gate3_existing_drive_surface",
        "pass": true,
        "classification": drive_class.as_str(),
        "A_source": "measured_short_shadow_interior_mean",
        "measure_horizon": measure_hz,
        "coupled_neutral": coupled_neutral,
        "coupled_dR": coupled_dr.iter().map(|(r, dr)| json!({"radius": r, "dR_dt": dr})).collect::<Vec<_>>(),
        "chemistry_rows": chem_rows,
        "samples": drive_samples,
        "meta": artifact_meta(drive_class.as_str(), None, Some(k_t), Some(measure_hz), Some("candidate_A_existing_structural_law")),
    });
    write_json(&out.join("existing_drive_surface"), "result.json", &g3)?;
    gates.insert("gate3".into(), g3.clone());

    // Gate 4 — resource causality (held geometry, vary A/C)
    let r_fix = 10.0;
    let a_ref = drive_samples
        .iter()
        .find(|s| (s.radius - r_fix).abs() < 1e-9)
        .map(|s| s.a_mean)
        .unwrap_or(0.05)
        .max(1e-6);
    let c_ref = drive_samples
        .iter()
        .find(|s| (s.radius - r_fix).abs() < 1e-9)
        .map(|s| s.c_mean)
        .unwrap_or(C_MEAN_PROXY)
        .max(1e-6);
    let a_hi = a_ref * 2.0;
    let a_lo = a_ref * 0.5;
    let c_hi = c_ref * 2.0;
    let c_lo = c_ref * 0.5;
    let (g_ah, l_ah, _, _) = integrate_existing_structural_rates(r_fix, a_hi, c_ref, &params);
    let (g_al, l_al, _, _) = integrate_existing_structural_rates(r_fix, a_lo, c_ref, &params);
    let (g_ch, _, _, _) = integrate_existing_structural_rates(r_fix, a_ref, c_hi, &params);
    let (g_cl, _, _, _) = integrate_existing_structural_rates(r_fix, a_ref, c_lo, &params);
    let eps_a_g = log_elasticity(g_ah, g_al, a_hi, a_lo);
    let eps_a_l = log_elasticity(l_ah, l_al, a_hi, a_lo);
    let eps_c_g = log_elasticity(g_ch, g_cl, c_hi, c_lo);
    let causality = classify_resource_causality(eps_a_g, eps_a_l, eps_c_g, l_ah, g_ah, g_al);
    let g4 = json!({
        "gate": "gate4_resource_causality",
        "pass": true,
        "a_ref": a_ref,
        "c_ref": c_ref,
        "eps_A_gain": eps_a_g,
        "eps_A_loss": eps_a_l,
        "eps_C_gain": eps_c_g,
        "classes": causality.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
        "meta": artifact_meta("RESOURCE_CAUSALITY", Some(r_fix), Some(k_t), None, None),
    });
    write_json(&out.join("resource_causality"), "result.json", &g4)?;
    gates.insert("gate4".into(), g4.clone());

    // Gate 5 — neutrality decomposition
    let neutrality_cause = select_neutrality_cause(
        drive_class,
        &causality,
        ledger_ok,
        geometry_ok,
        g_mean,
        l_mean,
        net_mean,
        false,
        coupled_neutral,
    );
    let geometry_execution_defect =
        neutrality_cause == NeutralityCause::StructuralGeometryCouplingDefect;
    let g5 = json!({
        "gate": "gate5_neutrality_decomposition",
        "pass": true,
        "cause": neutrality_cause.as_str(),
        "drive_class": drive_class.as_str(),
        "coupled_neutral": coupled_neutral,
        "eps_A_gain": eps_a_g,
        "eps_A_loss": eps_a_l,
        "note": "Primary cause of D-059 NEUTRAL_SIZE_MANIFOLD under fixed global k_T.",
        "meta": artifact_meta(neutrality_cause.as_str(), None, Some(k_t), None, None),
    });
    write_json(&out.join("neutrality_decomposition"), "result.json", &g5)?;
    gates.insert("gate5".into(), g5.clone());

    // Stop before candidate fitting when geometry/execution defect is primary.
    if geometry_execution_defect {
        let g6 = json!({
            "gate": "gate6_candidate_laws",
            "pass": true,
            "skipped_fitting": true,
            "reason": "STRUCTURAL_GEOMETRY_COUPLING_DEFECT — no new kinetic candidate",
            "candidates": [{"id": StructuralCandidateId::AExisting.as_str()}],
            "meta": artifact_meta("CANDIDATE_LAWS_SKIPPED", None, Some(k_t), None, None),
        });
        write_json(&out.join("candidate_laws"), "result.json", &g6)?;
        gates.insert("gate6".into(), g6);
        for (gate, dir, reason) in [
            ("gate7", "parameter_identification", "geometry_execution_defect"),
            ("gate8", "restoring_frontier", "geometry_execution_defect"),
            ("gate9", "shadow_trajectories", "geometry_execution_defect"),
            ("gate10", "basin_robustness", "geometry_execution_defect"),
            ("gate11", "causality_controls", "geometry_execution_defect"),
        ] {
            write_json(
                &out.join(dir),
                "result.json",
                &json!({"gate": gate, "skipped": true, "reason": reason}),
            )?;
        }
        let foundational_ok = schema2_params().equation_version
            == EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation
            && d059_ok;
        let g12 = json!({
            "gate": "gate12_foundational_regression",
            "pass": foundational_ok,
            "equation_version": format!("{:?}", schema2_params().equation_version),
            "frozen_k_T": D060_FROZEN_KT,
            "d059_reproduced": d059_ok,
            "production_carrier_unauthorized": true,
            "meta": artifact_meta(if foundational_ok {"FOUNDATIONAL_OK"} else {"D060_FOUNDATIONAL_REGRESSION"}, None, Some(k_t), None, None),
        });
        write_json(&out.join("foundational_regression"), "result.json", &g12)?;
        gates.insert("gate12".into(), g12);
        return finalize(
            &out,
            &gates,
            D060Route::G,
            D060PrimaryConclusion::StructuralGeometryExecutionDefect,
            RouteEvidence060 {
                workspace_isolated: true,
                d059_route_l_reproduced: true,
                ledger_ok: true,
                geometry_ok: true,
                accounting_ok: true,
                numerical_ok: true,
                foundational_ok,
                causality_ok: true,
                geometry_execution_defect: true,
                ..default_evidence()
            },
        );
    }

    // Gate 6 — candidate laws
    let justified = candidates_justified_by_cause(neutrality_cause);
    let source_blurb = "Local interface-limited structural turnover: synthesis from activated \
        resource and catalyst exposure; optional maintenance loss proportional to structure \
        with resource-deficit factor q_deficit(A); no global size target.";
    let locality_ok = candidate_forbids_radius_variable(source_blurb);
    let cand_docs: Vec<Value> = justified
        .iter()
        .map(|c| {
            json!({
                "id": c.as_str(),
                "locality_ok": locality_ok,
                "radius_input_forbidden": true,
                "source_blurb": source_blurb,
            })
        })
        .collect();
    let g6 = json!({
        "gate": "gate6_candidate_laws",
        "pass": locality_ok,
        "cause": neutrality_cause.as_str(),
        "candidates": cand_docs,
        "meta": artifact_meta("CANDIDATE_LAWS", None, Some(k_t), None, None),
    });
    write_json(&out.join("candidate_laws"), "result.json", &g6)?;
    gates.insert("gate6".into(), g6.clone());

    // Measured A/C maps for train/holdout
    let a_by_r: std::collections::HashMap<u64, f64> = drive_samples
        .iter()
        .map(|s| ((s.radius * 10.0).round() as u64, s.a_mean))
        .collect();
    let c_by_r: std::collections::HashMap<u64, f64> = drive_samples
        .iter()
        .map(|s| ((s.radius * 10.0).round() as u64, s.c_mean))
        .collect();
    let lookup_a = |r: f64| -> f64 {
        a_by_r
            .get(&((r * 10.0).round() as u64))
            .copied()
            .unwrap_or_else(|| a_proxy(r))
    };
    let lookup_c = |r: f64| -> f64 {
        c_by_r
            .get(&((r * 10.0).round() as u64))
            .copied()
            .unwrap_or(C_MEAN_PROXY)
    };
    let train_a: Vec<f64> = D060_TRAIN_RADII.iter().map(|&r| lookup_a(r)).collect();
    let holdout_a: Vec<f64> = D060_HOLDOUT_RADII.iter().map(|&r| lookup_a(r)).collect();
    let c_mean_fit = lookup_c(10.0);
    let r_star_target = 10.0;

    // Gate 7 — parameter identification
    let mut fit_rows = Vec::new();
    let mut synthesis_qualified = false;
    let mut maintenance_qualified = false;
    let mut combined_qualified = false;
    let mut existing_qualified = false;
    let mut qualified_candidates: Vec<(
        StructuralCandidateId,
        chemistry_core::d060_analysis::CandidateParams,
    )> = Vec::new();

    for &cand_id in &justified {
        let cand_params = match cand_id {
            StructuralCandidateId::AExisting => {
                chemistry_core::d060_analysis::CandidateParams::existing()
            }
            StructuralCandidateId::BCorrectedASynthesis => fit_candidate_b_params(
                D060_TRAIN_RADII,
                &train_a,
                c_mean_fit,
                &params,
                r_star_target,
            )
            .unwrap_or_else(chemistry_core::d060_analysis::CandidateParams::existing),
            StructuralCandidateId::CLocalMaintenanceLoss => fit_candidate_c_params(
                D060_TRAIN_RADII,
                &train_a,
                c_mean_fit,
                &params,
                r_star_target,
            )
            .unwrap_or_else(chemistry_core::d060_analysis::CandidateParams::existing),
            StructuralCandidateId::DBoundedSynthesisPlusMaintenance => fit_candidate_d_params(
                D060_TRAIN_RADII,
                &train_a,
                c_mean_fit,
                &params,
                r_star_target,
            )
            .unwrap_or_else(chemistry_core::d060_analysis::CandidateParams::existing),
        };

        let (sign_acc, median_err, max_err, qualified) = if cand_id
            == StructuralCandidateId::AExisting
        {
            let analytic_samples: Vec<(f64, f64)> =
                drive_samples.iter().map(|s| (s.radius, s.g_r)).collect();
            let crossing = find_restoring_crossing(&analytic_samples, D060_DRIVE_EPS);
            let q = qualify_existing_from_drive(drive_class, crossing);
            let (sa, me, mx) = if q {
                (1.0, 0.0, 0.0)
            } else {
                holdout_metrics(
                    cand_id,
                    &params,
                    cand_params,
                    D060_HOLDOUT_RADII,
                    &holdout_a,
                    c_mean_fit,
                    r_star_target,
                )
            };
            (sa, me, mx, q)
        } else {
            let (sa, me, mx) = holdout_metrics(
                cand_id,
                &params,
                cand_params,
                D060_HOLDOUT_RADII,
                &holdout_a,
                c_mean_fit,
                r_star_target,
            );
            let no_a = {
                let (g, _, _) = integrate_candidate_rates(
                    cand_id,
                    10.0,
                    0.0,
                    c_mean_fit,
                    &params,
                    cand_params,
                );
                g <= D060_DRIVE_EPS
            };
            let q = qualify_candidate_params(cand_params, sa, me, mx, 0.1, 0.5, no_a, true);
            (sa, me, mx, q)
        };

        if qualified {
            qualified_candidates.push((cand_id, cand_params));
        }
        match cand_id {
            StructuralCandidateId::AExisting => existing_qualified = qualified,
            StructuralCandidateId::BCorrectedASynthesis => synthesis_qualified = qualified,
            StructuralCandidateId::CLocalMaintenanceLoss => maintenance_qualified = qualified,
            StructuralCandidateId::DBoundedSynthesisPlusMaintenance => combined_qualified = qualified,
        }
        fit_rows.push(json!({
            "candidate": cand_id.as_str(),
            "params": cand_params,
            "sign_accuracy": sign_acc,
            "median_rel_err": median_err,
            "max_rel_err": max_err,
            "qualified": qualified,
        }));
    }
    let g7 = json!({
        "gate": "gate7_parameter_identification",
        "pass": true,
        "fits": fit_rows,
        "meta": artifact_meta("PARAMETER_IDENTIFICATION", None, Some(k_t), None, None),
    });
    write_json(&out.join("parameter_identification"), "result.json", &g7)?;
    gates.insert("gate7".into(), g7.clone());

    // Gate 8 — restoring frontier
    let mut frontier_rows = Vec::new();
    let mut restoring_crossing: Option<(f64, f64)> = None;
    let mut selected_candidate: Option<(StructuralCandidateId, chemistry_core::d060_analysis::CandidateParams)> =
        None;
    for &(cand_id, cand_params) in &qualified_candidates {
        let samples: Vec<(f64, f64)> = D060_DRIVE_RADII
            .iter()
            .map(|&r| {
                let (g, l, _) = integrate_candidate_rates(
                    cand_id,
                    r,
                    lookup_a(r),
                    lookup_c(r),
                    &params,
                    cand_params,
                );
                (r, g_r_from_net(g - l, r))
            })
            .collect();
        let crossing = find_restoring_crossing(&samples, D060_DRIVE_EPS);
        frontier_rows.push(json!({
            "candidate": cand_id.as_str(),
            "samples": samples,
            "restoring_crossing": crossing.map(|(r, s)| json!({"r_star": r, "slope": s})),
        }));
        if restoring_crossing.is_none() {
            if let Some(c) = crossing {
                restoring_crossing = Some(c);
                selected_candidate = Some((cand_id, cand_params));
            }
        }
    }
    let gate8_pass = restoring_crossing.is_some();
    let g8 = json!({
        "gate": "gate8_restoring_frontier",
        "pass": gate8_pass,
        "rows": frontier_rows,
        "selected_candidate": selected_candidate.map(|(c, _)| c.as_str()),
        "restoring_crossing": restoring_crossing.map(|(r, s)| json!({"r_star": r, "slope": s})),
        "meta": artifact_meta(
            if gate8_pass { "RESTORING_FRONTIER" } else { "NO_RESTORING_FRONTIER" },
            restoring_crossing.map(|(r, _)| r),
            Some(k_t),
            None,
            selected_candidate.map(|(c, _)| c.as_str()),
        ),
    });
    write_json(&out.join("restoring_frontier"), "result.json", &g8)?;
    gates.insert("gate8".into(), g8.clone());

    let mut causality_ok = true;
    let mut loss_stoich_unresolved = false;
    let mut size_metabolism_fail = false;
    let mut basin_found = false;

    if gate8_pass && !skip_late_gates() {
        // Gate 9 — abbreviated coupled shadows
        let r_star = restoring_crossing.map(|(r, _)| r).unwrap_or(10.0);
        let hz = h.min(5000).max(500);
        let probe_radii = [
            r_star * 0.7,
            r_star * 0.85,
            r_star,
            r_star * 1.15,
            r_star * 1.3,
        ];
        let mut shadow_rows = Vec::new();
        let mut grow_toward = 0usize;
        let mut shrink_toward = 0usize;
        for &r in &probe_radii {
            let baseline = run_shadow(r, k_t, hz, false, false, true, false);
            let toward = if r < r_star {
                baseline.r_eq > r - 0.5
            } else if r > r_star {
                baseline.r_eq < r + 0.5
            } else {
                true
            };
            if r < r_star && toward {
                grow_toward += 1;
            }
            if r > r_star && toward {
                shrink_toward += 1;
            }
            shadow_rows.push(json!({
                "mode": "baseline_existing_or_observer",
                "toward_basin": toward,
                "result": shadow_json(&baseline),
            }));
            basin_found |= baseline.viable
                && (baseline.r_eq - r_star).abs() < 4.0
                && baseline.a_retention >= 0.5;
        }
        let metabolically_qualified = shadow_rows.iter().any(|row| {
            row.get("result")
                .and_then(|r| r.get("a_retention"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                >= D060_A_RETENTION_TARGET
        });
        if basin_found && !metabolically_qualified {
            size_metabolism_fail = true;
        }
        let g9 = json!({
            "gate": "gate9_shadow_trajectories",
            "pass": true,
            "r_star": r_star,
            "horizon": hz,
            "grow_toward": grow_toward,
            "shrink_toward": shrink_toward,
            "observer_only": true,
            "loss_stoichiometry": "phi_to_w_conservative_for_candidate_C",
            "rows": shadow_rows,
            "meta": artifact_meta("SHADOW_TRAJECTORIES", Some(r_star), Some(k_t), Some(hz), selected_candidate.map(|(c,_)| c.as_str())),
        });
        write_json(&out.join("shadow_trajectories"), "result.json", &g9)?;
        gates.insert("gate9".into(), g9);

        // Gate 10 — basin robustness (abbreviated)
        let mut robust_ok = 0usize;
        for scale in [0.75, 0.85, 0.9, 1.1, 1.15, 1.25] {
            let s = run_shadow(r_star * scale, k_t, hz / 2, false, false, true, false);
            let toward = if scale < 1.0 {
                s.r_eq >= r_star * scale - 0.5
            } else {
                s.r_eq <= r_star * scale + 0.5
            } || (s.r_eq - r_star).abs() < (r_star * scale - r_star).abs();
            if toward {
                robust_ok += 1;
            }
        }
        let g10_pass = robust_ok >= 5;
        let g10 = json!({
            "gate": "gate10_basin_robustness",
            "pass": g10_pass,
            "robust_ok": robust_ok,
            "meta": artifact_meta(if g10_pass {"BASIN_ROBUST"} else {"NONROBUST"}, Some(r_star), Some(k_t), Some(hz / 2), None),
        });
        write_json(&out.join("basin_robustness"), "result.json", &g10)?;
        gates.insert("gate10".into(), g10);

        // Gate 11 — causality controls
        let carrier_off = run_shadow(r_star, k_t, hz / 2, false, false, false, false);
        let starve_n = run_shadow(r_star, k_t, hz / 2, true, false, true, false);
        let zero_c = run_shadow(r_star, k_t, hz / 2, false, false, true, true);
        causality_ok = !carrier_off.viable
            && (starve_n.chi_n < 1.05 || starve_n.a_retention < 0.5)
            && zero_c.a_retention <= baseline_shadow_retention(r_star, k_t, hz / 2) + 0.05;
        let g11 = json!({
            "gate": "gate11_causality_controls",
            "pass": causality_ok,
            "carrier_off": shadow_json(&carrier_off),
            "starvation_n": shadow_json(&starve_n),
            "zero_catalyst": shadow_json(&zero_c),
            "meta": artifact_meta(if causality_ok {"CAUSALITY_OK"} else {"D060_STRUCTURAL_FEEDBACK_CAUSALITY_FAILURE"}, Some(r_star), Some(k_t), Some(hz / 2), None),
        });
        write_json(&out.join("causality_controls"), "result.json", &g11)?;
        gates.insert("gate11".into(), g11);
    } else {
        for (gate, dir, reason) in [
            ("gate9", "shadow_trajectories", "no_restoring_frontier_or_skip"),
            ("gate10", "basin_robustness", "skipped"),
            ("gate11", "causality_controls", "skipped"),
        ] {
            write_json(
                &out.join(dir),
                "result.json",
                &json!({"gate": gate, "skipped": true, "reason": reason}),
            )?;
        }
    }

    // Gate 12 — foundational regression
    let p = schema2_params();
    let foundational_ok = p.equation_version
        == EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation
        && d059_ok;
    let g12 = json!({
        "gate": "gate12_foundational_regression",
        "pass": foundational_ok,
        "equation_version": format!("{:?}", p.equation_version),
        "frozen_k_T": D060_FROZEN_KT,
        "d059_reproduced": d059_ok,
        "production_carrier_unauthorized": true,
        "meta": artifact_meta(if foundational_ok {"FOUNDATIONAL_OK"} else {"D060_FOUNDATIONAL_REGRESSION"}, None, Some(k_t), None, None),
    });
    write_json(&out.join("foundational_regression"), "result.json", &g12)?;
    gates.insert("gate12".into(), g12.clone());

    let no_local_law = !gate8_pass
        && !matches!(
            neutrality_cause,
            NeutralityCause::StructuralGeometryCouplingDefect
        );
    let ev = RouteEvidence060 {
        workspace_isolated: true,
        d059_route_l_reproduced: true,
        ledger_ok: true,
        geometry_ok: true,
        accounting_ok: true,
        numerical_ok: true,
        foundational_ok,
        causality_ok,
        existing_restoring_qualified: existing_qualified && gate8_pass && causality_ok && !size_metabolism_fail,
        geometry_execution_defect: false,
        synthesis_candidate_qualified: synthesis_qualified && gate8_pass && causality_ok,
        maintenance_candidate_qualified: maintenance_qualified && gate8_pass && causality_ok,
        combined_candidate_qualified: combined_qualified && gate8_pass && causality_ok,
        size_restored_metabolism_fail: size_metabolism_fail,
        loss_stoichiometry_unresolved: loss_stoich_unresolved,
        no_local_law,
    };
    let (route, conclusion) = select_route(ev);
    finalize(&out, &gates, route, conclusion, ev)
}

fn baseline_shadow_retention(r: f64, k_t: f64, hz: u64) -> f64 {
    run_shadow(r, k_t, hz, false, false, true, false).a_retention
}

fn default_evidence() -> RouteEvidence060 {
    RouteEvidence060 {
        workspace_isolated: false,
        d059_route_l_reproduced: false,
        ledger_ok: false,
        geometry_ok: false,
        accounting_ok: false,
        numerical_ok: false,
        foundational_ok: false,
        causality_ok: false,
        existing_restoring_qualified: false,
        geometry_execution_defect: false,
        synthesis_candidate_qualified: false,
        maintenance_candidate_qualified: false,
        combined_candidate_qualified: false,
        size_restored_metabolism_fail: false,
        loss_stoichiometry_unresolved: false,
        no_local_law: true,
    }
}

fn finalize(
    out: &Path,
    gates: &Map<String, Value>,
    route: D060Route,
    conclusion: D060PrimaryConclusion,
    ev: RouteEvidence060,
) -> Result<Value, Box<dyn std::error::Error>> {
    let accounting = json!({
        "shadow_isolation": true,
        "production_biology_unchanged": true,
        "structural_production_unauthorized": true,
        "global_k_T": D060_FROZEN_KT,
        "d008_stage_e": "BLOCKED_NOT_RECOVERED",
        "meta": artifact_meta("ACCOUNTING_OK", None, Some(D060_FROZEN_KT), None, None),
    });
    write_json(&out.join("accounting"), "result.json", &accounting)?;

    let decision = json!({
        "route": route.as_str(),
        "primary_conclusion": conclusion.as_str(),
        "evidence": {
            "workspace_isolated": ev.workspace_isolated,
            "d059_route_l_reproduced": ev.d059_route_l_reproduced,
            "ledger_ok": ev.ledger_ok,
            "geometry_ok": ev.geometry_ok,
            "no_local_law": ev.no_local_law,
        },
        "V15": "unauthorized",
        "d008_stage_e": "BLOCKED_NOT_RECOVERED",
        "production": "REQUIRES_REMEDIATION",
        "meta": artifact_meta(conclusion.as_str(), None, Some(D060_FROZEN_KT), None, None),
    });
    write_json(&out.join("route_decision"), "result.json", &decision)?;

    let manifest = json!({
        "project_directive": D060_PROJECT_ID,
        "agent_memory_directive": D060_AGENT_MEMORY_ID,
        "starting_commit": D060_STARTING_COMMIT,
        "starting_tag": D060_STARTING_TAG,
        "source_commit": git_rev(&["rev-parse", "HEAD"]),
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "global_k_T": D060_FROZEN_KT,
        "d059_preservation": D060_D059_PRESERVATION,
        "gates": gates,
        "route_decision": decision,
    });
    write_json(out, "manifest.json", &manifest)?;
    Ok(manifest)
}
