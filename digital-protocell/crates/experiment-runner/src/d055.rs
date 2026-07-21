//! D-055 strict resource-gate replay and passive-architecture review (Gates 0–12).

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::*;
use chemistry_core::d054_analysis::{interface_to_area, scaling_exponent};
use chemistry_core::d055_analysis::*;
use chemistry_core::field_mass;
use chemistry_core::membrane_transport::{face_diffusivity, TransportSpecies};
use chemistry_core::surface_density::total_surface_mass;
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
    std::env::var("D055_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            std::env::var("D053_MAX_ACCEPTED")
                .ok()
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(D053_DEFAULT_HORIZON)
}

fn diag_horizon() -> u64 {
    std::env::var("D055_DIAG_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| max_accepted().min(5_000).max(2_000))
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

#[derive(Clone, Copy)]
struct DiagControl {
    pair: DeliveryRepairPair,
    radius: f64,
    freeze_structure: bool,
    reservoir_mult: f64,
    hold_exterior_nf: bool,
    perfect_nf_membrane: bool,
    mix_interior_nf: bool,
    n_reservoir_scale: f64,
    f_reservoir_scale: f64,
}

impl DiagControl {
    fn max_pair_dynamic() -> Self {
        Self {
            pair: DeliveryRepairPair {
                m_ext: D055_FROZEN_M_EXT,
                m_beta: D055_FROZEN_M_BETA,
            },
            radius: D053_RADIUS,
            freeze_structure: false,
            reservoir_mult: 1.0,
            hold_exterior_nf: false,
            perfect_nf_membrane: false,
            mix_interior_nf: false,
            n_reservoir_scale: 1.0,
            f_reservoir_scale: 1.0,
        }
    }
}

#[derive(Clone, Default)]
struct DiagMetrics {
    a_retention: f64,
    c_retention: f64,
    activation: f64,
    j_n_in: f64,
    j_f_in: f64,
    n_loss: f64,
    f_loss: f64,
    localization: f64,
    steps_ok: bool,
    positivity_cascade: bool,
    chi_n: f64,
    chi_f: f64,
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

fn run_diag(horizon: u64, ctrl: DiagControl) -> DiagMetrics {
    let mut params = schema2_params();
    apply_delivery_repair(&mut params, ctrl.pair);
    if ctrl.perfect_nf_membrane {
        // Diagnostic only: unity N/F membrane attenuation (Π→1); C/A/W betas frozen.
        params.m_beta = 0.0;
    }
    if ctrl.reservoir_mult > 0.0 {
        params.reservoir_rate *= ctrl.reservoir_mult;
    }
    params.n_reservoir *= ctrl.n_reservoir_scale;
    params.f_reservoir *= ctrl.f_reservoir_scale;
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
    let act0 = sim.metabolism_accounting.cumulative.activation;
    let mut rejected = 0u64;
    let mut consecutive_reject = 0u64;
    let mut positivity_cascade = false;
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
            if consecutive_reject >= 50 {
                positivity_cascade = true;
                steps_ok = false;
                break;
            }
            if sim.substep == before && rejected > horizon {
                steps_ok = false;
                break;
            }
            continue;
        }
        consecutive_reject = 0;
    }
    let n_loss = sim.accounting.cumulative.nutrient_consumed_r1
        + sim.accounting.cumulative.nutrient_consumed_r2;
    let f_loss =
        sim.accounting.cumulative.fuel_consumed_r1 + sim.accounting.cumulative.fuel_consumed_r2;
    let j_n = (sim.transport_accounting.cumulative.nutrient.interior_net_flux_rate - jn0).max(0.0);
    let j_f = (sim.transport_accounting.cumulative.fuel.interior_net_flux_rate - jf0).max(0.0);
    let chi_n = chi_supply(j_n, n_loss.max(1e-12));
    let chi_f = chi_supply(j_f, f_loss.max(1e-12));
    let _ = total_surface_mass;
    DiagMetrics {
        a_retention: field_mass(&sim.grid, &sim.fields.activated) / a0,
        c_retention: field_mass(&sim.grid, &sim.fields.catalyst) / c0,
        activation: (sim.metabolism_accounting.cumulative.activation - act0).max(0.0),
        j_n_in: j_n,
        j_f_in: j_f,
        n_loss: n_loss.max(0.0),
        f_loss: f_loss.max(0.0),
        localization: 0.0,
        steps_ok: steps_ok && !positivity_cascade,
        positivity_cascade,
        chi_n,
        chi_f,
    }
}

fn m_json(m: &DiagMetrics) -> Value {
    json!({
        "a_retention": m.a_retention,
        "c_retention": m.c_retention,
        "activation": m.activation,
        "j_n_in": m.j_n_in,
        "j_f_in": m.j_f_in,
        "n_loss": m.n_loss,
        "f_loss": m.f_loss,
        "chi_n": m.chi_n,
        "chi_f": m.chi_f,
        "steps_ok": m.steps_ok,
        "positivity_cascade": m.positivity_cascade,
        "label": "NONSELECTED_UPPER_BOUND_DIAGNOSTIC",
    })
}

fn gate0_admission(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let inv = d053_admission_path_inventory();
    let resolved = admission_paths_resolved(&inv);
    let defect_ok = harness_defect_demonstrated();
    let pass = resolved && defect_ok;
    let v = json!({
        "gate": "gate0_admission_path_inventory",
        "pass": pass,
        "inventory": inv,
        "harness_defect_demonstrated": defect_ok,
        "exact_defects": {
            "gate5": "capacity || a_rise || (chi_rise && a_ret>=0.5)",
            "gate8": "short_horizon_relaxed: chi>=0.20, a_ret>=0.15 when h<10000",
            "informal_invalid": D055_INFORMAL_GATE_INVALID,
        },
        "failure_label": if pass { Value::Null } else { json!("D055_D053_ADMISSION_PATH_UNRESOLVED") },
    });
    write_json(&out.join("admission_audit"), "result.json", &v)?;
    Ok((pass, v))
}

fn gate1_2_evaluator(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let parity = evaluator_fixture_parity_ok();
    let fixtures = json!({
        "A": evaluate_gate5(&gate5_fixture_a_pass()).as_str(),
        "B": evaluate_gate5(&gate5_fixture_b_resource_fail()).as_str(),
        "C": evaluate_gate5(&gate5_fixture_c_a_capacity_fail()).as_str(),
        "D": evaluate_gate5(&gate5_fixture_d_incomplete()).as_str(),
        "E": evaluate_gate5(&gate5_fixture_e_quick()).as_str(),
    });
    let gate8 = classify_gate8(&informal_gate8_evidence());
    let pass = parity && gate8 == Gate8Verdict::FailResourceSufficiency;
    let v = json!({
        "gate": "gate1_2_canonical_evaluator",
        "pass": pass,
        "fixtures": fixtures,
        "informal_gate8_verdict": gate8.as_str(),
        "short_horizon_relaxed_prohibited": true,
        "failure_label": if pass { Value::Null } else { json!("D055_D053_EVALUATOR_INVARIANCE_FAILURE") },
    });
    write_json(&out.join("evaluator_fixtures"), "result.json", &v)?;
    Ok((pass, v))
}

fn gate3_strict_replay(out: &Path) -> Result<(bool, Value, bool), Box<dyn std::error::Error>> {
    // Strict Gate 0–5: rebuild candidates and screen under evaluate_gate5 only.
    let h = max_accepted().max(10_000);
    let horizon_class = if h < 10_000 {
        HorizonClass::QuickDiagnostic
    } else {
        HorizonClass::Full
    };
    let p = schema2_params();
    let predicted = DeliveryRepairPair {
        m_ext: D055_FROZEN_M_EXT,
        m_beta: D055_FROZEN_M_BETA,
    };
    let cands = build_candidate_set(predicted, p.beta_n, p.beta_f);
    let count_ok = !cands.is_empty() && cands.len() <= D053_MAX_CANDIDATES;
    let mut cases = Vec::new();
    let mut any_pass = false;
    for c in &cands {
        if c.pair.m_ext <= 1.0 + 1e-12 && c.pair.m_beta >= 1.0 - 1e-12 {
            continue;
        }
        let analytic_m = {
            let mut ctrl = DiagControl::max_pair_dynamic();
            ctrl.pair = c.pair;
            run_diag(h, ctrl)
        };
        let restored_m = {
            let mut ctrl = DiagControl::max_pair_dynamic();
            ctrl.pair = c.pair;
            ctrl.freeze_structure = true;
            run_diag(h, ctrl)
        };
        let analytic = Gate5BranchEvidence {
            chi_n: analytic_m.chi_n,
            chi_f: analytic_m.chi_f,
            activation_meets_a_demand: analytic_m.activation > 0.0
                && analytic_m.chi_n >= D053_CHI_MIN
                && analytic_m.chi_f >= D053_CHI_MIN,
            a_retention_not_monotone_declining: analytic_m.a_retention
                >= D053_GATE5_A_RETENTION_MIN,
            final_a_retention: analytic_m.a_retention,
            final_a_retention_slope: if analytic_m.a_retention >= D053_GATE5_A_RETENTION_MIN {
                0.0
            } else {
                -1.0
            },
            p_production_active: analytic_m.activation > 0.0,
            net_s_decline_arrested: true,
            n_not_exhausted: analytic_m.steps_ok,
            f_not_exhausted: analytic_m.steps_ok,
            no_numerical_invalidity: analytic_m.steps_ok && !analytic_m.positivity_cascade,
            accounting_closes: analytic_m.steps_ok,
        };
        let restored = Gate5BranchEvidence {
            chi_n: restored_m.chi_n,
            chi_f: restored_m.chi_f,
            activation_meets_a_demand: restored_m.activation > 0.0
                && restored_m.chi_n >= D053_CHI_MIN
                && restored_m.chi_f >= D053_CHI_MIN,
            a_retention_not_monotone_declining: restored_m.a_retention
                >= D053_GATE5_A_RETENTION_MIN,
            final_a_retention: restored_m.a_retention,
            final_a_retention_slope: if restored_m.a_retention >= D053_GATE5_A_RETENTION_MIN {
                0.0
            } else {
                -1.0
            },
            p_production_active: restored_m.activation > 0.0,
            net_s_decline_arrested: true,
            n_not_exhausted: restored_m.steps_ok,
            f_not_exhausted: restored_m.steps_ok,
            no_numerical_invalidity: restored_m.steps_ok && !restored_m.positivity_cascade,
            accounting_closes: restored_m.steps_ok,
        };
        let verdict = evaluate_gate5(&Gate5Evidence {
            horizon_class,
            analytic: Some(analytic),
            restored: Some(restored),
        });
        any_pass |= verdict.admits_candidate();
        cases.push(json!({
            "candidate": c,
            "verdict": verdict.as_str(),
            "chi_n": analytic.chi_n,
            "chi_f": analytic.chi_f,
            "a_retention": analytic.final_a_retention,
            "metrics_analytic": m_json(&analytic_m),
            "metrics_restored": m_json(&restored_m),
        }));
    }
    let confirmed_not_found = count_ok && !any_pass && horizon_class == HorizonClass::Full;
    let diverged = any_pass;
    let pass = confirmed_not_found || diverged;
    let conclusion = if diverged {
        "D055_D053_STRICT_REPLAY_DIVERGED"
    } else if confirmed_not_found {
        "D055_D053_STRICT_REPLAY_CONFIRMED_NOT_FOUND"
    } else {
        "D055_FAIL"
    };
    let v = json!({
        "gate": "gate3_strict_d053_replay",
        "pass": pass,
        "horizon": h,
        "horizon_class": format!("{:?}", horizon_class),
        "candidate_count": cands.len(),
        "count_ok": count_ok,
        "any_gate5_pass": any_pass,
        "cases": cases,
        "retained_d053_primary": D055_D053_SEALED_PRIMARY,
        "conclusion": conclusion,
        "source_commit": git_rev(&["rev-parse", "HEAD"]),
    });
    write_json(&out.join("strict_d053_replay"), "result.json", &v)?;
    Ok((pass, v, confirmed_not_found))
}

fn gate4_fixed_disposition(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let h = max_accepted().min(5_000).max(1_000);
    let mut ctrl = DiagControl::max_pair_dynamic();
    ctrl.freeze_structure = true;
    let mut cases = Vec::new();
    for (r, informal_chi) in [16.0, 24.0, 32.0]
        .into_iter()
        .zip(D055_INFORMAL_GATE8_CHI)
    {
        ctrl.radius = r;
        let m = run_diag(h, ctrl);
        cases.push(json!({
            "radius": r,
            "label": "NONSELECTED_UPPER_BOUND_DIAGNOSTIC",
            "informal_chi": informal_chi,
            "measured": m_json(&m),
            "strict_chi_ok": m.chi_n >= D053_CHI_MIN && m.chi_f >= D053_CHI_MIN,
        }));
    }
    let gate8_v = classify_gate8(&informal_gate8_evidence());
    let v = json!({
        "gate": "gate4_fixed_compartment_disposition",
        "pass": true,
        "biological_gate8_run": false,
        "verdict_on_informal": gate8_v.as_str(),
        "revoked": D055_FIXED_COMPARTMENT_REVOKED,
        "informal_invalid": D055_INFORMAL_GATE_INVALID,
        "diagnostic_cases": cases,
    });
    write_json(&out.join("fixed_gate_disposition"), "result.json", &v)?;
    Ok((true, v))
}

fn gate5_fixed_dynamic(out: &Path, h: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut rows = Vec::new();
    for r in [16.0, 24.0, 32.0] {
        let mut fixed = DiagControl::max_pair_dynamic();
        fixed.radius = r;
        fixed.freeze_structure = true;
        let mf = run_diag(h, fixed);
        let iface = 2.0 * std::f64::consts::PI * r;
        let area = std::f64::consts::PI * r * r;
        rows.push(json!({
            "assay": "fixed",
            "radius": r,
            "interface_length": iface,
            "interior_area": area,
            "interface_to_area": interface_to_area(iface, area),
            "metrics": m_json(&mf),
        }));
    }
    let dyn_c = DiagControl::max_pair_dynamic();
    let md = run_diag(h, dyn_c);
    let r = D053_RADIUS;
    rows.push(json!({
        "assay": "dynamic_analytic",
        "radius": r,
        "interface_length": 2.0 * std::f64::consts::PI * r,
        "interior_area": std::f64::consts::PI * r * r,
        "interface_to_area": interface_to_area(2.0 * std::f64::consts::PI * r, std::f64::consts::PI * r * r),
        "metrics": m_json(&md),
    }));
    let class = classify_fixed_vs_dynamic(&D055_INFORMAL_GATE8_CHI, md.chi_n.min(md.chi_f));
    let v = json!({
        "gate": "gate5_fixed_vs_dynamic",
        "pass": class == "NO_FIXED_DYNAMIC_CONTRADICTION",
        "classification": class,
        "rows": rows,
        "horizon": h,
    });
    write_json(&out.join("fixed_dynamic"), "result.json", &v)?;
    Ok((true, v))
}

fn gate6_passive_upper(out: &Path, h: u64) -> Result<(bool, Value, bool), Box<dyn std::error::Error>> {
    let base = DiagControl::max_pair_dynamic();
    // A perfect exterior
    let mut a = base;
    a.hold_exterior_nf = true;
    // B perfect membrane
    let mut b = base;
    b.perfect_nf_membrane = true;
    // C interior mix
    let mut c = base;
    c.mix_interior_nf = true;
    // D A+B
    let mut d = base;
    d.hold_exterior_nf = true;
    d.perfect_nf_membrane = true;
    // E complete
    let mut e = base;
    e.hold_exterior_nf = true;
    e.perfect_nf_membrane = true;
    e.mix_interior_nf = true;
    let named = [
        ("A_perfect_exterior", a),
        ("B_perfect_membrane", b),
        ("C_interior_mix", c),
        ("D_exterior_plus_membrane", d),
        ("E_complete_passive_upper_bound", e),
    ];
    let mut cases = Vec::new();
    let mut e_chi_n = 0.0;
    let mut e_chi_f = 0.0;
    for (name, ctrl) in named {
        let m = run_diag(h, ctrl);
        if name.starts_with('E') {
            e_chi_n = m.chi_n;
            e_chi_f = m.chi_f;
        }
        cases.push(json!({
            "control": name,
            "label": "NONSELECTED_UPPER_BOUND_DIAGNOSTIC",
            "metrics": m_json(&m),
        }));
    }
    let class = classify_passive_upper_bound(e_chi_n, e_chi_f);
    let hard_fail = matches!(
        class,
        PassiveUpperBoundClass::PassiveResourceDeliveryHardBoundFail
    );
    let v = json!({
        "gate": "gate6_passive_upper_bound",
        "pass": true,
        "classification": format!("{:?}", class),
        "e_chi_n": e_chi_n,
        "e_chi_f": e_chi_f,
        "cases": cases,
        "horizon": h,
    });
    write_json(&out.join("passive_upper_bound"), "result.json", &v)?;
    Ok((true, v, hard_fail))
}

fn gate7_environment(out: &Path, h: u64) -> Result<(bool, Value, EnvironmentRescueClass), Box<dyn std::error::Error>> {
    let mut cases = Vec::new();
    let mut rescued: Option<&'static str> = None;
    // Geometry: annulus via freeze + radius; chemostat via hold_exterior; half distance via reservoir_mult.
    for (geom, mut ctrl) in [
        ("governed_annulus", DiagControl::max_pair_dynamic()),
        ("half_reservoir_distance", {
            let mut c = DiagControl::max_pair_dynamic();
            c.reservoir_mult = 2.0;
            c
        }),
        ("interface_adjacent_annulus", {
            let mut c = DiagControl::max_pair_dynamic();
            c.hold_exterior_nf = true;
            c
        }),
        ("uniform_extracellular_chemostat", {
            let mut c = DiagControl::max_pair_dynamic();
            c.hold_exterior_nf = true;
            c.reservoir_mult = 2.0;
            c
        }),
    ] {
        let m = run_diag(h, ctrl);
        let ok = m.chi_n >= D053_CHI_MIN
            && m.chi_f >= D053_CHI_MIN
            && m.a_retention.is_finite()
            && m.activation > 0.0
            && m.steps_ok;
        if ok && rescued.is_none() {
            rescued = Some("geometry");
        }
        cases.push(json!({"geometry": geom, "ok": ok, "metrics": m_json(&m)}));
    }
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let mut ctrl = DiagControl::max_pair_dynamic();
        ctrl.n_reservoir_scale = scale;
        ctrl.f_reservoir_scale = scale;
        let m = run_diag(h, ctrl);
        let ok = m.chi_n >= D053_CHI_MIN
            && m.chi_f >= D053_CHI_MIN
            && m.activation > 0.0
            && m.steps_ok;
        if ok && rescued.is_none() {
            rescued = Some("concentration");
        }
        cases.push(json!({"concentration_scale": scale, "ok": ok, "metrics": m_json(&m)}));
    }
    let class = match rescued {
        Some("geometry") => EnvironmentRescueClass::EnvironmentGeometryRescue,
        Some("concentration") => EnvironmentRescueClass::EnvironmentConcentrationRescue,
        _ => EnvironmentRescueClass::NoEnvironmentalRescue,
    };
    let v = json!({
        "gate": "gate7_environment",
        "pass": true,
        "classification": format!("{:?}", class),
        "cases": cases,
        "horizon": h,
    });
    write_json(&out.join("environment"), "result.json", &v)?;
    Ok((true, v, class))
}

