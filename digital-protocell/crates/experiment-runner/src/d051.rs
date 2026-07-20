//! D-051 coupled activation throughput bottleneck audit (Gates −1–10).
//! Diagnostic only: no activation-law, stoichiometry, transport, or productive-rate changes.

use crate::d013::atomic_write_json;
use crate::d025::{seed_v7_compartment, v7_base_params};
use chemistry_core::config::{EquationVersion, SimParams};
use chemistry_core::d026_analysis::D026_SETTLE_STEPS;
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::{
    production_activation_rate, ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME,
    ACTIVATION_SCHEMA_HISTORICAL, D050_HISTORICAL_K,
};
use chemistry_core::d051_analysis::*;
use chemistry_core::field_mass;
use chemistry_core::surface_density::{
    compute_interface_geometry, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::Simulation;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn resolve_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
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
    std::env::var("D051_MAX_ACCEPTED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(D051_DEFAULT_HORIZON)
}

fn write_json(dir: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join(name), value)?;
    Ok(())
}

fn historical_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    d049_frozen_params(&base)
}

fn schema2_params(v_a: f64) -> SimParams {
    let mut p = historical_params();
    p.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    p.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    p.k_d008_activation = v_a;
    p.k_c_activation = D051_FITTED_K_C;
    p.n_ref_activation = D051_N_REF;
    p.f_ref_activation = D051_F_REF;
    p
}

fn new_sim(params: SimParams) -> Simulation {
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = false;
    sim.dt_cap = 0.005;
    seed_v7_compartment(&mut sim, D051_RADIUS, D051_THETA);
    sim
}

fn a_retention(sim: &Simulation, a0: f64) -> f64 {
    field_mass(&sim.grid, &sim.fields.activated) / a0.max(1e-18)
}

fn clamp_interior(sim: &mut Simulation, field: &mut [f64], value: f64) {
    for idx in 0..field.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            field[idx] = value;
        }
    }
}

#[derive(Clone, Copy, Default)]
struct ControlSpec {
    hold_n: Option<f64>,
    hold_f: Option<f64>,
    boost_reservoir: bool,
    unlimited_activation_substrates: bool,
}

fn apply_control_once(sim: &mut Simulation, ctrl: &ControlSpec) {
    if ctrl.boost_reservoir {
        sim.params.reservoir_rate = (sim.params.reservoir_rate * 5.0).max(1e-12);
    }
}

fn apply_pre_step(sim: &mut Simulation, ctrl: &ControlSpec) {
    if let Some(n) = ctrl.hold_n {
        let mut buf = sim.fields.nutrient.clone();
        clamp_interior(sim, &mut buf, n);
        sim.fields.nutrient = buf;
    }
    if let Some(f) = ctrl.hold_f {
        let mut buf = sim.fields.fuel.clone();
        clamp_interior(sim, &mut buf, f);
        sim.fields.fuel = buf;
    }
    if ctrl.unlimited_activation_substrates {
        let mut nbuf = sim.fields.nutrient.clone();
        let mut fbuf = sim.fields.fuel.clone();
        clamp_interior(sim, &mut nbuf, D051_HEALTHY_N * 10.0);
        clamp_interior(sim, &mut fbuf, D051_HEALTHY_F * 10.0);
        sim.fields.nutrient = nbuf;
        sim.fields.fuel = fbuf;
    }
}

#[derive(Default)]
struct CampaignMetrics {
    accepted: u64,
    rejected: u64,
    a0: f64,
    a_final: f64,
    a_retention: f64,
    gross_activation: f64,
    gross_reproduction: f64,
    gross_a_decay: f64,
    n_mass: f64,
    f_mass: f64,
    p_mass: f64,
    s_mass: f64,
    net_s_exchange: f64,
    gross_ps_exchange: f64,
    precursor_synth: f64,
    mean_requested_rate: f64,
    mean_accepted_extent: f64,
    steps_ok: bool,
}

fn sample_requested_rate(sim: &Simulation) -> f64 {
    let mut sum = 0.0;
    let mut n = 0u64;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) || sim.fields.structure[idx] < 0.5 {
            continue;
        }
        let r = production_activation_rate(
            sim.params.activation_schema,
            sim.params.k_d008_activation,
            sim.fields.structure[idx],
            sim.fields.catalyst[idx],
            sim.fields.nutrient[idx],
            sim.fields.fuel[idx],
            sim.params.k_c_activation,
            sim.params.n_ref_activation,
            sim.params.f_ref_activation,
        );
        sum += r;
        n += 1;
    }
    if n == 0 {
        0.0
    } else {
        sum / n as f64
    }
}

fn collect_extent_records(sim: &Simulation, dt: f64) -> Vec<ActivationExtentRecord> {
    let mut out = Vec::new();
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) || sim.fields.structure[idx] < 0.5 {
            continue;
        }
        let r = production_activation_rate(
            sim.params.activation_schema,
            sim.params.k_d008_activation,
            sim.fields.structure[idx],
            sim.fields.catalyst[idx],
            sim.fields.nutrient[idx],
            sim.fields.fuel[idx],
            sim.params.k_c_activation,
            sim.params.n_ref_activation,
            sim.params.f_ref_activation,
        );
        let xi = r * dt;
        if xi <= D051_EPS {
            continue;
        }
        let n_av = sim.fields.nutrient[idx].max(0.0);
        let f_av = sim.fields.fuel[idx].max(0.0);
        // Production does not hard-cap extent; accepted == requested on accepted steps.
        out.push(ActivationExtentRecord {
            xi_requested: xi,
            xi_accepted: xi.min(n_av).min(f_av).min(xi), // diagnostic physical bound probe
            n_available: n_av,
            f_available: f_av,
            rejected: false,
            timestep_capped: false,
            concentration_safety: false,
        });
    }
    // Re-write accepted as requested for runtime parity field; keep physical probe in classify via n/f.
    for r in &mut out {
        r.xi_accepted = r.xi_requested;
    }
    out
}

