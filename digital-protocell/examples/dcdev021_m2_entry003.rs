//! DC-DEV-021 ENTRY-003: intrinsic exploration feasibility.
//!
//! The primary assay has no resource patch, sensor input, target, gradient, or
//! world-direction input.  It compares one opt-in self-exciting local state
//! against the frozen zero regulator and the same one-time seed without self
//! excitation.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use regulatory_core::{
    apply_intrinsic_exploration_with_stick_slip, apply_local_activated_energy_contractility,
    apply_local_activated_energy_contractility_with_stick_slip, commit_intrinsic_exploration_step,
    propose_intrinsic_exploration_step, stable_json_hash, ContractilityParamsV1,
    IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1, StickSlipTractionParamsV1,
    FROZEN_ADAPTATION_LOAD_RATE_PER_TIME, FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME, FROZEN_DT,
    FROZEN_K_STIMULUS, FROZEN_ZERO_MOTION_TOLERANCE, INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-021-M2-ENTRY-003-INTRINSIC-EXPLORATION-FEASIBILITY-001";
const ENTRY_HEAD: &str = "2ed0f6159b0169f1f7bd9c2c10e89a6b67d12167";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const T_LOAD: f64 = 1.0 / FROZEN_ADAPTATION_LOAD_RATE_PER_TIME;
const T_RECOVERY: f64 = 1.0 / FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME;
const ASSAY_STEPS: usize = ((T_LOAD + T_RECOVERY) / FROZEN_DT) as usize;
const SEED_SET: [u64; 4] = [1, 2, 3, 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    IntrinsicExploration,
    CurrentRegulatorControl,
    SeedOnlyControl,
    ZeroA,
    NoSubstrate,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::IntrinsicExploration => "INTRINSIC_EXPLORATION",
            Self::CurrentRegulatorControl => "CURRENT_REGULATOR_CONTROL",
            Self::SeedOnlyControl => "SEED_ONLY_CONTROL",
            Self::ZeroA => "ZERO_A",
            Self::NoSubstrate => "NO_SUBSTRATE",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    arm: String,
    seed: u64,
    initial_material_centroid: [f64; 2],
    final_material_centroid: [f64; 2],
    initial_vertex_centroid: [f64; 2],
    final_vertex_centroid: [f64; 2],
    path_length: f64,
    net_displacement: f64,
    vertex_net_displacement: f64,
    path_minus_net: f64,
    maximum_activity: f64,
    dominant_patch_changes: usize,
    dominant_patch_trace: Vec<usize>,
    a_spent: f64,
    w_generated: f64,
    a_to_w_residual: f64,
    reserve_before: f64,
    reserve_after: f64,
    maximum_active_tension: f64,
    stuck_contacts: usize,
    slipping_contacts: usize,
    substrate_work: f64,
    accepted_steps: usize,
    activity_trace: Vec<Vec<f64>>,
    final_mesh_hash: String,
    final_state_hash: Option<String>,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn norm(value: [f64; 2]) -> f64 {
    value[0].hypot(value[1])
}

fn sub(left: [f64; 2], right: [f64; 2]) -> [f64; 2] {
    [left[0] - right[0], left[1] - right[1]]
}

fn material_centroid(mesh: &MaterialMesh) -> [f64; 2] {
    let mut weighted = [0.0, 0.0];
    let mut total = 0.0;
    for index in 0..mesh.n() {
        let a = mesh.vertices[index];
        let b = mesh.vertices[(index + 1) % mesh.n()];
        let weight = (mesh.edges[index].m + mesh.edges[index].b).max(0.0);
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

fn settle_body(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed_mesh();
    for _ in 0..SETTLEMENT_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    assert!(mesh.area().is_finite() && mesh.area() > 0.0);
    assert!(mesh.lifecycle_invariants_hold());
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

fn dominant(activity: &[f64]) -> usize {
    activity
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.partial_cmp(right).unwrap())
        .map(|(index, _)| index)
        .unwrap_or(0)
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
    if arm == Arm::ZeroA {
        mesh.interior.a = 0.0;
    }
    let initial_material_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let initial_a = mesh.interior.a * mesh.area();
    let initial_w = mesh.interior.w * mesh.area();
    let reserve_before = mesh.interior.r;
    let mut state = (arm != Arm::CurrentRegulatorControl)
        .then(|| IntrinsicExplorationStateV1::new(mesh.n(), Some(seed)).unwrap());
    let mut previous_material_centroid = initial_material_centroid;
    let mut path_length = 0.0;
    let mut maximum_activity = 0.0_f64;
    // Compact Git evidence retains only state-change and terminal frames. The
    // generated dense ledger is the place for a per-step trajectory.
    let mut dominant_patch_trace = state
        .as_ref()
        .map(|value| vec![dominant(&value.activity)])
        .unwrap_or_default();
    let mut activity_trace = state
        .as_ref()
        .map(|value| vec![value.activity.clone()])
        .unwrap_or_default();
    let mut a_spent = 0.0;
    let mut maximum_active_tension = 0.0_f64;
    let mut stuck_contacts = 0;
    let mut slipping_contacts = 0;
    let mut substrate_work = 0.0;

    for _ in 0..ASSAY_STEPS {
        match arm {
            Arm::CurrentRegulatorControl => {
                let zero_activity = vec![0.0; mesh.n()];
                let ledger = apply_local_activated_energy_contractility_with_stick_slip(
                    &mut mesh,
                    &zero_activity,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                stuck_contacts += ledger.stuck_contacts;
                slipping_contacts += ledger.slipping_contacts;
                substrate_work += ledger.substrate_work;
            }
            Arm::IntrinsicExploration | Arm::ZeroA => {
                let intrinsic = apply_intrinsic_exploration_with_stick_slip(
                    &mut mesh,
                    state.as_mut().unwrap(),
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                let contractility_ledger = intrinsic.actuator.contractility.unwrap();
                a_spent += contractility_ledger.resource_spent;
                maximum_active_tension =
                    maximum_active_tension.max(contractility_ledger.maximum_tension);
                stuck_contacts += intrinsic.actuator.stuck_contacts;
                slipping_contacts += intrinsic.actuator.slipping_contacts;
                substrate_work += intrinsic.actuator.substrate_work;
            }
            Arm::SeedOnlyControl | Arm::NoSubstrate => {
                let proposal = propose_intrinsic_exploration_step(
                    state.as_ref().unwrap(),
                    mesh.n(),
                    mechanics.dt,
                    if arm == Arm::SeedOnlyControl {
                        IntrinsicExplorationDynamicsModeV1::SeedOnlyControl
                    } else {
                        IntrinsicExplorationDynamicsModeV1::FullSelfExcitation
                    },
                )
                .unwrap();
                if arm == Arm::SeedOnlyControl {
                    let ledger = apply_local_activated_energy_contractility_with_stick_slip(
                        &mut mesh,
                        &proposal.effective_activity,
                        mechanics,
                        contractility,
                        traction,
                    )
                    .unwrap();
                    let contractility_ledger = ledger.contractility.unwrap();
                    a_spent += contractility_ledger.resource_spent;
                    maximum_active_tension =
                        maximum_active_tension.max(contractility_ledger.maximum_tension);
                    stuck_contacts += ledger.stuck_contacts;
                    slipping_contacts += ledger.slipping_contacts;
                    substrate_work += ledger.substrate_work;
                } else {
                    let ledger = apply_local_activated_energy_contractility(
                        &mut mesh,
                        &proposal.effective_activity,
                        mechanics,
                        contractility,
                    )
                    .unwrap();
                    a_spent += ledger.resource_spent;
                    maximum_active_tension = maximum_active_tension.max(ledger.maximum_tension);
                }
                commit_intrinsic_exploration_step(state.as_mut().unwrap(), proposal).unwrap();
            }
        }
        assert!(mesh.lifecycle_invariants_hold());
        let current_material_centroid = material_centroid(&mesh);
        path_length += norm(sub(current_material_centroid, previous_material_centroid));
        previous_material_centroid = current_material_centroid;
        if let Some(intrinsic) = &state {
            maximum_activity =
                maximum_activity.max(intrinsic.activity.iter().copied().fold(0.0, f64::max));
            let patch = dominant(&intrinsic.activity);
            if dominant_patch_trace.last().copied() != Some(patch) {
                dominant_patch_trace.push(patch);
                activity_trace.push(intrinsic.activity.clone());
            }
        }
    }

    let final_material_centroid = material_centroid(&mesh);
    let final_vertex_centroid = mesh.centroid();
    let final_a = mesh.interior.a * mesh.area();
    let final_w = mesh.interior.w * mesh.area();
    if let Some(intrinsic) = &state {
        if activity_trace.last() != Some(&intrinsic.activity) {
            activity_trace.push(intrinsic.activity.clone());
        }
    }
    let net_displacement = norm(sub(final_material_centroid, initial_material_centroid));
    let vertex_net_displacement = norm(sub(final_vertex_centroid, initial_vertex_centroid));
    let dominant_patch_changes = dominant_patch_trace
        .windows(2)
        .filter(|pair| pair[0] != pair[1])
        .count();
    ArmResult {
        arm: arm.label().to_string(),
        seed,
        initial_material_centroid,
        final_material_centroid,
        initial_vertex_centroid,
        final_vertex_centroid,
        path_length,
        net_displacement,
        vertex_net_displacement,
        path_minus_net: path_length - net_displacement,
        maximum_activity,
        dominant_patch_changes,
        dominant_patch_trace,
        a_spent,
        w_generated: final_w - initial_w,
        a_to_w_residual: (initial_a - final_a - a_spent)
            .abs()
            .max((final_w - initial_w - a_spent).abs()),
        reserve_before,
        reserve_after: mesh.interior.r,
        maximum_active_tension,
        stuck_contacts,
        slipping_contacts,
        substrate_work,
        accepted_steps: ASSAY_STEPS,
        activity_trace,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_state_hash: state.map(|value| stable_json_hash(&value).unwrap()),
    }
}

fn restart_check(
    settled: &MaterialMesh,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> Value {
    let uninterrupted = run_arm(
        settled,
        Arm::IntrinsicExploration,
        SEED_SET[0],
        mechanics,
        contractility,
        traction,
    );
    let mut mesh = settled.clone();
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(SEED_SET[0])).unwrap();
    for _ in 0..ASSAY_STEPS / 2 {
        apply_intrinsic_exploration_with_stick_slip(
            &mut mesh,
            &mut state,
            mechanics,
            contractility,
            traction,
        )
        .unwrap();
    }
    let mid_mesh_hash = stable_json_hash(&mesh).unwrap();
    let mid_state_hash = stable_json_hash(&state).unwrap();
    let serialized = serde_json::to_vec(&(mesh, state)).unwrap();
    let (mut resumed_mesh, mut resumed_state): (MaterialMesh, IntrinsicExplorationStateV1) =
        serde_json::from_slice(&serialized).unwrap();
    let restored_mid_mesh_hash = stable_json_hash(&resumed_mesh).unwrap();
    let restored_mid_state_hash = stable_json_hash(&resumed_state).unwrap();
    for _ in ASSAY_STEPS / 2..ASSAY_STEPS {
        apply_intrinsic_exploration_with_stick_slip(
            &mut resumed_mesh,
            &mut resumed_state,
            mechanics,
            contractility,
            traction,
        )
        .unwrap();
    }
    let uninterrupted_state_hash = uninterrupted.final_state_hash.unwrap();
    let resumed_state_hash = stable_json_hash(&resumed_state).unwrap();
    json!({
        "serialized_at_step": ASSAY_STEPS / 2,
        "no_new_seed_injected": true,
        "mid_mesh_hash": mid_mesh_hash,
        "restored_mid_mesh_hash": restored_mid_mesh_hash,
        "mid_state_hash": mid_state_hash,
        "restored_mid_state_hash": restored_mid_state_hash,
        "uninterrupted_mesh_hash": uninterrupted.final_mesh_hash,
        "resumed_mesh_hash": stable_json_hash(&resumed_mesh).unwrap(),
        "uninterrupted_state_hash": uninterrupted_state_hash,
        "resumed_state_hash": resumed_state_hash,
        "pass": uninterrupted.final_mesh_hash == stable_json_hash(&resumed_mesh).unwrap()
            && uninterrupted_state_hash == stable_json_hash(&resumed_state).unwrap(),
    })
}

fn source_hash(relative: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    stable_json_hash(&fs::read(root.join(relative)).unwrap()).unwrap()
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let output = arguments
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry003"));
    let dense_output = arguments.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    assert!((mechanics.dt - FROZEN_DT).abs() <= 1e-12);
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settled = settle_body(&mechanics);

    let intrinsic = run_arm(
        &settled,
        Arm::IntrinsicExploration,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let current = run_arm(
        &settled,
        Arm::CurrentRegulatorControl,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let seed_only = run_arm(
        &settled,
        Arm::SeedOnlyControl,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let zero_a = run_arm(
        &settled,
        Arm::ZeroA,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let no_substrate = run_arm(
        &settled,
        Arm::NoSubstrate,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let rotated = run_arm(
        &rotate_180(settled.clone()),
        Arm::IntrinsicExploration,
        SEED_SET[0],
        &mechanics,
        &contractility,
        &traction,
    );
    let seed_diversity: Vec<ArmResult> = SEED_SET
        .iter()
        .copied()
        .map(|seed| {
            run_arm(
                &settled,
                Arm::IntrinsicExploration,
                seed,
                &mechanics,
                &contractility,
                &traction,
            )
        })
        .collect();
    let restart = restart_check(&settled, &mechanics, &contractility, &traction);

    let active_path_benefit = intrinsic.path_length
        > current.path_length + FROZEN_ZERO_MOTION_TOLERANCE
        && intrinsic.path_length > seed_only.path_length + FROZEN_ZERO_MOTION_TOLERANCE;
    let dynamics_pass = intrinsic.maximum_activity > 0.0 && intrinsic.dominant_patch_changes > 0;
    let energetic_pass = intrinsic.a_spent > 0.0
        && intrinsic.w_generated > 0.0
        && intrinsic.a_to_w_residual <= 1e-8
        && intrinsic.reserve_before == intrinsic.reserve_after;
    let zero_a_pass = zero_a.maximum_active_tension == 0.0
        && zero_a.a_spent == 0.0
        && zero_a.w_generated == 0.0
        && zero_a.path_length <= FROZEN_ZERO_MOTION_TOLERANCE;
    let no_substrate_pass = no_substrate.net_displacement <= FROZEN_ZERO_MOTION_TOLERANCE;
    let rotation_pass = intrinsic.final_state_hash == rotated.final_state_hash
        && (intrinsic.a_spent - rotated.a_spent).abs() <= 1e-9
        && (intrinsic.path_length - rotated.path_length).abs() <= 1e-9
        && norm([
            intrinsic.final_material_centroid[0] + rotated.final_material_centroid[0],
            intrinsic.final_material_centroid[1] + rotated.final_material_centroid[1],
        ]) <= 1e-8;
    let seed_diversity_pass = seed_diversity
        .iter()
        .map(|arm| arm.dominant_patch_trace[0])
        .collect::<Vec<_>>()
        == vec![1, 2, 3, 4];
    let centroid_agreement = (intrinsic.net_displacement - intrinsic.vertex_net_displacement).abs()
        <= FROZEN_ZERO_MOTION_TOLERANCE;
    let classification = if !energetic_pass {
        "M2_INTRINSIC_EXPLORATION_ENERGETIC_INVALID"
    } else if !dynamics_pass {
        "M2_INTRINSIC_EXPLORATION_DYNAMICS_INSUFFICIENT"
    } else if !active_path_benefit || !no_substrate_pass || !centroid_agreement {
        "M2_INTRINSIC_EXPLORATION_MECHANICALLY_INSUFFICIENT"
    } else if !zero_a_pass
        || !rotation_pass
        || !seed_diversity_pass
        || !restart["pass"].as_bool().unwrap()
    {
        "M2_INTRINSIC_EXPLORATION_PRESERVATION_REGRESSION"
    } else {
        "M2_INTRINSIC_EXPLORATION_FEASIBILITY_QUALIFIED"
    };

    let protocol = json!({
        "directive": DIRECTIVE,
        "entry_head": ENTRY_HEAD,
        "primary_assay_resource_patch": false,
        "topology_size": TOPOLOGY_SIZE,
        "settlement_steps": SETTLEMENT_STEPS,
        "t_load": T_LOAD,
        "t_recovery": T_RECOVERY,
        "assay_steps": ASSAY_STEPS,
        "dt": FROZEN_DT,
        "seed_set": SEED_SET,
    });
    let architecture = json!({
        "schema": INTRINSIC_EXPLORATION_REGULATOR_SCHEMA_V1,
        "opt_in": true,
        "reads": ["current local activity", "immediate neighbors", "existing local adaptation", "step index", "provenance seed", "accepted dt"],
        "does_not_read": ["world position", "resource position", "resource concentration", "contact signal", "resource inventory", "viability", "health", "observer state", "target", "gradient"],
        "coordinate_authority": "existing chemistry-core mechanics only",
        "direct_coordinate_writes": false,
    });
    let parameter_reuse = json!({
        "new_numeric_parameter": false,
        "self_excitation": "FROZEN_K_STIMULUS * a_i * (1-a_i) * (1-adaptation_i)",
        "neighbor": "FROZEN_K_NEIGHBOR * (neighbor_mean-a_i)",
        "decay": "FROZEN_K_DECAY * a_i",
        "adaptation_load": FROZEN_ADAPTATION_LOAD_RATE_PER_TIME,
        "adaptation_recovery": FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME,
        "initial_perturbation": FROZEN_K_STIMULUS * FROZEN_DT,
    });
    write_json(&output, "protocol.json", &protocol);
    write_json(&output, "architecture.json", &architecture);
    write_json(&output, "parameter_reuse.json", &parameter_reuse);
    write_json(
        &output,
        "initial_symmetry_breaking.json",
        &json!({
            "one_time": true,
            "material_local": true,
            "patch_rule": "provenance_seed modulo topology_size",
            "magnitude": FROZEN_K_STIMULUS * FROZEN_DT,
            "recurring_random_motor_command": false,
        }),
    );
    write_json(
        &output,
        "intrinsic_exploration.json",
        &serde_json::to_value(&intrinsic).unwrap(),
    );
    write_json(
        &output,
        "current_regulator_control.json",
        &serde_json::to_value(&current).unwrap(),
    );
    write_json(
        &output,
        "seed_only_control.json",
        &serde_json::to_value(&seed_only).unwrap(),
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
        "seed_diversity.json",
        &serde_json::to_value(&seed_diversity).unwrap(),
    );
    write_json(
        &output,
        "rotation_check.json",
        &json!({"pass": rotation_pass, "unrotated": intrinsic, "rotated": rotated}),
    );
    write_json(&output, "restart_check.json", &restart);
    write_json(
        &output,
        "material_closure.json",
        &json!({"a_to_w_closure": energetic_pass, "residual": intrinsic.a_to_w_residual, "r_unchanged": intrinsic.reserve_before == intrinsic.reserve_after}),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({
            "scientific_source_changed": false,
            "frozen_m1_modules_modified": false,
            "production_behavior_changed_when_unselected": false,
            "exact_head_workflow_required": true,
        }),
    );
    write_json(
        &output,
        "entry001_preservation.json",
        &json!({
            "actuator_schema": "ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1",
            "actuator_source_modified": false,
            "exact_head_workflow_required": true,
        }),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({
            "status": "PENDING_EXACT_HEAD_WORKFLOW",
            "historical_classifications_changed": false,
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification": classification,
            "intrinsic_exploration_feasible": classification == "M2_INTRINSIC_EXPLORATION_FEASIBILITY_QUALIFIED",
            "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "dynamics_pass": dynamics_pass,
            "active_path_benefit": active_path_benefit,
            "energetic_pass": energetic_pass,
            "zero_a_pass": zero_a_pass,
            "no_substrate_pass": no_substrate_pass,
            "centroid_agreement": centroid_agreement,
            "rotation_pass": rotation_pass,
            "seed_diversity_pass": seed_diversity_pass,
            "restart_pass": restart["pass"],
            "resource_signal_read": false,
            "target_gradient_planner": "NONE",
            "new_numeric_parameter": false,
            "recurring_random_motor_command": false,
        }),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({
            "directive": DIRECTIVE,
            "files": ["protocol.json", "architecture.json", "parameter_reuse.json", "initial_symmetry_breaking.json", "intrinsic_exploration.json", "current_regulator_control.json", "seed_only_control.json", "zero_a_control.json", "no_substrate_control.json", "seed_diversity.json", "rotation_check.json", "restart_check.json", "material_closure.json", "m1_preservation.json", "entry001_preservation.json", "downstream_preservation.json", "qualification.json"],
            "source_hashes": {
                "regulator": source_hash("lib.rs"),
                "plasticity": source_hash("plasticity.rs"),
                "contractility": source_hash("contractility.rs"),
                "traction": source_hash("stick_slip_traction.rs"),
                "intrinsic_exploration": source_hash("intrinsic_exploration.rs"),
            },
        }),
    );
    if let Some(dense) = dense_output {
        write_json(
            &dense,
            "dense_trajectories.json",
            &json!({
                "intrinsic": intrinsic,
                "current_regulator": current,
                "seed_only": seed_only,
                "zero_a": zero_a,
                "no_substrate": no_substrate,
                "seed_diversity": seed_diversity,
            }),
        );
    }
    println!("{classification}");
}