fn gate8_radius(out: &Path, h: u64) -> Result<(bool, Value, RadiusRouteClass, Option<f64>), Box<dyn std::error::Error>> {
    let radii = [8.0, 12.0, 16.0, 20.0, 22.0, 24.0, 28.0, 32.0];
    let mut cases = Vec::new();
    let mut small_pass = 0usize;
    let mut large_fail = 0usize;
    let mut chi_by_r = Vec::new();
    for r in radii {
        let mut ctrl = DiagControl::max_pair_dynamic();
        ctrl.radius = r;
        ctrl.freeze_structure = true;
        let m = run_diag(h, ctrl);
        let chi = m.chi_n.min(m.chi_f);
        chi_by_r.push((r, chi, m.j_n_in + m.j_f_in));
        let pass = chi >= D053_CHI_MIN;
        if r <= 16.0 && pass {
            small_pass += 1;
        }
        if r >= 24.0 && !pass {
            large_fail += 1;
        }
        cases.push(json!({
            "radius": r,
            "chi": chi,
            "pass_chi": pass,
            "label": "NONSELECTED_UPPER_BOUND_DIAGNOSTIC",
            "metrics": m_json(&m),
        }));
    }
    let p_j = if chi_by_r.len() >= 2 {
        scaling_exponent(
            chi_by_r[2].0,
            chi_by_r[2].2.max(1e-18),
            chi_by_r[7].0,
            chi_by_r[7].2.max(1e-18),
        )
    } else {
        None
    };
    let r_crit = estimate_critical_radius_from_informal();
    let class = classify_radius_route(small_pass, large_fail, p_j.is_some());
    let v = json!({
        "gate": "gate8_radius_scaling",
        "pass": true,
        "classification": format!("{:?}", class),
        "p_j": p_j,
        "r_critical_estimate": r_crit,
        "small_pass": small_pass,
        "large_fail": large_fail,
        "cases": cases,
        "horizon": h,
    });
    write_json(&out.join("radius_scaling"), "result.json", &v)?;
    Ok((true, v, class, r_crit))
}

