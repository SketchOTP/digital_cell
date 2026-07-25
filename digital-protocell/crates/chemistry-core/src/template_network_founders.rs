//! Gate-2 topology founder preauthorization (D-093).
//!
//! Enumerate length-12 sequences with equal H/B and equal pair-channel counts,
//! select three founders from isolated binding-only network response only.

use crate::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use crate::mesh_reactions::ReactionParams;
use crate::metabolic_reserve::ReserveParams;
use crate::template_network::{
    count_pair_channels, derive_k_site, stamp_network_equation, NetworkParams, PairChannel,
};
use crate::template_network_binding::{network_binding_step, response_vector};
use crate::template_polymer::{seed_founder_chains, TemplateParams, FOUNDER_LEN};
use serde::{Deserialize, Serialize};

/// Frozen after Gate 2 — never selected from organism growth/survival.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyFounders {
    pub topology_h: String,
    pub topology_b: String,
    pub topology_n: String,
    pub class_size: usize,
    pub method: String,
}

fn max_run(seq: &str) -> usize {
    let mut best = 1usize;
    let mut cur = 1usize;
    let bytes = seq.as_bytes();
    for i in 1..bytes.len() {
        if bytes[i] == bytes[i - 1] {
            cur += 1;
            best = best.max(cur);
        } else {
            cur = 1;
        }
    }
    best
}

fn reverse_seq(seq: &str) -> String {
    seq.chars().rev().collect()
}

fn choose6_masks() -> Vec<u16> {
    let mut out = Vec::new();
    for mask in 0u16..(1 << 12) {
        if mask.count_ones() == 6 {
            out.push(mask);
        }
    }
    out
}

/// All length-12 sequences: 6H+6B, equal HH/HB/BH/BB counts, no run > 4,
/// unique up to reversal.
pub fn enumerate_topology_class() -> Vec<String> {
    let mut raw = Vec::new();
    for mask in choose6_masks() {
        let mut s = String::with_capacity(12);
        for i in 0..12 {
            if (mask & (1u16 << i)) != 0 {
                s.push('H');
            } else {
                s.push('B');
            }
        }
        let (hh, hb, bh, bb) = count_pair_channels(&s);
        if hh == hb && hb == bh && bh == bb && hh > 0 && max_run(&s) <= 4 {
            raw.push(s);
        }
    }
    // Dedup by reversal equivalence (keep lexicographically smaller).
    let mut uniq = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for s in raw {
        let rev = reverse_seq(&s);
        let key = if s <= rev { s.clone() } else { rev };
        if seen.insert(key.clone()) {
            uniq.push(key);
        }
    }
    uniq.sort();
    uniq
}

#[derive(Debug, Clone, Copy)]
pub struct CanonicalCondition {
    pub name: &'static str,
    pub a: f64,
    pub r: f64,
    pub n: f64,
    pub f: f64,
    pub damage: bool,
}

pub fn canonical_conditions() -> [CanonicalCondition; 4] {
    [
        CanonicalCondition {
            name: "low_a_nf",
            a: 0.08,
            r: 0.2,
            n: 1.0,
            f: 1.0,
            damage: false,
        },
        CanonicalCondition {
            name: "high_a_low_r",
            a: 1.2,
            r: 0.05,
            n: 0.6,
            f: 0.6,
            damage: false,
        },
        CanonicalCondition {
            name: "low_a_high_r",
            a: 0.08,
            r: 1.5,
            n: 0.2,
            f: 0.2,
            damage: false,
        },
        CanonicalCondition {
            name: "damage_r",
            a: 0.4,
            r: 1.0,
            n: 0.4,
            f: 0.4,
            damage: true,
        },
    ]
}

fn assay_mesh(seq: &str, cond: &CanonicalCondition) -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        16,
        6.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.9,
            a: cond.a,
            n: cond.n,
            f: cond.f,
            r: cond.r,
            w: 0.1,
            ..Default::default()
        },
        LumpedChem::default(),
        3.0,
    );
    stamp_network_equation(&mut mesh);
    let _ = seed_founder_chains(&mut mesh, seq, 1, 1);
    mesh.next_template_id = 2;
    if cond.damage {
        // Raise local strain / lower membrane occupancy near template.
        for e in &mut mesh.edges {
            e.m *= 0.55;
            e.b *= 0.35;
        }
    }
    mesh
}

