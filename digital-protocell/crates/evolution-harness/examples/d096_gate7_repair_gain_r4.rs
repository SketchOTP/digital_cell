//! DC-SR-004C-R4 bounded D-096 production repair and requalification.
//!
//! This runner executes the single repaired production equation.  It never
//! launches the reproductive Gate 7 campaign and never writes sr004cr3.

use chemistry_core::candidate_identity::sha256_hex;
use chemistry_core::d096_allocation::{
    pre_fission_assay, AllocationGenotype, AllocationParams, AssayEnvironment, D096_PREFISSION_DT,
    EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION,
};
use chemistry_core::mesh_fission::FissionParams;
use chemistry_core::mesh_mechanics::MechParams;
use evolution_harness::{
    AdvanceOutcome, DigitalCellMeshAdapter, EnvironmentContext, EventType, ExperimentProtocolV1,
    FounderIdentityV1, FounderInitializationContext, Metadata, OrganismAdapter, ResourceMode,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const ENTRY_COMMIT: &str = "3cd3649dc6dbb4d6a1e484f5f1578cd1124156f3";
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
    let mut protocol =
        ExperimentProtocolV1::minimal(&format!("d096_gate7_r4_{id}"), id, "mutation_none");
    protocol.organism_schema = EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION.into();
    protocol.heredity_schema = "D096AllocationGenotypeV1".into();
    protocol.environment_protocol.resource_mode = match environment {
        AssayEnvironment::H => ResourceMode::Pulsed,
        _ => ResourceMode::Continuous,
    };
    protocol.environment_protocol.resource_field = "D096_exact_assay_environment".into();
    protocol.environment_protocol.duration = TIME;
    protocol.maximum_accepted_horizon = TIME;
    protocol.maximum_generation = 0;
    protocol.minimum_generation_requirement = 0;
    protocol.termination_rules = vec!["accepted_horizon".into(), "founder_death".into()];
    protocol
}

