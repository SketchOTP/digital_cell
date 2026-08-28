//! D-086 overdamped local mesh mechanics (stretch, bend, local pressure).
//!
//! No target radius, target area, or global shape energy.

use crate::material_mesh::{
    conserve_interior_amount_across_area_change, MaterialMesh, MeshContractVersion, MeshEdge,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MechParams {
    pub gamma: f64,
    pub k_s: f64,
    pub kappa_b: f64,
    /// Pressure coefficient converting local chem contrast → normal force density.
    pub k_pi: f64,
    pub dt: f64,
}

/// Bounded post-Phase-1 force-hook contract for local external geometry.
/// World adapters must normalize their contact force into this bound.
pub const MAX_EXTERNAL_FORCE_PER_VERTEX: f64 = 0.5;

impl Default for MechParams {
    fn default() -> Self {
        // Center candidate: see `mechanical_candidates` Laplace derivation.
        Self {
            gamma: 1.0,
            k_s: 14.0,
            kappa_b: 2.0,
            k_pi: 0.22,
            dt: 0.02,
        }
    }
}

/// Characteristic Gate-2 chem contrast used only to size global candidates.
/// Π_chem ≈ (C+A+0.5(N+F))_in − (…)_out with the passive Gate-2 fill.
pub const PI_CHEM_CHAR: f64 = 1.4;
/// Largest passive seed radius; basin needs α = k_pi·Π_chem / k_s < 1/R_max.
pub const R_MAX_SEED: f64 = 18.0;

/// Three globally defined mechanical candidates (weak / center / strong).
///
/// Dimensionless design (2D Laplace for fixed rest perimeter L0≈2πR0):
/// `1/R_eq = 1/R0 − α` with `α = k_pi·Π_chem / k_s`. Require `α < 1/R_max`
/// so every lawful seed size has a finite pressurized equilibrium; vary α
/// and stiffness within that band (no radius-/seed-specific knobs).
pub fn mechanical_candidates() -> [MechParams; 3] {
    // α targets: ~0.028 / 0.022 / 0.0175  (all < 1/18 ≈ 0.0556)
    let mk = |k_s: f64, alpha: f64, kappa_b: f64| -> MechParams {
        let k_pi = alpha * k_s / PI_CHEM_CHAR;
        MechParams {
            gamma: 1.0,
            k_s,
            kappa_b,
            k_pi,
            dt: 0.02,
        }
    };
    [
        mk(14.0, 0.022, 2.0),  // center — prefer moderate swell for metabolic basin
        mk(20.0, 0.0175, 2.8), // strong
        mk(10.0, 0.028, 1.4),  // weak
    ]
}

fn edge_unit(mesh: &MaterialMesh, i: usize) -> ([f64; 2], f64) {
    let n = mesh.n();
    let a = mesh.vertices[i];
    let b = mesh.vertices[(i + 1) % n];
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let len = (dx * dx + dy * dy).sqrt().max(1e-15);
    ([dx / len, dy / len], len)
}

fn outward_normal(mesh: &MaterialMesh, i: usize) -> [f64; 2] {
    // Edge tangent CCW → outward for positive area is rotate tangent by -90° in screen coords
    // if CCW (signed area > 0): outward = (ty, -tx) for inward?
    // For CCW polygon, inward normal = rotate tangent +90°: (-ty, tx); outward = (ty, -tx).
    let (t, _) = edge_unit(mesh, i);
    let sign = if mesh.signed_area() >= 0.0 { 1.0 } else { -1.0 };
    [sign * t[1], -sign * t[0]]
}

/// Local osmotic-like pressure from chemistry immediately inside vs outside the edge.
/// Uses lumped interior vs exterior only (no total area / radius / whole-organism mass).
pub fn local_pressure(mesh: &MaterialMesh, _i: usize) -> f64 {
    let inside = mesh.interior.c + mesh.interior.a + 0.5 * (mesh.interior.n + mesh.interior.f);
    let outside = mesh.exterior.c + mesh.exterior.a + 0.5 * (mesh.exterior.n + mesh.exterior.f);
    inside - outside
}

fn angle_cos(mesh: &MaterialMesh, i: usize) -> f64 {
    let n = mesh.n();
    let prev = (i + n - 1) % n;
    let (t0, _) = edge_unit(mesh, prev);
    let (t1, _) = edge_unit(mesh, i);
    (t0[0] * t1[0] + t0[1] * t1[1]).clamp(-1.0, 1.0)
}

/// Accumulate vertex forces from stretch, bend, and local pressure.
pub fn compute_forces(mesh: &MaterialMesh, params: &MechParams) -> Vec<[f64; 2]> {
    compute_forces_with_reference_lengths(mesh, params, None)
}

/// Accumulate forces using an optional diagnostic per-edge reference length.
///
/// The production path passes `None` and therefore remains byte/semantically
/// unchanged.  The reference slice is observer-only state owned by a
/// diagnostic clone; it has no material or serialized-organism authority.
pub fn compute_forces_with_reference_lengths(
    mesh: &MaterialMesh,
    params: &MechParams,
    reference_lengths: Option<&[f64]>,
) -> Vec<[f64; 2]> {
    if let Some(reference_lengths) = reference_lengths {
        if reference_lengths.len() != mesh.n()
            || reference_lengths
                .iter()
                .any(|length| !length.is_finite() || *length <= 0.0)
        {
            return Vec::new();
        }
    }
    let n = mesh.n();
    let mut f = vec![[0.0, 0.0]; n];
    for i in 0..n {
        if mesh.edges[i].ruptured {
            continue;
        }
        let (t, len) = edge_unit(mesh, i);
        let l0 = reference_lengths
            .and_then(|lengths| lengths.get(i).copied())
            .unwrap_or_else(|| mesh.rest_length(i));
        // Stretch: dE/dℓ = k_s (ℓ-ℓ0)/ℓ0 ; clamp reference length so mass-damaged
        // edges cannot produce unbounded restoring forces that hang remesh.
        let l_ref = l0.max(0.25 * len).max(1e-3);
        let fs = params.k_s * (len - l0) / l_ref;
        let fs = fs.clamp(-params.k_s * 8.0, params.k_s * 8.0);
        let j = (i + 1) % n;
        f[i][0] += fs * t[0];
        f[i][1] += fs * t[1];
        f[j][0] -= fs * t[0];
        f[j][1] -= fs * t[1];

        // Pressure: Π * ℓ * n_hat / 2 on each endpoint (local chem contrast only).
        let pi = local_pressure(mesh, i);
        let nh = outward_normal(mesh, i);
        let fp = params.k_pi * pi * len * 0.5;
        f[i][0] += fp * nh[0];
        f[i][1] += fp * nh[1];
        f[j][0] += fp * nh[0];
        f[j][1] += fp * nh[1];
    }
    // Bending: κ_b (1 - cos θ) → discrete hinge torque proxy on three vertices.
    for i in 0..n {
        if mesh.edges[i].ruptured || mesh.edges[(i + n - 1) % n].ruptured {
            continue;
        }
        let c = angle_cos(mesh, i);
        let s = (1.0 - c * c).sqrt().max(1e-15);
        // Force magnitude ~ κ_b * sinθ toward reducing curvature deviation from straight (θ=0).
        let mag = params.kappa_b * s;
        let prev = (i + n - 1) % n;
        let next = (i + 1) % n;
        let p0 = mesh.vertices[prev];
        let p1 = mesh.vertices[i];
        let p2 = mesh.vertices[next];
        // Bisector of exterior angle → push vertex toward flatter configuration.
        let v0 = [p0[0] - p1[0], p0[1] - p1[1]];
        let v2 = [p2[0] - p1[0], p2[1] - p1[1]];
        let n0 = (v0[0] * v0[0] + v0[1] * v0[1]).sqrt().max(1e-15);
        let n2 = (v2[0] * v2[0] + v2[1] * v2[1]).sqrt().max(1e-15);
        let u0 = [v0[0] / n0, v0[1] / n0];
        let u2 = [v2[0] / n2, v2[1] / n2];
        let bis = [u0[0] + u2[0], u0[1] + u2[1]];
        let bn = (bis[0] * bis[0] + bis[1] * bis[1]).sqrt().max(1e-15);
        f[i][0] += mag * bis[0] / bn;
        f[i][1] += mag * bis[1] / bn;
    }
    f
}

/// Overdamped step: γ dx/dt = F. Conserves edge material (m,b) — only vertex positions move.
pub fn mechanics_step(mesh: &mut MaterialMesh, params: &MechParams) -> bool {
    if !mesh.can_advance_physics() || mesh.n() < 3 {
        return false;
    }
    let forces = compute_forces(mesh, params);
    mechanics_step_from_forces(mesh, params, forces)
}

/// Diagnostic-only mechanics step using caller-owned per-edge reference
/// lengths.  The normal production function remains the authority whenever
/// this API is not explicitly selected by a diagnostic clone.
pub fn mechanics_step_with_reference_lengths(
    mesh: &mut MaterialMesh,
    params: &MechParams,
    reference_lengths: &[f64],
) -> bool {
    if !mesh.can_advance_physics()
        || mesh.n() < 3
        || reference_lengths.len() != mesh.n()
        || reference_lengths
            .iter()
            .any(|length| !length.is_finite() || *length <= 0.0)
    {
        return false;
    }
    let forces = compute_forces_with_reference_lengths(mesh, params, Some(reference_lengths));
    mechanics_step_from_forces(mesh, params, forces)
}

fn mechanics_step_from_forces(
    mesh: &mut MaterialMesh,
    params: &MechParams,
    forces: Vec<[f64; 2]>,
) -> bool {
    if forces.len() != mesh.n() {
        return false;
    }
    let inv_g = 1.0 / params.gamma.max(1e-15);
    let dt = params.dt;
    let m_before = mesh.total_structural_mass();
    let b_before = mesh.total_bound_membrane();
    let l_before = mesh.free_l;
    let area_before = matches!(
        mesh.contract_version,
        MeshContractVersion::GeometryConservativeV3 | MeshContractVersion::MaturationCoupledV4
    )
    .then(|| mesh.area());
    for (i, fi) in forces.iter().enumerate() {
        mesh.vertices[i][0] += dt * inv_g * fi[0];
        mesh.vertices[i][1] += dt * inv_g * fi[1];
    }
    let geometry_ok = area_before.map_or(true, |before| {
        conserve_interior_amount_across_area_change(mesh, before, mesh.area())
    });
    let ok = geometry_ok
        && (mesh.total_structural_mass() - m_before).abs() < 1e-12
        && (mesh.total_bound_membrane() - b_before).abs() < 1e-12
        && (mesh.free_l - l_before).abs() < 1e-12;
    ok
}

/// Overdamped mechanics step with bounded local forces supplied by an
/// external physical geometry.  The caller supplies forces only; this
/// function retains authority over vertex movement and material-conservation
/// checks.  A zero force vector follows the same force/integration path as
/// [`mechanics_step`].
pub fn mechanics_step_with_external_forces(
    mesh: &mut MaterialMesh,
    params: &MechParams,
    external_forces: &[[f64; 2]],
) -> bool {
    if !mesh.can_advance_physics() || mesh.n() < 3 || external_forces.len() != mesh.n() {
        return false;
    }
    let mut forces = compute_forces(mesh, params);
    for (force, external) in forces.iter_mut().zip(external_forces) {
        let magnitude = external[0].hypot(external[1]);
        if !external[0].is_finite()
            || !external[1].is_finite()
            || !magnitude.is_finite()
            || magnitude > MAX_EXTERNAL_FORCE_PER_VERTEX
        {
            return false;
        }
        force[0] += external[0];
        force[1] += external[1];
    }
    mechanics_step_from_forces(mesh, params, forces)
}

/// Overdamped mechanics step with additional bounded tension on existing
/// edges. The caller supplies tension only; this function retains authority
/// over vertex movement and material-conservation checks.
pub fn mechanics_step_with_edge_tensions(
    mesh: &mut MaterialMesh,
    params: &MechParams,
    edge_tensions: &[f64],
) -> bool {
    if !mesh.can_advance_physics() || mesh.n() < 3 || edge_tensions.len() != mesh.n() {
        return false;
    }
    let mut forces = compute_forces(mesh, params);
    for (i, tension) in edge_tensions.iter().copied().enumerate() {
        if mesh.edges[i].ruptured || !tension.is_finite() || tension < 0.0 {
            return false;
        }
        if tension == 0.0 {
            continue;
        }
        let n = mesh.n();
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % n];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let length = dx.hypot(dy).max(1e-15);
        let tx = dx / length;
        let ty = dy / length;
        // Positive tension pulls the two existing endpoints together.
        forces[i][0] += tension * tx;
        forces[i][1] += tension * ty;
        let j = (i + 1) % n;
        forces[j][0] -= tension * tx;
        forces[j][1] -= tension * ty;
    }

    mechanics_step_from_forces(mesh, params, forces)
}

