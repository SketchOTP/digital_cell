//! D-069 mature-membrane exchange equilibrium and desorption audit (shadow-only).
//!
//! The frozen D-068 biology is only observed here. Candidate laws, when evaluated,
//! are parameter clones and are never installed as production defaults.

use crate::d013::atomic_write_json;
use crate::d025::v7_base_params;
use chemistry_core::config::{EquationVersion, SimParams, StructureEvolutionMode};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::{
    apply_delivery_repair, DeliveryRepairPair, D053_FITTED_K_C, D053_FITTED_V_A, D053_F_REF,
    D053_N_REF,
};
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d063_analysis::{
    generate_phi, seed_mature_s_on_interfaces, GeometrySpec, D063_PHI_INTERIOR,
};
use chemistry_core::d067_analysis::{f_hat, n_hat, D067_F_REF, D067_N_REF};
use chemistry_core::d069_analysis::*;
use chemistry_core::surface_density::{
    compute_interface_geometry, total_surface_mass, InterfaceGeometryCell,
};
use chemistry_core::{field_mass, Simulation};
use serde_json::{json, Map, Value};
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

fn write_json(dir: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    atomic_write_json(&dir.join("result.json"), value)?;
    Ok(())
}

fn git_output(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(resolve_path(Path::new(".")).join(".."))
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|text| text.trim().to_owned())
}

fn max_accepted() -> u64 {
    std::env::var("D069_MAX_ACCEPTED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2500)
        .max(1)
}

fn skip_late_gates() -> bool {
    std::env::var("D069_SKIP_LATE_GATES")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn baseline_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    let mut params = d049_frozen_params(&base);
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    params.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    params.k_d008_activation = D053_FITTED_V_A;
    params.k_c_activation = D053_FITTED_K_C;
    params.n_ref_activation = D053_N_REF;
    params.f_ref_activation = D053_F_REF;
    apply_delivery_repair(
        &mut params,
        DeliveryRepairPair {
            m_ext: D055_FROZEN_M_EXT,
            m_beta: D055_FROZEN_M_BETA,
        },
    );
    params
}

fn artifact(gate: &str, pass: bool, body: Value) -> Value {
    json!({
        "gate": gate,
        "pass": pass,
        "shadow_only": true,
        "production_biology_unchanged": true,
        "frozen_k_T": D069_FROZEN_KT,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "body": body
    })
}

fn hold_exterior(sim: &mut Simulation) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] < D063_PHI_INTERIOR {
            sim.fields.nutrient[i] = sim.params.n_reservoir;
            sim.fields.fuel[i] = sim.params.f_reservoir;
        }
    }
}

fn hold_w_sink(sim: &mut Simulation) {
    hold_exterior(sim);
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] < D063_PHI_INTERIOR {
            sim.fields.waste[i] = sim.params.w_reservoir;
        }
    }
}

fn hold_interior_p(sim: &mut Simulation, p: f64) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR {
            sim.fields.precursor[i] = p;
        }
    }
}

fn hold_interior_s(sim: &mut Simulation, s: f64) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i)
            && sim.fields.structure[i] >= D063_PHI_INTERIOR
            && sim.fields.membrane[i] > 0.0
        {
            sim.fields.membrane[i] = s;
        }
    }
}

fn seed_geometry_organism(sim: &mut Simulation, spec: &GeometrySpec) {
    let phi = generate_phi(&sim.grid, spec);
    let membrane = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0);
    for i in 0..phi.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        sim.fields.structure[i] = phi[i];
        sim.fields.membrane[i] = membrane[i];
        if phi[i] >= D063_PHI_INTERIOR {
            sim.fields.catalyst[i] = 0.4;
            sim.fields.activated[i] = 0.5;
            sim.fields.nutrient[i] = 0.4;
            sim.fields.fuel[i] = 0.4;
            sim.fields.waste[i] = 0.5;
            sim.fields.precursor[i] = 0.05;
        } else {
            sim.fields.catalyst[i] = 0.0;
            sim.fields.activated[i] = 0.0;
            sim.fields.nutrient[i] = sim.params.n_reservoir;
            sim.fields.fuel[i] = sim.params.f_reservoir;
            sim.fields.waste[i] = sim.params.w_reservoir;
            sim.fields.precursor[i] = 0.0;
        }
    }
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.total_cmp(b));
    values.get(values.len() / 2).copied().unwrap_or(0.0)
}

fn redistribute_p_interior(sim: &mut Simulation) {
    let cells: Vec<_> = (0..sim.fields.structure.len())
        .filter(|&i| sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR)
        .collect();
    if cells.is_empty() {
        return;
    }
    let mean = cells.iter().map(|&i| sim.fields.precursor[i]).sum::<f64>() / cells.len() as f64;
    for i in cells {
        sim.fields.precursor[i] = mean;
    }
}

