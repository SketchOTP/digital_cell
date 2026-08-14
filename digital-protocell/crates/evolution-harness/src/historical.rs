use crate::{DamageMode, ExperimentProtocolV1, ProtocolProvenanceV1, ResourceMode};
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

pub fn d094_requalified_protocol() -> ExperimentProtocolV1 {
    let mut protocol = sealed_fixture(
        "d094_requalified_translation",
        "d094_autocatalytic_set",
        "digital_cell_material_mesh_v1",
        "d094_autocatalytic_set_heredity",
        &[
            (
                "historical_contract",
                "digital-protocell/docs/d094_distributed_autocatalytic_set.md",
            ),
            (
                "sealed_r2_evidence",
                "digital-protocell/docs/d094r2_gate6_execution.md",
            ),
            (
                "selection_analysis",
                "digital-protocell/docs/d094r2_selection_analysis.md",
            ),
        ],
        &[
            "founder artifact/config reference",
            "treatment-neutral field difference",
            "replicate count and seed campaign",
            "pulse interval and pulse magnitude",
            "damage interval and damage magnitude",
            "minimum viable population threshold",
            "accepted horizon and generation gate",
        ],
    );
    protocol.environment_protocol.resource_mode = ResourceMode::Pulsed;
    protocol.environment_protocol.damage_mode = DamageMode::Abrasion;
    protocol.environment_protocol.pulse_schedule.clear();
    protocol.environment_protocol.damage_interval = None;
    protocol.replicates = 1;
    protocol.random_seeds = vec![0];
    protocol.provenance.execution_authorized = false;
    protocol
}
