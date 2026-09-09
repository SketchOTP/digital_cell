//! Topological fission via local pinch rebonding + conservative partition.
//!
//! No `divide()` command. Topology change is local bond events; component
//! discovery is observer bookkeeping only.

use crate::autocatalytic_partition::partition_autocatalytic_edges;
use crate::material_mesh::{MaterialMesh, MeshEdge};
use crate::mesh_topology::{extract_loop, find_local_pinch, TopologyLedger, TopologyParams};
use crate::template_partition::partition_templates;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FissionParams {
    pub topo: TopologyParams,
    /// Minimum parent perimeter before pinch may succeed (observer gate only for reporting;
    /// biology uses local surplus growth; pinch still requires local stress+proximity).
    pub min_vertices: usize,
}

impl Default for FissionParams {
    fn default() -> Self {
        Self {
            topo: TopologyParams::default(),
            min_vertices: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionReport {
    pub residual_m: f64,
    pub residual_b: f64,
    pub residual_l: f64,
    pub residual_c: f64,
    pub residual_a: f64,
    pub residual_n: f64,
    pub residual_f: f64,
    pub residual_w: f64,
    #[serde(default)]
    pub residual_r: f64,
    #[serde(default)]
    pub residual_assimilation_n: f64,
    #[serde(default)]
    pub residual_assimilation_f: f64,
    #[serde(default)]
    pub residual_u_h: f64,
    #[serde(default)]
    pub residual_u_b: f64,
    #[serde(default)]
    pub residual_templates: f64,
    #[serde(default)]
    pub residual_autocatalytic_edges: f64,
    #[serde(default)]
    pub residual_allocation_catalysts: f64,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FissionEvent {
    pub parent_n: usize,
    pub daughter_a_n: usize,
    pub daughter_b_n: usize,
    pub pinch: (usize, usize),
    pub partition: PartitionReport,
    pub leakage_w: f64,
}

/// Observer-only comparison of the legacy segment-apposition stress test with
/// the V4 load-bearing signed-strain contract. This report does not alter
/// candidate ordering or authorize a topology change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentAppositionStressAudit {
    pub edge_i: usize,
    pub edge_j: usize,
    pub segment_i_fraction: f64,
    pub segment_j_fraction: f64,
    pub distance: f64,
    pub range: f64,
    pub ring_separation: usize,
    pub incident_edges: [usize; 4],
    pub raw_strains: [f64; 4],
    pub mature_fractions: [f64; 4],
    pub effective_signed_strains: [f64; 4],
    pub maximum_tensile_effective_strain: f64,
    pub maximum_compressive_effective_strain_magnitude: f64,
    pub legacy_tensile_only_predicate: bool,
    pub signed_magnitude_predicate: bool,
}

const ACCOUNTING_TOL: f64 = 1e-6;

/// Attempt one local pinch fission if a stressed neck exists.
/// Returns two daughters + event, or None if no lawful local topology change.
pub fn try_local_fission(
    parent: &MaterialMesh,
    params: &FissionParams,
) -> Option<(MaterialMesh, MaterialMesh, FissionEvent)> {
    if !parent.can_advance_physics() || parent.n() < params.min_vertices {
        return None;
    }
    if !params.topo.enable_rebond {
        return None;
    }
    let (i, j) = find_local_pinch(parent, &params.topo)?;
    try_local_fission_at_vertices(parent, params, i, j)
}

fn try_local_fission_at_vertices(
    parent: &MaterialMesh,
    params: &FissionParams,
    i: usize,
    j: usize,
) -> Option<(MaterialMesh, MaterialMesh, FissionEvent)> {
    // Cross-bond mass drawn from local A and nearby edge material.
    let a = parent.vertices[i];
    let b = parent.vertices[j];
    let dist = (b[0] - a[0]).hypot(b[1] - a[1]);
    let range = crate::mesh_topology::local_rebond_range(parent, &params.topo);
    if dist > range {
        return None;
    }
    let need = parent.rho_s * dist;
    let v4 = parent.is_maturation_coupled();
    // A fission creates two distinct daughter boundaries. Under V4 each new
    // edge receives the full line-density mass required for its own length;
    // historical contracts retain their exact shared-cross-bond semantics.
    let closure_mass_per_daughter = if v4 { need } else { need * 0.5 };
    let total_closure_mass = 2.0 * closure_mass_per_daughter;
    let v4_yield = crate::mesh_growth::Y_G_CANDIDATES[0];
    let v4_a_need = total_closure_mass / v4_yield.max(1e-15);
    let area = parent.area().max(1e-6);
    let have_a = parent.interior.a.max(0.0) * area;
    let conservative = parent.uses_observer_only_death();
    let required_a = if v4 {
        v4_a_need
    } else if conservative {
        need
    } else {
        need * 0.25
    };
    if have_a < required_a {
        return None;
    }

    // Build two loops: i→j and j→i with shared closing edges (cross bonds).
    let close_ab = MeshEdge {
        m: closure_mass_per_daughter,
        b: 0.0,
        tracer_m: 0.0,
        tracer_b: 0.0,
        m_young: closure_mass_per_daughter,
        ruptured: false,
    };
    let close_ba = MeshEdge {
        m: closure_mass_per_daughter,
        b: 0.0,
        tracer_m: 0.0,
        tracer_b: 0.0,
        m_young: closure_mass_per_daughter,
        ruptured: false,
    };

    let mut d1 = extract_loop(parent, i, j, Some(close_ab));
    let mut d2 = extract_loop(parent, j, i, Some(close_ba));

    // Area fractions for spatial chemistry inheritance.
    let a1 = d1.area().max(1e-9);
    let a2 = d2.area().max(1e-9);
    let at = a1 + a2;
    let f1 = a1 / at;
    let f2 = a2 / at;

    let pre_m = parent.total_structural_mass();
    let pre_b = parent.total_bound_membrane();
    let pre_l = parent.free_l.max(0.0);
    let pre_c = parent.interior.c * parent.area().max(1e-9);
    let pre_c_h = parent.interior.c_h * parent.area().max(1e-9);
    let pre_c_b = parent.interior.c_b * parent.area().max(1e-9);
    let pre_a = parent.interior.a * parent.area().max(1e-9);
    let pre_n = parent.interior.n * parent.area().max(1e-9);
    let pre_f = parent.interior.f * parent.area().max(1e-9);
    let pre_w = parent.interior.w * parent.area().max(1e-9);
    let pre_r = parent.interior.r * parent.area().max(1e-9);
    let pre_assimilation_n = parent.interior.assimilation_n * parent.area().max(1e-9);
    let pre_assimilation_f = parent.interior.assimilation_f * parent.area().max(1e-9);
    let pre_u_h = parent.interior.u_h * parent.area().max(1e-9);
    let pre_u_b = parent.interior.u_b * parent.area().max(1e-9);
    let pre_k_h = parent.interior.k_h * parent.area().max(1e-9);
    let pre_k_b = parent.interior.k_b * parent.area().max(1e-9);
    let pre_q_k = parent.interior.q_k * parent.area().max(1e-9);
    let pre_q_e = parent.interior.q_e * parent.area().max(1e-9);
    let pre_k_a = parent.interior.k_a * parent.area().max(1e-9);
    let pre_k_r = parent.interior.k_r * parent.area().max(1e-9);
    let pre_k_node_b = parent.interior.k_node_b * parent.area().max(1e-9);
    let pre_tmpl = parent.templates.len() as f64;
    let pre_acs_edges = parent.autocatalytic_edges.len() as f64;
    let pre_allocation_catalysts = parent
        .finite_allocation
        .map(|state| state.catalysts.iter().sum::<f64>())
        .unwrap_or(0.0);

    // Free L split by perimeter share.
    let p1 = d1.perimeter().max(1e-9);
    let p2 = d2.perimeter().max(1e-9);
    let pt = p1 + p2;
    d1.free_l = pre_l * (p1 / pt);
    d2.free_l = pre_l * (p2 / pt);

    // Interior concentrations: conserve mass pools by area fraction.
    // C_H / C_B are partitioned as actual material (never from a copied parent ratio).
    let set_conc = |mesh: &mut MaterialMesh, frac: f64| {
        let a = mesh.area().max(1e-9);
        mesh.interior.c = (pre_c * frac) / a;
        mesh.interior.c_h = (pre_c_h * frac) / a;
        mesh.interior.c_b = (pre_c_b * frac) / a;
        // Keep total consistent with parts when composition was active.
        if pre_c_h + pre_c_b > 1e-15 {
            mesh.interior.c = mesh.interior.c_h + mesh.interior.c_b;
        }
        mesh.interior.a = (pre_a * frac) / a;
        mesh.interior.n = (pre_n * frac) / a;
        mesh.interior.f = (pre_f * frac) / a;
        mesh.interior.w = (pre_w * frac) / a;
        // R is partitioned as actual material (never copied as a ratio template).
        mesh.interior.r = (pre_r * frac) / a;
        mesh.interior.assimilation_n = (pre_assimilation_n * frac) / a;
        mesh.interior.assimilation_f = (pre_assimilation_f * frac) / a;
        mesh.interior.u_h = (pre_u_h * frac) / a;
        mesh.interior.u_b = (pre_u_b * frac) / a;
        mesh.interior.k_h = (pre_k_h * frac) / a;
        mesh.interior.k_b = (pre_k_b * frac) / a;
        mesh.interior.q_k = (pre_q_k * frac) / a;
        mesh.interior.q_e = (pre_q_e * frac) / a;
        mesh.interior.k_a = (pre_k_a * frac) / a;
        mesh.interior.k_r = (pre_k_r * frac) / a;
        mesh.interior.k_node_b = (pre_k_node_b * frac) / a;
        mesh.interior.tracer_c = parent.interior.tracer_c * frac;
        mesh.exterior = parent.exterior;
        mesh.alive = true;
        mesh.equation_id = parent.equation_id.clone();
        mesh.schema_version = parent.schema_version;
        mesh.template_rng = parent.template_rng;
        mesh.next_template_id = parent.next_template_id;
        mesh.next_edge_id = parent.next_edge_id;
        mesh.finite_allocation = parent.finite_allocation.map(|mut state| {
            for catalyst in &mut state.catalysts {
                *catalyst *= frac;
            }
            state
        });
        mesh.contract_version = parent.contract_version;
    };
    set_conc(&mut d1, f1);
    set_conc(&mut d2, f2);

    // Physical template partition by spatial location (no sequence copy).
    let (_n1, _n2, residual_templates) = partition_templates(parent, &mut d1, &mut d2);
    // Physical autocatalytic edge partition by position (no whole-network clone).
    let (_e1, _e2, residual_acs) = partition_autocatalytic_edges(parent, &mut d1, &mut d2);

    // Cost of cross-bond: A consumed (leakage/waste).
    // Conservative v2 pays the full cross-bond mass from A. Historical v1
    // retains its legacy half-cost and explicit W leakage for compatibility.
    let take = if v4 {
        v4_a_need
    } else if conservative {
        need
    } else {
        (need * 0.5).min(have_a)
    };
    let v4_w_product = if v4 {
        (take - total_closure_mass).max(0.0)
    } else {
        0.0
    };
    let leakage = if v4 { 0.0 } else { take };
    // Deduct from daughters proportionally (already split); reduce A slightly.
    d1.interior.a = (d1.interior.a - (take * f1) / d1.area().max(1e-9)).max(0.0);
    d2.interior.a = (d2.interior.a - (take * f2) / d2.area().max(1e-9)).max(0.0);
    if v4 {
        d1.interior.w += (v4_w_product * f1) / d1.area().max(1e-9);
        d2.interior.w += (v4_w_product * f2) / d2.area().max(1e-9);
    } else if !parent.uses_observer_only_death() {
        d1.interior.w += (take * f1) / d1.area().max(1e-9);
        d2.interior.w += (take * f2) / d2.area().max(1e-9);
    }

    let post_m = d1.total_structural_mass() + d2.total_structural_mass();
    let post_b = d1.total_bound_membrane() + d2.total_bound_membrane();
    let post_l = d1.free_l + d2.free_l;
    let post_c = d1.interior.c * d1.area() + d2.interior.c * d2.area();
    let post_c_h = d1.interior.c_h * d1.area() + d2.interior.c_h * d2.area();
    let post_c_b = d1.interior.c_b * d1.area() + d2.interior.c_b * d2.area();
    let post_a = d1.interior.a * d1.area() + d2.interior.a * d2.area();
    let post_n = d1.interior.n * d1.area() + d2.interior.n * d2.area();
    let post_f = d1.interior.f * d1.area() + d2.interior.f * d2.area();
    let post_w = d1.interior.w * d1.area() + d2.interior.w * d2.area();
    let post_r = d1.interior.r * d1.area() + d2.interior.r * d2.area();
    let post_assimilation_n =
        d1.interior.assimilation_n * d1.area() + d2.interior.assimilation_n * d2.area();
    let post_assimilation_f =
        d1.interior.assimilation_f * d1.area() + d2.interior.assimilation_f * d2.area();
    let post_u_h = d1.interior.u_h * d1.area() + d2.interior.u_h * d2.area();
    let post_u_b = d1.interior.u_b * d1.area() + d2.interior.u_b * d2.area();
    let post_tmpl = (d1.templates.len() + d2.templates.len()) as f64;
    let post_acs_edges = (d1.autocatalytic_edges.len() + d2.autocatalytic_edges.len()) as f64;
    let post_allocation_catalysts = d1
        .finite_allocation
        .map(|state| state.catalysts.iter().sum::<f64>())
        .unwrap_or(0.0)
        + d2.finite_allocation
            .map(|state| state.catalysts.iter().sum::<f64>())
            .unwrap_or(0.0);

    // Structural: parent m plus the exact contract-specific daughter closure
    // mass. V4 creates two full-density edges; historical contracts preserve
    // the shared single-edge budget.
    let residual_m = (post_m - (pre_m + total_closure_mass)).abs();
    let residual_b = (post_b - pre_b).abs();
    let residual_l = (post_l - pre_l).abs();
    let residual_c = (post_c - pre_c).abs();
    let residual_c_h = (post_c_h - pre_c_h).abs();
    let residual_c_b = (post_c_b - pre_c_b).abs();
    // A and W: A decreases by take, W increases by take
    let residual_a = (post_a - (pre_a - take)).abs();
    let residual_n = (post_n - pre_n).abs();
    let residual_f = (post_f - pre_f).abs();
    let expected_w = if v4 {
        pre_w + v4_w_product
    } else if parent.uses_observer_only_death() {
        pre_w
    } else {
        pre_w + take
    };
    let residual_w = (post_w - expected_w).abs();
    let residual_r = (post_r - pre_r).abs();
    let residual_assimilation_n = (post_assimilation_n - pre_assimilation_n).abs();
    let residual_assimilation_f = (post_assimilation_f - pre_assimilation_f).abs();
    let residual_autocatalytic_edges = (post_acs_edges - pre_acs_edges).abs();
    let residual_allocation_catalysts =
        (post_allocation_catalysts - pre_allocation_catalysts).abs();
    // Paired monomers released into daughter free pools at fission — allow that transfer.
    let residual_u_h = (post_u_h - pre_u_h).abs(); // may increase from paired release
    let residual_u_b = (post_u_b - pre_u_b).abs();

    let ok = residual_m < ACCOUNTING_TOL * (1.0 + pre_m)
        && residual_b < ACCOUNTING_TOL * (1.0 + pre_b)
        && residual_l < ACCOUNTING_TOL * (1.0 + pre_l)
        && residual_c < 1e-4 * (1.0 + pre_c)
        && residual_c_h < 1e-4 * (1.0 + pre_c_h)
        && residual_c_b < 1e-4 * (1.0 + pre_c_b)
        && residual_a < 1e-4 * (1.0 + pre_a)
        && residual_n < 1e-4 * (1.0 + pre_n)
        && residual_f < 1e-4 * (1.0 + pre_f)
        && residual_w < 1e-4 * (1.0 + pre_w)
        && residual_r < 1e-4 * (1.0 + pre_r)
        && residual_assimilation_n < 1e-4 * (1.0 + pre_assimilation_n)
        && residual_assimilation_f < 1e-4 * (1.0 + pre_assimilation_f)
        && residual_templates < 0.5
        && (post_tmpl - pre_tmpl).abs() < 0.5
        && residual_autocatalytic_edges < 0.5
        && residual_allocation_catalysts < 1e-9 * (1.0 + pre_allocation_catalysts)
        && d1.n() >= 3
        && d2.n() >= 3
        && d1.closed_intact()
        && d2.closed_intact();

    let event = FissionEvent {
        parent_n: parent.n(),
        daughter_a_n: d1.n(),
        daughter_b_n: d2.n(),
        pinch: (i, j),
        partition: PartitionReport {
            residual_m,
            residual_b,
            residual_l,
            residual_c,
            residual_a,
            residual_n,
            residual_f,
            residual_w,
            residual_r,
            residual_assimilation_n,
            residual_assimilation_f,
            residual_u_h,
            residual_u_b,
            residual_templates,
            residual_autocatalytic_edges,
            residual_allocation_catalysts,
            ok,
        },
        leakage_w: leakage,
    };

    if !ok {
        // Still return daughters if topology closed; accounting failure flagged.
        // Caller decides defect vs continue.
    }
    Some((d1, d2, event))
}

fn closest_segment_points(
    p0: [f64; 2],
    p1: [f64; 2],
    q0: [f64; 2],
    q1: [f64; 2],
) -> (f64, f64, [f64; 2], [f64; 2], f64) {
    let u = [p1[0] - p0[0], p1[1] - p0[1]];
    let v = [q1[0] - q0[0], q1[1] - q0[1]];
    let w = [p0[0] - q0[0], p0[1] - q0[1]];
    let a = u[0] * u[0] + u[1] * u[1];
    let b = u[0] * v[0] + u[1] * v[1];
    let c = v[0] * v[0] + v[1] * v[1];
    let d = u[0] * w[0] + u[1] * w[1];
    let e = v[0] * w[0] + v[1] * w[1];
    let den = a * c - b * b;
    let mut s = if den.abs() > f64::EPSILON * (1.0 + a * c) {
        ((b * e - c * d) / den).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = if c > 0.0 { (b * s + e) / c } else { 0.0 };
    if t < 0.0 {
        t = 0.0;
        s = if a > 0.0 {
            (-d / a).clamp(0.0, 1.0)
        } else {
            0.0
        };
    } else if t > 1.0 {
        t = 1.0;
        s = if a > 0.0 {
            ((b - d) / a).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    let pp = [p0[0] + s * u[0], p0[1] + s * u[1]];
    let qq = [q0[0] + t * v[0], q0[1] + t * v[1]];
    (s, t, pp, qq, (pp[0] - qq[0]).hypot(pp[1] - qq[1]))
}

fn split_edge_at(mesh: &mut MaterialMesh, edge: usize, fraction: f64, point: [f64; 2]) -> usize {
    let n = mesh.n();
    let f = fraction.clamp(0.0, 1.0);
    let endpoint_tol = 4096.0 * f64::EPSILON;
    if f <= endpoint_tol {
        return edge;
    }
    if f >= 1.0 - endpoint_tol {
        return (edge + 1) % n;
    }
    let old = mesh.edges[edge];
    let first = |value: f64| value * f;
    let second = |value: f64| value * (1.0 - f);
    mesh.vertices.insert(edge + 1, point);
    mesh.edges[edge] = MeshEdge {
        m: first(old.m),
        b: first(old.b),
        tracer_m: first(old.tracer_m),
        tracer_b: first(old.tracer_b),
        m_young: first(old.m_young),
        ruptured: old.ruptured,
    };
    mesh.edges.insert(
        edge + 1,
        MeshEdge {
            m: second(old.m),
            b: second(old.b),
            tracer_m: second(old.tracer_m),
            tracer_b: second(old.tracer_b),
            m_young: second(old.m_young),
            ruptured: old.ruptured,
        },
    );
    edge + 1
}

/// Observer-only report of the nearest segment-apposition candidate under the
/// same local conditions used by [`try_local_segment_fission`].
pub fn find_local_segment_apposition(
    parent: &MaterialMesh,
    params: &FissionParams,
) -> Option<(usize, usize, f64, f64, f64)> {
    local_segment_appositions(parent, params).into_iter().next()
}

/// Signed edge strain under the accepted V4 load-bearing mechanics contract.
/// Compression remains fully load bearing, while tensile strain is weighted by
/// the mature structural fraction. Historical contracts retain raw strain.
pub fn effective_signed_strain(parent: &MaterialMesh, edge: usize) -> f64 {
    let raw = parent.strain(edge);
    if parent.is_maturation_coupled() && raw >= 0.0 {
        parent.mature_structural_fraction(edge) * raw
    } else {
        raw
    }
}

/// Enumerate all in-range segment pairs and compare the frozen legacy stress
/// predicate with the candidate signed-load interpretation. Observer only.
pub fn segment_apposition_stress_audit(
    parent: &MaterialMesh,
    params: &FissionParams,
) -> Vec<SegmentAppositionStressAudit> {
    if !parent.can_advance_physics()
        || parent.n() < params.min_vertices
        || !crate::mesh_self_contact::polygon_simple(&parent.vertices)
    {
        return Vec::new();
    }
    let n = parent.n();
    let min_sep = (n / 4).max(3);
    let range = crate::mesh_topology::local_rebond_range(parent, &params.topo);
    let mut rows = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let ring_separation = (j - i).min(n - (j - i));
            if ring_separation < min_sep || j == i + 1 || (i == 0 && j + 1 == n) {
                continue;
            }
            let (si, sj, _, _, distance) = closest_segment_points(
                parent.vertices[i],
                parent.vertices[(i + 1) % n],
                parent.vertices[j],
                parent.vertices[(j + 1) % n],
            );
            if distance > range {
                continue;
            }
            let prev_i = (i + n - 1) % n;
            let prev_j = (j + n - 1) % n;
            let incident_edges = [i, prev_i, j, prev_j];
            let raw_strains = incident_edges.map(|edge| parent.strain(edge));
            let mature_fractions =
                incident_edges.map(|edge| parent.mature_structural_fraction(edge));
            let effective_signed_strains =
                incident_edges.map(|edge| effective_signed_strain(parent, edge));
            let maximum_tensile_effective_strain = effective_signed_strains
                .iter()
                .copied()
                .fold(0.0_f64, f64::max);
            let maximum_compressive_effective_strain_magnitude = effective_signed_strains
                .iter()
                .copied()
                .filter(|strain| *strain < 0.0)
                .map(f64::abs)
                .fold(0.0_f64, f64::max);
            let legacy_tensile_only_predicate = raw_strains[0].max(raw_strains[1]) > 0.15
                || raw_strains[2].max(raw_strains[3]) > 0.15
                || parent.edges[i].ruptured
                || parent.edges[prev_j].ruptured
                || distance < range * 0.55;
            let signed_magnitude_predicate = effective_signed_strains[0]
                .abs()
                .max(effective_signed_strains[1].abs())
                > 0.15
                || effective_signed_strains[2]
                    .abs()
                    .max(effective_signed_strains[3].abs())
                    > 0.15
                || parent.edges[i].ruptured
                || parent.edges[prev_j].ruptured
                || distance < range * 0.55;
            rows.push(SegmentAppositionStressAudit {
                edge_i: i,
                edge_j: j,
                segment_i_fraction: si,
                segment_j_fraction: sj,
                distance,
                range,
                ring_separation,
                incident_edges,
                raw_strains,
                mature_fractions,
                effective_signed_strains,
                maximum_tensile_effective_strain,
                maximum_compressive_effective_strain_magnitude,
                legacy_tensile_only_predicate,
                signed_magnitude_predicate,
            });
        }
    }
    rows.sort_by(|left, right| left.distance.total_cmp(&right.distance));
    rows
}

fn local_segment_appositions(
    parent: &MaterialMesh,
    params: &FissionParams,
) -> Vec<(usize, usize, f64, f64, f64)> {
    if !parent.can_advance_physics()
        || parent.n() < params.min_vertices
        || !crate::mesh_self_contact::polygon_simple(&parent.vertices)
    {
        return Vec::new();
    }
    let n = parent.n();
    let min_sep = (n / 4).max(3);
    let range = crate::mesh_topology::local_rebond_range(parent, &params.topo);
    let mut candidates: Vec<(usize, usize, f64, f64, f64)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let ring_sep = (j - i).min(n - (j - i));
            if ring_sep < min_sep || j == i + 1 || (i == 0 && j + 1 == n) {
                continue;
            }
            let (si, sj, _, _, distance) = closest_segment_points(
                parent.vertices[i],
                parent.vertices[(i + 1) % n],
                parent.vertices[j],
                parent.vertices[(j + 1) % n],
            );
            if distance > range {
                continue;
            }
            let strain_i = parent.strain(i).max(parent.strain((i + n - 1) % n));
            let strain_j = parent.strain(j).max(parent.strain((j + n - 1) % n));
            let stressed = strain_i > 0.15
                || strain_j > 0.15
                || parent.edges[i].ruptured
                || parent.edges[(j + n - 1) % n].ruptured
                || distance < range * 0.55;
            if stressed {
                candidates.push((i, j, si, sj, distance));
            }
        }
    }
    candidates.sort_by(|a, b| a.4.total_cmp(&b.4));
    candidates
}

/// Mesh-independent local scission candidate based on closest points of two
/// nonadjacent membrane segments.  It uses the existing mass gate (at the
/// caller), ring separation, local rebond range, stress rule, structural
/// density, A-funded cross-boundary cost, and conservative partition kernel.
/// No target axis, timer, or new geometric threshold is introduced.
pub fn try_local_segment_fission(
    parent: &MaterialMesh,
    params: &FissionParams,
) -> Option<(MaterialMesh, MaterialMesh, FissionEvent)> {
    if !parent.can_advance_physics()
        || parent.n() < params.min_vertices
        || !params.topo.enable_rebond
        || !crate::mesh_self_contact::polygon_simple(&parent.vertices)
    {
        return None;
    }
    for (i, j, si, sj, _) in local_segment_appositions(parent, params) {
        let pi = [
            parent.vertices[i][0]
                + si * (parent.vertices[(i + 1) % parent.n()][0] - parent.vertices[i][0]),
            parent.vertices[i][1]
                + si * (parent.vertices[(i + 1) % parent.n()][1] - parent.vertices[i][1]),
        ];
        let pj = [
            parent.vertices[j][0]
                + sj * (parent.vertices[(j + 1) % parent.n()][0] - parent.vertices[j][0]),
            parent.vertices[j][1]
                + sj * (parent.vertices[(j + 1) % parent.n()][1] - parent.vertices[j][1]),
        ];
        let mut split = parent.clone();
        let vi = split_edge_at(&mut split, i, si, pi);
        let shifted_j = if split.n() > parent.n() { j + 1 } else { j };
        let vj = split_edge_at(&mut split, shifted_j, sj, pj);
        let Some((d1, d2, event)) = try_local_fission_at_vertices(&split, params, vi, vj) else {
            continue;
        };
        if crate::mesh_self_contact::polygon_simple(&d1.vertices)
            && crate::mesh_self_contact::polygon_simple(&d2.vertices)
            && event.partition.ok
        {
            return Some((d1, d2, event));
        }
    }
    None
}

/// Clone-only R10 diagnostic: evaluate the unchanged downstream segment split
/// and conservative fission kernel for one already-observed apposition. This
/// bypasses only candidate stress selection and must not be used by production.
pub fn try_local_segment_fission_at_observed_apposition(
    parent: &MaterialMesh,
    params: &FissionParams,
    edge_i: usize,
    edge_j: usize,
    segment_i_fraction: f64,
    segment_j_fraction: f64,
) -> Option<(MaterialMesh, MaterialMesh, FissionEvent)> {
    if !parent.can_advance_physics()
        || parent.n() < params.min_vertices
        || !params.topo.enable_rebond
        || !crate::mesh_self_contact::polygon_simple(&parent.vertices)
        || edge_i >= parent.n()
        || edge_j >= parent.n()
    {
        return None;
    }
    let n = parent.n();
    let ring_separation = (edge_j.abs_diff(edge_i)).min(n - edge_j.abs_diff(edge_i));
    let min_sep = (n / 4).max(3);
    if ring_separation < min_sep || edge_j == edge_i + 1 || (edge_i == 0 && edge_j + 1 == n) {
        return None;
    }
    let (si, sj, pi, pj, distance) = closest_segment_points(
        parent.vertices[edge_i],
        parent.vertices[(edge_i + 1) % n],
        parent.vertices[edge_j],
        parent.vertices[(edge_j + 1) % n],
    );
    let tolerance = f64::EPSILON * 64.0;
    if (si - segment_i_fraction).abs() > tolerance
        || (sj - segment_j_fraction).abs() > tolerance
        || distance > crate::mesh_topology::local_rebond_range(parent, &params.topo)
    {
        return None;
    }
    let mut split = parent.clone();
    let vi = split_edge_at(&mut split, edge_i, si, pi);
    let shifted_j = if split.n() > parent.n() {
        edge_j + 1
    } else {
        edge_j
    };
    let vj = split_edge_at(&mut split, shifted_j, sj, pj);
    let (d1, d2, event) = try_local_fission_at_vertices(&split, params, vi, vj)?;
    (crate::mesh_self_contact::polygon_simple(&d1.vertices)
        && crate::mesh_self_contact::polygon_simple(&d2.vertices)
        && event.partition.ok)
        .then_some((d1, d2, event))
}

/// Step topology operators (rupture + same-edge rebond). Fission is separate.
pub fn topology_step(mesh: &mut MaterialMesh, params: &FissionParams) -> TopologyLedger {
    let mut led = TopologyLedger::default();
    led.tension_ruptures = crate::mesh_topology::tension_rupture_step(mesh, &params.topo);
    led.local_rebonds = crate::mesh_topology::local_same_edge_rebond(mesh, &params.topo);
    led
}

#[cfg(test)]
mod segment_tests {
    use super::*;
    use crate::material_mesh::LumpedChem;

    #[test]
    fn segment_apposition_splits_a_simple_dumbbell_conservatively() {
        let points = vec![
            [-3.0, -2.0],
            [-1.0, -2.0],
            [-0.5, -0.2],
            [0.5, -0.2],
            [1.0, -2.0],
            [3.0, -2.0],
            [3.0, 2.0],
            [1.0, 2.0],
            [0.5, 0.2],
            [-0.5, 0.2],
            [-1.0, 2.0],
            [-3.0, 2.0],
        ];
        let mut mesh = MaterialMesh::seed_regular(
            points.len(),
            3.0,
            0.0,
            0.0,
            1.0,
            0.5,
            LumpedChem {
                a: 100.0,
                c: 1.0,
                ..Default::default()
            },
            LumpedChem::default(),
            0.0,
        );
        mesh.vertices = points;
        for i in 0..mesh.n() {
            mesh.edges[i].m = mesh.rho_s * mesh.edge_length(i);
        }
        assert!(crate::mesh_self_contact::polygon_simple(&mesh.vertices));
        let (a, b, event) = try_local_segment_fission(&mesh, &FissionParams::default())
            .expect("lawful segment apposition");
        assert!(event.partition.ok);
        assert!(crate::mesh_self_contact::polygon_simple(&a.vertices));
        assert!(crate::mesh_self_contact::polygon_simple(&b.vertices));
    }
}