/// Combined bounded local edge tension and external physical force step.
/// Existing mechanics remains the sole authority for movement.
pub fn mechanics_step_with_edge_tensions_and_external_forces(
    mesh: &mut MaterialMesh,
    params: &MechParams,
    edge_tensions: &[f64],
    external_forces: &[[f64; 2]],
) -> bool {
    if !mesh.can_advance_physics()
        || mesh.n() < 3
        || edge_tensions.len() != mesh.n()
        || external_forces.len() != mesh.n()
    {
        return false;
    }
    let mut forces = compute_forces(mesh, params);
    for (i, tension) in edge_tensions.iter().copied().enumerate() {
        if mesh.edges[i].ruptured || !tension.is_finite() || tension < 0.0 {
            return false;
        }
        if tension == 0.0 {
            continue;
        }
        let n = mesh.n();
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % n];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let length = dx.hypot(dy).max(1e-15);
        let tx = dx / length;
        let ty = dy / length;
        forces[i][0] += tension * tx;
        forces[i][1] += tension * ty;
        let j = (i + 1) % n;
        forces[j][0] -= tension * tx;
        forces[j][1] -= tension * ty;
    }
    for (force, external) in forces.iter_mut().zip(external_forces) {
        let magnitude = external[0].hypot(external[1]);
        if !external[0].is_finite()
            || !external[1].is_finite()
            || !magnitude.is_finite()
            || magnitude > MAX_EXTERNAL_FORCE_PER_VERTEX
        {
            return false;
        }
        force[0] += external[0];
        force[1] += external[1];
    }
    mechanics_step_from_forces(mesh, params, forces)
}

