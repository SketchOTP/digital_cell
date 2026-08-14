use crate::{
    AdapterEnvironmentEvent, AdapterError, AdvanceOutcome, EnvironmentCapability,
    EnvironmentContext, EnvironmentProtocolV1, FounderIdentityV1, FounderInitializationContext,
    HeredityAdapter, HeredityEvidenceV1, Metadata, MutationContext, MutationOperator,
    MutationProtocolV1, OrganismAdapter, PhenotypeEvidenceV1,
};
use chemistry_core::d096_allocation::{
    apply_assay_environment, expression_step, mutate_allocation_genotype, AllocationGenotype,
    AllocationParams, AllocationState, AssayEnvironment,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_fission::{try_local_fission, FissionParams};
use chemistry_core::mesh_growth::GrowthParams;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_population::coupled_step_growth;
use chemistry_core::mesh_reactions::{
    apply_membrane_damage, apply_structural_damage, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;

const FROZEN_D096_MUTATION_PROBABILITY: f64 = 0.01;

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
    /// Explicit D-096 founder genotype. Generic mesh founders remain generic
    /// when this is None, even when allocation parameters are configured.
    pub d096_founder_genotype: Option<AllocationGenotype>,
    /// Execute the already-qualified D-096 H/B/Neutral forcing verbatim.
    /// This is an observer-selected environment, never organism state.
    pub d096_assay_environment: Option<AssayEnvironment>,
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
            d096_founder_genotype: None,
            d096_assay_environment: None,
        }
    }
}

impl DigitalCellMeshAdapter {
    pub fn with_d096_founder(mut self, genotype: AllocationGenotype) -> Self {
        self.d096_founder_genotype = Some(genotype);
        self
    }

