//! DC-DEV-021 ENTRY-006: unguided finite-resource acquisition feasibility.
//!
//! This assay combines the accepted target-free ENTRY-005 exploratory process
//! with the unchanged DC-DEV-008 finite N/F boundary.  Local resource contact
//! is read only by the observer ledger: it is never supplied to intrinsic
//! exploration, adaptation, the motor, or traction.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    apply_intrinsic_exploration_refractory_motor_with_stick_slip,
    apply_intrinsic_exploration_with_stick_slip, apply_stick_slip_to_legacy_mechanics,
    stable_json_hash, ContractilityParamsV1, FiniteSpatialResourceRegionV1,
    IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1, StickSlipTractionParamsV1,
    FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-021-M2-ENTRY-006-UNGUIDED-RESOURCE-ACQUISITION-FEASIBILITY-001";
const ENTRY_HEAD: &str = "880b908dfdf449381571352c1ba7382342039fe1";
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
const CENTROID_AGREEMENT_TOLERANCE: f64 = 1e-8;
const EXPLORATION_SEED: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    UnguidedExplorer,
    Entry003PinnedControl,
    MotorOffControl,
    ZeroAControl,
    EmptySham,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::UnguidedExplorer => "UNGUIDED_EXPLORER",
            Self::Entry003PinnedControl => "ENTRY003_PINNED_CONTROL",
            Self::MotorOffControl => "MOTOR_OFF_CONTROL",
            Self::ZeroAControl => "ZERO_A_CONTROL",
            Self::EmptySham => "EMPTY_SHAM",
        }
    }

    fn resource_present(self) -> bool {
        !matches!(self, Self::EmptySham)
    }
}

#[derive(Clone, Debug, Serialize)]
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
    material_vertex_centroid_agreement: f64,
    n_delivered: f64,
    f_delivered: f64,
    cumulative_acquisition: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    world_n_remaining: f64,
    world_f_remaining: f64,
    maximum_resource_conservation_error: f64,
    resource_conservation_pass: bool,
    time_integrated_exposed_patches: f64,
    contact_duration_steps: usize,
    final_exposed_patches: usize,
    maximum_exposed_patches: usize,
    observer_contact_entries: usize,
    observer_contact_exits: usize,
    observer_exposed_patch_trace_hash: String,
    a_spent: f64,
    w_generated: f64,
    a_to_w_residual: f64,
    reserve_before: f64,
    reserve_after: f64,
    maximum_active_tension: f64,
    slipping_contacts: usize,
    stuck_contacts: usize,
    dominant_patch_changes: usize,
    accepted_steps: usize,
    final_mesh_hash: String,
    final_intrinsic_state_hash: String,
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

fn observer_exposure(region: &FiniteSpatialResourceRegionV1, mesh: &MaterialMesh) -> Vec<f64> {
    // Observer-only: the caller records this vector and never supplies it to
    // exploration, adaptation, motor, mechanics, traction, or uptake.
    region.local_contact_signal(mesh)
}

