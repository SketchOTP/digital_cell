//! Nullcline and fixed-point classification (D-005 reduced-order diagnostic).

use crate::basin::MacrostateVelocity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixedPointClass {
    Stable,
    Unstable,
    SaddleLike,
    Indeterminate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullclineIntersection {
    pub radius: f64,
    pub mean_c_inside: f64,
    pub classification: FixedPointClass,
    pub jacobian: [[f64; 2]; 2],
    pub max_real_eigenvalue: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGridPoint {
    pub r0: f64,
    pub c0: f64,
    pub velocity: MacrostateVelocity,
}

/// Find grid cells where v_R and v_C change sign (nullcline crossings).
pub fn find_nullcline_intersections(points: &[FlowGridPoint]) -> Vec<NullclineIntersection> {
    if points.len() < 4 {
        return vec![];
    }
    let mut out = Vec::new();
    let rs: Vec<f64> = points.iter().map(|p| p.r0).collect();
    let cs: Vec<f64> = points.iter().map(|p| p.c0).collect();
    let min_r = rs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_r = rs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_c = cs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_c = cs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let r_unique: Vec<f64> = {
        let mut v: Vec<f64> = rs.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        v
    };
    let c_unique: Vec<f64> = {
        let mut v: Vec<f64> = cs.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        v
    };

    for ir in 0..r_unique.len().saturating_sub(1) {
        for ic in 0..c_unique.len().saturating_sub(1) {
            let r0 = r_unique[ir];
            let r1 = r_unique[ir + 1];
            let c0 = c_unique[ic];
            let c1 = c_unique[ic + 1];
            let corners: [(f64, f64); 4] = [(r0, c0), (r1, c0), (r0, c1), (r1, c1)];
            let mut v_rs = [0.0; 4];
            let mut v_cs = [0.0; 4];
            let mut found = 0usize;
            for (k, (r, c)) in corners.iter().enumerate() {
                if let Some(p) = nearest_point(points, *r, *c) {
                    v_rs[k] = p.velocity.v_r;
                    v_cs[k] = p.velocity.v_c;
                    found += 1;
                }
            }
            if found < 4 {
                continue;
            }
            let vr_sign_change = sign_changes(&v_rs);
            let vc_sign_change = sign_changes(&v_cs);
            if vr_sign_change > 0 && vc_sign_change > 0 {
                let r_mid = (r0 + r1) / 2.0;
                let c_mid = (c0 + c1) / 2.0;
                let j = estimate_jacobian(points, r_mid, c_mid);
                let (class, max_ev) = classify_jacobian(&j);
                out.push(NullclineIntersection {
                    radius: r_mid,
                    mean_c_inside: c_mid,
                    classification: class,
                    jacobian: j,
                    max_real_eigenvalue: max_ev,
                });
            }
        }
    }
    out
}

fn nearest_point(points: &[FlowGridPoint], r: f64, c: f64) -> Option<&FlowGridPoint> {
    points
        .iter()
        .min_by(|a, b| {
            let da = (a.r0 - r).powi(2) + (a.c0 - c).powi(2);
            let db = (b.r0 - r).powi(2) + (b.c0 - c).powi(2);
            da.partial_cmp(&db).unwrap()
        })
}

fn sign_changes(vals: &[f64]) -> u32 {
    let mut changes = 0u32;
    for w in vals.windows(2) {
        if w[0].signum() != w[1].signum() && w[0].abs() > 1e-12 && w[1].abs() > 1e-12 {
            changes += 1;
        }
    }
    changes
}

pub fn estimate_jacobian(points: &[FlowGridPoint], r: f64, c: f64) -> [[f64; 2]; 2] {
    let eps_r = 2.0;
    let eps_c = 0.05;
    let v_rr = velocity_at(points, r + eps_r, c);
    let v_rl = velocity_at(points, r - eps_r, c);
    let v_rc = velocity_at(points, r, c + eps_c);
    let v_cl = velocity_at(points, r, c - eps_c);
    let dv_r_dr = (v_rr.v_r - v_rl.v_r) / (2.0 * eps_r);
    let dv_r_dc = (v_rc.v_r - v_cl.v_r) / (2.0 * eps_c);
    let dv_c_dr = (v_rr.v_c - v_rl.v_c) / (2.0 * eps_r);
    let dv_c_dc = (v_rc.v_c - v_cl.v_c) / (2.0 * eps_c);
    [[dv_r_dr, dv_r_dc], [dv_c_dr, dv_c_dc]]
}

fn velocity_at(points: &[FlowGridPoint], r: f64, c: f64) -> MacrostateVelocity {
    nearest_point(points, r, c)
        .map(|p| p.velocity.clone())
        .unwrap_or(MacrostateVelocity {
            radius: r,
            mean_c_inside: c,
            v_r: 0.0,
            v_c: 0.0,
        })
}

pub fn classify_jacobian(j: &[[f64; 2]; 2]) -> (FixedPointClass, f64) {
    let a = j[0][0];
    let b = j[0][1];
    let c = j[1][0];
    let d = j[1][1];
    let trace = a + d;
    let det = a * d - b * c;
    let disc = trace * trace - 4.0 * det;
    let max_real = if disc >= 0.0 {
        let s = disc.sqrt();
        ((trace + s) / 2.0).max((trace - s) / 2.0)
    } else {
        trace / 2.0
    };
    let class = if det < 0.0 {
        FixedPointClass::SaddleLike
    } else if max_real < 0.0 {
        FixedPointClass::Stable
    } else if max_real > 0.0 {
        FixedPointClass::Unstable
    } else {
        FixedPointClass::Indeterminate
    };
    (class, max_real)
}

/// Synthetic test helpers
pub fn synthetic_stable_jacobian() -> [[f64; 2]; 2] {
    [[-0.1, 0.0], [0.0, -0.2]]
}

pub fn synthetic_unstable_jacobian() -> [[f64; 2]; 2] {
    [[0.1, 0.0], [0.0, 0.2]]
}

pub fn synthetic_saddle_jacobian() -> [[f64; 2]; 2] {
    [[0.1, 0.0], [0.0, -0.1]]
}
