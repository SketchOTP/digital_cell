//! D-092 minimal catalytic template polymer — physical hereditary substrate.
//!
//! Sequence information lives only in bonded monomer order on physical chains.
//! No organism-level genome object; fission never reads sequence.

use crate::material_mesh::{
    MaterialMesh, MonomerKind, TemplateChain, EQUATION_VERSION_MATERIAL_MESH,
    MATERIAL_MESH_SCHEMA_VERSION,
};
use crate::mesh_reactions::{q_catalyst, ReactionParams};
use crate::metabolic_reserve::EQUATION_VERSION_METABOLIC_RESERVE;
use serde::{Deserialize, Serialize};

pub const EQUATION_VERSION_CATALYTIC_TEMPLATE: &str =
    "autopoietic_material_mesh_catalytic_template_v1";
pub const FIELD_SCHEMA_CATALYTIC_TEMPLATE: &str = "mesh_vertices_edges_reserve_template_polymer_v1";
pub const TEMPLATE_SCHEMA_VERSION: u32 = MATERIAL_MESH_SCHEMA_VERSION + 2;

pub const FOUNDER_LEN: usize = 12;
pub const FOUNDER_HARVEST: &str = "HHBHHBHHBBBB";
pub const FOUNDER_BUILD: &str = "BBHBBHBBHHHH";
pub const FOUNDER_NEUTRAL: &str = "HBHBHBHBHBHB";
/// Mass of one template monomer / ligation quantum (affordability + conservation scale).
pub const MONOMER_MASS: f64 = 0.05;

const EPS: f64 = 1e-15;

// MonomerKind / TemplateChain live in material_mesh to avoid module cycles.

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemplateParams {
    pub enable: bool,
    /// Equal baseline monomer production rate: N+A → U_H/U_B + W.
    pub k_mono: f64,
    /// Match association / dissociation rates.
    pub k_on_match: f64,
    pub k_off: f64,
    /// Match/mismatch affinity ratio (directive: 100).
    pub match_mismatch_ratio: f64,
    /// Backbone ligation rate (requires A + catalyst).
    pub k_ligate: f64,
    /// Slow backbone hydrolysis rate (per bond).
    pub k_hydrolysis: f64,
    /// Motif–catalyst binding / unbinding.
    pub k_bind_motif: f64,
    pub k_unbind_motif: f64,
    /// When false, association ignores mismatches (mutation-off).
    pub allow_mismatch: bool,
    /// When false, no association/ligation (copying knockout).
    pub enable_copying: bool,
    /// When false, hydrolysis off (turnover knockout).
    pub enable_turnover: bool,
    /// When false, motif binding off (expression knockout).
    pub enable_binding: bool,
    /// Complex efficiencies (fixed architecture).
    pub eff_kh_harvest: f64,
    pub eff_kh_build: f64,
    pub eff_kb_harvest: f64,
    pub eff_kb_build: f64,
    /// Next chain id counter seed for founders (observer).
    pub next_id_hint: u64,
}

impl Default for TemplateParams {
    fn default() -> Self {
        Self {
            enable: false,
            k_mono: 0.0,
            k_on_match: 0.0,
            k_off: 0.0,
            match_mismatch_ratio: 100.0,
            k_ligate: 0.0,
            k_hydrolysis: 0.0,
            k_bind_motif: 0.0,
            k_unbind_motif: 0.0,
            allow_mismatch: true,
            enable_copying: true,
            enable_turnover: true,
            enable_binding: true,
            eff_kh_harvest: 1.5,
            eff_kh_build: 0.5,
            eff_kb_harvest: 0.5,
            eff_kb_build: 1.5,
            next_id_hint: 1,
        }
    }
}

