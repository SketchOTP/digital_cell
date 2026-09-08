//! Geometry-only, frictionless self-contact for a closed material ring.
//!
//! The correction projects only vertices belonging to an imminent pair of
//! nonadjacent segments.  It has no access to chemistry, growth, lineage, or
//! reproductive state.  Unrelated vertices retain their complete proposed
//! displacement and tangential motion is preserved whenever the normal
//! projection alone is sufficient.

use crate::material_mesh::{
    conserve_interior_amount_across_area_change, MaterialMesh, MeshContractVersion,
};
use crate::mesh_mechanics::{compute_forces, MechParams};
use serde::{Deserialize, Serialize};

const EPS: f64 = f64::EPSILON;
const PROJECTION_PASSES: usize = 24;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelfContactLedger {
    pub candidate_pairs: usize,
    pub active_pairs: usize,
    pub projected_vertices: usize,
    pub fallback_pairs: usize,
    pub proposed_displacement_norm: f64,
    pub accepted_displacement_norm: f64,
    pub tangential_displacement_retained: f64,
    pub simple_before: bool,
    pub simple_after: bool,
}

fn orient(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn scale(points: &[[f64; 2]]) -> f64 {
    points
        .iter()
        .flat_map(|p| p)
        .map(|v| v.abs())
        .fold(1.0, f64::max)
}

fn tolerance(points: &[[f64; 2]]) -> f64 {
    256.0 * EPS * (1.0 + scale(points)).powi(2)
}

fn on_segment(a: [f64; 2], b: [f64; 2], p: [f64; 2], tol: f64) -> bool {
    orient(a, b, p).abs() <= tol
        && p[0] >= a[0].min(b[0]) - tol
        && p[0] <= a[0].max(b[0]) + tol
        && p[1] >= a[1].min(b[1]) - tol
        && p[1] <= a[1].max(b[1]) + tol
}

pub fn segments_intersect(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2], tol: f64) -> bool {
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);
    (((o1 > tol && o2 < -tol) || (o1 < -tol && o2 > tol))
        && ((o3 > tol && o4 < -tol) || (o3 < -tol && o4 > tol)))
        || on_segment(a, b, c, tol)
        || on_segment(a, b, d, tol)
        || on_segment(c, d, a, tol)
        || on_segment(c, d, b, tol)
}

pub fn polygon_simple(points: &[[f64; 2]]) -> bool {
    let n = points.len();
    if n < 3 {
        return false;
    }
    let tol = tolerance(points);
    for i in 0..n {
        for j in (i + 1)..n {
            if j == i + 1 || (i == 0 && j + 1 == n) {
                continue;
            }
            if segments_intersect(
                points[i],
                points[(i + 1) % n],
                points[j],
                points[(j + 1) % n],
                tol,
            ) {
                return false;
            }
        }
    }
    true
}

fn motion_state(old: &[[f64; 2]], disp: &[[f64; 2]], t: f64) -> Vec<[f64; 2]> {
    old.iter()
        .zip(disp)
        .map(|(p, d)| [p[0] + t * d[0], p[1] + t * d[1]])
        .collect()
}

