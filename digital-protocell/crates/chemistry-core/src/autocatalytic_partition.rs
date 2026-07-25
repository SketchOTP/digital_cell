//! Physical partition of autocatalytic edges at fission (by position).

use crate::material_mesh::MaterialMesh;

/// Assign each parent edge complex to d1 or d2 by point-in-polygon.
/// No whole-network clone. Returns (n1, n2, residual_count).
pub fn partition_autocatalytic_edges(
    parent: &MaterialMesh,
    d1: &mut MaterialMesh,
    d2: &mut MaterialMesh,
) -> (usize, usize, f64) {
    d1.autocatalytic_edges.clear();
    d2.autocatalytic_edges.clear();
    let mut n1 = 0usize;
    let mut n2 = 0usize;
    let c1 = d1.centroid();
    let c2 = d2.centroid();
    for e in &parent.autocatalytic_edges {
        let p = e.pos;
        let in1 = d1.point_inside(p[0], p[1]);
        let in2 = d2.point_inside(p[0], p[1]);
        let go_d1 = match (in1, in2) {
            (true, false) => true,
            (false, true) => false,
            (true, true) | (false, false) => {
                let d_a = (p[0] - c1[0]).hypot(p[1] - c1[1]);
                let d_b = (p[0] - c2[0]).hypot(p[1] - c2[1]);
                d_a <= d_b
            }
        };
        if go_d1 {
            d1.autocatalytic_edges.push(e.clone());
            n1 += 1;
        } else {
            d2.autocatalytic_edges.push(e.clone());
            n2 += 1;
        }
    }
    d1.next_edge_id = parent.next_edge_id;
    d2.next_edge_id = parent.next_edge_id;
    let residual = ((n1 + n2) as f64 - parent.autocatalytic_edges.len() as f64).abs();
    (n1, n2, residual)
}

/// Whether a mesh still has a directed cycle among present edge labels.
pub fn has_directed_cycle(mesh: &MaterialMesh) -> bool {
    let mut adj = [[false; 3]; 3];
    for e in &mesh.autocatalytic_edges {
        adj[e.source as usize][e.target as usize] = true;
    }
    // Nodes A=0,R=1,B=2 — DFS cycle detect.
    fn dfs(u: usize, adj: &[[bool; 3]; 3], stack: &mut [bool; 3], seen: &mut [bool; 3]) -> bool {
        stack[u] = true;
        seen[u] = true;
        for v in 0..3 {
            if !adj[u][v] {
                continue;
            }
            if stack[v] {
                return true;
            }
            if !seen[v] && dfs(v, adj, stack, seen) {
                return true;
            }
        }
        stack[u] = false;
        false
    }
    let mut seen = [false; 3];
    for u in 0..3 {
        if !seen[u] {
            let mut stack = [false; 3];
            if dfs(u, &adj, &mut stack, &mut seen) {
                return true;
            }
        }
    }
    false
}
