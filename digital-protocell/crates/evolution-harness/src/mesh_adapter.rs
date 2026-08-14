use crate::{
    AdapterEnvironmentEvent, AdapterError, AdvanceOutcome, EnvironmentCapability,
    EnvironmentContext, EnvironmentProtocolV1, FounderIdentityV1, FounderInitializationContext,
    HeredityEvidenceV1, Metadata, OrganismAdapter, PhenotypeEvidenceV1,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_fission::FissionParams;
use chemistry_core::mesh_growth::GrowthParams;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_population::coupled_step_growth;
use chemistry_core::mesh_reactions::{
    apply_membrane_damage, apply_structural_damage, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;

/// Narrow adapter over the existing material mesh. It uses the certified
/// transport/reaction/growth/mechanics/fission path and only applies ecology
/// through existing public mesh resources and damage operations.
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

    fn initialize_founder(
        &mut self,
        founder: &FounderIdentityV1,
        _context: FounderInitializationContext,
    ) -> Result<Self::Organism, AdapterError> {
        let n = 24 + (founder.seed % 3) as usize;
        let interior = LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.4,
            f: 0.4,
            w: 0.1,
            ..Default::default()
        };
        let exterior = LumpedChem {
            n: 2.0,
            f: 2.0,
            ..Default::default()
        };
        Ok(MaterialMesh::seed_regular(
            n,
            self.founder_radius,
            40.0,
            40.0,
            DEFAULT_RHO_S,
            0.7,
            interior,
            exterior,
            5.0,
        ))
    }

    fn accepted_dt(&self) -> f64 {
        self.mech.dt
    }

    fn environment_capabilities(&self) -> Vec<EnvironmentCapability> {
        vec![
            EnvironmentCapability::ContinuousResources,
            EnvironmentCapability::PulsedResources,
            EnvironmentCapability::Scarcity,
            EnvironmentCapability::Damage,
            EnvironmentCapability::Transitions,
        ]
    }

    fn apply_declared_environment(
        &mut self,
        organism: &mut Self::Organism,
        environment: &EnvironmentProtocolV1,
        _accepted_step: u64,
        accepted_simulated_time: f64,
        context: EnvironmentContext,
    ) -> Result<Vec<AdapterEnvironmentEvent>, AdapterError> {
        let ecology = &environment.resource_ecology;
        let mut events = Vec::new();
        if matches!(environment.resource_mode, crate::ResourceMode::Continuous) {
            organism.exterior.n =
                (organism.exterior.n + ecology.continuous_supply.n * context.accepted_dt).max(0.0);
            organism.exterior.f =
                (organism.exterior.f + ecology.continuous_supply.f * context.accepted_dt).max(0.0);
            organism.exterior.w =
                (organism.exterior.w + ecology.continuous_supply.w * context.accepted_dt).max(0.0);
        }
        let on_pulse = environment
            .pulse_schedule
            .iter()
            .chain(ecology.pulse_schedule.iter())
            .any(|time| {
                (*time - accepted_simulated_time).abs() <= context.accepted_dt * 0.5 + f64::EPSILON
            });
        if on_pulse && matches!(environment.resource_mode, crate::ResourceMode::Pulsed) {
            organism.exterior.n = (organism.exterior.n + ecology.pulse_delta.n).max(0.0);
            organism.exterior.f = (organism.exterior.f + ecology.pulse_delta.f).max(0.0);
            organism.exterior.w = (organism.exterior.w + ecology.pulse_delta.w).max(0.0);
        }
        if matches!(environment.resource_mode, crate::ResourceMode::Scarcity) {
            let scarce = environment
                .scarcity_schedule
                .iter()
                .chain(ecology.scarcity_schedule.iter())
                .any(|window| {
                    accepted_simulated_time >= window.start && accepted_simulated_time < window.end
                });
            let scale = if scarce { ecology.scarcity_scale } else { 1.0 };
            organism.exterior.n = (2.0 * scale).max(0.0);
            organism.exterior.f = (2.0 * scale).max(0.0);
        }
        let damage_mode = if environment.damage_mode != crate::DamageMode::None {
            &environment.damage_mode
        } else {
            &ecology.damage_mode
        };
        if *damage_mode != crate::DamageMode::None {
            if let Some(interval) = environment.damage_interval.or(ecology.damage_interval) {
                let crossed = (accepted_simulated_time / interval).floor()
                    < ((accepted_simulated_time + context.accepted_dt) / interval).floor();
                if interval > 0.0 && crossed {
                    let fraction = if ecology.damage_fraction > 0.0 {
                        ecology.damage_fraction
                    } else {
                        0.05
                    };
                    let structural = apply_structural_damage(organism, fraction);
                    let membrane = apply_membrane_damage(organism, fraction * 0.5);
                    let mut metadata = Metadata::new();
                    metadata.insert("structural".into(), format!("{structural:.9}"));
                    metadata.insert("membrane".into(), format!("{membrane:.9}"));
                    events.push(AdapterEnvironmentEvent {
                        event_type: crate::EventType::DamageApplied,
                        metadata,
                    });
                }
            }
        }
        let _ = context.living_population;
        Ok(events)
    }

    fn advance(
        &mut self,
        organism: &mut Self::Organism,
        _environment: &EnvironmentProtocolV1,
        _accepted_step: u64,
        _accepted_simulated_time: f64,
    ) -> Result<AdvanceOutcome<Self::Organism>, AdapterError> {
        if !organism.alive {
            return Ok(AdvanceOutcome::Died {
                reason: organism
                    .death_reason
                    .clone()
                    .unwrap_or_else(|| "mesh_derived_dead".into()),
                accepted_dt: self.mech.dt,
                metadata: Metadata::new(),
            });
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
            return Ok(AdvanceOutcome::Fission {
                offspring: vec![daughter_a, daughter_b],
                accepted_dt: self.mech.dt,
                metadata,
            });
        }
        if !organism.alive {
            return Ok(AdvanceOutcome::Died {
                reason: organism
                    .death_reason
                    .clone()
                    .unwrap_or_else(|| "mesh_derived_dead".into()),
                accepted_dt: self.mech.dt,
                metadata: Metadata::new(),
            });
        }
        Ok(AdvanceOutcome::Continuing {
            accepted_dt: self.mech.dt,
            metadata: Metadata::new(),
        })
    }

    fn is_alive(&self, organism: &Self::Organism) -> bool {
        organism.alive
    }
    fn phenotype(&self, organism: &Self::Organism) -> String {
        format!(
            "mesh_vertices:{};mass:{:.9};area:{:.9}",
            organism.n(),
            organism.total_structural_mass(),
            organism.area()
        )
    }
    fn hereditary_state(&self, organism: &Self::Organism) -> String {
        format!(
            "equation:{};templates:{};autocatalytic_edges:{}",
            organism.equation_id,
            organism.templates.len(),
            organism.autocatalytic_edges.len()
        )
    }
    fn heredity_evidence(
        &self,
        _parent: Option<&Self::Organism>,
        _organism: &Self::Organism,
    ) -> HeredityEvidenceV1 {
        HeredityEvidenceV1::unavailable("HARNESS_ADAPTER_UNAVAILABLE: D-094 mechanism-specific heredity qualifier is not adapted")
    }
    fn phenotype_evidence(
        &self,
        _environment: &EnvironmentProtocolV1,
        _organism: &Self::Organism,
    ) -> PhenotypeEvidenceV1 {
        PhenotypeEvidenceV1::unavailable("HARNESS_ADAPTER_UNAVAILABLE: D-094 mechanism-specific phenotype qualifier is not adapted")
    }
}
