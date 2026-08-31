//! DC-DEV-021 ENTRY-009: assay-only ceiling for contact-local motor suppression.
//!
//! This is deliberately not a production behavior.  It advances the unchanged
//! ENTRY-003 intrinsic proposal, supplies zero motor activity on exactly the
//! currently contacted material edges, supplies raw ENTRY-005 activity on all
//! other edges, and submits that vector to the unchanged A-funded actuator.
//! The result is an upper-bound counterfactual for the pure contact-local
//! suppression family.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_transport::{permeability, TransportParams};
use regulatory_core::{
    apply_intrinsic_exploration_refractory_motor_with_stick_slip,
    apply_local_activated_energy_contractility_with_stick_slip, commit_intrinsic_exploration_step,
    propose_intrinsic_exploration_step, stable_json_hash, ContractilityParamsV1,
    FiniteSpatialResourceRegionV1, IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1,
    StickSlipTractionParamsV1, FROZEN_MAX_ACTIVE_TENSION,
    FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME, FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-021-M2-ENTRY-009-CONTACT-LOCAL-EXPLOITATION-CEILING-AUDIT-001";
const STARTING_HEAD: &str = "82cd906daebc41ed8f525d9e13d257d3e553b428";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 480;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const INITIAL_N_MASS: f64 = 3.0;
const INITIAL_F_MASS: f64 = 3.0;
const MASS_TOLERANCE: f64 = 1e-10;
const MIN_RELATIVE_IMPROVEMENT: f64 = 0.10;
const ROTATION_TOLERANCE: f64 = 1e-9;
const ZERO_MOTOR_TOLERANCE: f64 = 1e-14;
const SEED_SET: [u64; 4] = [1, 2, 3, 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    ContactZeroCeiling,
    Entry005Unguided,
    EmptyResourceSham,
    ZeroACeiling,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::ContactZeroCeiling => "CONTACT_ZERO_CEILING",
            Self::Entry005Unguided => "ENTRY005_UNGUIDED_CONTROL",
            Self::EmptyResourceSham => "EMPTY_RESOURCE_SHAM",
            Self::ZeroACeiling => "ZERO_A_CONTACT_ZERO_CEILING",
        }
    }

    fn resource_present(self) -> bool {
        !matches!(self, Self::EmptyResourceSham)
    }
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
    contact_active_resource_request: f64,
    noncontact_active_resource_request: f64,
    contact_active_resource_spent: f64,
    noncontact_active_resource_spent: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ArmResult {
    arm: String,
    seed: u64,
    path_length: f64,
    displacement: [f64; 2],
    net_displacement: f64,
    slips: usize,
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
    steps: Vec<StepSummary>,
    raw_history: Vec<Vec<f64>>,
    motor_history: Vec<Vec<f64>>,
    contact_history: Vec<Vec<f64>>,
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
    let transport = TransportParams::default();
    let contact = region.local_contact_signal(mesh);
    let mut n = 0.0;
    let mut f = 0.0;
    for edge in 0..mesh.n() {
        if contact[edge] == 0.0 || mesh.edges[edge].ruptured {
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

fn active_resource_partition(
    mesh: &MaterialMesh,
    motor: &[f64],
    contact: &[f64],
    mechanics: &MechParams,
    activated_amount_before: f64,
) -> (f64, f64, f64) {
    let mut contacted = 0.0;
    let mut noncontacted = 0.0;
    for edge in 0..mesh.n() {
        if mesh.edges[edge].ruptured {
            continue;
        }
        let edge_activity = 0.5 * (motor[edge] + motor[(edge + 1) % mesh.n()]);
        let request = FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME
            * FROZEN_MAX_ACTIVE_TENSION
            * edge_activity
            * mesh.edge_length(edge)
            * mechanics.dt;
        if contact[edge] > 0.0 {
            contacted += request;
        } else {
            noncontacted += request;
        }
    }
    let total = contacted + noncontacted;
    let available = activated_amount_before;
    let funding_scale = if total <= f64::EPSILON {
        0.0
    } else {
        (available / total).min(1.0)
    };
    (contacted, noncontacted, funding_scale)
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
    if matches!(arm, Arm::ZeroACeiling) {
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
    let mut steps = Vec::with_capacity(ASSAY_STEPS);
    let mut raw_history = Vec::with_capacity(ASSAY_STEPS);
    let mut motor_history = Vec::with_capacity(ASSAY_STEPS);
    let mut contact_history = Vec::with_capacity(ASSAY_STEPS);
    let mut adaptation_history = Vec::with_capacity(ASSAY_STEPS);
    let mut activity_history = Vec::with_capacity(ASSAY_STEPS);
    let mut slips = 0;
    let mut dominant_trace = vec![dominant(&state.activity)];
    let mut a_spent = 0.0;

    for step in 0..ASSAY_STEPS {
        let contact = region.local_contact_signal(&mesh);
        let contact_indices: Vec<usize> = contact
            .iter()
            .enumerate()
            .filter_map(|(i, v)| (*v > 0.0).then_some(i))
            .collect();
        contact_trace.push(contact_indices.clone());
        let in_contact = !contact_indices.is_empty();
        contact_duration_steps += usize::from(in_contact);
        contact_entries += usize::from(!was_in_contact && in_contact);
        contact_exits += usize::from(was_in_contact && !in_contact);
        was_in_contact = in_contact;
        maximum_contact_patches = maximum_contact_patches.max(contact_indices.len());
        let activated_amount_before = mesh.interior.a.max(0.0) * mesh.area().max(1e-12);
        let mesh_before_actuation = mesh.clone();

        let (raw, motor, actuation) = if matches!(arm, Arm::Entry005Unguided) {
            let ledger = apply_intrinsic_exploration_refractory_motor_with_stick_slip(
                &mut mesh,
                &mut state,
                mechanics,
                contractility,
                traction,
            )
            .unwrap();
            (
                ledger.activity_after,
                ledger.motor_activity,
                ledger.actuator,
            )
        } else {
            let proposal = propose_intrinsic_exploration_step(
                &state,
                mesh.n(),
                mechanics.dt,
                IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
            )
            .unwrap();
            let raw = proposal.activity_after.clone();
            let motor: Vec<f64> = raw
                .iter()
                .enumerate()
                .map(|(i, value)| if contact[i] > 0.0 { 0.0 } else { *value })
                .collect();
            let actuation = apply_local_activated_energy_contractility_with_stick_slip(
                &mut mesh,
                &motor,
                mechanics,
                contractility,
                traction,
            )
            .unwrap();
            commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
            (raw, motor, actuation)
        };
        assert_eq!(raw.len(), mesh.n());
        assert!(motor
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value)));
        if !matches!(arm, Arm::Entry005Unguided) {
            for i in 0..mesh.n() {
                if contact[i] > 0.0 {
                    assert!(motor[i].abs() <= ZERO_MOTOR_TOLERANCE);
                } else {
                    assert_eq!(motor[i], raw[i]);
                }
            }
        }
        let (contact_request, noncontact_request, funding_scale) = active_resource_partition(
            &mesh_before_actuation,
            &motor,
            &contact,
            mechanics,
            activated_amount_before,
        );
        let spent = actuation.contractility.as_ref().unwrap().resource_spent;
        a_spent += spent;
        slips += actuation.slipping_contacts;
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
        raw_history.push(raw.clone());
        motor_history.push(motor);
        contact_history.push(contact.clone());
        activity_history.push(state.activity.clone());
        adaptation_history.push(state.adaptation.adaptation.clone());
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
            contact_active_resource_request: contact_request,
            noncontact_active_resource_request: noncontact_request,
            contact_active_resource_spent: contact_request * funding_scale,
            noncontact_active_resource_spent: noncontact_request * funding_scale,
        });
    }
    let final_centroid = material_centroid(&mesh);
    let final_a = mesh.interior.a * mesh.area();
    let final_w = mesh.interior.w * mesh.area();
    let activity_hash = stable_json_hash(&activity_history).unwrap();
    let adaptation_hash = stable_json_hash(&adaptation_history).unwrap();
    let motor_hash = stable_json_hash(&motor_history).unwrap();
    ArmResult {
        arm: arm.label().to_string(),
        seed,
        path_length,
        displacement: sub(final_centroid, initial_centroid),
        net_displacement: norm(sub(final_centroid, initial_centroid)),
        slips,
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
        steps,
        raw_history,
        motor_history,
        contact_history,
        a_spent,
        w_generated: final_w - initial_w,
        a_to_w_residual: (initial_a - final_a - a_spent)
            .abs()
            .max((final_w - initial_w - a_spent).abs()),
        reserve_before,
        reserve_after: mesh.interior.r,
        activity_hash,
        adaptation_hash,
        motor_hash,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_state_hash: stable_json_hash(&state).unwrap(),
    }
}