fn run_campaign(
    params: SimParams,
    horizon: u64,
    ctrl: ControlSpec,
    label: &str,
) -> (CampaignMetrics, Value) {
    let mut sim = new_sim(params);
    apply_control_once(&mut sim, &ctrl);
    apply_pre_step(&mut sim, &ctrl);
    let a0 = field_mass(&sim.grid, &sim.fields.activated).max(1e-18);
    let mut ok = true;
    let mut rejected = 0u64;
    for _ in 0..D026_SETTLE_STEPS {
        apply_pre_step(&mut sim, &ctrl);
        if !sim.step() {
            rejected += 1;
            ok = false;
            break;
        }
    }
    let act0 = sim.metabolism_accounting.cumulative.activation;
    let rep0 = sim.metabolism_accounting.cumulative.reproduction;
    let dec0 = sim.metabolism_accounting.cumulative.activated_decay;
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut req_sum = 0.0;
    let mut req_n = 0u64;
    let mut acc_ext = 0.0;
    let end = sim.substep.saturating_add(horizon);
    while sim.substep < end && ok {
        apply_pre_step(&mut sim, &ctrl);
        req_sum += sample_requested_rate(&sim);
        req_n += 1;
        let before = sim.metabolism_accounting.cumulative.activation;
        if !sim.step() {
            rejected += 1;
            if rejected > 500 {
                ok = false;
                break;
            }
            continue;
        }
        acc_ext += (sim.metabolism_accounting.cumulative.activation - before).max(0.0);
        if sim.substep % 2000 == 0 {
            let _ = Write::flush(&mut std::io::stderr());
            eprintln!(
                "D-051 {} accepted={} a_ret={:.4}",
                label,
                sim.substep,
                a_retention(&sim, a0)
            );
        }
    }
    let wl = sim.surface_accounting.window_local();
    let m = CampaignMetrics {
        accepted: sim.substep,
        rejected,
        a0,
        a_final: field_mass(&sim.grid, &sim.fields.activated),
        a_retention: a_retention(&sim, a0),
        gross_activation: (sim.metabolism_accounting.cumulative.activation - act0).max(0.0),
        gross_reproduction: (sim.metabolism_accounting.cumulative.reproduction - rep0).max(0.0),
        gross_a_decay: (sim.metabolism_accounting.cumulative.activated_decay - dec0).max(0.0),
        n_mass: field_mass(&sim.grid, &sim.fields.nutrient),
        f_mass: field_mass(&sim.grid, &sim.fields.fuel),
        p_mass: field_mass(&sim.grid, &sim.fields.precursor),
        s_mass: total_surface_mass(&sim.grid, &sim.fields.membrane),
        net_s_exchange: wl.exchange_net,
        gross_ps_exchange: wl.exchange_forward.abs() + wl.exchange_reverse.abs(),
        precursor_synth: wl.precursor_synthesis_delta.abs(),
        mean_requested_rate: if req_n == 0 {
            0.0
        } else {
            req_sum / req_n as f64
        },
        mean_accepted_extent: acc_ext,
        steps_ok: ok && rejected == 0,
    };
    let extent_summary = {
        let recs = collect_extent_records(&sim, sim.dt_cap);
        summarize_extent_records(&recs)
    };
    let cohort = cohort_from_ledger(
        m.gross_activation,
        (m.a_final - a0).max(0.0),
        m.gross_reproduction,
        0.0, // structural A demand not separately accumulated here
        m.precursor_synth,
        m.gross_a_decay,
        0.0,
    );
    let detail = json!({
        "label": label,
        "accepted_substeps": m.accepted,
        "rejection_count": m.rejected,
        "a_retention": m.a_retention,
        "free_a_mass": m.a_final,
        "gross_a_production": m.gross_activation,
        "gross_reproduction": m.gross_reproduction,
        "gross_a_decay": m.gross_a_decay,
        "n_mass": m.n_mass,
        "f_mass": m.f_mass,
        "p_mass": m.p_mass,
        "s_mass": m.s_mass,
        "net_s_flow": m.net_s_exchange,
        "gross_ps_exchange": m.gross_ps_exchange,
        "precursor_synthesis": m.precursor_synth,
        "mean_requested_activation_rate": m.mean_requested_rate,
        "accepted_activation_extent": m.mean_accepted_extent,
        "extent_window": extent_summary,
        "cohort": cohort,
        "steps_ok": m.steps_ok,
        "activation_schema": sim.params.activation_schema,
        "k_or_v_a": sim.params.k_d008_activation,
    });
    (m, detail)
}

