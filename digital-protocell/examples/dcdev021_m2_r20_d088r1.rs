//! R20: opt-in hard nonpenetration requalification of the D-088 campaign.
//!
//! This example is deliberately isolated from production.  It reuses the
//! frozen D-088 force, growth, topology, pinch, fission, and partition laws,
//! but advances the existing mechanics proposal only up to the first
//! nonadjacent segment contact detected by continuous orientation tests.

use chemistry_core::material_mesh::{conserve_interior_amount_across_area_change, MaterialMesh};
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{compute_forces, mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_topology::{find_local_pinch, local_rebond_range};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

const STEPS: usize = 4_000;
const EPS: f64 = f64::EPSILON;

#[derive(Debug, Clone, Serialize)]
struct ContactLedgerRow {
    step: usize,
    candidate_contact_pairs: usize,
    continuous_collision_predicted: bool,
    contact_pairs_resolved: usize,
    proposed_displacement_norm: f64,
    accepted_displacement_norm: f64,
    contact_fraction: f64,
    normal_motion_removed: f64,
    tangential_motion_retained: bool,
    minimum_nonadjacent_segment_distance: f64,
    polygon_simple_before: bool,
    polygon_simple_after: bool,
}

#[derive(Debug, Clone)]
struct ContactStep {
    fraction: f64,
    candidate_pairs: usize,
    resolved_pairs: usize,
    min_distance: f64,
    proposed_displacement_norm: f64,
    accepted_displacement_norm: f64,
}

fn scale(points: &[[f64; 2]]) -> f64 {
    points
        .iter()
        .flat_map(|p| p)
        .map(|v| v.abs())
        .fold(1.0, f64::max)
}

fn orient(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn geom_tol(points: &[[f64; 2]]) -> f64 {
    128.0 * EPS * (1.0 + scale(points)).powi(2)
}

fn on_segment(a: [f64; 2], b: [f64; 2], p: [f64; 2], tol: f64) -> bool {
    orient(a, b, p).abs() <= tol
        && p[0] >= a[0].min(b[0]) - tol
        && p[0] <= a[0].max(b[0]) + tol
        && p[1] >= a[1].min(b[1]) - tol
        && p[1] <= a[1].max(b[1]) + tol
}

fn segment_intersects(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2], tol: f64) -> bool {
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

fn point_segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let den = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if den > 0.0 {
        (ap[0] * ab[0] + ap[1] * ab[1]) / den
    } else {
        0.0
    }
    .clamp(0.0, 1.0);
    let q = [a[0] + t * ab[0], a[1] + t * ab[1]];
    (p[0] - q[0]).hypot(p[1] - q[1])
}

fn segment_distance(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> f64 {
    if segment_intersects(a, b, c, d, 0.0) {
        0.0
    } else {
        point_segment_distance(a, c, d)
            .min(point_segment_distance(b, c, d))
            .min(point_segment_distance(c, a, b))
            .min(point_segment_distance(d, a, b))
    }
}

fn simple(points: &[[f64; 2]]) -> (bool, usize, f64) {
    let n = points.len();
    let tol = geom_tol(points);
    let mut count = 0;
    let mut min_distance = f64::INFINITY;
    for i in 0..n {
        for j in (i + 1)..n {
            if j == i + 1 || (i == 0 && j + 1 == n) {
                continue;
            }
            let a = points[i];
            let b = points[(i + 1) % n];
            let c = points[j];
            let d = points[(j + 1) % n];
            min_distance = min_distance.min(segment_distance(a, b, c, d));
            if segment_intersects(a, b, c, d, tol) {
                count += 1;
            }
        }
    }
    (count == 0, count, min_distance)
}

fn geometry(points: &[[f64; 2]]) -> Value {
    let n = points.len();
    let mut signed = 0.0;
    let mut perimeter = 0.0;
    for i in 0..n {
        let q = points[(i + 1) % n];
        signed += points[i][0] * q[1] - q[0] * points[i][1];
        perimeter += (q[0] - points[i][0]).hypot(q[1] - points[i][1]);
    }
    let signed = 0.5 * signed;
    let area = signed.abs();
    let shape_factor = perimeter * perimeter / (4.0 * std::f64::consts::PI * area.max(1e-300));
    let (is_simple, intersections, min_distance) = simple(points);
    json!({
        "perimeter": perimeter,
        "signed_area": signed,
        "absolute_area": area,
        "shape_factor": shape_factor,
        "isoperimetric_quotient": 1.0 / shape_factor,
        "polygon_simple": is_simple,
        "intersection_count": intersections,
        "minimum_nonadjacent_segment_distance": min_distance
    })
}

fn interpolate(old: &[[f64; 2]], new: &[[f64; 2]], t: f64) -> Vec<[f64; 2]> {
    old.iter()
        .zip(new)
        .map(|(a, b)| [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])])
        .collect()
}

