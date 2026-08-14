//! DC-SR-004C-R2 parity-correct Gate 7 executor preflight.
//!
//! This example is deliberately non-reproductive. It runs the corrected
//! adapter for 1,000-step physiology-only preflights with fission disabled,
//! then serializes the frozen protocol that a future Gate 7 campaign would
//! use without executing it.

use chemistry_core::candidate_identity::sha256_hex;
use chemistry_core::d096_allocation::{
    pre_fission_assay, seed_d096_prefission_founder, AllocationGenotype, AllocationParams,
    AssayEnvironment, D096_PREFISSION_DT,
};
use chemistry_core::mesh_fission::FissionParams;
use chemistry_core::mesh_mechanics::MechParams;
use evolution_harness::{
    AdvanceOutcome, DigitalCellMeshAdapter, EnvironmentContext, EventType, ExperimentProtocolV1,
    FounderIdentityV1, FounderInitializationContext, Metadata, MutationProtocolV1, OrganismAdapter,
    ResourceMode,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY_COMMIT: &str = "aa98e40a75f662b53f5f05b8f4ae7dd0d495941d";
const PREFISSION_STEPS: usize = 1_000;
const DT: f64 = 0.02;
const PREFISSION_TIME: f64 = 20.0;
const HORIZON_STEPS: u64 = 4_000;
const HORIZON_TIME: f64 = 80.0;
const TOLERANCE_SCALE: f64 = 1e-9;

#[derive(Debug, Clone, Copy, Serialize)]
struct CellOutcome {
    reserve_change: f64,
    structural_change: f64,
    activated_produced: f64,
    damage_applied: f64,
    final_material: f64,
    survived: bool,
    accepted_steps: usize,
    accepted_simulated_time: f64,
    fission_observed: bool,
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    id: &'static str,
    genotype: AllocationGenotype,
}

fn candidates() -> [Candidate; 3] {
    [
        Candidate {
            id: "processing-heavy",
            genotype: AllocationGenotype([0.55, 0.25, 0.05, 0.15]),
        },
        Candidate {
            id: "repair-heavy",
            genotype: AllocationGenotype([0.10, 0.20, 0.55, 0.15]),
        },
        Candidate {
            id: "neutral",
            genotype: AllocationGenotype::neutral(),
        },
    ]
}

fn environments() -> [(&'static str, AssayEnvironment); 3] {
    [
        ("H", AssayEnvironment::H),
        ("B", AssayEnvironment::B),
        ("Neutral", AssayEnvironment::Neutral),
    ]
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE_SCALE * (1.0 + a.abs())
}

fn outcome_fields(a: CellOutcome, b: CellOutcome) -> [bool; 5] {
    [
        close(a.reserve_change, b.reserve_change),
        close(a.structural_change, b.structural_change),
        close(a.activated_produced, b.activated_produced),
        close(a.damage_applied, b.damage_applied),
        close(a.final_material, b.final_material),
    ]
}

fn metadata_f64(metadata: &Metadata, key: &str) -> f64 {
    metadata
        .get(key)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn damage_from_events(events: &[evolution_harness::AdapterEnvironmentEvent]) -> f64 {
    events
        .iter()
        .filter(|event| event.event_type == EventType::DamageApplied)
        .map(|event| {
            metadata_f64(&event.metadata, "structural") + metadata_f64(&event.metadata, "membrane")
        })
        .sum()
}

fn protocol_for(environment: AssayEnvironment, seeds: &[u64]) -> ExperimentProtocolV1 {
    let id = match environment {
        AssayEnvironment::H => "H",
        AssayEnvironment::B => "B",
        AssayEnvironment::Neutral => "Neutral",
    };
    let mut protocol = ExperimentProtocolV1::minimal(
        &format!("d096_gate7_r2_preflight_{}", id.to_lowercase()),
        id,
        "mutation_none",
    );
    protocol.organism_schema =
        chemistry_core::d096_allocation::EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION.into();
    protocol.heredity_schema = "D096AllocationGenotypeV1".into();
    protocol.environment_protocol.resource_mode = match environment {
        AssayEnvironment::H => ResourceMode::Pulsed,
        AssayEnvironment::B | AssayEnvironment::Neutral => ResourceMode::Continuous,
    };
    protocol.environment_protocol.resource_field = "D096_exact_assay_environment".into();
    protocol.environment_protocol.duration = PREFISSION_TIME;
    protocol.replicates = seeds.len() as u32;
    protocol.random_seeds = seeds.to_vec();
    protocol.maximum_accepted_horizon = PREFISSION_TIME;
    protocol.maximum_generation = 1;
    protocol.minimum_generation_requirement = 0;
    protocol.termination_rules = vec!["accepted_horizon".into(), "founder_death".into()];
    protocol.provenance.source_artifacts = BTreeMap::from([
        (
            "gate5_authority".into(),
            "chemistry-core::d096_allocation::pre_fission_assay".into(),
        ),
        (
            "shared_constructor".into(),
            "chemistry-core::d096_allocation::seed_d096_prefission_founder".into(),
        ),
        (
            "forcing".into(),
            "chemistry-core::d096_allocation::apply_assay_environment".into(),
        ),
    ]);
    protocol.provenance.derived_values = BTreeMap::from([
        ("dt".into(), DT.to_string()),
        ("steps".into(), PREFISSION_STEPS.to_string()),
        ("fission".into(), "disabled".into()),
        ("mutation".into(), "mutation_none".into()),
    ]);
    protocol
}

fn founder(candidate: Candidate, seed: u64, params: &AllocationParams) -> FounderIdentityV1 {
    FounderIdentityV1::new(
        1,
        chemistry_core::d096_allocation::EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION,
        &candidate.genotype.candidate_hash(params),
        "D096_frozen_candidate",
        &format!("D096_material_seed_{seed}"),
        seed,
        "none;DC-SR-004C-R2-preflight",
    )
}

fn run_adapter(
    candidate: Candidate,
    environment: AssayEnvironment,
    seed: u64,
    enable_mechanics: bool,
) -> Result<CellOutcome, String> {
    let params = AllocationParams::default();
    let protocol = protocol_for(environment, &[seed]);
    let mut adapter = DigitalCellMeshAdapter {
        enable_mechanics,
        enable_fission: false,
        allocation_params: Some(params),
        d096_founder_genotype: Some(candidate.genotype),
        fission: FissionParams::default(),
        mech: MechParams::default(),
        ..DigitalCellMeshAdapter::default()
    }
    .with_d096_assay_environment(environment);
    let founder = founder(candidate, seed, &params);
    let mut organism = adapter
        .initialize_founder(
            &founder,
            FounderInitializationContext {
                replicate: 0,
                founder_index: 0,
                population_size: 1,
                placement: [0.0, 0.0],
            },
        )
        .map_err(|error| error.to_string())?;
    let initial_reserve = organism.interior.r * organism.area();
    let initial_material = organism.total_structural_mass();
    let mut activated_produced = 0.0;
    let mut damage_applied = 0.0;
    let mut accepted_steps = 0;
    let mut fission_observed = false;

    for step in 0..PREFISSION_STEPS {
        if !organism.alive {
            break;
        }
        let events = adapter
            .apply_declared_environment(
                &mut organism,
                &protocol.environment_protocol,
                (step + 1) as u64,
                step as f64 * DT,
                EnvironmentContext {
                    living_population: 1,
                    organism_index: 0,
                    accepted_dt: DT,
                },
            )
            .map_err(|error| error.to_string())?;
        damage_applied += damage_from_events(&events);
        let outcome = adapter
            .advance(
                &mut organism,
                &protocol.environment_protocol,
                (step + 1) as u64,
                step as f64 * DT,
            )
            .map_err(|error| error.to_string())?;
        accepted_steps += 1;
        match outcome {
            AdvanceOutcome::Continuing { metadata, .. } => {
                activated_produced += metadata_f64(&metadata, "activated_produced");
            }
            AdvanceOutcome::Died { metadata, .. } => {
                activated_produced += metadata_f64(&metadata, "activated_produced");
                break;
            }
            AdvanceOutcome::Fission { metadata, .. } => {
                activated_produced += metadata_f64(&metadata, "activated_produced");
                fission_observed = true;
                break;
            }
        }
    }

    Ok(CellOutcome {
        reserve_change: organism.interior.r * organism.area() - initial_reserve,
        structural_change: organism.total_structural_mass() - initial_material,
        activated_produced,
        damage_applied,
        final_material: organism.total_structural_mass() + organism.total_bound_membrane(),
        survived: organism.alive,
        accepted_steps,
        accepted_simulated_time: accepted_steps as f64 * DT,
        fission_observed,
    })
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn survived_count(rows: &[Value], candidate: &str, environment: &str) -> usize {
    rows.iter()
        .filter(|row| row["candidate"] == candidate && row["environment"] == environment)
        .filter(|row| row["outcome"]["survived"].as_bool() == Some(true))
        .count()
}

fn candidate_hashes(params: &AllocationParams) -> BTreeMap<String, String> {
    candidates()
        .into_iter()
        .map(|candidate| {
            (
                candidate.id.to_string(),
                candidate.genotype.candidate_hash(params),
            )
        })
        .collect()
}

fn future_protocol_bundle(params: &AllocationParams) -> Value {
    let seeds: Vec<u64> = (1..=16).collect();
    let mut protocol_rows = Vec::new();
    let mut environment_hashes = BTreeMap::new();
    for (id, environment) in environments() {
        let protocol = protocol_for(environment, &seeds);
        environment_hashes.insert(id.to_string(), protocol.environment_protocol.hash());
        protocol_rows.push(json!({
            "environment": id,
            "protocol": protocol,
            "protocol_hash": protocol.hash(),
        }));
    }
    json!({
        "schema": "DC-SR-004C-R2-FutureGate7ProtocolFreezeV1",
        "executed": false,
        "campaign_executed": false,
        "candidate_genotypes": candidates().into_iter().map(|candidate| json!({
            "id": candidate.id,
            "genotype": candidate.genotype.0,
            "hash": candidate.genotype.candidate_hash(params),
        })).collect::<Vec<_>>(),
        "candidate_hashes": candidate_hashes(params),
        "environment_hashes": environment_hashes,
        "paired_seeds": seeds,
        "mutation": MutationProtocolV1::default(),
        "maximum_accepted_steps": HORIZON_STEPS,
        "dt": DT,
        "maximum_simulated_time": HORIZON_TIME,
        "one_generation_stop": ["founder_first_physical_fission", "founder_causal_death", "frozen_horizon"],
        "maximum_generation": 1,
        "gen2": false,
        "statistical_interpretation": {
            "paired_expected_direction_at_least": 12,
            "paired_bootstrap": "95% CI excluding zero in expected direction",
            "H": "processing-heavy expected faster reproduction than repair-heavy",
            "B": "repair-heavy expected faster reproduction than processing-heavy",
            "neutral": "absolute processing-vs-repair mean separation smaller than both selecting-environment effects"
        },
        "protocols": protocol_rows
    })
}

fn write_json(root: &Path, name: &str, value: &impl Serialize) {
    fs::write(
        root.join(name),
        serde_json::to_string_pretty(value).expect("serialize R2 artifact"),
    )
    .expect("write R2 artifact");
}

fn main() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let artifact_root = repo_root.join("experiments/generated/sr004cr2");
    fs::create_dir_all(&artifact_root).expect("create R2 artifact directory");
    let params = AllocationParams::default();
    let seeds: Vec<u64> = (1..=8).collect();

    let authority_mesh = seed_d096_prefission_founder(AllocationGenotype::neutral(), 1);
    let authority_reaction =
        chemistry_core::d096_allocation::d096_prefission_reaction_params(&authority_mesh);
    write_json(
        &artifact_root,
        "authority_profile.json",
        &json!({
            "schema": "DC-SR-004C-R2-Gate5AuthorityProfileV1",
            "source": "chemistry-core::d096_allocation::pre_fission_assay",
            "entry_commit": ENTRY_COMMIT,
            "founder": {
                "seed": 1,
                "vertex_count": "12 + seed % 3",
                "vertices_seed_1": authority_mesh.vertices.clone(),
                "radius": 8.0,
                "center": [0.0, 0.0],
                "rho_s": authority_mesh.rho_s,
                "theta_b": 0.8,
                "free_l": authority_mesh.free_l,
                "interior": authority_mesh.interior,
                "exterior": authority_mesh.exterior,
                "b_max_per_length": authority_mesh.b_max_per_length,
                "bond_threshold": authority_mesh.bond_threshold,
                "l_max": authority_mesh.l_max,
                "l_min": authority_mesh.l_min
            },
            "equation": chemistry_core::d096_allocation::EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION,
            "allocation_params": params,
            "reaction_params": authority_reaction,
            "transport_params": chemistry_core::d096_allocation::d096_prefission_transport_params(),
            "growth_params": chemistry_core::d096_allocation::d096_prefission_growth_params(),
            "reserve_enabled": authority_reaction.reserve.enable,
            "dt": D096_PREFISSION_DT,
            "steps": PREFISSION_STEPS,
            "expression_order": ["apply_assay_environment", "expression_step", "transport_step", "reactions_step", "growth_step"],
            "fission": false,
            "mechanics": false,
            "topology": false,
            "mutation": "mutation_none",
            "forcing": "chemistry-core::d096_allocation::apply_assay_environment"
        }),
    );
    write_json(
        &artifact_root,
        "execution_history_correction.json",
        &json!({
            "schema": "DC-SR-004C-R2-ExecutionHistoryCorrectionV1",
            "r1_audit_runner_invoked_gate7": false,
            "automatic_gate7_ci_rerun_observed": true,
            "automatic_original_gate7_workflow_runs": [
                {"run_id": "31845151456", "head": "b258126fb2ac1373515a09711d7dcaa07022550", "trigger": "push"},
                {"run_id": "31845154403", "head": "b258126fb2ac1373515a09711d7dcaa07022550", "trigger": "pull_request"},
                {"run_id": "31845606065", "head": ENTRY_COMMIT, "trigger": "pull_request"}
            ],
            "interpretation": "R1 gate7_rerun:false means only that the R1 shadow audit runner did not invoke Gate 7."
        }),
    );
    write_json(
        &artifact_root,
        "shared_constructor_audit.json",
        &json!({
            "schema": "DC-SR-004C-R2-SharedConstructorAuditV1",
            "authoritative_gate5": ["chemistry-core::d096_allocation::pre_fission_assay", "chemistry-core::d096_allocation::seed_d096_prefission_founder", "chemistry-core::d096_allocation::d096_prefission_reaction_params", "chemistry-core::d096_allocation::d096_prefission_transport_params", "chemistry-core::d096_allocation::d096_prefission_growth_params"],
            "corrected_gate7": ["evolution-harness::DigitalCellMeshAdapter::initialize_founder", "chemistry-core::d096_allocation::seed_d096_prefission_founder", "chemistry-core::d096_allocation::d096_prefission_reaction_params", "chemistry-core::d096_allocation::d096_prefission_transport_params", "chemistry-core::d096_allocation::d096_prefission_growth_params"],
            "behavior_preserving_refactor": true,
            "duplicate_handwritten_gate5_state": false
        }),
    );

    let mut exact_rows = Vec::new();
    let mut exact_pass_count = 0usize;
    let mut exact_max_residual: f64 = 0.0;
    let mut exact_h = BTreeMap::new();
    let mut exact_b = BTreeMap::new();
    let mut exact_neutral = BTreeMap::new();
    let mut mechanics_rows = Vec::new();
    for candidate in candidates() {
        for (environment_id, environment) in environments() {
            for seed in &seeds {
                let authority =
                    pre_fission_assay(candidate.genotype, environment, *seed, PREFISSION_STEPS);
                let adapter = run_adapter(candidate, environment, *seed, false)
                    .expect("exact adapter preflight");
                let adapter_mechanics = run_adapter(candidate, environment, *seed, true)
                    .expect("mechanics adapter preflight");
                let authority_json = serde_json::to_value(authority).unwrap();
                let adapter_json = serde_json::to_value(adapter).unwrap();
                let residuals = [
                    (authority.reserve_change - adapter.reserve_change).abs(),
                    (authority.structural_change - adapter.structural_change).abs(),
                    (authority.activated_produced - adapter.activated_produced).abs(),
                    (authority.damage_applied - adapter.damage_applied).abs(),
                    (authority.final_material - adapter.final_material).abs(),
                ];
                let row_max = residuals.into_iter().fold(0.0, f64::max);
                exact_max_residual = exact_max_residual.max(row_max);
                let fields = outcome_fields(
                    CellOutcome {
                        reserve_change: authority.reserve_change,
                        structural_change: authority.structural_change,
                        activated_produced: authority.activated_produced,
                        damage_applied: authority.damage_applied,
                        final_material: authority.final_material,
                        survived: authority.survived,
                        accepted_steps: PREFISSION_STEPS,
                        accepted_simulated_time: PREFISSION_STEPS as f64 * DT,
                        fission_observed: false,
                    },
                    adapter,
                );
                let pass = fields.into_iter().all(|value| value)
                    && authority.survived == adapter.survived
                    && !adapter.fission_observed;
                if pass {
                    exact_pass_count += 1;
                }
                exact_rows.push(json!({
                    "candidate": candidate.id,
                    "environment": environment_id,
                    "seed": seed,
                    "authority": authority_json,
                    "adapter": adapter_json,
                    "pass": pass,
                    "maximum_absolute_residual": row_max
                }));
                let selected = match environment {
                    AssayEnvironment::H => &mut exact_h,
                    AssayEnvironment::B => &mut exact_b,
                    AssayEnvironment::Neutral => &mut exact_neutral,
                };
                selected.insert(
                    format!("{}:{}", candidate.id, seed),
                    json!({"authority": authority, "adapter": adapter}),
                );
                mechanics_rows.push(json!({
                    "candidate": candidate.id,
                    "environment": environment_id,
                    "seed": seed,
                    "outcome": adapter_mechanics
                }));
            }
        }
    }

    let h_processing: Vec<f64> = exact_h
        .values()
        .filter_map(|row| row["adapter"]["reserve_change"].as_f64())
        .collect();
    let _ = h_processing;
    let exact_summary = json!({
        "schema": "DC-SR-004C-R2-ExactParitySummaryV1",
        "cells": exact_rows.len(),
        "passed": exact_pass_count,
        "failed": exact_rows.len() - exact_pass_count,
        "maximum_absolute_residual": exact_max_residual,
        "tolerance": "abs(A-B) <= 1e-9 * (1 + abs(A))",
        "all_boolean_endpoints_match": exact_pass_count == exact_rows.len(),
        "gate5_sealed_h_effect": 0.5988859008884848,
        "gate5_sealed_b_effect": 3.811469763347633,
        "gate5_replay": "pre_fission_assay",
        "adapter_replay": "DigitalCellMeshAdapter with mechanics=false, fission=false, topology=false",
        "h_parity": exact_pass_count == exact_rows.len(),
        "b_parity": exact_pass_count == exact_rows.len(),
        "neutral_parity": exact_pass_count == exact_rows.len()
    });
    write_json(
        &artifact_root,
        "exact_parity_results.json",
        &json!({"schema": "DC-SR-004C-R2-ExactParityResultsV1", "rows": exact_rows}),
    );
    write_json(&artifact_root, "exact_parity_summary.json", &exact_summary);

    let mechanics_value_rows: Vec<Value> = mechanics_rows;
    let value = |candidate: &str, environment: &str, field: &str| -> Vec<f64> {
        mechanics_value_rows
            .iter()
            .filter(|row| row["candidate"] == candidate && row["environment"] == environment)
            .filter_map(|row| row["outcome"][field].as_f64())
            .collect()
    };
    let h_processing = value("processing-heavy", "H", "reserve_change");
    let h_repair = value("repair-heavy", "H", "reserve_change");
    let h_neutral_processing = value("processing-heavy", "Neutral", "reserve_change");
    let h_neutral_repair = value("repair-heavy", "Neutral", "reserve_change");
    let b_processing = value("processing-heavy", "B", "final_material");
    let b_repair = value("repair-heavy", "B", "final_material");
    let b_neutral_processing = value("processing-heavy", "Neutral", "final_material");
    let b_neutral_repair = value("repair-heavy", "Neutral", "final_material");
    let h_diffs: Vec<f64> = h_processing
        .iter()
        .zip(&h_repair)
        .map(|(a, b)| a - b)
        .collect();
    let b_diffs: Vec<f64> = b_repair
        .iter()
        .zip(&b_processing)
        .map(|(a, b)| a - b)
        .collect();
    let h_neutral_diffs: Vec<f64> = h_neutral_processing
        .iter()
        .zip(&h_neutral_repair)
        .map(|(a, b)| a - b)
        .collect();
    let b_neutral_diffs: Vec<f64> = b_neutral_repair
        .iter()
        .zip(&b_neutral_processing)
        .map(|(a, b)| a - b)
        .collect();
    let h_effect = mean(&h_diffs);
    let b_effect = mean(&b_diffs);
    let h_neutral_effect = mean(&h_neutral_diffs);
    let b_neutral_effect = mean(&b_neutral_diffs);
    let h_positive = h_diffs.iter().all(|value| *value > 0.0);
    let b_positive = b_diffs.iter().all(|value| *value > 0.0);
    let mechanics_summary = json!({
        "schema": "DC-SR-004C-R2-MechanicsExtensionSummaryV1",
        "fission_enabled": false,
        "mechanics_enabled": true,
        "topology_enabled": true,
        "h": {"per_seed_positive": h_positive, "effect": h_effect, "neutral_effect": h_neutral_effect, "exceeds_neutral": h_effect > h_neutral_effect},
        "b": {"per_seed_positive": b_positive, "effect": b_effect, "neutral_effect": b_neutral_effect, "exceeds_neutral": b_effect > b_neutral_effect},
        "neutral": {"h_processing_vs_repair": h_neutral_effect, "b_repair_vs_processing": b_neutral_effect},
        "survival": {"processing_heavy": survived_count(&mechanics_value_rows, "processing-heavy", "H"), "repair_heavy": survived_count(&mechanics_value_rows, "repair-heavy", "H")},
        "pass": h_positive && b_positive && h_effect > h_neutral_effect && b_effect > b_neutral_effect
    });
    write_json(
        &artifact_root,
        "mechanics_extension_results.json",
        &json!({"schema": "DC-SR-004C-R2-MechanicsExtensionResultsV1", "rows": mechanics_value_rows}),
    );
    write_json(
        &artifact_root,
        "mechanics_extension_summary.json",
        &mechanics_summary,
    );

    let future = future_protocol_bundle(&params);
    let future_protocol_hash = sha256_hex(serde_json::to_string(&future).unwrap().as_bytes());
    write_json(
        &artifact_root,
        "future_gate7_protocol_freeze.json",
        &json!({"bundle": future, "bundle_sha256": future_protocol_hash}),
    );

    let exact_pass = exact_pass_count == exact_rows.len();
    let mechanics_pass = mechanics_summary["pass"].as_bool().unwrap_or(false);
    let conclusion = if exact_pass && mechanics_pass {
        "SR004CR2_PARITY_CORRECT_EXECUTOR_QUALIFIED"
    } else if !exact_pass && !mechanics_pass {
        "SR004CR2_BOTH_PREFLIGHTS_FAILED"
    } else if !exact_pass {
        "SR004CR2_EXACT_PARITY_FAILED"
    } else {
        "SR004CR2_MECHANICS_EXTENSION_ERASES_RECIPROCITY"
    };
    write_json(
        &artifact_root,
        "final_manifest.json",
        &json!({
            "schema": "DC-SR-004C-R2-FinalManifestV1",
            "directive": "DC-SR-004C-R2",
            "starting_commit": ENTRY_COMMIT,
            "gate7_campaign_executed": false,
            "gate8_started": false,
            "d094_executed": false,
            "certified_phase1_biology_modified": false,
            "exact_parity_cells": exact_rows.len(),
            "exact_parity_passed": exact_pass_count,
            "exact_parity_maximum_absolute_residual": exact_max_residual,
            "mechanics_extension_pass": mechanics_pass,
            "future_protocol_executed": false,
            "future_protocol_hash": future_protocol_hash,
            "horizon_steps": HORIZON_STEPS,
            "dt": DT,
            "horizon_time": HORIZON_TIME,
            "conclusion": conclusion
        }),
    );
    println!(
        "DC-SR-004C-R2 preflight artifacts written to {}",
        artifact_root.display()
    );
    println!("conclusion={conclusion}");
}