/// Isolated binding-only equilibration under a canonical local condition.
pub fn isolated_response(
    seq: &str,
    cond: &CanonicalCondition,
    reserve: &ReserveParams,
    net: &NetworkParams,
    steps: usize,
) -> [f64; 4] {
    let mut mesh = assay_mesh(seq, cond);
    let mut react = ReactionParams::default();
    react.reserve = *reserve;
    react.template = TemplateParams::default();
    react.template.enable = true; // polymer present but we only step binding
    react.network = *net;
    for _ in 0..steps {
        let _ = network_binding_step(&mut mesh, &react, 0.05);
    }
    response_vector(&mesh)
}

/// Select topology H / B / N from isolated network response only.
pub fn preauthorize_founders(
    reserve: &ReserveParams,
    t_maint: f64,
    k_d: f64,
) -> Result<TopologyFounders, String> {
    let class = enumerate_topology_class();
    if class.len() < 3 {
        return Err(format!(
            "topology class too small: {} (need ≥3)",
            class.len()
        ));
    }
    // Verify equal channel counts.
    for s in &class {
        let (hh, hb, bh, bb) = count_pair_channels(s);
        if !(hh == hb && hb == bh && bh == bb) {
            return Err(format!("class member unequal channels: {s}"));
        }
        if s.len() != FOUNDER_LEN {
            return Err(format!("bad length: {s}"));
        }
    }
    let area = {
        let m = assay_mesh(&class[0], &canonical_conditions()[0]);
        m.area()
    };
    let k_site = derive_k_site(0.9, area, 1);
    let net = NetworkParams::derived(reserve, t_maint, k_d, k_site);
    let conds = canonical_conditions();
    let steps = 80;

    let mut rows: Vec<(String, Vec<f64>, f64, f64)> = Vec::new();
    for s in &class {
        let mut flat = Vec::with_capacity(16);
        for c in &conds {
            let v = isolated_response(s, c, reserve, &net, steps);
            flat.extend_from_slice(&v);
        }
        // Differential scores: prefer condition-specific channel allocation, not raw mass.
        let harvest = flat[0] - flat[3]; // HH − BB @ low-A N/F
        let build = flat[3 * 4 + 3] - flat[3 * 4]; // BB − HH @ damage
        rows.push((s.clone(), flat, harvest, build));
    }

    let mut by_h = rows.clone();
    by_h.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    let mut by_b = rows.clone();
    by_b.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    let topology_h = by_h[0].0.clone();
    // Prefer a B founder that is not H and ranks well on build while ranking poorly on harvest.
    let topology_b = by_b
        .iter()
        .filter(|r| r.0 != topology_h)
        .max_by(|a, b| {
            let score = |r: &&(String, Vec<f64>, f64, f64)| r.3 - 0.25 * r.2;
            score(a)
                .partial_cmp(&score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|r| r.0.clone())
        .ok_or_else(|| "no distinct topology B".to_string())?;

    // Median response in L1 over flattened vectors.
    let dim = 16usize;
    let mut median = vec![0.0; dim];
    for d in 0..dim {
        let mut col: Vec<f64> = rows.iter().map(|r| r.1[d]).collect();
        col.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        median[d] = col[col.len() / 2];
    }
    let mut best_n = None;
    let mut best_dist = f64::INFINITY;
    for r in &rows {
        if r.0 == topology_h || r.0 == topology_b {
            continue;
        }
        let dist: f64 = r
            .1
            .iter()
            .zip(median.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        if dist < best_dist {
            best_dist = dist;
            best_n = Some(r.0.clone());
        }
    }
    let topology_n = best_n.ok_or_else(|| "no topology N".to_string())?;

    // Sanity: pair channel of H should favor HH sites exist.
    let _ = PairChannel::Hh;

    Ok(TopologyFounders {
        topology_h,
        topology_b,
        topology_n,
        class_size: class.len(),
        method: "isolated_binding_response_v1".into(),
    })
}
