//! D-086 autopoietic material mesh — closed polygonal body substrate.
//!
//! The mesh **is** the organism body. No φ / Cahn–Hilliard coupling.
//! Legacy phase-field equation versions remain unchanged.

use serde::{Deserialize, Serialize};

pub const EQUATION_VERSION_MATERIAL_MESH: &str = "autopoietic_material_mesh_v1";
pub const FIELD_SCHEMA_MATERIAL_MESH: &str = "mesh_vertices_edges_v1";
pub const MATERIAL_MESH_SCHEMA_VERSION: u32 = 1;

/// Template monomer identity (D-092). Sequence = ordered bonded monomers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonomerKind {
    H,
    B,
}

impl MonomerKind {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'H' | 'h' => Some(Self::H),
            'B' | 'b' => Some(Self::B),
            _ => None,
        }
    }

    pub fn as_char(self) -> char {
        match self {
            Self::H => 'H',
            Self::B => 'B',
        }
    }
}

/// Physical template chain: hereditary information in bond order (D-092).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateChain {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub pos: [f64; 2],
    pub orientation: f64,
    pub monomers: Vec<MonomerKind>,
    pub backbone: Vec<bool>,
    pub paired: Vec<Option<MonomerKind>>,
    pub nascent_backbone: Vec<bool>,
    pub complete: bool,
    /// Bound catalyst mass at circular overlapping pair sites (D-093).
    /// Complete L=12 templates use length L (wrap-around bond). Empty under D-092.
    /// Never an organism-level genome weight.
    #[serde(default)]
    pub site_k: Vec<f64>,
}

impl TemplateChain {
    pub fn sequence_string(&self) -> String {
        self.monomers.iter().map(|m| m.as_char()).collect()
    }

    pub fn is_complete_template(&self) -> bool {
        const L: usize = 12;
        self.complete
            && self.monomers.len() == L
            && self.backbone.len() + 1 == self.monomers.len()
            && self.backbone.iter().all(|&b| b)
    }

    pub fn refresh_complete(&mut self) {
        const L: usize = 12;
        self.complete = self.monomers.len() == L
            && !self.monomers.is_empty()
            && self.backbone.len() + 1 == self.monomers.len()
            && self.backbone.iter().all(|&b| b);
    }

    /// Ensure circular pair-site catalyst vector (one site per monomer on complete chains).
    pub fn ensure_site_k(&mut self) {
        let n = if self.is_complete_template() {
            self.monomers.len()
        } else {
            self.monomers.len().saturating_sub(1)
        };
        if self.site_k.len() != n {
            self.site_k.resize(n, 0.0);
        }
    }
}

/// Global structural line density ρ_s: ℓ⁰ = m / ρ_s.
pub const DEFAULT_RHO_S: f64 = 1.0;
pub const DEFAULT_B_MAX_PER_LENGTH: f64 = 1.0;
pub const DEFAULT_BOND_THRESHOLD: f64 = 0.05;
pub const DEFAULT_L_MAX: f64 = 3.5;
pub const DEFAULT_L_MIN: f64 = 0.6;
pub const DEFAULT_REBOND_DIST: f64 = 1.2;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MeshEdge {
    /// Structural material mass on this edge.
    pub m: f64,
    /// Bound membrane material.
    pub b: f64,
    /// Observer-only structural tracer.
    pub tracer_m: f64,
    /// Observer-only membrane tracer.
    pub tracer_b: f64,
    /// True when structural mass fell below bond threshold (connection ruptured).
    pub ruptured: bool,
}