/// Return the largest numerically simple fraction on a proposed linear
/// motion.  The CCD root calculation supplies the first contact estimate;
/// this bounded fallback makes the accepted step fail closed if floating
/// point error places that estimate infinitesimally beyond contact.
fn largest_simple_fraction(old: &[[f64; 2]], proposed: &[[f64; 2]], upper: f64) -> f64 {
    if simple(&interpolate(old, proposed, upper)).0 {
        return upper;
    }
    let mut lo = 0.0;
    let mut hi = upper;
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if simple(&interpolate(old, proposed, mid)).0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo.max(0.0)
}

fn aabb_overlap(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2], tol: f64) -> bool {
    let amin = [a[0].min(b[0]), a[1].min(b[1])];
    let amax = [a[0].max(b[0]), a[1].max(b[1])];
    let cmin = [c[0].min(d[0]), c[1].min(d[1])];
    let cmax = [c[0].max(d[0]), c[1].max(d[1])];
    amin[0] <= cmax[0] + tol
        && cmin[0] <= amax[0] + tol
        && amin[1] <= cmax[1] + tol
        && cmin[1] <= amax[1] + tol
}

fn orient_polynomial(
    a0: [f64; 2],
    a1: [f64; 2],
    b0: [f64; 2],
    b1: [f64; 2],
    c0: [f64; 2],
    c1: [f64; 2],
) -> [f64; 3] {
    let f = |t: f64| {
        orient(
            [a0[0] + t * (a1[0] - a0[0]), a0[1] + t * (a1[1] - a0[1])],
            [b0[0] + t * (b1[0] - b0[0]), b0[1] + t * (b1[1] - b0[1])],
            [c0[0] + t * (c1[0] - c0[0]), c0[1] + t * (c1[1] - c0[1])],
        )
    };
    let f0 = f(0.0);
    let fhalf = f(0.5);
    let f1 = f(1.0);
    let c2 = 2.0 * (f0 + f1 - 2.0 * fhalf);
    [f0, f1 - f0 - c2, c2]
}

fn roots_unit(poly: [f64; 3]) -> Vec<f64> {
    let [a, b, c] = poly;
    let scale = 1.0 + a.abs() + b.abs() + c.abs();
    let tol = 256.0 * EPS * scale;
    let mut out = Vec::new();
    if a.abs() <= tol {
        if b.abs() > tol {
            let r = -c / b;
            if r >= -tol && r <= 1.0 + tol {
                out.push(r.clamp(0.0, 1.0));
            }
        }
        return out;
    }
    let disc = b * b - 4.0 * a * c;
    if disc < -tol {
        return out;
    }
    let root = disc.max(0.0).sqrt();
    for r in [(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)] {
        if r >= -tol && r <= 1.0 + tol {
            out.push(r.clamp(0.0, 1.0));
        }
    }
    out
}

fn unique_sorted(mut values: Vec<f64>) -> Vec<f64> {
    values.sort_by(|a, b| a.total_cmp(b));
    values.dedup_by(|a, b| (*a - *b).abs() <= 512.0 * EPS);
    values
}

