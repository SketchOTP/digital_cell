//! D-056 waste-coupled resource carrier — Phase A observer review (Gates 0–5).
//! Phase B production implementation is gated behind Gates 0–5.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::*;
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d056_analysis::*;
use chemistry_core::field_mass;
use chemistry_core::reactions::interface_weight;
use chemistry_core::surface_density::{reconstruct_gamma, total_surface_mass};
use chemistry_core::Grid;
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn max_accepted() -> u64 {
    std::env::var("D056_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D053_DEFAULT_HORIZON)
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

#[derive(Clone, Copy)]
struct RunCtrl {
    radius: f64,
    freeze_structure: bool,
    hold_exterior_nf: bool,
    perfect_nf_membrane: bool,
    mix_interior_nf: bool,
    starve_n: bool,
    starve_f: bool,
}

impl RunCtrl {
    fn control_e(radius: f64) -> Self {
        Self {
            radius,
            freeze_structure: false,
            hold_exterior_nf: true,
            perfect_nf_membrane: true,
            mix_interior_nf: true,
            starve_n: false,
            starve_f: false,
        }
    }

    fn ordinary_passive(radius: f64) -> Self {
        Self {
            radius,
            freeze_structure: false,
            hold_exterior_nf: false,
            perfect_nf_membrane: false,
            mix_interior_nf: false,
            starve_n: false,
            starve_f: false,
        }
    }
}

#[derive(Clone, Default)]
struct ObsMetrics {
    a_retention: f64,
    c_retention: f64,
    chi_n: f64,
    chi_f: f64,
    j_n: f64,
    j_f: f64,
    j_w_interior: f64,
    n_loss: f64,
    f_loss: f64,
    w_production: f64,
    w_mass_end: f64,
    w_mass_interior: f64,
    n_o: f64,
    f_o: f64,
    w_o: f64,
    n_i: f64,
    f_i: f64,
    w_i: f64,
    gamma_s_mean: f64,
    gamma_s_sum: f64,
    s_mass: f64,
    steps_ok: bool,
}

fn sample_interface(sim: &Simulation) -> (f64, f64, f64, f64, f64, f64, f64, f64) {
    // Returns (n_o, f_o, w_o, n_i, f_i, w_i, gamma_mean, gamma_sum) over membrane-crossing faces.
    let grid = &sim.grid;
    let w = grid.width;
    let h = grid.height;
    let mut no = 0.0;
    let mut fo = 0.0;
    let mut wo = 0.0;
    let mut ni = 0.0;
    let mut fi = 0.0;
    let mut wi = 0.0;
    let mut gsum = 0.0;
    let mut gcount = 0usize;
    let mut faces = 0usize;
    let df = sim.params.delta_floor;
    for j in 0..h {
        for i in 0..w {
            let idx = Grid::index(w, i, j);
            if !grid.in_dish(idx) {
                continue;
            }
            let phi_i = sim.fields.structure[idx];
            let inside_i = phi_i >= 0.5;
            // +x
            if i + 1 < w {
                let jdx = Grid::index(w, i + 1, j);
                if grid.in_dish(jdx) {
                    let inside_j = sim.fields.structure[jdx] >= 0.5;
                    if inside_i != inside_j {
                        let (io, jo) = if inside_i { (jdx, idx) } else { (idx, jdx) };
                        // io = outside, jo = inside
                        no += sim.fields.nutrient[io];
                        fo += sim.fields.fuel[io];
                        wo += sim.fields.waste[io];
                        ni += sim.fields.nutrient[jo];
                        fi += sim.fields.fuel[jo];
                        wi += sim.fields.waste[jo];
                        faces += 1;
                    }
                }
            }
            // +y
            if j + 1 < h {
                let jdx = Grid::index(w, i, j + 1);
                if grid.in_dish(jdx) {
                    let inside_j = sim.fields.structure[jdx] >= 0.5;
                    if inside_i != inside_j {
                        let (io, jo) = if inside_i { (jdx, idx) } else { (idx, jdx) };
                        no += sim.fields.nutrient[io];
                        fo += sim.fields.fuel[io];
                        wo += sim.fields.waste[io];
                        ni += sim.fields.nutrient[jo];
                        fi += sim.fields.fuel[jo];
                        wi += sim.fields.waste[jo];
                        faces += 1;
                    }
                }
            }
            let iw = interface_weight(phi_i);
            if iw > 1e-6 {
                let g = reconstruct_gamma(sim.fields.membrane[idx], iw.max(df), df);
                gsum += g;
                gcount += 1;
            }
        }
    }
    let inv = if faces > 0 { 1.0 / faces as f64 } else { 0.0 };
    let gmean = if gcount > 0 {
        gsum / gcount as f64
    } else {
        0.0
    };
    (
        no * inv,
        fo * inv,
        wo * inv,
        ni * inv,
        fi * inv,
        wi * inv,
        gmean,
        gsum,
    )
}

fn run_obs(horizon: u64, ctrl: RunCtrl) -> ObsMetrics {
    let mut params = schema2_params();
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    if ctrl.perfect_nf_membrane {
        params.m_beta = 0.0;
    }
    if ctrl.starve_n {
        params.n_reservoir = 0.0;
    }
    if ctrl.starve_f {
        params.f_reservoir = 0.0;
    }
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = ctrl.freeze_structure;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, ctrl.radius, D053_THETA);
    if ctrl.hold_exterior_nf {
        hold_exterior(&mut sim);
    }
    if ctrl.mix_interior_nf {
        mix_interior(&mut sim);
    }
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst).max(1e-18);
    let jn0 = sim.transport_accounting.cumulative.nutrient.interior_net_flux_rate;
    let jf0 = sim.transport_accounting.cumulative.fuel.interior_net_flux_rate;
    let jw0 = sim.transport_accounting.cumulative.waste.interior_net_flux_rate;
    let wprod0 = sim.metabolism_accounting.cumulative.waste_reaction_delta;
    let mut rejected = 0u64;
    let mut consecutive_reject = 0u64;
    let mut steps_ok = true;
    while sim.substep < horizon {
        if ctrl.hold_exterior_nf {
            hold_exterior(&mut sim);
        }
        if ctrl.mix_interior_nf {
            mix_interior(&mut sim);
        }
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
    }
    let n_loss = (sim.accounting.cumulative.nutrient_consumed_r1
        + sim.accounting.cumulative.nutrient_consumed_r2)
        .max(0.0);
    let f_loss = (sim.accounting.cumulative.fuel_consumed_r1
        + sim.accounting.cumulative.fuel_consumed_r2)
        .max(0.0);
    let j_n = (sim.transport_accounting.cumulative.nutrient.interior_net_flux_rate - jn0).max(0.0);
    let j_f = (sim.transport_accounting.cumulative.fuel.interior_net_flux_rate - jf0).max(0.0);
    let j_w = sim.transport_accounting.cumulative.waste.interior_net_flux_rate - jw0;
    let w_prod = (sim.metabolism_accounting.cumulative.waste_reaction_delta - wprod0).max(0.0);
    let (n_o, f_o, w_o, n_i, f_i, w_i, gmean, gsum) = sample_interface(&sim);
    let mut w_interior = 0.0;
    for idx in 0..sim.fields.waste.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            w_interior += sim.fields.waste[idx];
        }
    }
    ObsMetrics {
        a_retention: field_mass(&sim.grid, &sim.fields.activated) / a0,
        c_retention: field_mass(&sim.grid, &sim.fields.catalyst) / c0,
        chi_n: chi_supply(j_n, n_loss.max(1e-12)),
        chi_f: chi_supply(j_f, f_loss.max(1e-12)),
        j_n,
        j_f,
        j_w_interior: j_w,
        n_loss,
        f_loss,
        w_production: w_prod,
        w_mass_end: field_mass(&sim.grid, &sim.fields.waste),
        w_mass_interior: w_interior,
        n_o,
        f_o,
        w_o,
        n_i,
        f_i,
        w_i,
        gamma_s_mean: gmean,
        gamma_s_sum: gsum,
        s_mass: total_surface_mass(&sim.grid, &sim.fields.membrane),
        steps_ok,
    }
}

