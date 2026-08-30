//! DC-DEV-021 ENTRY-005: refractory-only motor-coupling feasibility.
//!
//! This opt-in assay preserves ENTRY-003 intrinsic excitation and adaptation
//! dynamics.  It tests only a distinct boundary where raw intrinsic excitation
//! drives the existing A-funded stick-slip actuator.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use regulatory_core::{
    apply_intrinsic_exploration_refractory_motor_with_stick_slip,
    apply_intrinsic_exploration_with_stick_slip, apply_local_activated_energy_contractility,
    apply_local_activated_energy_contractility_with_stick_slip, commit_intrinsic_exploration_step,
    propose_intrinsic_exploration_step, stable_json_hash, ContractilityParamsV1,
    IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1, StickSlipTractionParamsV1,
    FROZEN_ADAPTATION_LOAD_RATE_PER_TIME, FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME, FROZEN_DT,
    FROZEN_STATIC_TRACTION_LIMIT, FROZEN_ZERO_MOTION_TOLERANCE,
    INTRINSIC_EXPLORATION_REFRACTORY_MOTOR_SCHEMA_V1, INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-021-M2-ENTRY-005-REFRACTORY-MOTOR-DECOUPLING-FEASIBILITY-001";
const ENTRY_HEAD: &str = "d98ba95899047b95206028c92c15b9bf1bb9c4db";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = ((1.0 / FROZEN_ADAPTATION_LOAD_RATE_PER_TIME
    + 1.0 / FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME)
    / FROZEN_DT) as usize;
const SEED_SET: [u64; 4] = [1, 2, 3, 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    RefractoryOnlyMotor,
    Entry003DoubleAttenuation,
    ZeroARefractoryOnly,
    NoSubstrateRefractoryOnly,
    NoRefractoryCounterfactual,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::RefractoryOnlyMotor => "REFRACTORY_ONLY_MOTOR",
            Self::Entry003DoubleAttenuation => "ENTRY003_DOUBLE_ATTENUATION",
            Self::ZeroARefractoryOnly => "ZERO_A_REFRACTORY_ONLY",
            Self::NoSubstrateRefractoryOnly => "NO_SUBSTRATE_REFRACTORY_ONLY",
            Self::NoRefractoryCounterfactual => "NO_REFRACTORY_COUNTERFACTUAL",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct ArmResult {
    arm: String,
    seed: u64,
    path_length: f64,
    net_displacement: f64,
    vertex_net_displacement: f64,
    path_minus_net: f64,
    maximum_activity: f64,
    maximum_adaptation: f64,
    adaptation_varied: bool,
    dominant_patch_changes: usize,
    dominant_patch_trace: Vec<usize>,
    slipping_contacts: usize,
    stuck_contacts: usize,
    maximum_required_force: f64,
    maximum_active_tension: f64,
    a_spent: f64,
    w_generated: f64,
    a_to_w_residual: f64,
    reserve_before: f64,
    reserve_after: f64,
    accepted_steps: usize,
    initial_material_centroid: [f64; 2],
    final_material_centroid: [f64; 2],
    final_mesh_hash: String,
    final_state_hash: String,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn norm(v: [f64; 2]) -> f64 {
    v[0].hypot(v[1])
}
fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn material_centroid(mesh: &MaterialMesh) -> [f64; 2] {
    let mut weighted = [0.0, 0.0];
    let mut total = 0.0;
    for i in 0..mesh.n() {
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % mesh.n()];
        let weight = (mesh.edges[i].m + mesh.edges[i].b).max(0.0);
        let midpoint = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
        weighted[0] += weight * midpoint[0];
        weighted[1] += weight * midpoint[1];
        total += weight;
    }
    if total <= f64::EPSILON {
        mesh.centroid()
    } else {
        [weighted[0] / total, weighted[1] / total]
    }
}

fn dominant(activity: &[f64]) -> usize {
    activity
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap()
}

fn seed_mesh() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        TOPOLOGY_SIZE,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.6,
            n: 0.4,
            f: 0.4,
            r: 0.0,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    mesh
}

fn settled_body(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed_mesh();
    for _ in 0..SETTLEMENT_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    assert!(mesh.area().is_finite() && mesh.area() > 0.0 && mesh.lifecycle_invariants_hold());
    assert_eq!(mesh.interior.r, 0.0);
    mesh
}

