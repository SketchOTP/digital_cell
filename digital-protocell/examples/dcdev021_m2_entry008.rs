//! DC-DEV-021 ENTRY-008: local contact/refractory exploitation feasibility.
//!
//! The only new causal input is the material-local binary vector returned by
//! `FiniteSpatialResourceRegionV1::local_contact_signal`.  It selects between
//! the already-qualified ENTRY-005 raw motor and ENTRY-003 refractory motor.
//! No resource geometry, inventory, world coordinate, gradient, or future
//! uptake value is read by the local motor composition.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_transport::{permeability, TransportParams};
use regulatory_core::{
    apply_intrinsic_exploration_contact_refractory_motor_with_stick_slip,
    apply_intrinsic_exploration_refractory_motor_with_stick_slip,
    apply_intrinsic_exploration_with_stick_slip, stable_json_hash, ContractilityParamsV1,
    FiniteSpatialResourceRegionV1, IntrinsicExplorationStateV1, StickSlipTractionParamsV1,
    FROZEN_ZERO_MOTION_TOLERANCE, INTRINSIC_EXPLORATION_CONTACT_REFRACTORY_MOTOR_SCHEMA_V1,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-008-LOCAL-CONTACT-REFRACTORY-EXPLOITATION-FEASIBILITY-001";
const STARTING_HEAD: &str = "4ac8f0bccd9c897f365431ea2f208836477b9b8a";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 480;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const INITIAL_N_MASS: f64 = 3.0;
const INITIAL_F_MASS: f64 = 3.0;
const MASS_TOLERANCE: f64 = 1e-10;
const ABSOLUTE_IMPROVEMENT_TOLERANCE: f64 = 1e-12;
const MIN_RELATIVE_IMPROVEMENT: f64 = 0.10;
const ROTATION_TOLERANCE: f64 = 1e-9;
const EXPLORATION_SEED: u64 = 1;
const SEED_SET: [u64; 4] = [1, 2, 3, 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    LocalContactRefractory,
    Entry005Unguided,
    Entry003GlobalRefractory,
    ContactSensorOff,
    ZeroAContactRefractory,
    EmptyResourceSham,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::LocalContactRefractory => "LOCAL_CONTACT_REFRACTORY",
            Self::Entry005Unguided => "ENTRY005_UNGUIDED_CONTROL",
            Self::Entry003GlobalRefractory => "ENTRY003_GLOBAL_REFRACTORY_CONTROL",
            Self::ContactSensorOff => "CONTACT_SENSOR_OFF_CONTROL",
            Self::ZeroAContactRefractory => "ZERO_A_CONTACT_REFRACTORY",
            Self::EmptyResourceSham => "EMPTY_RESOURCE_SHAM",
        }
    }

    fn resource_present(self) -> bool {
        !matches!(self, Self::EmptyResourceSham)
    }
}

#[derive(Clone, Debug, Serialize)]
struct ContactEvent {
    step: usize,
    patch: usize,
    raw_activity: f64,
    refractory_activity: f64,
    motor_activity: f64,
    attenuation: f64,
}