impl TemplateParams {
    /// Derive rates from D-088 generation horizon.
    /// mean complete copy ≈ 0.25 gen; template half-life ≈ 3 gen.
    pub fn derived(t_gen: f64) -> Self {
        let t_gen = t_gen.max(1.0);
        let t_copy = 0.25 * t_gen;
        let t_half = 3.0 * t_gen;
        // Hydrolysis of FOUNDER_LEN-1 backbone bonds ≈ half-life of complete chain.
        let bonds = (FOUNDER_LEN - 1) as f64;
        // Association fills sites; ligation finishes. Match/mismatch = 100.
        let k_on_match = 6.0 / t_copy.max(EPS);
        let k_off = k_on_match / 40.0;
        let k_ligate = 12.0 / t_copy.max(EPS);
        // Strong monomer bottleneck: keeps complete-template count O(10–30).
        let k_mono = 0.008;
        // Faster turnover so copy/death can balance under monomer limitation
        // (ponytail: material carrying capacity; ceiling = unbounded if k_mono raised).
        let k_hydrolysis = (std::f64::consts::LN_2 / (t_half * bonds.max(1.0))) * 3.0;
        Self {
            enable: true,
            k_mono,
            k_on_match,
            k_off,
            match_mismatch_ratio: 100.0,
            k_ligate,
            k_hydrolysis,
            k_bind_motif: 2.0,
            k_unbind_motif: 0.2,
            allow_mismatch: true,
            enable_copying: true,
            enable_turnover: true,
            enable_binding: true,
            eff_kh_harvest: 1.5,
            eff_kh_build: 0.5,
            eff_kb_harvest: 0.5,
            eff_kb_build: 1.5,
            next_id_hint: 1,
        }
    }

    pub fn with_baseline_efficiencies(mut self) -> Self {
        self.eff_kh_harvest = 1.0;
        self.eff_kh_build = 1.0;
        self.eff_kb_harvest = 1.0;
        self.eff_kb_build = 1.0;
        self
    }