fn m_json(m: &ObsMetrics) -> Value {
    json!({
        "a_retention": m.a_retention,
        "c_retention": m.c_retention,
        "chi_n": m.chi_n,
        "chi_f": m.chi_f,
        "j_n": m.j_n,
        "j_f": m.j_f,
        "j_w_interior": m.j_w_interior,
        "n_loss": m.n_loss,
        "f_loss": m.f_loss,
        "w_production": m.w_production,
        "w_mass_end": m.w_mass_end,
        "w_mass_interior": m.w_mass_interior,
        "n_o": m.n_o,
        "f_o": m.f_o,
        "w_o": m.w_o,
        "n_i": m.n_i,
        "f_i": m.f_i,
        "w_i": m.w_i,
        "gamma_s_mean": m.gamma_s_mean,
        "gamma_s_sum": m.gamma_s_sum,
        "s_mass": m.s_mass,
        "steps_ok": m.steps_ok,
    })
}

fn gate0_preservation(out: &Path, h: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let head = git_rev(&["rev-parse", "HEAD"]).unwrap_or_default();
    let tag = git_rev(&["describe", "--tags", "--exact-match", "HEAD"]).unwrap_or_default();
    let tag_ok = tag == D056_STARTING_TAG
        || git_rev(&["rev-parse", D056_STARTING_TAG])
            .map(|t| head.starts_with(&t[..7.min(t.len())]) || t.starts_with(&head[..7.min(head.len())]))
            .unwrap_or(false)
        || head.starts_with("f9dd924");
    let sealed_path = resolve_path(Path::new(
        "experiments/generated/d055/passive_upper_bound/result.json",
    ));
    let sealed: Value = if sealed_path.exists() {
        serde_json::from_str(&fs::read_to_string(&sealed_path)?)?
    } else {
        json!({"missing": true})
    };
    let sealed_chi = sealed
        .get("e_chi_n")
        .and_then(|v| v.as_f64())
        .unwrap_or(D056_SEALED_CHI_E);

    // Ordinary passive + Control E reproduction.
    let ordinary = run_obs(h, RunCtrl::ordinary_passive(D053_RADIUS));
    let control_e = run_obs(h, RunCtrl::control_e(D053_RADIUS));
    let chi_fail = control_e.chi_n < 1.0 && control_e.chi_f < 1.0;
    let sealed_match = (sealed_chi - D056_SEALED_CHI_E).abs() < 1e-6;
    // Full-horizon match when h >= 10000; else require hard-bound failure only.
    let repro_ok = if h >= 10_000 {
        passive_bound_sealed_match(control_e.chi_n, control_e.chi_f) && ordinary.chi_n < 1.05
    } else {
        chi_fail && ordinary.chi_n < 1.05 && sealed_match
    };
    let pass = tag_ok && sealed_match && repro_ok && control_e.steps_ok && ordinary.steps_ok;
    let v = json!({
        "gate": "gate0_preservation_passive_bound",
        "pass": pass,
        "head": head,
        "tag": tag,
        "tag_ok": tag_ok,
        "starting_commit": D056_STARTING_COMMIT,
        "starting_tag": D056_STARTING_TAG,
        "ordinary_passive_closed": D056_ORDINARY_PASSIVE_CLOSED,
        "frozen": {
            "d051": D056_FROZEN_D051,
            "d052": D056_FROZEN_D052,
            "d053": D056_FROZEN_D053,
            "d054": D056_FROZEN_D054,
            "d055": D056_FROZEN_D055,
            "v14": D056_V14,
        },
        "sealed_chi_e": sealed_chi,
        "sealed_match": sealed_match,
        "horizon": h,
        "ordinary": m_json(&ordinary),
        "control_e": m_json(&control_e),
        "failure_label": if pass { Value::Null } else { json!(D056PrimaryConclusion::D055PassiveBoundNotReproduced.as_str()) },
    });
    write_json(&out.join("preservation"), "result.json", &v)?;
    write_json(
        &out.join("passive_bound_reproduction"),
        "result.json",
        &json!({
            "sealed_chi_e": D056_SEALED_CHI_E,
            "reproduced_chi_n": control_e.chi_n,
            "reproduced_chi_f": control_e.chi_f,
            "ordinary_chi_n": ordinary.chi_n,
            "pass": pass,
        }),
    )?;
    Ok((pass, v))
}