fn earliest_pair_contact(
    old: &[[f64; 2]],
    proposed: &[[f64; 2]],
    i: usize,
    j: usize,
) -> Option<f64> {
    let n = old.len();
    let a0 = old[i];
    let a1 = proposed[i];
    let b0 = old[(i + 1) % n];
    let b1 = proposed[(i + 1) % n];
    let c0 = old[j];
    let c1 = proposed[j];
    let d0 = old[(j + 1) % n];
    let d1 = proposed[(j + 1) % n];
    let tol = geom_tol(old);
    if segment_intersects(a0, b0, c0, d0, tol) {
        return Some(0.0);
    }
    let swept_tol = tol + 512.0 * EPS * (1.0 + scale(proposed));
    if !aabb_overlap(a0, a1, c0, c1, swept_tol) || !aabb_overlap(b0, b1, d0, d1, swept_tol) {
        // The endpoint-paired boxes are only a cheap rejection.  The full
        // swept boxes are checked below because endpoints can exchange order.
        let amin = [a0[0].min(a1[0]), a0[1].min(a1[1])];
        let amax = [a0[0].max(a1[0]), a0[1].max(a1[1])];
        let bmin = [b0[0].min(b1[0]), b0[1].min(b1[1])];
        let bmax = [b0[0].max(b1[0]), b0[1].max(b1[1])];
        let cmin = [c0[0].min(c1[0]), c0[1].min(c1[1])];
        let cmax = [c0[0].max(c1[0]), c0[1].max(c1[1])];
        let dmin = [d0[0].min(d1[0]), d0[1].min(d1[1])];
        let dmax = [d0[0].max(d1[0]), d0[1].max(d1[1])];
        if amax[0] < cmin[0] - swept_tol && bmax[0] < dmin[0] - swept_tol
            || cmax[0] < amin[0] - swept_tol && dmax[0] < bmin[0] - swept_tol
            || amax[1] < cmin[1] - swept_tol && bmax[1] < dmin[1] - swept_tol
            || cmax[1] < amin[1] - swept_tol && dmax[1] < bmin[1] - swept_tol
        {
            return None;
        }
    }
    let mut roots = vec![0.0, 1.0];
    for p in [
        orient_polynomial(a0, a1, b0, b1, c0, c1),
        orient_polynomial(a0, a1, b0, b1, d0, d1),
        orient_polynomial(c0, c1, d0, d1, a0, a1),
        orient_polynomial(c0, c1, d0, d1, b0, b1),
    ] {
        roots.extend(roots_unit(p));
    }
    let roots = unique_sorted(roots);
    let intersects_at = |t: f64| {
        let p = interpolate(old, proposed, t);
        segment_intersects(p[i], p[(i + 1) % n], p[j], p[(j + 1) % n], geom_tol(&p))
    };
    for r in roots.iter().copied() {
        if r > 512.0 * EPS && r < 1.0 - 512.0 * EPS && intersects_at(r) {
            return Some(r);
        }
    }
    for pair in roots.windows(2) {
        let lo = pair[0];
        let hi = pair[1];
        if hi - lo <= 1024.0 * EPS {
            continue;
        }
        if intersects_at((lo + hi) * 0.5) {
            return Some(lo.max(0.0));
        }
    }
    if intersects_at(1.0) {
        Some(1.0)
    } else {
        None
    }
}

