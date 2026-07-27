//! Topological fission via local pinch rebonding + conservative partition.
//!
//! No `divide()` command. Topology change is local bond events; component
//! discovery is observer bookkeeping only.

use crate::material_mesh::{MaterialMesh, MeshEdge};
use crate::mesh_topology::{extract_loop, find_local_pinch, TopologyLedger, TopologyParams};
use crate::autocatalytic_partition::partition_autocatalytic_edges;
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
    pub residual_u_h: f64,
    #[serde(default)]
    pub residual_u_b: f64,
    #[serde(default)]
    pub residual_templates: f64,
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

const ACCOUNTING_TOL: f64 = 1e-6;

/// Attempt one local pinch fission if a stressed neck exists.
/// Returns two daughters + event, or None if no lawful local topology change.
pub fn try_local_fission(
    parent: &MaterialMesh,
    params: &FissionParams,
) -> Option<(MaterialMesh, MaterialMesh, FissionEvent)> {
    if !parent.alive || parent.n() < params.min_vertices {
        return None;
    }
    if !params.topo.enable_rebond {
        return None;
    }
    let (i, j) = find_local_pinch(parent, &params.topo)?;
    // Cross-bond mass drawn from local A and nearby edge material.
    let a = parent.vertices[i];
    let b = parent.vertices[j];
    let dist = (b[0] - a[0]).hypot(b[1] - a[1]);
    let range = crate::mesh_topology::local_rebond_range(parent, &params.topo);
    if dist > range {
        return None;
    }
    let need = parent.rho_s * dist;
    let area = parent.area().max(1e-6);
    let have_a = parent.interior.a.max(0.0) * area;
    if have_a < need * 0.25 {
        return None;
    }

    // Build two loops: i→j and j→i with shared closing edges (cross bonds).
    let close_ab = MeshEdge {
        m: need * 0.5,
        b: 0.0,
        tracer_m: 0.0,
        tracer_b: 0.0,
        ruptured: false,
    };
    let close_ba = MeshEdge {
        m: need * 0.5,
        b: 0.0,
        tracer_m: 0.0,
        tracer_b: 0.0,
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
        mesh.finite_allocation = parent.finite_allocation;
    };
    set_conc(&mut d1, f1);
    set_conc(&mut d2, f2);

    // Physical template partition by spatial location (no sequence copy).
    let (_n1, _n2, residual_templates) = partition_templates(parent, &mut d1, &mut d2);
    // Physical autocatalytic edge partition by position (no whole-network clone).
    let (_e1, _e2, residual_acs) = partition_autocatalytic_edges(parent, &mut d1, &mut d2);
    let _ = residual_acs;
    let _ = pre_acs_edges;

    // Cost of cross-bond: A consumed (leakage/waste).
    let take = (need * 0.5).min(have_a);
    let leakage = take;
    // Deduct from daughters proportionally (already split); reduce A slightly.
    d1.interior.a = (d1.interior.a - (take * f1) / d1.area().max(1e-9)).max(0.0);
    d2.interior.a = (d2.interior.a - (take * f2) / d2.area().max(1e-9)).max(0.0);
    d1.interior.w += (take * f1) / d1.area().max(1e-9);
    d2.interior.w += (take * f2) / d2.area().max(1e-9);

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
    let post_u_h = d1.interior.u_h * d1.area() + d2.interior.u_h * d2.area();
    let post_u_b = d1.interior.u_b * d1.area() + d2.interior.u_b * d2.area();
    let post_tmpl = (d1.templates.len() + d2.templates.len()) as f64;

    // Structural: parent m + new cross-bond mass ≈ post (cross bonds add need)
    let residual_m = (post_m - (pre_m + need)).abs();
    let residual_b = (post_b - pre_b).abs();
    let residual_l = (post_l - pre_l).abs();
    let residual_c = (post_c - pre_c).abs();
    let residual_c_h = (post_c_h - pre_c_h).abs();
    let residual_c_b = (post_c_b - pre_c_b).abs();
    // A and W: A decreases by take, W increases by take
    let residual_a = (post_a - (pre_a - take)).abs();
    let residual_n = (post_n - pre_n).abs();
    let residual_f = (post_f - pre_f).abs();
    let residual_w = (post_w - (pre_w + take)).abs();
    let residual_r = (post_r - pre_r).abs();
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
        && residual_templates < 0.5
        && (post_tmpl - pre_tmpl).abs() < 0.5
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
            residual_u_h,
            residual_u_b,
            residual_templates,
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

/// Step topology operators (rupture + same-edge rebond). Fission is separate.
pub fn topology_step(mesh: &mut MaterialMesh, params: &FissionParams) -> TopologyLedger {
    let mut led = TopologyLedger::default();
    led.tension_ruptures = crate::mesh_topology::tension_rupture_step(mesh, &params.topo);
    led.local_rebonds = crate::mesh_topology::local_same_edge_rebond(mesh, &params.topo);
    led
}