fn closest_points(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    d: [f64; 2],
) -> (f64, f64, [f64; 2], [f64; 2]) {
    let u = [b[0] - a[0], b[1] - a[1]];
    let v = [d[0] - c[0], d[1] - c[1]];
    let w = [a[0] - c[0], a[1] - c[1]];
    let aa = u[0] * u[0] + u[1] * u[1];
    let bb = u[0] * v[0] + u[1] * v[1];
    let cc = v[0] * v[0] + v[1] * v[1];
    let dd = u[0] * w[0] + u[1] * w[1];
    let ee = v[0] * w[0] + v[1] * w[1];
    let den = aa * cc - bb * bb;
    let mut s = if den.abs() > EPS * (1.0 + aa * cc) {
        ((bb * ee - cc * dd) / den).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = if cc > 0.0 { (bb * s + ee) / cc } else { 0.0 };
    if t < 0.0 {
        t = 0.0;
        s = if aa > 0.0 {
            (-dd / aa).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    if t > 1.0 {
        t = 1.0;
        s = if aa > 0.0 {
            ((bb - dd) / aa).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    let p = [a[0] + s * u[0], a[1] + s * u[1]];
    let q = [c[0] + t * v[0], c[1] + t * v[1]];
    (s, t, p, q)
}

fn contact_frame(old: &[[f64; 2]], i: usize, j: usize) -> ([f64; 2], f64, f64) {
    let n = old.len();
    let (s, t, p, q) = closest_points(old[i], old[(i + 1) % n], old[j], old[(j + 1) % n]);
    let mut v = [p[0] - q[0], p[1] - q[1]];
    let mut l = v[0].hypot(v[1]);
    if l <= tolerance(old).sqrt() {
        let e = [
            old[(i + 1) % n][0] - old[i][0],
            old[(i + 1) % n][1] - old[i][1],
        ];
        v = [-e[1], e[0]];
        l = v[0].hypot(v[1]);
    }
    ([v[0] / l.max(1e-300), v[1] / l.max(1e-300)], s, t)
}

fn displacement_norm(d: &[[f64; 2]]) -> f64 {
    d.iter().map(|v| v[0].hypot(v[1])).sum()
}

fn active_state(old: &[[f64; 2]], disp: &[[f64; 2]], active: &[bool], alpha: f64) -> Vec<[f64; 2]> {
    old.iter()
        .zip(disp.iter())
        .enumerate()
        .map(|(k, (p, d))| {
            let a = if active[k] { alpha } else { 1.0 };
            [p[0] + a * d[0], p[1] + a * d[1]]
        })
        .collect()
}

/// Apply one frozen mechanics proposal with local frictionless self-contact.
pub fn mechanics_step_with_local_self_contact(
    mesh: &mut MaterialMesh,
    params: &MechParams,
) -> Option<SelfContactLedger> {
    if !mesh.can_advance_physics() || mesh.n() < 3 || !polygon_simple(&mesh.vertices) {
        return None;
    }
    let old = mesh.vertices.clone();
    let forces = compute_forces(mesh, params);
    if forces.len() != old.len() {
        return None;
    }
    let factor = params.dt / params.gamma.max(1e-15);
    let mut disp: Vec<[f64; 2]> = forces
        .iter()
        .map(|f| [factor * f[0], factor * f[1]])
        .collect();
    let proposed_norm = displacement_norm(&disp);
    let n = old.len();
    let mut led = SelfContactLedger {
        simple_before: true,
        proposed_displacement_norm: proposed_norm,
        ..Default::default()
    };
    let mut touched = vec![false; n];

    // Ordinary mechanics is the overwhelmingly common path.  It must remain
    // exactly unchanged and should not pay the active-contact solve cost.
    let proposed = motion_state(&old, &disp, 1.0);
    if polygon_simple(&proposed) {
        let area_before = matches!(
            mesh.contract_version,
            MeshContractVersion::GeometryConservativeV3 | MeshContractVersion::MaturationCoupledV4
        )
        .then(|| mesh.area());
        mesh.vertices = proposed;
        if let Some(before) = area_before {
            if !conserve_interior_amount_across_area_change(mesh, before, mesh.area()) {
                return None;
            }
        }
        led.accepted_displacement_norm = proposed_norm;
        led.simple_after = true;
        return Some(led);
    }

    for _ in 0..PROJECTION_PASSES {
        let candidate = motion_state(&old, &disp, 1.0);
        if polygon_simple(&candidate) {
            break;
        }
        let tol = tolerance(&candidate);
        let mut changed = false;
        for i in 0..n {
            for j in (i + 1)..n {
                if j == i + 1 || (i == 0 && j + 1 == n) {
                    continue;
                }
                led.candidate_pairs += 1;
                if !segments_intersect(
                    candidate[i],
                    candidate[(i + 1) % n],
                    candidate[j],
                    candidate[(j + 1) % n],
                    tol,
                ) {
                    continue;
                }
                let (nn, si, sj) = contact_frame(&old, i, j);
                let ia = [i, (i + 1) % n];
                let ja = [j, (j + 1) % n];
                let wi = [1.0 - si, si];
                let wj = [1.0 - sj, sj];
                let vi = [
                    wi[0] * disp[ia[0]][0] + wi[1] * disp[ia[1]][0],
                    wi[0] * disp[ia[0]][1] + wi[1] * disp[ia[1]][1],
                ];
                let vj = [
                    wj[0] * disp[ja[0]][0] + wj[1] * disp[ja[1]][0],
                    wj[0] * disp[ja[0]][1] + wj[1] * disp[ja[1]][1],
                ];
                let closing = (vi[0] - vj[0]) * nn[0] + (vi[1] - vj[1]) * nn[1];
                if closing < 0.0 {
                    let denom = wi[0] * wi[0] + wi[1] * wi[1] + wj[0] * wj[0] + wj[1] * wj[1];
                    let lambda = -closing / denom.max(1e-300);
                    for k in 0..2 {
                        disp[ia[k]][0] += lambda * wi[k] * nn[0];
                        disp[ia[k]][1] += lambda * wi[k] * nn[1];
                        touched[ia[k]] = true;
                    }
                    for k in 0..2 {
                        disp[ja[k]][0] -= lambda * wj[k] * nn[0];
                        disp[ja[k]][1] -= lambda * wj[k] * nn[1];
                        touched[ja[k]] = true;
                    }
                    led.tangential_displacement_retained += vi[0] * (-nn[1]) + vi[1] * nn[0];
                    led.active_pairs += 1;
                    changed = true;
                } else {
                    // A deforming pair can still collide with zero centroid closing.
                    // Clip only its four endpoints to immediately before contact.
                    for &k in ia.iter().chain(ja.iter()) {
                        disp[k][0] *= 0.5;
                        disp[k][1] *= 0.5;
                        touched[k] = true;
                    }
                    led.active_pairs += 1;
                    led.fallback_pairs += 1;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }

    // Fail closed locally: build the connected active set of endpoints that
    // participate in the remaining collision and conservatively advance only
    // that set.  Unrelated vertices retain their full displacement.
    let mut candidate = motion_state(&old, &disp, 1.0);
    if !polygon_simple(&candidate) {
        let mut active = vec![false; n];
        let tol = tolerance(&candidate);
        for i in 0..n {
            for j in (i + 1)..n {
                if j == i + 1 || (i == 0 && j + 1 == n) {
                    continue;
                }
                if segments_intersect(
                    candidate[i],
                    candidate[(i + 1) % n],
                    candidate[j],
                    candidate[(j + 1) % n],
                    tol,
                ) {
                    for k in [i, (i + 1) % n, j, (j + 1) % n] {
                        active[k] = true;
                        touched[k] = true;
                    }
                    led.fallback_pairs += 1;
                }
            }
        }
        for _ in 0..n.max(1) {
            if !polygon_simple(&active_state(&old, &disp, &active, 0.0)) {
                let zero = active_state(&old, &disp, &active, 0.0);
                let tol = tolerance(&zero);
                let mut expanded = false;
                for i in 0..n {
                    for j in (i + 1)..n {
                        if j == i + 1 || (i == 0 && j + 1 == n) {
                            continue;
                        }
                        if segments_intersect(
                            zero[i],
                            zero[(i + 1) % n],
                            zero[j],
                            zero[(j + 1) % n],
                            tol,
                        ) {
                            for k in [i, (i + 1) % n, j, (j + 1) % n] {
                                if !active[k] {
                                    active[k] = true;
                                    touched[k] = true;
                                    expanded = true;
                                }
                            }
                        }
                    }
                }
                if expanded {
                    continue;
                }
                // Degenerate simultaneous contact can make the active-set
                // graph numerically closed without yielding a simple zero
                // state.  Expanding to the full connected ring is the final
                // conservative-advancement fallback; it is evidence-visible
                // through projected_vertices and is never the normal path.
                active.fill(true);
                touched.fill(true);
            }
            let mut lo = 0.0;
            let mut hi = 1.0;
            for _ in 0..52 {
                let mid = 0.5 * (lo + hi);
                if polygon_simple(&active_state(&old, &disp, &active, mid)) {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            for k in 0..n {
                if active[k] {
                    disp[k][0] *= lo;
                    disp[k][1] *= lo;
                }
            }
            candidate = motion_state(&old, &disp, 1.0);
            break;
        }
    }
    if !polygon_simple(&candidate) {
        return None;
    }

    let area_before = matches!(
        mesh.contract_version,
        MeshContractVersion::GeometryConservativeV3 | MeshContractVersion::MaturationCoupledV4
    )
    .then(|| mesh.area());
    mesh.vertices = candidate;
    if let Some(before) = area_before {
        if !conserve_interior_amount_across_area_change(mesh, before, mesh.area()) {
            return None;
        }
    }
    led.projected_vertices = touched.into_iter().filter(|v| *v).count();
    led.accepted_displacement_norm = displacement_norm(&disp);
    led.simple_after = true;
    Some(led)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_polygon_observer_distinguishes_crossing() {
        assert!(polygon_simple(&[
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 2.0],
            [0.0, 2.0]
        ]));
        assert!(!polygon_simple(&[
            [0.0, 0.0],
            [2.0, 2.0],
            [0.0, 2.0],
            [2.0, 0.0]
        ]));
    }

    #[test]
    fn no_contact_matches_frozen_proposal() {
        let mut mesh = MaterialMesh::seed_regular(
            16,
            5.0,
            0.0,
            0.0,
            1.0,
            0.5,
            Default::default(),
            Default::default(),
            0.0,
        );
        let params = MechParams::default();
        let old = mesh.vertices.clone();
        let forces = compute_forces(&mesh, &params);
        let expected: Vec<_> = old
            .iter()
            .zip(forces)
            .map(|(p, f)| {
                [
                    p[0] + params.dt * f[0] / params.gamma,
                    p[1] + params.dt * f[1] / params.gamma,
                ]
            })
            .collect();
        let led = mechanics_step_with_local_self_contact(&mut mesh, &params).unwrap();
        assert_eq!(led.active_pairs, 0);
        for (a, b) in mesh.vertices.iter().zip(expected) {
            assert!((a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12);
        }
    }
}