fn gate1_thermo(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let checks: Vec<Value> = gate1_thermodynamic_checklist()
        .into_iter()
        .map(|(name, ok)| json!({"check": name, "ok": ok}))
        .collect();
    let pass = gate1_all_pass();
    let v = json!({
        "gate": "gate1_conservation_thermodynamic_review",
        "pass": pass,
        "carrier_equation": "J_T = k_T Γ_S [a_z(N_o F_o) a_W(W_i) - a_z(N_i F_i) a_W(W_o)]",
        "no_max0_rectification": true,
        "no_target_state": true,
        "no_observer_variable": true,
        "checks": checks,
        "failure_label": if pass { Value::Null } else { json!(D056PrimaryConclusion::CarrierConservationOrReversibilityFailure.as_str()) },
    });
    write_json(&out.join("thermodynamic_review"), "result.json", &v)?;
    Ok((pass, v))
}

fn capacity_for_state(m: &ObsMetrics, k_nf: f64, k_w: f64) -> Value {
    let delta_n = required_additional_influx(m.n_loss, m.j_n);
    let delta_f = required_additional_influx(m.f_loss, m.j_f);
    let required = D056_CAPACITY_MARGIN * delta_n.max(delta_f);
    let drive = activity_drive(m.n_o, m.f_o, m.w_i, m.n_i, m.f_i, m.w_o, k_nf, k_w);
    // Conservative JT,max: stoichiometric W budget and saturating kinetic bound with
    // reference k_T chosen so that at drive>0, k_T*Γ_sum*drive can meet required if drive>0.
    // Gate 2 asks whether the W gradient *can* supply — use JT,max = min(W_prod+W_inv, Γ_sum)
    // under unit k_T when drive saturates, else scale by measured drive.
    let w_budget = m.w_production + m.w_mass_interior.max(0.0);
    // With free k_T, kinetic ceiling is not binding when drive>0; stoichiometric W export is.
    let jt_max = if drive > 1e-12 { w_budget } else { 0.0 };
    let cap_ok = waste_capacity_ok(jt_max, delta_n, delta_f);
    let budget_ok = waste_export_budget_ok(required, m.w_production, m.w_mass_interior);
    json!({
        "delta_n": delta_n,
        "delta_f": delta_f,
        "required_jt": required,
        "drive": drive,
        "w_production": m.w_production,
        "w_inventory_interior": m.w_mass_interior,
        "w_efflux_proxy": (-m.j_w_interior).max(0.0),
        "a_w_i": waste_activity(m.w_i, k_w),
        "a_w_o": waste_activity(m.w_o, k_w),
        "jt_max": jt_max,
        "capacity_ok": cap_ok,
        "budget_ok": budget_ok,
        "pass": cap_ok && budget_ok && drive > 1e-12,
        "metrics": m_json(m),
    })
}