/// Conservative split when length exceeds l_max.
pub fn remesh_split(mesh: &mut MaterialMesh) -> usize {
    let mut splits = 0usize;
    let mut i = 0usize;
    let max_splits = mesh.n().saturating_mul(4).max(16);
    while i < mesh.n() && splits < max_splits {
        let len = mesh.edge_length(i);
        if !mesh.edges[i].ruptured && len > mesh.l_max {
            let n = mesh.n();
            let a = mesh.vertices[i];
            let b = mesh.vertices[(i + 1) % n];
            let mid = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
            let e = mesh.edges[i];
            let m_half = 0.5 * e.m;
            let b_half = 0.5 * e.b;
            let tm_half = 0.5 * e.tracer_m;
            let tb_half = 0.5 * e.tracer_b;
            let my_half = 0.5 * e.m_young;
            mesh.vertices.insert(i + 1, mid);
            mesh.edges[i] = MeshEdge {
                m: m_half,
                b: b_half,
                tracer_m: tm_half,
                tracer_b: tb_half,
                m_young: my_half,
                ruptured: false,
            };
            mesh.edges.insert(
                i + 1,
                MeshEdge {
                    m: m_half,
                    b: b_half,
                    tracer_m: tm_half,
                    tracer_b: tb_half,
                    m_young: my_half,
                    ruptured: false,
                },
            );
            splits += 1;
            i += 2;
            continue;
        }
        i += 1;
    }
    splits
}

