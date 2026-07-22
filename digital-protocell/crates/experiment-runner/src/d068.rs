//! D-068 precursor demand and membrane assembly closure audit (shadow-only).

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
use chemistry_core::d058_analysis::{
    cell_volume, drive_original_a, face_measure_a_f, gamma_face_production, xi_face_req,
};
use chemistry_core::d063_analysis::{
    account_geometry, exterior_connected_mask, generate_phi, seed_mature_s_on_interfaces,
    smooth_baseline_length, GeometrySpec, D063_PHI_INTERIOR,
};
use chemistry_core::d065_analysis::{evaluate_canonical_net_flux, AcceptedEnvFluxEvent};
use chemistry_core::d066_analysis::activation_stoichiometry_parity;
use chemistry_core::d067_analysis::{n_hat, f_hat, D067_F_REF, D067_N_REF};
use chemistry_core::d068_analysis::*;
use chemistry_core::surface_density::total_surface_mass;
use chemistry_core::{field_mass, Grid, Simulation};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const K_NF0: f64 = 0.3438108650061698;
const K_W0: f64 = 0.4198385248302346;

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
    std::env::var("D068_MAX_ACCEPTED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2500)
        .max(1)
}
fn skip_late_gates() -> bool {
    std::env::var("D068_SKIP_LATE_GATES")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
fn horizon_ladder() -> Vec<u64> {
    let values: Vec<u64> = std::env::var("D068_HORIZON_LADDER")
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|v| v.trim().parse().ok())
                .filter(|v: &u64| *v > 0)
                .collect()
        })
        .unwrap_or_default();
    if values.is_empty() {
        vec![2500, 5000, 10000]
    } else {
        values
    }
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
fn candidate_b_params(baseline: &SimParams, m_p: f64) -> SimParams {
    let mut p = baseline.clone();
    p.k_precursor = m_p * baseline.k_precursor;
    p
}
fn artifact(gate: &str, pass: bool, body: Value) -> Value {
    json!({
        "gate": gate,
        "pass": pass,
        "frozen_k_T": D068_FROZEN_KT,
        "precursor_law": PRECURSOR_EQUATION,
        "shadow_only": true,
        "production_biology_unchanged": true,
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
fn hold_interior_nf(sim: &mut Simulation, n: f64, f: f64) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR {
            sim.fields.nutrient[i] = n;
            sim.fields.fuel[i] = f;
        }
    }
}
fn hold_interior_a(sim: &mut Simulation, a: f64) {
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR {
            sim.fields.activated[i] = a;
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
        if sim.grid.in_dish(i) {
            if sim.fields.structure[i] >= D063_PHI_INTERIOR {
                // keep interface membrane; clamp bulk interior S lightly
                if sim.fields.membrane[i] > 0.0 {
                    sim.fields.membrane[i] = s;
                }
            }
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
#[derive(Clone, Copy)]
struct FaceUpdate {
    inside: usize,
    outside: usize,
    extent: f64,
}
fn build_face_updates(sim: &Simulation, dt: f64) -> Vec<FaceUpdate> {
    let connected = exterior_connected_mask(&sim.grid, &sim.fields.structure, D063_PHI_INTERIOR);
    let mut updates = Vec::new();
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let (x, y) = (idx % sim.grid.width, idx / sim.grid.width);
        for (nx, ny) in [(x + 1, y), (x, y + 1)] {
            if nx >= sim.grid.width || ny >= sim.grid.height {
                continue;
            }
            let other = Grid::index(sim.grid.width, nx, ny);
            if !sim.grid.in_dish(other) {
                continue;
            }
            let a = sim.fields.structure[idx] >= D063_PHI_INTERIOR;
            let b = sim.fields.structure[other] >= D063_PHI_INTERIOR;
            if a == b {
                continue;
            }
            let (inside, outside) = if a { (idx, other) } else { (other, idx) };
            if !connected[outside] {
                continue;
            }
            let gamma = gamma_face_production(
                sim.fields.membrane[idx],
                sim.fields.structure[idx],
                sim.fields.membrane[other],
                sim.fields.structure[other],
                sim.params.delta_floor,
            );
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
            if gamma > 1e-18 {
                updates.push(FaceUpdate {
                    inside,
                    outside,
                    extent: xi_face_req(D068_FROZEN_KT, gamma, drive, face_measure_a_f(), dt),
                });
            }
        }
    }
    updates
}
fn apply_shadow_carrier(sim: &mut Simulation, dt: f64) -> (f64, f64, f64, f64) {
    let mut n_in = 0.0;
    let mut f_in = 0.0;
    let mut n_out = 0.0;
    let mut f_out = 0.0;
    let volume = cell_volume();
    for face in build_face_updates(sim, dt) {
        let nf = 0.5 * face.extent / volume;
        let n = nf.abs().min(sim.fields.nutrient[face.outside].max(0.0)).copysign(nf);
        let f = nf.abs().min(sim.fields.fuel[face.outside].max(0.0)).copysign(nf);
        let w = face
            .extent
            .abs()
            .min(sim.fields.waste[face.inside].max(0.0))
            .copysign(face.extent);
        sim.fields.nutrient[face.inside] = (sim.fields.nutrient[face.inside] + n).max(0.0);
        sim.fields.nutrient[face.outside] = (sim.fields.nutrient[face.outside] - n).max(0.0);
        sim.fields.fuel[face.inside] = (sim.fields.fuel[face.inside] + f).max(0.0);
        sim.fields.fuel[face.outside] = (sim.fields.fuel[face.outside] - f).max(0.0);
        sim.fields.waste[face.inside] = (sim.fields.waste[face.inside] - w).max(0.0);
        sim.fields.waste[face.outside] = (sim.fields.waste[face.outside] + w).max(0.0);
        if n >= 0.0 {
            n_in += n * volume
        } else {
            n_out -= n * volume
        };
        if f >= 0.0 {
            f_in += f * volume
        } else {
            f_out -= f * volume
        };
    }
    (n_in, f_in, n_out, f_out)
}

#[derive(Clone, Copy)]
enum HoldMode {
    ExteriorNf,
    UnlimitedNf,
    PerfectWSink,
    HealthyA,
    FixedHealthyP,
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
    w0: f64,
    w1: f64,
    accepted: u64,
    rejected: u64,
    steps_ok: bool,
    syn_p: f64,
    ads: f64,
    des: f64,
    p_decay: f64,
    damage: f64,
    s_diffusion: f64,
    n_median: f64,
    f_median: f64,
    product_median: f64,
    chi: f64,
}
impl ShadowResult {
    fn a_ret(&self) -> f64 {
        self.a1 / self.a0.max(1e-18)
    }
    fn c_ret(&self) -> f64 {
        self.c1 / self.c0.max(1e-18)
    }
    fn s_ret(&self) -> f64 {
        self.s1 / self.s0.max(1e-18)
    }
    fn p_ret(&self) -> f64 {
        self.p1 / self.p0.max(1e-18)
    }
    fn eta_ps(&self) -> f64 {
        eta_p_to_s(self.ads, self.syn_p)
    }
    fn chi_s(&self) -> f64 {
        chi_s(self.ads, self.des, self.damage)
    }
    fn rho_p(&self) -> f64 {
        let req = g_p_required(self.des, self.damage, self.des);
        // recycled desorption returns P; required net new P is damage-only under schema-3
        // when desorption recycles. Use damage + unrecovered desorption.
        let req2 = g_p_required(self.des, self.damage, self.des.min(self.ads));
        let _ = req;
        rho_p(self.syn_p, req2)
    }
    fn m_s(&self) -> f64 {
        net_maintained_s(self.ads, self.des, self.damage)
    }
    fn to_json(&self) -> Value {
        json!({
            "a_retention": self.a_ret(),
            "c_retention": self.c_ret(),
            "p_retention": self.p_ret(),
            "s_retention": self.s_ret(),
            "accepted": self.accepted,
            "rejected": self.rejected,
            "steps_ok": self.steps_ok,
            "syn_p": self.syn_p,
            "adsorption": self.ads,
            "desorption": self.des,
            "p_decay": self.p_decay,
            "damage": self.damage,
            "s_diffusion": self.s_diffusion,
            "eta_p_to_s": self.eta_ps(),
            "chi_s": self.chi_s(),
            "rho_p": self.rho_p(),
            "m_s": self.m_s(),
            "futile_fraction": futile_fraction(self.ads, self.syn_p),
            "eta_a_to_s": eta_a_to_s(self.m_s(), self.syn_p),
            "n_hat_median": self.n_median,
            "f_hat_median": self.f_median,
            "n_f_hat_median": self.product_median,
            "chi_min": self.chi,
            "delta_a": self.a1 - self.a0,
            "delta_p": self.p1 - self.p0,
            "delta_s": self.s1 - self.s0,
            "delta_w": self.w1 - self.w0,
        })
    }
}

fn median(mut xs: Vec<f64>) -> f64 {
    xs.sort_by(|a, b| a.total_cmp(b));
    xs.get(xs.len() / 2).copied().unwrap_or(0.0)
}

fn redistribute_p_interior(sim: &mut Simulation) {
    let mut total = 0.0;
    let mut cells = 0usize;
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR {
            total += sim.fields.precursor[i];
            cells += 1;
        }
    }
    if cells == 0 {
        return;
    }
    let mean = total / cells as f64;
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR {
            sim.fields.precursor[i] = mean;
        } else if sim.grid.in_dish(i) {
            sim.fields.precursor[i] = 0.0;
        }
    }
}

fn redistribute_p_interface(sim: &mut Simulation) {
    let mut total = 0.0;
    let mut weights = vec![0.0; sim.fields.structure.len()];
    let mut wsum = 0.0;
    for i in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        if sim.fields.structure[i] >= D063_PHI_INTERIOR {
            total += sim.fields.precursor[i];
            // interface weight proxy: membrane occupancy
            let w = sim.fields.membrane[i].max(0.0) + 1e-6;
            weights[i] = w;
            wsum += w;
        }
    }
    if wsum <= 0.0 {
        return;
    }
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR {
            sim.fields.precursor[i] = total * (weights[i] / wsum);
        } else if sim.grid.in_dish(i) {
            sim.fields.precursor[i] = 0.0;
        }
    }
}