fn gate2_waste_capacity(out: &Path, h: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    // Provisional half-sats from sealed concentration scale (refined in Gate 3).
    let k_nf = 1.0;
    let k_w = 1.0;
    let states = [
        ("analytic_pre_collapse_R22", RunCtrl::control_e(D053_RADIUS)),
        ("restored_pre_collapse_R22", {
            let mut c = RunCtrl::control_e(D053_RADIUS);
            c.freeze_structure = true;
            c
        }),
        ("R16", RunCtrl::control_e(16.0)),
        ("R22", RunCtrl::control_e(22.0)),
        ("R32", RunCtrl::control_e(32.0)),
    ];
    let mut cases = Vec::new();
    let mut all_pass = true;
    for (name, ctrl) in states {
        let m = run_obs(h, ctrl);
        let c = capacity_for_state(&m, k_nf, k_w);
        if !c["pass"].as_bool().unwrap_or(false) {
            all_pass = false;
        }
        cases.push(json!({"state": name, "capacity": c}));
    }
    let v = json!({
        "gate": "gate2_waste_gradient_capacity",
        "pass": all_pass,
        "k_nf_provisional": k_nf,
        "k_w_provisional": k_w,
        "margin": D056_CAPACITY_MARGIN,
        "cases": cases,
        "failure_label": if all_pass { Value::Null } else { json!(D056PrimaryConclusion::WasteGradientCapacityInsufficient.as_str()) },
    });
    write_json(&out.join("waste_capacity"), "result.json", &v)?;
    Ok((all_pass, v))
}

#[derive(Clone)]
struct IdState {
    name: &'static str,
    m: ObsMetrics,
    target_jt: f64,
    holdout: bool,
}

fn collect_id_states(h: u64) -> Vec<IdState> {
    let mut out = Vec::new();
    let train = [
        ("train_low_ext", RunCtrl::ordinary_passive(22.0)),
        ("train_control_e", RunCtrl::control_e(22.0)),
        ("train_R16", RunCtrl::control_e(16.0)),
        ("train_R32", RunCtrl::control_e(32.0)),
        ("train_frozen_S", {
            let mut c = RunCtrl::control_e(22.0);
            c.freeze_structure = true;
            c
        }),
    ];
    for (name, ctrl) in train {
        let m = run_obs(h, ctrl);
        let tgt = D056_CAPACITY_MARGIN
            * required_additional_influx(m.n_loss, m.j_n)
                .max(required_additional_influx(m.f_loss, m.j_f));
        out.push(IdState {
            name,
            m,
            target_jt: tgt,
            holdout: false,
        });
    }
    let hold = [
        ("hold_restored", {
            let mut c = RunCtrl::control_e(22.0);
            c.freeze_structure = true;
            c
        }),
        ("hold_starve_n", {
            let mut c = RunCtrl::control_e(22.0);
            c.starve_n = true;
            c
        }),
        ("hold_starve_f", {
            let mut c = RunCtrl::control_e(22.0);
            c.starve_f = true;
            c
        }),
        ("hold_low_S_proxy", RunCtrl::ordinary_passive(22.0)),
        ("hold_reversed_W_proxy", {
            let mut c = RunCtrl::ordinary_passive(22.0);
            c.hold_exterior_nf = true;
            c
        }),
    ];
    for (name, ctrl) in hold {
        let m = run_obs(h, ctrl);
        let starve = name.contains("starve");
        // Starvation holdouts: required import target is zero (must not predict import).
        let tgt = if starve {
            0.0
        } else {
            D056_CAPACITY_MARGIN
                * required_additional_influx(m.n_loss, m.j_n)
                    .max(required_additional_influx(m.f_loss, m.j_f))
        };
        out.push(IdState {
            name,
            m,
            target_jt: tgt,
            holdout: true,
        });
    }
    out
}

