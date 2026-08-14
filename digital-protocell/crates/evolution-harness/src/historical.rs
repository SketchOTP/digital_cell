use crate::{
    d094_pressure_contract, CampaignRole, DamageMode, ExperimentProtocolV1,
    PlacementProtocolV1, ProtocolProvenanceV1, ResourceMode, ResourceVectorV1,
};
use std::collections::BTreeMap;

fn sealed_fixture(
    experiment_id: &str,
    environment_id: &str,
    organism_schema: &str,
    heredity_schema: &str,
    sources: &[(&str, &str)],
    unresolved: &[&str],
) -> ExperimentProtocolV1 {
    let mut protocol =
        ExperimentProtocolV1::minimal(experiment_id, environment_id, "mutation_none");
    protocol.organism_schema = organism_schema.into();
    protocol.heredity_schema = heredity_schema.into();
    protocol.provenance = ProtocolProvenanceV1 {
        source_artifacts: sources
            .iter()
            .map(|(field, path)| ((*field).into(), (*path).into()))
            .collect::<BTreeMap<_, _>>(),
        derived_values: BTreeMap::new(),
        unresolved_values: unresolved.iter().map(|value| (*value).into()).collect(),
        execution_authorized: false,
    };
    protocol
}

/// Sealed mappings are evidence fixtures, not executable replacements for the
/// historical runners. Every unresolved runtime-dependent value is explicit.
pub fn historical_protocols() -> Vec<ExperimentProtocolV1> {
    let mut d090 = sealed_fixture(
        "d090_historical_mapping",
        "d090_spatial_shared_dish",
        "digital_cell_material_mesh_v1",
        "d089_composition_heredity",
        &[
            (
                "environment",
                "digital-protocell/crates/chemistry-core/src/d090_analysis.rs",
            ),
            (
                "dish",
                "digital-protocell/crates/chemistry-core/src/d090_dish.rs",
            ),
            (
                "sealed_result",
                "digital-protocell/docs/d090_ecological_selection_requalification.md",
            ),
        ],
        &[
            "campaign_steps depends on D090_SMOKE",
            "n_each depends on D090_SMOKE",
            "selected ecology row must be chosen from generated D-090 artifact",
        ],
    );
    d090.replicates = 4;
    d090.random_seeds = vec![100, 101, 102, 103];
    d090.environment_protocol.resource_mode = ResourceMode::Shared;
    d090.environment_protocol
        .resource_ecology
        .shared_resource_competition = true;
    d090.environment_protocol
        .resource_ecology
        .spatial_local_availability = true;
    d090.environment_protocol.spatial_constraints =
        "SpatialDish::new(8,8,2.5,[0,0],n0,f0,supply,supply,3.0)".into();
    d090.provenance.derived_values.insert(
        "random_seeds".into(),
        "seed = 100 + i; full campaign n_each() = 4".into(),
    );

    let mut d091 = sealed_fixture(
        "d091_historical_mapping",
        "d091_pulse_lean_abrasion",
        "digital_cell_material_mesh_metabolic_reserve_v1",
        "d091_reserve_heredity",
        &[
            (
                "environment",
                "digital-protocell/crates/chemistry-core/src/d091_analysis.rs",
            ),
            (
                "seasonal",
                "digital-protocell/crates/chemistry-core/src/seasonal_ecology.rs",
            ),
            (
                "sealed_result",
                "digital-protocell/docs/d091_metabolic_reserve_ecological_timescale.md",
            ),
        ],
        &[
            "t_maint is derived by the sealed runner at runtime",
            "selected pulse period is a sweep over PULSE_PERIOD_MULTS",
            "selected abrasion strength is a sweep over ABRASION_STRENGTHS",
        ],
    );
    d091.replicates = 4;
    d091.random_seeds = vec![100, 101, 102, 103];
    d091.environment_protocol.resource_mode = ResourceMode::Pulsed;
    d091.environment_protocol
        .resource_ecology
        .shared_resource_competition = true;
    d091.environment_protocol
        .resource_ecology
        .spatial_local_availability = true;
    d091.environment_protocol.damage_mode = DamageMode::Abrasion;
    d091.environment_protocol.spatial_constraints =
        "SpatialDish::new(8,8,2.5,[0,0],120,120,0,0,3.0)".into();
    d091.provenance.derived_values.insert(
        "founder_seeds".into(),
        "sealed Gate 5 H seeds 100..103; B seeds 200..203".into(),
    );

    let mut d092 = sealed_fixture(
        "d092_historical_mapping",
        "d092_template_motif_selection",
        "autopoietic_material_mesh_template_v1",
        "d092_template_heredity",
        &[
            (
                "environment",
                "digital-protocell/crates/chemistry-core/src/d092_analysis.rs",
            ),
            (
                "templates",
                "digital-protocell/crates/chemistry-core/src/template_polymer.rs",
            ),
            (
                "sealed_result",
                "digital-protocell/docs/d092_minimal_catalytic_template_heredity.md",
            ),
        ],
        &[
            "selected pulse period is derived from t_maint and PULSE_PERIOD_MULTS",
            "founder topology and mutation campaign values are runner-controlled",
        ],
    );
    d092.replicates = 4;
    d092.random_seeds = vec![100, 101, 102, 103];
    d092.environment_protocol.resource_mode = ResourceMode::Pulsed;
    d092.environment_protocol.damage_mode = DamageMode::Abrasion;
    d092.environment_protocol.spatial_constraints =
        "SpatialDish::new(8,8,2.5,[0,0],120,120,0,0,3.0)".into();

    let mut d093 = sealed_fixture(
        "d093_historical_mapping",
        "d093_template_network_selection",
        "autopoietic_material_mesh_template_network_v1",
        "d093_network_heredity",
        &[
            (
                "environment",
                "digital-protocell/crates/chemistry-core/src/d093_analysis.rs",
            ),
            (
                "network",
                "digital-protocell/crates/chemistry-core/src/template_network.rs",
            ),
            (
                "sealed_result",
                "digital-protocell/docs/d093_template_encoded_catalytic_network.md",
            ),
        ],
        &[
            "selection replicate count is controlled by the sealed runner",
            "t_maint and pulse period are runtime-derived",
            "D-093 selection campaigns completed zero generations",
        ],
    );
    d093.replicates = 4;
    d093.random_seeds = vec![100, 101, 102, 103];
    d093.environment_protocol.resource_mode = ResourceMode::Pulsed;
    d093.environment_protocol.damage_mode = DamageMode::Abrasion;
    d093.environment_protocol.spatial_constraints =
        "SpatialDish::new(8,8,2.5,[0,0],120,120,0,0,3.0); circular L=12 pair sites".into();
    vec![d090, d091, d092, d093]
}

