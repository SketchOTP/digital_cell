use crate::{
    AdapterError, AdvanceOutcome, EnvironmentProtocolV1, FounderIdentityV1, Metadata, OrganismAdapter,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_fission::FissionParams;
use chemistry_core::mesh_growth::GrowthParams;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::{apply_membrane_damage, apply_structural_damage, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::mesh_population::coupled_step_growth;

/// Narrow adapter over the existing material mesh. It observes actual
/// `coupled_step_growth` fission results and never adds a reproduction command.
#[derive(Debug, Clone)]
pub struct DigitalCellMeshAdapter {
    pub mech: MechParams,
    pub reactions: ReactionParams,
    pub transport: TransportParams,
    pub growth: GrowthParams,
    pub fission: FissionParams,
    pub enable_mechanics: bool,
    pub enable_fission: bool,
    pub founder_radius: f64,
}

impl Default for DigitalCellMeshAdapter {
    fn default() -> Self {
        Self {
            mech: MechParams::default(),
            reactions: ReactionParams::default(),
            transport: TransportParams::default(),
            growth: GrowthParams::default(),
            fission: FissionParams::default(),
            enable_mechanics: true,
            enable_fission: true,
            founder_radius: 14.0,
        }
    }
}

impl OrganismAdapter for DigitalCellMeshAdapter {
    type Organism = MaterialMesh;

    fn initialize_founder(&self, founder: &FounderIdentityV1) -> Result<Self::Organism, AdapterError> {
        let n = 24 + (founder.seed % 3) as usize;
        let interior = LumpedChem { c: 0.8, a: 0.5, n: 0.4, f: 0.4, w: 0.1, ..Default::default() };
        let exterior = LumpedChem { n: 2.0, f: 2.0, ..Default::default() };
        Ok(MaterialMesh::seed_regular(n, self.founder_radius, 40.0, 40.0, DEFAULT_RHO_S, 0.7, interior, exterior, 5.0))
    }

    fn advance(
        &self,
        organism: &mut Self::Organism,
        _environment: &EnvironmentProtocolV1,
        _accepted_step: u64,
        _accepted_simulated_time: u64,
    ) -> Result<AdvanceOutcome<Self::Organism>, AdapterError> {
        if !organism.alive {
            return Ok(AdvanceOutcome::Died { reason: organism.death_reason.clone().unwrap_or_else(|| "mesh_derived_dead".into()) });
        }
        let (_, _, split) = coupled_step_growth(
            organism,
            &self.mech,
            &self.reactions,
            &self.transport,
            &self.growth,
            &self.fission,
            self.enable_mechanics,
            self.enable_fission,
        );
        if let Some((daughter_a, daughter_b, event)) = split {
            organism.alive = false;
            organism.death_reason = Some("fissioned".into());
            let mut metadata = Metadata::new();
            metadata.insert("parent_vertices".into(), event.parent_n.to_string());
            metadata.insert("daughter_a_vertices".into(), event.daughter_a_n.to_string());
            metadata.insert("daughter_b_vertices".into(), event.daughter_b_n.to_string());
            metadata.insert("partition_ok".into(), event.partition.ok.to_string());
            return Ok(AdvanceOutcome::Fission { offspring: vec![daughter_a, daughter_b], metadata });
        }
        if !organism.alive {
            return Ok(AdvanceOutcome::Died { reason: organism.death_reason.clone().unwrap_or_else(|| "mesh_derived_dead".into()) });
        }
        Ok(AdvanceOutcome::Continuing)
    }

    fn is_alive(&self, organism: &Self::Organism) -> bool { organism.alive }

    fn phenotype(&self, organism: &Self::Organism) -> String {
        format!("mesh_vertices:{};mass:{:.9};area:{:.9}", organism.n(), organism.total_structural_mass(), organism.area())
    }

    fn hereditary_state(&self, organism: &Self::Organism) -> String {
        format!("equation:{};templates:{};autocatalytic_edges:{}", organism.equation_id, organism.templates.len(), organism.autocatalytic_edges.len())
    }

    fn resource_state(&self, organism: &Self::Organism) -> String {
        format!("interior_a:{:.9};interior_n:{:.9};interior_f:{:.9};free_l:{:.9}", organism.interior.a, organism.interior.n, organism.interior.f, organism.free_l)
    }

    fn apply_declared_environment(
        &self,
        organism: &mut Self::Organism,
        environment: &EnvironmentProtocolV1,
        accepted_step: u64,
    ) -> Result<Option<String>, AdapterError> {
        if let Some(interval) = environment.damage_interval {
            if interval == 0 || accepted_step % interval != 0 {
                return Ok(None);
            }
            let structural = apply_structural_damage(organism, 0.05);
            let membrane = apply_membrane_damage(organism, 0.025);
            return Ok(Some(format!("declared_mesh_damage;structural:{structural:.9};membrane:{membrane:.9}")));
        }
        Ok(None)
    }
}