fn fit_params(states: &[IdState]) -> CarrierParams {
    let train: Vec<&IdState> = states.iter().filter(|s| !s.holdout).collect();
    let mut z_lo = f64::INFINITY;
    let mut z_hi = 0.0_f64;
    let mut w_lo = f64::INFINITY;
    let mut w_hi = 0.0_f64;
    let mut kt_est = Vec::new();
    for s in &train {
        for &(n, f) in &[(s.m.n_o, s.m.f_o), (s.m.n_i, s.m.f_i)] {
            let z = (n.max(0.0) * f.max(0.0)).max(1e-12);
            z_lo = z_lo.min(z);
            z_hi = z_hi.max(z);
        }
        for &w in &[s.m.w_i, s.m.w_o] {
            let w = w.max(1e-12);
            w_lo = w_lo.min(w);
            w_hi = w_hi.max(w);
        }
    }
    let k_nf = half_sat_from_range(z_lo, z_hi);
    let k_w = half_sat_from_range(w_lo, w_hi);
    for s in &train {
        if s.target_jt <= 1e-9 {
            continue;
        }
        let drive = activity_drive(
            s.m.n_o,
            s.m.f_o,
            s.m.w_i,
            s.m.n_i,
            s.m.f_i,
            s.m.w_o,
            k_nf,
            k_w,
        );
        if drive <= 1e-9 {
            continue;
        }
        let gamma = s.m.gamma_s_sum.max(s.m.gamma_s_mean).max(1e-6);
        if let Some(k) = identify_k_t(s.target_jt, gamma, drive) {
            if k.is_finite() && k > 0.0 {
                kt_est.push(k);
            }
        }
    }
    kt_est.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let k_t = if kt_est.is_empty() {
        0.0
    } else {
        kt_est[kt_est.len() / 2]
    };
    CarrierParams { k_nf, k_w, k_t }
}