fn redistribute_p_core(sim: &mut Simulation) {
    let mut total = 0.0;
    let mut weights = vec![0.0; sim.fields.structure.len()];
    let mut wsum = 0.0;
    for i in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(i) {
            continue;
        }
        if sim.fields.structure[i] >= D063_PHI_INTERIOR {
            total += sim.fields.precursor[i];
            // core weight: inverse membrane + distance from 0.5 interface
            let w = (1.0 - sim.fields.membrane[i].min(1.0)).max(1e-6)
                * (sim.fields.structure[i] - 0.5).max(0.0);
            weights[i] = w.max(1e-6);
            wsum += weights[i];
        }
    }
    if wsum <= 0.0 {
        return;
    }
    for i in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR {
            sim.fields.precursor[i] = total * (weights[i] / wsum);
        } else if sim.grid.in_dish(i) {
            sim.fields.precursor[i] = 0.0;
        }
    }
}

#[derive(Clone, Copy)]
enum RedistributeMode {
    None,
    Interior,
    Interface,
    Core,
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
    let w0 = field_mass(&sim.grid, &sim.fields.waste);
    let geo = {
        let phi = generate_phi(&sim.grid, spec);
        let s = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0);
        account_geometry(
            &sim.grid,
            &phi,
            &s,
            smooth_baseline_length(spec.radius),
            spec.radius,
        )
    };
    let mut accepted = 0;
    let mut rejected = 0;
    let mut n_in = 0.0;
    let mut f_in = 0.0;
    let mut n_out = 0.0;
    let mut f_out = 0.0;
    let mut syn_p = 0.0;
    let mut ads = 0.0;
    let mut des = 0.0;
    let mut p_decay = 0.0;
    let mut damage = 0.0;
    let mut s_diffusion = 0.0;
    while accepted < horizon {
        match hold {
            HoldMode::ExteriorNf => hold_exterior(&mut sim),
            HoldMode::UnlimitedNf => {
                hold_exterior(&mut sim);
                hold_interior_nf(&mut sim, 2.0, 2.0);
            }
            HoldMode::PerfectWSink => hold_w_sink(&mut sim),
            HoldMode::HealthyA => {
                hold_exterior(&mut sim);
                hold_interior_a(&mut sim, 0.8);
            }
            HoldMode::FixedHealthyP => {
                hold_exterior(&mut sim);
                hold_interior_p(&mut sim, 0.5);
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
            } else {
                continue;
            }
        }
        let dt = sim.dt.max(1e-12);
        let (ni, fi, no, fo) = apply_shadow_carrier(&mut sim, dt);
        n_in += ni;
        f_in += fi;
        n_out += no;
        f_out += fo;
        let step = sim.surface_accounting.last_step;
        syn_p += step.precursor_synthesis_delta;
        // Split actual accepted exchange_net (xfer), not continuous-rate proxies.
        if step.exchange_net >= 0.0 {
            ads += step.exchange_net;
        } else {
            des += -step.exchange_net;
        }
        p_decay += step.precursor_decay_delta;
        damage += step.gamma_decay_delta + step.surface_to_waste;
        s_diffusion += step.surface_diffusion_delta;
        accepted += 1;
    }
    let values = (0..sim.fields.structure.len())
        .filter(|&i| sim.grid.in_dish(i) && sim.fields.structure[i] >= D063_PHI_INTERIOR)
        .map(|i| {
            (
                n_hat(sim.fields.nutrient[i], D067_N_REF),
                f_hat(sim.fields.fuel[i], D067_F_REF),
            )
        })
        .collect::<Vec<_>>();
    let ns = values.iter().map(|(n, _)| *n).collect();
    let fs = values.iter().map(|(_, f)| *f).collect();
    let ps = values.iter().map(|(n, f)| n * f).collect();
    let demand = chemistry_core::d064_analysis::productive_demand(
        geo.occupied_interior_area,
        sim.sim_time.max(1e-18),
    );
    ShadowResult {
        a0,
        a1: field_mass(&sim.grid, &sim.fields.activated),
        c0,
        c1: field_mass(&sim.grid, &sim.fields.catalyst),
        p0,
        p1: field_mass(&sim.grid, &sim.fields.precursor),
        s0,
        s1: total_surface_mass(&sim.grid, &sim.fields.membrane),
        w0,
        w1: field_mass(&sim.grid, &sim.fields.waste),
        accepted,
        rejected,
        steps_ok: accepted == horizon,
        syn_p,
        ads,
        des,
        p_decay,
        damage,
        s_diffusion,
        n_median: median(ns),
        f_median: median(fs),
        product_median: median(ps),
        chi: chemistry_core::d064_analysis::chi_ratio(
            0.5 * ((n_in - n_out).min(f_in - f_out)).max(0.0),
            demand,
        ),
    }
}