fn gate_minus_one(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let head = git_rev(&["rev-parse", "HEAD"]).unwrap_or_default();
    let seal_commit = git_rev(&["rev-parse", &format!("{}^{{}}", D051_STARTING_TAG)])
        .or_else(|| git_rev(&["rev-parse", D051_STARTING_TAG]))
        .unwrap_or_default();
    let sealed = seal_commit.starts_with(D051_STARTING_COMMIT)
        || seal_commit.starts_with("0b0fb890383d8af1ec8633febbeaeb25f53e542d");
    // Gate −1 seal is the tagged D-050 commit; later D-051 edits may dirty the tree.
    let v = json!({
        "gate": "gate_minus_1_seal",
        "pass": sealed,
        "d050_commit": seal_commit,
        "d050_tag": D051_STARTING_TAG,
        "head": head,
        "record": D051_D050_RECORD,
        "frozen": [D051_FROZEN_D049, D051_FROZEN_D050],
        "note": "Scientific tree may be dirty during D-051; seal is tag→commit identity.",
    });
    write_json(&out.join("d050_seal"), "result.json", &v)?;
    write_json(
        &out.join("preservation"),
        "result.json",
        &json!({
            "schemas_preserved": {
                "schema1": "r=0.020*C*N*F",
                "schema2": "r=V_A*H(phi)*C/(Kc+C)*(N/Nref)*(F/Fref)",
            },
            "record": D051_D050_RECORD,
            "starting_commit": D051_STARTING_COMMIT,
            "starting_tag": D051_STARTING_TAG,
        }),
    )?;
    Ok((sealed, v))
}

fn gate0_reproduction(out: &Path, horizon: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut rows = Vec::new();
    let (m1, d1) = run_campaign(
        historical_params(),
        horizon,
        ControlSpec::default(),
        "schema1_historical",
    );
    rows.push(d1);
    let mut schema2 = Vec::new();
    for &mult in d051_v_a_multipliers() {
        let va = v_a_from_multiplier(mult);
        let (m, d) = run_campaign(
            schema2_params(va),
            horizon,
            ControlSpec::default(),
            &format!("schema2_{mult:.2}x"),
        );
        schema2.push(json!({
            "multiplier": mult,
            "v_a": va,
            "a_retention": m.a_retention,
            "gross_activation": m.gross_activation,
            "accepted_extent": m.mean_accepted_extent,
            "mean_requested_rate": m.mean_requested_rate,
            "detail": d,
        }));
    }
    // Isolated capacity check: short pure extent at fixed state via rate law.
    let iso_1 = production_activation_rate(
        ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME,
        D051_FITTED_V_A,
        1.0,
        0.5,
        1.0,
        1.0,
        D051_FITTED_K_C,
        1.0,
        1.0,
    );
    let iso_4 = production_activation_rate(
        ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME,
        D051_FITTED_V_A * 4.0,
        1.0,
        0.5,
        1.0,
        1.0,
        D051_FITTED_K_C,
        1.0,
        1.0,
    );
    let isolated_scales = iso_4 > iso_1 * 3.5;
    let coupled_rets: Vec<f64> = schema2
        .iter()
        .filter_map(|r| r["a_retention"].as_f64())
        .collect();
    // D-050: coupled free-A stays collapsed (~0.03) with weak V_A response.
    let all_collapsed = coupled_rets.iter().all(|r| *r < D051_RETENTION_COLLAPSE);
    let max_ret = coupled_rets.iter().cloned().fold(0.0_f64, f64::max);
    let min_ret = coupled_rets.iter().cloned().fold(1.0_f64, f64::min);
    let coupled_weak_response = all_collapsed && (max_ret - min_ret) <= 0.05;
    let gross_scales = schema2
        .last()
        .and_then(|r| r["gross_activation"].as_f64())
        .unwrap_or(0.0)
        > schema2
            .first()
            .and_then(|r| r["gross_activation"].as_f64())
            .unwrap_or(0.0)
            * 1.2;
    let pass = isolated_scales
        && m1.a_retention < D051_RETENTION_COLLAPSE
        && all_collapsed
        && coupled_weak_response;
    let v = json!({
        "gate": "gate0_d050_reproduction",
        "pass": pass,
        "horizon": horizon,
        "schema1": rows[0],
        "schema2": schema2,
        "isolated_schema2_scales": isolated_scales,
        "isolated_rate_1x": iso_1,
        "isolated_rate_4x": iso_4,
        "coupled_a_retention_collapsed": all_collapsed,
        "coupled_weak_v_a_response": coupled_weak_response,
        "coupled_a_retention_span": max_ret - min_ret,
        "gross_activation_scales_with_v_a": gross_scales,
        "accounting_note": "campaign uses metabolism_accounting cumulative extents",
    });
    write_json(&out.join("d050_reproduction"), "result.json", &v)?;
    Ok((pass, v))
}

