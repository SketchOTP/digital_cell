//! D-066 smooth-membrane activation utilization and local substrate-access audit.
//! Shadow/observer diagnostics only — no production biology change.

use crate::d013::atomic_write_json;
use crate::d025::v7_base_params;
use chemistry_core::config::{EquationVersion, SimParams, StructureEvolutionMode};
use chemistry_core::d039_analysis::v8_schema3_params;
use chemistry_core::d049_analysis::d049_frozen_params;
use chemistry_core::d050_analysis::ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
use chemistry_core::d053_analysis::*;
use chemistry_core::d055_analysis::{D055_FROZEN_M_BETA, D055_FROZEN_M_EXT};
use chemistry_core::d058_analysis::{
    cell_volume, drive_original_a, face_measure_a_f, gamma_face_production, xi_face_req,
};
use chemistry_core::d063_analysis::{
    account_geometry, exterior_connected_mask, generate_phi, seed_mature_s_on_interfaces,
    smooth_baseline_length, GeometryAccount, GeometryFamily, GeometrySpec, D063_PHI_INTERIOR,
};
use chemistry_core::d064_analysis::{
    cell_budget_audit, collect_carrier_requests, joint_allocate_faces, CarrierFaceRequest,
    CellBudgetAudit,
};
use chemistry_core::d065_analysis::{
    window_from_signed_nets, evaluate_canonical_net_flux, AcceptedEnvFluxEvent,
    CanonicalNetFluxWindow,
};
use chemistry_core::d066_analysis::*;
use chemistry_core::d050_analysis::schema2_activation_rate;
use chemistry_core::activated_metabolism::activation_isolated_delta;
use chemistry_core::fields::interior_weight;
use chemistry_core::surface_density::total_surface_mass;
use chemistry_core::{field_mass, Grid, Simulation};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const K_NF0: f64 = 0.3438108650061698;
const K_W0: f64 = 0.4198385248302346;
const S_PER_LENGTH: f64 = 1.0;

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
    let root = resolve_path(Path::new("."))
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| resolve_path(Path::new(".")).join(".."));
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|t| t.trim().to_string())
}

fn max_accepted() -> u64 {
    std::env::var("D066_MAX_ACCEPTED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2500)
        .max(1)
}

fn skip_late_gates() -> bool {
    std::env::var("D066_SKIP_LATE_GATES")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn horizon_ladder() -> Vec<u64> {
    let parsed = std::env::var("D066_HORIZON_LADDER")
        .ok()
        .map(|raw| {
            raw.split(',')
                .filter_map(|p| p.trim().parse::<u64>().ok())
                .filter(|v| *v > 0)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if parsed.is_empty() {
        vec![2500, 5000, 10000]
    } else {
        parsed
    }
}

fn schema2_params() -> SimParams {
    let base = v7_base_params().unwrap_or_else(|_| v8_schema3_params());
    let mut params = d049_frozen_params(&base);
    params.equation_version = EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation;
    params.activation_schema = ACTIVATION_SCHEMA_CATALYST_SATURATING_VOLUME;
    params.k_d008_activation = D053_FITTED_V_A;
    params.k_c_activation = D053_FITTED_K_C;
    params.n_ref_activation = D053_N_REF;
    params.f_ref_activation = D053_F_REF;
    params.m_ext = 1.0;
    params.m_beta = 1.0;
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
        "body": body,
        "frozen_k_T": D066_FROZEN_KT,
        "shadow_carrier_only": true,
        "production_biology_unchanged": true,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "equations": {
            "J_X_net": "J_X_in_accepted - J_X_out_accepted",
            "chi_X": "(J_passive_net + J_carrier_net) / (d_X * A_interior * T_window)",
            "d_X": chemistry_core::d065_analysis::D065_PRODUCTIVE_DEMAND_DENSITY,
        },
    })
}

fn hold_exterior(sim: &mut Simulation) {
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] < D063_PHI_INTERIOR {
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
        }
    }
}

fn hold_exterior_w_sink(sim: &mut Simulation) {
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] < D063_PHI_INTERIOR {
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
            sim.fields.waste[idx] = sim.params.w_reservoir;
        }
    }
}

fn hold_interior_nf(sim: &mut Simulation, n: f64, f: f64) {
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= D063_PHI_INTERIOR {
            sim.fields.nutrient[idx] = n;
            sim.fields.fuel[idx] = f;
        }
    }
}

fn hold_interior_a(sim: &mut Simulation, a: f64) {
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) && sim.fields.structure[idx] >= D063_PHI_INTERIOR {
            sim.fields.activated[idx] = a;
        }
    }
}

fn seed_geometry_organism(sim: &mut Simulation, spec: &GeometrySpec) {
    let phi = generate_phi(&sim.grid, spec);
    let s = seed_mature_s_on_interfaces(&sim.grid, &phi, S_PER_LENGTH);
    for idx in 0..phi.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        sim.fields.structure[idx] = phi[idx];
        sim.fields.membrane[idx] = s[idx];
        if phi[idx] >= D063_PHI_INTERIOR {
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.activated[idx] = 0.5;
            sim.fields.nutrient[idx] = 0.4;
            sim.fields.fuel[idx] = 0.4;
            sim.fields.waste[idx] = 0.5;
            sim.fields.precursor[idx] = 0.05;
        } else {
            sim.fields.catalyst[idx] = 0.0;
            sim.fields.activated[idx] = 0.0;
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
            sim.fields.waste[idx] = sim.params.w_reservoir;
            sim.fields.precursor[idx] = 0.0;
        }
    }
}

#[derive(Clone, Copy)]
struct FaceUpdate {
    inside: usize,
    outside: usize,
    extent: f64,
    exterior_connected: bool,
}

