//! Independent simulation driver — uses mesh kernels, never d086_analysis gates.

use chemistry_core::material_mesh::{
    LumpedChem, MaterialMesh, DEFAULT_REBOND_DIST, DEFAULT_RHO_S,
};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{
    pulse_tracers, reactions_step, tracer_catalyst_fraction, tracer_membrane_fraction,
    tracer_structural_fraction, try_local_rebond, ReactionLedger, ReactionParams,
};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use serde::{Deserialize, Serialize};

use crate::frozen::{frozen_reactions, frozen_transport, FROZEN_CENTER, FROZEN_SCHEMA};
use crate::metrics::{replacement_report, retention_report, ReplacementReport, RetentionReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepLedger {
    pub reactions: ReactionLedger,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccumLedger {
    pub m_produced: f64,
    pub m_to_w: f64,
    pub bind_extent: f64,
    pub unbind_extent: f64,
    pub c_produced: f64,
    pub c_turned: f64,
    pub l_produced: f64,
    pub a_produced: f64,
    #[serde(default)]
    pub reserve_a_to_r: f64,
    #[serde(default)]
    pub reserve_r_to_a: f64,
    #[serde(default)]
    pub reserve_r_to_w: f64,
    #[serde(default)]
    pub reserve_rejected_steps: u64,
}

impl AccumLedger {
    pub fn absorb(&mut self, r: &ReactionLedger) {
        self.m_produced += r.m_produced;
        self.m_to_w += r.m_to_w;
        self.bind_extent += r.bind_extent;
        self.unbind_extent += r.unbind_extent;
        self.c_produced += r.c_produced;
        self.c_turned += r.c_turned;
        self.l_produced += r.l_produced;
        self.a_produced += r.a_produced;
        self.reserve_a_to_r += r.reserve.a_to_r;
        self.reserve_r_to_a += r.reserve.r_to_a;
        self.reserve_r_to_w += r.reserve.r_to_w;
        self.reserve_rejected_steps += r.reserve.rejected_steps;
    }
}

pub fn seed_mesh(radius: f64, seed: u64) -> MaterialMesh {
    let n = 24 + ((seed % 3) as usize);
    let interior = LumpedChem {
        c: 0.8,
        a: 0.5,
        n: 0.4,
        f: 0.4,
        w: 0.1,
        tracer_c: 0.0,
            c_h: 0.0,
            c_b: 0.0,
            r: 0.0,
            u_h: 0.0,
            u_b: 0.0,
            k_h: 0.0,
            k_b: 0.0,
            q_k: 0.0,
            q_e: 0.0,
            k_a: 0.0,
            k_r: 0.0,
            k_node_b: 0.0,
        };
    let exterior = LumpedChem {
        c: 0.0,
        a: 0.0,
        n: 1.0,
        f: 1.0,
        w: 0.0,
        tracer_c: 0.0,
            c_h: 0.0,
            c_b: 0.0,
            r: 0.0,
            u_h: 0.0,
            u_b: 0.0,
            k_h: 0.0,
            k_b: 0.0,
            q_k: 0.0,
            q_e: 0.0,
            k_a: 0.0,
            k_r: 0.0,
            k_node_b: 0.0,
        };
    let mut mesh = MaterialMesh::seed_regular(
        n,
        radius,
        40.0,
        40.0,
        DEFAULT_RHO_S,
        0.7,
        interior,
        exterior,
        5.0,
    );
    if reserve_enabled() {
        stamp_reserve_equation(&mut mesh);
    }
    if conservative_v2_enabled() {
        mesh.stamp_conservative_schema();
    }
    mesh
}

/// Select the physical/material contract for the observer-only R9 matrix.
///
/// The R9-R3 selector is deliberately independent from reserve selection. The
/// older R9-R2 switch remains supported so historical workflows retain their
/// exact behavior when the new selectors are absent.
pub fn conservative_v2_enabled() -> bool {
    match std::env::var("DCDEV020R9R3_CONTRACT").ok().as_deref() {
        Some("ConservativeV2") => true,
        Some("HistoricalV1") => false,
        Some(_) => false,
        None => std::env::var("DCDEV020R9R2_V2").as_deref() == Ok("1"),
    }
}

/// Select D-091 reserve physiology independently from the material contract.
pub fn reserve_enabled() -> bool {
    match std::env::var("DCDEV020R9R3_RESERVE").ok().as_deref() {
        Some("1") | Some("on") | Some("ON") | Some("true") => true,
        Some("0") | Some("off") | Some("OFF") | Some("false") => false,
        Some(_) => false,
        None => conservative_v2_enabled(),
    }
}

pub fn contract_label() -> &'static str {
    if conservative_v2_enabled() {
        "ConservativeV2"
    } else {
        "HistoricalV1"
    }
}

pub fn equation_lineage() -> &'static str {
    if reserve_enabled() {
        chemistry_core::metabolic_reserve::EQUATION_VERSION_METABOLIC_RESERVE
    } else {
        FROZEN_SCHEMA
    }
}