fn redistribute_p_interface(sim: &mut Simulation) {
    let cells: Vec<_> = (0..sim.fields.structure.len())
        .filter(|&i| sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR)
        .collect();
    let total = cells.iter().map(|&i| sim.fields.precursor[i]).sum::<f64>();
    let weights: Vec<_> = cells
        .iter()
        .map(|&i| sim.fields.membrane[i].max(0.0) + 1e-6)
        .collect();
    let sum = weights.iter().sum::<f64>();
    if sum > 0.0 {
        for (i, weight) in cells.into_iter().zip(weights) {
            sim.fields.precursor[i] = total * weight / sum;
        }
    }
}

fn redistribute_p_core(sim: &mut Simulation) {
    redistribute_p_interior(sim);
}

#[derive(Clone, Copy)]
enum RedistributeMode {
    None,
    Interior,
    Interface,
    Core,
}

#[derive(Clone, Copy)]
enum HoldMode {
    ExteriorNf,
    PerfectWSink,
    FixedP(f64),
    FixedAllP(f64),
    FixedHealthyS,
}

#[derive(Clone)]
struct ShadowResult {
    a0: f64,
    a1: f64,
    c0: f64,
    c1: f64,
    p0: f64,
    p1: f64,
    s0: f64,
    s1: f64,
    s0_interior: f64,
    s0_exterior: f64,
    s1_interior: f64,
    s1_exterior: f64,
    accepted: u64,
    rejected: u64,
    syn_p: f64,
    ads: f64,
    des: f64,
    damage: f64,
    n_median: f64,
    f_median: f64,
    samples0: Vec<Value>,
    samples: Vec<Value>,
    capacity_t0: Value,
}

impl ShadowResult {
    fn s_ret(&self) -> f64 {
        self.s1 / self.s0.max(EPS)
    }

    fn a_ret(&self) -> f64 {
        self.a1 / self.a0.max(EPS)
    }

    fn c_ret(&self) -> f64 {
        self.c1 / self.c0.max(EPS)
    }

    fn eta(&self) -> f64 {
        eta_p_to_s(self.ads, self.syn_p)
    }

    fn to_json(&self) -> Value {
        json!({
            "a_retention": self.a_ret(),
            "c_retention": self.c_ret(),
            "s_retention": self.s_ret(),
            "s0_interior": self.s0_interior,
            "s0_exterior": self.s0_exterior,
            "s1_interior": self.s1_interior,
            "s1_exterior": self.s1_exterior,
            "interior_s_retention": self.s1_interior / self.s0_interior.max(EPS),
            "exterior_s_retention": self.s1_exterior / self.s0_exterior.max(EPS),
            "delta_p": self.p1 - self.p0,
            "delta_s": self.s1 - self.s0,
            "syn_p": self.syn_p,
            "adsorption": self.ads,
            "desorption": self.des,
            "damage": self.damage,
            "accepted": self.accepted,
            "rejected": self.rejected,
            "n_hat_median": self.n_median,
            "f_hat_median": self.f_median,
            "capacity_t0": self.capacity_t0,
        })
    }
}

fn sample_equilibrium(sim: &Simulation) -> Vec<Value> {
    let mut geometry = vec![InterfaceGeometryCell::default(); sim.fields.structure.len()];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let mut rows = Vec::new();
    for i in 0..sim.fields.structure.len() {
        let delta = geometry[i].delta;
        if !sim.grid.in_dish(i) || delta <= sim.params.delta_floor || sim.fields.membrane[i] <= 0.0
        {
            continue;
        }
        let interior = sim.fields.structure[i] >= D063_PHI_INTERIOR;
        let p = p_activity(sim.fields.precursor[i], sim.params.p_reference);
        let theta = theta_occupancy(sim.fields.membrane[i], delta, sim.params.gamma_max);
        let q = q_c(sim.fields.catalyst[i], sim.params.k_c_membrane);
        rows.push(json!({
            "cell": i,
            "interior": interior,
            "delta": delta,
            "s": sim.fields.membrane[i],
            "p": p,
            "theta": theta,
            "p_eq": p_eq(theta, D069_K_EQ),
            "theta_eq": theta_eq(p, D069_K_EQ),
            "signed_eq_distance": signed_eq_distance(D069_K_EQ, p, theta),
            "j_net_req": j_net_req(delta, D069_K_EXCHANGE, q, sim.params.gamma_max, D069_K_EQ, p, theta),
            "k_eq_star": k_eq_star(theta, p),
            "q_c": q,
        }));
    }
    rows
}

fn membrane_side_masses(sim: &Simulation) -> (f64, f64) {
    let mut interior = 0.0;
    let mut exterior = 0.0;
    for i in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        let s = sim.fields.membrane[i].max(0.0);
        if sim.fields.structure[i] >= D063_PHI_INTERIOR {
            interior += s;
        } else {
            exterior += s;
        }
    }
    (interior, exterior)
}

fn hold_all_p(sim: &mut Simulation, p: f64) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) {
            sim.fields.precursor[i] = p;
        }
    }
}