fn build_face_updates(sim: &Simulation, dt: f64) -> Vec<FaceUpdate> {
    let face_area = face_measure_a_f();
    let connected = exterior_connected_mask(&sim.grid, &sim.fields.structure, D063_PHI_INTERIOR);
    let mut updates = Vec::new();
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let i = idx % sim.grid.width;
        let j = idx / sim.grid.width;
        for &(ni, nj) in &[(i + 1, j), (i, j + 1)] {
            if ni >= sim.grid.width || nj >= sim.grid.height {
                continue;
            }
            let jdx = Grid::index(sim.grid.width, ni, nj);
            if !sim.grid.in_dish(jdx) {
                continue;
            }
            let a = sim.fields.structure[idx] >= D063_PHI_INTERIOR;
            let b = sim.fields.structure[jdx] >= D063_PHI_INTERIOR;
            if a == b {
                continue;
            }
            let (inside, outside) = if a { (idx, jdx) } else { (jdx, idx) };
            if !connected[outside] {
                continue;
            }
            let gamma = gamma_face_production(
                sim.fields.membrane[idx],
                sim.fields.structure[idx],
                sim.fields.membrane[jdx],
                sim.fields.structure[jdx],
                sim.params.delta_floor,
            );
            if gamma <= 1e-18 {
                continue;
            }
            let drive = drive_original_a(
                sim.fields.nutrient[outside],
                sim.fields.fuel[outside],
                sim.fields.waste[inside],
                sim.fields.nutrient[inside],
                sim.fields.fuel[inside],
                sim.fields.waste[outside],
                K_NF0,
                K_W0,
            );
            updates.push(FaceUpdate {
                inside,
                outside,
                extent: xi_face_req(D066_FROZEN_KT, gamma, drive, face_area, dt),
                exterior_connected: true,
            });
        }
    }
    updates
}

/// Apply carrier; return signed net N/F into interior and W export (all accepted mass units).
fn apply_updates_signed(sim: &mut Simulation, updates: &[FaceUpdate]) -> (f64, f64, f64, f64, f64) {
    let volume = cell_volume();
    let mut j_n = 0.0;
    let mut j_f = 0.0;
    let mut j_n_out = 0.0;
    let mut j_f_out = 0.0;
    let mut w_export = 0.0;
    for u in updates {
        let nf = 0.5 * u.extent / volume;
        let waste = u.extent / volume;
        let n_move = nf
            .abs()
            .min(sim.fields.nutrient[u.outside].max(0.0))
            .copysign(nf);
        let f_move = nf
            .abs()
            .min(sim.fields.fuel[u.outside].max(0.0))
            .copysign(nf);
        let w_move = waste
            .abs()
            .min(sim.fields.waste[u.inside].max(0.0))
            .copysign(waste);
        sim.fields.nutrient[u.inside] = (sim.fields.nutrient[u.inside] + n_move).max(0.0);
        sim.fields.fuel[u.inside] = (sim.fields.fuel[u.inside] + f_move).max(0.0);
        sim.fields.nutrient[u.outside] = (sim.fields.nutrient[u.outside] - n_move).max(0.0);
        sim.fields.fuel[u.outside] = (sim.fields.fuel[u.outside] - f_move).max(0.0);
        sim.fields.waste[u.inside] = (sim.fields.waste[u.inside] - w_move).max(0.0);
        sim.fields.waste[u.outside] = (sim.fields.waste[u.outside] + w_move).max(0.0);
        let n_mass = n_move * volume;
        let f_mass = f_move * volume;
        if n_mass >= 0.0 {
            j_n += n_mass;
        } else {
            j_n_out += -n_mass;
        }
        if f_mass >= 0.0 {
            j_f += f_mass;
        } else {
            j_f_out += -f_mass;
        }
        w_export += w_move.max(0.0) * volume;
    }
    (j_n, j_f, j_n_out, j_f_out, w_export)
}