fn contact_step(mesh: &mut MaterialMesh, params: &MechParams) -> Option<ContactStep> {
    if !mesh.can_advance_physics() || mesh.n() < 3 {
        return None;
    }
    let old = mesh.vertices.clone();
    let forces = compute_forces(mesh, params);
    if forces.len() != mesh.n() {
        return None;
    }
    let proposed: Vec<[f64; 2]> = old
        .iter()
        .zip(forces.iter())
        .map(|(p, f)| {
            [
                p[0] + params.dt / params.gamma.max(1e-15) * f[0],
                p[1] + params.dt / params.gamma.max(1e-15) * f[1],
            ]
        })
        .collect();
    let (before_simple, _, before_min) = simple(&old);
    if !before_simple {
        return None;
    }
    let n = old.len();
    let mut candidate_pairs = 0;
    let mut earliest: Option<f64> = None;
    for i in 0..n {
        for j in (i + 1)..n {
            if j == i + 1 || (i == 0 && j + 1 == n) {
                continue;
            }
            let a0 = old[i];
            let a1 = proposed[i];
            let b0 = old[(i + 1) % n];
            let b1 = proposed[(i + 1) % n];
            let c0 = old[j];
            let c1 = proposed[j];
            let d0 = old[(j + 1) % n];
            let d1 = proposed[(j + 1) % n];
            let min_x = a0[0]
                .min(a1[0])
                .min(b0[0].min(b1[0]))
                .min(c0[0].min(c1[0]))
                .min(d0[0].min(d1[0]));
            let max_x = a0[0]
                .max(a1[0])
                .max(b0[0].max(b1[0]))
                .max(c0[0].max(c1[0]))
                .max(d0[0].max(d1[0]));
            let min_y = a0[1]
                .min(a1[1])
                .min(b0[1].min(b1[1]))
                .min(c0[1].min(c1[1]))
                .min(d0[1].min(d1[1]));
            let max_y = a0[1].max(a1[1]).max(b0[1].max(b1[1])).max(c0[1].max(c1[1]));
            if max_x < min_x || max_y < min_y {
                continue;
            }
            candidate_pairs += 1;
            if let Some(t) = earliest_pair_contact(&old, &proposed, i, j) {
                earliest = Some(earliest.map_or(t, |e| e.min(t)));
            }
        }
    }
    let mut fallback_clamp = false;
    let fraction = match earliest {
        Some(t) => {
            let upper = (t - 512.0 * EPS * (1.0 + scale(&proposed))).max(0.0);
            largest_simple_fraction(&old, &proposed, upper)
        }
        None if simple(&proposed).0 => 1.0,
        None => {
            // A final geometric postcondition is required in addition to
            // the analytic CCD roots: if roundoff or a degenerate contact
            // makes the root test inconclusive, fail closed at the largest
            // simple point on the same proposed motion.
            fallback_clamp = true;
            largest_simple_fraction(&old, &proposed, 1.0)
        }
    };
    let accepted = interpolate(&old, &proposed, fraction);
    let proposed_norm = old
        .iter()
        .zip(proposed.iter())
        .map(|(a, b)| (b[0] - a[0]).hypot(b[1] - a[1]))
        .sum();
    let accepted_norm = old
        .iter()
        .zip(accepted.iter())
        .map(|(a, b)| (b[0] - a[0]).hypot(b[1] - a[1]))
        .sum();
    let area_before = matches!(
        mesh.contract_version,
        chemistry_core::material_mesh::MeshContractVersion::GeometryConservativeV3
            | chemistry_core::material_mesh::MeshContractVersion::MaturationCoupledV4
    )
    .then(|| mesh.area());
    for (dst, src) in mesh.vertices.iter_mut().zip(accepted.iter()) {
        *dst = *src;
    }
    let area_ok = area_before.map_or(true, |before| {
        conserve_interior_amount_across_area_change(mesh, before, mesh.area())
    });
    if !area_ok {
        return None;
    }
    Some(ContactStep {
        fraction,
        candidate_pairs,
        resolved_pairs: usize::from(earliest.is_some() || fallback_clamp),
        min_distance: before_min,
        proposed_displacement_norm: proposed_norm,
        accepted_displacement_norm: accepted_norm,
    })
}

fn metrics(mesh: &MaterialMesh) -> Value {
    geometry(&mesh.vertices)
}