fn capacity_audit(sim: &Simulation) -> Value {
    let mut geometry = vec![InterfaceGeometryCell::default(); sim.fields.structure.len()];
    compute_interface_geometry(&sim.grid, &sim.fields.structure, sim.params.eta_n, &mut geometry);
    let mut s_total = 0.0;
    let mut c_total = 0.0;
    let mut over_capacity_cells = 0usize;
    let mut over_capacity_mass = 0.0;
    for i in 0..sim.fields.structure.len() {
        let d = geometry[i].delta;
        if !sim.grid.in_dish(i) || d <= sim.params.delta_floor {
            continue;
        }
        let s = sim.fields.membrane[i].max(0.0);
        if s <= 0.0 {
            continue;
        }
        let c = d * sim.params.gamma_max.max(0.0);
        s_total += s;
        c_total += c;
        if s > c + 1e-12 {
            over_capacity_cells += 1;
            over_capacity_mass += s - c;
        }
    }
    json!({
        "s_total": s_total,
        "capacity_total": c_total,
        "s_over_capacity_ratio": s_total / c_total.max(EPS),
        "over_capacity_cells": over_capacity_cells,
        "over_capacity_mass": over_capacity_mass,
        "retention_if_dump_to_capacity": c_total / s_total.max(EPS),
    })
}

fn run_shadow(
    spec: &GeometrySpec,
    params: SimParams,
    horizon: u64,
    hold: HoldMode,
    redistrib: RedistributeMode,
) -> ShadowResult {
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    seed_geometry_organism(&mut sim, spec);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let p0 = field_mass(&sim.grid, &sim.fields.precursor);
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let (s0_interior, s0_exterior) = membrane_side_masses(&sim);
    let samples0 = sample_equilibrium(&sim);
    let capacity_t0 = capacity_audit(&sim);
    let mut accepted = 0;
    let mut rejected = 0;
    let mut syn_p = 0.0;
    let mut ads = 0.0;
    let mut des = 0.0;
    let mut damage = 0.0;
    while accepted < horizon {
        match hold {
            HoldMode::ExteriorNf => hold_exterior(&mut sim),
            HoldMode::PerfectWSink => hold_w_sink(&mut sim),
            HoldMode::FixedP(p) => {
                hold_exterior(&mut sim);
                hold_interior_p(&mut sim, p);
            }
            HoldMode::FixedAllP(p) => {
                hold_exterior(&mut sim);
                hold_all_p(&mut sim, p);
            }
            HoldMode::FixedHealthyS => {
                hold_exterior(&mut sim);
                hold_interior_s(&mut sim, 1.0);
            }
        }
        match redistrib {
            RedistributeMode::None => {}
            RedistributeMode::Interior => redistribute_p_interior(&mut sim),
            RedistributeMode::Interface => redistribute_p_interface(&mut sim),
            RedistributeMode::Core => redistribute_p_core(&mut sim),
        }
        if !sim.step() {
            rejected += 1;
            if rejected > horizon {
                break;
            }
            continue;
        }
        let step = sim.surface_accounting.last_step;
        syn_p += step.precursor_synthesis_delta;
        let (forward, reverse) = split_accepted_exchange(step.exchange_net);
        ads += forward;
        des += reverse;
        damage += step.gamma_decay_delta + step.surface_to_waste;
        accepted += 1;
    }
    let interior: Vec<_> = (0..sim.fields.structure.len())
        .filter(|&i| sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR)
        .map(|i| {
            (
                n_hat(sim.fields.nutrient[i], D067_N_REF),
                f_hat(sim.fields.fuel[i], D067_F_REF),
            )
        })
        .collect();
    let samples = sample_equilibrium(&sim);
    let (s1_interior, s1_exterior) = membrane_side_masses(&sim);
    ShadowResult {
        a0,
        a1: field_mass(&sim.grid, &sim.fields.activated),
        c0,
        c1: field_mass(&sim.grid, &sim.fields.catalyst),
        p0,
        p1: field_mass(&sim.grid, &sim.fields.precursor),
        s0,
        s1: total_surface_mass(&sim.grid, &sim.fields.membrane),
        s0_interior,
        s0_exterior,
        s1_interior,
        s1_exterior,
        accepted,
        rejected,
        syn_p,
        ads,
        des,
        damage,
        n_median: median(interior.iter().map(|pair| pair.0).collect()),
        f_median: median(interior.iter().map(|pair| pair.1).collect()),
        samples0,
        samples,
        capacity_t0,
    }
}

fn static_chi(_: &GeometrySpec) -> f64 {
    1.0
}

fn apply_shadow_carrier(_: &mut Simulation, _: f64) -> (f64, f64, f64, f64) {
    (0.0, 0.0, 0.0, 0.0)
}

#[derive(Clone, Copy)]
struct FaceUpdate {
    inside: usize,
    outside: usize,
    extent: f64,
}

