//! D-094 autocatalytic edge copying with target-identity mutation.

use crate::autocatalytic_edges::{spawn_edge, EDGE_MASS};
use crate::autocatalytic_nodes::AutocatalyticLedger;
use crate::autocatalytic_nodes::{
    autocatalytic_schema_load_ok, node_conc, AutocatalyticParams, NodeKind,
};
use crate::material_mesh::MaterialMesh;
use crate::template_polymer::{RngLike, XorShift64};

const EPS: f64 = 1e-15;

/// K_i + E_ij + Q_E + A → K_i + 2 E_ij + W
/// Source identity fixed; target mutates with μ_E during copy only.
pub fn edge_copying_step(
    mesh: &mut MaterialMesh,
    p: &AutocatalyticParams,
    dt: f64,
    rng: &mut XorShift64,
) -> AutocatalyticLedger {
    let mut led = AutocatalyticLedger::default();
    if !p.enable || !p.enable_edge_copy {
        return led;
    }
    if !autocatalytic_schema_load_ok(mesh, p) {
        led.rejected_steps += 1;
        return led;
    }
    let area = mesh.area().max(EPS);
    let n = mesh.autocatalytic_edges.len();
    let mut newborns = Vec::new();
    for i in 0..n {
        let src = mesh.autocatalytic_edges[i].source;
        let tgt = mesh.autocatalytic_edges[i].target;
        let pos = mesh.autocatalytic_edges[i].pos;
        let parent_id = mesh.autocatalytic_edges[i].id;
        let k_i = node_conc(&mesh.interior, src);
        if k_i <= EPS {
            led.rejected_steps += 1;
            continue;
        }
        let a = mesh.interior.a.max(0.0);
        let qe = mesh.interior.q_e.max(0.0);
        let rate = p.k_edge_copy * k_i * a * qe / (qe + 0.2).max(EPS);
        let extent = (rate * dt).min(a).min(qe);
        // One discrete copy event when cumulative extent crosses edge mass unit.
        // Ponytail: probabilistic Bernoulli with p = 1 - exp(-extent/EDGE_MASS).
        let p_copy = 1.0 - (-extent / EDGE_MASS).exp();
        if rng.unit() >= p_copy {
            continue;
        }
        let cost = EDGE_MASS.min(a).min(qe);
        if cost < EDGE_MASS * 0.99 {
            led.rejected_steps += 1;
            continue;
        }
        mesh.interior.a = (mesh.interior.a - cost).max(0.0);
        mesh.interior.q_e = (mesh.interior.q_e - cost).max(0.0);
        mesh.interior.w += cost;
        led.a_consumed += cost * area;
        led.q_e_consumed += cost * area;
        led.w_produced += cost * area;
        led.edge_copied += EDGE_MASS;

        let mut new_tgt = tgt;
        if p.mu_e > 0.0 && rng.unit() < p.mu_e {
            let alts = tgt.other_targets();
            new_tgt = if rng.unit() < 0.5 { alts[0] } else { alts[1] };
            led.edge_mutated += 1.0;
        }
        // Physically separate: offset from parent.
        let ang = rng.unit() * std::f64::consts::TAU;
        let npos = [pos[0] + 0.15 * ang.cos(), pos[1] + 0.15 * ang.sin()];
        newborns.push((src, new_tgt, npos, parent_id));
    }
    for (src, tgt, pos, parent_id) in newborns {
        let _ = spawn_edge(mesh, src, tgt, pos, Some(parent_id));
    }
    led
}

