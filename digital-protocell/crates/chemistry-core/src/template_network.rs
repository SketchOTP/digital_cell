//! D-093 template-encoded catalytic network — schema and parameters.
//!
//! Overlapping pair sites on physical templates bind catalyst locally.
//! Sequence phenotype arises from pair ordering, overlap exclusion, and
//! finite-catalyst competition — not from total H/B content or motif tables.

use crate::material_mesh::{MaterialMesh, MATERIAL_MESH_SCHEMA_VERSION};
use crate::metabolic_reserve::ReserveParams;
use crate::template_polymer::{EQUATION_VERSION_CATALYTIC_TEMPLATE, FOUNDER_LEN};
use serde::{Deserialize, Serialize};

pub const EQUATION_VERSION_TEMPLATE_NETWORK: &str =
    "autopoietic_material_mesh_template_network_v1";
pub const FIELD_SCHEMA_TEMPLATE_NETWORK: &str =
    "mesh_vertices_edges_reserve_template_network_v1";
pub const TEMPLATE_NETWORK_SCHEMA_VERSION: u32 = MATERIAL_MESH_SCHEMA_VERSION + 3;

/// Frozen expression boost for bound channel catalyst.
pub const RHO_NETWORK: f64 = 1.5;

const EPS: f64 = 1e-15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairChannel {
    Hh,
    Hb,
    Bh,
    Bb,
}

