//! D-088 surplus-driven local structural growth (additive; Phase 1 laws frozen).
//!
//! Default path: growth from local A surplus (frozen D-088).
//! D-091 reserve schema: growth from accumulated R only (A surplus does not grow).

use crate::material_mesh::MaterialMesh;
use crate::mesh_reactions::{q_catalyst, ReactionLedger, ReactionParams};
use serde::{Deserialize, Serialize};

/// Analytically derived global structural growth yields (at most three).
pub const Y_G_CANDIDATES: [f64; 3] = [0.90, 1.10, 1.30];

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GrowthParams {
    /// Selected global structural yield y_g.
    pub y_g: f64,
    /// Enable surplus growth flux (false = Phase 1 laws only).
    pub enable_growth: bool,
}

impl Default for GrowthParams {
    fn default() -> Self {
        Self {
            y_g: Y_G_CANDIDATES[0],
            enable_growth: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrowthLedger {
    pub a_surplus_total: f64,
    pub a_consumed_growth: f64,
    pub m_grown: f64,
    pub w_from_growth: f64,
    /// Reserve mass consumed into structure under D-091 schema.
    #[serde(default)]
    pub r_consumed_growth: f64,
}

/// Bounded local gate h(ε, turn): stretched / straighter edges incorporate surplus preferentially.
#[inline]
pub fn h_strain(eps: f64) -> f64 {
    let e = eps.max(0.0);
    (0.15 + 0.85 * e / (0.20 + e)).clamp(0.0, 1.0)
}

#[inline]
pub fn h_local(eps: f64, cos_turn: f64) -> f64 {
    let h_e = h_strain(eps);
    // Local turning only: straighter segments elongate under surplus (no global axis).
    let h_c = 0.65 + 0.35 * cos_turn.clamp(-1.0, 1.0).max(0.0);
    (h_e * h_c).clamp(0.0, 1.0)
}

/// Local maintenance A demand rate for edge i.
/// Includes Phase 1 structural *turnover replacement*, catalyst share, and membrane production share.
/// Does **not** include the Phase 1 build flux (that remains the frozen homeostasis/repair law).
pub fn local_maintenance_a_rate(mesh: &MaterialMesh, i: usize, p: &ReactionParams) -> f64 {
    if mesh.edges[i].ruptured {
        return 0.0;
    }
    let ell = mesh.edge_length(i);
    let peri = mesh.perimeter().max(1e-6);
    let share = ell / peri;
    let qc = q_catalyst(mesh.interior.c, p.q_c);
    let a = mesh.interior.a.max(0.0);
    // A required to replace structural turnover on this edge.
    let turn = p.k_turn * mesh.edges[i].m.max(0.0);
    let a_turn = turn / p.yield_a_to_m.max(1e-15);
    let a_c_share = p.k_c_prod * a * share;
    let a_l_share = 0.02 * qc * a * ell;
    a_turn + a_c_share + a_l_share
}

/// Local A production share for edge i (from lumped activation, area-distributed by length).
pub fn local_a_production_rate(mesh: &MaterialMesh, i: usize, p: &ReactionParams) -> f64 {
    let ell = mesh.edge_length(i);
    let peri = mesh.perimeter().max(1e-6);
    let share = ell / peri;
    let area = mesh.area().max(1e-6);
    let qc = q_catalyst(mesh.interior.c, p.q_c);
    let gh = if p.composition.enable {
        let z = crate::catalyst_composition::composition_z(mesh.interior.c_h, mesh.interior.c_b);
        crate::catalyst_composition::g_harvest(z, p.composition.sigma)
    } else {
        1.0
    };
    let j_act = p.k_act
        * qc
        * gh
        * mesh.interior.n.max(0.0)
        * mesh.interior.f.max(0.0)
        * area;
    // Rate of A mass added to interior pool attributable to this segment.
    j_act * share
}

pub fn local_a_surplus_rate(mesh: &MaterialMesh, i: usize, p: &ReactionParams) -> f64 {
    (local_a_production_rate(mesh, i, p) - local_maintenance_a_rate(mesh, i, p)).max(0.0)
}

fn edge_cos_turn(mesh: &MaterialMesh, i: usize) -> f64 {
    let n = mesh.n();
    let p0 = mesh.vertices[(i + n - 1) % n];
    let p1 = mesh.vertices[i];
    let p2 = mesh.vertices[(i + 1) % n];
    let a = [p1[0] - p0[0], p1[1] - p0[1]];
    let b = [p2[0] - p1[0], p2[1] - p1[1]];
    let la = (a[0] * a[0] + a[1] * a[1]).sqrt().max(1e-15);
    let lb = (b[0] * b[0] + b[1] * b[1]).sqrt().max(1e-15);
    (a[0] * b[0] + a[1] * b[1]) / (la * lb)
}

/// Additive growth step.
/// - Reserve disabled: J_growth,i = y_g · J_A,surplus,i · h(ε_i) (frozen D-088).
/// - Reserve enabled: J_growth,i = y_g · g_build · q(C) · R/(K_g+R) · h(ε) · share; consumes R.
pub fn growth_step(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    growth: &GrowthParams,
    dt: f64,
) -> GrowthLedger {
    let mut led = GrowthLedger::default();
    if !growth.enable_growth || !mesh.alive {
        return led;
    }
    let n = mesh.n();
    let area = mesh.area().max(1e-6);

    if react.reserve.enable {
        // D-091: growth funded by R only. No instantaneous A surplus coupling.
        if !crate::metabolic_reserve::reserve_schema_load_ok(mesh, &react.reserve) {
            return led;
        }
        for i in 0..n {
            if mesh.edges[i].ruptured {
                continue;
            }
            let j_mass =
                crate::metabolic_reserve::local_r_growth_rate(mesh, i, react, growth.y_g) * dt;
            if j_mass <= 0.0 {
                continue;
            }
            let have_r = mesh.interior.r.max(0.0) * area;
            let take = j_mass.min(have_r);
            if take <= 0.0 {
                continue;
            }
            mesh.interior.r = (mesh.interior.r - take / area).max(0.0);
            mesh.interior.w += take / area;
            let dm = take * growth.y_g.max(0.0);
            mesh.edges[i].m += dm;
            led.r_consumed_growth += take;
            led.m_grown += dm;
            led.w_from_growth += take;
        }
        return led;
    }

    for i in 0..n {
        if mesh.edges[i].ruptured {
            continue;
        }
        let surplus = local_a_surplus_rate(mesh, i, react);
        led.a_surplus_total += surplus * dt;
        let gb = if react.composition.enable {
            let z = crate::catalyst_composition::composition_z(mesh.interior.c_h, mesh.interior.c_b);
            crate::catalyst_composition::g_build(z, react.composition.sigma)
        } else {
            1.0
        };
        let j_g = growth.y_g * surplus * h_local(mesh.strain(i), edge_cos_turn(mesh, i)) * gb * dt;
        if j_g <= 0.0 {
            continue;
        }
        let have = mesh.interior.a.max(0.0) * area;
        let take = j_g.min(have);
        if take <= 0.0 {
            continue;
        }
        mesh.interior.a = (mesh.interior.a - take / area).max(0.0);
        mesh.interior.w += take / area;
        // Structural mass from surplus A at yield y_g (A→m).
        let dm = take * growth.y_g.max(0.0);
        mesh.edges[i].m += dm;
        led.a_consumed_growth += take;
        led.m_grown += dm;
        led.w_from_growth += take;
    }
    led
}

/// Observer shape factor Ψ = P²/(4πA). Biology must not read this.
pub fn shape_factor_psi(mesh: &MaterialMesh) -> f64 {
    let a = mesh.area().max(1e-15);
    let p = mesh.perimeter();
    (p * p) / (4.0 * std::f64::consts::PI * a)
}

/// Absorb growth ledger fields into reaction-style totals when needed.
pub fn merge_growth_into_reaction(r: &mut ReactionLedger, g: &GrowthLedger) {
    r.a_consumed_build += g.a_consumed_growth;
    r.m_produced += g.m_grown;
    r.w_produced += g.w_from_growth;
    r.reserve.r_to_m += g.r_consumed_growth;
    r.reserve.w_from_r_growth += g.w_from_growth;
}