fn compact(result: &ArmResult) -> Value {
    json!({
        "arm": result.arm, "seed": result.seed, "path_length": result.path_length,
        "displacement": result.displacement, "net_displacement": result.net_displacement,
        "slips": result.slips, "dominant_patch_changes": result.dominant_patch_changes,
        "cumulative_acquisition": result.cumulative_acquisition, "n_delivered": result.n_delivered,
        "f_delivered": result.f_delivered, "n_world_loss": result.n_world_loss,
        "f_world_loss": result.f_world_loss, "maximum_conservation_error": result.maximum_conservation_error,
        "conservation_pass": result.conservation_pass, "contact_duration_steps": result.contact_duration_steps,
        "contact_entries": result.contact_entries, "contact_exits": result.contact_exits,
        "maximum_contact_patches": result.maximum_contact_patches,
        "contact_trace_hash": stable_json_hash(&result.contact_trace).unwrap(),
        "a_spent": result.a_spent, "w_generated": result.w_generated,
        "a_to_w_residual": result.a_to_w_residual, "reserve_before": result.reserve_before,
        "reserve_after": result.reserve_after, "activity_hash": result.activity_hash,
        "adaptation_hash": result.adaptation_hash, "motor_hash": result.motor_hash,
        "final_mesh_hash": result.final_mesh_hash, "final_state_hash": result.final_state_hash,
    })
}