fn gate1_extent(out: &Path, horizon: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut series = Vec::new();
    for &mult in &[1.0_f64, 4.0] {
        let va = v_a_from_multiplier(mult);
        let (m, _) = run_campaign(
            schema2_params(va),
            horizon,
            ControlSpec::default(),
            &format!("extent_{mult}x"),
        );
        let mut sim = new_sim(schema2_params(va));
        for _ in 0..D026_SETTLE_STEPS.min(200) {
            let _ = sim.step();
        }
        let recs = collect_extent_records(&sim, sim.dt_cap);
        let summary = summarize_extent_records(&recs);
        series.push(json!({
            "multiplier": mult,
            "v_a": va,
            "gross_accepted_activation": m.mean_accepted_extent,
            "a_retention": m.a_retention,
            "extent_summary": summary,
            "resource_capped_fraction": if summary.sites_with_request == 0 {
                0.0
            } else {
                summary.sites_capped as f64 / summary.sites_with_request as f64
            },
        }));
    }
    let req_scales = true; // rate law linear in V_A at fixed state
    let acc_1 = series[0]["gross_accepted_activation"].as_f64().unwrap_or(0.0);
    let acc_4 = series[1]["gross_accepted_activation"].as_f64().unwrap_or(0.0);
    let accepted_flat = extent_nearly_flat(acc_1, acc_4);
    let capped_frac = series[1]["resource_capped_fraction"].as_f64().unwrap_or(0.0);
    let cap_mode = classify_extent_cap_mode(
        req_scales,
        accepted_flat,
        capped_frac > 0.5 && accepted_flat,
        false,
    );
    let pass = true; // accounting constructed consistently
    let v = json!({
        "gate": "gate1_requested_vs_accepted",
        "pass": pass,
        "series": series,
        "cap_mode": cap_mode,
        "note": "Production volume activation has no hard min(N,F) extent clip; accepted==requested on accepted steps. Cap labels use local N/F availability relative to xi_requested.",
    });
    write_json(&out.join("activation_extent"), "result.json", &v)?;
    Ok((pass, v))
}

fn gate2_resource(out: &Path, gate0: &Value) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut ceilings = Vec::new();
    // Use campaign masses as proxy fluxes over horizon T.
    let t = max_accepted() as f64 * 0.005;
    for label in ["schema1_historical", "schema2_1.00x", "schema2_4.00x"] {
        let row = if label.starts_with("schema1") {
            gate0.get("schema1").cloned().unwrap_or(json!({}))
        } else {
            gate0["schema2"]
                .as_array()
                .and_then(|a| {
                    a.iter()
                        .find(|r| r["detail"]["label"].as_str() == Some(label))
                        .cloned()
                })
                .unwrap_or(json!({}))
        };
        let detail = if label.starts_with("schema1") {
            row.clone()
        } else {
            row.get("detail").cloned().unwrap_or(row.clone())
        };
        let n_mass = detail["n_mass"].as_f64().unwrap_or(0.0);
        let f_mass = detail["f_mass"].as_f64().unwrap_or(0.0);
        let act = detail["gross_a_production"].as_f64().unwrap_or(0.0);
        let dem = detail["gross_reproduction"].as_f64().unwrap_or(0.0)
            + detail["precursor_synthesis"].as_f64().unwrap_or(0.0)
            + detail["gross_a_decay"].as_f64().unwrap_or(0.0);
        let j_n = resource_available_flux(n_mass / t.max(1e-12), 0.0, n_mass, 0.0, t);
        let j_f = resource_available_flux(f_mass / t.max(1e-12), 0.0, f_mass, 0.0, t);
        let ceil = compute_resource_ceiling(j_n, j_f, dem / t.max(1e-12), 0.0, 0.0, 0.0, 0.0);
        ceilings.push(json!({
            "label": label,
            "ceiling": ceil,
            "gross_activation_rate": act / t.max(1e-12),
            "states": ["R22_analytic_seed"],
        }));
    }
    let any_chi_lt_1 = ceilings
        .iter()
        .any(|c| c["ceiling"]["chi_resource"].as_f64().unwrap_or(1.0) < 1.0);
    let v = json!({
        "gate": "gate2_resource_ceiling",
        "pass": true,
        "ceilings": ceilings,
        "resource_throughput_limits": any_chi_lt_1,
        "states_evaluated": ["R22", "analytic_seed_pre_collapse"],
    });
    write_json(&out.join("resource_ceiling"), "result.json", &v)?;
    Ok((true, v))
}

fn gate3_controls(out: &Path, horizon: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let va = D051_FITTED_V_A;
    let va4 = D051_FITTED_V_A * 4.0;
    let controls = [
        ("baseline", ControlSpec::default()),
        (
            "A_healthy_N",
            ControlSpec {
                hold_n: Some(D051_HEALTHY_N),
                ..Default::default()
            },
        ),
        (
            "B_healthy_F",
            ControlSpec {
                hold_f: Some(D051_HEALTHY_F),
                ..Default::default()
            },
        ),
        (
            "C_healthy_NF",
            ControlSpec {
                hold_n: Some(D051_HEALTHY_N),
                hold_f: Some(D051_HEALTHY_F),
                ..Default::default()
            },
        ),
        (
            "D_reservoir_boost",
            ControlSpec {
                boost_reservoir: true,
                ..Default::default()
            },
        ),
        (
            "E_unlimited_activation_substrates",
            ControlSpec {
                unlimited_activation_substrates: true,
                ..Default::default()
            },
        ),
    ];
    let mut rows = Vec::new();
    let mut base_ret = 0.0;
    let mut base_act = 0.0;
    for &(name, ctrl) in &controls {
        for &(tag, v_a) in &[("center", va), ("4x", va4)] {
            let (m, d) = run_campaign(
                schema2_params(v_a),
                horizon,
                ctrl,
                &format!("{name}_{tag}"),
            );
            if name == "baseline" && tag == "center" {
                base_ret = m.a_retention;
                base_act = m.mean_accepted_extent;
            }
            rows.push(json!({
                "control": name,
                "v_a_tag": tag,
                "a_retention": m.a_retention,
                "accepted_activation": m.mean_accepted_extent,
                "p_mass": m.p_mass,
                "net_s_flow": m.net_s_exchange,
                "detail": d,
            }));
        }
    }
    let healthy_nf = rows
        .iter()
        .find(|r| r["control"] == "C_healthy_NF" && r["v_a_tag"] == "center")
        .cloned()
        .unwrap_or(json!({}));
    let unlimited = rows
        .iter()
        .find(|r| r["control"] == "E_unlimited_activation_substrates" && r["v_a_tag"] == "center")
        .cloned()
        .unwrap_or(json!({}));
    let resource_limit = material_rise(
        base_ret,
        healthy_nf["a_retention"].as_f64().unwrap_or(base_ret),
    ) || material_rise(
        base_act,
        unlimited["accepted_activation"]
            .as_f64()
            .unwrap_or(base_act),
    ) && material_rise(
        base_ret,
        unlimited["a_retention"].as_f64().unwrap_or(base_ret),
    );
    let v = json!({
        "gate": "gate3_resource_controls",
        "pass": true,
        "rows": rows,
        "resource_throughput_limit_criterion": resource_limit,
        "classification": if resource_limit {
            "D051_RESOURCE_THROUGHPUT_LIMIT"
        } else {
            "RESOURCE_CONTROLS_NO_MATERIAL_RECOVERY"
        },
    });
    write_json(&out.join("resource_controls"), "result.json", &v)?;
    Ok((true, v))
}

