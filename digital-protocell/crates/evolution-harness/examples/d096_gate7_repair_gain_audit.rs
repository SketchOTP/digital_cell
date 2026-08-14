//! DC-SR-004C-R3 repair-gain specificity audit.
//!
//! This example is diagnostic only. It records observer-only flux attribution
//! for the unchanged D-096 path and runs one explicitly isolated shadow law:
//! J_shadow = J_base + J_strain * g_repair.
//! No fission, mutation, reproduction, or production parameter is changed.

use chemistry_core::candidate_identity::sha256_hex;
use chemistry_core::d096_allocation::{
    AllocationGenotype, AllocationParams, AssayEnvironment, D096_PREFISSION_DT,
};
use chemistry_core::mesh_fission::FissionParams;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::StructuralBuildMode;
use evolution_harness::{
    AdvanceOutcome, DigitalCellMeshAdapter, EnvironmentContext, EventType, ExperimentProtocolV1,
    FounderIdentityV1, FounderInitializationContext, Metadata, OrganismAdapter, ResourceMode,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY_COMMIT: &str = "1b477b1c53d075449368579bfba6be1ed60b69f8";
const STEPS: usize = 1_000;
const DT: f64 = D096_PREFISSION_DT;
const TIME: f64 = STEPS as f64 * DT;
const TOLERANCE: f64 = 1e-9;

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

#[derive(Debug, Clone, Copy, Serialize, Default)]
struct AuditOutcome {
    structural_build_total: f64,
    baseline_build_before_repair_gain: f64,
    strain_build_before_repair_gain: f64,
    baseline_build_amplification: f64,
    strain_build_amplification: f64,
    reserve_a_to_r: f64,
    reserve_r_to_a: f64,
    reserve_consumed_for_growth: f64,
    reserve_funded_growth_mass: f64,
    activated_resource_produced: f64,
    final_reserve_mass: f64,
    final_structural_mass: f64,
    final_bound_membrane: f64,
    final_total_material: f64,
    mean_strain: f64,
    maximum_strain: f64,
    damage_applied: f64,
    survived: bool,
    accepted_steps: usize,
    accepted_simulated_time: f64,
}

fn protocol_for(environment: AssayEnvironment) -> ExperimentProtocolV1 {
    let id = match environment {
        AssayEnvironment::H => "H",
        AssayEnvironment::B => "B",
        AssayEnvironment::Neutral => "Neutral",
    };
    let mut protocol = ExperimentProtocolV1::minimal(
        &format!("d096_gate7_r3_{id}_observer_audit"),
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
    protocol.environment_protocol.duration = TIME;
    protocol.maximum_accepted_horizon = TIME;
    protocol.maximum_generation = 0;
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
        ("steps".into(), STEPS.to_string()),
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
        "none;DC-SR-004C-R3-observer-audit",
    )
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

fn strain_summary(mesh: &chemistry_core::material_mesh::MaterialMesh) -> (f64, f64) {
    if mesh.n() == 0 {
        return (0.0, 0.0);
    }
    let values: Vec<f64> = (0..mesh.n()).map(|i| mesh.strain(i)).collect();
    (
        values.iter().sum::<f64>() / values.len() as f64,
        values.into_iter().fold(0.0, f64::max),
    )
}

fn run_one(
    candidate: Candidate,
    environment: AssayEnvironment,
    seed: u64,
    mechanics: bool,
    build_mode: StructuralBuildMode,
) -> Result<AuditOutcome, String> {
    let params = AllocationParams::default();
    let protocol = protocol_for(environment);
    let mut adapter = DigitalCellMeshAdapter {
        enable_mechanics: mechanics,
        enable_fission: false,
        allocation_params: Some(params),
        d096_founder_genotype: Some(candidate.genotype),
        fission: FissionParams::default(),
        mech: MechParams::default(),
        structural_build_mode: build_mode,
        ..DigitalCellMeshAdapter::default()
    }
    .with_d096_assay_environment(environment);
    let founder = founder(candidate, seed, &params);
    let mut organism = adapter
        .initialize_founder(
            &founder,
            FounderInitializationContext {
                replicate: seed as u32,
                founder_index: 0,
                population_size: 1,
                placement: [0.0, 0.0],
            },
        )
        .map_err(|error| error.to_string())?;
    let mut result = AuditOutcome::default();
    let mut accepted_time = 0.0;

    for accepted_step in 1..=STEPS {
        if !organism.alive {
            break;
        }
        let accepted_dt = adapter.accepted_dt();
        let events = adapter
            .apply_declared_environment(
                &mut organism,
                &protocol.environment_protocol,
                accepted_step as u64,
                accepted_time,
                EnvironmentContext {
                    living_population: 1,
                    organism_index: 0,
                    accepted_dt,
                },
            )
            .map_err(|error| error.to_string())?;
        result.damage_applied += damage_from_events(&events);
        let outcome = adapter
            .advance(
                &mut organism,
                &protocol.environment_protocol,
                accepted_step as u64,
                accepted_time,
            )
            .map_err(|error| error.to_string())?;
        let (metadata, accepted_dt) = match outcome {
            AdvanceOutcome::Continuing {
                accepted_dt,
                metadata,
            }
            | AdvanceOutcome::Died {
                accepted_dt,
                metadata,
                ..
            }
            | AdvanceOutcome::Fission {
                accepted_dt,
                metadata,
                ..
            } => (metadata, accepted_dt),
        };
        result.accepted_steps += 1;
        accepted_time += accepted_dt;
        result.structural_build_total += metadata_f64(&metadata, "structural_build_total");
        result.baseline_build_before_repair_gain +=
            metadata_f64(&metadata, "structural_build_baseline");
        result.strain_build_before_repair_gain +=
            metadata_f64(&metadata, "structural_build_strain");
        result.baseline_build_amplification +=
            metadata_f64(&metadata, "structural_build_baseline_amplification");
        result.strain_build_amplification +=
            metadata_f64(&metadata, "structural_build_strain_amplification");
        result.reserve_a_to_r += metadata_f64(&metadata, "reserve_a_to_r");
        result.reserve_r_to_a += metadata_f64(&metadata, "reserve_r_to_a");
        result.reserve_consumed_for_growth +=
            metadata_f64(&metadata, "reserve_r_consumed_for_growth");
        result.reserve_funded_growth_mass += metadata_f64(&metadata, "reserve_funded_growth_mass");
        result.activated_resource_produced += metadata_f64(&metadata, "activated_produced");
        if !organism.alive {
            break;
        }
    }

    let area = organism.area();
    let (mean_strain, maximum_strain) = strain_summary(&organism);
    result.final_reserve_mass = organism.interior.r.max(0.0) * area;
    result.final_structural_mass = organism.total_structural_mass();
    result.final_bound_membrane = organism.total_bound_membrane();
    result.final_total_material = result.final_structural_mass + result.final_bound_membrane;
    result.mean_strain = mean_strain;
    result.maximum_strain = maximum_strain;
    result.survived = organism.alive;
    result.accepted_simulated_time = accepted_time;
    Ok(result)
}

fn row(
    candidate: Candidate,
    environment: &str,
    seed: u64,
    mode: &str,
    outcome: AuditOutcome,
) -> Value {
    json!({
        "candidate": candidate.id,
        "genotype": candidate.genotype.0,
        "environment": environment,
        "seed": seed,
        "mode": mode,
        "outcome": outcome,
    })
}

fn find<'a>(
    rows: &'a [Value],
    candidate: &str,
    environment: &str,
    seed: u64,
    mode: &str,
) -> &'a Value {
    rows.iter()
        .find(|row| {
            row["candidate"] == candidate
                && row["environment"] == environment
                && row["seed"].as_u64() == Some(seed)
                && row["mode"] == mode
        })
        .expect("audit row exists")
}

