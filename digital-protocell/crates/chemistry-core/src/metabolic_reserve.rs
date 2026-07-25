//! D-091 metabolic reserve R — local conserved activated-resource storage.
//!
//! R is material, not readiness/age/fitness/division progress.
//! Topology and fission never read R.

use crate::material_mesh::{
    MaterialMesh, EQUATION_VERSION_MATERIAL_MESH, MATERIAL_MESH_SCHEMA_VERSION,
};
use crate::mesh_reactions::{q_catalyst, ReactionParams};
use serde::{Deserialize, Serialize};

pub const EQUATION_VERSION_METABOLIC_RESERVE: &str = "autopoietic_material_mesh_metabolic_reserve_v1";
pub const FIELD_SCHEMA_METABOLIC_RESERVE: &str = "mesh_vertices_edges_catalyst_composition_reserve_v1";

/// Charging timescale multipliers on the maintenance horizon (at most three).
pub const STORE_HORIZON_CANDIDATES: [f64; 3] = [2.0, 4.0, 8.0];

const EPS: f64 = 1e-15;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReserveParams {
    /// When false, D-088 instantaneous surplus-A growth path is used.
    pub enable: bool,
    pub k_store: f64,
    pub k_release: f64,
    pub k_r_loss: f64,
    pub k_store_half: f64,
    pub k_low: f64,
    /// Michaelis constant K_R for release: R/(K_R+R).
    pub k_r: f64,
    pub k_growth: f64,
    pub r_max: f64,
    /// Selected charging horizon multiplier (2/4/8 × maintenance horizon).
    pub store_horizon_mult: f64,
}

impl Default for ReserveParams {
    fn default() -> Self {
        Self {
            enable: false,
            k_store: 0.0,
            k_release: 0.0,
            k_r_loss: 0.0,
            k_store_half: 0.5,
            k_low: 0.25,
            k_r: 0.25,
            k_growth: 0.5,
            r_max: 2.0,
            store_horizon_mult: 4.0,
        }
    }
}

impl ReserveParams {
    /// Build from sealed Phase-1 / D-088 horizons and local A statistics.
    ///
    /// - `t_replace` = Phase 1 structural replacement horizon (1/k_turn)
    /// - `t_maint` = maintenance horizon (A-demand based)
    /// - `a_median` / `a_q25` from viable local A under certified maintenance
    /// - `store_mult` ∈ {2,4,8} × maintenance horizon for charging timescale
    pub fn derived(
        t_replace: f64,
        t_maint: f64,
        a_median: f64,
        a_q25: f64,
        store_mult: f64,
        fission_a_cost: f64,
        area: f64,
    ) -> Self {
        let t_replace = t_replace.max(1.0);
        let t_maint = t_maint.max(1.0);
        let store_mult = store_mult.max(1.0);
        let t_store = store_mult * t_maint;
        // Half-life of R = 4 × replacement horizon → k = ln2 / t½
        let t_half = 4.0 * t_replace;
        let k_r_loss = std::f64::consts::LN_2 / t_half;
        // Release timescale ≈ one maintenance horizon (Michaelis saturating).
        let k_release = 1.0 / t_maint;
        // Charging: characteristic fill of R_max over t_store at A ≫ K_store.
        let r_max = (fission_a_cost / area.max(EPS)).max(a_median * 2.0).max(1.0);
        let k_store = r_max / t_store.max(EPS);
        // K_growth must be large vs k_store so R accumulates before reproductive-scale growth.
        // At low R: dR/dt ≈ k_store − y_g·q·h·R/K_g; choose K_g so steady R ≳ 0.35 R_max.
        let y_g = 0.9;
        let qc_typ = 0.7;
        let h_typ = 0.5;
        let k_growth = ((0.35 * r_max) * y_g * qc_typ * h_typ / k_store.max(EPS)).max(r_max);
        Self {
            enable: true,
            k_store,
            k_release,
            k_r_loss,
            k_store_half: a_median.max(0.05),
            // Release gate: use lower quartile, but keep headroom so maintenance A does not
            // continuously empty R (tight A distributions otherwise keep release half-on).
            k_low: (a_q25 * 0.65).max(0.05),
            k_r: (r_max * 0.15).max(0.05),
            k_growth,
            r_max,
            store_horizon_mult: store_mult,
        }
    }