/// Slow edge material loss when unsupported (orphan decay); returns Q_E.
pub fn edge_loss_step(
    mesh: &mut MaterialMesh,
    p: &AutocatalyticParams,
    dt: f64,
    rng: &mut XorShift64,
) -> AutocatalyticLedger {
    let mut led = AutocatalyticLedger::default();
    if !p.enable || p.k_edge_loss <= 0.0 {
        return led;
    }
    if !autocatalytic_schema_load_ok(mesh, p) {
        led.rejected_steps += 1;
        return led;
    }
    let area = mesh.area().max(EPS);
    let p_loss = 1.0 - (-p.k_edge_loss * dt).exp();
    let mut keep = Vec::with_capacity(mesh.autocatalytic_edges.len());
    for e in mesh.autocatalytic_edges.drain(..) {
        // Edges without their source node are more fragile.
        let src_k = node_conc(&mesh.interior, e.source);
        let boost = if src_k < 1e-6 { 3.0 } else { 1.0 };
        if rng.unit() < p_loss * boost {
            mesh.interior.q_e += EDGE_MASS;
            mesh.interior.w += 0.05 * EDGE_MASS;
            led.edge_lost += EDGE_MASS;
            led.w_produced += 0.05 * EDGE_MASS * area;
        } else {
            keep.push(e);
        }
    }
    mesh.autocatalytic_edges = keep;
    led
}

fn place_inside(mesh: &MaterialMesh, target: [f64; 2]) -> [f64; 2] {
    if mesh.point_inside(target[0], target[1]) {
        return target;
    }
    let c = mesh.centroid();
    for t in [0.85, 0.7, 0.55, 0.4, 0.25, 0.1] {
        let p = [
            c[0] + t * (target[0] - c[0]),
            c[1] + t * (target[1] - c[1]),
        ];
        if mesh.point_inside(p[0], p[1]) {
            return p;
        }
    }
    c
}

/// Place hereditary edges throughout the body (not only on one axis).
/// Local pinch fission often buds off-axis; midline-only seeds empty one daughter.
pub fn redistribute_edges_along_axis(mesh: &mut MaterialMesh) {
    if mesh.autocatalytic_edges.is_empty() {
        return;
    }
    let c = mesh.centroid();
    let nv = mesh.n().max(1);
    let n = mesh.autocatalytic_edges.len();
    let mut positions = Vec::with_capacity(n);
    for i in 0..n {
        let vi = (i * nv / n.max(1)) % nv;
        let v = mesh.vertices[vi];
        // Offset from centroid toward distinct boundary vertices so both pinch
        // lobes can receive edge material.
        let t = 0.35 + 0.25 * ((i % 3) as f64) / 2.0;
        let target = [c[0] + t * (v[0] - c[0]), c[1] + t * (v[1] - c[1])];
        positions.push(place_inside(mesh, target));
    }
    for (e, pos) in mesh.autocatalytic_edges.iter_mut().zip(positions) {
        e.pos = pos;
    }
}

pub fn seed_founder_edges(mesh: &mut MaterialMesh, edges: &[(NodeKind, NodeKind)]) {
    let c = mesh.centroid();
    let nv = mesh.n().max(1);
    let n = edges.len().max(1);
    for (i, &(src, tgt)) in edges.iter().enumerate() {
        let vi = (i * nv / n) % nv;
        let v = mesh.vertices[vi];
        let t = 0.35 + 0.25 * ((i % 3) as f64) / 2.0;
        let target = [c[0] + t * (v[0] - c[0]), c[1] + t * (v[1] - c[1])];
        let pos = place_inside(mesh, target);
        let _ = spawn_edge(mesh, src, tgt, pos, None);
    }
}

/// Founder topologies (frozen before selection).
/// Each directed edge type is instantiated twice and spaced along the body axis
/// so physical fission can partition hereditary material into both daughters.
pub fn founder_h_edges() -> Vec<(NodeKind, NodeKind)> {
    use NodeKind::*;
    let base = [(A, A), (A, R), (R, A), (R, B), (B, A)];
    base.iter().chain(base.iter()).copied().collect()
}

pub fn founder_b_edges() -> Vec<(NodeKind, NodeKind)> {
    use NodeKind::*;
    let base = [(B, B), (B, R), (R, B), (R, A), (A, B)];
    base.iter().chain(base.iter()).copied().collect()
}

pub fn founder_n_edges() -> Vec<(NodeKind, NodeKind)> {
    use NodeKind::*;
    let base = [(A, R), (R, B), (B, A), (R, A), (B, R)];
    base.iter().chain(base.iter()).copied().collect()
}