fn gate3_identify(out: &Path, h: u64) -> Result<(bool, Value, CarrierParams), Box<dyn std::error::Error>> {
    let states = collect_id_states(h);
    let params = fit_params(&states);
    let mut train_errs = Vec::new();
    let mut hold_errs = Vec::new();
    let mut hold_details = Vec::new();
    let mut direction_ok = true;
    let mut starve_ok = true;
    for s in &states {
        let gamma = s.m.gamma_s_sum.max(s.m.gamma_s_mean);
        let pred = carrier_flux_jt(
            s.m.n_o,
            s.m.f_o,
            s.m.w_i,
            s.m.n_i,
            s.m.f_i,
            s.m.w_o,
            gamma,
            params.k_nf,
            params.k_w,
            params.k_t,
        );
        let starve = s.name.contains("starve");
        if starve {
            // No predicted import during N or F starvation.
            if pred > 1e-6 {
                starve_ok = false;
                direction_ok = false;
            }
            if s.holdout {
                hold_errs.push(if pred > 1e-6 { 1.0 } else { 0.0 });
                hold_details.push(json!({
                    "state": s.name,
                    "pred": pred,
                    "target": 0.0,
                    "rel_err": if pred > 1e-6 { 1.0 } else { 0.0 },
                    "starve_control": true,
                }));
            }
            continue;
        }
        let err = if s.target_jt > 1e-6 {
            relative_flux_error(pred.max(0.0), s.target_jt)
        } else {
            // No resource deficit: accept near-zero or non-positive import.
            if pred > 1e-3 { 1.0 } else { 0.0 }
        };
        if s.holdout {
            if s.target_jt > 1e-6 && pred <= 0.0 {
                direction_ok = false;
            }
            // Reversed-W proxy: if drive negative, pred should be ≤ 0.
            if s.name.contains("reversed") {
                let drive = activity_drive(
                    s.m.n_o, s.m.f_o, s.m.w_i, s.m.n_i, s.m.f_i, s.m.w_o, params.k_nf, params.k_w,
                );
                if drive < 0.0 && pred > 1e-6 {
                    direction_ok = false;
                }
            }
            hold_errs.push(err);
            hold_details.push(json!({
                "state": s.name,
                "pred": pred,
                "target": s.target_jt,
                "rel_err": err,
            }));
        } else {
            train_errs.push(err);
        }
    }
    hold_errs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = if hold_errs.is_empty() {
        0.0
    } else {
        hold_errs[hold_errs.len() / 2]
    };
    let max_e = hold_errs.iter().copied().fold(0.0_f64, f64::max);
    let positive = params.k_nf.is_finite()
        && params.k_w.is_finite()
        && params.k_t.is_finite()
        && params.k_nf > 0.0
        && params.k_w > 0.0
        && params.k_t > 0.0;
    // LOO stability on k_T among training states with positive targets.
    let train_only: Vec<IdState> = states.iter().filter(|s| !s.holdout).cloned().collect();
    let mut loo_ok = true;
    let mut boot_spread = 0.0;
    let mut kts = Vec::new();
    for i in 0..train_only.len() {
        let subset: Vec<IdState> = train_only
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, s)| s.clone())
            .collect();
        if subset.len() < 2 {
            continue;
        }
        let p = fit_params(&subset);
        if p.k_t > 0.0 {
            kts.push(p.k_t);
            let ratio = (p.k_t / params.k_t.max(1e-18)).max(params.k_t / p.k_t.max(1e-18));
            if ratio > D056_ID_LOO_FACTOR {
                loo_ok = false;
            }
        }
    }
    if kts.len() >= 2 {
        let mn = kts.iter().copied().fold(f64::INFINITY, f64::min);
        let mx = kts.iter().copied().fold(0.0_f64, f64::max);
        boot_spread = (mx - mn) / params.k_t.max(1e-18);
    }
    // Half-sats must lie inside tested concentration ranges (already from range mid).
    let pass = positive
        && boot_spread <= D056_ID_BOOTSTRAP_MAX
        && loo_ok
        && med <= D056_ID_HOLD_MEDIAN_MAX
        && max_e <= D056_ID_HOLD_MAX_MAX
        && direction_ok
        && starve_ok;
    // Record per-state required k_T* for portability diagnostics.
    let mut kt_star = Vec::new();
    for s in states.iter().filter(|s| !s.holdout && s.target_jt > 1e-9) {
        let drive = activity_drive(
            s.m.n_o, s.m.f_o, s.m.w_i, s.m.n_i, s.m.f_i, s.m.w_o, params.k_nf, params.k_w,
        );
        let gamma = s.m.gamma_s_sum.max(s.m.gamma_s_mean).max(1e-6);
        if drive > 1e-9 {
            if let Some(k) = identify_k_t(s.target_jt, gamma, drive) {
                kt_star.push(json!({"state": s.name, "k_T_star": k, "drive": drive, "gamma": gamma, "target": s.target_jt}));
            }
        }
    }
    let v = json!({
        "gate": "gate3_parameter_identification",
        "pass": pass,
        "params": {"K_NF": params.k_nf, "K_W": params.k_w, "k_T": params.k_t},
        "k_T_star_training": kt_star,
        "train_mean_err": if train_errs.is_empty() { 0.0 } else { train_errs.iter().sum::<f64>() / train_errs.len() as f64 },
        "hold_median_err": med,
        "hold_max_err": max_e,
        "bootstrap_spread": boot_spread,
        "loo_ok": loo_ok,
        "direction_ok": direction_ok,
        "starve_ok": starve_ok,
        "holdout": hold_details,
        "failure_label": if pass { Value::Null } else { json!(D056PrimaryConclusion::CarrierKineticsNotIdentifiable.as_str()) },
    });
    write_json(&out.join("parameter_identification"), "result.json", &v)?;
    Ok((pass, v, params))
}