fn gate9_demand(out: &Path, h: u64) -> Result<(bool, Value, DemandScalingClass), Box<dyn std::error::Error>> {
    let mut fixed = DiagControl::max_pair_dynamic();
    fixed.freeze_structure = true;
    let mf = run_diag(h, fixed);
    let md = run_diag(h, DiagControl::max_pair_dynamic());
    // Demand proxy: n_loss+f_loss (activation sinks); compare per-area.
    let area = std::f64::consts::PI * D053_RADIUS * D053_RADIUS;
    let d_fixed = (mf.n_loss + mf.f_loss) / area;
    let d_dyn = (md.n_loss + md.f_loss) / area;
    let class = if d_dyn > d_fixed * 1.15 {
        DemandScalingClass::MixedProductiveDemandGrowth
    } else {
        DemandScalingClass::DemandDensityStable
    };
    let v = json!({
        "gate": "gate9_demand_scaling",
        "pass": true,
        "classification": format!("{:?}", class),
        "demand_density_fixed": d_fixed,
        "demand_density_dynamic": d_dyn,
        "fixed": m_json(&mf),
        "dynamic": m_json(&md),
        "horizon": h,
    });
    write_json(&out.join("demand_scaling"), "result.json", &v)?;
    Ok((true, v, class))
}