pub fn reaction_params_for(mesh: &MaterialMesh) -> ReactionParams {
    let mut params = if conservative_v2_enabled() {
        ReactionParams::conservative_v2()
    } else {
        frozen_reactions()
    };
    if reserve_enabled() {
        params.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    }
    params
}

/// Lawful perturbation of a seeded mesh (deterministic path for Gate 3).
pub fn perturb_mesh(mesh: &mut MaterialMesh, kind: &str, mag: f64) {
    match kind {
        "vertex_noise" => {
            for (i, v) in mesh.vertices.iter_mut().enumerate() {
                let s = ((i as f64 + 1.0) * 12.9898).sin() * 43758.5453;
                let f = s - s.floor();
                v[0] += mag * (f - 0.5);
                v[1] += mag * ((f * 7.13).fract() - 0.5);
            }
        }
        "c_noise" => {
            mesh.interior.c = (mesh.interior.c * (1.0 + mag)).max(0.0);
        }
        "a_noise" => {
            mesh.interior.a = (mesh.interior.a * (1.0 + mag)).max(0.0);
        }
        "l_noise" => {
            mesh.free_l = (mesh.free_l * (1.0 + mag)).max(0.0);
        }
        "env_nf" => {
            mesh.exterior.n = (mesh.exterior.n * (1.0 + mag)).max(0.0);
            mesh.exterior.f = (mesh.exterior.f * (1.0 + mag)).max(0.0);
        }
        "rotate" => {
            let c = mesh.centroid();
            let ang = mag;
            let (s, co) = (ang.sin(), ang.cos());
            for v in &mut mesh.vertices {
                let x = v[0] - c[0];
                let y = v[1] - c[1];
                v[0] = c[0] + co * x - s * y;
                v[1] = c[1] + s * x + co * y;
            }
        }
        "translate" => {
            for v in &mut mesh.vertices {
                v[0] += mag;
                v[1] += mag * 0.5;
            }
        }
        _ => {}
    }
}

pub fn dish_contact(mesh: &MaterialMesh) -> bool {
    mesh.vertices
        .iter()
        .any(|p| p[0] < 2.0 || p[1] < 2.0 || p[0] > 78.0 || p[1] > 78.0)
}

pub fn coupled_step(
    mesh: &mut MaterialMesh,
    mech: &MechParams,
    react: &ReactionParams,
    transport: &TransportParams,
    build: bool,
    metab: bool,
) -> ReactionLedger {
    let _ = transport_step(mesh, transport, mech.dt);
    let led = reactions_step(mesh, react, mech.dt, build, metab);
    mechanics_step(mesh, mech);
    remesh(mesh);
    try_local_rebond(mesh, DEFAULT_REBOND_DIST);
    led
}