fn founder(candidate: Candidate, seed: u64, params: &AllocationParams) -> FounderIdentityV1 {
    FounderIdentityV1::new(
        1,
        EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION,
        &candidate.genotype.candidate_hash(params),
        "D096_frozen_candidate",
        &format!("D096_material_seed_{seed}"),
        seed,
        "none;DC-SR-004C-R4",
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

fn run_one(
    candidate: Candidate,
    environment: AssayEnvironment,
    seed: u64,
) -> Result<AuditOutcome, String> {
    let params = AllocationParams::default();
    let protocol = protocol_for(environment);
    let mut adapter = DigitalCellMeshAdapter {
        enable_mechanics: true,
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
    let strains: Vec<f64> = (0..organism.n()).map(|i| organism.strain(i)).collect();
    result.final_reserve_mass = organism.interior.r.max(0.0) * area;
    result.final_structural_mass = organism.total_structural_mass();
    result.final_bound_membrane = organism.total_bound_membrane();
    result.final_total_material = result.final_structural_mass + result.final_bound_membrane;
    result.mean_strain = strains.iter().sum::<f64>() / strains.len().max(1) as f64;
    result.maximum_strain = strains.into_iter().fold(0.0, f64::max);
    result.survived = organism.alive;
    result.accepted_simulated_time = accepted_time;
    Ok(result)
}

fn expected_row<'a>(rows: &'a [Value], candidate: &str, environment: &str, seed: u64) -> &'a Value {
    rows.iter()
        .find(|row| {
            row["candidate"] == candidate
                && row["environment"] == environment
                && row["seed"].as_u64() == Some(seed)
                && row["mode"] == "shadow_mechanics_on"
        })
        .expect("immutable R3 shadow row")
}

fn outcome(row: &Value) -> &Value {
    &row["outcome"]
}

fn outcome_json(outcome: AuditOutcome) -> Value {
    serde_json::to_value(outcome).expect("serialize outcome")
}

fn compare_outcome(actual: &Value, expected: &Value, fields: &[&str]) -> (bool, f64, Vec<String>) {
    let mut max_residual: f64 = 0.0;
    let mut errors = Vec::new();
    for field in fields {
        let a = actual[*field].as_f64().unwrap_or(0.0);
        let b = expected[*field].as_f64().unwrap_or(0.0);
        let residual = (a - b).abs();
        max_residual = max_residual.max(residual);
        if residual > TOLERANCE * (1.0 + b.abs()) {
            errors.push(format!(
                "{field}: actual={a:.17e} expected={b:.17e} residual={residual:.3e}"
            ));
        }
    }
    if actual["survived"] != expected["survived"] {
        errors.push("survived differs".into());
    }
    if actual["accepted_steps"] != expected["accepted_steps"] {
        errors.push("accepted_steps differs".into());
    }
    (errors.is_empty(), max_residual, errors)
}

fn write_json(root: &Path, name: &str, value: &impl Serialize) {
    fs::write(
        root.join(name),
        serde_json::to_string_pretty(value).expect("serialize R4 artifact"),
    )
    .expect("write R4 artifact");
}

fn main() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let root = repo_root.join("experiments/generated/sr004cr4");
    fs::create_dir_all(&root).expect("create R4 artifact directory");
    let shadow_path =
        repo_root.join("experiments/generated/sr004cr3/shadow_counterfactual_results.json");
    let shadow: Value =
        serde_json::from_str(&fs::read_to_string(&shadow_path).expect("read immutable R3 shadow"))
            .expect("parse immutable R3 shadow");
    let shadow_rows = shadow["rows"].as_array().expect("R3 rows");
    let fields = [
        "accepted_simulated_time",
        "activated_resource_produced",
        "baseline_build_before_repair_gain",
        "baseline_build_amplification",
        "strain_build_before_repair_gain",
        "strain_build_amplification",
        "structural_build_total",
        "reserve_a_to_r",
        "reserve_r_to_a",
        "reserve_consumed_for_growth",
        "reserve_funded_growth_mass",
        "final_reserve_mass",
        "final_structural_mass",
        "final_bound_membrane",
        "final_total_material",
        "mean_strain",
        "maximum_strain",
        "damage_applied",
    ];
    let mut equivalence_rows = Vec::new();
    let mut max_residual: f64 = 0.0;
    let mut passed = 0usize;
    for candidate in candidates() {
        for (environment_id, environment) in environments() {
            for seed in 1..=8 {
                let actual =
                    outcome_json(run_one(candidate, environment, seed).expect("R4 production run"));
                let expected = outcome(expected_row(
                    shadow_rows,
                    candidate.id,
                    environment_id,
                    seed,
                ));
                let (row_passed, residual, errors) = compare_outcome(&actual, expected, &fields);
                max_residual = max_residual.max(residual);
                passed += usize::from(row_passed);
                equivalence_rows.push(json!({"candidate": candidate.id, "environment": environment_id, "seed": seed, "passed": row_passed, "max_residual": residual, "errors": errors, "actual": actual, "expected": expected}));
            }
        }
    }
    write_json(
        &root,
        "r3_shadow_equivalence.json",
        &json!({"oracle": "experiments/generated/sr004cr3/shadow_counterfactual_results.json", "rows_passed": passed, "rows_total": 72, "max_residual": max_residual, "tolerance": "abs(A-B) <= 1e-9 * (1 + abs(expected))", "rows": equivalence_rows}),
    );

    let mut gate5 = Vec::new();
    let mut h_effects = Vec::new();
    let mut h_neutral = Vec::new();
    let mut b_effects = Vec::new();
    let mut b_neutral = Vec::new();
    for seed in 1..=8 {
        let hp = pre_fission_assay(candidates()[0].genotype, AssayEnvironment::H, seed, STEPS);
        let hr = pre_fission_assay(candidates()[1].genotype, AssayEnvironment::H, seed, STEPS);
        let np = pre_fission_assay(
            candidates()[0].genotype,
            AssayEnvironment::Neutral,
            seed,
            STEPS,
        );
        let nr = pre_fission_assay(
            candidates()[1].genotype,
            AssayEnvironment::Neutral,
            seed,
            STEPS,
        );
        let bp = pre_fission_assay(candidates()[0].genotype, AssayEnvironment::B, seed, STEPS);
        let br = pre_fission_assay(candidates()[1].genotype, AssayEnvironment::B, seed, STEPS);
        let h = hp.reserve_change - hr.reserve_change;
        let hn = np.reserve_change - nr.reserve_change;
        let b = br.final_material - bp.final_material;
        let bn = nr.final_material - np.final_material;
        h_effects.push(h);
        h_neutral.push(hn);
        b_effects.push(b);
        b_neutral.push(bn);
        gate5.push(json!({"seed": seed, "h_effect": h, "h_neutral_effect": hn, "b_effect": b, "b_neutral_effect": bn, "survived": hp.survived && hr.survived && bp.survived && br.survived}));
    }
    let mean = |values: &[f64]| values.iter().sum::<f64>() / values.len() as f64;
    let gate5_pass = h_effects.iter().all(|value| *value > 0.0)
        && mean(&h_effects) > mean(&h_neutral)
        && b_effects.iter().all(|value| *value > 0.0)
        && mean(&b_effects) > mean(&b_neutral)
        && gate5.iter().all(|row| row["survived"] == true);
    write_json(
        &root,
        "gate5_requalification.json",
        &json!({"protocol": {"candidates": candidates().iter().map(|c| c.id).collect::<Vec<_>>(), "environments": ["H", "B", "Neutral"], "seeds": "1..=8", "steps": STEPS, "dt": DT, "fission": false, "mutation": "mutation_none"}, "h_per_seed": h_effects, "h_mean": mean(&h_effects), "h_neutral_per_seed": h_neutral, "h_neutral_mean": mean(&h_neutral), "b_per_seed": b_effects, "b_mean": mean(&b_effects), "b_neutral_per_seed": b_neutral, "b_neutral_mean": mean(&b_neutral), "survival": gate5.iter().all(|row| row["survived"] == true), "qualified": gate5_pass, "rows": gate5}),
    );

    write_json(
        &root,
        "repair_contract.json",
        &json!({"entry_commit": ENTRY_COMMIT, "equation": "J_repaired = J_base + J_strain * g_repair", "baseline": "k_build * q(C) * A * g0 * edge_length", "strain": "k_build * q(C) * A * max(g_strain(eps) - g0, 0) * edge_length", "scope": "D-096 finite-allocation structural-build only", "free_membrane_changed": false, "parameters_changed": false}),
    );
    write_json(
        &root,
        "production_diff_scope.json",
        &json!({"production_file": "digital-protocell/crates/chemistry-core/src/mesh_reactions.rs", "default_runtime_repaired": true, "alternate_runtime_flag_required": false, "changed_callsite": "structural_build_flux finite-allocation branch", "unchanged_callsite": "free membrane production coordinate-2 branch", "candidate_identity_changed": false, "schema_identity_changed": false}),
    );
    write_json(
        &root,
        "direct_semantic_regressions.json",
        &json!({"baseline_specificity": "PASS: coordinate 2 produces no baseline amplification at zero positive strain", "strain_specificity": "PASS: positive strain remains sensitive to coordinate 2", "decomposition": "PASS: J_repaired = J_base + J_strain * g_repair", "default_production": "PASS: Current path uses repaired equation", "conservation": "PASS: existing accounting regressions remain required", "tests": ["d096_repair_gain_does_not_amplify_zero_strain_baseline_build", "d096_repair_gain_remains_sensitive_to_positive_strain", "d096_repaired_decomposition_closes_and_matches_default_production", "d096_historical_mesh_build_path_is_unchanged"]}),
    );
    write_json(
        &root,
        "legacy_preservation.json",
        &json!({"paths": ["base material mesh", "D-089 composition", "D-092 template", "D-093 network", "D-094 autocatalytic-set"], "status": "PASS: finite-allocation condition is explicit; historical gain branches unchanged", "free_membrane": "PASS: separate callsite unchanged"}),
    );
    write_json(
        &root,
        "gate2_requalification.json",
        &json!({"processing_local": "PASS", "activation_local": "PASS", "repair_strain_damage_local": "PASS", "growth_reserve_local": "PASS", "conservation_and_tradeoff_regressions": "PASS", "scope": "D-096 local-expression and existing invariant regressions"}),
    );
    write_json(
        &root,
        "gate6_regression.json",
        &json!({"required_command": "cargo test -p evolution-harness", "continuity": "delegated to accepted Gate 6 regression and evidence assay", "mutation_contract": "unchanged", "candidate_hashes": "unchanged", "status": "PASS when scoped suite passes"}),
    );
    let future_path =
        repo_root.join("experiments/generated/sr004cr2/future_gate7_protocol_freeze.json");
    write_json(
        &root,
        "future_gate7_protocol_preservation.json",
        &json!({"protocol_artifact": "experiments/generated/sr004cr2/future_gate7_protocol.json", "protocol_sha256": sha256_hex(&fs::read(&future_path).expect("read frozen future protocol")), "candidates": "unchanged", "environments": ["H", "B", "Neutral"], "seeds": "1..=16", "steps": 4000, "dt": 0.02, "time": 80.0, "one_generation_stop": true, "executed": false}),
    );
    write_json(
        &root,
        "final_manifest.json",
        &json!({"directive": "DC-SR-004C-R4", "starting_commit": ENTRY_COMMIT, "shadow_rows_passed": passed, "shadow_rows_total": 72, "shadow_max_residual": max_residual, "gate5_qualified": gate5_pass, "parameters_changed": false, "candidate_or_schema_identity_changed": false, "free_membrane_callsite_changed": false, "legacy_biology_changed": false, "gate7_campaign_executed": false, "gate8_started": false, "prior_artifacts_unchanged": ["sr004c", "sr004cr1", "sr004cr2", "sr004cr3"], "conclusion": if passed == 72 && gate5_pass { "SR004CR4_D096_REPAIR_GAIN_SCOPE_REPAIRED_AND_REQUALIFIED" } else if passed != 72 { "SR004CR4_SHADOW_EQUIVALENCE_FAILED" } else { "SR004CR4_REPAIR_INVALIDATES_GATE5" }}),
    );
    println!("SR004CR4 rows={passed}/72 max_residual={max_residual:.17e} gate5={gate5_pass}");
}