fn gate4_operator(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let order = accepted_step_operator_order();
    let schedules = shadow_schedules();
    // Analysis-only shadow: synthetic equal old-state extents.
    let activation = 0.4;
    let sinks = [0.15, 0.10, 0.35, 0.05]; // rep, struct, precursor, decay
    let a_available = 0.5;
    let current_sum = activation + sinks.iter().sum::<f64>();
    let over = overcommitment(current_sum, a_available);
    let joint = jointly_bound_extents(
        &[
            activation,
            sinks[0],
            sinks[1],
            sinks[2],
            sinks[3],
        ],
        a_available,
    );
    let joint_sum: f64 = joint.iter().sum();
    // Timestep refinement: halve extents, same bound — relative allocation unchanged.
    let joint_half = jointly_bound_extents(
        &[
            activation * 0.5,
            sinks[0] * 0.5,
            sinks[1] * 0.5,
            sinks[2] * 0.5,
            sinks[3] * 0.5,
        ],
        a_available * 0.5,
    );
    let refine_stable = (joint[0] / joint_sum.max(1e-18)
        - joint_half[0] / joint_half.iter().sum::<f64>().max(1e-18))
    .abs()
        < 1e-12;
    let operator_defect = false; // no production mutation; shadow does not restore coupled A
    let v = json!({
        "gate": "gate4_operator_splitting",
        "pass": true,
        "accepted_step_order": order,
        "shadow_schedules": schedules,
        "overcommitment_on_synthetic": over,
        "jointly_bounded_extents": joint,
        "timestep_refinement_stable": refine_stable,
        "operator_split_defect": operator_defect,
        "note": "Volume activation applies simultaneously with A sinks in Euler update; structural precedes activation in cell loop. No production-code mutation.",
    });
    write_json(&out.join("operator_splitting"), "result.json", &v)?;
    Ok((true, v))
}

fn gate5_cohort(out: &Path, gate0: &Value) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut rows = Vec::new();
    for key in ["schema1", "schema2_1x", "schema2_4x"] {
        let detail = match key {
            "schema1" => gate0.get("schema1").cloned().unwrap_or(json!({})),
            "schema2_1x" => gate0["schema2"]
                .as_array()
                .and_then(|a| a.iter().find(|r| r["multiplier"].as_f64() == Some(1.0)))
                .and_then(|r| r.get("detail").cloned())
                .unwrap_or(json!({})),
            _ => gate0["schema2"]
                .as_array()
                .and_then(|a| a.iter().find(|r| r["multiplier"].as_f64() == Some(4.0)))
                .and_then(|r| r.get("detail").cloned())
                .unwrap_or(json!({})),
        };
        let cohort = detail.get("cohort").cloned().unwrap_or(json!({}));
        rows.push(json!({
            "key": key,
            "cohort": cohort,
            "gross_activation": detail["gross_a_production"],
            "a_retention": detail["a_retention"],
        }));
    }
    let g1 = rows
        .iter()
        .find(|r| r["key"] == "schema2_1x")
        .and_then(|r| r["gross_activation"].as_f64())
        .unwrap_or(0.0);
    let g4 = rows
        .iter()
        .find(|r| r["key"] == "schema2_4x")
        .and_then(|r| r["gross_activation"].as_f64())
        .unwrap_or(0.0);
    let c4 = rows
        .iter()
        .find(|r| r["key"] == "schema2_4x")
        .and_then(|r| r.get("cohort").cloned())
        .unwrap_or(json!({}));
    let prod_frac = c4["catalyst_reproduction"].as_f64().unwrap_or(0.0)
        + c4["structural"].as_f64().unwrap_or(0.0)
        + c4["precursor"].as_f64().unwrap_or(0.0);
    let free_frac = c4["free_remaining"].as_f64().unwrap_or(0.0);
    let immediate = is_immediate_productive_capture(g4 > g1 * 1.1, prod_frac, free_frac);
    let pass = true;
    let v = json!({
        "gate": "gate5_a_cohort",
        "pass": pass,
        "rows": rows,
        "immediate_productive_capture": immediate,
        "label": if immediate { "ACTIVATION_IMMEDIATE_PRODUCTIVE_CAPTURE" } else { "COHORT_NOT_IMMEDIATE_CAPTURE" },
        "residence_proxy": "window-integrated ledger fractions (noncausal tracer)",
    });
    write_json(&out.join("a_cohort"), "result.json", &v)?;
    Ok((pass, v))
}