fn d094r2_protocol(
    experiment_id: &str,
    environment_id: &str,
    role: CampaignRole,
    treatment_environment: &str,
    neutral_environment: &str,
    condition: &str,
) -> ExperimentProtocolV1 {
    let mut protocol = sealed_fixture(
        experiment_id,
        environment_id,
        "autopoietic_material_mesh_autocatalytic_set_v1",
        "d094_autocatalytic_set_heredity_v1",
        &[
            (
                "architecture",
                "digital-protocell/docs/d094_distributed_autocatalytic_set.md",
            ),
            (
                "execution_contract",
                "digital-protocell/docs/d094r2_gate6_execution.md",
            ),
            (
                "selection_analysis",
                "digital-protocell/docs/d094r2_selection_analysis.md",
            ),
            (
                "sealed_attempt_manifest",
                "digital-protocell/experiments/generated/d094r/gate6/attempt_001/manifest.json",
            ),
            (
                "source_implementation",
                "digital-protocell/crates/chemistry-core/src/d094_selection.rs@bf58edddef40753107ba18854eb85cc41ec78859",
            ),
        ],
        &[
            "D-094-specific heredity and phenotype adapter is observer-only and not executable",
            "historical per-organism material initialization hash was not emitted",
            "historical placement coordinates were not emitted",
            "minimum viable population threshold was not preregistered as a numeric value",
            "phenotype differential endpoints were not recorded by D-094R2",
            "finite shared-resource state was not present in the parallel-lineage runner",
        ],
    );

    protocol.selective_pressure = Some(d094_pressure_contract(
        if treatment_environment.ends_with("h_ecology") {
            "d094r2_h_ecology_contrast"
        } else {
            "d094r2_b_ecology_contrast"
        },
        treatment_environment,
        neutral_environment,
        role,
        condition,
    ));
    protocol.replicates = 8;
    protocol.random_seeds = (0..8).collect();
    protocol.maximum_accepted_horizon = 22_000.0 * 0.02;
    protocol.maximum_generation = 8;
    protocol.minimum_generation_requirement = 8;
    // Zero is a provenance sentinel: the sealed runner emitted no numeric
    // viability threshold.  It cannot authorize execution.
    protocol.minimum_population_viability = 0;
    protocol.placement_protocol = PlacementProtocolV1 {
        schema: "PlacementProtocolV1".into(),
        initial_coordinates: Vec::new(),
        spacing: 0.0,
        founder_seed: 0,
        random_seed: None,
        dish_geometry: "parallel_single_founder_lineages_no_shared_space".into(),
        resource_geometry: "common_environment_clock_independent_resource_state".into(),
    };
    protocol.environment_protocol.resource_field =
        "per_organism_exterior_NF_override_from_d094_selection_source".into();
    protocol.environment_protocol.spatial_constraints =
        "none: independent single-founder lineages".into();
    protocol.environment_protocol.resource_ecology.shared_resource_competition = false;
    protocol.environment_protocol.resource_ecology.spatial_local_availability = false;
    protocol.environment_protocol.resource_ecology.continuous_supply = ResourceVectorV1 {
        n: if environment_id.ends_with("h_ecology") {
            2.2 * 0.18
        } else if environment_id.ends_with("b_ecology") {
            2.2 * 1.20
        } else {
            2.2 * 0.70
        },
        f: if environment_id.ends_with("h_ecology") {
            2.2 * 0.18
        } else if environment_id.ends_with("b_ecology") {
            2.2 * 1.20
        } else {
            2.2 * 0.70
        },
        w: 0.0,
    };
    protocol.environment_protocol.resource_ecology.pulse_delta = ResourceVectorV1::default();
    protocol.environment_protocol.resource_ecology.pulse_schedule.clear();
    protocol.environment_protocol.pulse_schedule.clear();
    protocol.environment_protocol.damage_mode = if environment_id.ends_with("b_ecology") {
        DamageMode::Abrasion
    } else {
        DamageMode::None
    };
    protocol.environment_protocol.damage_interval = None;
    protocol.environment_protocol.resource_ecology.damage_mode =
        protocol.environment_protocol.damage_mode.clone();
    protocol.environment_protocol.resource_ecology.damage_interval = None;
    protocol.environment_protocol.resource_ecology.damage_fraction = 0.0;
    protocol.provenance.derived_values.extend([
        (
            "founder_count_and_seed_rule".into(),
            "8 independent lineages per replicate; H seed=100+rep*20+i; B seed=200+rep*20+i; i=0..3".into(),
        ),
        (
            "resource_rich_level".into(),
            "rich=2.2 from run_campaign".into(),
        ),
        (
            "H_resource_schedule".into(),
            "pulse absolute exterior N/F=rich*1.25; lean absolute exterior N/F=rich*0.18; pulse fraction=0.40; cycle=0.5*t_maint*4".into(),
        ),
        (
            "B_resource_schedule".into(),
            "absolute exterior N/F=rich*1.20; abrasion fires every 1.5*t_maint with ABRASION_STRENGTHS[0] and membrane factor 0.6".into(),
        ),
        (
            "neutral_resource_schedule".into(),
            "absolute exterior N/F=rich*0.70; autocatalytic rho_node=0 via with_baseline_efficiencies".into(),
        ),
        (
            "reserve_parameters".into(),
            "ReserveParams::derived(80.0,40.0,0.5,0.3,2.0,0.1,area); area runtime-derived from seed mesh".into(),
        ),
        (
            "autocatalytic_parameters".into(),
            "AutocatalyticParams::derived(1/k_release) with mutation_off; mu_E=0.0 in Gate 6; historical frozen default mu_E=0.0089".into(),
        ),
        (
            "mechanical_growth_fission".into(),
            "MechParams::default(dt=0.02); GrowthParams{y_g=0.9,enable_growth=true}; FissionParams::default()".into(),
        ),
        (
            "generation_and_horizon".into(),
            "target=8 generations; n_steps=22000; accepted horizon=440.0".into(),
        ),
        (
            "selection_thresholds".into(),
            "frequency delta requirement=0.15; descendant ratio requirement=1.20x; no numeric minimum viability emitted".into(),
        ),
        (
            "shared_resource_classification".into(),
            "COMMON_ENVIRONMENT_INDEPENDENT_RESOURCES: lineages share a clock, not a finite mutable resource pool".into(),
        ),
    ]);
    protocol
}