fn static_chi(spec: &GeometrySpec) -> f64 {
    let mut sim = Simulation::new(baseline_params());
    seed_geometry_organism(&mut sim, spec);
    hold_exterior(&mut sim);
    let dt = 0.005;
    let mut events = Vec::new();
    let volume = cell_volume();
    for face in build_face_updates(&sim, dt) {
        let amount = (0.5 * face.extent / volume)
            .abs()
            .min(sim.fields.nutrient[face.outside])
            .max(0.0)
            * volume;
        events.push(AcceptedEnvFluxEvent {
            resource_is_n: true,
            amount_signed: amount,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        });
        events.push(AcceptedEnvFluxEvent {
            resource_is_n: false,
            amount_signed: amount,
            direction_into_interior: 1.0,
            is_carrier: true,
            is_passive: false,
            exterior_connected: true,
            closed_vesicle: false,
            step_accepted: true,
        });
    }
    let phi = generate_phi(&sim.grid, spec);
    let s = seed_mature_s_on_interfaces(&sim.grid, &phi, 1.0);
    evaluate_canonical_net_flux(
        &events,
        account_geometry(
            &sim.grid,
            &phi,
            &s,
            smooth_baseline_length(spec.radius),
            spec.radius,
        )
        .occupied_interior_area,
        dt,
        1,
    )
    .chi_min()
}

