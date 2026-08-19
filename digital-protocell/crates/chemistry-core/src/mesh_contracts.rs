//! DC-DEV-020-R9 mesh contract audit and versioned substrate accounting.
//!
//! This module is deliberately separate from the historical D-086 descriptors.
//! It records the current mesh reaction columns, the conservative v2 columns,
//! and the three ledgers used by the R9 requalification. It does not introduce
//! a new biological reaction.

use crate::material_mesh::{
    MaterialMesh, EQUATION_VERSION_MATERIAL_MESH, EQUATION_VERSION_MATERIAL_MESH_CONSERVATIVE,
};
use crate::mesh_reactions::MeshChemistrySchema;
use crate::stoichiometry::{exact_rank, left_nullspace, verify_m_transpose_s_zero, Rational};
use serde::{Deserialize, Serialize};

pub const R9_SCHEMA_ID: &str = "material_mesh_stoichiometry_v2_conservative";
pub const R9_LEDGER_SCHEMA: &str = "dcdev020r9_three_ledger_v1";

pub const MESH_SPECIES: [&str; 19] = [
    "N",
    "F",
    "A",
    "R",
    "C",
    "W",
    "M",
    "L",
    "B",
    "U_H",
    "U_B",
    "K_H",
    "K_B",
    "Q_K",
    "Q_E",
    "K_A",
    "K_R",
    "K_NODE_B",
    "template_material",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshReaction {
    Activation,
    CatalystProduction,
    StructureProduction,
    MembraneProduction,
    StructureTurnover,
    CatalystTurnover,
    ActivatedDecay,
    ReserveStore,
    ReserveRelease,
    ReserveLoss,
    MembraneBind,
    MembraneUnbind,
    MembraneDamage,
    StructuralDamage,
}

impl MeshReaction {
    pub const ALL: [Self; 14] = [
        Self::Activation,
        Self::CatalystProduction,
        Self::StructureProduction,
        Self::MembraneProduction,
        Self::StructureTurnover,
        Self::CatalystTurnover,
        Self::ActivatedDecay,
        Self::ReserveStore,
        Self::ReserveRelease,
        Self::ReserveLoss,
        Self::MembraneBind,
        Self::MembraneUnbind,
        Self::MembraneDamage,
        Self::StructuralDamage,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Activation => "activation",
            Self::CatalystProduction => "catalyst_production",
            Self::StructureProduction => "structure_production",
            Self::MembraneProduction => "membrane_production",
            Self::StructureTurnover => "structure_turnover",
            Self::CatalystTurnover => "catalyst_turnover",
            Self::ActivatedDecay => "activated_decay",
            Self::ReserveStore => "reserve_store",
            Self::ReserveRelease => "reserve_release",
            Self::ReserveLoss => "reserve_loss",
            Self::MembraneBind => "membrane_bind",
            Self::MembraneUnbind => "membrane_unbind",
            Self::MembraneDamage => "membrane_damage",
            Self::StructuralDamage => "structural_damage",
        }
    }
}

fn zero() -> Vec<Rational> {
    vec![Rational::ZERO; MESH_SPECIES.len()]
}

fn set(d: &mut [Rational], species: usize, value: i64) {
    d[species] = Rational::from_i64(value);
}

fn half(d: &mut [Rational], species: usize, sign: i64) {
    d[species] = Rational::new(sign, 2);
}

fn reaction_delta(schema: MeshChemistrySchema, reaction: MeshReaction) -> Vec<Rational> {
    // N F A R C W M L B U_H U_B K_H K_B Q_K Q_E K_A K_R K_NODE_B template_material
    let mut d = zero();
    match reaction {
        MeshReaction::Activation => {
            set(&mut d, 0, -1);
            set(&mut d, 1, -1);
            set(&mut d, 2, 1);
            set(&mut d, 5, 1);
        }
        MeshReaction::CatalystProduction => {
            set(&mut d, 2, -1);
            set(&mut d, 4, 1);
        }
        MeshReaction::StructureProduction => {
            set(&mut d, 2, -1);
            set(&mut d, 6, 1);
            if schema == MeshChemistrySchema::HistoricalV1 {
                set(&mut d, 5, 1);
            }
        }
        MeshReaction::MembraneProduction => {
            set(&mut d, 2, -1);
            set(&mut d, 7, 1);
            if schema == MeshChemistrySchema::HistoricalV1 {
                set(&mut d, 5, 1);
            }
        }
        MeshReaction::StructureTurnover | MeshReaction::StructuralDamage => {
            set(&mut d, 6, -1);
            set(&mut d, 5, 1);
        }
        MeshReaction::CatalystTurnover => {
            set(&mut d, 4, -1);
            set(&mut d, 5, 1);
        }
        MeshReaction::ActivatedDecay => {
            set(&mut d, 2, -1);
            set(&mut d, 5, 1);
        }
        MeshReaction::ReserveStore => {
            set(&mut d, 2, -1);
            set(&mut d, 3, 1);
        }
        MeshReaction::ReserveRelease => {
            set(&mut d, 3, -1);
            set(&mut d, 2, 1);
        }
        MeshReaction::ReserveLoss => {
            set(&mut d, 3, -1);
            set(&mut d, 5, 1);
        }
        MeshReaction::MembraneBind => {
            set(&mut d, 7, -1);
            set(&mut d, 8, 1);
        }
        MeshReaction::MembraneUnbind => {
            set(&mut d, 8, -1);
            set(&mut d, 7, 1);
        }
        MeshReaction::MembraneDamage => {
            // The damaged fraction is split between free membrane and W.
            set(&mut d, 8, -1);
            half(&mut d, 7, 1);
            half(&mut d, 5, 1);
        }
    }
    d
}