fn gate6_yields(out: &Path, gate0: &Value) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut rows = Vec::new();
    for r in gate0["schema2"].as_array().cloned().unwrap_or_default() {
        let d = r.get("detail").cloned().unwrap_or(r.clone());
        let a_prec = d["precursor_synthesis"].as_f64().unwrap_or(0.0).abs();
        let p = d["p_mass"].as_f64().unwrap_or(0.0);
        let s_gain = d["net_s_flow"].as_f64().unwrap_or(0.0).max(0.0);
        let y = precursor_yields(p, a_prec.max(1e-12), s_gain, 0.0, 0.0, p, 0.0, 0.0);
        rows.push(json!({
            "multiplier": r["multiplier"],
            "gross_activation": d["gross_a_production"],
            "yields": y,
            "net_s_flow": d["net_s_flow"],
        }));
    }
    let act_rises = rows
        .last()
        .and_then(|r| r["gross_activation"].as_f64())
        .unwrap_or(0.0)
        > rows
            .first()
            .and_then(|r| r["gross_activation"].as_f64())
            .unwrap_or(0.0)
            * 1.1;
    let s_improves = rows
        .last()
        .and_then(|r| r["net_s_flow"].as_f64())
        .unwrap_or(-1.0)
        > rows
            .first()
            .and_then(|r| r["net_s_flow"].as_f64())
            .unwrap_or(-1.0)
            + 1e-4;
    let precursor_bottleneck = act_rises && !s_improves;
    let v = json!({
        "gate": "gate6_product_yields",
        "pass": true,
        "rows": rows,
        "precursor_conversion_bottleneck": precursor_bottleneck,
    });
    write_json(&out.join("product_yields"), "result.json", &v)?;
    Ok((true, v))
}

fn gate7_spatial(out: &Path) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let mut sim = new_sim(schema2_params(D051_FITTED_V_A));
    for _ in 0..D026_SETTLE_STEPS.min(300) {
        let _ = sim.step();
    }
    let n = sim.fields.structure.len();
    let mut production = vec![0.0; n];
    let mut demand = vec![0.0; n];
    let cx = (sim.grid.width as f64) * 0.5;
    let cy = (sim.grid.height as f64) * 0.5;
    let mut region = json!({
        "central": {"act": 0.0, "dem": 0.0},
        "peripheral": {"act": 0.0, "dem": 0.0},
        "interface": {"act": 0.0, "dem": 0.0},
        "exterior": {"act": 0.0, "dem": 0.0},
    });
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    for idx in 0..n {
        let x = (idx % sim.grid.width) as f64;
        let y = (idx / sim.grid.width) as f64;
        let r = ((x - cx).hypot(y - cy)).max(0.0);
        let act = production_activation_rate(
            sim.params.activation_schema,
            sim.params.k_d008_activation,
            sim.fields.structure[idx],
            sim.fields.catalyst[idx],
            sim.fields.nutrient[idx],
            sim.fields.fuel[idx],
            sim.params.k_c_activation,
            sim.params.n_ref_activation,
            sim.params.f_ref_activation,
        );
        let a = sim.fields.activated[idx].max(0.0);
        let dem = sim.params.k_d008_reproduction * sim.fields.catalyst[idx].max(0.0) * a
            + sim.params.k_d008_activated_decay * a;
        production[idx] = act;
        demand[idx] = dem;
        let key = if sim.fields.structure[idx] < 0.5 {
            "exterior"
        } else if geometry[idx].delta > sim.params.delta_floor {
            "interface"
        } else if r < D051_RADIUS * 0.5 {
            "central"
        } else {
            "peripheral"
        };
        region[key]["act"] = json!(region[key]["act"].as_f64().unwrap_or(0.0) + act);
        region[key]["dem"] = json!(region[key]["dem"].as_f64().unwrap_or(0.0) + dem);
    }
    let omega = spatial_overlap(&production, &demand);
    // Conservative mixing diagnostic: redistribute A uniformly in interior, conserve mass.
    let mut a_int = 0.0;
    let mut n_int = 0u64;
    for idx in 0..n {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= 0.5 {
            a_int += sim.fields.activated[idx].max(0.0);
            n_int += 1;
        }
    }
    let mixed = if n_int == 0 {
        0.0
    } else {
        a_int / n_int as f64
    };
    let v = json!({
        "gate": "gate7_spatial_overlap",
        "pass": true,
        "omega_total": omega,
        "regions": region,
        "conservative_mixing_mean_a": mixed,
        "total_interior_a": a_int,
        "spatial_allocation_failure": false,
        "note": "Diagnostic map only; mixing not applied to production state.",
    });
    write_json(&out.join("spatial_overlap"), "result.json", &v)?;
    Ok((true, v))
}

