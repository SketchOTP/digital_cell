use crate::{
    AdapterEnvironmentEvent, AdapterError, AdvanceOutcome, EnvironmentCapability,
    EnvironmentContext, EnvironmentProtocolV1, FounderIdentityV1, FounderInitializationContext,
    HeredityAdapter, HeredityEvidenceV1, Metadata, MutationContext, MutationOperator,
    OrganismAdapter, PhenotypeEvidenceV1, MutationProtocolV1,
};
use chemistry_core::d096_allocation::{
    mutate_allocation_genotype, AllocationGenotype, AllocationParams,
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
    /// When present, the adapter executes the existing D-096 finite allocation
    /// representation; None preserves the historical mesh adapter path.
    pub allocation_params: Option<AllocationParams>,
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
            allocation_params: None,
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
        let mut mesh = MaterialMesh::seed_regular(
            n,
            self.founder_radius,
            40.0,
            40.0,
            DEFAULT_RHO_S,
            0.7,
            interior,
            exterior,
            5.0,
        );
        if let Some(params) = self.allocation_params {
            mesh.enable_finite_allocation(AllocationGenotype::neutral(), &params);
        }
        Ok(mesh)
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
        if let Some(params) = self.allocation_params {
            chemistry_core::d096_allocation::expression_step(organism, &params, self.mech.dt)
                .map_err(|error| AdapterError::Advance(format!("D-096 expression: {error:?}")))?;
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
        let allocation = organism.finite_allocation.map(|state| {
            let params = self.allocation_params.unwrap_or_default();
            format!(
                ";d096_genotype_hash:{}",
                state.genotype.candidate_hash(&params)
            )
        }).unwrap_or_default();
        format!(
            "equation:{};templates:{};autocatalytic_edges:{}{}",
            organism.equation_id,
            organism.templates.len(),
            organism.autocatalytic_edges.len(),
            allocation
        )
    }
    fn heredity_evidence(
        &self,
        parent: Option<&Self::Organism>,
        organism: &Self::Organism,
    ) -> HeredityEvidenceV1 {
        if let (Some(parent), Some(child), Some(params)) =
            (parent, organism.finite_allocation, self.allocation_params)
        {
            let valid = parent
                .finite_allocation
                .map(|state| state.genotype.valid(&params))
                .unwrap_or(false)
                && child.genotype.valid(&params);
            return HeredityEvidenceV1 {
                observable: true,
                preserved: valid,
                comparison_basis: "D096_fixed_simplex_parent_child_validity".into(),
                metric: "genotype_simplex_valid".into(),
                value: Some(if valid { 1.0 } else { 0.0 }),
                qualification: valid,
                reason: "D-096 genotype remains within the frozen simplex; mutation provenance is recorded separately".into(),
            };
        }
        HeredityEvidenceV1::unavailable("HARNESS_ADAPTER_UNAVAILABLE: D-094 mechanism-specific heredity qualifier is not adapted")
    }
    fn phenotype_evidence(
        &self,
        _environment: &EnvironmentProtocolV1,
        organism: &Self::Organism,
    ) -> PhenotypeEvidenceV1 {
        if let (Some(state), Some(params)) = (organism.finite_allocation, self.allocation_params) {
            let expressed = state.catalysts.iter().copied().sum::<f64>() > 0.0;
            let valid = state.genotype.valid(&params);
            return PhenotypeEvidenceV1 {
                observable: true,
                expressed,
                comparison_basis: "D096_finite_catalyst_expression".into(),
                metric: "catalyst_mass_sum".into(),
                value: Some(state.catalysts.iter().sum()),
                qualification: valid && expressed,
                reason: "D-096 catalyst expression is measured from physical catalyst mass".into(),
            };
        }
        PhenotypeEvidenceV1::unavailable("HARNESS_ADAPTER_UNAVAILABLE: D-094 mechanism-specific phenotype qualifier is not adapted")
    }

    fn apply_heredity_and_mutation(
        &mut self,
        parent: &Self::Organism,
        offspring: &mut Self::Organism,
        protocol: &MutationProtocolV1,
        context: &MutationContext,
    ) -> Result<Option<Metadata>, AdapterError> {
        if protocol.mutation_protocol_id != "d096_allocation_mutation_v1" {
            return if protocol.mutation_rate == 0.0 && protocol.mutation_protocol_id == "mutation_none" {
                Ok(None)
            } else {
                Err(AdapterError::Unavailable)
            };
        }
        let Some(params) = self.allocation_params else {
            return Err(AdapterError::Unavailable);
        };
        let (Some(parent_state), Some(offspring_state)) =
            (parent.finite_allocation, offspring.finite_allocation)
        else {
            return Err(AdapterError::Unavailable);
        };
        if parent.alive || offspring.alive || offspring.n() >= parent.n()
            || offspring_state.genotype != parent_state.genotype
        {
            return Err(AdapterError::Advance(
                "D-096 mutation requires a qualified physical fission copy".into(),
            ));
        }
        let mut mutation_params = params;
        mutation_params.mutation_probability = protocol.mutation_rate;
        mutation_params.mutation_sigma = protocol.mutation_sigma;
        let record = mutate_allocation_genotype(
            parent_state.genotype,
            &mutation_params,
            context.seed,
        )
        .map_err(|error| AdapterError::Advance(format!("D-096 mutation: {error:?}")))?;
        offspring.finite_allocation = Some(chemistry_core::d096_allocation::AllocationState {
            genotype: record.post_genotype,
            catalysts: offspring_state.catalysts,
        });
        let mut metadata = Metadata::new();
        metadata.insert("operator".into(), record.operator.into());
        metadata.insert("provenance".into(), record.provenance.into());
        metadata.insert("seed".into(), record.seed.to_string());
        metadata.insert("offspring_index".into(), context.offspring_index.to_string());
        metadata.insert("mutation_occurred".into(), record.mutation_occurred.to_string());
        metadata.insert("source".into(), format_opt(record.source));
        metadata.insert("target".into(), format_opt(record.target));
        metadata.insert("raw_abs_normal".into(), format!("{:.17e}", record.raw_abs_normal));
        metadata.insert("applied_delta".into(), format!("{:.17e}", record.applied_delta));
        metadata.insert("pre_genotype".into(), serde_json::to_string(&record.pre_genotype).unwrap());
        metadata.insert("post_genotype".into(), serde_json::to_string(&record.post_genotype).unwrap());
        metadata.insert("candidate_hash".into(), record.post_genotype.candidate_hash(&mutation_params));
        Ok(Some(metadata))
    }
}

fn format_opt(value: Option<usize>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "none".into())
}

/// Adapter-facing implementation of the accepted reusable mutation boundary.
#[derive(Debug, Clone, Copy)]
pub struct D096AllocationMutationOperator {
    pub params: AllocationParams,
}

impl HeredityAdapter for D096AllocationMutationOperator {
    type HereditaryState = AllocationGenotype;

    fn encode(&self, state: &Self::HereditaryState) -> String {
        serde_json::to_string(state).expect("AllocationGenotype is serializable")
    }

    fn decode(&self, encoded: &str) -> Result<Self::HereditaryState, AdapterError> {
        serde_json::from_str(encoded)
            .map_err(|error| AdapterError::Observation(format!("D-096 genotype decode: {error}")))
    }
}

impl MutationOperator for D096AllocationMutationOperator {
    type HereditaryState = AllocationGenotype;

    fn mutate(
        &self,
        state: &Self::HereditaryState,
        context: &MutationContext,
    ) -> Result<Self::HereditaryState, AdapterError> {
        mutate_allocation_genotype(*state, &self.params, context.seed)
            .map(|record| record.post_genotype)
            .map_err(|error| AdapterError::Advance(format!("D-096 mutation: {error:?}")))
    }
}
