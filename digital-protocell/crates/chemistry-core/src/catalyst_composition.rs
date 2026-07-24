//! D-089 compositional catalytic materials C_H / C_B.
//!
//! Catalyst materials only — not genes, fitness, or organism settings.
//! Mutation occurs solely during catalyst production; division never mutates.

use crate::material_mesh::{LumpedChem, MaterialMesh};
use serde::{Deserialize, Serialize};

pub const EQUATION_VERSION_CATALYTIC_COMPOSITION: &str =
    "autopoietic_material_mesh_catalytic_composition_v1";
pub const FIELD_SCHEMA_CATALYST_COMPOSITION: &str = "mesh_vertices_edges_catalyst_composition_v1";

/// Frozen tradeoff strength σ (Gate 0 uses σ=0 for preservation).
pub const SIGMA_TRADEOFF: f64 = 0.15;

const EPS: f64 = 1e-15;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CompositionParams {
    /// When false, scalar catalyst path is used (D-086/D-088 frozen behavior).
    pub enable: bool,
    /// Global mutation probability during catalyst production (frozen once derived).
    pub mu: f64,
    /// Tradeoff strength; 0 → g_harvest = g_build = 1.
    pub sigma: f64,
}

impl Default for CompositionParams {
    fn default() -> Self {
        Self {
            enable: false,
            mu: 0.0,
            sigma: 0.0,
        }
    }
}

impl CompositionParams {
    pub fn evolutionary(mu: f64) -> Self {
        Self {
            enable: true,
            mu,
            sigma: SIGMA_TRADEOFF,
        }
    }

    pub fn preservation() -> Self {
        Self {
            enable: true,
            mu: 0.0,
            sigma: 0.0,
        }
    }

    pub fn neutral_mu(mu: f64) -> Self {
        Self {
            enable: true,
            mu,
            sigma: 0.0,
        }
    }
}

/// Derive μ = clamp(2/B_C, 1e-5, 1e-2) from median catalyst-production equivalents/generation.
pub fn derive_mutation_rate(b_c: f64) -> f64 {
    let raw = 2.0 / b_c.max(EPS);
    raw.clamp(1e-5, 1e-2)
}

#[inline]
pub fn composition_z(c_h: f64, c_b: f64) -> f64 {
    let t = c_h.max(0.0) + c_b.max(0.0) + EPS;
    ((c_h.max(0.0) - c_b.max(0.0)) / t).clamp(-1.0, 1.0)
}

#[inline]
pub fn p_h(c_h: f64, c_b: f64) -> f64 {
    c_h.max(0.0) / (c_h.max(0.0) + c_b.max(0.0) + EPS)
}

#[inline]
pub fn p_b(c_h: f64, c_b: f64) -> f64 {
    c_b.max(0.0) / (c_h.max(0.0) + c_b.max(0.0) + EPS)
}

#[inline]
pub fn g_harvest(z: f64, sigma: f64) -> f64 {
    (1.0 + sigma * z).clamp(0.85, 1.15)
}

#[inline]
pub fn g_build(z: f64, sigma: f64) -> f64 {
    (1.0 - sigma * z).clamp(0.85, 1.15)
}

/// Split total catalyst production flux J_C into J_CH, J_CB with mutation μ.
/// Requires some parent catalyst (no catalyst ⇒ no copying); empty pool yields zeros.
#[inline]
pub fn copy_production_fluxes(j_c: f64, c_h: f64, c_b: f64, mu: f64) -> (f64, f64) {
    let total = c_h.max(0.0) + c_b.max(0.0);
    if total <= EPS || j_c <= 0.0 {
        return (0.0, 0.0);
    }
    let mu = mu.clamp(0.0, 1.0);
    let ph = p_h(c_h, c_b);
    let pb = p_b(c_h, c_b);
    let j_h = j_c * ((1.0 - mu) * ph + mu * pb);
    let j_b = j_c * ((1.0 - mu) * pb + mu * ph);
    (j_h, j_b)
}

/// Set C_H / C_B from total C and composition z ∈ [-1,1].
pub fn set_composition_from_z(chem: &mut LumpedChem, z: f64) {
    let c = chem.c.max(0.0);
    let z = z.clamp(-1.0, 1.0);
    chem.c_h = c * (1.0 + z) * 0.5;
    chem.c_b = c * (1.0 - z) * 0.5;
    sync_total_c(chem);
}

pub fn sync_total_c(chem: &mut LumpedChem) {
    chem.c = (chem.c_h.max(0.0) + chem.c_b.max(0.0)).max(0.0);
}

pub fn sync_mesh_total_c(mesh: &mut MaterialMesh) {
    sync_total_c(&mut mesh.interior);
}

/// Bootstrap composition fields from scalar c when enabling composition mode.
pub fn ensure_composition_initialized(chem: &mut LumpedChem) {
    let parts = chem.c_h.max(0.0) + chem.c_b.max(0.0);
    if parts <= EPS && chem.c > EPS {
        // Default balanced when enabling without prior composition.
        chem.c_h = chem.c * 0.5;
        chem.c_b = chem.c * 0.5;
    }
    sync_total_c(chem);
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompositionLedger {
    pub c_h_produced: f64,
    pub c_b_produced: f64,
    pub c_h_turned: f64,
    pub c_b_turned: f64,
    /// Observer: catalytic conversion mass (μ-driven alternate-type production).
    pub conversion_events: f64,
}

/// Apply proportional turnover to both catalyst types; conserve type fractions on decay.
pub fn turnover_composition(c_h: f64, c_b: f64, c_turn: f64) -> (f64, f64, f64, f64) {
    let total = c_h.max(0.0) + c_b.max(0.0);
    if total <= EPS || c_turn <= 0.0 {
        return (c_h.max(0.0), c_b.max(0.0), 0.0, 0.0);
    }
    let frac = (c_turn / total).clamp(0.0, 1.0);
    let t_h = c_h.max(0.0) * frac;
    let t_b = c_b.max(0.0) * frac;
    (
        (c_h.max(0.0) - t_h).max(0.0),
        (c_b.max(0.0) - t_b).max(0.0),
        t_h,
        t_b,
    )
}