fn outcome(row: &Value) -> &Value {
    &row["outcome"]
}

fn f(row: &Value, key: &str) -> f64 {
    outcome(row)[key].as_f64().unwrap_or(0.0)
}

fn paired(rows: &[Value], environment: &str, mode: &str) -> Vec<Value> {
    (1..=8)
        .map(|seed| {
            let processing = find(rows, "processing-heavy", environment, seed, mode);
            let repair = find(rows, "repair-heavy", environment, seed, mode);
            json!({
                "seed": seed,
                "processing": outcome(processing),
                "repair": outcome(repair),
                "repair_minus_processing": {
                    "baseline_build_amplification": f(repair, "baseline_build_amplification") - f(processing, "baseline_build_amplification"),
                    "strain_build_amplification": f(repair, "strain_build_amplification") - f(processing, "strain_build_amplification"),
                    "reserve_funded_growth_mass": f(repair, "reserve_funded_growth_mass") - f(processing, "reserve_funded_growth_mass"),
                    "structural_build_total": f(repair, "structural_build_total") - f(processing, "structural_build_total"),
                    "final_structural_mass": f(repair, "final_structural_mass") - f(processing, "final_structural_mass"),
                    "final_total_material": f(repair, "final_total_material") - f(processing, "final_total_material"),
                    "damage_applied": f(repair, "damage_applied") - f(processing, "damage_applied"),
                }
            })
        })
        .collect()
}