pub fn run_coupled(
    mesh: &mut MaterialMesh,
    steps: usize,
    build: bool,
    metab: bool,
) -> AccumLedger {
    let mech = FROZEN_CENTER;
    let react = reaction_params_for(mesh);
    let transport = frozen_transport();
    let mut acc = AccumLedger::default();
    for _ in 0..steps {
        if !mesh.can_advance_physics() {
            break;
        }
        let led = coupled_step(mesh, &mech, &react, &transport, build, metab);
        acc.absorb(&led);
    }
    acc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnoverAudit {
    pub retention_c: RetentionReport,
    pub retention_a: RetentionReport,
    pub structural: ReplacementReport,
    pub membrane: ReplacementReport,
    pub catalyst: ReplacementReport,
    pub d086_tracer_interpretation: Vec<String>,
    pub dual_requirement_pass: bool,
    pub d086_pool_m: f64,
    pub d086_pool_b: f64,
    pub d086_pool_c: f64,
    pub d086_soft_pass: bool,
}

pub fn audit_turnover(steps: usize) -> TurnoverAudit {
    let mut mesh = seed_mesh(14.0, 2);
    pulse_tracers(&mut mesh, 1.0);
    let label_m0: f64 = mesh.edges.iter().map(|e| e.tracer_m).sum();
    let label_b0: f64 = mesh.edges.iter().map(|e| e.tracer_b).sum();
    let label_c0 = mesh.interior.tracer_c;
    let mut series_c = vec![mesh.interior.c];
    let mut series_a = vec![mesh.interior.a];
    let mut mass_m = Vec::new();
    let mut mass_b = Vec::new();
    let mut mass_c = Vec::new();
    let mut acc = AccumLedger::default();
    let mech = FROZEN_CENTER;
    let react = reaction_params_for(&mesh);
    let transport = frozen_transport();
    for _ in 0..steps {
        if !mesh.can_advance_physics() {
            break;
        }
        let led = coupled_step(&mut mesh, &mech, &react, &transport, true, true);
        acc.absorb(&led);
        series_c.push(mesh.interior.c);
        series_a.push(mesh.interior.a);
        mass_m.push(mesh.total_structural_mass());
        mass_b.push(mesh.total_bound_membrane());
        mass_c.push(mesh.interior.c * mesh.area().max(1e-6));
    }
    let mean = |v: &[f64]| {
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };
    let label_m_t: f64 = mesh.edges.iter().map(|e| e.tracer_m).sum();
    let label_b_t: f64 = mesh.edges.iter().map(|e| e.tracer_b).sum();
    let label_c_t = mesh.interior.tracer_c;
    // Gross replacement: newly incorporated material (production / bind / c_prod).
    let structural = replacement_report(
        "m",
        mean(&mass_m),
        acc.m_produced,
        label_m0,
        label_m_t,
        mesh.total_structural_mass(),
    );
    let membrane = replacement_report(
        "b",
        mean(&mass_b),
        acc.bind_extent,
        label_b0,
        label_b_t,
        mesh.total_bound_membrane(),
    );
    let catalyst = replacement_report(
        "C",
        mean(&mass_c),
        acc.c_produced,
        label_c0,
        label_c_t,
        mesh.interior.c.max(1e-15),
    );
    let dual = structural.r_x_ok
        && structural.f_label_ok
        && membrane.r_x_ok
        && membrane.f_label_ok
        && catalyst.r_x_ok
        && catalyst.f_label_ok;
    let d086_pool_m = tracer_structural_fraction(&mesh);
    let d086_pool_b = tracer_membrane_fraction(&mesh);
    let d086_pool_c = tracer_catalyst_fraction(&mesh);
    TurnoverAudit {
        retention_c: retention_report("C", &series_c, acc.c_produced),
        retention_a: retention_report("A", &series_a, acc.a_produced),
        structural,
        membrane,
        catalyst,
        d086_tracer_interpretation: vec![
            crate::metrics::interpret_d086_tracer("m", 0.35),
            crate::metrics::interpret_d086_tracer("b", 0.00),
            crate::metrics::interpret_d086_tracer("c", 0.23),
            format!(
                "independent_recompute_f_pool: m={d086_pool_m:.3} b={d086_pool_b:.3} c={d086_pool_c:.3} (D-086 reported ~0.35/0.00/0.23)"
            ),
        ],
        dual_requirement_pass: dual,
        d086_pool_m,
        d086_pool_b,
        d086_pool_c,
        d086_soft_pass: d086_pool_m < 0.55 && d086_pool_b < 0.70 && d086_pool_c < 0.70,
    }
}

pub fn fingerprint(mesh: &MaterialMesh) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    mesh.n().hash(&mut h);
    for v in &mesh.vertices {
        ((v[0] * 1e6).round() as i64).hash(&mut h);
        ((v[1] * 1e6).round() as i64).hash(&mut h);
    }
    ((mesh.total_structural_mass() * 1e6).round() as i64).hash(&mut h);
    ((mesh.interior.c * 1e6).round() as i64).hash(&mut h);
    ((mesh.interior.a * 1e6).round() as i64).hash(&mut h);
    mesh.alive.hash(&mut h);
    h.finish()
}

pub fn pass_basin_row(mesh: &MaterialMesh, a0: f64, c0: f64, aa0: f64) -> bool {
    let a1 = mesh.area();
    let c_ret = if c0 <= 1e-15 {
        1.0
    } else {
        mesh.interior.c / c0
    };
    let a_ret = if aa0 <= 1e-15 {
        1.0
    } else {
        mesh.interior.a / aa0
    };
    mesh.alive
        && mesh.closed_intact()
        && !dish_contact(mesh)
        && a1 > 0.2 * a0
        && a1 < 5.0 * a0
        && c_ret >= crate::metrics::RETENTION_MIN
        && a_ret >= crate::metrics::RETENTION_MIN
}

pub use chemistry_core::mesh_reactions::{
    apply_local_rupture, apply_membrane_damage, apply_structural_damage, evaluate_death,
};