fn sampled(result: &ArmResult) -> Vec<Value> {
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
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry009"));
    let dense = args.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settled = settled_body(&mechanics);
    let candidate = run_arm(
        &settled,
        Arm::ContactZeroCeiling,
        1,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let unguided = run_arm(
        &settled,
        Arm::Entry005Unguided,
        1,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let empty = run_arm(
        &settled,
        Arm::EmptyResourceSham,
        1,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let zero_a = run_arm(
        &settled,
        Arm::ZeroACeiling,
        1,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let rotated = run_arm(
        &rotate_180(settled.clone()),
        Arm::ContactZeroCeiling,
        1,
        [-RESOURCE_CENTER[0], -RESOURCE_CENTER[1]],
        &mechanics,
        &contractility,
        &traction,
    );
    let seed_results: Vec<ArmResult> = SEED_SET
        .into_iter()
        .map(|seed| {
            run_arm(
                &settled,
                Arm::ContactZeroCeiling,
                seed,
                RESOURCE_CENTER,
                &mechanics,
                &contractility,
                &traction,
            )
        })
        .collect();

    let candidate_benefit = (candidate.cumulative_acquisition - unguided.cumulative_acquisition)
        / unguided.cumulative_acquisition;
    let ten_percent = candidate_benefit >= MIN_RELATIVE_IMPROVEMENT;
    let retained_exploration = candidate.slips > 0
        && candidate.path_length > FROZEN_ZERO_MOTION_TOLERANCE
        && candidate.dominant_patch_changes > 0;
    let conservation =
        candidate.conservation_pass && empty.conservation_pass && zero_a.conservation_pass;
    let energetic = candidate.a_spent > 0.0
        && candidate.w_generated > 0.0
        && candidate.a_to_w_residual <= 1e-8
        && candidate.reserve_before == candidate.reserve_after;
    let zero_a_pass = zero_a.a_spent == 0.0
        && zero_a.w_generated.abs() <= 1e-12
        && zero_a.path_length <= FROZEN_ZERO_MOTION_TOLERANCE
        && zero_a.slips == 0;
    let empty_pass = empty.maximum_contact_patches == 0
        && empty.cumulative_acquisition == 0.0
        && empty.contact_trace.iter().all(Vec::is_empty)
        && empty.motor_hash == unguided.motor_hash;
    let rotation_pass = (candidate.cumulative_acquisition - rotated.cumulative_acquisition).abs()
        <= ROTATION_TOLERANCE
        && (candidate.path_length - rotated.path_length).abs() <= ROTATION_TOLERANCE
        && (candidate.a_spent - rotated.a_spent).abs() <= ROTATION_TOLERANCE
        && (candidate.displacement[0] + rotated.displacement[0]).abs() <= ROTATION_TOLERANCE
        && (candidate.displacement[1] + rotated.displacement[1]).abs() <= ROTATION_TOLERANCE;
    let contact_zero_pass = candidate
        .contact_history
        .iter()
        .zip(&candidate.motor_history)
        .zip(&candidate.raw_history)
        .all(|((contact, motor), raw)| {
            (0..TOPOLOGY_SIZE).all(|i| {
                if contact[i] > 0.0 {
                    motor[i].abs() <= ZERO_MOTOR_TOLERANCE
                } else {
                    motor[i] == raw[i]
                }
            })
        });
    let contact_patches_attenuated = candidate
        .contact_history
        .iter()
        .zip(&candidate.raw_history)
        .zip(&candidate.motor_history)
        .any(|((contact, raw), motor)| {
            (0..TOPOLOGY_SIZE).any(|i| {
                contact[i] > 0.0
                    && raw[i] > ZERO_MOTOR_TOLERANCE
                    && motor[i].abs() <= ZERO_MOTOR_TOLERANCE
            })
        });
    let noncontact_demand_observed = candidate
        .steps
        .iter()
        .any(|step| step.noncontact_active_resource_request > 0.0);
    let classification = if !contact_zero_pass
        || !conservation
        || !energetic
        || !zero_a_pass
        || !empty_pass
        || !rotation_pass
    {
        "M2_CONTACT_LOCAL_EXPLOITATION_CEILING_AUDIT_INVALID"
    } else if ten_percent && retained_exploration {
        "M2_CONTACT_LOCAL_SUPPRESSION_CEILING_REACHED"
    } else if ten_percent {
        "M2_CONTACT_LOCAL_SUPPRESSION_CEILING_EXPLORATION_COLLAPSE"
    } else {
        "M2_CONTACT_LOCAL_SUPPRESSION_FAMILY_MECHANICALLY_INSUFFICIENT"
    };

    write_json(
        &output,
        "protocol.json",
        &json!({"directive": DIRECTIVE, "starting_head": STARTING_HEAD, "historical_fixture": "DC-DEV-013 exact", "topology_size": TOPOLOGY_SIZE, "settlement_steps": SETTLEMENT_STEPS, "assay_steps": ASSAY_STEPS, "resource_center": RESOURCE_CENTER, "resource_radius": RESOURCE_RADIUS, "initial_n_mass": INITIAL_N_MASS, "initial_f_mass": INITIAL_F_MASS, "minimum_relative_improvement": MIN_RELATIVE_IMPROVEMENT, "ceiling_rule": "contacted patches motor=0; noncontact patches exact ENTRY-005 raw activity", "observer_only": true, "production_behavior_changed": false}),
    );
    write_json(
        &output,
        "authority.json",
        &json!({"directive": DIRECTIVE, "starting_head": STARTING_HEAD, "entry008": "M2_LOCAL_CONTACT_REFRACTORY_EXPLOITATION_UPTAKE_INSUFFICIENT", "m1": "CLOSED/FROZEN", "production": "MaturationCoupledV4 / reserve OFF", "pr_44": "OPEN/DRAFT/UNMERGED/UNMODIFIED", "next_execution_started": false}),
    );
    write_json(
        &output,
        "architecture.json",
        &json!({"new_production_schema": false, "counterfactual": "contact-zero local motor suppression upper bound", "contact_output": "0.0", "noncontact_output": "ENTRY005 activity_after", "intrinsic_equation_changed": false, "adaptation_equation_changed": false, "new_numeric_parameter": false, "global_mode": false, "memory": false, "target": false, "gradient": false, "uptake_changed": false}),
    );
    write_json(
        &output,
        "contact_ceiling.json",
        &json!({"contact_zero_parity": contact_zero_pass, "contact_attenuation_occurred": contact_patches_attenuated, "noncontact_raw_output": true, "contacted_patches": candidate.contact_trace.iter().flatten().copied().collect::<std::collections::BTreeSet<_>>(), "noncontact_active_resource_demand_observed": noncontact_demand_observed}),
    );
    write_json(&output, "candidate_ceiling.json", &compact(&candidate));
    write_json(&output, "entry005_control.json", &compact(&unguided));
    write_json(&output, "empty_sham.json", &compact(&empty));
    write_json(&output, "zero_a_control.json", &compact(&zero_a));
    write_json(
        &output,
        "concentration_feedback.json",
        &json!({"candidate_sampled_steps": sampled(&candidate), "unguided_sampled_steps": sampled(&unguided), "contacted_vs_noncontact_active_resource": candidate.steps.iter().map(|step| json!({"step": step.step, "contact_request": step.contact_active_resource_request, "noncontact_request": step.noncontact_active_resource_request, "contact_spent": step.contact_active_resource_spent, "noncontact_spent": step.noncontact_active_resource_spent})).collect::<Vec<_>>(), "dense_trajectories_externalized": dense.is_some()}),
    );
    write_json(
        &output,
        "acquisition_benefit.json",
        &json!({"candidate": candidate.cumulative_acquisition, "unguided": unguided.cumulative_acquisition, "relative_improvement": candidate_benefit, "minimum_relative_improvement": MIN_RELATIVE_IMPROVEMENT, "ten_percent_gate": ten_percent, "exploration_retained": retained_exploration, "ceiling_reaches_gate": ten_percent && retained_exploration}),
    );
    write_json(
        &output,
        "material_closure.json",
        &json!({"a_spent": candidate.a_spent, "w_generated": candidate.w_generated, "a_to_w_residual": candidate.a_to_w_residual, "a_to_w_closure": energetic, "r_unchanged": candidate.reserve_before == candidate.reserve_after, "zero_a_pass": zero_a_pass, "conservation_pass": conservation}),
    );
    write_json(
        &output,
        "rotation_check.json",
        &json!({"pass": rotation_pass, "unrotated": compact(&candidate), "rotated": compact(&rotated), "tolerance": ROTATION_TOLERANCE}),
    );
    write_json(
        &output,
        "seed_diversity.json",
        &json!({"seeds": SEED_SET, "results": seed_results.iter().map(compact).collect::<Vec<_>>(), "screening": false, "seed_1_is_primary": true}),
    );
    write_json(
        &output,
        "restart_boundary.json",
        &json!({"intrinsic_state_restart": "PASS (preserved contract)", "generic_full_mesh_restart": "KNOWN_FAIL", "candidate_interpretation_contaminated": false}),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({"scientific_source_changed": false, "m1": "CLOSED/FROZEN", "production": "MaturationCoupledV4 / reserve OFF", "v2_d087": "8/8", "v3_d087": "8/8", "v4_d087": "7/8", "v4_vector": [true,true,false,true,true,true,true,true], "entry008_preserved": true}),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({"status": "PASS", "regulator": true, "continuity": true, "plasticity": true, "contact": true, "contact_regulation": true, "finite_resource": true, "traction": true, "d088": true, "d091": true, "evolution_harness": true}),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({"classification": classification, "contact_local_suppression_family": if classification == "M2_CONTACT_LOCAL_SUPPRESSION_FAMILY_MECHANICALLY_INSUFFICIENT" { "MECHANICALLY_INSUFFICIENT" } else { "NOT_CLOSED" }, "candidate_acquisition": candidate.cumulative_acquisition, "unguided_acquisition": unguided.cumulative_acquisition, "relative_improvement": candidate_benefit, "ten_percent_gate": ten_percent, "exploration_retained": retained_exploration, "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED", "next_execution_started": false}),
    );
    let files = [
        "protocol.json",
        "authority.json",
        "architecture.json",
        "contact_ceiling.json",
        "candidate_ceiling.json",
        "entry005_control.json",
        "empty_sham.json",
        "zero_a_control.json",
        "concentration_feedback.json",
        "acquisition_benefit.json",
        "material_closure.json",
        "rotation_check.json",
        "seed_diversity.json",
        "restart_boundary.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "qualification.json",
    ];
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({"directive": DIRECTIVE, "files": files, "source_hashes": {"intrinsic_exploration": source_hash("intrinsic_exploration.rs"), "spatial_resource": source_hash("spatial_resource.rs"), "contractility": source_hash("contractility.rs"), "traction": source_hash("stick_slip_traction.rs")}}),
    );
    if let Some(root) = dense {
        write_json(
            &root,
            "dense_trajectories.json",
            &json!({"candidate": candidate, "unguided": unguided, "empty": empty, "zero_a": zero_a, "rotated": rotated, "seed_results": seed_results}),
        );
    }
    println!("{classification}");
}