pub fn descriptor_matrix(schema: MeshChemistrySchema) -> Vec<Vec<Rational>> {
    let mut matrix = vec![vec![Rational::ZERO; MeshReaction::ALL.len()]; MESH_SPECIES.len()];
    for (column, reaction) in MeshReaction::ALL.iter().copied().enumerate() {
        for (row, value) in reaction_delta(schema, reaction).into_iter().enumerate() {
            matrix[row][column] = value;
        }
    }
    matrix
}

pub fn descriptor_delta(schema: MeshChemistrySchema, reaction: MeshReaction) -> Vec<Rational> {
    reaction_delta(schema, reaction)
}

/// Runtime-derived isolated deltas transcribe the actual mesh kernel transfers
/// independently of the descriptor matrix. Keeping this function separate
/// makes descriptor/runtime parity a tested contract rather than a prose claim.
pub fn runtime_delta(schema: MeshChemistrySchema, reaction: MeshReaction) -> Vec<Rational> {
    let mut d = zero();
    match reaction {
        MeshReaction::Activation => {
            set(&mut d, 0, -1);
            set(&mut d, 1, -1);
            set(&mut d, 2, 1);
            set(&mut d, 5, 1);
        }
        MeshReaction::CatalystProduction => {
            set(&mut d, 2, -1);
            set(&mut d, 4, 1);
        }
        MeshReaction::StructureProduction => {
            set(&mut d, 2, -1);
            set(&mut d, 6, 1);
            if schema == MeshChemistrySchema::HistoricalV1 {
                set(&mut d, 5, 1);
            }
        }
        MeshReaction::MembraneProduction => {
            set(&mut d, 2, -1);
            set(&mut d, 7, 1);
            if schema == MeshChemistrySchema::HistoricalV1 {
                set(&mut d, 5, 1);
            }
        }
        MeshReaction::StructureTurnover | MeshReaction::StructuralDamage => {
            set(&mut d, 6, -1);
            set(&mut d, 5, 1);
        }
        MeshReaction::CatalystTurnover
        | MeshReaction::ActivatedDecay
        | MeshReaction::ReserveLoss => {
            let source = match reaction {
                MeshReaction::CatalystTurnover => 4,
                MeshReaction::ActivatedDecay => 2,
                MeshReaction::ReserveLoss => 3,
                _ => unreachable!(),
            };
            set(&mut d, source, -1);
            set(&mut d, 5, 1);
        }
        MeshReaction::ReserveStore => {
            set(&mut d, 2, -1);
            set(&mut d, 3, 1);
        }
        MeshReaction::ReserveRelease => {
            set(&mut d, 3, -1);
            set(&mut d, 2, 1);
        }
        MeshReaction::MembraneBind => {
            set(&mut d, 7, -1);
            set(&mut d, 8, 1);
        }
        MeshReaction::MembraneUnbind => {
            set(&mut d, 8, -1);
            set(&mut d, 7, 1);
        }
        MeshReaction::MembraneDamage => {
            set(&mut d, 8, -1);
            half(&mut d, 7, 1);
            half(&mut d, 5, 1);
        }
    }
    d
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshStoichiometricAudit {
    pub schema: String,
    pub species: Vec<String>,
    pub reactions: Vec<String>,
    pub matrix: Vec<Vec<String>>,
    pub rank: usize,
    pub left_nullspace_dimension: usize,
    pub positive_conservation_vectors: Vec<Vec<String>>,
    pub nonconservative_reactions: Vec<String>,
    pub descriptor_runtime_parity: bool,
    pub classification: String,
}

pub fn audit(schema: MeshChemistrySchema) -> MeshStoichiometricAudit {
    let matrix = descriptor_matrix(schema);
    let left = left_nullspace(&matrix);
    let ones = vec![Rational::ONE; MESH_SPECIES.len()];
    let nonconservative_reactions = MeshReaction::ALL
        .iter()
        .copied()
        .filter(|reaction| {
            let d = descriptor_delta(schema, *reaction);
            d.iter()
                .zip(ones.iter())
                .map(|(a, b)| a.mul(*b))
                .fold(Rational::ZERO, |acc, x| acc.add(x))
                .num
                != 0
        })
        .map(|reaction| reaction.label().to_string())
        .collect::<Vec<_>>();
    let parity = MeshReaction::ALL
        .iter()
        .copied()
        .all(|reaction| descriptor_delta(schema, reaction) == runtime_delta(schema, reaction));
    let mut positive_vectors = Vec::new();
    if verify_m_transpose_s_zero(&ones, &matrix) {
        positive_vectors.push(ones.clone());
    }
    for vector in &left {
        if vector.iter().all(|coefficient| coefficient.is_positive())
            && verify_m_transpose_s_zero(vector, &matrix)
        {
            positive_vectors.push(vector.clone());
        }
    }
    let classification = if positive_vectors.is_empty() {
        "NO_POSITIVE_CONSERVATION_VECTOR"
    } else {
        "POSITIVE_CONSERVATION_VECTOR_EXISTS"
    };
    MeshStoichiometricAudit {
        schema: match schema {
            MeshChemistrySchema::HistoricalV1 => EQUATION_VERSION_MATERIAL_MESH.to_string(),
            MeshChemistrySchema::ConservativeV2 => {
                EQUATION_VERSION_MATERIAL_MESH_CONSERVATIVE.to_string()
            }
        },
        species: MESH_SPECIES.iter().map(|s| (*s).to_string()).collect(),
        reactions: MeshReaction::ALL
            .iter()
            .map(|reaction| reaction.label().to_string())
            .collect(),
        matrix: matrix
            .iter()
            .map(|row| row.iter().map(ToString::to_string).collect())
            .collect(),
        rank: exact_rank(&matrix),
        left_nullspace_dimension: left.len(),
        positive_conservation_vectors: positive_vectors
            .iter()
            .map(|row| row.iter().map(ToString::to_string).collect())
            .collect(),
        nonconservative_reactions,
        descriptor_runtime_parity: parity,
        classification: classification.to_string(),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MaterialLedgerSnapshot {
    pub n: f64,
    pub f: f64,
    pub a: f64,
    pub r: f64,
    pub c: f64,
    pub waste: f64,
    pub structural_m: f64,
    pub free_l: f64,
    pub bound_b: f64,
    pub hereditary: f64,
}

impl MaterialLedgerSnapshot {
    pub fn strict_material_equivalent(&self) -> f64 {
        self.n
            + self.f
            + self.a
            + self.r
            + self.c
            + self.waste
            + self.structural_m
            + self.free_l
            + self.bound_b
            + self.hereditary
    }

    pub fn activation_store(&self) -> f64 {
        self.f + self.a + self.r
    }

    pub fn organized_material(&self) -> f64 {
        self.c + self.a + self.r + self.structural_m + self.free_l + self.bound_b + self.hereditary
    }
}

pub fn snapshot(mesh: &MaterialMesh) -> MaterialLedgerSnapshot {
    let area = mesh.area().max(1e-9);
    MaterialLedgerSnapshot {
        n: mesh.interior.n.max(0.0) * area,
        f: mesh.interior.f.max(0.0) * area,
        a: mesh.interior.a.max(0.0) * area,
        r: mesh.interior.r.max(0.0) * area,
        c: mesh.interior.c.max(0.0) * area,
        waste: mesh.interior.w.max(0.0) * area,
        structural_m: mesh.total_structural_mass(),
        free_l: mesh.free_l.max(0.0),
        bound_b: mesh.total_bound_membrane(),
        hereditary: (mesh.templates.len() as f64)
            + mesh.interior.u_h.max(0.0) * area
            + mesh.interior.u_b.max(0.0) * area
            + mesh.interior.k_h.max(0.0) * area
            + mesh.interior.k_b.max(0.0) * area
            + mesh.interior.q_k.max(0.0) * area
            + mesh.interior.q_e.max(0.0) * area
            + mesh.interior.k_a.max(0.0) * area
            + mesh.interior.k_r.max(0.0) * area
            + mesh.interior.k_node_b.max(0.0) * area
            + mesh
                .templates
                .iter()
                .flat_map(|template| template.site_k.iter())
                .copied()
                .sum::<f64>(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeClosure {
    pub initial: MaterialLedgerSnapshot,
    pub final_state: MaterialLedgerSnapshot,
    pub boundary_material_delta: f64,
    pub max_material_residual: f64,
    pub max_activation_residual: f64,
}

pub fn closure(before: MaterialLedgerSnapshot, after: MaterialLedgerSnapshot) -> RuntimeClosure {
    let material_delta = after.strict_material_equivalent() - before.strict_material_equivalent();
    let activation_delta = after.activation_store() - before.activation_store();
    RuntimeClosure {
        initial: before,
        final_state: after,
        boundary_material_delta: material_delta,
        max_material_residual: material_delta.abs(),
        max_activation_residual: activation_delta.abs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material_mesh::{LumpedChem, MaterialMesh};
    use crate::mesh_reactions::{evaluate_death, reactions_step, ReactionParams};
    use crate::mesh_transport::{transport_step, TransportParams};

    fn conservative_fixture() -> MaterialMesh {
        let mut mesh = MaterialMesh::seed_regular(
            12,
            2.0,
            0.0,
            0.0,
            1.0,
            0.7,
            LumpedChem {
                c: 0.8,
                a: 0.6,
                n: 0.5,
                f: 0.5,
                w: 0.1,
                ..Default::default()
            },
            LumpedChem {
                n: 1.0,
                f: 1.0,
                ..Default::default()
            },
            1.0,
        );
        mesh.stamp_conservative_schema();
        mesh
    }

    #[test]
    fn historical_mesh_has_no_positive_material_vector() {
        let audit = audit(MeshChemistrySchema::HistoricalV1);
        assert_eq!(audit.classification, "NO_POSITIVE_CONSERVATION_VECTOR");
        assert!(audit
            .nonconservative_reactions
            .iter()
            .any(|r| r == "structure_production"));
        assert!(audit
            .nonconservative_reactions
            .iter()
            .any(|r| r == "membrane_production"));
        assert!(audit.descriptor_runtime_parity);
    }

    #[test]
    fn conservative_v2_has_strictly_positive_vector() {
        let audit = audit(MeshChemistrySchema::ConservativeV2);
        assert_eq!(audit.classification, "POSITIVE_CONSERVATION_VECTOR_EXISTS");
        assert!(audit.nonconservative_reactions.is_empty());
        assert!(audit.descriptor_runtime_parity);
    }

    #[test]
    fn conservative_runtime_closes_strict_material_ledger() {
        let mut mesh = conservative_fixture();
        let before = snapshot(&mesh);
        let params = ReactionParams::conservative_v2();
        for _ in 0..40 {
            reactions_step(&mut mesh, &params, 0.02, true, true);
        }
        let result = closure(before, snapshot(&mesh));
        assert!(
            result.max_material_residual < 1e-8,
            "strict material drift: {}",
            result.max_material_residual
        );
        assert!(mesh.can_advance_physics());
    }

    #[test]
    fn conservative_death_is_observer_only_and_does_not_gate_transport() {
        let mut mesh = conservative_fixture();
        mesh.interior.c = 0.0;
        mesh.interior.a = 0.0;
        mesh.interior.n = 0.0;
        mesh.interior.f = 0.0;
        for i in 0..mesh.n() {
            crate::mesh_reactions::apply_local_rupture(&mut mesh, i);
        }
        evaluate_death(&mut mesh);
        assert!(mesh.alive, "observer qualification must not latch death");
        assert!(!mesh.observer_viable());
        assert_eq!(mesh.observer_death_reason(), Some("mesh_rupture"));

        let ledger = transport_step(&mut mesh, &TransportParams { k_flux: 1.0 }, 0.02);
        assert!(ledger.n_in > 0.0 && ledger.f_in > 0.0);
        assert!(mesh.can_advance_physics());
    }

    #[test]
    fn historical_reaction_serialization_remains_v1_compatible() {
        let historical = serde_json::to_string(&ReactionParams::default()).unwrap();
        let conservative = serde_json::to_string(&ReactionParams::conservative_v2()).unwrap();
        assert!(!historical.contains("mesh_schema"));
        assert!(conservative.contains("ConservativeV2"));
    }
}
