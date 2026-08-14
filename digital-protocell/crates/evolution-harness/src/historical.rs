use crate::{DamageMode, EnvironmentProtocolV1, ExperimentProtocolV1, ResourceMode};

pub fn historical_protocols() -> Vec<ExperimentProtocolV1> {
    [
        ("d090_style_ecology", "d090_spatial_shared_dish_v1"),
        ("d091_style_seasonal_reserve", "d091_seasonal_lean_v1"),
        ("d092_style_selection", "d092_template_selection_v1"),
        ("d093_style_generation_counting", "d093_network_generation_v1"),
    ]
    .into_iter()
    .map(|(experiment, environment)| {
        let mut protocol = ExperimentProtocolV1::minimal(experiment, environment, "historical_unchanged");
        protocol.organism_schema = "digital_cell_existing_organism".into();
        protocol.heredity_schema = experiment.replace("_style", "");
        protocol.replicates = 1;
        protocol.random_seeds = vec![0];
        protocol
    })
    .collect()
}

pub fn d094_requalified_protocol() -> ExperimentProtocolV1 {
    let mut protocol = ExperimentProtocolV1::minimal("d094_requalified_translation", "d094_autocatalytic_ecology_v1", "d094_existing_mutation_contract");
    protocol.organism_schema = "digital_cell_material_mesh_v1".into();
    protocol.heredity_schema = "d094_autocatalytic_set_heredity_existing".into();
    protocol.replicates = 24;
    protocol.random_seeds = (0..24).collect();
    protocol.maximum_accepted_horizon = 250_000;
    protocol.maximum_generation = 8;
    protocol.minimum_generation_requirement = 2;
    protocol.primary_endpoints = vec![
        "completed_generations".into(),
        "descendant_count".into(),
        "treatment_relative_persistence".into(),
    ];
    protocol.secondary_endpoints = vec![
        "hereditary_state_frequency".into(),
        "resource_consumption".into(),
        "lineage_survival".into(),
    ];
    protocol.termination_rules = vec![
        "accepted_horizon".into(),
        "global_extinction".into(),
        "minimum_population_viability".into(),
    ];
    protocol.environment_protocol.resource_mode = ResourceMode::Pulsed;
    protocol.environment_protocol.damage_mode = DamageMode::Abrasion;
    protocol.environment_protocol.pulse_schedule = vec![0, 1000, 2000];
    protocol.environment_protocol.damage_interval = Some(350);
    protocol
}
