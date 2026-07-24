//! D-089 catalyst inheritance observers (partition is physical; IDs are not causal).

use crate::catalyst_composition::composition_z;
use crate::material_mesh::MaterialMesh;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentOffspringPair {
    pub z_parent: f64,
    pub z_daughter: f64,
    pub c_h_parent: f64,
    pub c_b_parent: f64,
    pub c_h_daughter: f64,
    pub c_b_daughter: f64,
    pub area_frac: f64,
}

/// Pearson correlation of paired samples.
pub fn pearson(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len().min(ys.len()) as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut dx = 0.0;
    let mut dy = 0.0;
    for i in 0..xs.len().min(ys.len()) {
        let a = xs[i] - mx;
        let b = ys[i] - my;
        num += a * b;
        dx += a * a;
        dy += b * b;
    }
    if dx <= 1e-18 || dy <= 1e-18 {
        return 0.0;
    }
    num / (dx.sqrt() * dy.sqrt())
}

/// Ordinary least-squares slope of y ~ a + b x.
pub fn ols_slope(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len().min(ys.len()) as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..xs.len().min(ys.len()) {
        num += (xs[i] - mx) * (ys[i] - my);
        den += (xs[i] - mx) * (xs[i] - mx);
    }
    if den <= 1e-18 {
        0.0
    } else {
        num / den
    }
}

pub fn mesh_z(mesh: &MaterialMesh) -> f64 {
    composition_z(mesh.interior.c_h, mesh.interior.c_b)
}

pub fn catalyst_masses(mesh: &MaterialMesh) -> (f64, f64, f64) {
    let a = mesh.area().max(1e-9);
    let ch = mesh.interior.c_h.max(0.0) * a;
    let cb = mesh.interior.c_b.max(0.0) * a;
    (ch, cb, ch + cb)
}
