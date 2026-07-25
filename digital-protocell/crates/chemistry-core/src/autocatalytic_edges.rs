//! D-094 physical catalytic-edge complexes E_ij.

use crate::autocatalytic_nodes::{
    add_node_conc, autocatalytic_schema_load_ok, AutocatalyticLedger, AutocatalyticParams, NodeKind,
};
use crate::material_mesh::MaterialMesh;
use serde::{Deserialize, Serialize};

const EPS: f64 = 1e-15;
/// Each edge complex carries one catalyst-equivalent unit of edge material.
pub const EDGE_MASS: f64 = 1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalyticEdgeComplex {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub source: NodeKind,
    pub target: NodeKind,
    pub pos: [f64; 2],
    /// Bound catalyst equivalents stored on this edge (observer mass unit).
    pub bound_c: f64,
    /// Observer-only ancestry / tracer (never enters chemistry decisions).
    #[serde(default)]
    pub tracer: f64,
    #[serde(default)]
    pub lineage_tag: u64,
}

impl CatalyticEdgeComplex {
    pub fn label(&self) -> String {
        format!("E_{}{}", self.source.as_str(), self.target.as_str())
    }
}

pub fn edge_counts(mesh: &MaterialMesh) -> [[usize; 3]; 3] {
    let mut m = [[0usize; 3]; 3];
    for e in &mesh.autocatalytic_edges {
        let i = e.source as usize;
        let j = e.target as usize;
        m[i][j] += 1;
    }
    m
}

pub fn edge_frequency_vector(mesh: &MaterialMesh) -> [f64; 9] {
    let counts = edge_counts(mesh);
    let tot = counts.iter().flatten().sum::<usize>().max(1) as f64;
    let mut v = [0.0; 9];
    for i in 0..3 {
        for j in 0..3 {
            v[i * 3 + j] = counts[i][j] as f64 / tot;
        }
    }
    v
}

/// Target-node allocation of present edges (network phenotype orientation).
pub fn network_response_vector(mesh: &MaterialMesh) -> [f64; 3] {
    let mut t = [0.0; 3];
    for e in &mesh.autocatalytic_edges {
        t[e.target as usize] += 1.0;
    }
    let s = t.iter().sum::<f64>().max(1.0);
    [t[0] / s, t[1] / s, t[2] / s]
}

pub fn total_edge_mass(mesh: &MaterialMesh) -> f64 {
    mesh.autocatalytic_edges.len() as f64 * EDGE_MASS
}

/// E_ij + Q_K + A → E_ij + K_j + W  (E catalytic, not consumed).
pub fn node_production_step(
    mesh: &mut MaterialMesh,
    p: &AutocatalyticParams,
    dt: f64,
) -> AutocatalyticLedger {
    let mut led = AutocatalyticLedger::default();
    if !p.enable || !p.enable_node_prod {
        return led;
    }
    if !autocatalytic_schema_load_ok(mesh, p) {
        led.rejected_steps += 1;
        return led;
    }
    let area = mesh.area().max(EPS);
    let n_edges = mesh.autocatalytic_edges.len();
    for i in 0..n_edges {
        let target = mesh.autocatalytic_edges[i].target;
        let enabled = match target {
            NodeKind::A => p.enable_ka,
            NodeKind::R => p.enable_kr,
            NodeKind::B => p.enable_kb,
        };
        if !enabled {
            continue;
        }
        let a = mesh.interior.a.max(0.0);
        let qk = mesh.interior.q_k.max(0.0);
        let rate = p.k_node_prod * a * qk / (qk + 0.2).max(EPS);
        let extent = (rate * dt).min(a).min(qk);
        if extent <= 0.0 {
            led.rejected_steps += 1;
            continue;
        }
        mesh.interior.a = (mesh.interior.a - extent).max(0.0);
        mesh.interior.q_k = (mesh.interior.q_k - extent).max(0.0);
        add_node_conc(&mut mesh.interior, target, extent);
        mesh.interior.w += extent;
        led.node_produced += extent * area;
        led.a_consumed += extent * area;
        led.q_k_consumed += extent * area;
        led.w_produced += extent * area;
    }
    led
}

/// K_j → Q_K + W
pub fn node_turnover_step(
    mesh: &mut MaterialMesh,
    p: &AutocatalyticParams,
    dt: f64,
) -> AutocatalyticLedger {
    let mut led = AutocatalyticLedger::default();
    if !p.enable {
        return led;
    }
    if !autocatalytic_schema_load_ok(mesh, p) {
        led.rejected_steps += 1;
        return led;
    }
    let area = mesh.area().max(EPS);
    for kind in NodeKind::all() {
        let k = crate::autocatalytic_nodes::node_conc(&mesh.interior, kind);
        let turn = (p.k_node_turn * k * dt).min(k);
        if turn <= 0.0 {
            continue;
        }
        crate::autocatalytic_nodes::set_node_conc(&mut mesh.interior, kind, k - turn);
        mesh.interior.q_k += turn;
        mesh.interior.w += turn;
        led.node_turned += turn * area;
        led.w_produced += turn * area;
    }
    led
}

pub fn merge_acs_ledgers(dst: &mut AutocatalyticLedger, src: &AutocatalyticLedger) {
    dst.node_produced += src.node_produced;
    dst.node_turned += src.node_turned;
    dst.edge_copied += src.edge_copied;
    dst.edge_mutated += src.edge_mutated;
    dst.edge_lost += src.edge_lost;
    dst.a_consumed += src.a_consumed;
    dst.w_produced += src.w_produced;
    dst.q_k_consumed += src.q_k_consumed;
    dst.q_e_consumed += src.q_e_consumed;
    dst.rejected_steps += src.rejected_steps;
}

/// Seed one physical edge at a position.
pub fn spawn_edge(
    mesh: &mut MaterialMesh,
    source: NodeKind,
    target: NodeKind,
    pos: [f64; 2],
    parent_id: Option<u64>,
) -> u64 {
    let id = mesh.next_edge_id.max(1);
    mesh.next_edge_id = id + 1;
    mesh.autocatalytic_edges.push(CatalyticEdgeComplex {
        id,
        parent_id,
        source,
        target,
        pos,
        bound_c: EDGE_MASS,
        tracer: 0.0,
        lineage_tag: 0,
    });
    id
}