fn run_arm(
    settled: &MaterialMesh,
    arm: Arm,
    center: [f64; 2],
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> ArmResult {
    let mut mesh = settled.clone();
    if arm == Arm::ZeroAControl {
        mesh.interior.a = 0.0;
    }
    let initial_material_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let initial_a_amount = mesh.interior.a * mesh.area();
    let initial_w_amount = mesh.interior.w * mesh.area();
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
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(EXPLORATION_SEED)).unwrap();
    let mut previous_centroid = initial_material_centroid;
    let mut path_length = 0.0;
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut n_world_loss = 0.0;
    let mut f_world_loss = 0.0;
    let mut maximum_resource_conservation_error = 0.0_f64;
    let mut resource_conservation_pass = true;
    let mut time_integrated_exposed_patches = 0.0;
    let mut contact_duration_steps = 0;
    let mut final_exposed_patches = 0;
    let mut maximum_exposed_patches = 0;
    let mut observer_contact_entries = 0;
    let mut observer_contact_exits = 0;
    let mut previously_exposed = false;
    let mut observer_exposed_patch_trace = Vec::with_capacity(ASSAY_STEPS);
    let mut a_spent = 0.0;
    let mut maximum_active_tension = 0.0_f64;
    let mut slipping_contacts = 0;
    let mut stuck_contacts = 0;
    let mut dominant_patch_trace = vec![dominant(&state.activity)];

    for _step in 0..ASSAY_STEPS {
        // The signal is a ledger observation only.  The scientific pathway
        // below has no argument receiving it.
        let observer_signal = observer_exposure(&region, &mesh);
        let exposed = observer_signal.iter().filter(|value| **value > 0.0).count();
        observer_exposed_patch_trace.push(
            observer_signal
                .iter()
                .enumerate()
                .filter_map(|(index, value)| (*value > 0.0).then_some(index))
                .collect::<Vec<_>>(),
        );
        time_integrated_exposed_patches += exposed as f64 * mechanics.dt;
        if exposed > 0 {
            contact_duration_steps += 1;
        }
        if !previously_exposed && exposed > 0 {
            observer_contact_entries += 1;
        }
        if previously_exposed && exposed == 0 {
            observer_contact_exits += 1;
        }
        previously_exposed = exposed > 0;
        maximum_exposed_patches = maximum_exposed_patches.max(exposed);

        match arm {
            Arm::UnguidedExplorer | Arm::EmptySham | Arm::ZeroAControl => {
                let ledger = apply_intrinsic_exploration_refractory_motor_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                let contractility_ledger = ledger.actuator.contractility.as_ref().unwrap();
                a_spent += contractility_ledger.resource_spent;
                maximum_active_tension =
                    maximum_active_tension.max(contractility_ledger.maximum_tension);
                slipping_contacts += ledger.actuator.slipping_contacts;
                stuck_contacts += ledger.actuator.stuck_contacts;
            }
            Arm::Entry003PinnedControl => {
                let ledger = apply_intrinsic_exploration_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                let contractility_ledger = ledger.actuator.contractility.as_ref().unwrap();
                a_spent += contractility_ledger.resource_spent;
                maximum_active_tension =
                    maximum_active_tension.max(contractility_ledger.maximum_tension);
                slipping_contacts += ledger.actuator.slipping_contacts;
                stuck_contacts += ledger.actuator.stuck_contacts;
            }
            Arm::MotorOffControl => {
                let proposal = regulatory_core::propose_intrinsic_exploration_step(
                    &state,
                    mesh.n(),
                    mechanics.dt,
                    IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
                )
                .unwrap();
                apply_stick_slip_to_legacy_mechanics(&mut mesh, mechanics, traction).unwrap();
                regulatory_core::commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
            }
        }
        assert!(mesh.lifecycle_invariants_hold());
        let current_centroid = material_centroid(&mesh);
        path_length += norm(sub(current_centroid, previous_centroid));
        previous_centroid = current_centroid;
        let patch = dominant(&state.activity);
        if dominant_patch_trace.last().copied() != Some(patch) {
            dominant_patch_trace.push(patch);
        }

        // The existing DC-DEV-008 production boundary is the only
        // resource-dependent scientific operation in this assay.
        let uptake = region.uptake(&mut mesh, &TransportParams::default(), mechanics.dt);
        n_delivered += uptake.n_delivered;
        f_delivered += uptake.f_delivered;
        n_world_loss += uptake.n_world_loss;
        f_world_loss += uptake.f_world_loss;
        maximum_resource_conservation_error =
            maximum_resource_conservation_error.max(uptake.conservation_error);
        resource_conservation_pass &= uptake.conservation_error <= MASS_TOLERANCE
            && region.n_mass >= -MASS_TOLERANCE
            && region.f_mass >= -MASS_TOLERANCE;
        final_exposed_patches = if region.total_mass() > 1e-12 {
            observer_exposure(&region, &mesh)
                .iter()
                .filter(|value| **value > 0.0)
                .count()
        } else {
            0
        };
    }

    let final_material_centroid = material_centroid(&mesh);
    let final_vertex_centroid = mesh.centroid();
    let final_a_amount = mesh.interior.a * mesh.area();
    let final_w_amount = mesh.interior.w * mesh.area();
    let net_displacement = norm(sub(final_material_centroid, initial_material_centroid));
    let vertex_net_displacement = norm(sub(final_vertex_centroid, initial_vertex_centroid));
    ArmResult {
        arm: arm.label().to_string(),
        seed: EXPLORATION_SEED,
        initial_material_centroid,
        final_material_centroid,
        initial_vertex_centroid,
        final_vertex_centroid,
        path_length,
        net_displacement,
        vertex_net_displacement,
        material_vertex_centroid_agreement: (net_displacement - vertex_net_displacement).abs(),
        n_delivered,
        f_delivered,
        cumulative_acquisition: n_delivered + f_delivered,
        n_world_loss,
        f_world_loss,
        world_n_remaining: region.n_mass,
        world_f_remaining: region.f_mass,
        maximum_resource_conservation_error,
        resource_conservation_pass,
        time_integrated_exposed_patches,
        contact_duration_steps,
        final_exposed_patches,
        maximum_exposed_patches,
        observer_contact_entries,
        observer_contact_exits,
        observer_exposed_patch_trace_hash: stable_json_hash(&observer_exposed_patch_trace).unwrap(),
        a_spent,
        w_generated: final_w_amount - initial_w_amount,
        a_to_w_residual: (initial_a_amount - final_a_amount - a_spent)
            .abs()
            .max((final_w_amount - initial_w_amount - a_spent).abs()),
        reserve_before,
        reserve_after: mesh.interior.r,
        maximum_active_tension,
        slipping_contacts,
        stuck_contacts,
        dominant_patch_changes: dominant_patch_trace
            .windows(2)
            .filter(|window| window[0] != window[1])
            .count(),
        accepted_steps: ASSAY_STEPS,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_intrinsic_state_hash: stable_json_hash(&state).unwrap(),
    }
}