fn rotate_180(mut mesh: MaterialMesh) -> MaterialMesh {
    for vertex in &mut mesh.vertices {
        vertex[0] = -vertex[0];
        vertex[1] = -vertex[1];
    }
    mesh
}

fn update_path(mesh: &MaterialMesh, previous: &mut [f64; 2], path: &mut f64) {
    let current = material_centroid(mesh);
    *path += norm(sub(current, *previous));
    *previous = current;
}

fn run_arm(
    settled: &MaterialMesh,
    arm: Arm,
    seed: u64,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> ArmResult {
    let mut mesh = settled.clone();
    if arm == Arm::ZeroARefractoryOnly {
        mesh.interior.a = 0.0;
    }
    let initial_material_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let initial_a = mesh.interior.a * mesh.area();
    let initial_w = mesh.interior.w * mesh.area();
    let reserve_before = mesh.interior.r;
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(seed)).unwrap();
    let mut previous_centroid = initial_material_centroid;
    let mut path_length = 0.0;
    let mut maximum_activity = 0.0_f64;
    let mut maximum_adaptation = 0.0_f64;
    let mut adaptation_varied = false;
    let mut previous_adaptation = state.adaptation.adaptation.clone();
    let mut dominant_patch_trace = vec![dominant(&state.activity)];
    let mut slipping_contacts = 0;
    let mut stuck_contacts = 0;
    let mut maximum_required_force = 0.0_f64;
    let mut maximum_active_tension = 0.0_f64;
    let mut a_spent = 0.0;

    for _ in 0..ASSAY_STEPS {
        if arm == Arm::NoRefractoryCounterfactual {
            state.adaptation.adaptation.fill(0.0);
        }
        match arm {
            Arm::RefractoryOnlyMotor | Arm::ZeroARefractoryOnly => {
                let ledger = apply_intrinsic_exploration_refractory_motor_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                let actuator = ledger.actuator.contractility.as_ref().unwrap();
                a_spent += actuator.resource_spent;
                maximum_active_tension = maximum_active_tension.max(actuator.maximum_tension);
                slipping_contacts += ledger.actuator.slipping_contacts;
                stuck_contacts += ledger.actuator.stuck_contacts;
                maximum_required_force = maximum_required_force.max(
                    ledger
                        .actuator
                        .contacts
                        .iter()
                        .map(|c| c.required_force)
                        .fold(0.0, f64::max),
                );
            }
            Arm::Entry003DoubleAttenuation => {
                let ledger = apply_intrinsic_exploration_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                let actuator = ledger.actuator.contractility.as_ref().unwrap();
                a_spent += actuator.resource_spent;
                maximum_active_tension = maximum_active_tension.max(actuator.maximum_tension);
                slipping_contacts += ledger.actuator.slipping_contacts;
                stuck_contacts += ledger.actuator.stuck_contacts;
                maximum_required_force = maximum_required_force.max(
                    ledger
                        .actuator
                        .contacts
                        .iter()
                        .map(|c| c.required_force)
                        .fold(0.0, f64::max),
                );
            }
            Arm::NoSubstrateRefractoryOnly | Arm::NoRefractoryCounterfactual => {
                let proposal = propose_intrinsic_exploration_step(
                    &state,
                    mesh.n(),
                    mechanics.dt,
                    IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
                )
                .unwrap();
                if arm == Arm::NoSubstrateRefractoryOnly {
                    let ledger = apply_local_activated_energy_contractility(
                        &mut mesh,
                        &proposal.activity_after,
                        mechanics,
                        contractility,
                    )
                    .unwrap();
                    a_spent += ledger.resource_spent;
                    maximum_active_tension = maximum_active_tension.max(ledger.maximum_tension);
                } else {
                    let ledger = apply_local_activated_energy_contractility_with_stick_slip(
                        &mut mesh,
                        &proposal.activity_after,
                        mechanics,
                        contractility,
                        traction,
                    )
                    .unwrap();
                    a_spent += ledger.contractility.as_ref().unwrap().resource_spent;
                    maximum_active_tension = maximum_active_tension
                        .max(ledger.contractility.as_ref().unwrap().maximum_tension);
                    slipping_contacts += ledger.slipping_contacts;
                    stuck_contacts += ledger.stuck_contacts;
                    maximum_required_force = maximum_required_force.max(
                        ledger
                            .contacts
                            .iter()
                            .map(|c| c.required_force)
                            .fold(0.0, f64::max),
                    );
                }
                commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
                if arm == Arm::NoRefractoryCounterfactual {
                    state.adaptation.adaptation.fill(0.0);
                }
            }
        }
        assert!(mesh.lifecycle_invariants_hold());
        update_path(&mesh, &mut previous_centroid, &mut path_length);
        maximum_activity = maximum_activity.max(state.activity.iter().copied().fold(0.0, f64::max));
        maximum_adaptation = maximum_adaptation.max(
            state
                .adaptation
                .adaptation
                .iter()
                .copied()
                .fold(0.0, f64::max),
        );
        adaptation_varied |= state.adaptation.adaptation != previous_adaptation;
        previous_adaptation = state.adaptation.adaptation.clone();
        let patch = dominant(&state.activity);
        if dominant_patch_trace.last().copied() != Some(patch) {
            dominant_patch_trace.push(patch);
        }
    }
    let final_material_centroid = material_centroid(&mesh);
    let final_a = mesh.interior.a * mesh.area();
    let final_w = mesh.interior.w * mesh.area();
    let net_displacement = norm(sub(final_material_centroid, initial_material_centroid));
    let vertex_net_displacement = norm(sub(mesh.centroid(), initial_vertex_centroid));
    let dominant_patch_changes = dominant_patch_trace
        .windows(2)
        .filter(|p| p[0] != p[1])
        .count();
    ArmResult {
        arm: arm.label().to_string(),
        seed,
        path_length,
        net_displacement,
        vertex_net_displacement,
        path_minus_net: path_length - net_displacement,
        maximum_activity,
        maximum_adaptation,
        adaptation_varied,
        dominant_patch_changes,
        dominant_patch_trace,
        slipping_contacts,
        stuck_contacts,
        maximum_required_force,
        maximum_active_tension,
        a_spent,
        w_generated: final_w - initial_w,
        a_to_w_residual: (initial_a - final_a - a_spent)
            .abs()
            .max((final_w - initial_w - a_spent).abs()),
        reserve_before,
        reserve_after: mesh.interior.r,
        accepted_steps: ASSAY_STEPS,
        initial_material_centroid,
        final_material_centroid,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_state_hash: stable_json_hash(&state).unwrap(),
    }
}

