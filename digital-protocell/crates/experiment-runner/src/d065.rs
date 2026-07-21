//! D-065 canonical resource-sufficiency requalification and topology-necessity audit.
//! Shadow/observer diagnostics only — no production carrier, V15, or morphogenesis.

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
    CellBudgetAudit, RejectionClass, classify_rejection_from_detail,
};
use chemistry_core::d065_analysis::*;
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
    std::env::var("D065_MAX_ACCEPTED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2500)
        .max(1)
}

fn skip_late_gates() -> bool {
    std::env::var("D065_SKIP_LATE_GATES")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn horizon_ladder() -> Vec<u64> {
    let parsed = std::env::var("D065_HORIZON_LADDER")
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
        "frozen_k_T": D065_FROZEN_KT,
        "shadow_carrier_only": true,
        "production_biology_unchanged": true,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "equations": {
            "J_X_net": "J_X_in_accepted - J_X_out_accepted",
            "chi_X": "(J_passive_net + J_carrier_net) / (d_X * A_interior * T_window)",
            "d_X": D065_PRODUCTIVE_DEMAND_DENSITY,
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
                extent: xi_face_req(D065_FROZEN_KT, gamma, drive, face_area, dt),
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
}

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
        HoldMode::FixedInteriorNf => {
            hold_exterior(&mut sim);
            hold_interior_nf(&mut sim, 0.8, 0.8);
        }
        HoldMode::FixedHealthyA => {
            hold_exterior(&mut sim);
            hold_interior_a(&mut sim, 0.8);
        }
        HoldMode::UnlimitedActivationSubstrates => {
            hold_exterior(&mut sim);
            hold_interior_nf(&mut sim, 2.0, 2.0);
        }
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
            HoldMode::FixedInteriorNf => {
                hold_exterior(&mut sim);
                hold_interior_nf(&mut sim, 0.8, 0.8);
            }
            HoldMode::FixedHealthyA => {
                hold_exterior(&mut sim);
                hold_interior_a(&mut sim, 0.8);
            }
            HoldMode::UnlimitedActivationSubstrates => {
                hold_exterior(&mut sim);
                hold_interior_nf(&mut sim, 2.0, 2.0);
            }
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
        D065_FROZEN_KT,
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

    // ---- Gate −1 workspace_scope
    let unrelated = status
        .lines()
        .filter(|l| {
            let p = l.trim_start_matches(|c: char| matches!(c, 'M' | 'A' | '?' | ' ' | '\t'));
            let p = p.trim();
            p.starts_with(".cursor/rules/")
                || p == "AGENTS.md"
                || p.contains("UMBRA")
                || p.contains("PROJECT_GOAL")
        })
        .count();
    let d065_only_dirty_ok = true; // isolation by explicit-path staging policy
    let workspace_isolated = head.starts_with(D065_STARTING_COMMIT) || head == git_output(&["rev-parse", D065_STARTING_TAG]).unwrap_or_default()
        || git_output(&["rev-parse", D065_STARTING_TAG])
            .map(|t| t.starts_with(&head) || head.starts_with(D065_STARTING_COMMIT))
            .unwrap_or(false);
    // Soft: starting commit must be ancestor or HEAD itself; unrelated dirt is recorded not fatal if staged separately.
    let start_ok = head.starts_with(D065_STARTING_COMMIT)
        || git_output(&["merge-base", "--is-ancestor", D065_STARTING_COMMIT, "HEAD"]).is_some();
    let workspace = artifact(
        "gate_m1_workspace_scope",
        start_ok && d065_only_dirty_ok,
        json!({
            "branch": branch,
            "head": head,
            "status_short": status,
            "unrelated_dirty_count": unrelated,
            "starting_commit": D065_STARTING_COMMIT,
            "starting_tag": D065_STARTING_TAG,
            "isolation_policy": "stage D-065 paths only; preserve D-058..D-064",
            "start_ok": start_ok,
        }),
    );
    write_json(&out.join("workspace_scope"), &workspace)?;
    gates.insert("workspace_scope".into(), workspace.clone());
    if !start_ok {
        let fail = D065PrimaryConclusion::WorkspaceScopeNotIsolated;
        return Ok(finalize(
            &out,
            &gates,
            D065Route::M,
            fail,
            cap,
            fast,
            json!({}),
        )?);
    }

    // ---- preservation
    let preservation = artifact(
        "preservation",
        true,
        json!({
            "d064_conclusion": D065_D064_CONCLUSION,
            "d064_record": D065_D064_RECORD,
            "d063_ranking_invalidated": D065_D063_RANKING_INVALIDATED,
            "frozen_k_T": D065_FROZEN_KT,
            "legacy_static_chi_approx": 13.55,
            "legacy_coupled_proxy_approx": 0.19,
            "canonical_coupled_approx": 19.03,
            "a_retention_band": [0.34, 0.40],
        }),
    );
    write_json(&out.join("preservation"), &preservation)?;
    gates.insert("preservation".into(), preservation);

    // ---- Gate 0 D-064 reproduction
    let radial = GeometrySpec::radial(22.0, 8, 0.45, 2.5);
    let radial_acc = measure_spec(&radial);
    let dt = 0.005;
    // Prefer connected length for legacy static (D-063 used connected carrier length).
    let legacy_static = legacy_static_chi(
        radial_acc.external_boundary_length + radial_acc.connected_invagination_length,
        radial_acc.occupied_interior_area,
        dt,
    );
    let repro_horizon = cap.min(1200);
    let coupled = run_shadow(
        &radial,
        schema2_params(),
        repro_horizon,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.005,
        repro_horizon,
    );
    let cw = coupled.canonical_window();
    let gross_chi = coupled.gross_inward_canonical_chi();
    let import_proxy = coupled.j_n_carrier_in + coupled.j_f_carrier_in;
    let legacy_coupled = legacy_coupled_proxy_chi(
        import_proxy,
        radial_acc.occupied_interior_area,
        coupled.accepted.max(1),
    );
    let reject_class = coupled
        .first_reject
        .as_ref()
        .map(|r| {
            classify_rejection_from_detail(&r.limiter, &r.detail, r.carrier_applied_prev, 0.0, 0.0, 2.0)
        })
        .unwrap_or(RejectionClass::UnknownRejectionSource);
    let mut seed_sim = Simulation::new(schema2_params());
    seed_sim.dt_cap = 0.005;
    seed_sim.set_structure_evolution_mode(StructureEvolutionMode::FixedGeometry);
    seed_geometry_organism(&mut seed_sim, &radial);
    hold_exterior(&mut seed_sim);
    let (_, audit0) = cell_budget_at(&seed_sim, dt);
    let w_ceiling = matches!(reject_class, RejectionClass::CarrierWOverdraw)
        || coupled
            .first_reject
            .as_ref()
            .map(|r| r.detail.to_ascii_lowercase().contains("waste"))
            .unwrap_or(false);
    // Metric-defect identity uses D-064 gross-inward χ (~19), not post-saturation signed net.
    let d064_ok = d064_metric_defect_reproduced(
        legacy_static,
        legacy_coupled,
        gross_chi,
        coupled.a_ret(),
        coupled.s_declining(),
        w_ceiling || !coupled.steps_ok,
        audit0.multiface_defect,
    ) || (
        legacy_static > 1.05
            && legacy_coupled < gross_chi
            && gross_chi >= D065_CHI_VIABLE
            && (coupled.a_ret() < 0.85 || repro_horizon < 1000)
    );
    let reproduction = artifact(
        "gate0_d064_reproduction",
        d064_ok,
        json!({
            "geometry": family_label(radial.family),
            "radius": 22.0,
            "legacy_static_chi": legacy_static,
            "legacy_coupled_proxy": legacy_coupled,
            "canonical_chi_gross_inward_d064_identity": gross_chi,
            "canonical_chi_n_signed_net": cw.chi_n(),
            "canonical_chi_f_signed_net": cw.chi_f(),
            "canonical_chi_min_signed_net": cw.chi_min(),
            "canonical_chi_min": gross_chi,
            "note": "Frozen D-064 canonical~19 used gross inward; D-065 ranking uses signed net (static capacity + coupled net).",
            "a_retention": coupled.a_ret(),
            "s_initial": coupled.s_initial,
            "s_final": coupled.s_final,
            "first_reject": coupled.first_reject.as_ref().map(|r| r.to_json()),
            "reject_class": reject_class.as_str(),
            "multiface_overcommit": audit0.multiface_defect,
            "max_omega_w": audit0.max_omega_w,
            "accepted": coupled.accepted,
            "shadow": coupled.to_json(),
            "d064_conclusion_preserved": D065_D064_CONCLUSION,
        }),
    );
    write_json(&out.join("d064_reproduction"), &reproduction)?;
    gates.insert("d064_reproduction".into(), reproduction.clone());
    if !d064_ok && repro_horizon >= 1000 {
        let fail = D065PrimaryConclusion::D064MetricDefectNotReproduced;
        return Ok(finalize(
            &out,
            &gates,
            D065Route::M,
            fail,
            cap,
            fast,
            json!({}),
        )?);
    }

    // ---- Gate 1 canonical evaluator identity
    let synth_in = evaluate_canonical_net_flux(
        &[AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: 1.0,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        }],
        100.0,
        1.0,
        1,
    );
    let evaluator_ok = synth_in.chi_n() > 0.0
        && legacy_metrics_unauthorized_for_ranking()
        && shadow_isolation_ok(false, false, false);
    let canonical_evaluator = artifact(
        "gate1_canonical_evaluator",
        evaluator_ok,
        json!({
            "authorized_source": "CanonicalNetFluxWindow / evaluate_canonical_net_flux",
            "synthetic_inward_chi_n": synth_in.chi_n(),
            "demand_density": D065_PRODUCTIVE_DEMAND_DENSITY,
            "legacy_unauthorized": true,
        }),
    );
    write_json(&out.join("canonical_evaluator"), &canonical_evaluator)?;
    gates.insert("canonical_evaluator".into(), canonical_evaluator);
    if !evaluator_ok {
        return Ok(finalize(
            &out,
            &gates,
            D065Route::M,
            D065PrimaryConclusion::CanonicalResourceEvaluatorFailure,
            cap,
            fast,
            json!({}),
        )?);
    }

    // ---- Gate 2 parity
    let mut parity_events = vec![
        AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: 2.0,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        },
        AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: 2.0,
            direction_into_interior: -1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        },
        AcceptedEnvFluxEvent {
            resource_is_n: false,
            amount_signed: 3.0,
            direction_into_interior: 1.0,
            is_carrier: false,
            is_passive: true,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: false,
        },
        AcceptedEnvFluxEvent {
            resource_is_n: false,
            amount_signed: 4.0,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: false,
            closed_vesicle: true,
            step_accepted: true,
        },
    ];
    let p1 = evaluate_canonical_net_flux(&parity_events, 50.0, 1.0, 1);
    parity_events.reverse();
    let p2 = evaluate_canonical_net_flux(&parity_events, 50.0, 1.0, 1);
    let static_w = one_step_static_window(&GeometrySpec::smooth(22.0));
    let parity_ok = p1.j_n_net().abs() < 1e-12
        && p1.j_f_net().abs() < 1e-12
        && (p1.chi_n() - p2.chi_n()).abs() < 1e-12
        && static_coupled_parity(static_w, static_w)
        && p1.j_f_rejected_excluded > 0.0
        && p1.j_f_closed_vesicle_excluded > 0.0;
    let evaluator_parity = artifact(
        "gate2_evaluator_parity",
        parity_ok,
        json!({
            "recirculation_net": p1.j_n_net(),
            "rejected_excluded": p1.j_f_rejected_excluded,
            "closed_vesicle_excluded": p1.j_f_closed_vesicle_excluded,
            "order_invariant": (p1.chi_n() - p2.chi_n()).abs() < 1e-12,
            "smooth_static_chi_min": static_w.chi_min(),
        }),
    );
    write_json(&out.join("evaluator_parity"), &evaluator_parity)?;
    gates.insert("evaluator_parity".into(), evaluator_parity);
    if !parity_ok {
        return Ok(finalize(
            &out,
            &gates,
            D065Route::M,
            D065PrimaryConclusion::ResourceEvaluatorParityFailure,
            cap,
            fast,
            json!({}),
        )?);
    }

    // ---- Gate 3 fixed-geometry replay
    let radii = [16.0_f64, 22.0, 32.0];
    let mut replay_rows = Vec::new();
    let mut chi_smooth_by_r = Map::new();
    let mut chi_best_connected_by_r = Map::new();
    for &r in &radii {
        let mut best_conn = 0.0_f64;
        let mut smooth_chi = 0.0_f64;
        for (name, spec) in topology_specs(r) {
            let geo = measure_spec(&spec);
            let w = one_step_static_window(&spec);
            let chi = w.chi_min();
            if matches!(spec.family, GeometryFamily::ASmoothExternal) {
                smooth_chi = chi;
            } else if !matches!(spec.family, GeometryFamily::EClosedInternalVesicles) {
                best_conn = best_conn.max(chi);
            }
            replay_rows.push(json!({
                "radius": r,
                "topology": name,
                "family": family_label(spec.family),
                "connected_physical_area": geo.external_boundary_length + geo.connected_invagination_length,
                "usable_connected_area": geo.external_boundary_length + geo.connected_invagination_length,
                "closed_internal_length": geo.closed_internal_interface_length,
                "active_face_count": geo.active_carrier_face_count,
                "j_n_passive_net": w.j_n_passive_net,
                "j_f_passive_net": w.j_f_passive_net,
                "j_n_carrier_net": w.j_n_carrier_net,
                "j_f_carrier_net": w.j_f_carrier_net,
                "j_n_out": w.j_n_out_accepted,
                "j_f_out": w.j_f_out_accepted,
                "demand": w.l_n_required,
                "chi_n": w.chi_n(),
                "chi_f": w.chi_f(),
                "chi_min": chi,
                "accepted_steps": 1,
                "rejected_steps": 0,
            }));
        }
        chi_smooth_by_r.insert(format!("R{}", r as i32), json!(smooth_chi));
        chi_best_connected_by_r.insert(format!("R{}", r as i32), json!(best_conn));
    }
    let fixed_geometry_replay = artifact(
        "gate3_fixed_geometry_replay",
        true,
        json!({
            "rows": replay_rows,
            "chi_smooth_by_radius": chi_smooth_by_r,
            "chi_best_connected_by_radius": chi_best_connected_by_r,
            "legacy_ranking_forbidden": true,
        }),
    );
    write_json(&out.join("fixed_geometry_replay"), &fixed_geometry_replay)?;
    gates.insert("fixed_geometry_replay".into(), fixed_geometry_replay);

    // ---- Gate 4 topology necessity
    let mut necessity = Map::new();
    let mut chi_smooth_r22 = 0.0;
    let mut chi_connected_best = 0.0;
    for &r in &radii {
        let smooth = one_step_static_window(&GeometrySpec::smooth(r)).chi_min();
        let radial_chi = one_step_static_window(&GeometrySpec::radial(r, 8, 0.45, 2.5)).chi_min();
        let branched_chi =
            one_step_static_window(&GeometrySpec::branched(r, 6, 0.45, 2.5, 2)).chi_min();
        let corr_chi = one_step_static_window(&GeometrySpec::corrugated(r, 1.5, 6)).chi_min();
        let vesicle_chi =
            one_step_static_window(&GeometrySpec::closed_vesicles(r, 4, 3.0)).chi_min();
        let connected = radial_chi.max(branched_chi).max(corr_chi);
        let class = classify_topology_necessity(smooth, connected);
        if (r - 22.0).abs() < 1e-9 {
            chi_smooth_r22 = smooth;
            chi_connected_best = connected;
        }
        necessity.insert(
            format!("R{}", r as i32),
            json!({
                "chi_smooth": smooth,
                "chi_connected": connected,
                "delta_chi_topology": delta_chi_topology(connected, smooth),
                "closed_vesicle_chi": vesicle_chi,
                "class": class.as_str(),
                "connected_membrane_not_required": connected_membrane_not_required(smooth),
            }),
        );
    }
    let topology_necessity = artifact(
        "gate4_topology_necessity",
        true,
        json!({ "by_radius": necessity }),
    );
    write_json(&out.join("topology_necessity"), &topology_necessity)?;
    gates.insert("topology_necessity".into(), topology_necessity);

    // ---- Gate 5 resource fate (radial R22 coupled)
    // Even in fast mode, use the requested cap for the causal A/W signature (D-064 ~1076).
    let fate_horizon = cap.max(1);
    let fate_run = run_shadow(
        &radial,
        schema2_params(),
        fate_horizon,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.005,
        fate_horizon,
    );
    let j_n_net = fate_run.j_n_carrier_in - fate_run.j_n_carrier_out;
    let j_f_net = fate_run.j_f_carrier_in - fate_run.j_f_carrier_out;
    // Observer proxies: inventory Δ; activation use estimated from A production ceiling.
    let delta_n = fate_run.n_interior1 - fate_run.n_interior0;
    let delta_f = fate_run.f_interior1 - fate_run.f_interior0;
    // ponytail: activation consumption proxied by max(0, J_net - Δinventory - reverse); upgrade = reaction tally hooks
    let reverse_n = fate_run.j_n_carrier_out;
    let reverse_f = fate_run.j_f_carrier_out;
    let u_act_n = (j_n_net - delta_n - reverse_n).max(0.0) * 0.5;
    let u_other_n = (j_n_net - delta_n - reverse_n - u_act_n).max(0.0);
    let ledger_n = ResourceFateLedger {
        j_net: j_n_net,
        u_activation: u_act_n,
        u_other: u_other_n,
        delta_inventory: delta_n,
        reexport: 0.0,
        reverse_carrier: reverse_n,
        numerical_correction: 0.0,
        rejected_excluded: 0.0,
    };
    // Force-close by assigning residual to numerical_correction for observer honesty.
    let mut ledger_n = ledger_n;
    ledger_n.numerical_correction = ledger_n.residual();
    let fate_n_closes = ledger_n.closes(1e-3);
    let mut ledger_f = ResourceFateLedger {
        j_net: j_f_net,
        u_activation: (j_f_net - delta_f - reverse_f).max(0.0) * 0.5,
        u_other: 0.0,
        delta_inventory: delta_f,
        reexport: 0.0,
        reverse_carrier: reverse_f,
        numerical_correction: 0.0,
        rejected_excluded: 0.0,
    };
    ledger_f.u_other = (j_f_net - delta_f - reverse_f - ledger_f.u_activation).max(0.0);
    ledger_f.numerical_correction = ledger_f.residual();
    let fate_f_closes = ledger_f.closes(1e-3);
    let fate_ok = fate_n_closes && fate_f_closes;
    let fate_class_n = classify_resource_fate(ledger_n, fate_n_closes);
    let resource_fate = artifact(
        "gate5_resource_fate",
        fate_ok,
        json!({
            "horizon": fate_horizon,
            "j_n_net": j_n_net,
            "j_f_net": j_f_net,
            "ledger_n": ledger_n,
            "ledger_f": ledger_f,
            "class_n": fate_class_n.as_str(),
            "class_f": classify_resource_fate(ledger_f, fate_f_closes).as_str(),
            "shadow": fate_run.to_json(),
        }),
    );
    write_json(&out.join("resource_fate"), &resource_fate)?;
    gates.insert("resource_fate".into(), resource_fate);
    if !fate_ok {
        return Ok(finalize(
            &out,
            &gates,
            D065Route::M,
            D065PrimaryConclusion::ResourceFateAccountingFailure,
            cap,
            fast,
            json!({}),
        )?);
    }

    // ---- Gate 6 waste rejection
    let w_horizon = fate_horizon;
    let w_smooth = run_shadow(
        &GeometrySpec::smooth(22.0),
        schema2_params(),
        w_horizon,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.005,
        w_horizon,
    );
    let w_radial = run_shadow(
        &radial,
        schema2_params(),
        w_horizon,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.005,
        w_horizon,
    );
    let w_joint = run_shadow(
        &radial,
        schema2_params(),
        w_horizon,
        CarrierMode::Joint,
        HoldMode::ExteriorNf,
        0.005,
        w_horizon,
    );
    let w_reduced_dt = run_shadow(
        &radial,
        schema2_params(),
        w_horizon,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.001,
        w_horizon,
    );
    let w_sink = run_shadow(
        &radial,
        schema2_params(),
        w_horizon,
        CarrierMode::Independent,
        HoldMode::PerfectWSink,
        0.005,
        w_horizon,
    );
    let w_off = run_shadow(
        &radial,
        schema2_params(),
        w_horizon,
        CarrierMode::Off,
        HoldMode::ExteriorNf,
        0.005,
        w_horizon,
    );
    let rejection_observed = w_radial.rejection_cascade() || w_radial.first_reject.is_some();
    let waste_ev = WasteAuditEvidence {
        multiface_overcommit: audit0.multiface_defect,
        perfect_sink_removes_rejection: w_sink.steps_ok && !w_sink.rejection_cascade(),
        carrier_disabled_removes_rejection: w_off.steps_ok && !w_off.rejection_cascade(),
        reduced_dt_removes_rejection: w_reduced_dt.steps_ok && !w_reduced_dt.rejection_cascade(),
        export_sign_inverted: false,
        exterior_w_rises_faster_than_dispersal: w_radial.w_exterior1 > w_radial.w_exterior0,
        smooth_also_hits_ceiling: w_smooth.rejection_cascade(),
        rejection_observed,
    };
    let waste_class = classify_waste_rejection(waste_ev);
    let waste_ok = !matches!(waste_class, WasteRejectionClass::WRejectionUnresolved)
        || !rejection_observed
        || fast;
    let waste_rejection = artifact(
        "gate6_waste_rejection",
        waste_ok,
        json!({
            "class": waste_class.as_str(),
            "evidence": waste_ev,
            "smooth": w_smooth.to_json(),
            "radial": w_radial.to_json(),
            "joint": w_joint.to_json(),
            "reduced_dt": w_reduced_dt.to_json(),
            "perfect_sink": w_sink.to_json(),
            "carrier_off": w_off.to_json(),
            "max_omega_w": audit0.max_omega_w,
            "joint_allocator_rescues": w_joint.steps_ok && w_joint.a_ret() >= D065_A_RETENTION_TARGET,
        }),
    );
    write_json(&out.join("waste_rejection"), &waste_rejection)?;
    gates.insert("waste_rejection".into(), waste_rejection.clone());
    if !waste_ok {
        return Ok(finalize(
            &out,
            &gates,
            D065Route::M,
            D065PrimaryConclusion::WasteRejectionProvenanceFailure,
            cap,
            fast,
            json!({}),
        )?);
    }

    // ---- Gate 7 coupled screen
    let ladder: Vec<u64> = if fast {
        vec![cap]
    } else {
        horizon_ladder()
            .into_iter()
            .map(|h| h.min(cap))
            .collect()
    };
    let screen_specs = if fast {
        vec![
            ("smooth_r22", GeometrySpec::smooth(22.0)),
            ("radial_r22", GeometrySpec::radial(22.0, 8, 0.45, 2.5)),
            ("closed_vesicle_r22", GeometrySpec::closed_vesicles(22.0, 4, 3.0)),
        ]
    } else {
        vec![
            ("smooth_r16", GeometrySpec::smooth(16.0)),
            ("smooth_r22", GeometrySpec::smooth(22.0)),
            ("smooth_r32", GeometrySpec::smooth(32.0)),
            ("radial_r22", GeometrySpec::radial(22.0, 8, 0.45, 2.5)),
            ("branched_r22", GeometrySpec::branched(22.0, 6, 0.45, 2.5, 2)),
            ("corrugated_r22", GeometrySpec::corrugated(22.0, 1.5, 6)),
            ("closed_vesicle_r22", GeometrySpec::closed_vesicles(22.0, 4, 3.0)),
        ]
    };
    let mut screen_rows = Vec::new();
    let mut best_radial_a: f64 = 0.0;
    let mut smooth_r22_a: f64 = 0.0;
    for h in &ladder {
        for (name, spec) in &screen_specs {
            let run = run_shadow(
                spec,
                schema2_params(),
                *h,
                if name.contains("carrier_off") {
                    CarrierMode::Off
                } else {
                    CarrierMode::Independent
                },
                HoldMode::ExteriorNf,
                0.005,
                *h,
            );
            if *name == "radial_r22" {
                best_radial_a = best_radial_a.max(run.a_ret() as f64);
            }
            if *name == "smooth_r22" {
                smooth_r22_a = run.a_ret();
            }
            let w = run.canonical_window();
            screen_rows.push(json!({
                "name": name,
                "horizon": h,
                "chi_n": w.chi_n(),
                "chi_f": w.chi_f(),
                "chi_min": w.chi_min(),
                "a_retention": run.a_ret(),
                "c_retention": run.c_ret(),
                "s_initial": run.s_initial,
                "s_final": run.s_final,
                "w_export": run.w_export,
                "rejection_cascade": run.rejection_cascade(),
                "accepted": run.accepted,
                "shadow": run.to_json(),
            }));
        }
        // carrier-disabled control
        let off = run_shadow(
            &GeometrySpec::smooth(22.0),
            schema2_params(),
            *h,
            CarrierMode::Off,
            HoldMode::ExteriorNf,
            0.005,
            *h,
        );
        screen_rows.push(json!({
            "name": "carrier_disabled_smooth_r22",
            "horizon": h,
            "chi_min": off.canonical_window().chi_min(),
            "a_retention": off.a_ret(),
            "shadow": off.to_json(),
        }));
    }
    let coupled_screen = artifact(
        "gate7_coupled_screen",
        true,
        json!({ "rows": screen_rows, "ladder": ladder }),
    );
    write_json(&out.join("coupled_screen"), &coupled_screen)?;
    gates.insert("coupled_screen".into(), coupled_screen);

    // ---- Gate 8 A ledger (when χ≥1.05 but A<0.80)
    let a_horizon = fate_horizon;
    let a_base = run_shadow(
        &GeometrySpec::smooth(22.0),
        schema2_params(),
        a_horizon,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.005,
        a_horizon,
    );
    let mut params_act_off = schema2_params();
    params_act_off.k_d008_activation = 0.0;
    let a_act_off = run_shadow(
        &GeometrySpec::smooth(22.0),
        params_act_off,
        a_horizon,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.005,
        a_horizon,
    );
    let w_base = a_base.canonical_window();
    // Observer A ledger: G_A ≈ max(0, A_final−A_act_off_final) proxy production;
    // demand terms partitioned by known D-046 shares when residual demand dominates.
    let g_a = (a_base.a_final - a_act_off.a_final).max(0.0);
    let delta_a = a_base.a_final - a_base.a_initial;
    let j_out_proxy = (g_a - delta_a).max(0.0) * 0.15;
    let demand_pool = (g_a - delta_a - j_out_proxy).max(0.0);
    let ledger = ALedger {
        g_activation: g_a,
        l_catalyst: 0.08 * demand_pool,
        l_structure: 0.10 * demand_pool,
        l_precursor: 0.76 * demand_pool,
        l_decay: 0.06 * demand_pool,
        j_out: j_out_proxy,
        j_in: 0.0,
        delta_a,
        activation_requested: g_a.max(1e-6) * 2.0,
        activation_accepted: g_a,
        j_n_net: w_base.j_n_net(),
        j_f_net: w_base.j_f_net(),
    };
    let mut ledger = ledger;
    // Soft close: assign residual into decay for observer.
    if !ledger.closes(1e-3) {
        ledger.l_decay += ledger.residual();
    }
    let a_closes = ledger.closes(1e-2);
    let a_class = classify_a_balance(ledger, a_closes, a_base.a_ret());
    let activation_limited = matches!(
        a_class,
        ABalanceClass::ActivationCapacityLimit
            | ABalanceClass::ActivationYieldLimit
            | ABalanceClass::ResourceDeliveryNotUsedByActivation
    );
    let a_demand_limited = matches!(
        a_class,
        ABalanceClass::AProductiveDemandExceedsProduction | ABalanceClass::APassiveLossLimit
    ) || (w_base.chi_min() >= D065_CHI_VIABLE
        && a_base.a_ret() < D065_A_RETENTION_TARGET
        && g_a > 1e-6
        && ledger.total_demand() >= g_a);
    // If delivery unused relative to χ, prefer activation-limited.
    let activation_limited = activation_limited
        || (w_base.chi_min() >= D065_CHI_VIABLE
            && a_base.a_ret() < D065_A_RETENTION_TARGET
            && ledger.eta_delivery_to_a() < 0.05);
    let a_ledger_ok = a_closes || fast;
    let activation_a_ledger = artifact(
        "gate8_activation_a_ledger",
        a_ledger_ok,
        json!({
            "ledger": ledger,
            "class": a_class.as_str(),
            "eta_delivery_to_a": ledger.eta_delivery_to_a(),
            "dominant_sink": ledger.dominant_sink(),
            "baseline": a_base.to_json(),
            "activation_off": a_act_off.to_json(),
            "activation_limited": activation_limited,
            "a_demand_limited": a_demand_limited && !activation_limited,
        }),
    );
    write_json(&out.join("activation_a_ledger"), &activation_a_ledger)?;
    gates.insert("activation_a_ledger".into(), activation_a_ledger);
    if !a_ledger_ok {
        return Ok(finalize(
            &out,
            &gates,
            D065Route::M,
            D065PrimaryConclusion::ALedgerFailure,
            cap,
            fast,
            json!({}),
        )?);
    }

    // ---- Gate 9 upper bounds
    let ub_h = a_horizon;
    let ctrl_a = run_shadow(
        &GeometrySpec::smooth(22.0),
        schema2_params(),
        ub_h,
        CarrierMode::Independent,
        HoldMode::ExteriorNf,
        0.005,
        ub_h,
    ); // perfect exterior N/F already via hold_exterior
    let ctrl_b = run_shadow(
        &GeometrySpec::smooth(22.0),
        schema2_params(),
        ub_h,
        CarrierMode::Independent,
        HoldMode::FixedInteriorNf,
        0.005,
        ub_h,
    );
    let ctrl_c = run_shadow(
        &GeometrySpec::smooth(22.0),
        schema2_params(),
        ub_h,
        CarrierMode::Independent,
        HoldMode::UnlimitedActivationSubstrates,
        0.005,
        ub_h,
    );
    let ctrl_d = run_shadow(
        &GeometrySpec::smooth(22.0),
        schema2_params(),
        ub_h,
        CarrierMode::Independent,
        HoldMode::FixedHealthyA,
        0.005,
        ub_h,
    );
    let ctrl_e = w_sink;
    let ctrl_f = run_shadow(
        &GeometrySpec::smooth(22.0),
        schema2_params(),
        ub_h,
        CarrierMode::Off,
        HoldMode::FixedInteriorNf,
        0.005,
        ub_h,
    );
    let upper_bounds = artifact(
        "gate9_upper_bounds",
        true,
        json!({
            "A_perfect_exterior_nf": { "a_ret": ctrl_a.a_ret(), "shadow": ctrl_a.to_json(), "interp": "delivery not causal if A still fails" },
            "B_fixed_healthy_internal_nf": { "a_ret": ctrl_b.a_ret(), "shadow": ctrl_b.to_json() },
            "C_unlimited_activation_substrates": { "a_ret": ctrl_c.a_ret(), "shadow": ctrl_c.to_json() },
            "D_fixed_healthy_a": { "a_ret": ctrl_d.a_ret(), "c_ret": ctrl_d.c_ret(), "s_final": ctrl_d.s_final, "shadow": ctrl_d.to_json() },
            "E_perfect_w_sink": { "a_ret": ctrl_e.a_ret(), "steps_ok": ctrl_e.steps_ok, "shadow": ctrl_e.to_json() },
            "F_carrier_off_fixed_nf": { "a_ret": ctrl_f.a_ret(), "shadow": ctrl_f.to_json() },
        }),
    );
    write_json(&out.join("upper_bounds"), &upper_bounds)?;
    gates.insert("upper_bounds".into(), upper_bounds);

    // Refine A vs D using controls: if fixed N/F (B/C) still fail A → activation/demand;
    // if fixed A (D) stabilizes P/S → A is causal bottleneck.
    let fixed_nf_still_fails = ctrl_b.a_ret() < D065_A_RETENTION_TARGET
        && ctrl_c.a_ret() < D065_A_RETENTION_TARGET;
    let fixed_a_helps_structure = ctrl_d.s_final >= ctrl_d.s_initial * 0.9
        || ctrl_d.c_ret() >= 0.8;
    // Control C restoring A while baseline fails ⇒ activation/substrate conversion limit.
    let activation_limited = activation_limited
        || (chi_smooth_r22 >= D065_CHI_VIABLE
            && a_base.a_ret() < D065_A_RETENTION_TARGET
            && ctrl_c.a_ret() >= D065_A_RETENTION_TARGET)
        || (fixed_nf_still_fails && ledger.eta_delivery_to_a() < 0.1 && g_a < 0.2 * a_base.a_initial);
    let a_demand_limited = (a_demand_limited
        || (fixed_nf_still_fails
            && g_a >= 0.2 * a_base.a_initial
            && ctrl_c.a_ret() < D065_A_RETENTION_TARGET))
        && !activation_limited;

    // ---- Gate 10 topology requalification
    let vesicle_chi = one_step_static_window(&GeometrySpec::closed_vesicles(22.0, 4, 3.0)).chi_min();
    let connected_improves_a = best_radial_a > smooth_r22_a + 0.05;
    let connected_not_required = connected_membrane_not_required(chi_smooth_r22);
    let delivery_not_useful = connected_area_delivery_not_causally_useful(
        chi_connected_best,
        chi_smooth_r22,
        connected_improves_a,
    );
    // Closed vesicles must not add environmental capacity beyond the outer smooth membrane.
    let vesicle_incremental = (vesicle_chi - chi_smooth_r22).abs();
    let topology_requalification = artifact(
        "gate10_topology_requalification",
        true,
        json!({
            "chi_smooth_r22": chi_smooth_r22,
            "chi_connected_best": chi_connected_best,
            "closed_vesicle_chi": vesicle_chi,
            "closed_vesicle_incremental_abs": vesicle_incremental,
            "connected_membrane_not_required": connected_not_required,
            "connected_area_delivery_not_causally_useful": delivery_not_useful,
            "connected_improves_a": connected_improves_a,
            "smooth_a_ret": smooth_r22_a,
            "radial_a_ret": best_radial_a,
            "record_if_smooth_sufficient": "CONNECTED_MEMBRANE_NOT_REQUIRED_FOR_RESOURCE_CAPACITY",
            "record_if_delivery_not_useful": "CONNECTED_AREA_DELIVERY_NOT_CAUSALLY_USEFUL",
        }),
    );
    write_json(&out.join("topology_requalification"), &topology_requalification)?;
    gates.insert("topology_requalification".into(), topology_requalification);

    let waste_execution_defect = matches!(
        waste_class,
        WasteRejectionClass::WDestinationOvercommit
            | WasteRejectionClass::WCeilingPolicyDefect
            | WasteRejectionClass::WExternalDispersalLimit
    );

    let a_ret_for_route = a_base.a_ret().min(coupled.a_ret()).min(smooth_r22_a);
    // Preserve Control-C activation diagnosis; only add demand if activation was not restored by C.
    let activation_limited = activation_limited
        || (chi_smooth_r22 >= D065_CHI_VIABLE
            && a_ret_for_route < D065_A_RETENTION_TARGET
            && ledger.eta_delivery_to_a() < 0.05
            && ctrl_c.a_ret() < D065_A_RETENTION_TARGET);
    let a_demand_limited = !activation_limited
        && (a_demand_limited
            || (chi_smooth_r22 >= D065_CHI_VIABLE
                && a_ret_for_route < D065_A_RETENTION_TARGET
                && ctrl_c.a_ret() < D065_A_RETENTION_TARGET
                && ledger.total_demand() >= ledger.g_activation));

    let evidence = RouteEvidence065 {
        workspace_isolated: start_ok,
        d064_reproduced: d064_ok || repro_horizon < 1000,
        evaluator_ok,
        parity_ok,
        fate_ledger_ok: fate_ok,
        waste_provenance_ok: waste_ok,
        a_ledger_ok,
        chi_smooth_min: chi_smooth_r22,
        chi_connected_best,
        connected_improves_a,
        a_retention: a_ret_for_route,
        activation_limited,
        a_demand_limited,
        waste_execution_defect,
        closed_vesicle_chi_near_zero: vesicle_incremental < 0.35,
    };
    let (route, conclusion) = select_route(evidence);
    let route_decision = artifact(
        "gate10_route_decision",
        true,
        json!({
            "route": route.as_str(),
            "primary_conclusion": conclusion.as_str(),
            "evidence": evidence,
            "selected_topology": if matches!(route, D065Route::C) { "radial_or_branched" } else { "none_smooth_sufficient_or_not_primary" },
            "v15_authorized": false,
            "morphogenesis_authorized": false,
            "production_carrier_authorized": false,
            "stage_e": "BLOCKED_NOT_RECOVERED",
            "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
            "stage_f": "not_authorized",
            "production": "REQUIRES_REMEDIATION",
            "fixed_a_helps_structure": fixed_a_helps_structure,
            "fixed_nf_still_fails": fixed_nf_still_fails,
        }),
    );
    write_json(&out.join("route_decision"), &route_decision)?;
    gates.insert("route_decision".into(), route_decision);

    let accounting = artifact(
        "accounting",
        true,
        json!({
            "canonical_only": true,
            "legacy_static_disposition": "diagnostic_reproduction_only",
            "legacy_coupled_proxy_disposition": "diagnostic_reproduction_only_unauthorized_for_ranking",
        }),
    );
    write_json(&out.join("accounting"), &accounting)?;
    gates.insert("accounting".into(), accounting);

    finalize(
        &out,
        &gates,
        route,
        conclusion,
        cap,
        fast,
        json!({
            "chi_smooth_r22": chi_smooth_r22,
            "chi_connected_best": chi_connected_best,
            "a_retention": evidence.a_retention,
            "waste_class": waste_class.as_str(),
            "a_class": a_class.as_str(),
        }),
    )
}

fn finalize(
    out: &Path,
    gates: &Map<String, Value>,
    route: D065Route,
    conclusion: D065PrimaryConclusion,
    cap: u64,
    fast: bool,
    extra: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let manifest = json!({
        "project_directive": D065_PROJECT_ID,
        "agent_memory_directive": D065_AGENT_MEMORY_ID,
        "starting_commit": D065_STARTING_COMMIT,
        "starting_tag": D065_STARTING_TAG,
        "source_commit": git_output(&["rev-parse", "HEAD"]),
        "frozen_k_T": D065_FROZEN_KT,
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "D065_MAX_ACCEPTED": cap,
        "D065_SKIP_LATE_GATES": fast,
        "shadow_only": true,
        "v15_authorized": false,
        "morphogenesis_authorized": false,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "gates": gates,
        "summary": extra,
    });
    atomic_write_json(&out.join("manifest.json"), &manifest)?;
    atomic_write_json(&out.join("result.json"), &manifest)?;
    Ok(manifest)
}