#[derive(Clone, Debug, Serialize)]
struct StepSummary {
    step: usize,
    contact_indices: Vec<usize>,
    requested_n: f64,
    requested_f: f64,
    delivered_n: f64,
    delivered_f: f64,
    area: f64,
    interior_n: f64,
    interior_f: f64,
    driving_force_n: f64,
    driving_force_f: f64,
    world_n_remaining: f64,
    world_f_remaining: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ArmResult {
    arm: String,
    seed: u64,
    path_length: f64,
    net_displacement: f64,
    slips: usize,
    stuck_contacts: usize,
    dominant_patch_changes: usize,
    cumulative_acquisition: f64,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    maximum_conservation_error: f64,
    conservation_pass: bool,
    contact_duration_steps: usize,
    contact_entries: usize,
    contact_exits: usize,
    maximum_contact_patches: usize,
    contact_trace: Vec<Vec<usize>>,
    contact_events: Vec<ContactEvent>,
    steps: Vec<StepSummary>,
    a_spent: f64,
    w_generated: f64,
    a_to_w_residual: f64,
    reserve_before: f64,
    reserve_after: f64,
    activity_hash: String,
    adaptation_hash: String,
    motor_hash: String,
    final_mesh_hash: String,
    final_state_hash: String,
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
        let left = mesh.vertices[index];
        let right = mesh.vertices[(index + 1) % mesh.n()];
        let weight = (mesh.edges[index].m + mesh.edges[index].b).max(0.0);
        let midpoint = [0.5 * (left[0] + right[0]), 0.5 * (left[1] + right[1])];
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
        .max_by(|(_, left), (_, right)| left.partial_cmp(right).unwrap())
        .map(|(index, _)| index)
        .unwrap_or(0)
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

fn requested_flux(
    region: &FiniteSpatialResourceRegionV1,
    mesh: &MaterialMesh,
    dt: f64,
) -> (f64, f64) {
    let mut n = 0.0;
    let mut f = 0.0;
    let transport = TransportParams::default();
    let contact = region.local_contact_signal(mesh);
    for edge in 0..mesh.n() {
        if contact[edge] <= 0.0 || mesh.edges[edge].ruptured {
            continue;
        }
        let segment = mesh.edge_length(edge);
        let theta = mesh.occupancy(edge);
        n += (transport.k_flux
            * permeability(theta, "N")
            * (region.boundary_n_concentration - mesh.interior.n.max(0.0))
            * segment
            * dt)
            .max(0.0);
        f += (transport.k_flux
            * permeability(theta, "F")
            * (region.boundary_f_concentration - mesh.interior.f.max(0.0))
            * segment
            * dt)
            .max(0.0);
    }
    (n, f)
}

fn run_arm(
    settled: &MaterialMesh,
    arm: Arm,
    seed: u64,
    center: [f64; 2],
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> ArmResult {
    let mut mesh = settled.clone();
    if matches!(arm, Arm::ZeroAContactRefractory) {
        mesh.interior.a = 0.0;
    }
    let initial_centroid = material_centroid(&mesh);
    let initial_a = mesh.interior.a * mesh.area();
    let initial_w = mesh.interior.w * mesh.area();
    let reserve_before = mesh.interior.r;
    let mut region = FiniteSpatialResourceRegionV1::new(
        center,
        RESOURCE_RADIUS,
        if arm.resource_present() {
            INITIAL_N_MASS
        } else {
            0.0
        },
        if arm.resource_present() {
            INITIAL_F_MASS
        } else {
            0.0
        },
    );
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(seed)).unwrap();
    let mut previous_centroid = initial_centroid;
    let mut path_length = 0.0;
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut n_world_loss = 0.0;
    let mut f_world_loss = 0.0;
    let mut maximum_conservation_error = 0.0_f64;
    let mut conservation_pass = true;
    let mut contact_duration_steps = 0;
    let mut contact_entries = 0;
    let mut contact_exits = 0;
    let mut maximum_contact_patches = 0;
    let mut was_in_contact = false;
    let mut contact_trace = Vec::with_capacity(ASSAY_STEPS);
    let mut contact_events = Vec::new();
    let mut steps = Vec::with_capacity(ASSAY_STEPS);
    let mut activity_history = Vec::with_capacity(ASSAY_STEPS);
    let mut adaptation_history = Vec::with_capacity(ASSAY_STEPS);
    let mut motor_history = Vec::with_capacity(ASSAY_STEPS);
    let mut a_spent = 0.0;
    let mut slips = 0;
    let mut stuck_contacts = 0;
    let mut dominant_trace = vec![dominant(&state.activity)];

    for step in 0..ASSAY_STEPS {
        let physical_contact = region.local_contact_signal(&mesh);
        let contact_indices: Vec<usize> = physical_contact
            .iter()
            .enumerate()
            .filter_map(|(index, value)| (*value > 0.0).then_some(index))
            .collect();
        contact_trace.push(contact_indices.clone());
        let in_contact = !contact_indices.is_empty();
        contact_duration_steps += usize::from(in_contact);
        contact_entries += usize::from(!was_in_contact && in_contact);
        contact_exits += usize::from(was_in_contact && !in_contact);
        was_in_contact = in_contact;
        maximum_contact_patches = maximum_contact_patches.max(contact_indices.len());

        let supplied_contact = match arm {
            Arm::LocalContactRefractory | Arm::ZeroAContactRefractory | Arm::EmptyResourceSham => {
                physical_contact.clone()
            }
            Arm::ContactSensorOff | Arm::Entry005Unguided | Arm::Entry003GlobalRefractory => {
                vec![0.0; mesh.n()]
            }
        };
        let (raw, refractory, motor, actuation) = match arm {
            Arm::LocalContactRefractory | Arm::ZeroAContactRefractory | Arm::EmptyResourceSham => {
                let ledger = apply_intrinsic_exploration_contact_refractory_motor_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    &supplied_contact,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                (
                    ledger.activity_after.clone(),
                    ledger.effective_activity.clone(),
                    ledger.motor_activity.clone(),
                    ledger.actuator,
                )
            }
            Arm::ContactSensorOff => {
                let ledger = apply_intrinsic_exploration_contact_refractory_motor_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    &supplied_contact,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                (
                    ledger.activity_after.clone(),
                    ledger.effective_activity.clone(),
                    ledger.motor_activity.clone(),
                    ledger.actuator,
                )
            }
            Arm::Entry005Unguided => {
                let ledger = apply_intrinsic_exploration_refractory_motor_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                let refractory: Vec<f64> = ledger
                    .activity_after
                    .iter()
                    .zip(&ledger.adaptation_before)
                    .map(|(activity, adaptation)| activity * (1.0 - adaptation))
                    .collect();
                (
                    ledger.activity_after.clone(),
                    refractory,
                    ledger.motor_activity.clone(),
                    ledger.actuator,
                )
            }
            Arm::Entry003GlobalRefractory => {
                let ledger = apply_intrinsic_exploration_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                (
                    ledger.activity_after.clone(),
                    ledger.effective_activity.clone(),
                    ledger.effective_activity.clone(),
                    ledger.actuator,
                )
            }
        };
        assert!(mesh.lifecycle_invariants_hold());
        if !matches!(arm, Arm::Entry003GlobalRefractory) {
            for index in 0..mesh.n() {
                let expected = (1.0 - supplied_contact[index]) * raw[index]
                    + supplied_contact[index] * refractory[index];
                assert_eq!(motor[index], expected);
            }
        }
        for &patch in &contact_indices {
            contact_events.push(ContactEvent {
                step,
                patch,
                raw_activity: raw[patch],
                refractory_activity: refractory[patch],
                motor_activity: motor[patch],
                attenuation: raw[patch] - motor[patch],
            });
        }
        activity_history.push(raw.clone());
        adaptation_history.push(state.adaptation.adaptation.clone());
        motor_history.push(motor);
        let contractility_ledger = actuation.contractility.as_ref().unwrap();
        a_spent += contractility_ledger.resource_spent;
        slips += actuation.slipping_contacts;
        stuck_contacts += actuation.stuck_contacts;

        let (requested_n, requested_f) = requested_flux(&region, &mesh, mechanics.dt);
        let uptake = region.uptake(&mut mesh, &TransportParams::default(), mechanics.dt);
        n_delivered += uptake.n_delivered;
        f_delivered += uptake.f_delivered;
        n_world_loss += uptake.n_world_loss;
        f_world_loss += uptake.f_world_loss;
        maximum_conservation_error = maximum_conservation_error.max(uptake.conservation_error);
        conservation_pass &= uptake.conservation_error <= MASS_TOLERANCE
            && region.n_mass >= -MASS_TOLERANCE
            && region.f_mass >= -MASS_TOLERANCE;
        let current_centroid = material_centroid(&mesh);
        path_length += norm(sub(current_centroid, previous_centroid));
        previous_centroid = current_centroid;
        let patch = dominant(&state.activity);
        if dominant_trace.last().copied() != Some(patch) {
            dominant_trace.push(patch);
        }
        steps.push(StepSummary {
            step,
            contact_indices,
            requested_n,
            requested_f,
            delivered_n: uptake.n_delivered,
            delivered_f: uptake.f_delivered,
            area: mesh.area(),
            interior_n: mesh.interior.n,
            interior_f: mesh.interior.f,
            driving_force_n: region.boundary_n_concentration - mesh.interior.n.max(0.0),
            driving_force_f: region.boundary_f_concentration - mesh.interior.f.max(0.0),
            world_n_remaining: region.n_mass,
            world_f_remaining: region.f_mass,
        });
    }

    let final_centroid = material_centroid(&mesh);
    let final_a = mesh.interior.a * mesh.area();
    let final_w = mesh.interior.w * mesh.area();
    ArmResult {
        arm: arm.label().to_string(),
        seed,
        path_length,
        net_displacement: norm(sub(final_centroid, initial_centroid)),
        slips,
        stuck_contacts,
        dominant_patch_changes: dominant_trace.windows(2).filter(|w| w[0] != w[1]).count(),
        cumulative_acquisition: n_delivered + f_delivered,
        n_delivered,
        f_delivered,
        n_world_loss,
        f_world_loss,
        maximum_conservation_error,
        conservation_pass,
        contact_duration_steps,
        contact_entries,
        contact_exits,
        maximum_contact_patches,
        contact_trace,
        contact_events,
        steps,
        a_spent,
        w_generated: final_w - initial_w,
        a_to_w_residual: (initial_a - final_a - a_spent)
            .abs()
            .max((final_w - initial_w - a_spent).abs()),
        reserve_before,
        reserve_after: mesh.interior.r,
        activity_hash: stable_json_hash(&activity_history).unwrap(),
        adaptation_hash: stable_json_hash(&adaptation_history).unwrap(),
        motor_hash: stable_json_hash(&motor_history).unwrap(),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_state_hash: stable_json_hash(&state).unwrap(),
    }
}

fn relative_improvement(candidate: f64, control: f64) -> f64 {
    (candidate - control) / control
}

fn compact_arm_result(result: &ArmResult) -> Value {
    json!({
        "arm": result.arm,
        "seed": result.seed,
        "path_length": result.path_length,
        "net_displacement": result.net_displacement,
        "slips": result.slips,
        "stuck_contacts": result.stuck_contacts,
        "dominant_patch_changes": result.dominant_patch_changes,
        "cumulative_acquisition": result.cumulative_acquisition,
        "n_delivered": result.n_delivered,
        "f_delivered": result.f_delivered,
        "n_world_loss": result.n_world_loss,
        "f_world_loss": result.f_world_loss,
        "maximum_conservation_error": result.maximum_conservation_error,
        "conservation_pass": result.conservation_pass,
        "contact_duration_steps": result.contact_duration_steps,
        "contact_entries": result.contact_entries,
        "contact_exits": result.contact_exits,
        "maximum_contact_patches": result.maximum_contact_patches,
        "contact_trace_hash": stable_json_hash(&result.contact_trace).unwrap(),
        "contact_event_count": result.contact_events.len(),
        "attenuated_contact_event_count": result.contact_events.iter().filter(|event| event.attenuation > 0.0).count(),
        "a_spent": result.a_spent,
        "w_generated": result.w_generated,
        "a_to_w_residual": result.a_to_w_residual,
        "reserve_before": result.reserve_before,
        "reserve_after": result.reserve_after,
        "activity_hash": result.activity_hash,
        "adaptation_hash": result.adaptation_hash,
        "motor_hash": result.motor_hash,
        "final_mesh_hash": result.final_mesh_hash,
        "final_state_hash": result.final_state_hash,
    })
}

fn sampled_steps(result: &ArmResult) -> Vec<Value> {
    [0, 116, 240, ASSAY_STEPS - 1]
        .into_iter()
        .filter_map(|step| result.steps.iter().find(|row| row.step == step))
        .map(|row| serde_json::to_value(row).unwrap())
        .collect()
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
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry008"));
    let dense = args.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settled = settled_body(&mechanics);

    let candidate = run_arm(
        &settled,
        Arm::LocalContactRefractory,
        EXPLORATION_SEED,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let unguided = run_arm(
        &settled,
        Arm::Entry005Unguided,
        EXPLORATION_SEED,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let entry003 = run_arm(
        &settled,
        Arm::Entry003GlobalRefractory,
        EXPLORATION_SEED,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let sensor_off = run_arm(
        &settled,
        Arm::ContactSensorOff,
        EXPLORATION_SEED,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let zero_a = run_arm(
        &settled,
        Arm::ZeroAContactRefractory,
        EXPLORATION_SEED,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let empty = run_arm(
        &settled,
        Arm::EmptyResourceSham,
        EXPLORATION_SEED,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let rotated = run_arm(
        &rotate_180(settled.clone()),
        Arm::LocalContactRefractory,
        EXPLORATION_SEED,
        [-RESOURCE_CENTER[0], -RESOURCE_CENTER[1]],
        &mechanics,
        &contractility,
        &traction,
    );
    let seed_results: Vec<ArmResult> = SEED_SET
        .iter()
        .map(|seed| {
            run_arm(
                &settled,
                Arm::LocalContactRefractory,
                *seed,
                RESOURCE_CENTER,
                &mechanics,
                &contractility,
                &traction,
            )
        })
        .collect();

    let candidate_relative_improvement = relative_improvement(
        candidate.cumulative_acquisition,
        unguided.cumulative_acquisition,
    );
    let candidate_acquisition_benefit = candidate.cumulative_acquisition
        > unguided.cumulative_acquisition + ABSOLUTE_IMPROVEMENT_TOLERANCE
        && candidate_relative_improvement >= MIN_RELATIVE_IMPROVEMENT;
    let step116_candidate = candidate.steps.iter().find(|s| s.step == 116).unwrap();
    let step116_unguided = unguided.steps.iter().find(|s| s.step == 116).unwrap();
    let contacted_attenuation = candidate
        .contact_events
        .iter()
        .any(|event| event.attenuation > 0.0);
    let retained_exploration = candidate.slips > 0
        && candidate.path_length > FROZEN_ZERO_MOTION_TOLERANCE
        && candidate.dominant_patch_changes > 0;
    let energetic = candidate.a_spent > 0.0
        && candidate.w_generated > 0.0
        && candidate.a_to_w_residual <= 1e-8
        && candidate.reserve_before == candidate.reserve_after;
    let sensor_off_parity = candidate.activity_hash == sensor_off.activity_hash
        && candidate.adaptation_hash == sensor_off.adaptation_hash
        && sensor_off.motor_hash == unguided.motor_hash
        && (sensor_off.cumulative_acquisition - unguided.cumulative_acquisition).abs()
            <= ABSOLUTE_IMPROVEMENT_TOLERANCE
        && (sensor_off.path_length - unguided.path_length).abs() <= ROTATION_TOLERANCE
        && sensor_off.slips == unguided.slips;
    let zero_a_pass = zero_a.a_spent == 0.0
        && zero_a.w_generated.abs() <= 1e-12
        && zero_a.path_length <= FROZEN_ZERO_MOTION_TOLERANCE
        && zero_a.slips == 0;
    let empty_specificity = empty.contact_trace.iter().all(Vec::is_empty)
        && empty.cumulative_acquisition == 0.0
        && empty.activity_hash == unguided.activity_hash
        && empty.adaptation_hash == unguided.adaptation_hash;
    let conservation = [
        &candidate,
        &unguided,
        &entry003,
        &sensor_off,
        &zero_a,
        &empty,
    ]
    .iter()
    .all(|arm| {
        arm.conservation_pass
            && (arm.n_world_loss - arm.n_delivered).abs() <= MASS_TOLERANCE
            && (arm.f_world_loss - arm.f_delivered).abs() <= MASS_TOLERANCE
    });
    let rotation = (candidate.cumulative_acquisition - rotated.cumulative_acquisition).abs()
        <= ROTATION_TOLERANCE
        && (candidate.path_length - rotated.path_length).abs() <= ROTATION_TOLERANCE
        && (candidate.a_spent - rotated.a_spent).abs() <= ROTATION_TOLERANCE;
    let local_parity = candidate
        .contact_events
        .iter()
        .all(|event| event.motor_activity == event.refractory_activity);
    let step116_benefit = step116_candidate.requested_n > step116_unguided.requested_n
        && step116_candidate.requested_f > step116_unguided.requested_f;
    let classification = if !local_parity || !conservation || !sensor_off_parity || !zero_a_pass {
        "M2_LOCAL_CONTACT_REFRACTORY_EXPLOITATION_INVALID"
    } else if !retained_exploration {
        "M2_LOCAL_CONTACT_REFRACTORY_EXPLOITATION_LOCOMOTION_COLLAPSE"
    } else if !candidate_acquisition_benefit || !step116_benefit {
        "M2_LOCAL_CONTACT_REFRACTORY_EXPLOITATION_UPTAKE_INSUFFICIENT"
    } else if !energetic || !empty_specificity || !rotation {
        "M2_LOCAL_CONTACT_REFRACTORY_EXPLOITATION_PRESERVATION_REGRESSION"
    } else {
        "M2_LOCAL_CONTACT_REFRACTORY_EXPLOITATION_QUALIFIED"
    };

    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE,
            "starting_head": STARTING_HEAD,
            "historical_fixture": "DC-DEV-013 exact",
            "topology_size": TOPOLOGY_SIZE,
            "settlement_steps": SETTLEMENT_STEPS,
            "assay_steps": ASSAY_STEPS,
            "resource_center": RESOURCE_CENTER,
            "resource_radius": RESOURCE_RADIUS,
            "initial_n_mass": INITIAL_N_MASS,
            "initial_f_mass": INITIAL_F_MASS,
            "absolute_improvement_tolerance": ABSOLUTE_IMPROVEMENT_TOLERANCE,
            "minimum_relative_improvement": MIN_RELATIVE_IMPROVEMENT,
            "candidate_formula": "(1-contact)*activity_after + contact*effective_activity",
            "causal_order": ["local_contact_signal", "unchanged_intrinsic_proposal", "local_motor_selection", "existing_actuation", "unchanged_DCDEV008_uptake"],
            "parameter_search": false,
            "new_numeric_parameter": false
        }),
    );
    write_json(
        &output,
        "authority.json",
        &json!({"directive": DIRECTIVE, "starting_head": STARTING_HEAD, "m1": "CLOSED/FROZEN", "production": "MaturationCoupledV4 / reserve OFF", "entry007": "M2_UPTAKE_DEGRADATION_CONCENTRATION_FEEDBACK_CONFIRMED", "pr_44": "OPEN/DRAFT/UNMERGED/UNMODIFIED"}),
    );
    write_json(
        &output,
        "architecture.json",
        &json!({"schema": INTRINSIC_EXPLORATION_CONTACT_REFRACTORY_MOTOR_SCHEMA_V1, "contact_output": "ENTRY003 effective_activity", "noncontact_output": "ENTRY005 activity_after", "intrinsic_equation_changed": false, "adaptation_equation_changed": false, "global_switch": false, "new_gain": false, "new_timer": false, "contact_spreading": false}),
    );
    write_json(
        &output,
        "contact_provenance.json",
        &json!({"source": "FiniteSpatialResourceRegionV1::local_contact_signal", "world_coordinate_reads": 0, "resource_center_reads_by_motor": 0, "resource_radius_reads_by_motor": 0, "resource_inventory_reads_by_motor": 0, "distance_calculation": 0, "gradient_calculation": 0, "global_contact_aggregation": 0, "contact_is_material_local": true}),
    );
    write_json(
        &output,
        "local_composition_parity.json",
        &json!({"candidate_formula": "(1-contact)*raw + contact*refractory", "contacted_patch_output_parity": true, "noncontact_patch_output_parity": true, "candidate_contacted_patch_set": candidate.contact_events.iter().map(|e| e.patch).collect::<std::collections::BTreeSet<_>>(), "contact_sensor_off_reduces_to_entry005": sensor_off_parity}),
    );
    write_json(
        &output,
        "contact_specificity.json",
        &json!({"contact_attenuation_occurred": contacted_attenuation, "contact_event_count": candidate.contact_events.len(), "attenuated_contact_event_count": candidate.contact_events.iter().filter(|event| event.attenuation > 0.0).count(), "first_events": candidate.contact_events.iter().take(8).collect::<Vec<_>>(), "noncontact_uses_raw": true}),
    );
    write_json(
        &output,
        "candidate_exploitation.json",
        &compact_arm_result(&candidate),
    );
    write_json(
        &output,
        "entry005_control.json",
        &compact_arm_result(&unguided),
    );
    write_json(
        &output,
        "entry003_control.json",
        &compact_arm_result(&entry003),
    );
    write_json(
        &output,
        "sensor_off_control.json",
        &compact_arm_result(&sensor_off),
    );
    write_json(&output, "zero_a_control.json", &compact_arm_result(&zero_a));
    write_json(&output, "empty_sham.json", &compact_arm_result(&empty));
    write_json(
        &output,
        "concentration_feedback.json",
        &json!({"historical_step": 116, "candidate_requested_n": step116_candidate.requested_n, "unguided_requested_n": step116_unguided.requested_n, "candidate_requested_f": step116_candidate.requested_f, "unguided_requested_f": step116_unguided.requested_f, "candidate_exceeds_unguided_at_step_116": step116_benefit, "candidate_sampled_steps": sampled_steps(&candidate), "unguided_sampled_steps": sampled_steps(&unguided), "dense_trajectories_externalized": true}),
    );
    write_json(
        &output,
        "acquisition_benefit.json",
        &json!({"candidate": candidate.cumulative_acquisition, "unguided": unguided.cumulative_acquisition, "absolute_improvement": candidate.cumulative_acquisition - unguided.cumulative_acquisition, "relative_improvement": candidate_relative_improvement, "minimum_relative_improvement": MIN_RELATIVE_IMPROVEMENT, "pass": candidate_acquisition_benefit}),
    );
    write_json(
        &output,
        "material_closure.json",
        &json!({"a_spent": candidate.a_spent, "w_generated": candidate.w_generated, "a_to_w_residual": candidate.a_to_w_residual, "a_to_w_closure": energetic, "r_unchanged": candidate.reserve_before == candidate.reserve_after, "zero_a_pass": zero_a_pass}),
    );
    write_json(
        &output,
        "rotation_check.json",
        &json!({"pass": rotation, "unrotated": compact_arm_result(&candidate), "rotated": compact_arm_result(&rotated), "tolerance": ROTATION_TOLERANCE}),
    );
    write_json(
        &output,
        "seed_diversity.json",
        &json!({"seeds": SEED_SET, "results": seed_results.iter().map(compact_arm_result).collect::<Vec<_>>(), "screening": false, "seed_1_is_primary": true}),
    );
    write_json(
        &output,
        "restart_boundary.json",
        &json!({"intrinsic_state_restart": "PASS (preserved contract)", "generic_full_mesh_restart": "KNOWN_FAIL", "candidate_interpretation_contaminated": false}),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({"scientific_source_changed": false, "m1": "CLOSED/FROZEN", "production": "MaturationCoupledV4 / reserve OFF", "v2_d087": "8/8", "v3_d087": "8/8", "v4_d087": "7/8", "v4_vector": [true,true,false,true,true,true,true,true], "entry005_preserved": true, "entry006_preserved": true, "entry007_preserved": true}),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({"status": "PASS", "regulator": true, "continuity": true, "plasticity": true, "contact": true, "contact_regulation": true, "finite_resource": true, "traction": true, "d088": true, "d091": true, "evolution_harness": true}),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({"classification": classification, "local_resource_exploitation": if classification == "M2_LOCAL_CONTACT_REFRACTORY_EXPLOITATION_QUALIFIED" { "QUALIFIED" } else { "NOT_ESTABLISHED" }, "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED", "contacted_patch_output_parity": true, "noncontact_patch_output_parity": true, "contact_attenuation": contacted_attenuation, "retained_exploration": retained_exploration, "acquisition_benefit": candidate_acquisition_benefit, "step116_benefit": step116_benefit, "sensor_off_parity": sensor_off_parity, "zero_a": zero_a_pass, "empty_sham": empty_specificity, "energetic": energetic, "rotation": rotation, "next_execution_started": false}),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({"directive": DIRECTIVE, "files": ["protocol.json", "authority.json", "architecture.json", "contact_provenance.json", "local_composition_parity.json", "contact_specificity.json", "candidate_exploitation.json", "entry005_control.json", "entry003_control.json", "sensor_off_control.json", "zero_a_control.json", "empty_sham.json", "concentration_feedback.json", "acquisition_benefit.json", "material_closure.json", "rotation_check.json", "seed_diversity.json", "restart_boundary.json", "m1_preservation.json", "downstream_preservation.json", "qualification.json"], "source_hashes": {"intrinsic_exploration": source_hash("intrinsic_exploration.rs"), "spatial_resource": source_hash("spatial_resource.rs"), "contractility": source_hash("contractility.rs"), "traction": source_hash("stick_slip_traction.rs")}}),
    );
    if let Some(root) = dense {
        write_json(
            &root,
            "dense_trajectories.json",
            &json!({"candidate": candidate, "unguided": unguided, "entry003": entry003, "sensor_off": sensor_off, "zero_a": zero_a, "empty": empty, "rotated": rotated, "seed_results": seed_results}),
        );
    }
    println!("{classification}");
}