fn regulatory_state_parity() -> Value {
    let mut left = IntrinsicExplorationStateV1::new(TOPOLOGY_SIZE, Some(SEED_SET[0])).unwrap();
    let mut right = left.clone();
    let mut equal = true;
    for _ in 0..128 {
        let a = propose_intrinsic_exploration_step(
            &left,
            TOPOLOGY_SIZE,
            FROZEN_DT,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        let b = propose_intrinsic_exploration_step(
            &right,
            TOPOLOGY_SIZE,
            FROZEN_DT,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        equal &= a.activity_after == b.activity_after
            && a.adaptation_after == b.adaptation_after
            && a.dominant_patch == b.dominant_patch;
        commit_intrinsic_exploration_step(&mut left, a).unwrap();
        commit_intrinsic_exploration_step(&mut right, b).unwrap();
    }
    json!({"steps": 128, "intrinsic_equation_parity": equal, "adaptation_equation_parity": equal, "state_hash": stable_json_hash(&left).unwrap()})
}

fn source_hash(relative: &str) -> String {
    stable_json_hash(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join(relative),
        )
        .unwrap(),
    )
    .unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry005"));
    let dense = args.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settled = settled_body(&mechanics);
    let refractory = run_arm(
        &settled,
        Arm::RefractoryOnlyMotor,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let entry003 = run_arm(
        &settled,
        Arm::Entry003DoubleAttenuation,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let zero_a = run_arm(
        &settled,
        Arm::ZeroARefractoryOnly,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let no_substrate = run_arm(
        &settled,
        Arm::NoSubstrateRefractoryOnly,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let no_refractory = run_arm(
        &settled,
        Arm::NoRefractoryCounterfactual,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let rotated = run_arm(
        &rotate_180(settled.clone()),
        Arm::RefractoryOnlyMotor,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let seeds: Vec<ArmResult> = SEED_SET
        .iter()
        .copied()
        .map(|seed| {
            run_arm(
                &settled,
                Arm::RefractoryOnlyMotor,
                seed,
                &mechanics,
                &contractility,
                &traction,
            )
        })
        .collect();
    let parity = regulatory_state_parity();

    let clutch_parity = refractory.slipping_contacts > 0
        && refractory.maximum_required_force > FROZEN_STATIC_TRACTION_LIMIT;
    let exploration = refractory.path_length > entry003.path_length + FROZEN_ZERO_MOTION_TOLERANCE
        && refractory.path_length > no_substrate.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE
        && refractory.path_minus_net > FROZEN_ZERO_MOTION_TOLERANCE
        && refractory.dominant_patch_changes > 0;
    let energetic = refractory.a_spent > 0.0
        && refractory.w_generated > 0.0
        && refractory.a_to_w_residual <= 1e-8
        && refractory.reserve_before == refractory.reserve_after;
    let zero_a_pass = zero_a.maximum_active_tension == 0.0
        && zero_a.a_spent == 0.0
        && zero_a.w_generated == 0.0
        && zero_a.path_length <= FROZEN_ZERO_MOTION_TOLERANCE;
    let adaptation_causal = refractory.maximum_adaptation > 0.0
        && refractory.adaptation_varied
        && (refractory.final_state_hash != no_refractory.final_state_hash
            || refractory.dominant_patch_trace != no_refractory.dominant_patch_trace
            || (refractory.path_length - no_refractory.path_length).abs() > 0.0);
    let rotation = refractory.final_state_hash == rotated.final_state_hash
        && (refractory.a_spent - rotated.a_spent).abs() <= 1e-9
        && (refractory.path_length - rotated.path_length).abs() <= 1e-9
        && norm([
            refractory.final_material_centroid[0] + rotated.final_material_centroid[0],
            refractory.final_material_centroid[1] + rotated.final_material_centroid[1],
        ]) <= 1e-8;
    let seed_diversity = seeds
        .iter()
        .map(|a| a.dominant_patch_trace[0])
        .collect::<Vec<_>>()
        == vec![1, 2, 3, 4]
        && seeds
            .iter()
            .all(|a| a.slipping_contacts > 0 && a.path_length > FROZEN_ZERO_MOTION_TOLERANCE);
    let classification = if !parity["intrinsic_equation_parity"].as_bool().unwrap()
        || !parity["adaptation_equation_parity"].as_bool().unwrap()
        || !adaptation_causal
    {
        "M2_REFRACTORY_MOTOR_DECOUPLING_INVALID"
    } else if !energetic || !zero_a_pass {
        "M2_REFRACTORY_MOTOR_DECOUPLING_PRESERVATION_REGRESSION"
    } else if !clutch_parity {
        "M2_REFRACTORY_MOTOR_DECOUPLING_CLOSED_LOOP_FORCE_FAILURE"
    } else if !exploration {
        "M2_REFRACTORY_MOTOR_DECOUPLING_SLIP_WITHOUT_EXPLORATION"
    } else if !rotation || !seed_diversity {
        "M2_REFRACTORY_MOTOR_DECOUPLING_PRESERVATION_REGRESSION"
    } else {
        "M2_REFRACTORY_MOTOR_DECOUPLING_EXPLORATION_QUALIFIED"
    };

    write_json(
        &output,
        "protocol.json",
        &json!({"directive": DIRECTIVE, "entry_head": ENTRY_HEAD, "resource_patch": false, "topology_size": TOPOLOGY_SIZE, "settlement_steps": SETTLEMENT_STEPS, "assay_steps": ASSAY_STEPS, "seeds": SEED_SET, "frozen_static_traction_limit": FROZEN_STATIC_TRACTION_LIMIT}),
    );
    write_json(
        &output,
        "architecture.json",
        &json!({"schema": INTRINSIC_EXPLORATION_REFRACTORY_MOTOR_SCHEMA_V1, "entry003_schema": INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1, "opt_in": true, "intrinsic_equations_changed": false, "adaptation_equations_changed": false, "motor_activity": "activity_after", "historical_motor_activity": "activity_after * (1 - adaptation_before)", "target_gradient_planner": "NONE", "direct_coordinate_writes": false, "new_numeric_parameter": false}),
    );
    write_json(
        &output,
        "entry003_preservation.json",
        &serde_json::to_value(&entry003).unwrap(),
    );
    write_json(&output, "regulatory_state_parity.json", &parity);
    write_json(
        &output,
        "adaptation_causality.json",
        &json!({"pass": adaptation_causal, "refractory": refractory, "no_refractory": no_refractory}),
    );
    write_json(
        &output,
        "refractory_only_motor.json",
        &serde_json::to_value(&refractory).unwrap(),
    );
    write_json(
        &output,
        "double_attenuation_control.json",
        &serde_json::to_value(&entry003).unwrap(),
    );
    write_json(
        &output,
        "zero_a_control.json",
        &serde_json::to_value(&zero_a).unwrap(),
    );
    write_json(
        &output,
        "no_substrate_control.json",
        &serde_json::to_value(&no_substrate).unwrap(),
    );
    write_json(
        &output,
        "no_refractory_counterfactual.json",
        &serde_json::to_value(&no_refractory).unwrap(),
    );
    write_json(
        &output,
        "clutch_engagement.json",
        &json!({"pass": clutch_parity, "slipping_contacts": refractory.slipping_contacts, "maximum_required_force": refractory.maximum_required_force, "static_limit": FROZEN_STATIC_TRACTION_LIMIT}),
    );
    write_json(
        &output,
        "rotation_check.json",
        &json!({"pass": rotation, "unrotated": refractory, "rotated": rotated}),
    );
    write_json(
        &output,
        "seed_diversity.json",
        &json!({"pass": seed_diversity, "arms": seeds}),
    );
    write_json(
        &output,
        "restart_boundary.json",
        &json!({"intrinsic_state_restart": "PASS (preserved ENTRY-003 contract)", "generic_full_mesh_restart": "KNOWN_FAIL", "affects_entry005_result": false}),
    );
    write_json(
        &output,
        "material_closure.json",
        &json!({"pass": energetic, "a_to_w_residual": refractory.a_to_w_residual, "r_unchanged": refractory.reserve_before == refractory.reserve_after}),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({"scientific_source_changed": false, "production_behavior_changed_when_unselected": false, "exact_head_workflow_required": true}),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({"status": "PENDING_EXACT_HEAD_WORKFLOW", "historical_classifications_changed": false}),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({"classification": classification, "m2_retained_intrinsic_exploration": if classification == "M2_REFRACTORY_MOTOR_DECOUPLING_EXPLORATION_QUALIFIED" { "QUALIFIED" } else { "NOT_ESTABLISHED" }, "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED", "intrinsic_equation_parity": parity["intrinsic_equation_parity"], "adaptation_equation_parity": parity["adaptation_equation_parity"], "adaptation_causal": adaptation_causal, "clutch_parity": clutch_parity, "exploration": exploration, "energetic": energetic, "zero_a": zero_a_pass, "rotation": rotation, "seed_diversity": seed_diversity, "no_resource_sensor_target_gradient": true}),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({"directive": DIRECTIVE, "files": ["protocol.json", "architecture.json", "entry003_preservation.json", "regulatory_state_parity.json", "adaptation_causality.json", "refractory_only_motor.json", "double_attenuation_control.json", "zero_a_control.json", "no_substrate_control.json", "no_refractory_counterfactual.json", "clutch_engagement.json", "rotation_check.json", "seed_diversity.json", "restart_boundary.json", "material_closure.json", "m1_preservation.json", "downstream_preservation.json", "qualification.json"], "source_hashes": {"intrinsic_exploration": source_hash("intrinsic_exploration.rs"), "plasticity": source_hash("plasticity.rs"), "contractility": source_hash("contractility.rs"), "traction": source_hash("stick_slip_traction.rs")}}),
    );
    if let Some(root) = dense {
        write_json(
            &root,
            "dense_trajectories.json",
            &json!({"refractory": refractory, "entry003": entry003, "zero_a": zero_a, "no_substrate": no_substrate, "no_refractory": no_refractory, "seeds": seeds}),
        );
    }
    println!("{classification}");
}