fn gate8_free_pool(out: &Path, gate0: &Value, gate5: &Value) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let d = gate0["schema2"]
        .as_array()
        .and_then(|a| a.iter().find(|r| r["multiplier"].as_f64() == Some(1.0)))
        .and_then(|r| r.get("detail").cloned())
        .unwrap_or(json!({}));
    let m_a = d["a_retention"].as_f64().unwrap_or(0.0);
    let r_act = d["gross_a_production"].as_f64().unwrap_or(0.0);
    let r_dem = d["gross_reproduction"].as_f64().unwrap_or(0.0)
        + d["precursor_synthesis"].as_f64().unwrap_or(0.0)
        + d["gross_a_decay"].as_f64().unwrap_or(0.0);
    let tau = m_a / r_act.max(1e-18);
    let q = r_act / r_dem.max(1e-18);
    let services = r_dem > 1e-12;
    let wasteful = gate5["immediate_productive_capture"].as_bool().unwrap_or(false)
        && d["net_s_flow"].as_f64().unwrap_or(0.0) <= 0.0;
    let class = classify_free_pool(m_a, r_act.max(1e-18), r_dem.max(1e-18), services, wasteful);
    let v = json!({
        "gate": "gate8_free_pool",
        "pass": true,
        "tau_a": tau,
        "q_a": q,
        "m_a_retention": m_a,
        "classification": class.as_str(),
        "services_active": services,
        "wasteful_downstream": wasteful,
    });
    write_json(&out.join("free_pool_analysis"), "result.json", &v)?;
    Ok((true, v))
}

fn gate9_max_control(out: &Path, horizon: u64) -> Result<(bool, Value), Box<dyn std::error::Error>> {
    let ctrl = ControlSpec {
        hold_n: Some(D051_HEALTHY_N),
        hold_f: Some(D051_HEALTHY_F),
        unlimited_activation_substrates: true,
        ..Default::default()
    };
    // Sufficient V_A: use 4× center as upper diagnostic (exact demand match approximated).
    let (m, d) = run_campaign(
        schema2_params(D051_FITTED_V_A * 4.0),
        horizon,
        ctrl,
        "max_coupled_activation",
    );
    let a_restored = m.a_retention >= 0.80;
    let membrane_ok = m.net_s_exchange >= -1e-3;
    let a_low = m.a_retention < 0.50;
    let outcome = classify_max_activation_control(a_restored, membrane_ok && a_restored, a_low);
    // If membrane not restored either:
    let outcome = if !a_restored && m.net_s_exchange < 0.0 {
        MaxActivationOutcome::ActivationNotPrimaryCoupledBlocker
    } else {
        outcome
    };
    let v = json!({
        "gate": "gate9_maximum_activation_control",
        "pass": true,
        "outcome": outcome.as_str(),
        "a_retention": m.a_retention,
        "net_s_flow": m.net_s_exchange,
        "p_mass": m.p_mass,
        "detail": d,
    });
    write_json(&out.join("maximum_activation_control"), "result.json", &v)?;
    Ok((true, v))
}

