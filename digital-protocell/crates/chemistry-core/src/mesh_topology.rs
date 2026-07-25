//! Observer-only mesh topology helpers and local tension rupture / cross-rebond.
//!
//! Topology component detection never drives biology. Bond events are local.

use crate::material_mesh::{MaterialMesh, MeshEdge, DEFAULT_REBOND_DIST};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyParams {
    /// Strain above which an intact bond may rupture from tension.
    pub strain_rupture: f64,
    /// Max distance for free-end rebonding / cross-bonding.
    pub rebond_dist: f64,
    pub enable_rupture: bool,
    pub enable_rebond: bool,
}

impl Default for TopologyParams {
    fn default() -> Self {
        Self {
            strain_rupture: 1.25,
            // Neck cross-bond: ≤2× certified free-end rebond distance (local thin-neck geometry).
            rebond_dist: DEFAULT_REBOND_DIST * 2.0,
            enable_rupture: true,
            enable_rebond: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopologyLedger {
    pub tension_ruptures: usize,
    pub local_rebonds: usize,
    pub cross_bonds: usize,
}

/// Rupture bonds under excessive local tension (or below mass threshold already handled elsewhere).
pub fn tension_rupture_step(mesh: &mut MaterialMesh, topo: &TopologyParams) -> usize {
    if !topo.enable_rupture || !mesh.alive {
        return 0;
    }
    let n = mesh.n();
    let mut count = 0usize;
    for i in 0..n {
        if mesh.edges[i].ruptured {
            continue;
        }
        if mesh.strain(i) > topo.strain_rupture {
            let rem = mesh.edges[i].m;
            mesh.edges[i].m = 0.0;
            mesh.edges[i].b *= 0.5;
            mesh.edges[i].ruptured = true;
            mesh.interior.w += rem / mesh.area().max(1e-6);
            count += 1;
        }
    }
    count
}

/// Local rebond of original ruptured edge if ends are close (Phase 1 behavior).
pub fn local_same_edge_rebond(mesh: &mut MaterialMesh, topo: &TopologyParams) -> usize {
    if !topo.enable_rebond || !mesh.alive {
        return 0;
    }
    if mesh.interior.a < 0.05 || mesh.interior.c < 0.05 {
        return 0;
    }
    let n = mesh.n();
    let mut count = 0usize;
    for i in 0..n {
        if !mesh.edges[i].ruptured {
            continue;
        }
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % n];
        let dist = (b[0] - a[0]).hypot(b[1] - a[1]);
        if dist <= topo.rebond_dist {
            let need = mesh.rho_s * dist;
            let area = mesh.area().max(1e-6);
            let have = mesh.interior.a.max(0.0) * area;
            if have >= need * 0.5 {
                let take = need.min(have);
                mesh.interior.a = (mesh.interior.a - take / area).max(0.0);
                mesh.edges[i].m = take;
                mesh.edges[i].ruptured = false;
                count += 1;
            }
        }
    }
    count
}

/// Local geometric bond range: scales with mean edge length (organism scale), not a fixed world length.
pub fn local_rebond_range(mesh: &MaterialMesh, topo: &TopologyParams) -> f64 {
    let n = mesh.n().max(1) as f64;
    let mean_ell = mesh.perimeter() / n;
    // Allow cross-neck bonding when opposing free ends are within a few local edge lengths.
    (topo.rebond_dist.max(DEFAULT_REBOND_DIST) * 0.5 + 3.5 * mean_ell).clamp(DEFAULT_REBOND_DIST, 18.0)
}

/// Find a local pinch candidate with O(n) sampling (stride) rather than full O(n²).
pub fn find_local_pinch(mesh: &MaterialMesh, topo: &TopologyParams) -> Option<(usize, usize)> {
    let n = mesh.n();
    if n < 8 {
        return None;
    }
    let min_sep = (n / 4).max(3);
    let range = local_rebond_range(mesh, topo);
    let stride = ((n / 16).max(1)).min(4);
    let mut best: Option<(f64, usize, usize)> = None;
    for i in (0..n).step_by(stride) {
        for dj in (min_sep..=(n - min_sep)).step_by(stride) {
            let j = (i + dj) % n;
            if j <= i {
                continue;
            }
            let ring_sep = (j - i).min(n - (j - i));
            if ring_sep < min_sep {
                continue;
            }
            let a = mesh.vertices[i];
            let b = mesh.vertices[j];
            let dist = (b[0] - a[0]).hypot(b[1] - a[1]);
            if dist > range {
                continue;
            }
            let strain_i = mesh.strain(i).max(mesh.strain((i + n - 1) % n));
            let strain_j = mesh.strain(j).max(mesh.strain((j + n - 1) % n));
            let stressed = strain_i > 0.15
                || strain_j > 0.15
                || mesh.edges[i].ruptured
                || mesh.edges[(j + n - 1) % n].ruptured
                || dist < range * 0.55;
            if !stressed {
                continue;
            }
            let score = dist;
            if best.map(|(s, _, _)| score < s).unwrap_or(true) {
                best = Some((score, i, j));
            }
        }
    }
    best.map(|(_, i, j)| (i, j))
}

/// Observer: is the mesh a single closed loop?
pub fn is_closed_component(mesh: &MaterialMesh) -> bool {
    mesh.n() >= 3 && mesh.edges.iter().all(|e| !e.ruptured && e.m > 0.0)
}

/// Build a new closed mesh from a contiguous vertex range [start, end] inclusive,
/// inheriting edge materials along the path and a closing edge if needed.
pub fn extract_loop(
    parent: &MaterialMesh,
    start: usize,
    end: usize,
    closing: Option<MeshEdge>,
) -> MaterialMesh {
    let n = parent.n();
    let mut verts = Vec::new();
    let mut edges = Vec::new();
    let mut i = start;
    loop {
        verts.push(parent.vertices[i]);
        if i == end {
            break;
        }
        edges.push(parent.edges[i]);
        i = (i + 1) % n;
        // safety
        if verts.len() > n + 2 {
            break;
        }
    }
    if let Some(c) = closing {
        edges.push(c);
    } else if verts.len() >= 2 {
        // closing edge between end and start — mass from leftover if any
        let dist = {
            let a = verts[verts.len() - 1];
            let b = verts[0];
            (b[0] - a[0]).hypot(b[1] - a[1])
        };
        edges.push(MeshEdge {
            m: parent.rho_s * dist,
            b: 0.0,
            tracer_m: 0.0,
            tracer_b: 0.0,
            ruptured: false,
        });
    }
    // Ensure edges.len() == verts.len()
    while edges.len() < verts.len() {
        edges.push(MeshEdge::default());
    }
    edges.truncate(verts.len());
    MaterialMesh {
        vertices: verts,
        edges,
        free_l: 0.0, // filled by partition
        interior: parent.interior,
        exterior: parent.exterior,
        rho_s: parent.rho_s,
        b_max_per_length: parent.b_max_per_length,
        bond_threshold: parent.bond_threshold,
        l_max: parent.l_max,
        l_min: parent.l_min,
        alive: true,
        death_reason: None,
        equation_id: parent.equation_id.clone(),
        schema_version: parent.schema_version,
        // Templates partitioned separately after both daughters exist.
        templates: Vec::new(),
        next_template_id: parent.next_template_id,
        template_rng: parent.template_rng,
        autocatalytic_edges: Vec::new(),
        next_edge_id: parent.next_edge_id,
    }
}