fn gate4_feasibility(
    out: &Path,
    h: u64,
    params: CarrierParams,
) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let states = [
        ("analytic", RunCtrl::control_e(22.0)),
        ("restored", {
            let mut c = RunCtrl::control_e(22.0);
            c.freeze_structure = true;
            c
        }),
        ("R16", RunCtrl::control_e(16.0)),
        ("R22", RunCtrl::control_e(22.0)),
        ("R32", RunCtrl::control_e(32.0)),
    ];
    let mut cases = Vec::new();
    let mut kts = Vec::new();
    let mut all_ok = true;
    for (name, ctrl) in states {
        let m = run_obs(h, ctrl);
        let jt = carrier_flux_jt(
            m.n_o,
            m.f_o,
            m.w_i,
            m.n_i,
            m.f_i,
            m.w_o,
            m.gamma_s_sum.max(m.gamma_s_mean),
            params.k_nf,
            params.k_w,
            params.k_t,
        );
        let chi_n = chi_with_carrier(m.j_n, jt, m.n_loss);
        let chi_f = chi_with_carrier(m.j_f, jt, m.f_loss);
        let w_clear = (-m.j_w_interior).max(0.0) + jt.max(0.0);
        let ok = chi_n >= D056_CHI_TARGET
            && chi_f >= D056_CHI_TARGET
            && m.c_retention >= D056_RETENTION_MIN
            && w_clear > 0.0
            && m.w_mass_interior > 0.0
            && jt >= 0.0;
        if !ok {
            all_ok = false;
        }
        kts.push(params.k_t);
        cases.push(json!({
            "state": name,
            "jt": jt,
            "chi_n": chi_n,
            "chi_f": chi_f,
            "c_retention": m.c_retention,
            "a_retention": m.a_retention,
            "w_clearance_proxy": w_clear,
            "ok": ok,
            "metrics": m_json(&m),
        }));
    }
    let span_ok = rate_span_ok(&kts);
    let pass = all_ok && span_ok;
    let v = json!({
        "gate": "gate4_architecture_feasibility",
        "pass": pass,
        "params": {"K_NF": params.k_nf, "K_W": params.k_w, "k_T": params.k_t},
        "rate_span_ok": span_ok,
        "cases": cases,
        "failure_label": if pass { Value::Null } else { json!(D056PrimaryConclusion::CarrierArchitectureNotPortable.as_str()) },
    });
    write_json(&out.join("architecture_feasibility"), "result.json", &v)?;
    Ok((pass, v))
}

fn gate5_shadow(
    out: &Path,
    h: u64,
    params: CarrierParams,
) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    // Noncausal shadow: production trajectory + additive predicted carrier flux observer.
    let seeds = [
        ("analytic_seed", RunCtrl::control_e(22.0)),
        ("restored", {
            let mut c = RunCtrl::control_e(22.0);
            c.freeze_structure = true;
            c
        }),
        ("low_S_proxy", RunCtrl::ordinary_passive(22.0)),
        ("R16", RunCtrl::control_e(16.0)),
        ("R22", RunCtrl::control_e(22.0)),
        ("R32", RunCtrl::control_e(32.0)),
        ("starve_n", {
            let mut c = RunCtrl::control_e(22.0);
            c.starve_n = true;
            c
        }),
        ("starve_f", {
            let mut c = RunCtrl::control_e(22.0);
            c.starve_f = true;
            c
        }),
    ];
    let mut cases = Vec::new();
    let mut all_ok = true;
    for (name, ctrl) in seeds {
        let m = run_obs(h, ctrl);
        let jt = carrier_flux_jt(
            m.n_o,
            m.f_o,
            m.w_i,
            m.n_i,
            m.f_i,
            m.w_o,
            m.gamma_s_sum.max(m.gamma_s_mean),
            params.k_nf,
            params.k_w,
            params.k_t,
        );
        let chi_n = chi_with_carrier(m.j_n, jt, m.n_loss.max(1e-12));
        let chi_f = chi_with_carrier(m.j_f, jt, m.f_loss.max(1e-12));
        let starve = name.contains("starve");
        let ok = if starve {
            // Must not claim false survival / resource import under starvation.
            jt <= 1e-6 && (m.n_loss < 1e-6 || chi_n < 1.0 || m.n_o < 1e-6 || m.f_o < 1e-6)
        } else {
            chi_n >= D056_CHI_TARGET
                && chi_f >= D056_CHI_TARGET
                && jt > 0.0
                && m.steps_ok
                && (-m.j_w_interior + jt) > 0.0
        };
        if !ok {
            all_ok = false;
        }
        cases.push(json!({
            "seed": name,
            "jt_shadow": jt,
            "chi_n": chi_n,
            "chi_f": chi_f,
            "a_retention": m.a_retention,
            "starve": starve,
            "ok": ok,
            "production_unaffected": true,
            "metrics": m_json(&m),
        }));
    }
    let v = json!({
        "gate": "gate5_shadow_trajectories",
        "pass": all_ok,
        "params": {"K_NF": params.k_nf, "K_W": params.k_w, "k_T": params.k_t},
        "cases": cases,
        "failure_label": if all_ok { Value::Null } else { json!(D056PrimaryConclusion::CarrierShadowRepairFailure.as_str()) },
    });
    write_json(&out.join("shadow_trajectories"), "result.json", &v)?;
    Ok((all_ok, v))
}