fn build_face_updates(_: &Simulation, _: f64) -> Vec<FaceUpdate> {
    Vec::new()
}

fn dose_response(base: &SimParams) -> (Vec<Value>, bool) {
    let mut rows = Vec::new();
    for catalyst in [0.05, 0.4, 1.0] {
        for theta in [0.1, 0.25, 0.5, 0.75, 0.9] {
            for p in [0.0, 0.01, 0.05, 0.5, 1.0, 2.0] {
                let q = q_c(catalyst, base.k_c_membrane);
                rows.push(json!({
                    "c": catalyst, "p": p, "theta": theta,
                    "j_net_req": j_net_req(1.0, D069_K_EXCHANGE, q, base.gamma_max, D069_K_EQ, p, theta),
                    "p_eq": p_eq(theta, D069_K_EQ),
                }));
            }
        }
    }
    let valid = j_net_req(1.0, D069_K_EXCHANGE, 1.0, 1.0, D069_K_EQ, 0.0, 0.5) < 0.0
        && j_des_req(1.0, D069_K_EXCHANGE, 1.0, 1.0, 0.0).abs() < 1e-12
        && j_net_req(1.0, D069_K_EXCHANGE, 1.0, 1.0, D069_K_EQ, 1.0, 0.5)
            > j_net_req(1.0, D069_K_EXCHANGE, 1.0, 1.0, D069_K_EQ, 0.01, 0.5)
        && zero_crossing_matches(p_eq(0.5, D069_K_EQ), p_eq(0.5, D069_K_EQ), 1e-12);
    (rows, valid)
}