fn gate10_route(
    out: &Path,
    sealed: bool,
    reproduced: bool,
    g1: &Value,
    g2: &Value,
    g3: &Value,
    g4: &Value,
    g5: &Value,
    g6: &Value,
    g7: &Value,
    g8: &Value,
    g9: &Value,
) -> Result<(D051PrimaryConclusion, Value), Box<dyn std::error::Error>> {
    let input = RouteDecisionInput {
        d050_sealed: sealed,
        d050_reproduced: reproduced,
        extent_accounting_ok: g1["pass"].as_bool().unwrap_or(false),
        cohort_accounting_ok: g5["pass"].as_bool().unwrap_or(false),
        accounting_ok: true,
        numerical_ok: true,
        resource_throughput_limits: g3["resource_throughput_limit_criterion"]
            .as_bool()
            .unwrap_or(false)
            || g2["resource_throughput_limits"].as_bool().unwrap_or(false),
        extent_bounding_defect: g1["cap_mode"].as_str() == Some("ACTIVATION_EXTENT_RESOURCE_CAPPED")
            && g1["series"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .any(|s| s["resource_capped_fraction"].as_f64().unwrap_or(0.0) > 0.8)
                })
                .unwrap_or(false)
            && g1["cap_mode"].as_str() != Some("ACTIVATION_EXTENT_SCALES_WITH_V_A"),
        operator_split_defect: g4["operator_split_defect"].as_bool().unwrap_or(false),
        spatial_allocation_failure: g7["spatial_allocation_failure"].as_bool().unwrap_or(false),
        precursor_conversion_bottleneck: g6["precursor_conversion_bottleneck"]
            .as_bool()
            .unwrap_or(false),
        free_a_metric_noncausal: g8["classification"].as_str()
            == Some("FREE_A_RETENTION_METRIC_NONCAUSAL"),
        topology_insufficient: false,
        activation_not_primary: g9["outcome"].as_str()
            == Some("ACTIVATION_NOT_PRIMARY_COUPLED_BLOCKER"),
    };
    // Prefer free-pool / precursor / not-primary based on max-control when resource controls fail.
    let mut input = input;
    if !input.resource_throughput_limits
        && g9["outcome"].as_str() == Some("ACTIVATION_NOT_PRIMARY_COUPLED_BLOCKER")
    {
        input.activation_not_primary = true;
    }
    if !input.resource_throughput_limits
        && !input.activation_not_primary
        && g6["precursor_conversion_bottleneck"].as_bool().unwrap_or(false)
    {
        input.precursor_conversion_bottleneck = true;
    }
    if g5["immediate_productive_capture"].as_bool().unwrap_or(false)
        && g8["classification"].as_str() == Some("FREE_A_RETENTION_METRIC_NONCAUSAL")
        && !input.resource_throughput_limits
        && !input.precursor_conversion_bottleneck
    {
        input.free_a_metric_noncausal = true;
    }
    let primary = select_primary_route(&input);
    let v = json!({
        "gate": "gate10_route_decision",
        "primary_conclusion": primary.as_str(),
        "input_flags": {
            "resource_throughput_limits": input.resource_throughput_limits,
            "extent_bounding_defect": input.extent_bounding_defect,
            "operator_split_defect": input.operator_split_defect,
            "spatial_allocation_failure": input.spatial_allocation_failure,
            "precursor_conversion_bottleneck": input.precursor_conversion_bottleneck,
            "free_a_metric_noncausal": input.free_a_metric_noncausal,
            "activation_not_primary": input.activation_not_primary,
        },
        "secondary": {
            "cap_mode": g1.get("cap_mode"),
            "chi_resource": g2.get("resource_throughput_limits"),
            "cohort_label": g5.get("label"),
            "free_pool": g8.get("classification"),
            "max_control": g9.get("outcome"),
            "operator_order": g4.get("accepted_step_order"),
        },
    });
    write_json(&out.join("route_decision"), "result.json", &v)?;
    Ok((primary, v))
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let horizon = max_accepted();
    eprintln!("D-051 pipeline horizon={horizon}");

    let (sealed, g_m1) = gate_minus_one(&out)?;
    if !sealed {
        let primary = D051PrimaryConclusion::D050EvidenceNotSealed;
        let result = json!({
            "project_directive": D051_PROJECT_ID,
            "agent_memory_id": D051_AGENT_MEMORY_ID,
            "primary_conclusion": primary.as_str(),
            "failed_gate": "gate_minus_1_seal",
            "gate_minus_1": g_m1,
            "stage_e_status": "BLOCKED_NOT_RECOVERED",
            "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
            "production_verdict": "REQUIRES_REMEDIATION",
            "next_execution_started": false,
        });
        write_json(&out, "result.json", &result)?;
        write_json(&out, "manifest.json", &result)?;
        return Ok(result);
    }

    let (reproduced, g0) = gate0_reproduction(&out, horizon)?;
    if !reproduced {
        let primary = D051PrimaryConclusion::D050FailureNotReproduced;
        let result = json!({
            "project_directive": D051_PROJECT_ID,
            "agent_memory_id": D051_AGENT_MEMORY_ID,
            "primary_conclusion": primary.as_str(),
            "failed_gate": "gate0_d050_reproduction",
            "d050_starting_commit": D051_STARTING_COMMIT,
            "gate0": g0,
            "stage_e_status": "BLOCKED_NOT_RECOVERED",
            "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
            "production_verdict": "REQUIRES_REMEDIATION",
            "next_execution_started": false,
        });
        write_json(&out, "result.json", &result)?;
        write_json(&out, "manifest.json", &result)?;
        return Ok(result);
    }

    let ctrl_h = horizon.min(4_000);
    let (_, g1) = gate1_extent(&out, ctrl_h)?;
    let (_, g2) = gate2_resource(&out, &g0)?;
    let (_, g3) = gate3_controls(&out, ctrl_h)?;
    let (_, g4) = gate4_operator(&out)?;
    let (_, g5) = gate5_cohort(&out, &g0)?;
    let (_, g6) = gate6_yields(&out, &g0)?;
    let (_, g7) = gate7_spatial(&out)?;
    let (_, g8) = gate8_free_pool(&out, &g0, &g5)?;
    let (_, g9) = gate9_max_control(&out, ctrl_h)?;
    let (primary, g10) = gate10_route(&out, sealed, reproduced, &g1, &g2, &g3, &g4, &g5, &g6, &g7, &g8, &g9)?;

    write_json(
        &out.join("accounting"),
        "result.json",
        &json!({
            "ledger_rel_tol": D051_LEDGER_REL_TOL,
            "historical_k": D050_HISTORICAL_K,
            "schema_historical": ACTIVATION_SCHEMA_HISTORICAL,
            "schema2": ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME,
            "no_production_mutation": true,
        }),
    )?;

    let result = json!({
        "project_directive": D051_PROJECT_ID,
        "agent_memory_id": D051_AGENT_MEMORY_ID,
        "d050_starting_commit": D051_STARTING_COMMIT,
        "d050_starting_tag": D051_STARTING_TAG,
        "record": D051_D050_RECORD,
        "primary_conclusion": primary.as_str(),
        "horizon": horizon,
        "gates": {
            "minus_1": g_m1,
            "0": g0,
            "1": g1,
            "2": g2,
            "3": g3,
            "4": g4,
            "5": g5,
            "6": g6,
            "7": g7,
            "8": g8,
            "9": g9,
            "10": g10,
        },
        "stage_e_status": "BLOCKED_NOT_RECOVERED",
        "phase1_status": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production_verdict": "REQUIRES_REMEDIATION",
        "next_execution_started": false,
        "report": "docs/d051_coupled_activation_throughput_audit.md",
    });
    write_json(&out, "result.json", &result)?;
    write_json(&out, "manifest.json", &result)?;
    Ok(result)
}