    pub fn with_d096_assay_environment(mut self, environment: AssayEnvironment) -> Self {
        self.d096_assay_environment = Some(environment);
        self
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
        if let (Some(params), Some(genotype)) = (self.allocation_params, self.d096_founder_genotype)
        {
            mesh.enable_finite_allocation(genotype, &params);
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
        accepted_step: u64,
        accepted_simulated_time: f64,
        context: EnvironmentContext,
    ) -> Result<Vec<AdapterEnvironmentEvent>, AdapterError> {
        if let Some(assay_environment) = self.d096_assay_environment {
            let ledger = apply_assay_environment(
                organism,
                assay_environment,
                accepted_step.saturating_sub(1),
            );
            let mut events = Vec::new();
            if ledger.structural_damage > 0.0 || ledger.membrane_damage > 0.0 {
                let mut metadata = Metadata::new();
                metadata.insert(
                    "structural".into(),
                    format!("{:.9}", ledger.structural_damage),
                );
                metadata.insert("membrane".into(), format!("{:.9}", ledger.membrane_damage));
                metadata.insert("forcing".into(), "D096_exact_assay_environment".into());
                events.push(AdapterEnvironmentEvent {
                    event_type: crate::EventType::DamageApplied,
                    metadata,
                });
            }
            return Ok(events);
        }
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
        let allocation = organism
            .finite_allocation
            .map(|state| {
                let params = self.allocation_params.unwrap_or_default();
                format!(
                    ";d096_genotype_hash:{}",
                    state.genotype.candidate_hash(&params)
                )
            })
            .unwrap_or_default();
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
                && child.genotype.valid(&params)
                && lawful_d096_copy(
                    parent.finite_allocation.unwrap().genotype,
                    child.genotype,
                    &params,
                );
            return HeredityEvidenceV1 {
                observable: true,
                preserved: valid,
                comparison_basis: "D096_lawful_parent_child_copy_or_transfer".into(),
                metric: "simplex_copy_or_single_transfer".into(),
                value: Some(if valid { 1.0 } else { 0.0 }),
                qualification: valid,
                reason: "D-096 child genotype is either an exact parent copy or one bounded source-to-target transfer; event provenance is checked by the harness".into(),
            };
        }
        HeredityEvidenceV1::unavailable("HARNESS_ADAPTER_UNAVAILABLE: D-094 mechanism-specific heredity qualifier is not adapted")
    }
    fn phenotype_evidence(
        &self,
        _environment: &EnvironmentProtocolV1,
        _organism: &Self::Organism,
    ) -> PhenotypeEvidenceV1 {
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
            return if protocol.mutation_rate == 0.0
                && protocol.mutation_protocol_id == "mutation_none"
            {
                Ok(None)
            } else {
                Err(AdapterError::Unavailable)
            };
        }
        let Some(params) = self.allocation_params else {
            return Err(AdapterError::Unavailable);
        };
        if (params.mutation_probability - FROZEN_D096_MUTATION_PROBABILITY).abs() > 1e-12
            || (protocol.mutation_rate - FROZEN_D096_MUTATION_PROBABILITY).abs() > 1e-12
        {
            return Err(AdapterError::Advance(
                "D-096 mutation rate must remain the frozen p=0.01".into(),
            ));
        }
        let (Some(parent_state), Some(offspring_state)) =
            (parent.finite_allocation, offspring.finite_allocation)
        else {
            return Err(AdapterError::Unavailable);
        };
        if !context.qualified_physical_copy || offspring_state.genotype != parent_state.genotype {
            return Err(AdapterError::Advance(
                "D-096 mutation requires a qualified physical fission copy".into(),
            ));
        }
        let mut mutation_params = params;
        mutation_params.mutation_probability = protocol.mutation_rate;
        let record =
            mutate_allocation_genotype(parent_state.genotype, &mutation_params, context.seed)
                .map_err(|error| AdapterError::Advance(format!("D-096 mutation: {error:?}")))?;
        offspring.finite_allocation = Some(chemistry_core::d096_allocation::AllocationState {
            genotype: record.post_genotype,
            catalysts: offspring_state.catalysts,
        });
        let mut metadata = Metadata::new();
        metadata.insert("operator".into(), record.operator.into());
        metadata.insert("provenance".into(), record.provenance.into());
        metadata.insert("seed".into(), record.seed.to_string());
        metadata.insert(
            "offspring_index".into(),
            context.offspring_index.to_string(),
        );
        metadata.insert(
            "qualified_copy_ordinal".into(),
            context.qualified_copy_ordinal.to_string(),
        );
        metadata.insert(
            "mutation_occurred".into(),
            record.mutation_occurred.to_string(),
        );
        metadata.insert("source".into(), format_opt(record.source));
        metadata.insert("target".into(), format_opt(record.target));
        metadata.insert(
            "raw_abs_normal".into(),
            format!("{:.17e}", record.raw_abs_normal),
        );
        metadata.insert(
            "applied_delta".into(),
            format!("{:.17e}", record.applied_delta),
        );
        metadata.insert(
            "pre_genotype".into(),
            serde_json::to_string(&record.pre_genotype).unwrap(),
        );
        metadata.insert(
            "post_genotype".into(),
            serde_json::to_string(&record.post_genotype).unwrap(),
        );
        metadata.insert(
            "candidate_hash".into(),
            record.post_genotype.candidate_hash(&params),
        );
        Ok(Some(metadata))
    }
}

fn format_opt(value: Option<usize>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "none".into())
}