impl Default for MeshEdge {
    fn default() -> Self {
        Self {
            m: 0.0,
            b: 0.0,
            tracer_m: 0.0,
            tracer_b: 0.0,
            ruptured: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct LumpedChem {
    /// Total catalyst C = C_H + C_B (scalar path uses only this field).
    pub c: f64,
    pub a: f64,
    pub n: f64,
    pub f: f64,
    pub w: f64,
    /// Observer-only catalyst tracer (pulse-chase).
    #[serde(default)]
    pub tracer_c: f64,
    /// Harvesting-biased catalyst material (D-089 composition schema).
    #[serde(default)]
    pub c_h: f64,
    /// Building-biased catalyst material (D-089 composition schema).
    #[serde(default)]
    pub c_b: f64,
    /// Metabolic reserve R — stored activated-resource equivalents (D-091).
    /// Not readiness, age, fitness, or a division trigger.
    #[serde(default)]
    pub r: f64,
    /// Free template monomer U_H (D-092).
    #[serde(default)]
    pub u_h: f64,
    /// Free template monomer U_B (D-092).
    #[serde(default)]
    pub u_b: f64,
    /// Harvesting catalyst–motif complex K_H (concentration; D-092).
    #[serde(default)]
    pub k_h: f64,
    /// Building catalyst–motif complex K_B (concentration; D-092).
    #[serde(default)]
    pub k_b: f64,
    /// Autocatalytic node precursor Q_K (D-094).
    #[serde(default)]
    pub q_k: f64,
    /// Autocatalytic edge precursor Q_E (D-094).
    #[serde(default)]
    pub q_e: f64,
    /// Free catalytic node K_A (D-094).
    #[serde(default)]
    pub k_a: f64,
    /// Free catalytic node K_R (D-094).
    #[serde(default)]
    pub k_r: f64,
    // Note: k_b above is D-092 motif complex; D-094 building node reuses no separate field —
    // building node mass is stored in `k_node_b` to avoid colliding with motif K_B.
    /// Free catalytic node K_B (D-094 building node).
    #[serde(default)]
    pub k_node_b: f64,
}

fn default_equation_id() -> String {
    EQUATION_VERSION_MATERIAL_MESH.to_string()
}

fn default_schema_version() -> u32 {
    MATERIAL_MESH_SCHEMA_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialMesh {
    /// Closed ring vertices (CCW).
    pub vertices: Vec<[f64; 2]>,
    /// Edge i connects vertices[i] → vertices[(i+1) % n].
    pub edges: Vec<MeshEdge>,
    /// Free membrane reserve L.
    pub free_l: f64,
    /// Interior lumped chemistry (inside closed mesh).
    pub interior: LumpedChem,
    /// Exterior reservoir chemistry (environment).
    pub exterior: LumpedChem,
    pub rho_s: f64,
    pub b_max_per_length: f64,
    pub bond_threshold: f64,
    pub l_max: f64,
    pub l_min: f64,
    pub alive: bool,
    pub death_reason: Option<String>,
    /// Equation identity stamp; old snapshots default to Phase-1 mesh schema.
    #[serde(default = "default_equation_id")]
    pub equation_id: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Physical template polymer chains (D-092). Empty under older schemas.
    #[serde(default)]
    pub templates: Vec<TemplateChain>,
    /// Observer allocator for new chain ids (never enters chemistry decisions).
    #[serde(default)]
    pub next_template_id: u64,
    /// Deterministic RNG state for template chemistry (D-092).
    #[serde(default = "default_template_rng")]
    pub template_rng: u64,
    /// Physical autocatalytic edge complexes E_ij (D-094). Empty under older schemas.
    #[serde(default)]
    pub autocatalytic_edges: Vec<crate::autocatalytic_edges::CatalyticEdgeComplex>,
    /// Observer allocator for new edge ids (never enters chemistry decisions).
    #[serde(default)]
    pub next_edge_id: u64,
}

fn default_template_rng() -> u64 {
    0xD092_CAFE_u64
}

impl MaterialMesh {
    pub fn n(&self) -> usize {
        self.vertices.len()
    }

    pub fn edge_length(&self, i: usize) -> f64 {
        let n = self.n();
        let a = self.vertices[i];
        let b = self.vertices[(i + 1) % n];
        ((b[0] - a[0]).hypot(b[1] - a[1])).max(1e-15)
    }

    pub fn rest_length(&self, i: usize) -> f64 {
        (self.edges[i].m / self.rho_s.max(1e-15)).max(1e-15)
    }

    pub fn strain(&self, i: usize) -> f64 {
        let l0 = self.rest_length(i);
        (self.edge_length(i) - l0) / l0
    }

    pub fn perimeter(&self) -> f64 {
        (0..self.n()).map(|i| self.edge_length(i)).sum()
    }

    pub fn signed_area(&self) -> f64 {
        let n = self.n();
        let mut a = 0.0;
        for i in 0..n {
            let p = self.vertices[i];
            let q = self.vertices[(i + 1) % n];
            a += p[0] * q[1] - q[0] * p[1];
        }
        0.5 * a
    }

    pub fn area(&self) -> f64 {
        self.signed_area().abs()
    }

    pub fn total_structural_mass(&self) -> f64 {
        self.edges.iter().map(|e| e.m.max(0.0)).sum()
    }

    pub fn total_bound_membrane(&self) -> f64 {
        self.edges.iter().map(|e| e.b.max(0.0)).sum()
    }

    pub fn total_membrane(&self) -> f64 {
        self.free_l.max(0.0) + self.total_bound_membrane()
    }

    pub fn closed_intact(&self) -> bool {
        self.alive && self.n() >= 3 && self.edges.iter().all(|e| !e.ruptured && e.m > 0.0)
    }

    pub fn b_max_for_edge(&self, i: usize) -> f64 {
        self.b_max_per_length.max(0.0) * self.edge_length(i)
    }

    pub fn occupancy(&self, i: usize) -> f64 {
        let cap = self.b_max_for_edge(i).max(1e-15);
        (self.edges[i].b / cap).clamp(0.0, 1.0)
    }

    /// Seed a regular n-gon of approximate radius `radius` with uniform structural density.
    pub fn seed_regular(
        n: usize,
        radius: f64,
        cx: f64,
        cy: f64,
        rho_s: f64,
        theta_b: f64,
        interior: LumpedChem,
        exterior: LumpedChem,
        free_l: f64,
    ) -> Self {
        let n = n.max(6);
        let mut vertices = Vec::with_capacity(n);
        for k in 0..n {
            let ang = 2.0 * std::f64::consts::PI * (k as f64) / (n as f64);
            vertices.push([cx + radius * ang.cos(), cy + radius * ang.sin()]);
        }
        let mut mesh = Self {
            vertices,
            edges: vec![MeshEdge::default(); n],
            free_l: free_l.max(0.0),
            interior,
            exterior,
            rho_s: rho_s.max(1e-15),
            b_max_per_length: DEFAULT_B_MAX_PER_LENGTH,
            bond_threshold: DEFAULT_BOND_THRESHOLD,
            l_max: DEFAULT_L_MAX,
            l_min: DEFAULT_L_MIN,
            alive: true,
            death_reason: None,
            equation_id: default_equation_id(),
            schema_version: default_schema_version(),
            templates: Vec::new(),
            next_template_id: 1,
            template_rng: default_template_rng(),
            autocatalytic_edges: Vec::new(),
            next_edge_id: 1,
        };
        for i in 0..n {
            let ell = mesh.edge_length(i);
            mesh.edges[i].m = rho_s * ell;
            mesh.edges[i].b = (theta_b.clamp(0.0, 1.0)) * mesh.b_max_for_edge(i);
            mesh.edges[i].tracer_m = 0.0;
            mesh.edges[i].tracer_b = 0.0;
            mesh.edges[i].ruptured = false;
        }
        mesh
    }

    pub fn centroid(&self) -> [f64; 2] {
        let n = self.n().max(1) as f64;
        let mut cx = 0.0;
        let mut cy = 0.0;
        for p in &self.vertices {
            cx += p[0];
            cy += p[1];
        }
        [cx / n, cy / n]
    }

    /// Point-in-polygon (ray cast) for local inside/outside sampling.
    pub fn point_inside(&self, x: f64, y: f64) -> bool {
        let n = self.n();
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let pi = self.vertices[i];
            let pj = self.vertices[j];
            let intersect = ((pi[1] > y) != (pj[1] > y))
                && (x < (pj[0] - pi[0]) * (y - pi[1]) / (pj[1] - pi[1] + 1e-30) + pi[0]);
            if intersect {
                inside = !inside;
            }
            j = i;
        }
        inside
    }
}