    pub fn candidate_identity_suffix(&self) -> String {
        format!(
            "reserve:k_store={:.6e}:k_rel={:.6e}:k_loss={:.6e}:Ks={:.6}:Kl={:.6}:Kr={:.6}:Kg={:.6}:Rmax={:.6}:H={:.3}",
            self.k_store,
            self.k_release,
            self.k_r_loss,
            self.k_store_half,
            self.k_low,
            self.k_r,
            self.k_growth,
            self.r_max,
            self.store_horizon_mult
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReserveLedger {
    pub a_to_r: f64,
    pub r_to_a: f64,
    pub r_to_w: f64,
    pub r_to_m: f64,
    pub w_from_r_growth: f64,
    pub rejected_steps: u64,
}

/// Stamp mesh equation identity for the metabolic-reserve schema.
pub fn stamp_reserve_equation(mesh: &mut MaterialMesh) {
    mesh.equation_id = EQUATION_VERSION_METABOLIC_RESERVE.to_string();
    mesh.schema_version = MATERIAL_MESH_SCHEMA_VERSION + 1;
}

pub fn stamp_base_equation(mesh: &mut MaterialMesh) {
    mesh.equation_id = EQUATION_VERSION_MATERIAL_MESH.to_string();
    mesh.schema_version = MATERIAL_MESH_SCHEMA_VERSION;
}

/// Reject loading/running reserve chemistry on an unmarked (old) mesh snapshot.
pub fn reserve_schema_load_ok(mesh: &MaterialMesh, reserve: &ReserveParams) -> bool {
    if !reserve.enable {
        return true;
    }
    mesh.equation_id == EQUATION_VERSION_METABOLIC_RESERVE
        || mesh.equation_id == crate::template_polymer::EQUATION_VERSION_CATALYTIC_TEMPLATE
        || mesh.equation_id == crate::template_network::EQUATION_VERSION_TEMPLATE_NETWORK
        || mesh.equation_id == crate::autocatalytic_nodes::EQUATION_VERSION_AUTOCATALYTIC_SET
}

/// Store flux density (concentration/time): A → R.
#[inline]
pub fn j_store(a: f64, r: f64, qc: f64, p: &ReserveParams) -> f64 {
    if !p.enable || p.k_store <= 0.0 || p.r_max <= EPS {
        return 0.0;
    }
    let a = a.max(0.0);
    let r = r.max(0.0);
    let sat = (a * a) / (p.k_store_half * p.k_store_half + a * a + EPS);
    let room = (1.0 - r / p.r_max).max(0.0);
    p.k_store * qc * sat * room
}

/// Release flux density: R → A (strongest when A is locally low).
#[inline]
pub fn j_release(a: f64, r: f64, qc: f64, p: &ReserveParams) -> f64 {
    if !p.enable || p.k_release <= 0.0 {
        return 0.0;
    }
    let a = a.max(0.0);
    let r = r.max(0.0);
    let r_term = r / (p.k_r + r + EPS);
    let low_a = p.k_low / (p.k_low + a + EPS);
    p.k_release * qc * r_term * low_a
}

/// Slow physical loss: R → W.
#[inline]
pub fn j_r_loss(r: f64, p: &ReserveParams) -> f64 {
    if !p.enable || p.k_r_loss <= 0.0 {
        return 0.0;
    }
    p.k_r_loss * r.max(0.0)
}

/// Local reserve chemistry step (lumped interior). Conserves A↔R activation equivalents.
pub fn reserve_metab_step(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    dt: f64,
) -> ReserveLedger {
    let mut led = ReserveLedger::default();
    let p = &react.reserve;
    if !p.enable || !mesh.alive || dt <= 0.0 {
        return led;
    }
    if !reserve_schema_load_ok(mesh, p) {
        led.rejected_steps += 1;
        return led;
    }
    let area = mesh.area().max(EPS);
    let qc = q_catalyst(mesh.interior.c, react.q_c);
    let (qc_store, qc_rel) = if react.autocatalytic.enable {
        let g = crate::autocatalytic_nodes::node_storage_release_gain(
            mesh,
            &react.autocatalytic,
            react.q_c,
        );
        (qc * g, qc * g)
    } else if react.network.enable {
        let gs = crate::template_network_expression::network_storage_gain(
            mesh,
            &react.network,
            react.q_c,
        );
        let gr = crate::template_network_expression::network_release_gain(
            mesh,
            &react.network,
            react.q_c,
        );
        (qc * gs, qc * gr)
    } else {
        (qc, qc)
    };
    let a0 = mesh.interior.a.max(0.0);
    let r0 = mesh.interior.r.max(0.0);

    // Explicit Euler with capacity clamps (rejected partial steps leave state unchanged for that flux).
    let js = j_store(a0, r0, qc_store, p) * dt;
    let jr = j_release(a0, r0, qc_rel, p) * dt;
    let jl = j_r_loss(r0, p) * dt;

    // Store A→R
    let store = js.min(a0).min((p.r_max - r0).max(0.0));
    if store > 0.0 {
        mesh.interior.a = (a0 - store).max(0.0);
        mesh.interior.r = (r0 + store).min(p.r_max);
        led.a_to_r += store * area;
    } else if js > EPS && (a0 <= EPS || r0 >= p.r_max - EPS) {
        led.rejected_steps += 1;
    }

    let a1 = mesh.interior.a.max(0.0);
    let r1 = mesh.interior.r.max(0.0);

    // Release R→A
    let release = jr.min(r1);
    if release > 0.0 {
        mesh.interior.r = (r1 - release).max(0.0);
        mesh.interior.a = a1 + release;
        led.r_to_a += release * area;
    } else if jr > EPS && r1 <= EPS {
        led.rejected_steps += 1;
    }

    let r2 = mesh.interior.r.max(0.0);
    // Loss R→W
    let loss = jl.min(r2);
    if loss > 0.0 {
        mesh.interior.r = (r2 - loss).max(0.0);
        mesh.interior.w += loss;
        led.r_to_w += loss * area;
    } else if jl > EPS && r2 <= EPS {
        led.rejected_steps += 1;
    }

    mesh.interior.r = mesh.interior.r.clamp(0.0, p.r_max);
    led
}

/// Growth flux from R (schema-enabled path only): J_growth = y_g · g_build · q(C) · R/(K_g+R) · h(ε)
#[inline]
pub fn local_r_growth_rate(
    mesh: &MaterialMesh,
    i: usize,
    react: &ReactionParams,
    y_g: f64,
) -> f64 {
    let p = &react.reserve;
    if !p.enable || mesh.edges[i].ruptured {
        return 0.0;
    }
    let r = mesh.interior.r.max(0.0);
    let qc = q_catalyst(mesh.interior.c, react.q_c);
    let gb = if react.composition.enable {
        let z = crate::catalyst_composition::composition_z(mesh.interior.c_h, mesh.interior.c_b);
        crate::catalyst_composition::g_build(z, react.composition.sigma)
    } else if react.autocatalytic.enable {
        crate::autocatalytic_nodes::node_building_gain(mesh, &react.autocatalytic, react.q_c)
    } else if react.network.enable {
        crate::template_network_expression::network_building_gain(mesh, &react.network, react.q_c)
    } else if react.template.enable {
        crate::template_motifs::template_activity_gains(mesh, &react.template).1
    } else {
        1.0
    };
    let n = mesh.n();
    let cos_turn = {
        let p0 = mesh.vertices[(i + n - 1) % n];
        let p1 = mesh.vertices[i];
        let p2 = mesh.vertices[(i + 1) % n];
        let a = [p1[0] - p0[0], p1[1] - p0[1]];
        let b = [p2[0] - p1[0], p2[1] - p1[1]];
        let la = (a[0] * a[0] + a[1] * a[1]).sqrt().max(EPS);
        let lb = (b[0] * b[0] + b[1] * b[1]).sqrt().max(EPS);
        (a[0] * b[0] + a[1] * b[1]) / (la * lb)
    };
    // Same frozen D-088 local strain gate (duplicated to avoid module cycle).
    let eps = mesh.strain(i).max(0.0);
    let h_e = (0.15 + 0.85 * eps / (0.20 + eps)).clamp(0.0, 1.0);
    let h_c = 0.65 + 0.35 * cos_turn.clamp(-1.0, 1.0).max(0.0);
    let h = (h_e * h_c).clamp(0.0, 1.0);
    let r_sat = r / (p.k_growth + r + EPS);
    let share = mesh.edge_length(i) / mesh.perimeter().max(EPS);
    y_g.max(0.0) * gb * qc * r_sat * h * share * mesh.area().max(EPS)
}