fn lawful_d096_copy(
    parent: AllocationGenotype,
    child: AllocationGenotype,
    params: &AllocationParams,
) -> bool {
    if parent == child {
        return true;
    }
    if !child.valid(params) {
        return false;
    }
    let deltas: [f64; chemistry_core::d096_allocation::FUNCTIONS] =
        std::array::from_fn(|i| child.0[i] - parent.0[i]);
    let changed = deltas
        .iter()
        .enumerate()
        .filter(|(_, delta)| delta.abs() > 1e-12)
        .collect::<Vec<_>>();
    if changed.len() != 2 || deltas.iter().sum::<f64>().abs() > 1e-12 {
        return false;
    }
    let positive = changed.iter().filter(|(_, delta)| **delta > 0.0).count();
    let negative = changed.iter().filter(|(_, delta)| **delta < 0.0).count();
    positive == 1 && negative == 1 && (changed[0].1.abs() - changed[1].1.abs()).abs() <= 1e-12
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FounderIdentityV1, FounderInitializationContext, OrganismAdapter};

    fn real_fission_parent(
        genotype: AllocationGenotype,
        params: &AllocationParams,
    ) -> MaterialMesh {
        let mut parent = MaterialMesh::seed_regular(
            12,
            8.0,
            0.0,
            0.0,
            1.0,
            0.8,
            LumpedChem::default(),
            LumpedChem::default(),
            1.0,
        );
        let center = parent.centroid();
        for vertex in &mut parent.vertices {
            vertex[0] = center[0] + (vertex[0] - center[0]) * 1.55;
            vertex[1] = center[1] + (vertex[1] - center[1]) * 0.72;
        }
        parent.interior.a = 2.0;
        parent.interior.c = 1.0;
        parent.enable_finite_allocation(genotype, params);
        parent.finite_allocation = Some(AllocationState {
            genotype,
            catalysts: [0.11, 0.22, 0.33, 0.44],
        });
        parent
    }

    fn d096_mutation_protocol() -> MutationProtocolV1 {
        MutationProtocolV1 {
            schema: "MutationProtocolV1".into(),
            mutation_protocol_id: "d096_allocation_mutation_v1".into(),
            mutation_rate: FROZEN_D096_MUTATION_PROBABILITY,
            magnitude_distribution: "abs_normal".into(),
            bounds: "simplex_and_allocation_bounds".into(),
            provenance: "DC-SR-004B;D-096_GATE6;test".into(),
        }
    }

    #[test]
    fn d096_live_qualified_fission_copy_enters_mutation_adapter() {
        let params = AllocationParams::default();
        let mut adapter = DigitalCellMeshAdapter {
            allocation_params: Some(params),
            ..DigitalCellMeshAdapter::default()
        }
        .with_d096_founder(AllocationGenotype::neutral());
        let founder = FounderIdentityV1::new(
            1,
            "digital_cell_mesh_v1",
            "d096",
            "baseline",
            "material",
            7,
            "none",
        );
        let context = FounderInitializationContext {
            replicate: 0,
            founder_index: 0,
            population_size: 1,
            placement: [0.0, 0.0],
        };
        let parent = adapter
            .initialize_founder(&founder, context)
            .expect("explicit D-096 founder should initialize");
        let mut live_daughter = parent.clone();
        assert!(parent.alive && live_daughter.alive);

        let protocol = MutationProtocolV1 {
            schema: "MutationProtocolV1".into(),
            mutation_protocol_id: "d096_allocation_mutation_v1".into(),
            mutation_rate: FROZEN_D096_MUTATION_PROBABILITY,
            magnitude_distribution: "abs_normal".into(),
            bounds: "simplex_and_allocation_bounds".into(),
            provenance: "DC-SR-004B;D-096_GATE6;test".into(),
        };
        let mutation_context = MutationContext {
            accepted_step: 1,
            accepted_simulated_time: adapter.accepted_dt(),
            seed: 99,
            offspring_index: 0,
            qualified_physical_copy: true,
            qualified_copy_ordinal: 0,
            parent_hereditary_state: adapter.hereditary_state(&parent),
        };

        let metadata = adapter
            .apply_heredity_and_mutation(&parent, &mut live_daughter, &protocol, &mutation_context)
            .expect("qualified live daughter should be accepted");
        let metadata = metadata.expect("D-096 mutation provenance should be recorded");
        assert_eq!(
            metadata.get("qualified_copy_ordinal").map(String::as_str),
            Some("0")
        );
        assert!(metadata.get("candidate_hash").is_some());

        let mut invalid_daughter = parent.clone();
        let mut invalid_protocol = protocol.clone();
        invalid_protocol.mutation_rate = 0.20;
        assert!(matches!(
            adapter.apply_heredity_and_mutation(
                &parent,
                &mut invalid_daughter,
                &invalid_protocol,
                &mutation_context,
            ),
            Err(AdapterError::Advance(_))
        ));
    }

    #[test]
    fn d096_adapter_reports_copy_and_mutation_provenance_separately() {
        let params = AllocationParams::default();
        let mut adapter = DigitalCellMeshAdapter {
            allocation_params: Some(params),
            ..DigitalCellMeshAdapter::default()
        }
        .with_d096_founder(AllocationGenotype::neutral());
        let founder = FounderIdentityV1::new(
            1,
            "digital_cell_mesh_v1",
            "d096",
            "baseline",
            "material",
            7,
            "none",
        );
        let context = FounderInitializationContext {
            replicate: 0,
            founder_index: 0,
            population_size: 1,
            placement: [0.0, 0.0],
        };
        let parent = adapter.initialize_founder(&founder, context).unwrap();
        let protocol = MutationProtocolV1 {
            schema: "MutationProtocolV1".into(),
            mutation_protocol_id: "d096_allocation_mutation_v1".into(),
            mutation_rate: FROZEN_D096_MUTATION_PROBABILITY,
            magnitude_distribution: "abs_normal".into(),
            bounds: "simplex_and_allocation_bounds".into(),
            provenance: "DC-SR-004B;D-096_GATE6;test".into(),
        };

        let mut mutation_offspring = parent.clone();
        let mutation_none = MutationProtocolV1::default();
        let mutation_none_context = MutationContext {
            accepted_step: 1,
            accepted_simulated_time: adapter.accepted_dt(),
            seed: 0,
            offspring_index: 0,
            qualified_physical_copy: true,
            qualified_copy_ordinal: 0,
            parent_hereditary_state: adapter.hereditary_state(&parent),
        };
        assert_eq!(
            adapter
                .apply_heredity_and_mutation(
                    &parent,
                    &mut mutation_offspring,
                    &mutation_none,
                    &mutation_none_context,
                )
                .unwrap(),
            None
        );
        assert_eq!(
            mutation_offspring.finite_allocation.unwrap().genotype,
            parent.finite_allocation.unwrap().genotype
        );

        let mut saw_copy = false;
        let mut saw_mutation = false;
        for seed in 0..10_000_u64 {
            let mut offspring = parent.clone();
            let metadata = adapter
                .apply_heredity_and_mutation(
                    &parent,
                    &mut offspring,
                    &protocol,
                    &MutationContext {
                        accepted_step: 1,
                        accepted_simulated_time: adapter.accepted_dt(),
                        seed,
                        offspring_index: 0,
                        qualified_physical_copy: true,
                        qualified_copy_ordinal: seed,
                        parent_hereditary_state: adapter.hereditary_state(&parent),
                    },
                )
                .unwrap()
                .unwrap();
            let occurred = metadata.get("mutation_occurred").map(String::as_str);
            let pre = metadata.get("pre_genotype").unwrap();
            let post = metadata.get("post_genotype").unwrap();
            if occurred == Some("false") {
                saw_copy = true;
                assert_eq!(pre, post);
                assert_eq!(
                    metadata.get("qualified_copy_ordinal"),
                    Some(&seed.to_string())
                );
            } else if occurred == Some("true") {
                saw_mutation = true;
                assert_ne!(pre, post);
                assert!(metadata.get("candidate_hash").is_some());
            }
            if saw_copy && saw_mutation {
                break;
            }
        }
        assert!(saw_copy && saw_mutation);
    }

    #[test]
    fn d096_real_fission_daughters_close_mutation_and_expression_continuity() {
        let params = AllocationParams::default();
        let protocol = d096_mutation_protocol();
        let mutation_none = MutationProtocolV1::default();
        let mut adapter = DigitalCellMeshAdapter {
            allocation_params: Some(params),
            ..DigitalCellMeshAdapter::default()
        };

        let founder_classes = [
            AllocationGenotype([0.55, 0.25, 0.05, 0.15]),
            AllocationGenotype([0.10, 0.20, 0.55, 0.15]),
            AllocationGenotype::neutral(),
        ];
        for genotype in founder_classes {
            let parent = real_fission_parent(genotype, &params);
            let (daughter_a, daughter_b, event) =
                try_local_fission(&parent, &FissionParams::default())
                    .expect("the controlled D-096 parent must physically fission");
            assert!(
                event
                    .partition
                    .catalyst_partition
                    .expect("D-096 fission audit")
                    .conserved
            );
            for mut daughter in [daughter_a, daughter_b] {
                let before = daughter.finite_allocation.expect("D-096 daughter state");
                let before_hash = before.genotype.candidate_hash(&params);
                let result = adapter
                    .apply_heredity_and_mutation(
                        &parent,
                        &mut daughter,
                        &mutation_none,
                        &MutationContext {
                            accepted_step: 1,
                            accepted_simulated_time: adapter.accepted_dt(),
                            seed: 7,
                            offspring_index: 0,
                            qualified_physical_copy: true,
                            qualified_copy_ordinal: 0,
                            parent_hereditary_state: adapter.hereditary_state(&parent),
                        },
                    )
                    .expect("mutation_none must accept the physical daughter");
                assert_eq!(result, None);
                let after = daughter.finite_allocation.expect("D-096 daughter state");
                assert_eq!(after.genotype, parent.finite_allocation.unwrap().genotype);
                assert_eq!(after.genotype.candidate_hash(&params), before_hash);
                assert_eq!(after.catalysts, before.catalysts);
            }
        }

        let parent = real_fission_parent(AllocationGenotype::neutral(), &params);
        let (daughter_a, daughter_b, event) =
            try_local_fission(&parent, &FissionParams::default()).expect("real D-096 fission");
        assert!(daughter_a.alive && daughter_b.alive);
        assert!(
            event
                .partition
                .catalyst_partition
                .expect("D-096 fission audit")
                .conserved
        );

        let daughter_catalysts = daughter_a.finite_allocation.unwrap().catalysts;
        let mut mutated_daughter = None;
        for ordinal in 0..10_000_u64 {
            let mut candidate = daughter_a.clone();
            let metadata = adapter
                .apply_heredity_and_mutation(
                    &parent,
                    &mut candidate,
                    &protocol,
                    &MutationContext {
                        accepted_step: 1,
                        accepted_simulated_time: adapter.accepted_dt(),
                        seed: crate::d096_mutation_stream_seed(17, ordinal),
                        offspring_index: 0,
                        qualified_physical_copy: true,
                        qualified_copy_ordinal: ordinal,
                        parent_hereditary_state: adapter.hereditary_state(&parent),
                    },
                )
                .expect("mutation-on must accept the real fission daughter")
                .expect("mutation-on provenance must be present");
            assert_eq!(
                metadata.get("qualified_copy_ordinal"),
                Some(&ordinal.to_string())
            );
            if metadata.get("mutation_occurred").map(String::as_str) == Some("true") {
                assert_ne!(
                    candidate.finite_allocation.unwrap().genotype,
                    parent.finite_allocation.unwrap().genotype
                );
                assert_eq!(
                    candidate.finite_allocation.unwrap().catalysts,
                    daughter_catalysts
                );
                mutated_daughter = Some(candidate);
                break;
            }
        }
        let mut mutated_daughter = mutated_daughter.expect("frozen p=0.01 mutation observed");
        let post_genotype = mutated_daughter.finite_allocation.unwrap().genotype;
        let pre_genotype = parent.finite_allocation.unwrap().genotype;
        let source = (0..4)
            .find(|&index| post_genotype.0[index] < pre_genotype.0[index])
            .expect("mutation source");
        let target = (0..4)
            .find(|&index| post_genotype.0[index] > pre_genotype.0[index])
            .expect("mutation target");

        let mut inherited_control = mutated_daughter.clone();
        inherited_control
            .finite_allocation
            .as_mut()
            .unwrap()
            .genotype = pre_genotype;
        let mut mutated_synthesis = [0.0; 4];
        let mut inherited_synthesis = [0.0; 4];
        for _ in 0..3 {
            let mutated_ledger = expression_step(&mut mutated_daughter, &params, 0.1)
                .expect("mutated daughter expression");
            let inherited_ledger = expression_step(&mut inherited_control, &params, 0.1)
                .expect("inherited control expression");
            for index in 0..4 {
                mutated_synthesis[index] += mutated_ledger.synthesis[index];
                inherited_synthesis[index] += inherited_ledger.synthesis[index];
            }
        }
        assert!(mutated_synthesis[target] > inherited_synthesis[target]);
        assert!(mutated_synthesis[source] < inherited_synthesis[source]);
    }
}