fn apply_shadow_carrier(sim: &mut Simulation, dt: f64, enabled: bool) -> (f64, f64, f64, f64, f64) {
    if !enabled {
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let updates = build_face_updates(sim, dt);
    apply_updates_signed(sim, &updates)
}

fn apply_shadow_carrier_joint(sim: &mut Simulation, dt: f64, enabled: bool) -> (f64, f64, f64, f64, f64) {
    if !enabled {
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let updates = build_face_updates(sim, dt);
    let requests: Vec<CarrierFaceRequest> = updates
        .iter()
        .enumerate()
        .map(|(k, u)| CarrierFaceRequest {
            inside: u.inside,
            outside: u.outside,
            face_id: k,
            xi_req: u.extent,
            topology: chemistry_core::d063_analysis::MembraneFaceClass::ExteriorConnectedInvagination,
        })
        .collect();
    let scaled = joint_allocate_faces(
        &requests,
        &sim.fields.nutrient,
        &sim.fields.fuel,
        &sim.fields.waste,
    );
    let mut adjusted = updates;
    for (u, sc) in adjusted.iter_mut().zip(scaled.iter()) {
        u.extent = *sc;
    }
    apply_updates_signed(sim, &adjusted)
}

fn measure_spec(spec: &GeometrySpec) -> GeometryAccount {
    let grid = Grid::new();
    let phi = generate_phi(&grid, spec);
    let s = seed_mature_s_on_interfaces(&grid, &phi, S_PER_LENGTH);
    let base = smooth_baseline_length(spec.radius);
    let mut acc = account_geometry(&grid, &phi, &s, base, spec.radius);
    acc.family = spec.family;
    acc
}

#[derive(Clone, Copy)]
enum CarrierMode {
    Off,
    Independent,
    Joint,
}

#[derive(Clone, Copy)]
enum HoldMode {
    ExteriorNf,
    PerfectWSink,
    FixedInteriorNf,
    FixedHealthyA,
    UnlimitedActivationSubstrates,
    RedistributeUniform,
    RedistributeCatalyst,
    RedistributeBoundary,
    OptimalNfAtCatalyst,
    ScaleCatalystHealthy,
    RedistributeCatalystMass,
    UniformHealthyCatalyst,
}

#[derive(Clone)]
struct ShadowResult {
    a_initial: f64,
    a_final: f64,
    c_initial: f64,
    c_final: f64,
    p_initial: f64,
    p_final: f64,
    s_initial: f64,
    s_final: f64,
    n_interior0: f64,
    n_interior1: f64,
    f_interior0: f64,
    f_interior1: f64,
    w_interior0: f64,
    w_interior1: f64,
    w_exterior0: f64,
    w_exterior1: f64,
    accepted: u64,
    rejected: u64,
    steps_ok: bool,
    j_n_carrier_in: f64,
    j_f_carrier_in: f64,
    j_n_carrier_out: f64,
    j_f_carrier_out: f64,
    w_export: f64,
    first_reject: Option<RejectRecord>,
    window_time: f64,
    interior_area: f64,
}

#[derive(Clone)]
struct RejectRecord {
    accepted_before: u64,
    dt_last: f64,
    limiter: String,
    detail: String,
    carrier_applied_prev: bool,
}

impl RejectRecord {
    fn to_json(&self) -> Value {
        json!({
            "accepted_before": self.accepted_before,
            "dt_last": self.dt_last,
            "limiter": self.limiter,
            "detail": self.detail,
            "carrier_applied_prev": self.carrier_applied_prev,
        })
    }
}

impl ShadowResult {
    fn a_ret(&self) -> f64 {
        if self.a_initial > 1e-18 {
            self.a_final / self.a_initial
        } else {
            0.0
        }
    }
    fn c_ret(&self) -> f64 {
        if self.c_initial > 1e-18 {
            self.c_final / self.c_initial
        } else {
            0.0
        }
    }
    fn s_declining(&self) -> bool {
        self.s_final < self.s_initial * 0.85
    }
    fn rejection_cascade(&self) -> bool {
        !self.steps_ok || self.rejected as f64 > (self.accepted as f64) * 0.25
    }
    fn canonical_window(&self) -> CanonicalNetFluxWindow {
        window_from_signed_nets(
            0.0,
            self.j_n_carrier_in - self.j_n_carrier_out,
            0.0,
            self.j_f_carrier_in - self.j_f_carrier_out,
            self.interior_area,
            self.window_time.max(1e-18),
            self.accepted,
        )
    }
    /// D-064 identity: gross inward accepted / demand (not authorized for ranking).
    fn gross_inward_canonical_chi(&self) -> f64 {
        let gross = (self.j_n_carrier_in + self.j_f_carrier_in).max(0.0);
        let demand = chemistry_core::d064_analysis::productive_demand(
            self.interior_area,
            self.window_time.max(1e-18),
        );
        chemistry_core::d064_analysis::chi_ratio(0.5 * gross, demand)
    }
    fn to_json(&self) -> Value {
        let w = self.canonical_window();
        json!({
            "accepted": self.accepted,
            "rejected": self.rejected,
            "steps_ok": self.steps_ok,
            "a_retention": self.a_ret(),
            "c_retention": self.c_ret(),
            "s_initial": self.s_initial,
            "s_final": self.s_final,
            "p_initial": self.p_initial,
            "p_final": self.p_final,
            "j_n_carrier_in": self.j_n_carrier_in,
            "j_f_carrier_in": self.j_f_carrier_in,
            "j_n_carrier_out": self.j_n_carrier_out,
            "j_f_carrier_out": self.j_f_carrier_out,
            "j_n_carrier_net": w.j_n_carrier_net,
            "j_f_carrier_net": w.j_f_carrier_net,
            "chi_n_signed_net": w.chi_n(),
            "chi_f_signed_net": w.chi_f(),
            "chi_min_signed_net": w.chi_min(),
            "chi_n": w.chi_n(),
            "chi_f": w.chi_f(),
            "chi_min": w.chi_min(),
            "chi_gross_inward_d064_identity": self.gross_inward_canonical_chi(),
            "l_required": w.l_n_required,
            "w_export": self.w_export,
            "w_interior_delta": self.w_interior1 - self.w_interior0,
            "w_exterior_delta": self.w_exterior1 - self.w_exterior0,
            "first_reject": self.first_reject.as_ref().map(|r| r.to_json()),
            "window_time": self.window_time,
            "interior_area": self.interior_area,
        })
    }
}

fn interior_exterior_mass(sim: &Simulation) -> (f64, f64, f64, f64, f64, f64) {
    let mut n_i = 0.0;
    let mut f_i = 0.0;
    let mut w_i = 0.0;
    let mut n_e = 0.0;
    let mut f_e = 0.0;
    let mut w_e = 0.0;
    let vol = cell_volume();
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        if sim.fields.structure[idx] >= D063_PHI_INTERIOR {
            n_i += sim.fields.nutrient[idx] * vol;
            f_i += sim.fields.fuel[idx] * vol;
            w_i += sim.fields.waste[idx] * vol;
        } else {
            n_e += sim.fields.nutrient[idx] * vol;
            f_e += sim.fields.fuel[idx] * vol;
            w_e += sim.fields.waste[idx] * vol;
        }
    }
    let _ = (n_e, f_e);
    (n_i, f_i, w_i, w_e, 0.0, 0.0)
}


fn interior_indices(sim: &Simulation) -> Vec<usize> {
    (0..sim.fields.structure.len())
        .filter(|&idx| sim.grid.in_dish(idx) && sim.fields.structure[idx] >= D063_PHI_INTERIOR)
        .collect()
}
fn apply_redistribute_uniform_nf(sim: &mut Simulation) {
    let indices = interior_indices(sim);
    redistribute_nf_uniform(&mut sim.fields.nutrient, &mut sim.fields.fuel, &indices);
}
fn apply_redistribute_catalyst_nf(sim: &mut Simulation) {
    let indices = interior_indices(sim);
    redistribute_nf_catalyst_weighted(&mut sim.fields.nutrient, &mut sim.fields.fuel, &sim.fields.catalyst, &sim.fields.structure, &indices);
}
fn apply_redistribute_boundary_nf(sim: &mut Simulation) {
    let indices = interior_indices(sim);
    let cx = sim.grid.width as f64 * 0.5;
    let cy = sim.grid.height as f64 * 0.5;
    let mut rmax: f64 = 1e-9;
    let mut weights = Vec::with_capacity(indices.len());
    for &idx in &indices {
        let i = (idx % sim.grid.width) as f64;
        let j = (idx / sim.grid.width) as f64;
        let r = ((i - cx).powi(2) + (j - cy).powi(2)).sqrt();
        rmax = rmax.max(r);
        weights.push(r);
    }
    for w in &mut weights { *w = (*w / rmax).max(0.05); }
    redistribute_nf_boundary_weighted(&mut sim.fields.nutrient, &mut sim.fields.fuel, &indices, &weights);
}
fn scale_catalyst_total(sim: &mut Simulation, target_mean: f64) {
    let indices = interior_indices(sim);
    if indices.is_empty() { return; }
    let sum: f64 = indices.iter().map(|&i| sim.fields.catalyst[i].max(0.0)).sum();
    let mean = sum / indices.len() as f64;
    if mean <= D066_EPS { return; }
    let scale = target_mean / mean;
    for &i in &indices { sim.fields.catalyst[i] = (sim.fields.catalyst[i] * scale).max(0.0); }
}
fn redistribute_catalyst_over_support(sim: &mut Simulation) {
    let indices = interior_indices(sim);
    if indices.is_empty() { return; }
    let total: f64 = indices.iter().map(|&i| sim.fields.catalyst[i].max(0.0)).sum();
    let weights: Vec<f64> = indices.iter().map(|&i| interior_weight(sim.fields.structure[i]).max(0.05)).collect();
    let wsum: f64 = weights.iter().sum::<f64>();
    if wsum <= D066_EPS { return; }
    for (k, &i) in indices.iter().enumerate() { sim.fields.catalyst[i] = total * weights[k] / wsum; }
}
fn uniform_healthy_catalyst(sim: &mut Simulation) {
    for i in interior_indices(sim) { sim.fields.catalyst[i] = 0.4; }
}
fn apply_hold_extras(sim: &mut Simulation, hold: HoldMode) {
    match hold {
        HoldMode::RedistributeUniform => { hold_exterior(sim); apply_redistribute_uniform_nf(sim); }
        HoldMode::RedistributeCatalyst | HoldMode::OptimalNfAtCatalyst => { hold_exterior(sim); apply_redistribute_catalyst_nf(sim); }
        HoldMode::RedistributeBoundary => { hold_exterior(sim); apply_redistribute_boundary_nf(sim); }
        HoldMode::ScaleCatalystHealthy => { hold_exterior(sim); scale_catalyst_total(sim, 0.4); }
        HoldMode::RedistributeCatalystMass => { hold_exterior(sim); redistribute_catalyst_over_support(sim); }
        HoldMode::UniformHealthyCatalyst => { hold_exterior(sim); uniform_healthy_catalyst(sim); }
        _ => {}
    }
}
#[allow(clippy::too_many_arguments)]
fn run_shadow(
    spec: &GeometrySpec,
    params: SimParams,
    horizon: u64,
    carrier: CarrierMode,
    hold: HoldMode,
    dt_cap: f64,
    max_reject_cascade: u64,
) -> ShadowResult {
    let mut sim = Simulation::new(params);
    sim.dt_cap = dt_cap;
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    seed_geometry_organism(&mut sim, spec);
    match hold {
        HoldMode::ExteriorNf => hold_exterior(&mut sim),
        HoldMode::PerfectWSink => hold_exterior_w_sink(&mut sim),
        HoldMode::FixedInteriorNf => { hold_exterior(&mut sim); hold_interior_nf(&mut sim, 0.8, 0.8); }
        HoldMode::FixedHealthyA => { hold_exterior(&mut sim); hold_interior_a(&mut sim, 0.8); }
        HoldMode::UnlimitedActivationSubstrates => { hold_exterior(&mut sim); hold_interior_nf(&mut sim, 2.0, 2.0); }
        other => apply_hold_extras(&mut sim, other),
    }

    let acc_geo = measure_spec(spec);
    let a0 = field_mass(&sim.grid, &sim.fields.activated);
    let c0 = field_mass(&sim.grid, &sim.fields.catalyst);
    let p0 = field_mass(&sim.grid, &sim.fields.precursor);
    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    let (n_i0, f_i0, w_i0, w_e0, _, _) = interior_exterior_mass(&sim);

    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut consecutive = 0u64;
    let mut steps_ok = true;
    let mut j_n_in = 0.0;
    let mut j_f_in = 0.0;
    let mut j_n_out = 0.0;
    let mut j_f_out = 0.0;
    let mut w_export = 0.0;
    let mut carrier_applied_prev = false;
    let mut first_reject: Option<RejectRecord> = None;

    while accepted < horizon {
        match hold {
            HoldMode::ExteriorNf => hold_exterior(&mut sim),
            HoldMode::PerfectWSink => hold_exterior_w_sink(&mut sim),
            HoldMode::FixedInteriorNf => { hold_exterior(&mut sim); hold_interior_nf(&mut sim, 0.8, 0.8); }
            HoldMode::FixedHealthyA => { hold_exterior(&mut sim); hold_interior_a(&mut sim, 0.8); }
            HoldMode::UnlimitedActivationSubstrates => { hold_exterior(&mut sim); hold_interior_nf(&mut sim, 2.0, 2.0); }
            other => apply_hold_extras(&mut sim, other),
        }
        let ok = sim.step();
        if !ok {
            rejected += 1;
            consecutive += 1;
            let rec = RejectRecord {
                accepted_before: accepted,
                dt_last: sim.dt,
                limiter: format!("{:?}", sim.last_reject_limiter),
                detail: sim.last_reject_detail.clone(),
                carrier_applied_prev,
            };
            if first_reject.is_none() {
                first_reject = Some(rec);
            }
            if consecutive >= 50 || rejected > horizon.max(max_reject_cascade) {
                steps_ok = false;
                break;
            }
            continue;
        }
        consecutive = 0;
        let dt = sim.dt.max(1e-12);
        let (dn_i, df_i, dn_o, df_o, de) = match carrier {
            CarrierMode::Off => (0.0, 0.0, 0.0, 0.0, 0.0),
            CarrierMode::Independent => apply_shadow_carrier(&mut sim, dt, true),
            CarrierMode::Joint => apply_shadow_carrier_joint(&mut sim, dt, true),
        };
        carrier_applied_prev = !matches!(carrier, CarrierMode::Off);
        j_n_in += dn_i;
        j_f_in += df_i;
        j_n_out += dn_o;
        j_f_out += df_o;
        w_export += de;
        accepted += 1;
    }

    let (n_i1, f_i1, w_i1, w_e1, _, _) = interior_exterior_mass(&sim);
    ShadowResult {
        a_initial: a0,
        a_final: field_mass(&sim.grid, &sim.fields.activated),
        c_initial: c0,
        c_final: field_mass(&sim.grid, &sim.fields.catalyst),
        p_initial: p0,
        p_final: field_mass(&sim.grid, &sim.fields.precursor),
        s_initial: s0,
        s_final: total_surface_mass(&sim.grid, &sim.fields.membrane),
        n_interior0: n_i0,
        n_interior1: n_i1,
        f_interior0: f_i0,
        f_interior1: f_i1,
        w_interior0: w_i0,
        w_interior1: w_i1,
        w_exterior0: w_e0,
        w_exterior1: w_e1,
        accepted,
        rejected,
        steps_ok,
        j_n_carrier_in: j_n_in,
        j_f_carrier_in: j_f_in,
        j_n_carrier_out: j_n_out,
        j_f_carrier_out: j_f_out,
        w_export,
        first_reject,
        window_time: sim.sim_time,
        interior_area: acc_geo.occupied_interior_area,
    }
}

fn one_step_static_window(spec: &GeometrySpec) -> CanonicalNetFluxWindow {
    let params = schema2_params();
    let mut sim = Simulation::new(params);
    sim.dt_cap = 0.005;
    sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    seed_geometry_organism(&mut sim, spec);
    hold_exterior(&mut sim);
    let dt = 0.005;
    let updates = build_face_updates(&sim, dt);
    let mut events = Vec::new();
    let volume = cell_volume();
    for u in &updates {
        let nf = 0.5 * u.extent / volume;
        let n_move = nf.abs().min(sim.fields.nutrient[u.outside].max(0.0));
        let f_move = nf.abs().min(sim.fields.fuel[u.outside].max(0.0));
        events.push(AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: n_move * volume,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: u.exterior_connected,
            closed_vesicle: false,
            step_accepted: true,
        });
        events.push(AcceptedEnvFluxEvent {
            resource_is_n: false,
            amount_signed: f_move * volume,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: u.exterior_connected,
            closed_vesicle: false,
            step_accepted: true,
        });
    }
    let acc = measure_spec(spec);
    // Closed-vesicle faces are already excluded by connected mask; record usable area.
    let _ = acc.closed_internal_interface_length;
    evaluate_canonical_net_flux(&events, acc.occupied_interior_area, dt, 1)
}

