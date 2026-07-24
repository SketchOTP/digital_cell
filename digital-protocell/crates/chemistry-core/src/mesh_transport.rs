//! D-086 conservative transport between mesh boundary and Cartesian-style reservoirs.
//!
//! Permeability depends on bound-membrane occupancy θ = b / b_max.
//! Targets (D-079/D-083 lineage): C,A ≤0.05; N,F ∈[0.20,0.50]; W ≥0.70.

use crate::material_mesh::MaterialMesh;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransportParams {
    pub k_flux: f64,
}

impl Default for TransportParams {
    fn default() -> Self {
        // Sized so perimeter influx can fund interior N+F→A against build demand
        // (Gate 3 failure mode was activation-starved A collapse).
        Self { k_flux: 1.1 }
    }
}

/// Occupancy → permeability for species class.
pub fn permeability(theta: f64, species: &str) -> f64 {
    let t = theta.clamp(0.0, 1.0);
    // Higher binding → lower C/A leak, moderate N/F, high W.
    match species {
        "C" | "A" => 0.05 * (1.0 - 0.9 * t),
        "N" | "F" => 0.20 + 0.30 * (1.0 - t),
        "W" => 0.70 + 0.25 * t,
        _ => 0.1,
    }
}

pub fn permeability_in_targets(theta: f64) -> bool {
    let pc = permeability(theta, "C");
    let pa = permeability(theta, "A");
    let pn = permeability(theta, "N");
    let pf = permeability(theta, "F");
    let pw = permeability(theta, "W");
    pc <= 0.05 + 1e-9
        && pa <= 0.05 + 1e-9
        && (0.20 - 1e-9..=0.50 + 1e-9).contains(&pn)
        && (0.20 - 1e-9..=0.50 + 1e-9).contains(&pf)
        && pw + 1e-9 >= 0.70
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportLedger {
    pub n_in: f64,
    pub f_in: f64,
    pub w_out: f64,
    pub c_leak: f64,
    pub a_leak: f64,
}

/// Mean occupancy across intact edges.
pub fn mean_occupancy(mesh: &MaterialMesh) -> f64 {
    let mut s = 0.0;
    let mut n = 0.0;
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        s += mesh.occupancy(i);
        n += 1.0;
    }
    if n <= 0.0 {
        0.0
    } else {
        s / n
    }
}

/// Conservative exchange: interior ⇄ exterior across perimeter, using mean θ.
/// Ruptured edges increase leak (higher effective permeability).
pub fn transport_step(
    mesh: &mut MaterialMesh,
    p: &TransportParams,
    dt: f64,
) -> TransportLedger {
    let mut led = TransportLedger::default();
    if !mesh.alive {
        return led;
    }
    let area = mesh.area().max(1e-6);
    let peri = mesh.perimeter().max(1e-6);
    let theta = mean_occupancy(mesh);
    let ruptured_frac = mesh.edges.iter().filter(|e| e.ruptured).count() as f64
        / mesh.n().max(1) as f64;
    let leak_boost = 1.0 + 4.0 * ruptured_frac;

    let flux = |species: &str, c_in: f64, c_out: f64| -> f64 {
        let perm = permeability(theta, species) * leak_boost;
        p.k_flux * perm * (c_out - c_in) * peri * dt / area
    };

    // N, F enter from exterior reservoir.
    let dn = flux("N", mesh.interior.n, mesh.exterior.n);
    let df = flux("F", mesh.interior.f, mesh.exterior.f);
    mesh.interior.n = (mesh.interior.n + dn).max(0.0);
    mesh.interior.f = (mesh.interior.f + df).max(0.0);
    if dn > 0.0 {
        led.n_in += dn * area;
    }
    if df > 0.0 {
        led.f_in += df * area;
    }

    // W exits.
    let dw = flux("W", mesh.interior.w, mesh.exterior.w);
    mesh.interior.w = (mesh.interior.w + dw).max(0.0);
    if dw < 0.0 {
        led.w_out += (-dw) * area;
    }

    // C, A leak (should be small when bound membrane high).
    let c_before = mesh.interior.c.max(0.0);
    let dc = flux("C", mesh.interior.c, mesh.exterior.c);
    let da = flux("A", mesh.interior.a, mesh.exterior.a);
    if dc < 0.0 && c_before > 1e-15 {
        let frac = ((-dc) / c_before).clamp(0.0, 1.0);
        mesh.interior.tracer_c = (mesh.interior.tracer_c * (1.0 - frac)).max(0.0);
    }
    mesh.interior.c = (mesh.interior.c + dc).max(0.0);
    mesh.interior.a = (mesh.interior.a + da).max(0.0);
    if dc < 0.0 {
        led.c_leak += (-dc) * area;
    }
    if da < 0.0 {
        led.a_leak += (-da) * area;
    }

    led
}

/// Approximate retention over a window: final/initial for C and A.
pub fn retention(c0: f64, c1: f64) -> f64 {
    if c0 <= 1e-15 {
        1.0
    } else {
        (c1 / c0).clamp(0.0, 10.0)
    }
}
