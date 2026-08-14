use chemistry_core::d096_allocation::{
    AllocationGenotype, AllocationParams, AssayEnvironment,
    EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION,
};
use evolution_harness::{
    DigitalCellMeshAdapter, EnvironmentProtocolV1, EventType, EventV1, EvolutionHarness,
    ExperimentProtocolV1, FounderIdentityV1, MutationProtocolV1, PopulationState,
    ProtocolProvenanceV1, ResourceMode,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY_COMMIT: &str = "f4adec147f6d8c0763d1264b8954eaf2788edcca";
const HORIZON_STEPS: u64 = 4_000;
const DT: f64 = 0.02;
const HORIZON_TIME: f64 = 80.0;
const BOOTSTRAP_RESAMPLES: usize = 10_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
enum Gate7Environment {
    H,
    B,
    Neutral,
}

impl Gate7Environment {
    fn id(self) -> &'static str {
        match self {
            Self::H => "H",
            Self::B => "B",
            Self::Neutral => "Neutral",
        }
    }

    fn assay(self) -> AssayEnvironment {
        match self {
            Self::H => AssayEnvironment::H,
            Self::B => AssayEnvironment::B,
            Self::Neutral => AssayEnvironment::Neutral,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum CandidateClass {
    ProcessingHeavy,
    RepairHeavy,
    Neutral,
}

impl CandidateClass {
    fn id(self) -> &'static str {
        match self {
            Self::ProcessingHeavy => "processing-heavy",
            Self::RepairHeavy => "repair-heavy",
            Self::Neutral => "neutral",
        }
    }

    fn genotype(self) -> AllocationGenotype {
        match self {
            Self::ProcessingHeavy => AllocationGenotype([0.55, 0.25, 0.05, 0.15]),
            Self::RepairHeavy => AllocationGenotype([0.10, 0.20, 0.55, 0.15]),
            Self::Neutral => AllocationGenotype::neutral(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Gate7Endpoint {
    candidate: CandidateClass,
    candidate_id: String,
    candidate_hash: String,
    environment: Gate7Environment,
    environment_id: String,
    replicate: u32,
    seed: u64,
    first_fission_by_horizon: bool,
    first_fission_time: Option<f64>,
    restricted_reproduction_time: f64,
    founder_death_before_reproduction: bool,
    live_gen1_daughter_count_at_first_fission: u64,
    accepted_steps_at_stop: u64,
    accepted_simulated_time_at_stop: f64,
    partition_conserved: bool,
    every_birth_real_physical_fission: bool,
    maximum_generation_observed: u32,
    event_ledger_valid: bool,
    event_ledger_hash: String,
    mutation_protocol: String,
    treatment_label_leakage: bool,
    fitness_controller_leakage: bool,
    stop_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CellSummary {
    candidate: CandidateClass,
    environment: Gate7Environment,
    candidate_hash: String,
    environment_hash: String,
    n: u64,
    fission_count: u64,
    death_before_fission_count: u64,
    horizon_without_fission_count: u64,
    mean_restricted_reproduction_time: f64,
    min_restricted_reproduction_time: f64,
    max_restricted_reproduction_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BootstrapInterval {
    mean: f64,
    lower_95: f64,
    upper_95: f64,
    resamples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PairedContrast {
    label: String,
    environment: Gate7Environment,
    left_candidate: CandidateClass,
    right_candidate: CandidateClass,
    expected_direction: String,
    paired_differences: Vec<f64>,
    mean_difference: f64,
    direction_count_expected: u64,
    bootstrap_95_ci: BootstrapInterval,
    expected_direction_at_least_12_of_16: bool,
    ci_excludes_zero_in_expected_direction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PairedAnalysis {
    h: PairedContrast,
    b: PairedContrast,
    neutral: PairedContrast,
    neutral_abs_mean_smaller_than_h: bool,
    neutral_abs_mean_smaller_than_b: bool,
    primary_pass_h: bool,
    primary_pass_b: bool,
    primary_pass: bool,
    interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProtocolBundle {
    schema: &'static str,
    entry_commit: &'static str,
    mutation_protocol: MutationProtocolV1,
    candidates: Vec<CandidateRecord>,
    environments: Vec<EnvironmentRecord>,
    protocols: Vec<ExperimentProtocolV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidateRecord {
    candidate: CandidateClass,
    candidate_id: String,
    genotype: AllocationGenotype,
    candidate_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnvironmentRecord {
    environment: Gate7Environment,
    environment_id: String,
    environment_hash: String,
    forcing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FinalManifest {
    schema: &'static str,
    entry_commit: &'static str,
    ending_commit: String,
    gate6_authority: &'static str,
    mutation_protocol: String,
    horizon_steps: u64,
    dt: f64,
    maximum_accepted_simulated_time: f64,
    paired_seeds: Vec<u64>,
    candidate_hashes: BTreeMap<String, String>,
    environment_hashes: BTreeMap<String, String>,
    result_artifact_hashes: BTreeMap<String, String>,
    maximum_generation_observed: u32,
    gate8_executed: bool,
    d094_executed: bool,
    selection_operator_executed: bool,
}

fn mutation_none() -> MutationProtocolV1 {
    MutationProtocolV1::default()
}

fn protocol_for(environment: Gate7Environment, params: &AllocationParams) -> ExperimentProtocolV1 {
    let mut protocol = ExperimentProtocolV1::minimal(
        &format!("d096_gate7_{}", environment.id().to_lowercase()),
        environment.id(),
        "mutation_none",
    );
    let mut env = EnvironmentProtocolV1::new(environment.id());
    env.resource_mode = match environment {
        Gate7Environment::H => ResourceMode::Pulsed,
        Gate7Environment::B | Gate7Environment::Neutral => ResourceMode::Continuous,
    };
    env.resource_field = "D096_exact_assay_environment".into();
    env.rich_duration = 2.0;
    env.lean_duration = 6.0;
    if environment == Gate7Environment::H {
        env.pulse_schedule = (0..10)
            .flat_map(|cycle| [cycle as f64 * 8.0, cycle as f64 * 8.0 + 2.0])
            .collect();
    }
    env.damage_mode = if environment == Gate7Environment::B {
        evolution_harness::DamageMode::DeclaredExternal
    } else {
        evolution_harness::DamageMode::None
    };
    env.damage_interval = if environment == Gate7Environment::B {
        Some(7.0)
    } else {
        None
    };
    env.duration = HORIZON_TIME;
    env.termination_rules = vec![
        "first_founder_fission".into(),
        "founder_death".into(),
        "accepted_horizon".into(),
    ];
    protocol.protocol_id = format!(
        "d096_gate7_{}_mutation_none",
        environment.id().to_lowercase()
    );
    protocol.organism_schema = EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION.into();
    protocol.heredity_schema = "D096AllocationGenotypeV1".into();
    protocol.mutation_protocol = mutation_none();
    protocol.environment_protocol = env;
    protocol.replicates = 16;
    protocol.random_seeds = (1..=16).collect();
    protocol.maximum_accepted_horizon = HORIZON_TIME;
    protocol.maximum_generation = 1;
    protocol.minimum_generation_requirement = 0;
    protocol.minimum_population_viability = 1;
    protocol.termination_rules = vec![
        "first_qualified_physical_fission".into(),
        "founder_death".into(),
        "accepted_horizon".into(),
    ];
    protocol.primary_endpoints = vec![
        "first_fission_by_horizon".into(),
        "time_to_first_qualified_fission".into(),
        "restricted_reproduction_time".into(),
        "founder_death_before_reproduction".into(),
        "live_gen1_daughter_count_at_first_fission".into(),
    ];
    protocol.secondary_endpoints = vec![
        "accepted_steps_at_stop".into(),
        "accepted_simulated_time_at_stop".into(),
        "partition_conserved".into(),
        "event_ledger_hash".into(),
    ];
    protocol.provenance = ProtocolProvenanceV1 {
        source_artifacts: BTreeMap::from([
            ("gate6_authority".into(), "f4adec147f6d8c0763d1264b8954eaf2788edcca".into()),
            ("d088_horizon".into(), "digital-protocell/crates/chemistry-core/src/d088_analysis.rs:steps(4000)".into()),
            ("d096_forcing".into(), "digital-protocell/crates/chemistry-core/src/d096_allocation.rs:apply_assay_environment".into()),
            ("d096_params".into(), "digital-protocell/crates/chemistry-core/src/d096_allocation.rs:AllocationParams::default".into()),
        ]),
        derived_values: BTreeMap::from([
            ("dt".into(), format!("MechParams::default().dt={DT}")),
            ("maximum_simulated_time".into(), format!("4000*{DT}={HORIZON_TIME}")),
            ("mutation_state".into(), "mutation_none control".into()),
            ("candidate_hash_basis".into(), "frozen D-096 AllocationParams".into()),
        ]),
        unresolved_values: Vec::new(),
        execution_authorized: true,
    };
    let _ = params;
    protocol
}

fn founder(candidate: CandidateClass, seed: u64, params: &AllocationParams) -> FounderIdentityV1 {
    let genotype = candidate.genotype();
    FounderIdentityV1::new(
        1,
        EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION,
        &genotype.candidate_hash(params),
        "D096_frozen_candidate",
        &format!("D096_material_seed_{seed}"),
        seed,
        "none;Gate7_single_founder",
    )
}

fn run_replicate(
    candidate: CandidateClass,
    environment: Gate7Environment,
    replicate: u32,
    seed: u64,
    params: AllocationParams,
) -> Result<Gate7Endpoint, String> {
    let protocol = protocol_for(environment, &params);
    let adapter = DigitalCellMeshAdapter {
        founder_radius: 14.0,
        allocation_params: Some(params),
        d096_founder_genotype: Some(candidate.genotype()),
        ..DigitalCellMeshAdapter::default()
    }
    .with_d096_assay_environment(environment.assay());
    let mut harness = EvolutionHarness::new(adapter, protocol.clone())
        .map_err(|error| error.to_string())?
        .with_replicate_seed(replicate, seed);
    harness
        .initialize_founder(founder(candidate, seed, &params))
        .map_err(|error| error.to_string())?;

    let mut stop_reason = "accepted_horizon".to_string();
    for _ in 0..HORIZON_STEPS {
        if harness.accepted_simulated_time >= HORIZON_TIME || harness.population.living_count() == 0
        {
            break;
        }
        harness.advance_one().map_err(|error| error.to_string())?;
        let fission = harness.ledger.events.iter().any(|event| {
            event.event_type == EventType::FissionCompleted && event.organism_id == Some(1)
        });
        let death = harness
            .ledger
            .events
            .iter()
            .any(|event| event.event_type == EventType::Death && event.organism_id == Some(1));
        if fission {
            stop_reason = "first_qualified_physical_fission".into();
            break;
        }
        if death {
            stop_reason = "founder_death".into();
            break;
        }
    }

    let fission_event = harness.ledger.events.iter().find(|event| {
        event.event_type == EventType::FissionCompleted && event.organism_id == Some(1)
    });
    let first_fission_time = fission_event.map(|event| event.accepted_simulated_time);
    let first_fission_by_horizon = first_fission_time.is_some();
    let founder_death = harness
        .ledger
        .events
        .iter()
        .any(|event| event.event_type == EventType::Death && event.organism_id == Some(1));
    let live_gen1_daughter_count_at_first_fission = if first_fission_by_horizon {
        harness
            .population
            .records
            .values()
            .filter(|record| {
                record.parent_id == Some(1)
                    && record.birth_generation == 1
                    && !matches!(
                        record.state,
                        PopulationState::Dead | PopulationState::Removed
                    )
            })
            .count() as u64
    } else {
        0
    };
    let every_birth_real_physical_fission = harness
        .ledger
        .events
        .iter()
        .all(|event| event.event_type != EventType::Birth || event.parent_id.is_some())
        && harness
            .ledger
            .events
            .iter()
            .filter(|event| event.event_type == EventType::Birth)
            .all(|event| event.metadata.get("qualified_physical_copy") == Some(&"true".into()));
    let partition_conserved = fission_event
        .and_then(|event| event.metadata.get("partition_ok"))
        .is_some_and(|value| value == "true");
    let maximum_generation_observed = harness
        .population
        .records
        .values()
        .map(|record| record.birth_generation)
        .max()
        .unwrap_or(0);
    assert!(maximum_generation_observed <= 1, "Gate 7 produced gen2");

    let end = EventV1::experiment_end(
        0,
        harness.accepted_simulated_time,
        harness.accepted_step,
        replicate,
        environment.id(),
        &protocol.protocol_id,
    );
    harness
        .ledger
        .append(end)
        .map_err(|error| error.to_string())?;
    let event_ledger_valid = harness.ledger.validate().is_ok();
    let event_ledger_hash = harness.ledger.hash().map_err(|error| error.to_string())?;
    let serialized_state =
        serde_json::to_string(&harness.organisms).map_err(|error| error.to_string())?;
    let leakage_terms = [
        "fitness",
        "fitness_score",
        "selection_score",
        "survival_probability",
        "winner",
        "preferred_candidate",
        "force_reproduce",
        "treatment",
    ];
    let treatment_label_leakage = serialized_state.contains("treatment")
        || harness
            .ledger
            .events
            .iter()
            .any(|event| event.metadata.keys().any(|key| key.contains("treatment")));
    let fitness_controller_leakage = leakage_terms[..7]
        .iter()
        .any(|term| serialized_state.contains(term));
    let restricted_reproduction_time = first_fission_time.unwrap_or(HORIZON_TIME);

    Ok(Gate7Endpoint {
        candidate,
        candidate_id: candidate.id().into(),
        candidate_hash: candidate.genotype().candidate_hash(&params),
        environment,
        environment_id: environment.id().into(),
        replicate,
        seed,
        first_fission_by_horizon,
        first_fission_time,
        restricted_reproduction_time,
        founder_death_before_reproduction: founder_death && !first_fission_by_horizon,
        live_gen1_daughter_count_at_first_fission,
        accepted_steps_at_stop: harness.accepted_step,
        accepted_simulated_time_at_stop: harness.accepted_simulated_time,
        partition_conserved,
        every_birth_real_physical_fission,
        maximum_generation_observed,
        event_ledger_valid,
        event_ledger_hash,
        mutation_protocol: "mutation_none".into(),
        treatment_label_leakage,
        fitness_controller_leakage,
        stop_reason,
    })
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

fn bootstrap_interval(values: &[f64]) -> BootstrapInterval {
    let observed = mean(values);
    let mut samples = Vec::with_capacity(BOOTSTRAP_RESAMPLES);
    for draw in 0..BOOTSTRAP_RESAMPLES {
        let total = (0..values.len())
            .map(|index| {
                values[(splitmix64(draw as u64 ^ (index as u64).rotate_left(17)) as usize)
                    % values.len()]
            })
            .sum::<f64>();
        samples.push(total / values.len() as f64);
    }
    samples.sort_by(f64::total_cmp);
    BootstrapInterval {
        mean: observed,
        lower_95: samples[BOOTSTRAP_RESAMPLES * 25 / 1_000],
        upper_95: samples[BOOTSTRAP_RESAMPLES * 975 / 1_000 - 1],
        resamples: BOOTSTRAP_RESAMPLES,
    }
}

fn paired_contrast(
    label: &str,
    environment: Gate7Environment,
    left: CandidateClass,
    right: CandidateClass,
    endpoints: &[Gate7Endpoint],
    expected_negative: bool,
) -> PairedContrast {
    let differences = (1..=16)
        .map(|seed| {
            let left_value = endpoints
                .iter()
                .find(|endpoint| {
                    endpoint.environment == environment
                        && endpoint.candidate == left
                        && endpoint.seed == seed
                })
                .unwrap()
                .restricted_reproduction_time;
            let right_value = endpoints
                .iter()
                .find(|endpoint| {
                    endpoint.environment == environment
                        && endpoint.candidate == right
                        && endpoint.seed == seed
                })
                .unwrap()
                .restricted_reproduction_time;
            left_value - right_value
        })
        .collect::<Vec<_>>();
    let ci = bootstrap_interval(&differences);
    let direction_count_expected = differences
        .iter()
        .filter(|difference| {
            if expected_negative {
                **difference < 0.0
            } else {
                **difference > 0.0
            }
        })
        .count() as u64;
    let ci_excludes_zero_in_expected_direction = if expected_negative {
        ci.upper_95 < 0.0
    } else {
        ci.lower_95 > 0.0
    };
    PairedContrast {
        label: label.into(),
        environment,
        left_candidate: left,
        right_candidate: right,
        expected_direction: if expected_negative {
            "negative"
        } else {
            "positive"
        }
        .into(),
        paired_differences: differences,
        mean_difference: ci.mean,
        direction_count_expected,
        bootstrap_95_ci: ci,
        expected_direction_at_least_12_of_16: direction_count_expected >= 12,
        ci_excludes_zero_in_expected_direction,
    }
}

fn summarize(
    candidate: CandidateClass,
    environment: Gate7Environment,
    endpoints: &[Gate7Endpoint],
    environment_hash: &str,
    params: &AllocationParams,
) -> CellSummary {
    let values = endpoints
        .iter()
        .filter(|endpoint| endpoint.candidate == candidate && endpoint.environment == environment)
        .map(|endpoint| endpoint.restricted_reproduction_time)
        .collect::<Vec<_>>();
    CellSummary {
        candidate,
        environment,
        candidate_hash: candidate.genotype().candidate_hash(params),
        environment_hash: environment_hash.into(),
        n: values.len() as u64,
        fission_count: endpoints
            .iter()
            .filter(|endpoint| {
                endpoint.candidate == candidate
                    && endpoint.environment == environment
                    && endpoint.first_fission_by_horizon
            })
            .count() as u64,
        death_before_fission_count: endpoints
            .iter()
            .filter(|endpoint| {
                endpoint.candidate == candidate
                    && endpoint.environment == environment
                    && endpoint.founder_death_before_reproduction
            })
            .count() as u64,
        horizon_without_fission_count: endpoints
            .iter()
            .filter(|endpoint| {
                endpoint.candidate == candidate
                    && endpoint.environment == environment
                    && !endpoint.first_fission_by_horizon
                    && !endpoint.founder_death_before_reproduction
            })
            .count() as u64,
        mean_restricted_reproduction_time: mean(&values),
        min_restricted_reproduction_time: values.iter().copied().fold(f64::INFINITY, f64::min),
        max_restricted_reproduction_time: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) {
    fs::write(path, serde_json::to_string_pretty(value).unwrap() + "\n").unwrap();
}

fn raw_file_hash(path: &Path) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in fs::read(path).unwrap() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn main() {
    let params = AllocationParams::default();
    assert!((params.mutation_probability - 0.01).abs() <= 1e-12);
    assert!((params.mutation_sigma - 0.15).abs() <= 1e-12);
    assert_eq!(DT, chemistry_core::mesh_mechanics::MechParams::default().dt);

    let candidates = [
        CandidateClass::ProcessingHeavy,
        CandidateClass::RepairHeavy,
        CandidateClass::Neutral,
    ];
    let environments = [
        Gate7Environment::H,
        Gate7Environment::B,
        Gate7Environment::Neutral,
    ];
    let mut endpoints = Vec::with_capacity(3 * 3 * 16);
    let mut protocols = Vec::new();
    for environment in environments {
        protocols.push(protocol_for(environment, &params));
        for candidate in candidates {
            for seed in 1_u64..=16 {
                endpoints.push(
                    run_replicate(candidate, environment, (seed - 1) as u32, seed, params)
                        .unwrap_or_else(|error| {
                            panic!("{candidate:?}/{environment:?}/{seed}: {error}")
                        }),
                );
            }
        }
    }
    assert_eq!(endpoints.len(), 144);
    assert!(endpoints
        .iter()
        .all(|endpoint| endpoint.mutation_protocol == "mutation_none"));
    assert!(endpoints
        .iter()
        .all(|endpoint| endpoint.maximum_generation_observed <= 1));
    assert!(endpoints
        .iter()
        .all(|endpoint| endpoint.every_birth_real_physical_fission));
    assert!(endpoints
        .iter()
        .all(|endpoint| !endpoint.treatment_label_leakage));
    assert!(endpoints
        .iter()
        .all(|endpoint| !endpoint.fitness_controller_leakage));
    assert!(endpoints.iter().all(|endpoint| endpoint.event_ledger_valid));

    let environment_hashes = environments
        .iter()
        .map(|environment| {
            let protocol = protocol_for(*environment, &params);
            (
                environment.id().to_string(),
                protocol.environment_protocol.hash(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let summaries = environments
        .iter()
        .flat_map(|environment| {
            candidates
                .iter()
                .map(|candidate| {
                    summarize(
                        *candidate,
                        *environment,
                        &endpoints,
                        &environment_hashes[environment.id()],
                        &params,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let h = paired_contrast(
        "D_H",
        Gate7Environment::H,
        CandidateClass::ProcessingHeavy,
        CandidateClass::RepairHeavy,
        &endpoints,
        true,
    );
    let b = paired_contrast(
        "D_B",
        Gate7Environment::B,
        CandidateClass::RepairHeavy,
        CandidateClass::ProcessingHeavy,
        &endpoints,
        true,
    );
    let neutral = paired_contrast(
        "D_Neutral",
        Gate7Environment::Neutral,
        CandidateClass::ProcessingHeavy,
        CandidateClass::RepairHeavy,
        &endpoints,
        false,
    );
    let neutral_abs_mean_smaller_than_h = neutral.mean_difference.abs() < h.mean_difference.abs();
    let neutral_abs_mean_smaller_than_b = neutral.mean_difference.abs() < b.mean_difference.abs();
    let primary_pass_h =
        h.expected_direction_at_least_12_of_16 && h.ci_excludes_zero_in_expected_direction;
    let primary_pass_b =
        b.expected_direction_at_least_12_of_16 && b.ci_excludes_zero_in_expected_direction;
    let primary_pass = primary_pass_h
        && primary_pass_b
        && neutral_abs_mean_smaller_than_h
        && neutral_abs_mean_smaller_than_b;
    let any_fission = endpoints
        .iter()
        .any(|endpoint| endpoint.first_fission_by_horizon);
    let interpretation = if primary_pass {
        "SR004C_D096_GATE7_SINGLE_GENERATION_FITNESS_QUALIFIED"
    } else if !any_fission {
        "SR004C_D096_GATE7_UNTESTABLE_IN_FROZEN_HORIZON"
    } else if !primary_pass_h || !primary_pass_b {
        "SR004C_D096_GATE7_RECIPROCITY_NOT_ESTABLISHED"
    } else {
        "SR004C_D096_GATE7_REPRODUCTIVE_CONSEQUENCE_NOT_ESTABLISHED"
    };
    let analysis = PairedAnalysis {
        h,
        b,
        neutral,
        neutral_abs_mean_smaller_than_h,
        neutral_abs_mean_smaller_than_b,
        primary_pass_h,
        primary_pass_b,
        primary_pass,
        interpretation: interpretation.into(),
    };

    let output_root = PathBuf::from("experiments/generated/sr004c");
    fs::create_dir_all(&output_root).unwrap();
    let candidate_records = candidates
        .iter()
        .map(|candidate| CandidateRecord {
            candidate: *candidate,
            candidate_id: candidate.id().into(),
            genotype: candidate.genotype(),
            candidate_hash: candidate.genotype().candidate_hash(&params),
        })
        .collect::<Vec<_>>();
    let environment_records = environments
        .iter()
        .map(|environment| EnvironmentRecord {
            environment: *environment,
            environment_id: environment.id().into(),
            environment_hash: environment_hashes[environment.id()].clone(),
            forcing: match environment {
                Gate7Environment::H => {
                    "fuel=1.0;nutrient=2.75 when step%400<100 else 0.264;damage=none".into()
                }
                Gate7Environment::B => {
                    "nutrient=1.98;fuel=1.0;damage=0.08 structural + 0.048 membrane every 350 steps"
                        .into()
                }
                Gate7Environment::Neutral => "nutrient=1.54;fuel=1.0;damage=none".into(),
            },
        })
        .collect::<Vec<_>>();
    let bundle = ProtocolBundle {
        schema: "D096Gate7ProtocolBundleV1",
        entry_commit: ENTRY_COMMIT,
        mutation_protocol: mutation_none(),
        candidates: candidate_records,
        environments: environment_records,
        protocols,
    };
    write_json(&output_root.join("protocol.json"), &bundle);
    write_json(
        &output_root.join("horizon_provenance.json"),
        &serde_json::json!({
            "schema": "D096Gate7HorizonProvenanceV1",
            "authority": "D-088 reproduction-qualified non-smoke physical-fission campaign",
            "source": "digital-protocell/crates/chemistry-core/src/d088_analysis.rs:steps(4000)",
            "accepted_steps": HORIZON_STEPS,
            "dt": DT,
            "maximum_accepted_simulated_time": HORIZON_TIME,
            "frozen_before_results": true
        }),
    );
    write_json(
        &output_root.join("replicate_preregistration.json"),
        &serde_json::json!({
            "schema": "D096Gate7ReplicatePreregistrationV1",
            "replicates": 16,
            "paired_seeds": (1..=16).collect::<Vec<_>>(),
            "same_seeds_across_candidates": true,
            "same_seeds_across_environments": true,
            "primary_contrasts_preregistered": ["D_H<0", "D_B<0", "abs(D_Neutral)<abs(D_H)", "abs(D_Neutral)<abs(D_B)"]
        }),
    );
    write_json(&output_root.join("replicate_endpoints.json"), &endpoints);
    write_json(
        &output_root.join("h_results.json"),
        &summaries
            .iter()
            .filter(|summary| summary.environment == Gate7Environment::H)
            .collect::<Vec<_>>(),
    );
    write_json(
        &output_root.join("b_results.json"),
        &summaries
            .iter()
            .filter(|summary| summary.environment == Gate7Environment::B)
            .collect::<Vec<_>>(),
    );
    write_json(
        &output_root.join("neutral_results.json"),
        &summaries
            .iter()
            .filter(|summary| summary.environment == Gate7Environment::Neutral)
            .collect::<Vec<_>>(),
    );
    write_json(&output_root.join("paired_analysis.json"), &analysis);
    write_json(
        &output_root.join("event_ledger_hashes.json"),
        &endpoints
            .iter()
            .map(|endpoint| {
                serde_json::json!({
                    "candidate": endpoint.candidate_id,
                    "environment": endpoint.environment_id,
                    "replicate": endpoint.replicate,
                    "seed": endpoint.seed,
                    "event_ledger_hash": endpoint.event_ledger_hash
                })
            })
            .collect::<Vec<_>>(),
    );

    let artifact_names = [
        "protocol.json",
        "horizon_provenance.json",
        "replicate_preregistration.json",
        "replicate_endpoints.json",
        "h_results.json",
        "b_results.json",
        "neutral_results.json",
        "paired_analysis.json",
        "event_ledger_hashes.json",
    ];
    let result_artifact_hashes = artifact_names
        .iter()
        .map(|name| ((*name).into(), raw_file_hash(&output_root.join(name))))
        .collect::<BTreeMap<_, _>>();
    let candidate_hashes = candidates
        .iter()
        .map(|candidate| {
            (
                candidate.id().into(),
                candidate.genotype().candidate_hash(&params),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let manifest = FinalManifest {
        schema: "D096Gate7FinalManifestV1",
        entry_commit: ENTRY_COMMIT,
        ending_commit: "recorded_at_artifact_generation;final_git_head_in_completion_report".into(),
        gate6_authority: "SR004B_D096_GATE6_HEREDITY_MUTATION_QUALIFIED",
        mutation_protocol: "mutation_none".into(),
        horizon_steps: HORIZON_STEPS,
        dt: DT,
        maximum_accepted_simulated_time: HORIZON_TIME,
        paired_seeds: (1..=16).collect(),
        candidate_hashes,
        environment_hashes,
        result_artifact_hashes,
        maximum_generation_observed: endpoints
            .iter()
            .map(|endpoint| endpoint.maximum_generation_observed)
            .max()
            .unwrap_or(0),
        gate8_executed: false,
        d094_executed: false,
        selection_operator_executed: false,
    };
    write_json(&output_root.join("final_manifest.json"), &manifest);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "D096Gate7AssayResultV1",
            "entry_commit": ENTRY_COMMIT,
            "horizon_steps": HORIZON_STEPS,
            "dt": DT,
            "maximum_accepted_simulated_time": HORIZON_TIME,
            "endpoints": endpoints.len(),
            "analysis": analysis,
            "artifacts": output_root,
        }))
        .unwrap()
    );
}