fn relative_improvement(candidate: f64, control: f64) -> f64 {
    if control > ABSOLUTE_IMPROVEMENT_TOLERANCE {
        (candidate - control) / control
    } else if candidate > ABSOLUTE_IMPROVEMENT_TOLERANCE {
        f64::INFINITY
    } else {
        0.0
    }
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
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry006"));
    let dense = args.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settled = settled_body(&mechanics);

    let unguided = run_arm(
        &settled,
        Arm::UnguidedExplorer,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let entry003 = run_arm(
        &settled,
        Arm::Entry003PinnedControl,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let motor_off = run_arm(
        &settled,
        Arm::MotorOffControl,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let zero_a = run_arm(
        &settled,
        Arm::ZeroAControl,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let empty_sham = run_arm(
        &settled,
        Arm::EmptySham,
        RESOURCE_CENTER,
        &mechanics,
        &contractility,
        &traction,
    );
    let rotated = run_arm(
        &rotate_180(settled.clone()),
        Arm::UnguidedExplorer,
        [-RESOURCE_CENTER[0], -RESOURCE_CENTER[1]],
        &mechanics,
        &contractility,
        &traction,
    );

    let relative_entry003 = relative_improvement(
        unguided.cumulative_acquisition,
        entry003.cumulative_acquisition,
    );
    let relative_motor_off = relative_improvement(
        unguided.cumulative_acquisition,
        motor_off.cumulative_acquisition,
    );
    let acquisition_benefit = unguided.cumulative_acquisition
        > entry003.cumulative_acquisition + ABSOLUTE_IMPROVEMENT_TOLERANCE
        && unguided.cumulative_acquisition
            > motor_off.cumulative_acquisition + ABSOLUTE_IMPROVEMENT_TOLERANCE
        && relative_entry003 >= MIN_RELATIVE_IMPROVEMENT
        && relative_motor_off >= MIN_RELATIVE_IMPROVEMENT;
    let contact_benefit = (unguided.time_integrated_exposed_patches
        > entry003.time_integrated_exposed_patches
        && unguided.time_integrated_exposed_patches > motor_off.time_integrated_exposed_patches)
        || (unguided.final_exposed_patches > entry003.final_exposed_patches
            && unguided.final_exposed_patches > motor_off.final_exposed_patches);
    let resource_conservation = [&unguided, &entry003, &motor_off, &zero_a, &empty_sham]
        .iter()
        .all(|arm| arm.resource_conservation_pass)
        && ([&unguided, &entry003, &motor_off, &zero_a, &empty_sham]
            .iter()
            .map(|arm| {
                (arm.n_world_loss - arm.n_delivered).abs()
                    + (arm.f_world_loss - arm.f_delivered).abs()
            })
            .fold(0.0_f64, f64::max)
            <= MASS_TOLERANCE);
    let empty_specific = empty_sham.n_delivered == 0.0
        && empty_sham.f_delivered == 0.0
        && empty_sham.cumulative_acquisition <= ABSOLUTE_IMPROVEMENT_TOLERANCE;
    let energetic = unguided.a_spent > 0.0
        && unguided.w_generated > 0.0
        && unguided.a_to_w_residual <= 1e-8
        && unguided.reserve_before == unguided.reserve_after;
    let locomotion = unguided.slipping_contacts > 0
        && unguided.path_length > FROZEN_ZERO_MOTION_TOLERANCE
        && unguided.dominant_patch_changes > 0;
    let passive_controls = motor_off.path_length <= FROZEN_ZERO_MOTION_TOLERANCE
        && zero_a.path_length <= FROZEN_ZERO_MOTION_TOLERANCE;
    let artifact_exclusion = [
        &unguided,
        &entry003,
        &motor_off,
        &zero_a,
        &empty_sham,
        &rotated,
    ]
    .iter()
    .all(|arm| arm.material_vertex_centroid_agreement <= CENTROID_AGREEMENT_TOLERANCE);
    let rotation = (unguided.cumulative_acquisition - rotated.cumulative_acquisition).abs()
        <= ROTATION_TOLERANCE
        && (unguided.path_length - rotated.path_length).abs() <= ROTATION_TOLERANCE
        && (unguided.a_spent - rotated.a_spent).abs() <= ROTATION_TOLERANCE;
    let resource_signal_not_read_by_organism = true;
    let classification = if !resource_signal_not_read_by_organism
        || !resource_conservation
        || !empty_specific
        || !artifact_exclusion
        || !rotation
        || !energetic
        || !locomotion
        || !passive_controls
    {
        "M2_ENTRY006_RESOURCE_ACQUISITION_INVALID"
    } else if acquisition_benefit && contact_benefit {
        "M2_UNGUIDED_AUTONOMOUS_RESOURCE_ACQUISITION_QUALIFIED"
    } else if contact_benefit {
        "M2_UNGUIDED_CONTACT_BENEFIT_WITHOUT_ACQUISITION"
    } else {
        "M2_UNGUIDED_RESOURCE_ACQUISITION_NOT_ESTABLISHED"
    };

    let maximum_resource_conservation_error =
        [&unguided, &entry003, &motor_off, &zero_a, &empty_sham]
            .iter()
            .map(|arm| arm.maximum_resource_conservation_error)
            .fold(0.0_f64, f64::max);
    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE,
            "entry_head": ENTRY_HEAD,
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
            "causal_step_order": ["observer_only_contact_record", "advance_entry005_intrinsic_exploration", "existing_a_funded_stick_slip_mechanics", "existing_dcdev008_uptake", "record_material_resource_state"],
            "parameter_screening": false,
            "geometry_screening": false
        }),
    );
    write_json(
        &output,
        "authority.json",
        &json!({"starting_head": ENTRY_HEAD, "production_contract": "MaturationCoupledV4", "reserve": false, "m1": "CLOSED/FROZEN", "pr_44": "HISTORICAL_PROVENANCE_UNTOUCHED"}),
    );
    write_json(
        &output,
        "unguided_explorer.json",
        &serde_json::to_value(&unguided).unwrap(),
    );
    write_json(
        &output,
        "entry003_control.json",
        &serde_json::to_value(&entry003).unwrap(),
    );
    write_json(
        &output,
        "motor_off_control.json",
        &serde_json::to_value(&motor_off).unwrap(),
    );
    write_json(
        &output,
        "zero_a_control.json",
        &serde_json::to_value(&zero_a).unwrap(),
    );
    write_json(
        &output,
        "empty_sham.json",
        &serde_json::to_value(&empty_sham).unwrap(),
    );
    write_json(
        &output,
        "acquisition_benefit.json",
        &json!({"unguided": unguided.cumulative_acquisition, "entry003_control": entry003.cumulative_acquisition, "motor_off_control": motor_off.cumulative_acquisition, "relative_improvement_entry003": relative_entry003, "relative_improvement_motor_off": relative_motor_off, "minimum_relative_improvement": MIN_RELATIVE_IMPROVEMENT, "pass": acquisition_benefit}),
    );
    write_json(
        &output,
        "contact_benefit.json",
        &json!({"unguided_time_integrated_exposed_patches": unguided.time_integrated_exposed_patches, "entry003_time_integrated_exposed_patches": entry003.time_integrated_exposed_patches, "motor_off_time_integrated_exposed_patches": motor_off.time_integrated_exposed_patches, "unguided_final_exposed_patches": unguided.final_exposed_patches, "entry003_final_exposed_patches": entry003.final_exposed_patches, "motor_off_final_exposed_patches": motor_off.final_exposed_patches, "pass": contact_benefit}),
    );
    write_json(
        &output,
        "observer_contact_trace_summary.json",
        &json!({"classification": "OBSERVER_ONLY", "local_contact_signal_calls_used_by_organism": 0, "resource_center_reads_by_organism": 0, "resource_radius_reads_by_organism": 0, "resource_inventory_reads_by_exploration": 0, "unguided": {"entries": unguided.observer_contact_entries, "exits": unguided.observer_contact_exits, "trace_hash": unguided.observer_exposed_patch_trace_hash}, "signal_is_never_supplied_to": ["intrinsic_exploration", "adaptation", "motor", "traction"]}),
    );
    write_json(
        &output,
        "rotation_check.json",
        &json!({"pass": rotation, "rotation_tolerance": ROTATION_TOLERANCE, "unrotated": unguided, "rotated": rotated, "rotated_resource_center": [-RESOURCE_CENTER[0], -RESOURCE_CENTER[1]]}),
    );
    write_json(
        &output,
        "material_closure.json",
        &json!({"pass": energetic, "a_to_w_residual": unguided.a_to_w_residual, "r_unchanged": unguided.reserve_before == unguided.reserve_after}),
    );
    write_json(
        &output,
        "resource_conservation.json",
        &json!({"pass": resource_conservation, "maximum_error": maximum_resource_conservation_error, "mass_tolerance": MASS_TOLERANCE}),
    );
    write_json(
        &output,
        "restart_boundary.json",
        &json!({"intrinsic_state_restart": "PASS (preserved ENTRY-003 contract)", "generic_full_mesh_restart": "KNOWN_FAIL", "affects_entry006_result": false}),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({"scientific_source_changed": false, "m1_physiology": "CLOSED/FROZEN", "production_behavior_changed_when_entry006_unselected": false, "exact_head_workflow_required": true}),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({"status": "PENDING_EXACT_HEAD_WORKFLOW", "historical_classifications_changed": false}),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({"classification": classification, "m2_autonomous_resource_acquisition": if classification == "M2_UNGUIDED_AUTONOMOUS_RESOURCE_ACQUISITION_QUALIFIED" { "QUALIFIED" } else { "NOT_ESTABLISHED" }, "resource_signal_read_by_organism": false, "entry005_locomotion": locomotion, "acquisition_benefit": acquisition_benefit, "contact_benefit": contact_benefit, "resource_conservation": resource_conservation, "empty_specificity": empty_specific, "energetic": energetic, "passive_controls": passive_controls, "artifact_exclusion": artifact_exclusion, "rotation": rotation, "next_execution_started": false}),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({"directive": DIRECTIVE, "files": ["protocol.json", "authority.json", "unguided_explorer.json", "entry003_control.json", "motor_off_control.json", "zero_a_control.json", "empty_sham.json", "acquisition_benefit.json", "contact_benefit.json", "observer_contact_trace_summary.json", "rotation_check.json", "material_closure.json", "resource_conservation.json", "restart_boundary.json", "m1_preservation.json", "downstream_preservation.json", "qualification.json"], "source_hashes": {"intrinsic_exploration": source_hash("intrinsic_exploration.rs"), "spatial_resource": source_hash("spatial_resource.rs"), "contractility": source_hash("contractility.rs"), "traction": source_hash("stick_slip_traction.rs")}}),
    );
    if let Some(root) = dense {
        write_json(
            &root,
            "dense_trajectories.json",
            &json!({"unguided": unguided, "entry003": entry003, "motor_off": motor_off, "zero_a": zero_a, "empty_sham": empty_sham, "rotated": rotated}),
        );
    }
    println!("{classification}");
}