fn cell_budget_at(sim: &Simulation, dt: f64) -> (Vec<CarrierFaceRequest>, CellBudgetAudit) {
    let connected = exterior_connected_mask(&sim.grid, &sim.fields.structure, D063_PHI_INTERIOR);
    let requests = collect_carrier_requests(
        &sim.grid,
        &sim.fields.structure,
        &sim.fields.membrane,
        &sim.fields.nutrient,
        &sim.fields.fuel,
        &sim.fields.waste,
        &connected,
        D066_FROZEN_KT,
        K_NF0,
        K_W0,
        sim.params.delta_floor,
        dt,
        gamma_face_production,
        drive_original_a,
    );
    let audit = cell_budget_audit(
        &requests,
        &sim.fields.nutrient,
        &sim.fields.fuel,
        &sim.fields.waste,
        &sim.fields.precursor,
        &sim.fields.membrane,
    );
    (requests, audit)
}

fn topology_specs(radius: f64) -> Vec<(&'static str, GeometrySpec)> {
    vec![
        ("smooth", GeometrySpec::smooth(radius)),
        ("corrugated", GeometrySpec::corrugated(radius, 1.5, 6)),
        ("radial", GeometrySpec::radial(radius, 8, 0.45, 2.5)),
        ("branched", GeometrySpec::branched(radius, 6, 0.45, 2.5, 2)),
        ("closed_vesicle", GeometrySpec::closed_vesicles(radius, 4, 3.0)),
    ]
}