/// Exact D-094R2 contract translations. They intentionally remain
/// provenance-gated and non-executable because the accepted harness has no
/// D-094-specific organism adapter or finite shared-resource implementation.
pub fn d094r2_protocols() -> Vec<ExperimentProtocolV1> {
    vec![
        d094r2_protocol(
            "d094r2_h_treatment",
            "d094r2_h_ecology",
            CampaignRole::Treatment,
            "d094r2_h_ecology",
            "d094r2_neutral_ecology",
            "H pulse/lean resource schedule",
        ),
        d094r2_protocol(
            "d094r2_b_treatment",
            "d094r2_b_ecology",
            CampaignRole::Treatment,
            "d094r2_b_ecology",
            "d094r2_neutral_ecology",
            "B identity-blind abrasion schedule",
        ),
        d094r2_protocol(
            "d094r2_h_neutral",
            "d094r2_neutral_ecology",
            CampaignRole::Neutral,
            "d094r2_h_ecology",
            "d094r2_neutral_ecology",
            "H pulse/lean resource schedule",
        ),
        d094r2_protocol(
            "d094r2_b_neutral",
            "d094r2_neutral_ecology",
            CampaignRole::Neutral,
            "d094r2_b_ecology",
            "d094r2_neutral_ecology",
            "B identity-blind abrasion schedule",
        ),
    ]
}

pub fn d094_requalified_protocol() -> ExperimentProtocolV1 {
    let mut protocols = d094r2_protocols();
    protocols.remove(0)
}