impl PairChannel {
    pub fn from_monomers(a: crate::material_mesh::MonomerKind, b: crate::material_mesh::MonomerKind) -> Self {
        use crate::material_mesh::MonomerKind::*;
        match (a, b) {
            (H, H) => Self::Hh,
            (H, B) => Self::Hb,
            (B, H) => Self::Bh,
            (B, B) => Self::Bb,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hh => "HH",
            Self::Hb => "HB",
            Self::Bh => "BH",
            Self::Bb => "BB",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NetworkParams {
    pub enable: bool,
    pub k_on: f64,
    pub k_off: f64,
    /// Max bound catalyst mass per pair site.
    pub k_site: f64,
    pub rho: f64,
    /// Gate constants (from sealed D-091 / local demand).
    pub k_store: f64,
    pub k_low: f64,
    pub k_r: f64,
    pub k_growth: f64,
    pub k_d: f64,
    pub r_max: f64,
    pub w_s: f64,
    pub w_m: f64,
    /// Channel knockout masks (all true = full network).
    pub enable_hh: bool,
    pub enable_hb: bool,
    pub enable_bh: bool,
    pub enable_bb: bool,
}

impl Default for NetworkParams {
    fn default() -> Self {
        Self {
            enable: false,
            k_on: 0.0,
            k_off: 0.0,
            k_site: 1.0,
            rho: RHO_NETWORK,
            k_store: 0.5,
            k_low: 0.25,
            k_r: 0.25,
            k_growth: 0.5,
            k_d: 0.2,
            r_max: 2.0,
            w_s: 0.6,
            w_m: 0.4,
            enable_hh: true,
            enable_hb: true,
            enable_bh: true,
            enable_bb: true,
        }
    }
}

impl NetworkParams {
    /// Derive one global parameter set. Selection results must not influence this.
    pub fn derived(reserve: &ReserveParams, t_maint: f64, k_d: f64, k_site: f64) -> Self {
        let t_maint = t_maint.max(1.0);
        // Median site half-occupation in 0.25 × maintenance horizon under activating gate≈1.
        let k_on = (std::f64::consts::LN_2) / (0.25 * t_maint).max(EPS);
        // Median complex residence = 0.5 × maintenance horizon.
        let k_off = (std::f64::consts::LN_2) / (0.5 * t_maint).max(EPS);
        Self {
            enable: true,
            k_on,
            k_off,
            k_site: k_site.max(EPS),
            rho: RHO_NETWORK,
            k_store: reserve.k_store_half,
            k_low: reserve.k_low,
            k_r: reserve.k_r,
            k_growth: reserve.k_growth,
            k_d: k_d.max(EPS),
            r_max: reserve.r_max,
            w_s: 0.6,
            w_m: 0.4,
            enable_hh: true,
            enable_hb: true,
            enable_bh: true,
            enable_bb: true,
        }
    }

    pub fn with_binding_off(mut self) -> Self {
        self.k_on = 0.0;
        self
    }

    pub fn channel_enabled(self, ch: PairChannel) -> bool {
        match ch {
            PairChannel::Hh => self.enable_hh,
            PairChannel::Hb => self.enable_hb,
            PairChannel::Bh => self.enable_bh,
            PairChannel::Bb => self.enable_bb,
        }
    }

    pub fn candidate_identity_suffix(&self) -> String {
        format!(
            "net:k_on={:.6e}:k_off={:.6e}:k_site={:.6e}:rho={:.3}:Ks={:.6}:Kl={:.6}:Kr={:.6}:Kg={:.6}:Kd={:.6}:ch={}/{}/{}/{}",
            self.k_on,
            self.k_off,
            self.k_site,
            self.rho,
            self.k_store,
            self.k_low,
            self.k_r,
            self.k_growth,
            self.k_d,
            self.enable_hh as u8,
            self.enable_hb as u8,
            self.enable_bh as u8,
            self.enable_bb as u8
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkLedger {
    pub bind_mass: f64,
    pub unbind_mass: f64,
    pub rejected_steps: u64,
    pub occupancy_violations: u64,
    pub k_hh: f64,
    pub k_hb: f64,
    pub k_bh: f64,
    pub k_bb: f64,
}

pub fn stamp_network_equation(mesh: &mut MaterialMesh) {
    mesh.equation_id = EQUATION_VERSION_TEMPLATE_NETWORK.to_string();
    mesh.schema_version = TEMPLATE_NETWORK_SCHEMA_VERSION;
}

/// Fail-closed: network chemistry requires the network schema stamp.
pub fn network_schema_load_ok(mesh: &MaterialMesh, net: &NetworkParams) -> bool {
    if !net.enable {
        return true;
    }
    mesh.equation_id == EQUATION_VERSION_TEMPLATE_NETWORK
}

/// Polymer (copying/hydrolysis/monomers) may run under D-092 or D-093 stamps.
pub fn polymer_schema_load_ok(mesh: &MaterialMesh, enable: bool) -> bool {
    if !enable {
        return true;
    }
    mesh.equation_id == EQUATION_VERSION_CATALYTIC_TEMPLATE
        || mesh.equation_id == EQUATION_VERSION_TEMPLATE_NETWORK
}

/// Circular overlapping pair sites: L sites for length-L complete templates.
/// (Linear L−1 cannot host equal HH/HB/BH/BB counts; circular L=12 yields 3 each.)
pub fn pair_sites_for_sequence(seq: &str) -> Vec<PairChannel> {
    let chars: Vec<char> = seq.chars().collect();
    let n = chars.len();
    if n < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let a = crate::material_mesh::MonomerKind::from_char(chars[i]);
        let b = crate::material_mesh::MonomerKind::from_char(chars[(i + 1) % n]);
        if let (Some(a), Some(b)) = (a, b) {
            out.push(PairChannel::from_monomers(a, b));
        }
    }
    out
}

pub fn count_pair_channels(seq: &str) -> (usize, usize, usize, usize) {
    let mut hh = 0;
    let mut hb = 0;
    let mut bh = 0;
    let mut bb = 0;
    for ch in pair_sites_for_sequence(seq) {
        match ch {
            PairChannel::Hh => hh += 1,
            PairChannel::Hb => hb += 1,
            PairChannel::Bh => bh += 1,
            PairChannel::Bb => bb += 1,
        }
    }
    (hh, hb, bh, bb)
}

/// Lawful catalyst-per-template site budget from typical interior C and founder count.
pub fn derive_k_site(c_typ: f64, area: f64, n_templates: usize) -> f64 {
    let sites = FOUNDER_LEN.max(1) as f64; // circular sites
    let n = n_templates.max(1) as f64;
    (c_typ.max(0.0) * area.max(EPS) / (n * sites)).max(EPS)
}

/// Total bound catalyst mass on all templates.
pub fn total_bound_catalyst_mass(mesh: &MaterialMesh) -> f64 {
    mesh.templates
        .iter()
        .map(|t| t.site_k.iter().map(|k| k.max(0.0)).sum::<f64>())
        .sum()
}

/// Free catalyst concentration: C_total − bound/area.
pub fn c_free(mesh: &MaterialMesh) -> f64 {
    let area = mesh.area().max(EPS);
    let bound = total_bound_catalyst_mass(mesh);
    (mesh.interior.c.max(0.0) - bound / area).max(0.0)
}

/// Ensure every chain has a lawful site_k length.
pub fn ensure_all_site_k(mesh: &mut MaterialMesh) {
    for t in &mut mesh.templates {
        t.ensure_site_k();
    }
}

/// Apply equal turnover fraction to free and bound catalyst after total-C turnover.
pub fn scale_bound_catalyst(mesh: &mut MaterialMesh, retain: f64) {
    let r = retain.clamp(0.0, 1.0);
    for t in &mut mesh.templates {
        for k in &mut t.site_k {
            *k *= r;
        }
    }
}
