//! D-094 catalytic node materials K_A / K_R / K_B and precursors Q_K.
//!
//! Nodes supplement baseline free catalyst C; they do not replace it.

use crate::material_mesh::{LumpedChem, MaterialMesh, MATERIAL_MESH_SCHEMA_VERSION};
use crate::mesh_reactions::q_catalyst;
use serde::{Deserialize, Serialize};

pub const EQUATION_VERSION_AUTOCATALYTIC_SET: &str =
    "autopoietic_material_mesh_autocatalytic_set_v1";
pub const FIELD_SCHEMA_AUTOCATALYTIC_SET: &str =
    "mesh_vertices_edges_reserve_autocatalytic_network_v1";
pub const AUTOCATALYTIC_SET_SCHEMA_VERSION: u32 = MATERIAL_MESH_SCHEMA_VERSION + 4;

/// Frozen edge-copying mismatch probability (D-093 measured fidelity).
pub const MU_E: f64 = 0.0089;

const EPS: f64 = 1e-15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    A,
    R,
    B,
}

impl NodeKind {
    pub fn all() -> [NodeKind; 3] {
        [NodeKind::A, NodeKind::R, NodeKind::B]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::R => "R",
            Self::B => "B",
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'A' | 'a' => Some(Self::A),
            'R' | 'r' => Some(Self::R),
            'B' | 'b' => Some(Self::B),
            _ => None,
        }
    }

    pub fn other_targets(self) -> [NodeKind; 2] {
        match self {
            Self::A => [Self::R, Self::B],
            Self::R => [Self::A, Self::B],
            Self::B => [Self::A, Self::R],
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AutocatalyticParams {
    pub enable: bool,
    /// Node production rate per edge: E + Q_K + A → E + K_j + W
    pub k_node_prod: f64,
    /// Node turnover: K_j → Q_K + W
    pub k_node_turn: f64,
    /// Edge copying: K_i + E + Q_E + A → K_i + 2E + W
    pub k_edge_copy: f64,
    /// Edge material hydrolysis / loss (bounded decay of orphan edges).
    pub k_edge_loss: f64,
    /// Expression boost of free node catalyst relative to baseline C.
    pub rho_node: f64,
    /// Copying mismatch probability (frozen μ_E).
    pub mu_e: f64,
    pub enable_node_prod: bool,
    pub enable_edge_copy: bool,
    pub enable_ka: bool,
    pub enable_kr: bool,
    pub enable_kb: bool,
}

impl Default for AutocatalyticParams {
    fn default() -> Self {
        Self {
            enable: false,
            k_node_prod: 0.0,
            k_node_turn: 0.0,
            k_edge_copy: 0.0,
            k_edge_loss: 0.0,
            rho_node: 1.0,
            mu_e: MU_E,
            enable_node_prod: true,
            enable_edge_copy: true,
            enable_ka: true,
            enable_kr: true,
            enable_kb: true,
        }
    }
}

impl AutocatalyticParams {
    /// Derive rates from maintenance horizon. Selection must not influence this.
    pub fn derived(t_maint: f64) -> Self {
        let t = t_maint.max(1.0);
        Self {
            enable: true,
            // Modest rates — keep A affordable for reserve-funded growth/fission.
            k_node_prod: (std::f64::consts::LN_2) / (1.0 * t),
            k_node_turn: (std::f64::consts::LN_2) / (2.0 * t),
            k_edge_copy: (std::f64::consts::LN_2) / (1.5 * t),
            k_edge_loss: (std::f64::consts::LN_2) / (5.0 * t),
            rho_node: 0.8,
            mu_e: MU_E,
            enable_node_prod: true,
            enable_edge_copy: true,
            enable_ka: true,
            enable_kr: true,
            enable_kb: true,
        }
    }

    pub fn with_mutation_off(mut self) -> Self {
        self.mu_e = 0.0;
        self
    }

    pub fn with_node_prod_off(mut self) -> Self {
        self.enable_node_prod = false;
        self
    }

    pub fn with_edge_copy_off(mut self) -> Self {
        self.enable_edge_copy = false;
        self
    }

    pub fn with_baseline_efficiencies(mut self) -> Self {
        self.rho_node = 0.0;
        self
    }

    pub fn candidate_identity_suffix(&self) -> String {
        format!(
            "acs:k_np={:.6e}:k_nt={:.6e}:k_ec={:.6e}:k_el={:.6e}:rho={:.3}:mu={:.4}:flags={}/{}/{}",
            self.k_node_prod,
            self.k_node_turn,
            self.k_edge_copy,
            self.k_edge_loss,
            self.rho_node,
            self.mu_e,
            self.enable_ka as u8,
            self.enable_kr as u8,
            self.enable_kb as u8
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutocatalyticLedger {
    pub node_produced: f64,
    pub node_turned: f64,
    pub edge_copied: f64,
    pub edge_mutated: f64,
    pub edge_lost: f64,
    pub a_consumed: f64,
    pub w_produced: f64,
    pub q_k_consumed: f64,
    pub q_e_consumed: f64,
    pub rejected_steps: u64,
}

pub fn stamp_autocatalytic_equation(mesh: &mut MaterialMesh) {
    mesh.equation_id = EQUATION_VERSION_AUTOCATALYTIC_SET.to_string();
    mesh.schema_version = AUTOCATALYTIC_SET_SCHEMA_VERSION;
}

pub fn autocatalytic_schema_load_ok(mesh: &MaterialMesh, p: &AutocatalyticParams) -> bool {
    if !p.enable {
        return true;
    }
    mesh.equation_id == EQUATION_VERSION_AUTOCATALYTIC_SET
}

pub fn node_conc(chem: &LumpedChem, kind: NodeKind) -> f64 {
    match kind {
        NodeKind::A => chem.k_a.max(0.0),
        NodeKind::R => chem.k_r.max(0.0),
        // D-094 building node; distinct from D-092 motif complex `k_b`.
        NodeKind::B => chem.k_node_b.max(0.0),
    }
}

pub fn set_node_conc(chem: &mut LumpedChem, kind: NodeKind, v: f64) {
    match kind {
        NodeKind::A => chem.k_a = v.max(0.0),
        NodeKind::R => chem.k_r = v.max(0.0),
        NodeKind::B => chem.k_node_b = v.max(0.0),
    }
}

pub fn add_node_conc(chem: &mut LumpedChem, kind: NodeKind, dv: f64) {
    set_node_conc(chem, kind, node_conc(chem, kind) + dv);
}

pub fn total_node_conc(chem: &LumpedChem) -> f64 {
    chem.k_a.max(0.0) + chem.k_r.max(0.0) + chem.k_node_b.max(0.0)
}

/// Multiplicative gain: q(C + ρ K_channel) / q(C).
pub fn node_channel_gain(
    mesh: &MaterialMesh,
    p: &AutocatalyticParams,
    q_c: f64,
    kind: NodeKind,
) -> f64 {
    if !p.enable || p.rho_node <= 0.0 {
        return 1.0;
    }
    let enabled = match kind {
        NodeKind::A => p.enable_ka,
        NodeKind::R => p.enable_kr,
        NodeKind::B => p.enable_kb,
    };
    if !enabled {
        return 1.0;
    }
    let c = mesh.interior.c.max(0.0);
    let k = node_conc(&mesh.interior, kind);
    let q_base = q_catalyst(c, q_c).max(EPS);
    let q_ch = q_catalyst(c + p.rho_node * k, q_c);
    (q_ch / q_base).max(0.0)
}

pub fn node_activation_gain(mesh: &MaterialMesh, p: &AutocatalyticParams, q_c: f64) -> f64 {
    node_channel_gain(mesh, p, q_c, NodeKind::A)
}

pub fn node_storage_release_gain(mesh: &MaterialMesh, p: &AutocatalyticParams, q_c: f64) -> f64 {
    node_channel_gain(mesh, p, q_c, NodeKind::R)
}

pub fn node_building_gain(mesh: &MaterialMesh, p: &AutocatalyticParams, q_c: f64) -> f64 {
    node_channel_gain(mesh, p, q_c, NodeKind::B)
}