/// Conservative merge of short adjacent edges.
pub fn remesh_merge(mesh: &mut MaterialMesh) -> usize {
    let mut merges = 0usize;
    let max_merges = mesh.n().saturating_mul(2).max(8);
    while merges < max_merges {
        if mesh.n() <= 6 {
            break;
        }
        let n = mesh.n();
        let mut pick: Option<usize> = None;
        for i in 0..n {
            if mesh.edges[i].ruptured {
                continue;
            }
            if mesh.edge_length(i) < mesh.l_min {
                let j = (i + 1) % n;
                if !mesh.edges[j].ruptured {
                    pick = Some(i);
                    break;
                }
            }
        }
        let Some(i) = pick else {
            break;
        };
        let n = mesh.n();
        let j = (i + 1) % n;
        let e0 = mesh.edges[i];
        let e1 = mesh.edges[j];
        let new_e = MeshEdge {
            m: e0.m + e1.m,
            b: e0.b + e1.b,
            tracer_m: e0.tracer_m + e1.tracer_m,
            tracer_b: e0.tracer_b + e1.tracer_b,
            m_young: e0.m_young + e1.m_young,
            ruptured: false,
        };
        // Remove vertex j and edge j; keep combined material on edge i.
        // Handle wrap by rotating so i is never the last index when j==0.
        if j == 0 {
            // i == n-1: drop vertex 0, drop edge 0, rewrite edge i-1? After remove(0),
            // former i becomes i-1. Simpler: rotate arrays so merge is interior.
            mesh.vertices.rotate_left(1);
            mesh.edges.rotate_left(1);
            // Now former edge n-1 is at n-2, former edge 0 at end — re-pick last edge.
            let i2 = mesh.n() - 2;
            let e_a = mesh.edges[i2];
            let e_b = mesh.edges[i2 + 1];
            mesh.edges[i2] = MeshEdge {
                m: e_a.m + e_b.m,
                b: e_a.b + e_b.b,
                tracer_m: e_a.tracer_m + e_b.tracer_m,
                tracer_b: e_a.tracer_b + e_b.tracer_b,
                m_young: e_a.m_young + e_b.m_young,
                ruptured: false,
            };
            mesh.edges.pop();
            mesh.vertices.pop();
        } else {
            mesh.edges[i] = new_e;
            mesh.edges.remove(j);
            mesh.vertices.remove(j);
        }
        debug_assert_eq!(mesh.edges.len(), mesh.vertices.len());
        merges += 1;
    }
    merges
}

pub fn remesh(mesh: &mut MaterialMesh) -> (usize, usize) {
    let m0 = mesh.total_structural_mass();
    let b0 = mesh.total_bound_membrane();
    let area_before = matches!(
        mesh.contract_version,
        MeshContractVersion::GeometryConservativeV3 | MeshContractVersion::MaturationCoupledV4
    )
    .then(|| mesh.area());
    let s = remesh_split(mesh);
    let m = remesh_merge(mesh);
    if let Some(before) = area_before {
        let _ = conserve_interior_amount_across_area_change(mesh, before, mesh.area());
    }
    let _ = (
        (mesh.total_structural_mass() - m0).abs() < 1e-9,
        (mesh.total_bound_membrane() - b0).abs() < 1e-9,
    );
    (s, m)
}