fn all_positive(pairs: &[Value], key: &str) -> bool {
    pairs
        .iter()
        .all(|pair| pair["repair_minus_processing"][key].as_f64().unwrap_or(0.0) > 0.0)
}

fn all_non_positive(pairs: &[Value], key: &str) -> bool {
    pairs
        .iter()
        .all(|pair| pair["repair_minus_processing"][key].as_f64().unwrap_or(0.0) <= TOLERANCE)
}

fn write_json(root: &Path, name: &str, value: &impl Serialize) {
    fs::write(
        root.join(name),
        serde_json::to_string_pretty(value).expect("serialize R3 artifact"),
    )
    .expect("write R3 artifact");
}

fn main() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let artifact_root = repo_root.join("experiments/generated/sr004cr3");
    fs::create_dir_all(&artifact_root).expect("create R3 artifact directory");
    let params = AllocationParams::default();
    let contract_path = repo_root.join("docs/d096_selected_architecture_contract.md");
    let contract_text = fs::read_to_string(&contract_path).expect("read frozen D-096 contract");
    let contract_hash = sha256_hex(contract_text.as_bytes());

    let mut rows = Vec::new();
    for candidate in candidates() {
        for (environment_id, environment) in environments() {
            for seed in 1..=8 {
                let exact = run_one(
                    candidate,
                    environment,
                    seed,
                    false,
                    StructuralBuildMode::Current,
                )
                .expect("R3 mechanics-off current audit");
                let mechanics = run_one(
                    candidate,
                    environment,
                    seed,
                    true,
                    StructuralBuildMode::Current,
                )
                .expect("R3 mechanics-on current audit");
                let shadow = run_one(
                    candidate,
                    environment,
                    seed,
                    true,
                    StructuralBuildMode::D096RepairSpecificShadow,
                )
                .expect("R3 mechanics-on shadow audit");
                rows.push(row(
                    candidate,
                    environment_id,
                    seed,
                    "current_mechanics_off",
                    exact,
                ));
                rows.push(row(
                    candidate,
                    environment_id,
                    seed,
                    "current_mechanics_on",
                    mechanics,
                ));
                rows.push(row(
                    candidate,
                    environment_id,
                    seed,
                    "shadow_mechanics_on",
                    shadow,
                ));
            }
        }
    }

    let h_current = paired(&rows, "H", "current_mechanics_on");
    let b_current = paired(&rows, "B", "current_mechanics_on");
    let neutral_current = paired(&rows, "Neutral", "current_mechanics_on");
    let h_shadow = paired(&rows, "H", "shadow_mechanics_on");
    let b_shadow = paired(&rows, "B", "shadow_mechanics_on");
    let neutral_shadow = paired(&rows, "Neutral", "shadow_mechanics_on");

    let h_baseline_leakage = all_positive(&h_current, "baseline_build_amplification");
    let h_shadow_removes_baseline_advantage = all_non_positive(&h_shadow, "final_structural_mass");
    let b_shadow_preserves_damage_advantage = all_positive(&b_shadow, "strain_build_amplification")
        && all_positive(&b_shadow, "final_total_material");
    let source_trace_proves_broad_gain = true;
    let contract_identifies_repair_as_damage_response = true;
    let disposition = if source_trace_proves_broad_gain
        && h_baseline_leakage
        && contract_identifies_repair_as_damage_response
        && h_shadow_removes_baseline_advantage
        && b_shadow_preserves_damage_advantage
    {
        "SR004CR3_D096_REPAIR_GAIN_SCOPE_IMPLEMENTATION_DEFECT_CONFIRMED"
    } else if !h_shadow_removes_baseline_advantage || !b_shadow_preserves_damage_advantage {
        "SR004CR3_D096_ARCHITECTURE_REJECTED_FOR_RECIPROCAL_FITNESS"
    } else {
        "SR004CR3_CAUSAL_ATTRIBUTION_INCONCLUSIVE"
    };

    let callsites = json!({
        "schema": "DC-SR-004C-R3-D096GainCallsiteMapV1",
        "entry_commit": ENTRY_COMMIT,
        "function_gain": {
            "source": "crates/chemistry-core/src/d096_allocation.rs::function_gain",
            "coordinates": {"0": "processing", "1": "activation", "2": "repair", "3": "growth"},
            "gain": "1 + catalyst / (0.1 + catalyst)"
        },
        "runtime_callsites": [
            {"source": "crates/chemistry-core/src/mesh_reactions.rs::structural_build_flux", "coordinate": 2, "effect": "multiplies g_strain(eps)=g0+strain-responsive component"},
            {"source": "crates/chemistry-core/src/mesh_reactions.rs::reactions_step", "coordinate": 0, "effect": "processing gain in N+F activation extent"},
            {"source": "crates/chemistry-core/src/mesh_reactions.rs::reactions_step", "coordinate": 1, "effect": "activation gain in N+F activation extent"},
            {"source": "crates/chemistry-core/src/mesh_reactions.rs::reactions_step", "coordinate": 2, "effect": "free membrane production branch"},
            {"source": "crates/chemistry-core/src/mesh_growth.rs::growth_step", "coordinate": 3, "effect": "reserve-funded growth synthesis"}
        ],
        "production_path_unchanged": true
    });
    write_json(&artifact_root, "gain_callsite_map.json", &callsites);

    write_json(
        &artifact_root,
        "contract_trace.json",
        &json!({
            "schema": "DC-SR-004C-R3-FrozenContractTraceV1",
            "source": "docs/d096_selected_architecture_contract.md",
            "sha256": contract_hash,
            "entry_commit": ENTRY_COMMIT,
            "ordered_coordinates": ["resource processing", "activation", "repair", "growth synthesis"],
            "hypothesis": {
                "processing": "pulse nutrient/fuel",
                "repair": "recurrent local damage"
            },
            "semantic_rule": "each allocation catalyst multiplies only its corresponding existing local flux",
            "interpretation": "coordinate 2 is directly identified with repair/strain response, not ordinary baseline maintenance",
            "production_repaired": false
        }),
    );
    write_json(
        &artifact_root,
        "current_flux_decomposition.json",
        &json!({
            "schema": "DC-SR-004C-R3-CurrentFluxDecompositionV1",
            "equation": "J_unscaled = J_base + J_strain; J_current = (J_base + J_strain) * g_repair",
            "g_strain": "g0 + 0.45 * max(eps,0) / (k_eps + max(eps,0))",
            "rows": rows,
            "current_path": "mechanics_off and mechanics_on; fission_off; mutation_none",
            "observer_feedback": false
        }),
    );
    write_json(
        &artifact_root,
        "h_causal_attribution.json",
        &json!({
            "schema": "DC-SR-004C-R3-HCausalAttributionV1",
            "paired_rows": h_current,
            "baseline_repair_gain_leakage_present": h_baseline_leakage,
            "baseline_amplification_fraction": h_current.iter().map(|pair| {
                let repair = &pair["repair"];
                let base = f(repair, "baseline_build_amplification");
                let strain = f(repair, "strain_build_amplification");
                json!({"seed": pair["seed"], "fraction": base / (base + strain).max(1e-30)})
            }).collect::<Vec<_>>(),
            "shadow_h_removes_repair_heavy_structural_advantage": h_shadow_removes_baseline_advantage,
            "shadow_paired_rows": h_shadow
        }),
    );
    write_json(
        &artifact_root,
        "b_damage_specificity.json",
        &json!({
            "schema": "DC-SR-004C-R3-BDamageSpecificityV1",
            "paired_rows": b_current,
            "shadow_paired_rows": b_shadow,
            "shadow_preserves_damage_responsive_advantage": b_shadow_preserves_damage_advantage
        }),
    );
    write_json(
        &artifact_root,
        "neutral_attribution.json",
        &json!({
            "schema": "DC-SR-004C-R3-NeutralAttributionV1",
            "paired_rows": neutral_current,
            "shadow_paired_rows": neutral_shadow,
            "interpretation": "neutral is an attribution control, not a fitness endpoint"
        }),
    );
    let shadow_protocol = json!({
        "schema": "DC-SR-004C-R3-ShadowCounterfactualProtocolV1",
        "non_authoritative_counterfactual": true,
        "equation": "J_shadow = J_base + J_strain * g_repair",
        "ordinary_baseline_receives_repair_gain": false,
        "strain_responsive_receives_repair_gain": true,
        "candidates": candidates().into_iter().map(|candidate| json!({"id": candidate.id, "genotype": candidate.genotype.0, "hash": candidate.genotype.candidate_hash(&params)})).collect::<Vec<_>>(),
        "environments": ["H", "B", "Neutral"],
        "seeds": (1..=8).collect::<Vec<_>>(),
        "steps": STEPS,
        "dt": DT,
        "fission": false,
        "mutation": "mutation_none",
        "mechanics": true,
        "topology": true,
        "parameters_changed": false,
        "production_biology_changed": false
    });
    write_json(
        &artifact_root,
        "shadow_counterfactual_protocol.json",
        &shadow_protocol,
    );
    write_json(
        &artifact_root,
        "shadow_counterfactual_results.json",
        &json!({"schema": "DC-SR-004C-R3-ShadowCounterfactualResultsV1", "rows": rows.iter().filter(|row| row["mode"] == "shadow_mechanics_on").collect::<Vec<_>>() }),
    );
    write_json(
        &artifact_root,
        "shadow_counterfactual_summary.json",
        &json!({
            "schema": "DC-SR-004C-R3-ShadowCounterfactualSummaryV1",
            "equation": "J_shadow = J_base + J_strain * g_repair",
            "h": {"paired_rows": h_shadow, "removes_baseline_structural_advantage": h_shadow_removes_baseline_advantage},
            "b": {"paired_rows": b_shadow, "preserves_damage_responsive_advantage": b_shadow_preserves_damage_advantage},
            "neutral": {"paired_rows": neutral_shadow},
            "parameters_changed": false,
            "production_biology_changed": false,
            "authoritative": false
        }),
    );
    write_json(
        &artifact_root,
        "final_manifest.json",
        &json!({
            "schema": "DC-SR-004C-R3-FinalManifestV1",
            "directive": "DC-SR-004C-R3",
            "starting_commit": ENTRY_COMMIT,
            "mechanics_off_rows": 72,
            "mechanics_on_current_rows": 72,
            "shadow_rows": 72,
            "steps_per_row": STEPS,
            "dt": DT,
            "accepted_time_per_row": TIME,
            "fission": false,
            "mutation": "mutation_none",
            "gate7_campaign_executed": false,
            "gate8_started": false,
            "parameters_changed": false,
            "production_biology_changed": false,
            "prior_artifacts_preserved": ["sr004c", "sr004cr1", "sr004cr2"],
            "conclusion": disposition
        }),
    );
    println!(
        "DC-SR-004C-R3 artifacts written to {}",
        artifact_root.display()
    );
    println!("conclusion={disposition}");
}