fn perturb(mesh: &mut MaterialMesh, kind: &str, mag: f64) {
    match kind {
        "rotate" => {
            let c = mesh.centroid();
            let (s, co) = mag.sin_cos();
            for p in &mut mesh.vertices {
                let (x, y) = (p[0] - c[0], p[1] - c[1]);
                p[0] = c[0] + co * x - s * y;
                p[1] = c[1] + s * x + co * y;
            }
        }
        "vertex" => {
            for (i, p) in mesh.vertices.iter_mut().enumerate() {
                let f = (((i as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
                p[0] += mag * (f - 0.5);
                p[1] += mag * ((f * 7.13).fract() - 0.5);
            }
        }
        "c" => mesh.interior.c = (mesh.interior.c * (1.0 + mag)).max(0.0),
        "a" => mesh.interior.a = (mesh.interior.a * (1.0 + mag)).max(0.0),
        "l" => mesh.free_l = (mesh.free_l * (1.0 + mag)).max(0.0),
        "env" => {
            mesh.exterior.n = (mesh.exterior.n * (1.0 + mag)).max(0.0);
            mesh.exterior.f = (mesh.exterior.f * (1.0 + mag)).max(0.0);
        }
        _ => {}
    }
}

fn fixture(seed: u64, kind: &str, mag: f64) -> MaterialMesh {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .remove(0)
        .mesh;
    perturb(&mut mesh, kind, mag);
    perturb(&mut mesh, "vertex", 0.35);
    let c = mesh.centroid();
    for p in &mut mesh.vertices {
        p[0] = c[0] + (p[0] - c[0]) * 1.25;
        p[1] = c[1] + (p[1] - c[1]) * 1.25;
    }
    mesh
}

fn fission_readiness(mesh: &MaterialMesh, birth_mass: f64, fission: &FissionParams) -> Value {
    let n = mesh.n();
    let area = mesh.area().abs();
    let perimeter = mesh.perimeter();
    let shape_factor = perimeter * perimeter / (4.0 * std::f64::consts::PI * area.max(1e-300));
    let strains: Vec<f64> = (0..n).map(|i| mesh.strain(i)).collect();
    let max_strain = strains.iter().copied().fold(0.0, f64::max);
    let mean_strain = if n == 0 {
        0.0
    } else {
        strains.iter().sum::<f64>() / n as f64
    };
    let signed_area = {
        let mut sum = 0.0;
        for i in 0..n {
            let q = mesh.vertices[(i + 1) % n];
            sum += mesh.vertices[i][0] * q[1] - q[0] * mesh.vertices[i][1];
        }
        0.5 * sum
    };
    let orientation = signed_area.signum();
    let concave_vertices = if n < 3 || orientation == 0.0 {
        0
    } else {
        (0..n)
            .filter(|&i| {
                let prev = mesh.vertices[(i + n - 1) % n];
                let curr = mesh.vertices[i];
                let next = mesh.vertices[(i + 1) % n];
                orientation * orient(prev, curr, next) < -geom_tol(&mesh.vertices)
            })
            .count()
    };
    let mass = mesh.total_structural_mass();
    let mass_gate = mass >= 1.35 * birth_mass;
    let range = local_rebond_range(mesh, &fission.topo);
    let pinch = find_local_pinch(mesh, &fission.topo);
    let (pinch_distance, pinch_stress, pinch_proximity, cross_bond_need, a_available) =
        if let Some((i, j)) = pinch {
            let a = mesh.vertices[i];
            let b = mesh.vertices[j];
            let distance = (b[0] - a[0]).hypot(b[1] - a[1]);
            let stress = mesh.strain(i).max(mesh.strain((i + n - 1) % n)) > 0.15
                || mesh.strain(j).max(mesh.strain((j + n - 1) % n)) > 0.15
                || mesh.edges[i].ruptured
                || mesh.edges[(j + n - 1) % n].ruptured
                || distance < range * 0.55;
            (
                Some(distance),
                stress,
                distance <= range,
                Some(mesh.rho_s * distance),
                mesh.interior.a.max(0.0) * area,
            )
        } else {
            (None, false, false, None, mesh.interior.a.max(0.0) * area)
        };
    let a_sufficient = cross_bond_need.map_or(false, |need| {
        let required = if mesh.uses_observer_only_death() {
            need
        } else {
            need * 0.25
        };
        a_available >= required
    });
    let shadow_try = if mass_gate {
        try_local_fission(&mesh.clone(), fission).is_some()
    } else {
        false
    };
    let reason = if !mass_gate {
        "MASS_NOT_ELIGIBLE"
    } else if n < fission.min_vertices {
        "VERTEX_REQUIREMENT"
    } else if pinch.is_none() {
        "NO_PINCH"
    } else if !pinch_proximity {
        "PINCH_OUT_OF_RANGE"
    } else if !pinch_stress {
        "PINCH_NOT_STRESSED"
    } else if !a_sufficient {
        "CROSS_BOND_A_INSUFFICIENT"
    } else if shadow_try {
        "FISSION_READY"
    } else {
        "UNRESOLVED"
    };
    json!({
        "vertex_count": n,
        "can_advance_physics": mesh.can_advance_physics(),
        "area": area,
        "perimeter": perimeter,
        "shape_factor": shape_factor,
        "max_edge_strain": max_strain,
        "mean_edge_strain": mean_strain,
        "ruptured_edge_count": mesh.edges.iter().filter(|e| e.ruptured).count(),
        "concave_vertex_count": concave_vertices,
        "local_rebond_range": range,
        "best_nonadjacent_distance": geometry(&mesh.vertices)["minimum_nonadjacent_segment_distance"],
        "mass": mass,
        "birth_mass": birth_mass,
        "mass_over_birth_mass": mass / birth_mass.max(1e-300),
        "mass_gate_reached": mass_gate,
        "pinch_candidate_exists": pinch.is_some(),
        "pinch_i": pinch.map(|(i, _)| i),
        "pinch_j": pinch.map(|(_, j)| j),
        "pinch_distance": pinch_distance,
        "pinch_stress_condition": pinch_stress,
        "pinch_proximity_condition": pinch_proximity,
        "absolute_a_mass": a_available,
        "cross_bond_mass_needed": cross_bond_need,
        "a_over_cross_bond_need": cross_bond_need.map(|need| a_available / need.max(1e-300)),
        "cross_bond_a_sufficient": a_sufficient,
        "shadow_try_local_fission": shadow_try,
        "reason_not_ready": reason
    })
}

fn step(
    mesh: &mut MaterialMesh,
    index: usize,
    mech: &MechParams,
    react: &ReactionParams,
    transport: &TransportParams,
    growth: &GrowthParams,
    fission: &FissionParams,
    nonpenetration: bool,
) -> (Value, Option<ContactStep>) {
    let _ = transport_step(mesh, transport, mech.dt);
    let _ = reactions_step(mesh, react, mech.dt, true, true);
    let _ = growth_step(mesh, react, growth, mech.dt);
    let pre_mechanics = metrics(mesh);
    let contact = if nonpenetration {
        contact_step(mesh, mech)
    } else {
        let before = mesh.vertices.clone();
        let _ = mechanics_step(mesh, mech);
        let proposed = mesh.vertices.clone();
        let displacement = before
            .iter()
            .zip(proposed.iter())
            .map(|(a, b)| (b[0] - a[0]).hypot(b[1] - a[1]))
            .sum::<f64>();
        Some(ContactStep {
            fraction: 1.0,
            candidate_pairs: 0,
            resolved_pairs: 0,
            min_distance: geometry(&before)
                .get("minimum_nonadjacent_segment_distance")
                .and_then(Value::as_f64)
                .unwrap_or(0.0),
            proposed_displacement_norm: displacement,
            accepted_displacement_norm: displacement,
        })
    };
    let post_mechanics = metrics(mesh);
    let _ = remesh(mesh);
    let post_remesh = metrics(mesh);
    let topology = if index % 10 == 0 {
        topology_step(mesh, fission)
    } else {
        chemistry_core::mesh_topology::TopologyLedger::default()
    };
    let post_topology = metrics(mesh);
    (
        json!({
            "step": index + 1,
            "pre_mechanics": pre_mechanics,
            "post_mechanics": post_mechanics,
            "post_remesh": post_remesh,
            "post_topology": post_topology,
            "topology": topology,
            "contact": contact.as_ref().map(|c| json!({
                "fraction": c.fraction,
                "candidate_pairs": c.candidate_pairs,
                "resolved_pairs": c.resolved_pairs,
                "minimum_nonadjacent_segment_distance": c.min_distance
            }))
        }),
        contact,
    )
}

fn run_campaign(mesh: MaterialMesh, name: &str, nonpenetration: bool) -> Value {
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let birth_mass = mesh.total_structural_mass();
    let mut mesh = mesh;
    let mut rows = Vec::new();
    let mut contacts = Vec::new();
    let mut event = None;
    for s in 0..STEPS {
        if !mesh.can_advance_physics() {
            break;
        }
        let (mut row, contact) = step(
            &mut mesh,
            s,
            &mech,
            &react,
            &transport,
            &growth,
            &fission,
            nonpenetration,
        );
        if let Some(c) = contact {
            contacts.push(ContactLedgerRow {
                step: s + 1,
                candidate_contact_pairs: c.candidate_pairs,
                continuous_collision_predicted: c.resolved_pairs > 0,
                contact_pairs_resolved: c.resolved_pairs,
                proposed_displacement_norm: c.proposed_displacement_norm,
                accepted_displacement_norm: c.accepted_displacement_norm,
                contact_fraction: c.fraction,
                normal_motion_removed: 1.0 - c.fraction,
                // The assay has no friction model; when contact truncates the
                // proposed step it does not claim post-contact tangential
                // sliding.  A full step is unchanged motion.
                tangential_motion_retained: c.fraction >= 1.0,
                minimum_nonadjacent_segment_distance: c.min_distance,
                polygon_simple_before: row["pre_mechanics"]["polygon_simple"]
                    .as_bool()
                    .unwrap_or(false),
                polygon_simple_after: row["post_mechanics"]["polygon_simple"]
                    .as_bool()
                    .unwrap_or(false),
            });
        }
        let mass = mesh.total_structural_mass();
        let attempt = mass >= 1.35 * birth_mass && s % 25 == 0;
        row["fission_readiness"] = fission_readiness(&mesh, birth_mass, &fission);
        row["fission_attempt_tick"] = Value::Bool(attempt);
        if attempt {
            if let Some((d1, d2, ev)) = try_local_fission(&mesh, &fission) {
                event = Some(json!({
                    "step": s + 1,
                    "pinch": ev.pinch,
                    "parent_geometry": metrics(&mesh),
                    "daughter_a_geometry": metrics(&d1),
                    "daughter_b_geometry": metrics(&d2),
                    "partition": ev.partition
                }));
                break;
            }
        }
        rows.push(row);
    }
    json!({
        "campaign": name,
        "nonpenetration": nonpenetration,
        "birth_mass": birth_mass,
        "physical_fission": event.is_some(),
        "event": event,
        "final_mass": mesh.total_structural_mass(),
        "rows": rows,
        "contact_ledger": contacts,
        "final_geometry": metrics(&mesh)
    })
}

fn synthetic_tests() -> Value {
    let convex = vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
    let concave = vec![[0.0, 0.0], [3.0, 0.0], [3.0, 3.0], [1.5, 1.0], [0.0, 3.0]];
    let bow = vec![[0.0, 0.0], [2.0, 2.0], [0.0, 2.0], [2.0, 0.0]];
    let near = vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [0.01, 4.0],
        [0.01, 0.01],
        [0.0, 0.01],
    ];
    json!({
        "convex": geometry(&convex),
        "concave": geometry(&concave),
        "bow_tie": geometry(&bow),
        "near_contact": geometry(&near),
        "expected": {"convex": true, "concave": true, "bow_tie": false, "near_contact": true}
    })
}

fn no_contact_motion_test() -> Value {
    let mut mesh = MaterialMesh::seed_regular(
        12,
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
    let forces = compute_forces(&mesh, &params);
    let original: Vec<[f64; 2]> = mesh
        .vertices
        .iter()
        .zip(forces.iter())
        .map(|(p, f)| [p[0] + params.dt * f[0], p[1] + params.dt * f[1]])
        .collect();
    let _ = contact_step(&mut mesh, &params);
    let max_error = mesh
        .vertices
        .iter()
        .zip(original.iter())
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0, f64::max);
    json!({"max_coordinate_error": max_error, "pass": max_error <= 1e-12, "simple": simple(&mesh.vertices).0})
}

fn crossing_tests() -> Value {
    let cases = [
        (
            "direct_edge_crossing",
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            vec![[0.0, 0.0], [2.0, 2.0], [0.0, 2.0], [2.0, 0.0]],
        ),
        (
            "vertex_through_edge",
            vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
            vec![[0.0, 0.0], [4.0, 4.0], [0.0, 4.0], [4.0, 0.0]],
        ),
        (
            "near_miss",
            vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
            vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
        ),
    ];
    let rows: Vec<Value> = cases
        .iter()
        .map(|(name, old, new)| {
            let mut mesh = MaterialMesh::seed_regular(
                old.len(),
                1.0,
                0.0,
                0.0,
                1.0,
                0.5,
                Default::default(),
                Default::default(),
                0.0,
            );
            mesh.vertices = old.clone();
            let mut proposed = mesh.clone();
            proposed.vertices = new.clone();
            let contact = earliest_pair_contact(old, new, 0, 2);
            let raw_simple = simple(&proposed.vertices).0;
            let safe_fraction = contact
                .map(|t| (t - 512.0 * EPS * (1.0 + scale(new))).max(0.0))
                .unwrap_or(1.0);
            let safe_fraction = largest_simple_fraction(old, new, safe_fraction);
            let corrected = interpolate(old, new, safe_fraction);
            json!({
                "name":name,
                "ccd_contact":contact,
                "raw_final_simple":raw_simple,
                "accepted_fraction":safe_fraction,
                "corrected_final_simple":simple(&corrected).0,
                "continuous_penetration":!simple(&corrected).0
            })
        })
        .collect();
    json!(rows)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out = PathBuf::from("/tmp/dcdev021_m2_r20_d088r1.json");
    for i in 1..args.len() {
        if args[i] == "--output" && i + 1 < args.len() {
            out = PathBuf::from(&args[i + 1]);
        }
    }
    let kinds = [
        ("rotate", 0.3),
        ("vertex", 0.12),
        ("c", 0.08),
        ("a", 0.08),
        ("env", 0.1),
        ("l", 0.1),
        ("rotate", -0.5),
        ("vertex", -0.1),
        ("c", -0.05),
        ("env", -0.08),
    ];
    let mut legacy = Vec::new();
    let mut corrected = Vec::new();
    for (i, (kind, magnitude)) in kinds.iter().enumerate() {
        let seed = (i + 1) as u64;
        let mesh = fixture(seed, kind, *magnitude);
        legacy.push(run_campaign(
            mesh.clone(),
            &format!("seed_{seed}_{kind}_{magnitude}"),
            false,
        ));
        corrected.push(run_campaign(
            mesh,
            &format!("seed_{seed}_{kind}_{magnitude}"),
            true,
        ));
    }
    let value = json!({
        "directive": "DC-DEV-021-M2-R20-D088R1-SIMPLE-BOUNDARY-NONPENETRATION-AND-PHYSICAL-REPRODUCTION-REQUALIFICATION-001",
        "observer_only": true,
        "contact_architecture": {
            "model": "frictionless hard nonpenetration of nonadjacent membrane segments",
            "method": "continuous orientation-polynomial collision detection plus conservative advancement",
            "new_free_physical_parameters": 0,
            "production_default_changed": false
        },
        "synthetic_tests": synthetic_tests(),
        "no_contact_parity": no_contact_motion_test(),
        "continuous_collision_tests": crossing_tests(),
        "legacy": legacy,
        "d088r1": corrected,
        "qualification_population": 10
    });
    fs::write(out, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