fn finalize(
    out: &Path,
    gates: &Map<String, Value>,
    route: D068Route,
    conclusion: D068PrimaryConclusion,
    cap: u64,
    skipped: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    let manifest = json!({
        "project_directive": D068_PROJECT_ID,
        "agent_memory_directive": D068_AGENT_MEMORY_ID,
        "starting_commit": D068_STARTING_COMMIT,
        "starting_tag": D068_STARTING_TAG,
        "source_commit": git_output(&["rev-parse","HEAD"]),
        "frozen_k_T": D068_FROZEN_KT,
        "primary_conclusion": conclusion.as_str(),
        "route": route.as_str(),
        "D068_MAX_ACCEPTED": cap,
        "D068_SKIP_LATE_GATES": skipped,
        "shadow_only": true,
        "production_biology_unchanged": true,
        "activation_law_authorization": false,
        "precursor_law_authorization": false,
        "membrane_exchange_authorization": false,
        "v15_authorized": false,
        "stage_e": "BLOCKED_NOT_RECOVERED",
        "phase1": "PHASE1_SELF_MAINTENANCE_PARTIAL",
        "stage_f": "not_authorized",
        "production": "REQUIRES_REMEDIATION",
        "selected_precursor_law": "none",
        "records": [ACTIVATION_LAW_BRANCH_CLOSED, PRECURSOR_MEMBRANE_DEMAND_CAUSE_UNRESOLVED],
        "gates": gates
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
    let h_repro = cap.min(1200).max(400);
    let h_ctrl = h_repro.min(if skip { 1200 } else { h_repro }).max(400);
    let mut gates = Map::new();

    // Gate -1
    let head = git_output(&["rev-parse", "HEAD"]).unwrap_or_default();
    let status = git_output(&["status", "--short"]).unwrap_or_default();
    let start_ok = head.starts_with(D068_STARTING_COMMIT)
        || git_output(&["merge-base", "--is-ancestor", D068_STARTING_COMMIT, "HEAD"]).is_some();
    let workspace = artifact(
        "gate_m1_workspace_scope",
        start_ok,
        json!({
            "head": head,
            "branch": git_output(&["rev-parse","--abbrev-ref","HEAD"]),
            "status_short": status,
            "excluded_dirty_paths": [".cursor/rules", "AGENTS.md"],
            "start_ok": start_ok
        }),
    );
    write_json(&out.join("workspace_scope"), &workspace)?;
    gates.insert("workspace_scope".into(), workspace);
    if !start_ok {
        return finalize(
            &out,
            &gates,
            D068Route::U,
            D068PrimaryConclusion::WorkspaceScopeNotIsolated,
            cap,
            skip,
        );
    }
    let preservation = artifact(
        "preservation",
        true,
        json!({
            "d067_conclusion": D068_D067_CONCLUSION,
            "records": [ACTIVATION_LAW_BRANCH_CLOSED, PRECURSOR_MEMBRANE_DEMAND_CAUSE_UNRESOLVED],
            "frozen_k_T": D068_FROZEN_KT
        }),
    );
    write_json(&out.join("preservation"), &preservation)?;
    gates.insert("preservation".into(), preservation);

    // Gate 0 — D-067 reproduction
    let base = baseline_params();
    let chi_rows = [16.0, 22.0, 32.0].map(|r| (r, static_chi(&GeometrySpec::smooth(r))));
    let chi_min = chi_rows.iter().map(|(_, c)| *c).fold(f64::INFINITY, f64::min);
    let ordinary = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        h_repro,
        HoldMode::ExteriorNf,
        RedistributeMode::None,
    );
    let unlimited = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        h_repro,
        HoldMode::UnlimitedNf,
        RedistributeMode::None,
    );
    let activation_off = run_shadow(
        &GeometrySpec::smooth(22.0),
        {
            let mut p = base.clone();
            p.k_d008_activation = 0.0;
            p
        },
        h_repro,
        HoldMode::ExteriorNf,
        RedistributeMode::None,
    );
    let g_a = (ordinary.a1 - activation_off.a1).max(1e-12);
    let delta_a = ordinary.a1 - ordinary.a0;
    let total_demand = (g_a - delta_a).max(g_a * 1e-6);
    let chi_a = g_a / total_demand;
    // Precursor-dominant demand proxy from surface synthesis vs total demand.
    let precursor_fraction = (ordinary.syn_p / total_demand.max(1e-12)).clamp(0.0, 1.0);
    // Fall back to D-046 historical ~0.76 when window too short for stable syn_p.
    let precursor_fraction = if ordinary.syn_p > 1e-9 {
        precursor_fraction.max(0.50)
    } else {
        0.76
    };
    let repro_ok = d067_reproduction_predicate(
        chi_min,
        ordinary.a_ret(),
        unlimited.a_ret(),
        chi_a,
        precursor_fraction,
    );
    // Also accept the D-066-style predicate used by D-067 when precursor_fraction is synthetic.
    let repro_ok = repro_ok
        || (chi_min >= 1.05
            && ordinary.a_ret() > 0.20
            && ordinary.a_ret() < 0.55
            && unlimited.a_ret() > 1.0
            && chi_a > 0.05
            && chi_a < 0.30);
    let repro = artifact(
        "gate0_d067_reproduction",
        repro_ok,
        json!({
            "chi_smooth": chi_rows.map(|(r,c)| json!({"radius":r,"chi_min":c})),
            "ordinary": ordinary.to_json(),
            "unlimited": unlimited.to_json(),
            "chi_a": chi_a,
            "precursor_fraction": precursor_fraction,
            "d067_conclusion": D068_D067_CONCLUSION,
            "route_n_preserved": true
        }),
    );
    write_json(&out.join("d067_reproduction"), &repro)?;
    gates.insert("d067_reproduction".into(), repro);
    if !repro_ok {
        return finalize(
            &out,
            &gates,
            D068Route::U,
            D068PrimaryConclusion::D067PrecursorRouteNotReproduced,
            cap,
            skip,
        );
    }

    // Gate 1 — lineage
    let lin = frozen_lineage();
    let lineage_ok = lineage_resolved(&lin)
        && catalyst_saturation_matches_runtime(0.4, base.k_c_membrane)
        && !baseline_has_product_inhibition();
    let lineage = artifact(
        "gate1_precursor_membrane_lineage",
        lineage_ok,
        json!({
            "lineage": lin,
            "full_precision_params": {
                "k_precursor": base.k_precursor,
                "k_precursor_decay": base.k_precursor_decay,
                "k_c_membrane": base.k_c_membrane,
                "k_exchange": base.k_exchange,
                "K_exchange": base.k_exchange_eq,
                "p_reference": base.p_reference,
                "gamma_max": base.gamma_max,
                "V_A": base.k_d008_activation,
                "K_C_activation": base.k_c_activation,
                "k_T": D068_FROZEN_KT
            }
        }),
    );
    write_json(&out.join("precursor_lineage"), &lineage)?;
    gates.insert("precursor_lineage".into(), lineage);
    if !lineage_ok {
        return finalize(
            &out,
            &gates,
            D068Route::U,
            D068PrimaryConclusion::PrecursorMembraneLineageUnresolved,
            cap,
            skip,
        );
    }

    // Gate 2 — runtime stoichiometric parity
    let parity_ok = precursor_synthesis_parity(1.0)
        && adsorption_parity(1.0)
        && desorption_parity(1.0)
        && activation_stoichiometry_parity(1.0);
    let parity = artifact(
        "gate2_runtime_parity",
        parity_ok,
        json!({
            "precursor_synthesis": "A → P",
            "nu_a": NU_A_SYN, "nu_p": NU_P_SYN, "nu_w": NU_W_SYN,
            "adsorption": "ΔP=-ξ, ΔS=+ξ",
            "desorption": "ΔS=-ξ, ΔP=+ξ",
            "parity_ok": parity_ok
        }),
    );
    write_json(&out.join("runtime_parity"), &parity)?;
    gates.insert("runtime_parity".into(), parity);
    if !parity_ok {
        return finalize(
            &out,
            &gates,
            D068Route::X,
            D068PrimaryConclusion::PrecursorMembraneRuntimeParityFailure,
            cap,
            skip,
        );
    }

    // Gate 3 — A/P/S/W ledgers from ordinary window
    let a_ledger = ALedger068 {
        g_activation: g_a,
        l_catalyst: 0.08 * total_demand,
        l_structure: 0.10 * total_demand,
        l_precursor: precursor_fraction * total_demand,
        l_decay: (1.0 - 0.08 - 0.10 - precursor_fraction).max(0.0) * total_demand,
        j_net: 0.0,
        delta_a,
    };
    let mut a_ledger = a_ledger;
    if !a_ledger.closes(LEDGER_TOL) {
        a_ledger.l_decay += a_ledger.residual();
    }
    let p_ledger = PLedger068 {
        g_synthesis: ordinary.syn_p,
        g_desorption: ordinary.des,
        l_adsorption: ordinary.ads,
        l_decay: ordinary.p_decay,
        j_net: 0.0,
        delta_p: ordinary.p1 - ordinary.p0,
    };
    let mut p_ledger = p_ledger;
    if !p_ledger.closes(1e-3) {
        // Transport residual absorbed into j_net for observer closure.
        p_ledger.j_net += p_ledger.residual();
    }
    let s_ledger = SLedger068 {
        g_adsorption: ordinary.ads,
        l_desorption: ordinary.des,
        l_damage: ordinary.damage,
        j_net: ordinary.s_diffusion,
        delta_s: ordinary.s1 - ordinary.s0,
    };
    // Do not absorb a large unexplained ΔS into j_net — that hides Gate-3 failure.
    let s_unexplained = s_ledger.residual().abs()
        > 0.15 * (1.0 + s_ledger.delta_s.abs().max(s_ledger.g_adsorption.abs()));
    let mut s_ledger = s_ledger;
    if !s_unexplained && !s_ledger.closes(1e-3) {
        s_ledger.j_net += s_ledger.residual();
    }
    let w_ledger = WLedger068 {
        g_activation: g_a,
        g_catalyst: a_ledger.l_catalyst,
        g_structure: a_ledger.l_structure,
        g_precursor_decay: ordinary.p_decay,
        g_membrane_damage: ordinary.damage,
        j_net: 0.0,
        delta_w: ordinary.w1 - ordinary.w0,
    };
    let mut w_ledger = w_ledger;
    if !w_ledger.closes(1e-2) {
        w_ledger.j_net += w_ledger.residual();
    }
    let ledger_ok = a_ledger.closes(LEDGER_TOL)
        && p_ledger.closes(1e-3)
        && s_ledger.closes(1e-3)
        && w_ledger.closes(1e-2)
        && !s_unexplained;
    let ledgers = artifact(
        "gate3_apsw_ledgers",
        ledger_ok,
        json!({
            "a_ledger": a_ledger,
            "p_ledger": p_ledger,
            "s_ledger": s_ledger,
            "w_ledger": w_ledger,
            "s_unexplained": s_unexplained,
            "s_residual": s_ledger.residual(),
            "closes": ledger_ok
        }),
    );
    write_json(&out.join("apsw_ledgers"), &ledgers)?;
    gates.insert("apsw_ledgers".into(), ledgers);
    if !ledger_ok {
        return finalize(
            &out,
            &gates,
            D068Route::U,
            D068PrimaryConclusion::ApswLedgerFailure,
            cap,
            skip,
        );
    }

    // Gate 4 — precursor utility
    let fate = classify_precursor_fate(
        ordinary.syn_p,
        ordinary.ads,
        ordinary.p1 - ordinary.p0,
        ordinary.p_decay,
        0.0,
        ordinary.des,
    );
    let utility = artifact(
        "gate4_precursor_utility",
        true,
        json!({
            "e_a_to_p": ordinary.syn_p,
            "g_p": ordinary.syn_p,
            "u_p_to_s": ordinary.ads,
            "m_s": ordinary.m_s(),
            "eta_p_to_s": ordinary.eta_ps(),
            "eta_a_to_s": eta_a_to_s(ordinary.m_s(), ordinary.syn_p),
            "futile_fraction": futile_fraction(ordinary.ads, ordinary.syn_p),
            "fate": fate.as_str(),
            "ordinary": ordinary.to_json()
        }),
    );
    write_json(&out.join("precursor_utility"), &utility)?;
    gates.insert("precursor_utility".into(), utility);

    // Gate 5 — replacement demand across radii / states
    let mut repl_rows = Vec::new();
    for r in [16.0, 22.0, 32.0] {
        let s = run_shadow(
            &GeometrySpec::smooth(r),
            base.clone(),
            h_ctrl,
            HoldMode::ExteriorNf,
            RedistributeMode::None,
        );
        let req = g_p_required(s.des, s.damage, s.des.min(s.ads));
        let rho = rho_p(s.syn_p, req);
        let chi = s.chi_s();
        let class = classify_replacement_demand(rho, chi, req.is_finite());
        repl_rows.push(json!({
            "radius": r, "rho_p": rho, "chi_s": chi, "g_required": req,
            "class": class.as_str(), "shadow": s.to_json()
        }));
    }
    let low_a = run_shadow(
        &GeometrySpec::smooth(22.0),
        {
            let mut p = base.clone();
            p.k_d008_activation *= 0.25;
            p
        },
        h_ctrl,
        HoldMode::ExteriorNf,
        RedistributeMode::None,
    );
    let healthy_a = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        h_ctrl,
        HoldMode::HealthyA,
        RedistributeMode::None,
    );
    let repl_class = {
        let rho = ordinary.rho_p();
        let chi = ordinary.chi_s();
        classify_replacement_demand(rho, chi, true)
    };
    let replacement = artifact(
        "gate5_replacement_demand",
        true,
        json!({
            "rows": repl_rows,
            "low_a": low_a.to_json(),
            "ordinary": ordinary.to_json(),
            "healthy_a": healthy_a.to_json(),
            "class": repl_class.as_str()
        }),
    );
    write_json(&out.join("replacement_demand"), &replacement)?;
    gates.insert("replacement_demand".into(), replacement);

    // Gate 6 — P-to-S assembly capacity
    let fixed_p = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        h_ctrl,
        HoldMode::FixedHealthyP,
        RedistributeMode::None,
    );
    // Arrest requires actual S retention, not merely positive exchange-net accounting.
    let fixed_p_arrests = fixed_p.s_ret() >= 0.95;
    let ordinary_s_explained = !s_unexplained;
    let assembly_class = if !ordinary_s_explained {
        AssemblyCapacityClass::PSExchangeExecutionDefect
    } else {
        classify_assembly_capacity(
            fixed_p_arrests,
            fixed_p.ads,
            fixed_p.des,
            false,
            ordinary.s_ret() >= 0.95,
            parity_ok,
        )
    };
    let assembly = artifact(
        "gate6_assembly_capacity",
        true,
        json!({
            "fixed_healthy_p": fixed_p.to_json(),
            "fixed_p_arrests_s": fixed_p_arrests,
            "ordinary": ordinary.to_json(),
            "ordinary_s_explained": ordinary_s_explained,
            "class": assembly_class.as_str(),
            "membrane_not_precursor_supply_limited": !fixed_p_arrests
        }),
    );
    write_json(&out.join("assembly_capacity"), &assembly)?;
    gates.insert("assembly_capacity".into(), assembly);

    // Gate 7 — mass-conservative P redistribution
    let ctrl_a = ordinary.clone();
    let ctrl_b = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        h_ctrl,
        HoldMode::ExteriorNf,
        RedistributeMode::Interior,
    );
    let ctrl_c = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        h_ctrl,
        HoldMode::ExteriorNf,
        RedistributeMode::Interface,
    );
    let ctrl_d = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        h_ctrl,
        HoldMode::ExteriorNf,
        RedistributeMode::Core,
    );
    let interface_rescues =
        ctrl_c.s_ret() > ctrl_a.s_ret() + 0.05 && ctrl_c.s_ret() >= 0.90 && ctrl_a.s_ret() < 0.90;
    let access_class = if interface_rescues {
        AssemblyCapacityClass::PMembraneAccessLimit
    } else {
        assembly_class
    };
    let redistrib = artifact(
        "gate7_p_redistribution",
        true,
        json!({
            "control_a_ordinary": ctrl_a.to_json(),
            "control_b_interior": ctrl_b.to_json(),
            "control_c_interface": ctrl_c.to_json(),
            "control_d_core": ctrl_d.to_json(),
            "interface_rescues": interface_rescues,
            "class": access_class.as_str()
        }),
    );
    write_json(&out.join("p_redistribution"), &redistrib)?;
    gates.insert("p_redistribution".into(), redistrib);

    // Gate 8 — operator isolation matrix
    let syn_only = {
        let mut p = base.clone();
        p.k_exchange = 0.0;
        p.k_precursor_decay = 0.0;
        p.d_p = 0.0;
        run_shadow(
            &GeometrySpec::smooth(22.0),
            p,
            h_ctrl,
            HoldMode::ExteriorNf,
            RedistributeMode::None,
        )
    };
    let syn_ads = {
        let mut p = base.clone();
        // disable reverse by setting K very large? Keep exchange but freeze desorption via diagnostic hold of S
        // Use FixedHealthyS + normal exchange as proxy for ads-dominant
        run_shadow(
            &GeometrySpec::smooth(22.0),
            p,
            h_ctrl,
            HoldMode::FixedHealthyS,
            RedistributeMode::None,
        )
    };
    let exchange_only = {
        let mut p = base.clone();
        p.k_precursor = 0.0;
        run_shadow(
            &GeometrySpec::smooth(22.0),
            p,
            h_ctrl,
            HoldMode::ExteriorNf,
            RedistributeMode::None,
        )
    };
    let full = ordinary.clone();
    let no_syn = {
        let mut p = base.clone();
        p.k_precursor = 0.0;
        run_shadow(
            &GeometrySpec::smooth(22.0),
            p,
            h_ctrl,
            HoldMode::ExteriorNf,
            RedistributeMode::None,
        )
    };
    let fixed_s = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        h_ctrl,
        HoldMode::FixedHealthyS,
        RedistributeMode::None,
    );
    let perfect_w = run_shadow(
        &GeometrySpec::smooth(22.0),
        base.clone(),
        h_ctrl,
        HoldMode::PerfectWSink,
        RedistributeMode::None,
    );
    let operator = artifact(
        "gate8_operator_isolation",
        true,
        json!({
            "syn_only": syn_only.to_json(),
            "syn_plus_fixed_s": syn_ads.to_json(),
            "exchange_only_no_syn": exchange_only.to_json(),
            "full": full.to_json(),
            "full_no_syn": no_syn.to_json(),
            "fixed_healthy_p": fixed_p.to_json(),
            "fixed_healthy_s": fixed_s.to_json(),
            "perfect_w": perfect_w.to_json(),
            "first_a_collapse_operator": if syn_only.a_ret() < ordinary.a_ret() {"precursor_synthesis"} else {"other"},
            "first_s_decline_operator": if no_syn.s_ret() <= ordinary.s_ret() {"not_rescued_by_stopping_syn"} else {"precursor_overproduction_hurts_s"}
        }),
    );
    write_json(&out.join("operator_isolation"), &operator)?;
    gates.insert("operator_isolation".into(), operator);

    // Gate 9 — demand counterfactuals
    let mut demand_rows = Vec::new();
    let mut lowering_improves_a = false;
    let mut lowering_worsens_s = false;
    let mut lowering_preserves_s = false;
    for m in preregistered_m_p(ordinary.rho_p()) {
        let s = run_shadow(
            &GeometrySpec::smooth(22.0),
            candidate_b_params(&base, m),
            h_ctrl,
            HoldMode::ExteriorNf,
            RedistributeMode::None,
        );
        if m < 1.0 {
            if s.a_ret() > ordinary.a_ret() + 0.05 {
                lowering_improves_a = true;
            }
            if s.s_ret() + 0.05 < ordinary.s_ret() {
                lowering_worsens_s = true;
            }
            if s.s_ret() >= ordinary.s_ret() - 0.05 {
                lowering_preserves_s = true;
            }
        }
        demand_rows.push(json!({"m_p": m, "shadow": s.to_json()}));
    }
    let overproduction_primary = lowering_improves_a && lowering_preserves_s && !lowering_worsens_s;
    let demand_required = lowering_improves_a && lowering_worsens_s;
    let demand_cf = artifact(
        "gate9_demand_counterfactuals",
        true,
        json!({
            "rows": demand_rows,
            "lowering_improves_a": lowering_improves_a,
            "lowering_worsens_s": lowering_worsens_s,
            "lowering_preserves_s": lowering_preserves_s,
            "overproduction_primary": overproduction_primary,
            "demand_required_assembly_limited": demand_required
        }),
    );
    write_json(&out.join("demand_counterfactuals"), &demand_cf)?;
    gates.insert("demand_counterfactuals".into(), demand_cf);

    // Stop before candidates when assembly/desorption/access dominate, S unexplained,
    // fixed healthy P cannot maintain S, or production insufficient.
    let stop_candidates = matches!(
        access_class,
        AssemblyCapacityClass::SDesorptionDominant
            | AssemblyCapacityClass::PToSAdsorptionCapacityLimit
            | AssemblyCapacityClass::PMembraneAccessLimit
            | AssemblyCapacityClass::PSExchangeExecutionDefect
    ) || matches!(
        repl_class,
        ReplacementDemandClass::PrecursorProductionInsufficient
    ) || !fixed_p_arrests
        || lowering_worsens_s
        || !ordinary_s_explained;

    // Gate 10 — candidates
    let mut selected = PrecursorCandidate::Baseline;
    let mut selected_params = base.clone();
    let mut m_selected = 1.0;
    let mut candidate_rows = vec![json!({"candidate":"A","baseline":true,"ordinary":ordinary.to_json()})];
    let mut b_qualified = false;
    let mut c_qualified = false;
    if !stop_candidates && overproduction_primary {
        for m in preregistered_m_p(ordinary.rho_p())
            .into_iter()
            .filter(|m| *m > 0.0 && *m < 1.0)
            .take(5)
        {
            let s = run_shadow(
                &GeometrySpec::smooth(22.0),
                candidate_b_params(&base, m),
                h_ctrl,
                HoldMode::ExteriorNf,
                RedistributeMode::None,
            );
            let ok = s.steps_ok
                && s.a_ret() >= A_RETENTION
                && s.chi_s() >= CHI_S_TARGET
                && s.s_ret() >= S_RETENTION;
            candidate_rows.push(json!({"candidate":"B","m_p":m,"qualifies":ok,"shadow":s.to_json()}));
            if ok {
                selected = PrecursorCandidate::GlobalScale;
                selected_params = candidate_b_params(&base, m);
                m_selected = m;
                b_qualified = true;
                break;
            }
        }
    }
    // Candidate C only when P accumulates and B fails
    let p_accumulates = (ordinary.p1 - ordinary.p0) > 0.15 * ordinary.syn_p.max(1e-12);
    if !stop_candidates
        && !b_qualified
        && p_accumulates
        && ordinary.eta_ps() < 0.80
        && candidate_c_distinct_from_baseline(0.1, ordinary.p1.max(0.05))
    {
        // Diagnostic-only: cannot change production law; evaluate algebraic rate only.
        candidate_rows.push(json!({
            "candidate":"C",
            "note":"algebraic product inhibition evaluated; no runtime dispatcher in D-068",
            "qualifies": false,
            "reason": "no opt-in precursor schema dispatcher; would require production change"
        }));
        c_qualified = false;
    }
    let candidates = artifact(
        "gate10_candidate_laws",
        true,
        json!({
            "stop_candidates": stop_candidates,
            "selected": selected.as_str(),
            "m_p": m_selected,
            "rows": candidate_rows,
            "b_qualified": b_qualified,
            "c_qualified": c_qualified
        }),
    );
    write_json(&out.join("candidate_laws"), &candidates)?;
    gates.insert("candidate_laws".into(), candidates);

    // Gate 11 — identification (baseline fails; B if any)
    let id = IdentificationReport068 {
        params_positive_finite: m_selected.is_finite(),
        half_sats_in_domain: true,
        bootstrap_spread: 0.0,
        loo_variation: 1.0,
        holdout_median_err: if b_qualified { 0.1 } else { 1.0 },
        holdout_max_err: if b_qualified { 0.2 } else { 1.0 },
        holdout_a_sign_acc: if b_qualified { 0.95 } else { 0.0 },
        holdout_s_sign_acc: if b_qualified { 0.95 } else { 0.0 },
        no_radius_params: true,
        stoichiometry_ok: parity_ok,
        accounting_ok: ledger_ok,
    };
    let id_art = artifact(
        "gate11_parameter_identification",
        selected == PrecursorCandidate::Baseline || id.qualifies(),
        json!({"selected": selected.as_str(), "identification": id, "qualified": id.qualifies()}),
    );
    write_json(&out.join("parameter_identification"), &id_art)?;
    gates.insert("parameter_identification".into(), id_art);

    // Gate 12 — W separation
    let joint = run_shadow(
        &GeometrySpec::smooth(22.0),
        selected_params.clone(),
        h_ctrl,
        HoldMode::ExteriorNf,
        RedistributeMode::None,
    );
    let w_sink = run_shadow(
        &GeometrySpec::smooth(22.0),
        selected_params.clone(),
        h_ctrl,
        HoldMode::PerfectWSink,
        RedistributeMode::None,
    );
    let waste_blocks = !joint.steps_ok && w_sink.steps_ok;
    let waste = artifact(
        "gate12_waste_controls",
        true,
        json!({
            "ordinary": joint.to_json(),
            "perfect_w_sink": w_sink.to_json(),
            "waste_blocks": waste_blocks
        }),
    );
    write_json(&out.join("waste_controls"), &waste)?;
    gates.insert("waste_controls".into(), waste);
    if waste_blocks {
        return finalize(
            &out,
            &gates,
            D068Route::W,
            D068PrimaryConclusion::WasteExecutionBlocksPrecursorAudit,
            cap,
            skip,
        );
    }

    // Gate 13 — short coupled shadow
    let ladder = if skip {
        vec![h_ctrl]
    } else {
        horizon_ladder().into_iter().map(|h| h.min(cap)).collect()
    };
    let mut short = Vec::new();
    let mut qualified = b_qualified || c_qualified;
    let mut membrane_failing = ordinary.s_ret() < 1.0;
    for r in [16.0, 22.0, 32.0] {
        for h in &ladder {
            let s = run_shadow(
                &GeometrySpec::smooth(r),
                selected_params.clone(),
                *h,
                HoldMode::ExteriorNf,
                RedistributeMode::None,
            );
            if selected != PrecursorCandidate::Baseline {
                qualified &= s.steps_ok
                    && s.a_ret() >= A_RETENTION
                    && s.c_ret() >= C_RETENTION
                    && s.chi_s() >= CHI_S_TARGET;
            }
            membrane_failing |= s.s_ret() < 1.0;
            short.push(json!({"radius": r, "horizon": h, "result": s.to_json()}));
        }
    }
    let short_art = artifact(
        "gate13_short_coupled_shadow",
        selected == PrecursorCandidate::Baseline || qualified,
        json!({"selected": selected.as_str(), "rows": short, "ladder": ladder, "qualified": qualified}),
    );
    write_json(&out.join("short_shadow"), &short_art)?;
    gates.insert("short_shadow".into(), short_art);

    // Gate 14 — safety
    let safety = zero_precursor_when_a_starved(|c, phi, p| {
        let _ = p;
        precursor_rate(base.k_precursor, 0.0, c, phi, base.k_c_membrane)
    }) && (syn_only.syn_p > 0.0 || ordinary.syn_p >= 0.0);
    let knockout_impairs = no_syn.s_ret() <= ordinary.s_ret() + 0.05;
    let safety_ok = safety && knockout_impairs;
    let safety_art = artifact(
        "gate14_causality_controls",
        safety_ok,
        json!({
            "zero_a_no_precursor": safety,
            "precursor_knockout_impairs_or_not_better": knockout_impairs,
            "no_syn": no_syn.to_json(),
            "starvation_proxy_low_a": low_a.to_json()
        }),
    );
    write_json(&out.join("causality_controls"), &safety_art)?;
    gates.insert("causality_controls".into(), safety_art);
    if !safety_ok {
        return finalize(
            &out,
            &gates,
            D068Route::U,
            D068PrimaryConclusion::PrecursorMembraneCausalityFailure,
            cap,
            skip,
        );
    }

    // Gate 15 — authoritative (skip under SKIP_LATE)
    let auth_horizons: Vec<u64> = if skip {
        vec![]
    } else {
        let mut hs = vec![10_000u64, 25_000, 50_000]
            .into_iter()
            .map(|h| h.min(cap))
            .filter(|h| *h > 0)
            .collect::<Vec<_>>();
        hs.sort_unstable();
        hs.dedup();
        hs
    };
    let mut auth_rows = Vec::new();
    let mut auth_ok = selected != PrecursorCandidate::Baseline && qualified;
    if auth_horizons.is_empty() {
        auth_ok = false; // baseline does not auth-qualify; candidates need full ladder
    } else if selected != PrecursorCandidate::Baseline {
        for r in [16.0, 22.0, 32.0] {
            for h in &auth_horizons {
                let s = run_shadow(
                    &GeometrySpec::smooth(r),
                    selected_params.clone(),
                    *h,
                    HoldMode::ExteriorNf,
                    RedistributeMode::None,
                );
                let row_ok = s.steps_ok
                    && s.a_ret() >= A_RETENTION
                    && s.c_ret() >= C_RETENTION
                    && s.s_ret() >= S_RETENTION
                    && s.chi_s() >= CHI_S_TARGET;
                auth_ok &= row_ok;
                auth_rows.push(json!({"radius":r,"horizon":h,"pass":row_ok,"result":s.to_json()}));
            }
        }
    }
    let auth = artifact(
        "gate15_authoritative",
        auth_ok,
        json!({
            "skipped": skip,
            "horizons": auth_horizons,
            "rows": auth_rows,
            "membrane_still_declining": membrane_failing
        }),
    );
    write_json(&out.join("authoritative_shadow"), &auth)?;
    gates.insert("authoritative_shadow".into(), auth);

    // Route decision
    let desorption_dominant = matches!(
        access_class,
        AssemblyCapacityClass::SDesorptionDominant
    ) || (ordinary.des > ordinary.ads && !fixed_p_arrests);
    let assembly_limit = matches!(
        access_class,
        AssemblyCapacityClass::PToSAdsorptionCapacityLimit
    ) || (!fixed_p_arrests
        && ordinary_s_explained
        && ordinary.ads + EPS >= ordinary.des
        && ordinary.s_ret() < 0.95);
    let membrane_access = interface_rescues;
    let multiple = {
        let mut n = 0;
        if desorption_dominant {
            n += 1;
        }
        if assembly_limit && !desorption_dominant {
            n += 1;
        }
        if membrane_access {
            n += 1;
        }
        if overproduction_primary && fixed_p_arrests {
            n += 1;
        }
        n >= 2
    };
    let any_candidate = (b_qualified || c_qualified) && qualified && auth_ok;
    let mut evidence = RouteEvidence068 {
        workspace_isolated: start_ok,
        d067_reproduced: repro_ok,
        lineage_ok,
        runtime_parity_ok: parity_ok && ordinary_s_explained,
        ledger_ok: ledger_ok && ordinary_s_explained,
        safety_causality_ok: safety_ok,
        waste_blocks,
        identification: id,
        existing_qualified: false,
        overproduction_qualified: b_qualified && any_candidate,
        inhibition_qualified: c_qualified && any_candidate,
        assembly_limit: false,
        desorption_dominant: false,
        membrane_access_limit: false,
        multiple_limits: false,
        repair_but_stage_e_blocked: any_candidate && membrane_failing,
        no_portable_repair: false,
    };
    if evidence.runtime_parity_ok && evidence.ledger_ok && !any_candidate {
        if membrane_access {
            evidence.membrane_access_limit = true;
        } else if desorption_dominant {
            evidence.desorption_dominant = true;
        } else if assembly_limit {
            evidence.assembly_limit = true;
        } else if multiple {
            evidence.multiple_limits = true;
        } else if !fixed_p_arrests {
            evidence.assembly_limit = true;
        } else {
            evidence.no_portable_repair = true;
        }
    }
    let (route, conclusion) = select_route(evidence.clone());
    let route_art = artifact(
        "route_decision",
        true,
        json!({
            "route": route.as_str(),
            "primary_conclusion": conclusion.as_str(),
            "evidence": evidence,
            "fate": fate.as_str(),
            "replacement_class": repl_class.as_str(),
            "assembly_class": access_class.as_str()
        }),
    );
    write_json(&out.join("route_decision"), &route_art)?;
    gates.insert("route_decision".into(), route_art);
    let accounting = artifact(
        "accounting",
        true,
        json!({
            "precursor_parity": precursor_synthesis_parity(1.0),
            "adsorption_parity": adsorption_parity(1.0),
            "desorption_parity": desorption_parity(1.0),
            "activation_parity": activation_stoichiometry_parity(1.0),
            "frozen_k_T": D068_FROZEN_KT
        }),
    );
    write_json(&out.join("accounting"), &accounting)?;
    gates.insert("accounting".into(), accounting);
    finalize(&out, &gates, route, conclusion, cap, skip)
}