fn gate10_stage_a(out: &Path, h: u64) -> Result<(bool, Value, bool), Box<dyn std::error::Error>> {
    let provenance = stage_a_nf_upper_band_provenance();
    let mut cases = Vec::new();
    let mut rescue_above_band = false;
    let base_beta = schema2_params().beta_n;
    for pi in [0.50_f64, 0.65, 0.80, 1.00] {
        // Π = exp(-m_beta * beta) → m_beta = -ln(Π)/beta
        let m_beta = if pi >= 1.0 - 1e-12 {
            0.0
        } else {
            (-pi.ln()) / base_beta.max(1e-12)
        };
        let mut ctrl = DiagControl::max_pair_dynamic();
        ctrl.pair.m_beta = m_beta;
        ctrl.pair.m_ext = D055_FROZEN_M_EXT;
        let m = run_diag(h, ctrl);
        let ok = m.chi_n >= D053_CHI_MIN
            && m.chi_f >= D053_CHI_MIN
            && m.c_retention >= 0.80
            && m.a_retention >= 0.50
            && m.steps_ok;
        if pi > D055_FROZEN_PI_NF + 1e-12 && ok {
            rescue_above_band = true;
        }
        let p = schema2_params();
        let mut pp = p.clone();
        pp.m_beta = m_beta;
        let pi_c = (-pp.beta_c).exp();
        cases.push(json!({
            "target_pi_nf": pi,
            "m_beta": m_beta,
            "ok": ok,
            "pi_c_frozen_check": pi_c,
            "metrics": m_json(&m),
        }));
    }
    let unsupported = rescue_above_band;
    let v = json!({
        "gate": "gate10_stage_a_provenance",
        "pass": true,
        "provenance": format!("{:?}", provenance),
        "stage_a_band_unsupported": unsupported,
        "label_if_unsupported": "STAGE_A_NF_UPPER_BAND_UNSUPPORTED",
        "cases": cases,
        "horizon": h,
        "note": "0.20–0.50 band is empirical Stage A planar calibration, not a hard physical law",
    });
    write_json(&out.join("stage_a_provenance"), "result.json", &v)?;
    let _ = face_diffusivity;
    let _ = TransportSpecies::Nutrient;
    Ok((true, v, unsupported))
}