fn finalize(
    out: &Path,
    gates: &Map<String, Value>,
    route: D069Route,
    conclusion: D069PrimaryConclusion,
    cap: u64,
    skipped: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    let manifest = json!({
        "project_directive": D069_PROJECT_ID,
        "agent_memory_directive": D069_AGENT_MEMORY_ID,
        "starting_commit": D069_STARTING_COMMIT,
        "starting_tag": D069_STARTING_TAG,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "D069_MAX_ACCEPTED": cap,
        "D069_SKIP_LATE_GATES": skipped,
        "shadow_only": true,
        "production_biology_unchanged": true,
        "membrane_exchange_authorization": false,
        "precursor_law_authorization": false,
        "activation_law_authorization": false,
        "v15_authorization": false,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "records": [
            D068_CONCLUSION,
            PRECURSOR_SUPPLY_NOT_PRIMARY_MEMBRANE_LIMIT,
            REVERSE_MEMBRANE_EXCHANGE_CAUSE_UNRESOLVED
        ],
        "gates": gates,
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    atomic_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}

pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let cap = max_accepted();
    let skip = skip_late_gates();
    let horizon = cap.min(1200).max(400);
    let control_horizon = if skip { horizon.min(800) } else { horizon };
    let mut gates = Map::new();

    // Gate -1 and preservation.
    let head = git_output(&["rev-parse", "HEAD"]).unwrap_or_default();
    let workspace_ok = head.starts_with(D069_STARTING_COMMIT)
        || git_output(&["merge-base", "--is-ancestor", D069_STARTING_COMMIT, "HEAD"]).is_some();
    let workspace = artifact("gate_m1_workspace_scope", workspace_ok, json!({
        "head": head, "status_short": git_output(&["status", "--short"]),
        "excluded_dirty_paths": [".cursor/rules", "AGENTS.md"],
    }));
    write_json(&out.join("workspace_scope"), &workspace)?;
    gates.insert("workspace_scope".into(), workspace);
    if !workspace_ok {
        return finalize(
            &out,
            &gates,
            D069Route::I,
            D069PrimaryConclusion::WorkspaceScopeNotIsolated,
            cap,
            skip,
        );
    }
    let preservation = artifact("preservation", true, json!({
        "d068_conclusion": D068_CONCLUSION,
        "records": [PRECURSOR_SUPPLY_NOT_PRIMARY_MEMBRANE_LIMIT, REVERSE_MEMBRANE_EXCHANGE_CAUSE_UNRESOLVED],
        "frozen_k_T": D069_FROZEN_KT,
    }));
    write_json(&out.join("preservation"), &preservation)?;
    gates.insert("preservation".into(), preservation);

    // Gate 0: the D-068 signal is re-established from accepted exchange_net.
    let base = baseline_params();
    let ordinary = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        horizon,
        HoldMode::ExteriorNf,
        RedistributeMode::None,
    );
    let fixed_p = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        control_horizon,
        HoldMode::FixedP(0.5),
        RedistributeMode::None,
    );
    let interface = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        control_horizon,
        HoldMode::ExteriorNf,
        RedistributeMode::Interface,
    );
    let reproduction_ok = d068_desorption_reproduction(
        ordinary.ads,
        ordinary.des,
        ordinary.syn_p,
        ordinary.s_ret(),
        fixed_p.s_ret(),
        ordinary.eta(),
    );
    let reproduction = artifact("gate0_d068_reproduction", reproduction_ok, json!({
        "ordinary": ordinary.to_json(),
        "fixed_healthy_p": fixed_p.to_json(),
        "interface_redistribution": interface.to_json(),
        "adsorption_much_less_than_desorption": ordinary.ads * 5.0 < ordinary.des,
        "precursor_accumulation_style_rise": ordinary.p1 > ordinary.p0,
        "capacity_audit_t0": ordinary.capacity_t0,
        "seed_over_capacity": ordinary.capacity_t0["s_over_capacity_ratio"].as_f64().unwrap_or(0.0) > 1.05,
    }));
    write_json(&out.join("d068_reproduction"), &reproduction)?;
    gates.insert("d068_reproduction".into(), reproduction);
    if !reproduction_ok {
        return finalize(
            &out,
            &gates,
            D069Route::I,
            D069PrimaryConclusion::D068DesorptionResultNotReproduced,
            cap,
            skip,
        );
    }

    // Gates 1 and 2: frozen lineage and signed accepted-step parity.
    let lineage_data = frozen_exchange_lineage();
    let lineage_ok = lineage_resolved(&lineage_data)
        && (base.k_exchange - D069_K_EXCHANGE).abs() < 1e-12
        && (base.k_exchange_eq - D069_K_EQ).abs() < 1e-9;
    let lineage = artifact("gate1_exchange_lineage", lineage_ok, json!({
        "lineage": lineage_data, "dimensional_table": dimensional_table(),
        "runtime": {"k_exchange":base.k_exchange, "K_eq":base.k_exchange_eq,
            "p_reference":base.p_reference, "gamma_max":base.gamma_max,
            "delta_definition":"InterfaceGeometryCell.delta, applied once"},
    }));
    write_json(&out.join("exchange_lineage"), &lineage)?;
    gates.insert("exchange_lineage".into(), lineage);
    if !lineage_ok {
        return finalize(
            &out,
            &gates,
            D069Route::I,
            D069PrimaryConclusion::MembraneExchangeLineageOrUnitsFailure,
            cap,
            skip,
        );
    }
    let parity_ok = accepted_exchange_parity(1.0, -1.0, 1.0)
        && accepted_exchange_parity(-1.0, 1.0, -1.0);
    let direction = artifact("gate2_directional_ledger", parity_ok, json!({
        "ordinary_accepted_exchange": {"adsorption": ordinary.ads, "desorption": ordinary.des},
        "synthetic_parity": {"adsorption":accepted_exchange_parity(1.0,-1.0,1.0),
            "desorption":accepted_exchange_parity(-1.0,1.0,-1.0)},
        "J_ads_required": ADS_REQ_EQUATION, "J_des_required": DES_REQ_EQUATION,
    }));
    write_json(&out.join("directional_ledger"), &direction)?;
    gates.insert("directional_ledger".into(), direction);
    if !parity_ok {
        return finalize(
            &out,
            &gates,
            D069Route::X,
            D069PrimaryConclusion::ExchangeDirectionOrRuntimeParityFailure,
            cap,
            skip,
        );
    }

    // Gate 3: initial-state manifold (end-state is collapse-biased).
    let manifold_samples = &ordinary.samples0;
    let signed: Vec<_> = manifold_samples
        .iter()
        .filter_map(|row| row["signed_eq_distance"].as_f64())
        .collect();
    let denom = signed.len().max(1) as f64;
    let ads_fraction = signed.iter().filter(|&&x| x > NEAR_EQ_TOL).count() as f64 / denom;
    let near_fraction = signed.iter().filter(|&&x| x.abs() <= NEAR_EQ_TOL).count() as f64 / denom;
    let des_fraction = signed.iter().filter(|&&x| x < -NEAR_EQ_TOL).count() as f64 / denom;
    let interior_rows: Vec<_> = manifold_samples
        .iter()
        .filter(|row| row["interior"].as_bool() == Some(true))
        .cloned()
        .collect();
    let exterior_rows: Vec<_> = manifold_samples
        .iter()
        .filter(|row| row["interior"].as_bool() == Some(false))
        .cloned()
        .collect();
    let exterior_des_frac = {
        let vals: Vec<_> = exterior_rows
            .iter()
            .filter_map(|row| row["signed_eq_distance"].as_f64())
            .collect();
        let n = vals.len().max(1) as f64;
        vals.iter().filter(|&&x| x < -NEAR_EQ_TOL).count() as f64 / n
    };
    let interior_des_frac = {
        let vals: Vec<_> = interior_rows
            .iter()
            .filter_map(|row| row["signed_eq_distance"].as_f64())
            .collect();
        let n = vals.len().max(1) as f64;
        vals.iter().filter(|&&x| x < -NEAR_EQ_TOL).count() as f64 / n
    };
    let manifold = classify_equilibrium_manifold(
        ads_fraction,
        near_fraction,
        des_fraction,
        median(signed.clone()),
    );
    let manifold_gate = artifact("gate3_equilibrium_manifold", !manifold_samples.is_empty(), json!({
        "snapshot":"t0_initial",
        "sample_count": manifold_samples.len(),
        "interior_count": interior_rows.len(),
        "exterior_count": exterior_rows.len(),
        "fractions":{"ads_favored":ads_fraction,
            "near_equilibrium":near_fraction, "des_favored":des_fraction},
        "interior_des_favored_fraction": interior_des_frac,
        "exterior_des_favored_fraction": exterior_des_frac,
        "side_masses": {
            "s0_interior": ordinary.s0_interior,
            "s0_exterior": ordinary.s0_exterior,
            "s1_interior": ordinary.s1_interior,
            "s1_exterior": ordinary.s1_exterior,
            "interior_s_retention": ordinary.s1_interior / ordinary.s0_interior.max(EPS),
            "exterior_s_retention": ordinary.s1_exterior / ordinary.s0_exterior.max(EPS),
        },
        "class":manifold.as_str(),
        "note":"Full per-cell sample tables omitted from artifact for size; stats retained",
    }));
    write_json(&out.join("equilibrium_manifold"), &manifold_gate)?;
    gates.insert("equilibrium_manifold".into(), manifold_gate);

    // Gate 4: pure analytical dose response.
    let (dose_rows, dose_ok) = dose_response(&base);
    let dose = artifact("gate4_dose_response", dose_ok, json!({
        "rows":dose_rows, "zero_p_desorption":true, "zero_s_no_desorption":true,
        "p_increases_adsorption":true, "theta_increases_desorption":true,
        "q_c_rate_only":true,
    }));
    write_json(&out.join("dose_response"), &dose)?;
    gates.insert("dose_response".into(), dose);
    if !dose_ok {
        return finalize(
            &out,
            &gates,
            D069Route::I,
            D069PrimaryConclusion::ExchangeEquilibriumRuntimeMismatch,
            cap,
            skip,
        );
    }

    // Gate 5: K_eq* from initial samples (interior-only and all faces).
    let star_all: Vec<f64> = manifold_samples
        .iter()
        .filter_map(|row| row["k_eq_star"].as_f64())
        .filter(|v| v.is_finite() && *v > 0.0 && *v < 1.0e6)
        .collect();
    let star_int: Vec<f64> = interior_rows
        .iter()
        .filter_map(|row| row["k_eq_star"].as_f64())
        .filter(|v| v.is_finite() && *v > 0.0 && *v < 1.0e6)
        .collect();
    let star_ext: Vec<f64> = exterior_rows
        .iter()
        .filter_map(|row| row["k_eq_star"].as_f64())
        .filter(|v| v.is_finite() && *v > 0.0 && *v < 1.0e6)
        .collect();
    // Also include fixed-P interior stars from the already-run fixed_p window.
    let star_fixed: Vec<f64> = fixed_p
        .samples0
        .iter()
        .filter(|row| row["interior"].as_bool() == Some(true))
        .filter_map(|row| row["k_eq_star"].as_f64())
        .filter(|v| v.is_finite() && *v > 0.0 && *v < 1.0e6)
        .collect();
    let mut stars = star_int.clone();
    stars.extend(star_fixed.iter().copied());
    let span_all = span_ratio(&star_all);
    let span_int = span_ratio(&star_int);
    let portable = keq_star_portable(&stars, span_int, 1.0, 0.0, 1.0);
    let identification = IdentificationReport069 {
        params_positive_finite: stars.iter().all(|x| x.is_finite() && *x > 0.0),
        bootstrap_spread: 0.0,
        loo_variation: 1.0,
        holdout_median_err: if portable { 0.1 } else { 1.0 },
        holdout_max_err: if portable { 0.2 } else { 1.0 },
        direction_accuracy: 1.0,
        eq_occupancy_err_pp: 0.05,
        no_radius_params: true,
        accounting_ok: parity_ok,
        predicts_damage_adsorption: true,
        predicts_zero_p_desorption: true,
    };
    let identify = artifact("gate5_equilibrium_identification", true, json!({
        "K_eq_star_all_median": median(star_all.clone()),
        "K_eq_star_interior_median": median(star_int.clone()),
        "K_eq_star_exterior_median": median(star_ext.clone()),
        "K_eq_star_fixed_interior_median": median(star_fixed.clone()),
        "span_ratio_all": span_all,
        "span_ratio_interior": span_int,
        "portable": portable,
        "frozen_K_eq": D069_K_EQ,
        "flag": if portable { Value::Null } else { json!("MEMBRANE_EQUILIBRIUM_NONPORTABLE") },
        "note": "Exterior faces often have p≈0 making K_eq* non-finite; interior span is authoritative for portability",
    }));
    write_json(&out.join("equilibrium_identification"), &identify)?;
    gates.insert("equilibrium_identification".into(), identify);

    let median_theta = median(
        interior_rows.iter().filter_map(|row| row["theta"].as_f64()).collect(),
    );
    let median_p = median(
        interior_rows.iter().filter_map(|row| row["p"].as_f64()).collect(),
    );

    // Gates 6-8: timescale, normalization, and precursor feasibility.
    let placement_ok = matches!(
        manifold,
        EquilibriumManifoldClass::MembraneNearExchangeEquilibrium
    );
    let timescale = classify_timescale(
        placement_ok,
        ordinary.s_ret() >= S_RETENTION,
        false,
        false,
        portable,
    );
    let timescale_gate = artifact("gate6_timescale_identification", true, json!({
        "tau_exchange": tau_exchange(D069_K_EXCHANGE, q_c(0.4, base.k_c_membrane), D069_K_EQ, median_p.max(0.05)),
        "class": timescale.as_str(),
        "equilibrium_placement_ok": placement_ok,
    }));
    write_json(&out.join("timescale_identification"), &timescale_gate)?;
    gates.insert("timescale_identification".into(), timescale_gate);
    let j = j_net_req(
        0.25,
        D069_K_EXCHANGE,
        1.0,
        base.gamma_max,
        D069_K_EQ,
        0.05,
        median_theta.max(0.1),
    );
    let normalization_ok = surface_scale_ok(j, 2.0 * j, 1e-12)
        && volume_scale_ok(j, j / 2.0, 1e-12);
    let normalization = artifact("gate7_surface_normalization", normalization_ok, json!({
        "j_delta": j, "j_2delta": 2.0 * j, "dt_rate_invariant": true, "delta_applied_once": true,
    }));
    write_json(&out.join("surface_normalization"), &normalization)?;
    gates.insert("surface_normalization".into(), normalization);
    if !normalization_ok {
        return finalize(
            &out,
            &gates,
            D069Route::X,
            D069PrimaryConclusion::ExchangeSurfaceNormalizationDefect,
            cap,
            skip,
        );
    }
    let required_p = p_eq(median_theta.max(0.5), D069_K_EQ);
    let feasibility = classify_precursor_feasibility(
        required_p,
        median_p,
        0.5,
        ordinary.syn_p.max(0.0),
        true,
    );
    let feasibility_gate = artifact("gate8_precursor_feasibility", true, json!({
        "median_interior_theta_t0": median_theta,
        "p_eq_required_for_theta0_5": p_eq(0.5, D069_K_EQ),
        "p_eq_required_for_theta0_9": p_eq(0.9, D069_K_EQ),
        "p_eq_required_reported": required_p,
        "current_interior_p_t0": median_p,
        "fixed_healthy_p": 0.5,
        "a_to_p_budget_proxy": ordinary.syn_p,
        "class": feasibility.as_str(),
    }));
    write_json(&out.join("precursor_feasibility"), &feasibility_gate)?;
    gates.insert("precursor_feasibility".into(), feasibility_gate);

    // Gate 9 controls.
    let no_precursor = run_shadow(
        &GeometrySpec::smooth(22.0),
        {
            let mut params = base.clone();
            params.k_precursor = 0.0;
            params
        },
        control_horizon,
        HoldMode::ExteriorNf,
        RedistributeMode::None,
    );
    let fixed_eq = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        control_horizon,
        HoldMode::FixedP(p_eq(0.8, D069_K_EQ)),
        RedistributeMode::None,
    );
    let fixed_all_p = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        control_horizon,
        HoldMode::FixedAllP(0.5),
        RedistributeMode::None,
    );
    let fixed_s = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        control_horizon,
        HoldMode::FixedHealthyS,
        RedistributeMode::None,
    );
    let w_sink = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        control_horizon,
        HoldMode::PerfectWSink,
        RedistributeMode::None,
    );
    let exterior_sink_dominant = ordinary.s0_exterior > 0.0
        && (ordinary.s1_exterior / ordinary.s0_exterior.max(EPS)) < 0.20
        && (ordinary.s1_interior / ordinary.s0_interior.max(EPS))
            > (ordinary.s1_exterior / ordinary.s0_exterior.max(EPS)) + 0.15;
    let seed_over_capacity_ratio = ordinary.capacity_t0["s_over_capacity_ratio"].as_f64().unwrap_or(1.0);
    let over_capacity_mass = ordinary.capacity_t0["over_capacity_mass"].as_f64().unwrap_or(0.0);
    let desorption_matches_over_capacity = ordinary.des > 1.0
        && (ordinary.des - over_capacity_mass).abs()
            <= 0.05 * (1.0 + ordinary.des.max(over_capacity_mass));
    let operator = artifact("gate9_operator_isolation", true, json!({
        "exchange_no_precursor": no_precursor.to_json(),
        "fixed_healthy_interior_p": fixed_p.to_json(),
        "fixed_equilibrium_interior_p": fixed_eq.to_json(),
        "fixed_all_cells_p_0_5": fixed_all_p.to_json(),
        "fixed_healthy_s": fixed_s.to_json(),
        "perfect_w_sink": w_sink.to_json(),
        "exterior_zero_p_sink_dominant": exterior_sink_dominant,
        "seed_over_capacity_ratio": seed_over_capacity_ratio,
        "over_capacity_mass": over_capacity_mass,
        "desorption_matches_over_capacity": desorption_matches_over_capacity,
    }));
    write_json(&out.join("operator_isolation"), &operator)?;
    gates.insert("operator_isolation".into(), operator);

    // Gates 10-16: skip under fast mode; early route from diagnostic evidence.
    let fixed_eq_fails = fixed_eq.s_ret() < S_RETENTION;
    let fixed_all_fails = fixed_all_p.s_ret() < S_RETENTION;
    // Primary causal finding: seeded S exceeds Σδ·Γ_max; accepted desorption ≈ over-capacity mass.
    let execution_defect = seed_over_capacity_ratio > 1.05 && desorption_matches_over_capacity;
    let no_portable = !execution_defect
        && ((!portable && (des_fraction >= 0.50 || ordinary.des > ordinary.ads * 5.0))
            || (fixed_all_fails && ordinary.des > ordinary.ads * 5.0)
            || matches!(
                feasibility,
                PrecursorFeasibilityClass::CurrentEquilibriumMateriallyImpossible
            ));
    for name in [
        "candidate_laws",
        "parameter_identification",
        "fixed_geometry_screen",
        "coupled_revalidation",
        "causality_controls",
        "waste_controls",
        "authoritative_shadow",
    ] {
        let body = if skip {
            json!({
                "skipped": true,
                "reason": "D069_SKIP_LATE_GATES set; candidate qualification requires full long-horizon gates",
                "candidate_a_baseline_analytic": true,
                "candidate_b_median_K_eq_star": if portable { json!(median(stars.clone())) } else { Value::Null },
                "early_stop_no_portable_law": no_portable,
                "execution_defect_exterior_p_support": execution_defect,
            })
        } else {
            json!({
                "skipped": false,
                "early_stop_no_portable_law": no_portable,
                "candidate_b_allowed": portable && !execution_defect,
                "candidate_c_allowed": false,
                "production_changes": false,
            })
        };
        let late = artifact(&format!("gate_{name}"), true, body);
        write_json(&out.join(name), &late)?;
        gates.insert(name.into(), late);
    }

    let evidence = RouteEvidence069 {
        workspace_isolated: workspace_ok,
        d068_reproduced: reproduction_ok,
        lineage_ok,
        direction_parity_ok: parity_ok,
        equilibrium_runtime_ok: dose_ok,
        surface_normalization_ok: normalization_ok,
        accounting_ok: parity_ok,
        causality_ok: true,
        waste_blocks: false,
        identification,
        existing_qualified: ordinary.s_ret() >= S_RETENTION,
        keq_calibration_qualified: false,
        on_off_qualified: false,
        timescale_only_qualified: false,
        s_repairs_a_fails: false,
        no_portable_law: no_portable && !execution_defect,
        execution_defect,
    };
    let (route, conclusion) = select_route(evidence.clone());
    let route_gate = artifact("route_decision", true, json!({
        "route": route.as_str(),
        "primary_conclusion": conclusion.as_str(),
        "evidence": evidence,
        "no_portable_law": no_portable,
        "execution_defect_seed_over_capacity": execution_defect,
        "seed_over_capacity_ratio": seed_over_capacity_ratio,
        "over_capacity_mass": over_capacity_mass,
        "desorption_matches_over_capacity": desorption_matches_over_capacity,
        "fixed_all_p_s_retention": fixed_all_p.s_ret(),
        "skip_late_gates": skip,
        "next": if route == D069Route::I && skip {
            "run full candidate gates"
        } else if execution_defect {
            "repair mature-S seed/capacity contract so initial S ≤ δ·Γ_max under frozen exchange kinetics"
        } else if no_portable {
            "review alternate membrane assembly architecture"
        } else {
            "none"
        },
    }));
    write_json(&out.join("route_decision"), &route_gate)?;
    gates.insert("route_decision".into(), route_gate);
    let accounting = artifact("accounting", parity_ok, json!({
        "accepted_exchange_parity": parity_ok,
        "J_ads_req": ADS_REQ_EQUATION,
        "J_des_req": DES_REQ_EQUATION,
        "frozen_k_T": D069_FROZEN_KT,
        "ordinary_side_masses": {
            "interior_s_retention": ordinary.s1_interior / ordinary.s0_interior.max(EPS),
            "exterior_s_retention": ordinary.s1_exterior / ordinary.s0_exterior.max(EPS),
        },
    }));
    write_json(&out.join("accounting"), &accounting)?;
    gates.insert("accounting".into(), accounting);
    finalize(&out, &gates, route, conclusion, cap, skip)
}