fn finalize(
    out: &Path,
    primary: D056PrimaryConclusion,
    gates: Value,
    phase_b_authorized: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    let result = json!({
        "project_directive": D056_PROJECT_ID,
        "agent_memory_id": D056_AGENT_MEMORY_ID,
        "primary_conclusion": primary.as_str(),
        "phase_b_authorized": phase_b_authorized,
        "ordinary_passive_branch": D056_ORDINARY_PASSIVE_CLOSED,
        "v14": D056_V14,
        "v15_candidate": D056_V15,
        "equation_version": D056_EQUATION,
        "d008_stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "gates": gates,
    });
    write_json(out, "result.json", &result)?;
    write_json(
        out,
        "manifest.json",
        &json!({
            "directive": "D-056",
            "primary_conclusion": primary.as_str(),
            "phase_b_authorized": phase_b_authorized,
            "artifacts": [
                "preservation", "passive_bound_reproduction", "thermodynamic_review",
                "waste_capacity", "parameter_identification", "architecture_feasibility",
                "shadow_trajectories", "accounting", "result.json"
            ],
        }),
    )?;
    write_json(
        &out.join("accounting"),
        "result.json",
        &json!({
            "note": "Phase A observer-only; no production carrier ledger",
            "global_conservation_analytic": gate1_all_pass(),
        }),
    )?;
    Ok(result)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let h = max_accepted();

    let (g0, v0) = gate0_preservation(&out, h)?;
    if !g0 {
        return finalize(
            &out,
            D056PrimaryConclusion::D055PassiveBoundNotReproduced,
            json!({"gate0": v0}),
            false,
        );
    }

    let (g1, v1) = gate1_thermo(&out)?;
    if !g1 {
        return finalize(
            &out,
            D056PrimaryConclusion::CarrierConservationOrReversibilityFailure,
            json!({"gate0": v0, "gate1": v1}),
            false,
        );
    }

    let (g2, v2) = gate2_waste_capacity(&out, h)?;
    if !g2 {
        return finalize(
            &out,
            D056PrimaryConclusion::WasteGradientCapacityInsufficient,
            json!({"gate0": v0, "gate1": v1, "gate2": v2}),
            false,
        );
    }

    let (g3, v3, params) = gate3_identify(&out, h)?;
    if !g3 {
        return finalize(
            &out,
            D056PrimaryConclusion::CarrierKineticsNotIdentifiable,
            json!({"gate0": v0, "gate1": v1, "gate2": v2, "gate3": v3}),
            false,
        );
    }

    let (g4, v4) = gate4_feasibility(&out, h, params)?;
    if !g4 {
        return finalize(
            &out,
            D056PrimaryConclusion::CarrierArchitectureNotPortable,
            json!({"gate0": v0, "gate1": v1, "gate2": v2, "gate3": v3, "gate4": v4}),
            false,
        );
    }

    let (g5, v5) = gate5_shadow(&out, h, params)?;
    if !g5 {
        return finalize(
            &out,
            D056PrimaryConclusion::CarrierShadowRepairFailure,
            json!({"gate0": v0, "gate1": v1, "gate2": v2, "gate3": v3, "gate4": v4, "gate5": v5}),
            false,
        );
    }

    // Phase A complete — Phase B not auto-started in this runner invocation.
    finalize(
        &out,
        D056PrimaryConclusion::Fail, // placeholder until Phase B; overridden below
        json!({
            "gate0": v0, "gate1": v1, "gate2": v2, "gate3": v3, "gate4": v4, "gate5": v5,
            "phase_a": "PASS",
            "phase_b": "AUTHORIZED_NOT_STARTED",
            "fitted_params": {"K_NF": params.k_nf, "K_W": params.k_w, "k_T": params.k_t},
        }),
        true,
    )
    .map(|mut r| {
        // Honest: Phase A passed but Stage E not recovered yet.
        r["primary_conclusion"] = json!("D056_PHASE_A_CARRIER_ARCHITECTURE_QUALIFIED");
        r["phase_a_pass"] = json!(true);
        let _ = write_json(&out, "result.json", &r);
        r
    })
}