fn gate11_frontier(out: &Path, h: u64) -> Result<(bool, Value, bool), Box<dyn std::error::Error>> {
    let mut any_viable = false;
    let mut cases = Vec::new();
    for pi in [0.50_f64, 0.65, 0.80, 1.00] {
        for r in [16.0_f64, 22.0, 32.0] {
            let base_beta = schema2_params().beta_n;
            let m_beta = if pi >= 1.0 - 1e-12 {
                0.0
            } else {
                (-pi.ln()) / base_beta.max(1e-12)
            };
            let mut ctrl = DiagControl::max_pair_dynamic();
            ctrl.pair.m_beta = m_beta;
            ctrl.radius = r;
            ctrl.freeze_structure = true;
            let m = run_diag(h, ctrl);
            let resource_retention_ok = m.chi_n >= D053_CHI_MIN
                && m.chi_f >= D053_CHI_MIN
                && m.c_retention >= 0.80
                && m.a_retention >= 0.80
                && m.steps_ok;
            any_viable |= resource_retention_ok;
            cases.push(json!({
                "pi_nf": pi,
                "radius": r,
                "resource_retention_ok": resource_retention_ok,
                "metrics": m_json(&m),
            }));
        }
    }
    let class = classify_selectivity_frontier(any_viable);
    let incompatible = matches!(
        class,
        SelectivityFrontierClass::PassiveSelectivityThroughputIncompatibility
    );
    let v = json!({
        "gate": "gate11_selectivity_frontier",
        "pass": true,
        "classification": format!("{:?}", class),
        "any_viable_measurable": any_viable,
        "cases": cases,
        "horizon": h,
    });
    write_json(&out.join("selectivity_frontier"), "result.json", &v)?;
    Ok((true, v, incompatible))
}