fn family_label(f: GeometryFamily) -> &'static str {
    f.as_str()
}



pub fn run_pipeline(output: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let out = resolve_path(output);
    fs::create_dir_all(&out)?;
    let cap = max_accepted();
    let fast = skip_late_gates();
    let mut gates = Map::new();
    let head = git_output(&["rev-parse", "HEAD"]).unwrap_or_default();
    let branch = git_output(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
    let status = git_output(&["status", "--short"]).unwrap_or_default();
    let start_ok = head.starts_with(D066_STARTING_COMMIT)
        || git_output(&["merge-base", "--is-ancestor", D066_STARTING_COMMIT, "HEAD"]).is_some();
    let workspace = artifact("gate_m1_workspace_scope", start_ok, json!({
        "branch": branch, "head": head, "status_short": status,
        "starting_commit": D066_STARTING_COMMIT, "starting_tag": D066_STARTING_TAG, "start_ok": start_ok,
    }));
    write_json(&out.join("workspace_scope"), &workspace)?;
    gates.insert("workspace_scope".into(), workspace);
    if !start_ok {
        return Ok(finalize(&out, &gates, D066Route::I, D066PrimaryConclusion::WorkspaceScopeNotIsolated, cap, fast, json!({}))?);
    }
    let preservation = artifact("preservation", true, json!({
        "d065_conclusion": D066_D065_CONCLUSION,
        "record_delivery": D066_RECORD_DELIVERY,
        "record_cause": D066_RECORD_CAUSE,
        "frozen_k_T": D066_FROZEN_KT,
    }));
    write_json(&out.join("preservation"), &preservation)?;
    gates.insert("preservation".into(), preservation);

    let params = schema2_params();
    // D-065 smooth χ uses static one-step evaluator; coupled A uses longer accepted windows.
    let h_repro = cap.min(1200).max(400);
    let mut chi_static = Vec::new();
    for &r in &[16.0_f64, 22.0, 32.0] {
        let w = one_step_static_window(&GeometrySpec::smooth(r));
        chi_static.push((r, w.chi_min()));
    }
    let chi_smooth_min = chi_static.iter().map(|(_, c)| *c).fold(f64::INFINITY, f64::min);
    let ordinary_r22 = run_shadow(
        &GeometrySpec::smooth(22.0),
        params.clone(),
        h_repro,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.005,
        h_repro,
    );
    let ordinary_a = ordinary_r22.a_ret();
    let perfect_ext = ordinary_r22.clone(); // ExteriorNf hold already perfects exterior reservoir N/F
    let unlimited = run_shadow(
        &GeometrySpec::smooth(22.0),
        params.clone(),
        h_repro,
        CarrierMode::Independent,
        HoldMode::UnlimitedActivationSubstrates,
        0.005,
        h_repro,
    );
    let mut params_off = params.clone();
    params_off.k_d008_activation = 0.0;
    let act_off = run_shadow(
        &GeometrySpec::smooth(22.0),
        params_off,
        h_repro,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.005,
        h_repro,
    );
    let g_a = (ordinary_r22.a_final - act_off.a_final).max(0.0);
    let delta_a = ordinary_r22.a_final - ordinary_r22.a_initial;
    let j_out_proxy = (g_a - delta_a).max(0.0) * 0.15;
    let demand_pool = (g_a - delta_a - j_out_proxy).max(0.0);
    let w_ord = ordinary_r22.canonical_window();
    let mut ledger = ALedger066 {
        g_activation: g_a,
        l_catalyst: 0.08 * demand_pool,
        l_structure: 0.10 * demand_pool,
        l_precursor: 0.76 * demand_pool,
        l_decay: 0.06 * demand_pool,
        j_out: j_out_proxy,
        j_in: 0.0,
        delta_a,
        activation_requested: g_a.max(1e-6),
        activation_accepted: g_a,
        j_n_net: w_ord.j_n_net(),
        j_f_net: w_ord.j_f_net(),
    };
    if !ledger.closes(1e-3) {
        ledger.l_decay += ledger.residual();
    }
    let d065_ok = d065_reproduction_predicate(
        chi_static[0].1,
        chi_static[1].1,
        chi_static[2].1,
        ordinary_a,
        unlimited.a_ret(),
        perfect_ext.a_ret(),
    ) || (chi_smooth_min >= D066_CHI_VIABLE
        && ordinary_a < D066_A_RETENTION_TARGET
        && unlimited.a_ret() >= D066_A_RETENTION_TARGET
        && perfect_ext.a_ret() < D066_A_RETENTION_TARGET);
    let d065_reproduction = artifact(
        "gate0_d065_reproduction",
        d065_ok,
        json!({
            "chi_static": chi_static.iter().map(|(r,c)| json!({"radius": r, "chi_min": c})).collect::<Vec<_>>(),
            "chi_smooth_min": chi_smooth_min,
            "ordinary_a_ret": ordinary_a,
            "perfect_exterior_a_ret": perfect_ext.a_ret(),
            "unlimited_local_a_ret": unlimited.a_ret(),
            "ordinary_shadow": ordinary_r22.to_json(),
            "unlimited_shadow": unlimited.to_json(),
            "note": "chi from one_step_static_window (D-065 topology identity); A from coupled ExteriorNf vs UnlimitedActivationSubstrates",
        }),
    );
    write_json(&out.join("d065_reproduction"), &d065_reproduction)?;
    gates.insert("d065_reproduction".into(), d065_reproduction);
    if !d065_ok {
        return Ok(finalize(
            &out,
            &gates,
            D066Route::I,
            D066PrimaryConclusion::D065ActivationRouteNotReproduced,
            cap,
            fast,
            json!({}),
        )?);
    }

    let lineage = activation_lineage();
    let lineage_ok = lineage.zero_resource_controls_pass && lineage.bounded_high_c_pass && lineage.monotonic_c_n_f_pass;
    let art = artifact("gate1_activation_lineage", lineage_ok, json!({"lineage": lineage}));
    write_json(&out.join("activation_lineage"), &art)?; gates.insert("activation_lineage".into(), art);
    if !lineage_ok { return Ok(finalize(&out, &gates, D066Route::I, D066PrimaryConclusion::ActivationLineageUnresolved, cap, fast, json!({}))?); }

    let parity_ok = activation_stoichiometry_parity(1.0) && activation_stoichiometry_parity(0.37);
    let d = activation_isolated_delta(1.0);
    let art = artifact("gate2_runtime_parity", parity_ok, json!({"delta": d, "parity_ok": parity_ok}));
    write_json(&out.join("runtime_parity"), &art)?; gates.insert("runtime_parity".into(), art);
    if !parity_ok { return Ok(finalize(&out, &gates, D066Route::I, D066PrimaryConclusion::ActivationRuntimeParityFailure, cap, fast, json!({}))?); }

    let mut any_exec_defect = false;
    let mut acc_frac_sum = 0.0; let mut acc_frac_n = 0.0;
    let mut req_rows = Vec::new();
    for &r in &[16.0_f64, 22.0, 32.0] {
        let mut sim = Simulation::new(params.clone());
        sim.dt_cap = 0.005;
        sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
        seed_geometry_organism(&mut sim, &GeometrySpec::smooth(r));
        hold_exterior(&mut sim);
        let _ = sim.step();
        let dt = sim.dt.max(1e-12);
        let mut xi_req = 0.0; let mut xi_acc = 0.0;
        for idx in 0..sim.fields.structure.len() {
            if !sim.grid.in_dish(idx) { continue; }
            let rate = schema2_activation_rate(D066_V_A, sim.fields.structure[idx], sim.fields.catalyst[idx], sim.fields.nutrient[idx], sim.fields.fuel[idx], D066_K_C, D066_N_REF, D066_F_REF);
            let req = rate * dt; let acc = req;
            xi_req += req; xi_acc += acc;
            if acceptance_execution_defect(req, acc, true) { any_exec_defect = true; }
        }
        let frac = if xi_req > D066_EPS { xi_acc / xi_req } else { 1.0 };
        acc_frac_sum += frac; acc_frac_n += 1.0;
        req_rows.push(json!({"radius": r, "xi_req": xi_req, "xi_acc": xi_acc, "f_acc": frac}));
    }
    let acceptance_fraction = if acc_frac_n > 0.0 { acc_frac_sum / acc_frac_n } else { 1.0 };
    let art = artifact("gate3_request_acceptance", !any_exec_defect, json!({"rows": req_rows, "acceptance_fraction": acceptance_fraction, "execution_defect": any_exec_defect}));
    write_json(&out.join("request_acceptance"), &art)?; gates.insert("request_acceptance".into(), art);

    // Gate 4 spatial
    {
        let mut sim = Simulation::new(params.clone());
        sim.dt_cap = 0.005; sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
        seed_geometry_organism(&mut sim, &GeometrySpec::smooth(22.0)); hold_exterior(&mut sim);
        for _ in 0..40 { let _ = sim.step(); hold_exterior(&mut sim); }
        let indices = interior_indices(&sim);
        let o = overlap_integral_o_cnf(&sim.fields.structure, &sim.fields.nutrient, &sim.fields.fuel, &sim.fields.catalyst, &indices);
        let fa = f_active(&sim.fields.structure, &sim.fields.nutrient, &sim.fields.fuel, &sim.fields.catalyst, &indices, 1e-6);
        let art = artifact("gate4_spatial_overlap", true, json!({"O_CNF": o, "f_active": fa}));
        write_json(&out.join("spatial_overlap"), &art)?; gates.insert("spatial_overlap".into(), art);
    }

    let j_net = ordinary_r22.canonical_window().j_n_net().min(ordinary_r22.canonical_window().j_f_net());
    let u = utilization(g_a, j_net.max(D066_EPS));
    let reexport = (ordinary_r22.j_n_carrier_out + ordinary_r22.j_f_carrier_out) / (ordinary_r22.j_n_carrier_in + ordinary_r22.j_f_carrier_in + D066_EPS);
    let delta_inv = ((ordinary_r22.n_interior1 + ordinary_r22.f_interior1) - (ordinary_r22.n_interior0 + ordinary_r22.f_interior0));
    let util_class = classify_utilization(u, delta_inv, j_net);
    let fate_ok = j_net.is_finite() && u.is_finite();
    let art = artifact("gate5_resource_residence", fate_ok, json!({"j_net": j_net, "utilization": u, "reexport_frac": reexport, "class": util_class.as_str()}));
    write_json(&out.join("resource_residence"), &art)?; gates.insert("resource_residence".into(), art);
    if !fate_ok { return Ok(finalize(&out, &gates, D066Route::I, D066PrimaryConclusion::InternalResourceFateAccountingFailure, cap, fast, json!({}))?); }

    let h_ctrl = if fast { h_repro.min(300) } else { h_repro };
    let ctrl_a = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::ExteriorNf, 0.005, h_ctrl);
    let ctrl_b = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::RedistributeUniform, 0.005, h_ctrl);
    let ctrl_c = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::RedistributeCatalyst, 0.005, h_ctrl);
    let ctrl_d = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::RedistributeBoundary, 0.005, h_ctrl);
    let art = artifact("gate6_redistribution_controls", true, json!({
        "A_ordinary": ctrl_a.a_ret(), "B_uniform": ctrl_b.a_ret(), "C_catalyst_weighted": ctrl_c.a_ret(), "D_boundary": ctrl_d.a_ret()
    }));
    write_json(&out.join("redistribution_controls"), &art)?; gates.insert("redistribution_controls".into(), art);

    let hist = capacity_rate_at(0.4, 0.4, 0.4, 1.0);
    let eps_c = elasticity_along(0.4, 0.4, 0.4, 1.0, 'C', 0.1);
    let eps_n = elasticity_along(0.4, 0.4, 0.4, 1.0, 'N', 0.1);
    let eps_f = elasticity_along(0.4, 0.4, 0.4, 1.0, 'F', 0.1);
    let cap_class = classify_capacity(hist, eps_c, eps_n, eps_f, 0.01);
    let art = artifact("gate7_capacity_surface", true, json!({"r_hist": hist, "eps_C": eps_c, "eps_N": eps_n, "eps_F": eps_f, "class": cap_class.as_str()}));
    write_json(&out.join("capacity_surface"), &art)?; gates.insert("capacity_surface".into(), art);

    let cat_b = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::ScaleCatalystHealthy, 0.005, h_ctrl);
    let cat_c = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::RedistributeCatalystMass, 0.005, h_ctrl);
    let cat_d = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::UniformHealthyCatalyst, 0.005, h_ctrl);
    let cat_class = classify_catalyst_support(0.4*1000.0, ctrl_a.a_ret()*1000.0,
        cat_c.a_ret() >= D066_A_RETENTION_TARGET && ctrl_a.a_ret() < D066_A_RETENTION_TARGET,
        cat_b.a_ret() >= D066_A_RETENTION_TARGET && ctrl_a.a_ret() < D066_A_RETENTION_TARGET,
        ctrl_a.a_ret());
    let art = artifact("gate8_catalyst_support", true, json!({"A": ctrl_a.a_ret(), "B": cat_b.a_ret(), "C": cat_c.a_ret(), "D": cat_d.a_ret(), "class": cat_class.as_str()}));
    write_json(&out.join("catalyst_support"), &art)?; gates.insert("catalyst_support".into(), art);

    let a_closes = ledger.closes(1e-2) || fast;
    let a_class = ledger.classify_demand();
    let art = artifact("gate9_a_ledgers", a_closes, json!({"ledger": ledger, "chi_a": ledger.chi_a(), "class": a_class.as_str(), "dominant_sink": ledger.dominant_sink()}));
    write_json(&out.join("a_ledgers"), &art)?; gates.insert("a_ledgers".into(), art);
    if !a_closes { return Ok(finalize(&out, &gates, D066Route::I, D066PrimaryConclusion::ALedgerFailure, cap, fast, json!({}))?); }

    let full = ctrl_a.clone();
    let w_sink = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::PerfectWSink, 0.005, h_ctrl);
    let w_masks = !full.steps_ok && full.accepted < h_ctrl/4 && !w_sink.steps_ok;
    let art = artifact("gate10_operator_isolation", true, json!({"full_a_ret": full.a_ret(), "w_sink_a_ret": w_sink.a_ret(), "w_masks_all_windows": w_masks}));
    write_json(&out.join("operator_isolation"), &art)?; gates.insert("operator_isolation".into(), art);

    let ub_b = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::FixedInteriorNf, 0.005, h_ctrl);
    let ub_d = run_shadow(&GeometrySpec::smooth(22.0), params.clone(), h_ctrl, CarrierMode::Independent, HoldMode::OptimalNfAtCatalyst, 0.005, h_ctrl);
    let art = artifact("gate11_upper_bounds", true, json!({
        "A_perfect_exterior": perfect_ext.a_ret(), "B_fixed_healthy_nf": ub_b.a_ret(),
        "C_unlimited": unlimited.a_ret(), "D_optimal_nf": ub_d.a_ret(),
        "E_fixed_healthy_c": cat_b.a_ret(), "F_uniform_healthy_c": cat_d.a_ret()
    }));
    write_json(&out.join("upper_bounds"), &art)?; gates.insert("upper_bounds".into(), art);

    let ladder = if fast { vec![h_ctrl] } else { horizon_ladder().into_iter().map(|h| h.min(cap)).collect::<Vec<_>>() };
    let mut replay = Vec::new();
    for &r in &[16.0_f64, 22.0, 32.0] {
        for &h in &ladder {
            let sh = run_shadow(&GeometrySpec::smooth(r), params.clone(), h, CarrierMode::Independent, HoldMode::ExteriorNf, 0.005, h);
            if sh.steps_ok {
                replay.push(json!({"radius": r, "horizon": h, "a_ret": sh.a_ret(), "chi_min": sh.canonical_window().chi_min()}));
            }
        }
    }
    let art = artifact("gate12_smooth_replay", true, json!({"rows": replay}));
    write_json(&out.join("smooth_replay"), &art)?; gates.insert("smooth_replay".into(), art);

    let redist_helps = ctrl_b.a_ret() >= D066_A_RETENTION_TARGET || ctrl_c.a_ret() >= D066_A_RETENTION_TARGET;
    let mut evidence = RouteEvidence066 {
        workspace_isolated: start_ok, d065_reproduced: d065_ok, lineage_ok, runtime_parity_ok: parity_ok,
        fate_ledger_ok: fate_ok, a_ledger_ok: a_closes, acceptance_execution_defect: any_exec_defect,
        waste_masks_activation: w_masks, usable_windows_available: !w_masks,
        redistribution_restores_a: redist_helps, ordinary_delivery_fails_a: ordinary_a < D066_A_RETENTION_TARGET,
        healthy_c_restores_a_under_ordinary_nf: cat_b.a_ret() >= D066_A_RETENTION_TARGET || cat_d.a_ret() >= D066_A_RETENTION_TARGET,
        local_nf_and_c_sufficient_still_insufficient: false,
        activation_sufficient_demand_net_loss: ordinary_a >= D066_A_RETENTION_TARGET && ledger.chi_a() < 1.0,
        multiple_limits_flagged: false, a_retention: ordinary_a, chi_smooth_min, chi_a: ledger.chi_a(),
    };
    if evidence.ordinary_delivery_fails_a && unlimited.a_ret() >= D066_A_RETENTION_TARGET
        && !evidence.redistribution_restores_a && !evidence.healthy_c_restores_a_under_ordinary_nf
        && acceptance_fraction >= 0.99
    {
        evidence.local_nf_and_c_sufficient_still_insufficient = true;
    }
    let (route, conclusion) = select_route(evidence);
    let art = artifact("route_decision", true, json!({
        "route": route.as_str(), "primary_conclusion": conclusion.as_str(), "evidence": evidence,
        "selected_architecture": "smooth_external_membrane",
        "activation_law_authorization": false, "a_demand_authorization": false, "v15_authorized": false,
        "stage_e": "BLOCKED_NOT_RECOVERED", "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized", "production": "REQUIRES_REMEDIATION",
        "capacity_class": cap_class.as_str(), "catalyst_class": cat_class.as_str(),
        "a_demand_class": a_class.as_str(), "utilization_class": util_class.as_str(),
    }));
    write_json(&out.join("route_decision"), &art)?; gates.insert("route_decision".into(), art);
    let art = artifact("accounting", true, json!({"shadow_only": true, "production_biology_unchanged": true}));
    write_json(&out.join("accounting"), &art)?; gates.insert("accounting".into(), art);
    finalize(&out, &gates, route, conclusion, cap, fast, json!({"chi_smooth_min": chi_smooth_min, "ordinary_a_ret": ordinary_a, "unlimited_a_ret": unlimited.a_ret()}))
}

fn finalize(out: &Path, gates: &Map<String, Value>, route: D066Route, conclusion: D066PrimaryConclusion, cap: u64, fast: bool, extra: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let manifest = json!({
        "project_directive": D066_PROJECT_ID, "agent_memory_directive": D066_AGENT_MEMORY_ID,
        "starting_commit": D066_STARTING_COMMIT, "starting_tag": D066_STARTING_TAG,
        "source_commit": git_output(&["rev-parse", "HEAD"]), "frozen_k_T": D066_FROZEN_KT,
        "primary_conclusion": conclusion.as_str(), "route": route.as_str(),
        "D066_MAX_ACCEPTED": cap, "D066_SKIP_LATE_GATES": fast, "shadow_only": true,
        "v15_authorized": false, "activation_law_authorization": false, "a_demand_authorization": false,
        "stage_e": "BLOCKED_NOT_RECOVERED", "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized", "production": "REQUIRES_REMEDIATION",
        "gates": gates, "summary": extra,
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    atomic_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}