    pub fn candidate_identity_suffix(&self) -> String {
        format!(
            "tmpl:k_mono={:.6e}:k_on={:.6e}:k_off={:.6e}:ratio={:.1}:k_lig={:.6e}:k_hyd={:.6e}:k_bm={:.6e}:k_um={:.6e}:mm={}:copy={}:turn={}:bind={}:eKH={:.2}/{:.2}:eKB={:.2}/{:.2}",
            self.k_mono,
            self.k_on_match,
            self.k_off,
            self.match_mismatch_ratio,
            self.k_ligate,
            self.k_hydrolysis,
            self.k_bind_motif,
            self.k_unbind_motif,
            self.allow_mismatch as u8,
            self.enable_copying as u8,
            self.enable_turnover as u8,
            self.enable_binding as u8,
            self.eff_kh_harvest,
            self.eff_kh_build,
            self.eff_kb_harvest,
            self.eff_kb_build
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateLedger {
    pub u_h_produced: f64,
    pub u_b_produced: f64,
    pub n_consumed_mono: f64,
    pub a_consumed_mono: f64,
    pub a_consumed_ligation: f64,
    pub w_produced: f64,
    pub match_binds: u64,
    pub mismatch_binds: u64,
    pub dissociations: u64,
    pub ligations: u64,
    pub complete_copies: u64,
    pub hydrolysis_bonds: u64,
    pub monomers_from_hydrolysis: f64,
    pub rejected_steps: u64,
    pub k_h_bound: f64,
    pub k_b_bound: f64,
}

pub fn stamp_template_equation(mesh: &mut MaterialMesh) {
    mesh.equation_id = EQUATION_VERSION_CATALYTIC_TEMPLATE.to_string();
    mesh.schema_version = TEMPLATE_SCHEMA_VERSION;
}

pub fn stamp_reserve_equation_id(mesh: &mut MaterialMesh) {
    mesh.equation_id = EQUATION_VERSION_METABOLIC_RESERVE.to_string();
    mesh.schema_version = MATERIAL_MESH_SCHEMA_VERSION + 1;
}

pub fn stamp_base_equation(mesh: &mut MaterialMesh) {
    mesh.equation_id = EQUATION_VERSION_MATERIAL_MESH.to_string();
    mesh.schema_version = MATERIAL_MESH_SCHEMA_VERSION;
}

/// Fail-closed: template chemistry requires the template schema stamp.
pub fn template_schema_load_ok(mesh: &MaterialMesh, tmpl: &TemplateParams) -> bool {
    if !tmpl.enable {
        return true;
    }
    mesh.equation_id == EQUATION_VERSION_CATALYTIC_TEMPLATE
}

/// Old reserve snapshots must not run under template chemistry.
pub fn reserve_snapshot_rejected_under_template(mesh: &MaterialMesh) -> bool {
    mesh.equation_id == EQUATION_VERSION_METABOLIC_RESERVE
        || mesh.equation_id == EQUATION_VERSION_MATERIAL_MESH
}

pub fn parse_founder(seq: &str) -> Option<Vec<MonomerKind>> {
    let v: Option<Vec<_>> = seq.chars().map(MonomerKind::from_char).collect();
    let v = v?;
    if v.len() != FOUNDER_LEN {
        return None;
    }
    let h = v.iter().filter(|m| **m == MonomerKind::H).count();
    let b = v.iter().filter(|m| **m == MonomerKind::B).count();
    if h != 6 || b != 6 {
        return None;
    }
    Some(v)
}

pub fn seed_founder_chains(
    mesh: &mut MaterialMesh,
    seq: &str,
    count: usize,
    start_id: u64,
) -> u64 {
    let Some(monomers) = parse_founder(seq) else {
        return start_id;
    };
    let c = mesh.centroid();
    let mut id = start_id;
    for k in 0..count {
        let ang = 2.0 * std::f64::consts::PI * (k as f64) / (count.max(1) as f64);
        let r = 0.35 * mesh.vertices[0][0].hypot(mesh.vertices[0][1] - c[1]).max(1.0);
        // Place inside roughly near centroid.
        let pos = [c[0] + 0.4 * r * ang.cos(), c[1] + 0.4 * r * ang.sin()];
        let mut chain = TemplateChain {
            id,
            parent_id: None,
            pos,
            orientation: ang,
            monomers: monomers.clone(),
            backbone: vec![true; FOUNDER_LEN - 1],
            paired: vec![None; FOUNDER_LEN],
            nascent_backbone: vec![false; FOUNDER_LEN - 1],
            complete: true,
        };
        chain.refresh_complete();
        mesh.templates.push(chain);
        id += 1;
    }
    id
}

pub fn count_complete_templates(mesh: &MaterialMesh) -> usize {
    mesh.templates.iter().filter(|t| t.is_complete_template()).count()
}

pub fn total_template_monomers(mesh: &MaterialMesh) -> f64 {
    let mut n = 0.0;
    for t in &mesh.templates {
        n += t.monomers.len() as f64;
        for p in &t.paired {
            if p.is_some() {
                n += 1.0;
            }
        }
    }
    n + mesh.interior.u_h.max(0.0) * mesh.area() + mesh.interior.u_b.max(0.0) * mesh.area()
}

/// Free-monomer production: equal baseline N+A → U_H/U_B + W (requires catalyst).
pub fn monomer_production_step(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    dt: f64,
) -> TemplateLedger {
    let mut led = TemplateLedger::default();
    let p = &react.template;
    if !p.enable || !template_schema_load_ok(mesh, p) {
        if p.enable {
            led.rejected_steps += 1;
        }
        return led;
    }
    let area = mesh.area().max(EPS);
    let qc = q_catalyst(mesh.interior.c, react.q_c);
    if qc <= 0.0 || p.k_mono <= 0.0 {
        return led;
    }
    // Equal split: half to U_H, half to U_B.
    let j = p.k_mono * qc * mesh.interior.n.max(0.0) * mesh.interior.a.max(0.0) * dt * area;
    let each = 0.5 * j;
    let n_have = mesh.interior.n.max(0.0) * area;
    let a_have = mesh.interior.a.max(0.0) * area;
    let take = each.min(n_have * 0.5).min(a_have * 0.5).max(0.0);
    if take <= 0.0 {
        return led;
    }
    // Two equal channels consume 2*take of N and A total.
    let total = 2.0 * take;
    if total > n_have || total > a_have {
        led.rejected_steps += 1;
        return led;
    }
    mesh.interior.n = (mesh.interior.n - total / area).max(0.0);
    mesh.interior.a = (mesh.interior.a - total / area).max(0.0);
    mesh.interior.u_h += take / area;
    mesh.interior.u_b += take / area;
    mesh.interior.w += total / area;
    led.u_h_produced += take;
    led.u_b_produced += take;
    led.n_consumed_mono += total;
    led.a_consumed_mono += total;
    led.w_produced += total;
    led
}

/// Slow backbone hydrolysis: T → free monomers + W by identity.
pub fn hydrolysis_step(
    mesh: &mut MaterialMesh,
    react: &ReactionParams,
    dt: f64,
    rng: &mut impl RngLike,
) -> TemplateLedger {
    let mut led = TemplateLedger::default();
    let p = &react.template;
    if !p.enable || !p.enable_turnover || !template_schema_load_ok(mesh, p) {
        if p.enable && !template_schema_load_ok(mesh, p) {
            led.rejected_steps += 1;
        }
        return led;
    }
    if p.k_hydrolysis <= 0.0 {
        return led;
    }
    let area = mesh.area().max(EPS);
    let p_break = 1.0 - (-p.k_hydrolysis * dt).exp();

    // Decide breaks without holding conflicting borrows.
    let mut breaks: Vec<(usize, usize)> = Vec::new();
    for (ci, chain) in mesh.templates.iter().enumerate() {
        for (b, bonded) in chain.backbone.iter().enumerate() {
            if *bonded && rng.unit() < p_break {
                breaks.push((ci, b));
                break;
            }
        }
    }
    // Apply from highest index so removals stay valid.
    breaks.sort_by(|a, b| b.0.cmp(&a.0));
    for (ci, b) in breaks {
        if ci >= mesh.templates.len() {
            continue;
        }
        let chain = mesh.templates.remove(ci);
        led.hydrolysis_bonds += 1;
        let mut released_h = 0.0;
        let mut released_b = 0.0;
        for slot in &chain.paired {
            if let Some(m) = slot {
                match m {
                    MonomerKind::H => released_h += 1.0,
                    MonomerKind::B => released_b += 1.0,
                }
            }
        }
        let left_m = chain.monomers[..=b].to_vec();
        let right_m = chain.monomers[b + 1..].to_vec();
        let left_bb: Vec<bool> = chain.backbone[..b].to_vec();
        let right_bb: Vec<bool> = chain.backbone[b + 1..].to_vec();
        let mut frags = Vec::new();
        for (mons, bbs) in [(left_m, left_bb), (right_m, right_bb)] {
            if mons.is_empty() {
                continue;
            }
            if mons.len() == 1 {
                match mons[0] {
                    MonomerKind::H => released_h += 1.0,
                    MonomerKind::B => released_b += 1.0,
                }
            } else {
                let n = mons.len();
                let mut frag = TemplateChain {
                    id: chain.id,
                    parent_id: chain.parent_id,
                    pos: chain.pos,
                    orientation: chain.orientation,
                    monomers: mons,
                    backbone: bbs,
                    paired: vec![None; n],
                    nascent_backbone: vec![false; n.saturating_sub(1)],
                    complete: false,
                };
                frag.refresh_complete();
                frags.push(frag);
            }
        }
        mesh.interior.u_h += released_h / area;
        mesh.interior.u_b += released_b / area;
        mesh.interior.w += MONOMER_MASS / area;
        led.monomers_from_hydrolysis += released_h + released_b;
        led.w_produced += MONOMER_MASS;
        for frag in frags.into_iter().rev() {
            mesh.templates.insert(ci, frag);
        }
    }
    led
}

/// Tiny RNG trait to avoid pulling rand into chemistry-core if unused elsewhere.
pub trait RngLike {
    fn unit(&mut self) -> f64;
}

/// Deterministic xorshift64* for reproducible gate campaigns.
#[derive(Debug, Clone)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.max(1),
        }
    }

    pub fn state(&self) -> u64 {
        self.state
    }
}

impl RngLike for XorShift64 {
    fn unit(&mut self) -> f64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x as f64) / (u64::MAX as f64)
    }
}

pub fn merge_template_ledgers(a: &mut TemplateLedger, b: &TemplateLedger) {
    a.u_h_produced += b.u_h_produced;
    a.u_b_produced += b.u_b_produced;
    a.n_consumed_mono += b.n_consumed_mono;
    a.a_consumed_mono += b.a_consumed_mono;
    a.a_consumed_ligation += b.a_consumed_ligation;
    a.w_produced += b.w_produced;
    a.match_binds += b.match_binds;
    a.mismatch_binds += b.mismatch_binds;
    a.dissociations += b.dissociations;
    a.ligations += b.ligations;
    a.complete_copies += b.complete_copies;
    a.hydrolysis_bonds += b.hydrolysis_bonds;
    a.monomers_from_hydrolysis += b.monomers_from_hydrolysis;
    a.rejected_steps += b.rejected_steps;
    a.k_h_bound = b.k_h_bound;
    a.k_b_bound = b.k_b_bound;
}