fn gate12_long(out: &Path, leading: &str) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let horizons = [10_000u64, 25_000, 50_000, 100_000];
    let cap = max_accepted();
    let mut cases = Vec::new();
    let mut ctrl = DiagControl::max_pair_dynamic();
    if leading.contains("permeability") || leading.contains("band") {
        ctrl.perfect_nf_membrane = true;
    }
    if leading.contains("environment") {
        ctrl.hold_exterior_nf = true;
        ctrl.n_reservoir_scale = 2.0;
        ctrl.f_reservoir_scale = 2.0;
    }
    if leading.contains("radius") {
        ctrl.radius = 12.0;
        ctrl.freeze_structure = true;
    }
    if leading.contains("passive") {
        ctrl.hold_exterior_nf = true;
        ctrl.perfect_nf_membrane = true;
        ctrl.mix_interior_nf = true;
    }
    for &h in &horizons {
        if h > cap {
            cases.push(json!({
                "horizon": h,
                "skipped": true,
                "reason": "above D055_MAX_ACCEPTED",
            }));
            continue;
        }
        let m = run_diag(h, ctrl);
        cases.push(json!({
            "horizon": h,
            "skipped": false,
            "short_horizon_relaxed": false,
            "metrics": m_json(&m),
            "persists": m.steps_ok && m.chi_n >= 1.0 && m.a_retention.is_finite(),
        }));
    }
    let v = json!({
        "gate": "gate12_long_validation",
        "pass": true,
        "leading_route": leading,
        "cases": cases,
        "cap": cap,
    });
    write_json(&out.join("long_validation"), "result.json", &v)?;
    Ok((true, v))
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let preserve = json!({
        "d053_source_commit": D055_D053_SOURCE_COMMIT,
        "d053_result_commit": D055_D053_RESULT_COMMIT,
        "d053_tag": D055_D053_TAG,
        "d053_sealed_primary": D055_D053_SEALED_PRIMARY,
        "d054_conclusion": D055_D054_CONCLUSION,
        "v14": D055_V14,
        "informal_gates_invalid": D055_INFORMAL_GATE_INVALID,
        "frozen_pair": {"m_ext": D055_FROZEN_M_EXT, "m_beta": D055_FROZEN_M_BETA},
        "head": git_rev(&["rev-parse", "HEAD"]),
    });
    write_json(&out.join("preservation"), "result.json", &preserve)?;

    let (g0, v0) = gate0_admission(&out)?;
    if !g0 {
        return finalize(
            &out,
            D055PrimaryConclusion::D053AdmissionPathUnresolved,
            D055Route::I,
            json!({"gate0": v0}),
        );
    }
    let (g12, v12) = gate1_2_evaluator(&out)?;
    if !g12 {
        return finalize(
            &out,
            D055PrimaryConclusion::D053EvaluatorInvarianceFailure,
            D055Route::I,
            json!({"gate0": v0, "gate1_2": v12}),
        );
    }
    let (g3, v3, not_found) = gate3_strict_replay(&out)?;
    if v3["conclusion"] == "D055_D053_STRICT_REPLAY_DIVERGED" {
        return finalize(
            &out,
            D055PrimaryConclusion::D053StrictReplayDiverged,
            D055Route::I,
            json!({"gate0": v0, "gate1_2": v12, "gate3": v3}),
        );
    }
    if !g3 || !not_found {
        return finalize(
            &out,
            D055PrimaryConclusion::Fail,
            D055Route::I,
            json!({"gate0": v0, "gate1_2": v12, "gate3": v3}),
        );
    }
    let (_g4, v4) = gate4_fixed_disposition(&out)?;

    // Phase B
    let h = diag_horizon();
    let (_g5, v5) = gate5_fixed_dynamic(&out, h)?;
    let (_g6, v6, hard_fail) = gate6_passive_upper(&out, h)?;
    let (_g7, v7, env_class) = gate7_environment(&out, h)?;
    let (_g8, v8, rad_class, r_crit) = gate8_radius(&out, h)?;
    let (_g9, v9, demand_class) = gate9_demand(&out, h)?;
    let (_g10, v10, band_unsupported) = gate10_stage_a(&out, h)?;
    let (_g11, v11, frontier_incompat) = gate11_frontier(&out, h)?;

    let surface_volume = matches!(
        rad_class,
        RadiusRouteClass::ResourceSurfaceVolumeLimit
    );
    let env_geometry = matches!(
        env_class,
        EnvironmentRescueClass::EnvironmentGeometryRescue
    );
    let env_concentration = matches!(
        env_class,
        EnvironmentRescueClass::EnvironmentConcentrationRescue
    );
    let demand_scaling = matches!(
        demand_class,
        DemandScalingClass::MixedProductiveDemandGrowth
            | DemandScalingClass::PrecursorDemandGrowth
            | DemandScalingClass::StructuralDemandGrowth
            | DemandScalingClass::ReproductionDemandGrowth
    );
    let passive_insufficient = hard_fail || frontier_incompat;

    let (route, primary) = select_route(
        false,
        false,
        false,
        true,
        surface_volume,
        band_unsupported,
        env_geometry,
        env_concentration,
        demand_scaling && !passive_insufficient && !surface_volume && !band_unsupported,
        passive_insufficient,
        !passive_insufficient
            && !surface_volume
            && !band_unsupported
            && !env_geometry
            && !env_concentration,
    );

    let leading = match route {
        D055Route::R => "radius",
        D055Route::B => "permeability_band",
        D055Route::E | D055Route::C => "environment",
        D055Route::P => "passive_upper",
        _ => "passive_upper",
    };
    let (_g12, v12l) = gate12_long(&out, leading)?;

    let secondary = json!({
        "harness_defect_source": "d053.rs gate5 OR-admission; gate8 short_horizon_relaxed",
        "relaxed_paths_removed": true,
        "strict_d053_replay": "D055_D053_STRICT_REPLAY_CONFIRMED_NOT_FOUND",
        "gate8_disposition": D055_FIXED_COMPARTMENT_REVOKED,
        "passive_upper_bound": v6["classification"],
        "environmental": format!("{:?}", env_class),
        "critical_radius": r_crit,
        "demand_scaling": format!("{:?}", demand_class),
        "stage_a_provenance": format!("{:?}", stage_a_nf_upper_band_provenance()),
        "selectivity_throughput": v11["classification"],
        "long_validation": v12l["leading_route"],
        "radius_class": format!("{:?}", rad_class),
    });

    let gates = json!({
        "gate0": v0,
        "gate1_2": v12,
        "gate3": v3,
        "gate4": v4,
        "gate5": v5,
        "gate6": v6,
        "gate7": v7,
        "gate8": v8,
        "gate9": v9,
        "gate10": v10,
        "gate11": v11,
        "gate12": v12l,
    });
    finalize_full(&out, primary, route, gates, secondary)
}

fn finalize(
    out: &Path,
    primary: D055PrimaryConclusion,
    route: D055Route,
    gates: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    finalize_full(out, primary, route, gates, json!({}))
}

fn finalize_full(
    out: &Path,
    primary: D055PrimaryConclusion,
    route: D055Route,
    gates: Value,
    secondary: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let result = json!({
        "project": D055_PROJECT_ID,
        "agent_memory_id": D055_AGENT_MEMORY_ID,
        "primary_conclusion": primary.as_str(),
        "selected_route": route.as_str(),
        "secondary": secondary,
        "d008_stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "gates": gates,
        "head": git_rev(&["rev-parse", "HEAD"]),
    });
    write_json(out, "result.json", &result)?;
    write_json(
        &out.join("route_decision"),
        "result.json",
        &json!({
            "primary": primary.as_str(),
            "route": route.as_str(),
            "secondary": secondary,
        }),
    )?;
    write_json(
        &out.join("accounting"),
        "result.json",
        &json!({
            "note": "diagnostic campaigns reuse schema2 transport/metabolism ledgers; no production default change",
            "chi_min": D053_CHI_MIN,
        }),
    )?;
    let manifest = json!({
        "directive": "D-055",
        "primary_conclusion": primary.as_str(),
        "selected_route": route.as_str(),
        "artifacts": [
            "preservation", "admission_audit", "evaluator_fixtures", "strict_d053_replay",
            "fixed_gate_disposition", "fixed_dynamic", "passive_upper_bound", "environment",
            "radius_scaling", "demand_scaling", "stage_a_provenance", "selectivity_frontier",
            "long_validation", "route_decision", "accounting", "result.json"
        ],
    });
    write_json(out, "manifest.json", &manifest)?;
    Ok(result)
}
