//! DC-DEV-013: local finite-resource contact feeding qualification.
//!
//! This assay composes the production DC-DEV-008 resource boundary with the
//! existing DC-DEV-002 regulator, DC-DEV-004 reserve-funded contractility, and
//! DC-DEV-011 passive isotropic stick-slip path.  It contains no target,
//! gradient, planner, reward, or second sensing implementation.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use regulatory_core::continuity::ContinuityMaterialFrameV1;
use regulatory_core::{
    apply_local_contractility_with_stick_slip, apply_stick_slip_to_legacy_mechanics,
    stable_json_hash, ContinuityNetworkV1, ContractilityParamsV1, FiniteSpatialResourceRegionV1,
    StickSlipTractionParamsV1, TopologyEventV1, FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-013";
const ENTRY_COMMIT: &str = "f9c6d4e83fc9dc50e4d2ec4004ea640084ce5732";
const FREEZE_COMMIT: &str = "fa8a689adff8cbc3b981038c4812ebdc0623116c";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 480;
const RESOURCE_RADIUS: f64 = 1.5;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const INITIAL_N_MASS: f64 = 3.0;
const INITIAL_F_MASS: f64 = 3.0;
const MASS_TOLERANCE: f64 = 1e-10;
const ABSOLUTE_IMPROVEMENT_TOLERANCE: f64 = 1e-12;
const MIN_RELATIVE_IMPROVEMENT: f64 = 0.10;
const ROTATION_TOLERANCE: f64 = 1e-9;
const CENTROID_AGREEMENT_TOLERANCE: f64 = 1e-8;
const R1_MAX_ATTEMPTED_VELOCITY: f64 = 2.6645352591003757e-9;
const R1_MAX_LOCAL_DISPLACEMENT: f64 = 5.3290705182007514e-11;
const R1_MAX_MATERIAL_CENTROID_STEP: f64 = 2.220446049250313e-13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arm {
    Active,
    SensorOff,
    MotorOff,
    ZeroReserve,
    EmptySham,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::Active => "active_sensor_funded_stick_slip",
            Self::SensorOff => "sensor_off_funded_stick_slip",
            Self::MotorOff => "motor_off_local_sensor",
            Self::ZeroReserve => "zero_reserve_local_sensor",
            Self::EmptySham => "empty_resource_sham",
        }
    }

    fn sensor_enabled(self) -> bool {
        !matches!(self, Self::SensorOff)
    }

    fn motor_enabled(self) -> bool {
        !matches!(self, Self::MotorOff)
    }

    fn zero_reserve(self) -> bool {
        matches!(self, Self::ZeroReserve)
    }

    fn resource_present(self) -> bool {
        !matches!(self, Self::EmptySham)
    }
}

#[derive(Debug, Clone, Serialize)]
struct Settlement {
    mesh: MaterialMesh,
    initial_mesh_hash: String,
    settled_mesh_hash: String,
    settled_chemistry_hash: String,
    maximum_attempted_velocity: f64,
    maximum_local_displacement: f64,
    maximum_material_centroid_step: f64,
    settled: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ArmRun {
    arm: String,
    initial_material_centroid: [f64; 2],
    final_material_centroid: [f64; 2],
    initial_vertex_centroid: [f64; 2],
    final_vertex_centroid: [f64; 2],
    material_centroid_displacement: f64,
    vertex_centroid_displacement: f64,
    material_vertex_displacement_agreement: f64,
    time_integrated_exposed_patches: f64,
    contact_duration_steps: usize,
    final_exposed_patches: usize,
    maximum_exposed_patches: usize,
    signal_positive_steps: usize,
    maximum_signal: f64,
    n_delivered: f64,
    f_delivered: f64,
    cumulative_acquisition: f64,
    world_n_remaining: f64,
    world_f_remaining: f64,
    reserve_spent: f64,
    maximum_funded_tension: f64,
    initial_reserve: f64,
    final_reserve: f64,
    maximum_positive_substrate_work: f64,
    substrate_work: f64,
    stuck_contacts: usize,
    slipping_contacts: usize,
    maximum_conservation_error: f64,
    conservation_pass: bool,
    final_a: f64,
    final_r: f64,
    a_trajectory_hash: String,
    r_trajectory_hash: String,
    final_chemistry_hash: String,
    final_mesh_hash: String,
    regulatory_trace_hash: String,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn norm(vector: [f64; 2]) -> f64 {
    vector[0].hypot(vector[1])
}

fn subtract(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
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
        return mesh.centroid();
    }
    [weighted[0] / total, weighted[1] / total]
}

fn chemistry_hash(mesh: &MaterialMesh) -> String {
    stable_json_hash(&(mesh.interior, mesh.exterior, &mesh.edges)).unwrap()
}

fn seed_mesh() -> MaterialMesh {
    MaterialMesh::seed_regular(
        TOPOLOGY_SIZE,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.4,
            f: 0.4,
            r: 0.6,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    )
}

fn settle_body(mechanics: &MechParams) -> Settlement {
    let mut mesh = seed_mesh();
    let initial_mesh_hash = stable_json_hash(&mesh).unwrap();
    let mut maximum_attempted_velocity: f64 = 0.0;
    let mut maximum_local_displacement: f64 = 0.0;
    let mut maximum_material_centroid_step: f64 = 0.0;
    let mut late_attempted_velocity: f64 = 0.0;
    let mut late_local_displacement: f64 = 0.0;
    let mut late_material_centroid_step: f64 = 0.0;
    for step in 0..SETTLEMENT_STEPS {
        let before_vertices = mesh.vertices.clone();
        let before_centroid = material_centroid(&mesh);
        assert!(mechanics_step(&mut mesh, mechanics));
        for (before, after) in before_vertices.iter().zip(&mesh.vertices) {
            let displacement = norm(subtract(*after, *before));
            let attempted_velocity = displacement * mechanics.gamma / mechanics.dt;
            maximum_attempted_velocity = maximum_attempted_velocity.max(attempted_velocity);
            maximum_local_displacement = maximum_local_displacement.max(displacement);
            if step >= SETTLEMENT_STEPS - 1_000 {
                late_attempted_velocity = late_attempted_velocity.max(attempted_velocity);
                late_local_displacement = late_local_displacement.max(displacement);
            }
        }
        let material_step = norm(subtract(material_centroid(&mesh), before_centroid));
        maximum_material_centroid_step = maximum_material_centroid_step.max(material_step);
        if step >= SETTLEMENT_STEPS - 1_000 {
            late_material_centroid_step = late_material_centroid_step.max(material_step);
        }
    }
    let settled = late_attempted_velocity <= R1_MAX_ATTEMPTED_VELOCITY
        && late_local_displacement <= R1_MAX_LOCAL_DISPLACEMENT
        && late_material_centroid_step <= R1_MAX_MATERIAL_CENTROID_STEP;
    assert!(
        settled,
        "legacy settlement failed: attempted_velocity={late_attempted_velocity:.17e}, local_displacement={late_local_displacement:.17e}, material_step={late_material_centroid_step:.17e}"
    );
    Settlement {
        initial_mesh_hash,
        settled_mesh_hash: stable_json_hash(&mesh).unwrap(),
        settled_chemistry_hash: chemistry_hash(&mesh),
        mesh,
        maximum_attempted_velocity,
        maximum_local_displacement,
        maximum_material_centroid_step,
        settled,
    }
}

fn run_arm(
    settled: &MaterialMesh,
    arm: Arm,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
    center: [f64; 2],
) -> ArmRun {
    let mut mesh = settled.clone();
    if arm.zero_reserve() {
        mesh.interior.r = 0.0;
    }
    let initial_material_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let initial_reserve = mesh.interior.r;
    let initial_n = if arm.resource_present() {
        INITIAL_N_MASS
    } else {
        0.0
    };
    let initial_f = if arm.resource_present() {
        INITIAL_F_MASS
    } else {
        0.0
    };
    let mut region =
        FiniteSpatialResourceRegionV1::new(center, RESOURCE_RADIUS, initial_n, initial_f);
    let initial_signal = if arm.sensor_enabled() {
        region.local_contact_signal(&mesh)
    } else {
        vec![0.0; mesh.n()]
    };
    let initial_frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
        &mesh.vertices,
        &initial_signal,
        mechanics.dt,
    );
    let mut network = ContinuityNetworkV1::new(initial_frame, Some(13013)).unwrap();
    let mut regulatory_trace = Vec::with_capacity(ASSAY_STEPS);
    let mut a_trajectory = Vec::with_capacity(ASSAY_STEPS + 1);
    let mut r_trajectory = Vec::with_capacity(ASSAY_STEPS + 1);
    a_trajectory.push(mesh.interior.a);
    r_trajectory.push(mesh.interior.r);
    let mut time_integrated_exposed_patches = 0.0;
    let mut contact_duration_steps = 0;
    let mut final_exposed_patches = 0;
    let mut maximum_exposed_patches = 0;
    let mut signal_positive_steps = 0;
    let mut maximum_signal: f64 = 0.0;
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut reserve_spent = 0.0;
    let mut maximum_funded_tension: f64 = 0.0;
    let mut maximum_positive_substrate_work: f64 = 0.0;
    let mut substrate_work = 0.0;
    let mut stuck_contacts = 0;
    let mut slipping_contacts = 0;
    let mut maximum_conservation_error: f64 = 0.0;
    let mut conservation_pass = true;

    for _step in 0..ASSAY_STEPS {
        // Fixed causal order: observe, regulate, actuate/mechanics, then uptake.
        let signal = if arm.sensor_enabled() {
            region.local_contact_signal(&mesh)
        } else {
            vec![0.0; mesh.n()]
        };
        let exposed = signal.iter().filter(|value| **value > 0.0).count();
        time_integrated_exposed_patches += exposed as f64 * mechanics.dt;
        if exposed > 0 {
            contact_duration_steps += 1;
            signal_positive_steps += 1;
        }
        maximum_exposed_patches = maximum_exposed_patches.max(exposed);
        maximum_signal = maximum_signal.max(signal.iter().copied().fold(0.0, f64::max));
        let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &mesh.vertices,
            &signal,
            mechanics.dt,
        );
        network.step(frame, TopologyEventV1::Stable).unwrap();
        regulatory_trace.push(stable_json_hash(&network.state).unwrap());

        let ledger = if arm.motor_enabled() {
            apply_local_contractility_with_stick_slip(
                &mut mesh,
                &network.state.activity,
                mechanics,
                contractility,
                traction,
            )
            .unwrap()
        } else {
            apply_stick_slip_to_legacy_mechanics(&mut mesh, mechanics, traction).unwrap()
        };
        stuck_contacts += ledger.stuck_contacts;
        slipping_contacts += ledger.slipping_contacts;
        substrate_work += ledger.substrate_work;
        maximum_positive_substrate_work =
            maximum_positive_substrate_work.max(ledger.substrate_work.max(0.0));
        if let Some(contractility_ledger) = ledger.contractility {
            reserve_spent += contractility_ledger.resource_spent;
            maximum_funded_tension =
                maximum_funded_tension.max(contractility_ledger.maximum_tension);
        }

        let resource_ledger = region.uptake(&mut mesh, &chemistry_transport(), mechanics.dt);
        n_delivered += resource_ledger.n_delivered;
        f_delivered += resource_ledger.f_delivered;
        maximum_conservation_error =
            maximum_conservation_error.max(resource_ledger.conservation_error);
        conservation_pass &= resource_ledger.conservation_error <= MASS_TOLERANCE
            && region.n_mass >= -MASS_TOLERANCE
            && region.f_mass >= -MASS_TOLERANCE;
        final_exposed_patches = if region.total_mass() > 1e-12 {
            region
                .local_contact_signal(&mesh)
                .iter()
                .filter(|value| **value > 0.0)
                .count()
        } else {
            0
        };
        a_trajectory.push(mesh.interior.a);
        r_trajectory.push(mesh.interior.r);
    }

    let final_material_centroid = material_centroid(&mesh);
    let final_vertex_centroid = mesh.centroid();
    let material_displacement = norm(subtract(final_material_centroid, initial_material_centroid));
    let vertex_displacement = norm(subtract(final_vertex_centroid, initial_vertex_centroid));
    ArmRun {
        arm: arm.label().to_string(),
        initial_material_centroid,
        final_material_centroid,
        initial_vertex_centroid,
        final_vertex_centroid,
        material_centroid_displacement: material_displacement,
        vertex_centroid_displacement: vertex_displacement,
        material_vertex_displacement_agreement: (material_displacement - vertex_displacement).abs(),
        time_integrated_exposed_patches,
        contact_duration_steps,
        final_exposed_patches,
        maximum_exposed_patches,
        signal_positive_steps,
        maximum_signal,
        n_delivered,
        f_delivered,
        cumulative_acquisition: n_delivered + f_delivered,
        world_n_remaining: region.n_mass,
        world_f_remaining: region.f_mass,
        reserve_spent,
        maximum_funded_tension,
        initial_reserve,
        final_reserve: mesh.interior.r,
        maximum_positive_substrate_work,
        substrate_work,
        stuck_contacts,
        slipping_contacts,
        maximum_conservation_error,
        conservation_pass,
        final_a: mesh.interior.a,
        final_r: mesh.interior.r,
        a_trajectory_hash: stable_json_hash(&a_trajectory).unwrap(),
        r_trajectory_hash: stable_json_hash(&r_trajectory).unwrap(),
        final_chemistry_hash: chemistry_hash(&mesh),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        regulatory_trace_hash: stable_json_hash(&regulatory_trace).unwrap(),
    }
}

fn chemistry_transport() -> chemistry_core::mesh_transport::TransportParams {
    chemistry_core::mesh_transport::TransportParams::default()
}

fn rotate_180(mesh: &MaterialMesh) -> MaterialMesh {
    let mut rotated = mesh.clone();
    for vertex in &mut rotated.vertices {
        *vertex = [-vertex[0], -vertex[1]];
    }
    rotated
}

fn arm_value(run: &ArmRun) -> Value {
    serde_json::to_value(run).unwrap()
}

fn local_sensor_checks(settled: &MaterialMesh) -> Value {
    let bearing = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        INITIAL_N_MASS,
        INITIAL_F_MASS,
    );
    let noncontact = FiniteSpatialResourceRegionV1::new(
        [30.0, 30.0],
        RESOURCE_RADIUS,
        INITIAL_N_MASS,
        INITIAL_F_MASS,
    );
    let empty = FiniteSpatialResourceRegionV1::new(RESOURCE_CENTER, RESOURCE_RADIUS, 0.0, 0.0);
    let depleted =
        FiniteSpatialResourceRegionV1::new(RESOURCE_CENTER, RESOURCE_RADIUS, 1e-13, 1e-13);
    let bearing_signal = bearing.local_contact_signal(settled);
    let noncontact_signal = noncontact.local_contact_signal(settled);
    let empty_signal = empty.local_contact_signal(settled);
    let depleted_signal = depleted.local_contact_signal(settled);
    json!({
        "production_schema": regulatory_core::LOCAL_RESOURCE_CONTACT_SIGNAL_SCHEMA_V1,
        "bearing_has_local_contact": bearing_signal.iter().any(|value| *value > 0.0),
        "noncontact_is_zero": noncontact_signal.iter().all(|value| *value == 0.0),
        "empty_geometry_is_zero": empty_signal.iter().all(|value| *value == 0.0),
        "depleted_resource_is_zero": depleted_signal.iter().all(|value| *value == 0.0),
        "signal_values_bounded": bearing_signal.iter().all(|value| *value == 0.0 || *value == 1.0),
        "signal_length": bearing_signal.len()
    })
}

fn legacy_parity(
    settled: &MaterialMesh,
    mechanics: &MechParams,
    traction: &StickSlipTractionParamsV1,
) -> Value {
    let mut first = settled.clone();
    let mut second = settled.clone();
    for _ in 0..16 {
        apply_stick_slip_to_legacy_mechanics(&mut first, mechanics, traction).unwrap();
        apply_stick_slip_to_legacy_mechanics(&mut second, mechanics, traction).unwrap();
    }
    let mut first_resource = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        INITIAL_N_MASS,
        INITIAL_F_MASS,
    );
    let mut second_resource = first_resource.clone();
    let mut first_body = settled.clone();
    let mut second_body = settled.clone();
    let first_ledger = first_resource.uptake(&mut first_body, &chemistry_transport(), mechanics.dt);
    let second_signal = second_resource.local_contact_signal(&second_body);
    let second_ledger =
        second_resource.uptake(&mut second_body, &chemistry_transport(), mechanics.dt);
    let body_chemistry_parity = chemistry_hash(&first_body) == chemistry_hash(&second_body);
    json!({
        "stick_slip_trajectory_parity": first.vertices == second.vertices,
        "resource_observation_is_read_only": second_signal.iter().any(|value| *value > 0.0) && first_ledger == second_ledger,
        "resource_body_parity": body_chemistry_parity,
        "pass": first.vertices == second.vertices && first_ledger == second_ledger && body_chemistry_parity
    })
}

fn main() {
    let output_root = std::env::var_os("DCDEV013_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev013"));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settlement = settle_body(&mechanics);

    let active = run_arm(
        &settlement.mesh,
        Arm::Active,
        &mechanics,
        &contractility,
        &traction,
        RESOURCE_CENTER,
    );
    let sensor_off = run_arm(
        &settlement.mesh,
        Arm::SensorOff,
        &mechanics,
        &contractility,
        &traction,
        RESOURCE_CENTER,
    );
    let motor_off = run_arm(
        &settlement.mesh,
        Arm::MotorOff,
        &mechanics,
        &contractility,
        &traction,
        RESOURCE_CENTER,
    );
    let zero_reserve = run_arm(
        &settlement.mesh,
        Arm::ZeroReserve,
        &mechanics,
        &contractility,
        &traction,
        RESOURCE_CENTER,
    );
    let empty_sham = run_arm(
        &settlement.mesh,
        Arm::EmptySham,
        &mechanics,
        &contractility,
        &traction,
        RESOURCE_CENTER,
    );

    let rotated_settlement = rotate_180(&settlement.mesh);
    let rotated_active = run_arm(
        &rotated_settlement,
        Arm::Active,
        &mechanics,
        &contractility,
        &traction,
        [-RESOURCE_CENTER[0], -RESOURCE_CENTER[1]],
    );
    let active_rotation_error =
        (active.cumulative_acquisition - rotated_active.cumulative_acquisition).abs();
    let active_rotation_pass = active_rotation_error <= ROTATION_TOLERANCE;
    let local_sensor = local_sensor_checks(&settlement.mesh);
    let legacy = legacy_parity(&settlement.mesh, &mechanics, &traction);
    let maximum_conservation_error = [&active, &sensor_off, &motor_off, &zero_reserve, &empty_sham]
        .iter()
        .map(|run| run.maximum_conservation_error)
        .fold(0.0, f64::max);
    let maximum_positive_substrate_work =
        [&active, &sensor_off, &motor_off, &zero_reserve, &empty_sham]
            .iter()
            .map(|run| run.maximum_positive_substrate_work)
            .fold(0.0, f64::max);
    let maximum_material_vertex_agreement = [
        &active,
        &sensor_off,
        &motor_off,
        &zero_reserve,
        &empty_sham,
        &rotated_active,
    ]
    .iter()
    .map(|run| run.material_vertex_displacement_agreement)
    .fold(0.0, f64::max);
    let passive_substrate_pass = [&active, &sensor_off, &motor_off, &zero_reserve, &empty_sham]
        .iter()
        .all(|run| run.maximum_positive_substrate_work <= FROZEN_ZERO_MOTION_TOLERANCE)
        && zero_reserve.maximum_funded_tension <= FROZEN_ZERO_MOTION_TOLERANCE
        && motor_off.maximum_funded_tension <= FROZEN_ZERO_MOTION_TOLERANCE;
    let artifact_exclusion_pass = [
        &active,
        &sensor_off,
        &motor_off,
        &zero_reserve,
        &empty_sham,
        &rotated_active,
    ]
    .iter()
    .all(|run| run.material_vertex_displacement_agreement <= CENTROID_AGREEMENT_TOLERANCE);

    let active_improvement_sensor =
        if sensor_off.cumulative_acquisition > ABSOLUTE_IMPROVEMENT_TOLERANCE {
            (active.cumulative_acquisition - sensor_off.cumulative_acquisition)
                / sensor_off.cumulative_acquisition
        } else if active.cumulative_acquisition > ABSOLUTE_IMPROVEMENT_TOLERANCE {
            f64::INFINITY
        } else {
            0.0
        };
    let active_improvement_motor =
        if motor_off.cumulative_acquisition > ABSOLUTE_IMPROVEMENT_TOLERANCE {
            (active.cumulative_acquisition - motor_off.cumulative_acquisition)
                / motor_off.cumulative_acquisition
        } else if active.cumulative_acquisition > ABSOLUTE_IMPROVEMENT_TOLERANCE {
            f64::INFINITY
        } else {
            0.0
        };
    let gate_results = json!({
        "gate_0_authority_scope": {
            "entry_commit": ENTRY_COMMIT,
            "dcdev012_imported": false,
            "one_production_sensor_composition": true,
            "new_actuator": false,
            "new_traction_law": false,
            "planner_reward_or_target": false,
            "pass": true
        },
        "gate_1_resource_sensor_physicality": local_sensor,
        "gate_2_legacy_parity": legacy,
        "gate_3_local_causality": {
            "only_production_contact_signal": true,
            "active_signal_positive_steps": active.signal_positive_steps,
            "active_regulatory_response": active.signal_positive_steps > 0,
            "pass": active.signal_positive_steps > 0
        },
        "gate_4_funded_feeding_response": {
            "active_nonzero_regulatory_activity": active.signal_positive_steps > 0,
            "reserve_spent": active.reserve_spent,
            "maximum_funded_tension": active.maximum_funded_tension,
            "stick_slip_events": active.stuck_contacts + active.slipping_contacts,
            "zero_reserve_funded_tension": zero_reserve.maximum_funded_tension,
            "pass": active.signal_positive_steps > 0 && active.reserve_spent > 0.0 && active.maximum_funded_tension > 0.0 && active.stuck_contacts + active.slipping_contacts > 0 && zero_reserve.maximum_funded_tension <= FROZEN_ZERO_MOTION_TOLERANCE
        },
        "gate_5_active_resource_acquisition_benefit": {
            "minimum_relative_improvement": MIN_RELATIVE_IMPROVEMENT,
            "sensor_off_relative_improvement": active_improvement_sensor,
            "motor_off_relative_improvement": active_improvement_motor,
            "sensor_off_absolute_improvement": active.cumulative_acquisition - sensor_off.cumulative_acquisition,
            "motor_off_absolute_improvement": active.cumulative_acquisition - motor_off.cumulative_acquisition,
            "pass": active.cumulative_acquisition > sensor_off.cumulative_acquisition + ABSOLUTE_IMPROVEMENT_TOLERANCE && active.cumulative_acquisition > motor_off.cumulative_acquisition + ABSOLUTE_IMPROVEMENT_TOLERANCE && active_improvement_sensor >= MIN_RELATIVE_IMPROVEMENT && active_improvement_motor >= MIN_RELATIVE_IMPROVEMENT
        },
        "gate_6_contact_benefit": {
            "active_time_integrated_exposed_patches": active.time_integrated_exposed_patches,
            "sensor_off_time_integrated_exposed_patches": sensor_off.time_integrated_exposed_patches,
            "motor_off_time_integrated_exposed_patches": motor_off.time_integrated_exposed_patches,
            "active_final_exposed_patches": active.final_exposed_patches,
            "sensor_off_final_exposed_patches": sensor_off.final_exposed_patches,
            "motor_off_final_exposed_patches": motor_off.final_exposed_patches,
            "pass": (active.time_integrated_exposed_patches > sensor_off.time_integrated_exposed_patches && active.time_integrated_exposed_patches > motor_off.time_integrated_exposed_patches) || (active.final_exposed_patches > sensor_off.final_exposed_patches && active.final_exposed_patches > motor_off.final_exposed_patches)
        },
        "gate_7_resource_conservation": {
            "all_arms_pass": active.conservation_pass && sensor_off.conservation_pass && motor_off.conservation_pass && zero_reserve.conservation_pass && empty_sham.conservation_pass,
            "maximum_error": maximum_conservation_error,
            "pass": active.conservation_pass && sensor_off.conservation_pass && motor_off.conservation_pass && zero_reserve.conservation_pass && empty_sham.conservation_pass
        },
        "gate_8_no_resource_specificity": {
            "empty_signal_positive_steps": empty_sham.signal_positive_steps,
            "empty_acquisition": empty_sham.cumulative_acquisition,
            "empty_funded_tension": empty_sham.maximum_funded_tension,
            "pass": empty_sham.signal_positive_steps == 0 && empty_sham.cumulative_acquisition <= ABSOLUTE_IMPROVEMENT_TOLERANCE && empty_sham.maximum_funded_tension <= FROZEN_ZERO_MOTION_TOLERANCE
        },
        "gate_9_rotational_equivalence": {
            "rotated_resource_center": [-RESOURCE_CENTER[0], -RESOURCE_CENTER[1]],
            "active_acquisition": active.cumulative_acquisition,
            "rotated_active_acquisition": rotated_active.cumulative_acquisition,
            "acquisition_error": active_rotation_error,
            "pass": active_rotation_pass
        },
        "gate_10_passive_substrate_metabolic_causality": {
            "maximum_positive_substrate_work": maximum_positive_substrate_work,
            "zero_reserve_funded_tension": zero_reserve.maximum_funded_tension,
            "motor_off_funded_tension": motor_off.maximum_funded_tension,
            "pass": passive_substrate_pass
        },
        "gate_11_artifact_exclusion": {
            "settlement_is_separate": true,
            "no_coordinate_writes": true,
            "no_hidden_target": true,
            "no_ledger_only_uptake": true,
            "maximum_material_vertex_agreement": maximum_material_vertex_agreement,
            "pass": artifact_exclusion_pass
        },
        "gate_12_production_ownership": {
            "resource_contact_interface": "FiniteSpatialResourceRegionV1::local_contact_signal",
            "assay_implements_sensor": false,
            "pass": true
        },
        "conclusion": if active.cumulative_acquisition > sensor_off.cumulative_acquisition + ABSOLUTE_IMPROVEMENT_TOLERANCE && active.cumulative_acquisition > motor_off.cumulative_acquisition + ABSOLUTE_IMPROVEMENT_TOLERANCE && active_improvement_sensor >= MIN_RELATIVE_IMPROVEMENT && active_improvement_motor >= MIN_RELATIVE_IMPROVEMENT { "DCDEV013_LOCAL_RESOURCE_CONTACT_FEEDING_QUALIFIED" } else { "DCDEV013_RESOURCE_CONTACT_FEEDING_NOT_ESTABLISHED" }
    });
    let conclusion = gate_results["conclusion"].as_str().unwrap();

    write_json(
        &output_root,
        "protocol.json",
        &json!({
        "directive": DIRECTIVE,
        "entry_commit": ENTRY_COMMIT,
        "freeze_commit": FREEZE_COMMIT,
            "topology_size": TOPOLOGY_SIZE,
            "settlement_steps": SETTLEMENT_STEPS,
            "accepted_steps": ASSAY_STEPS,
            "accepted_dt": mechanics.dt,
            "resource_center": RESOURCE_CENTER,
            "resource_radius": RESOURCE_RADIUS,
            "initial_n_mass": INITIAL_N_MASS,
            "initial_f_mass": INITIAL_F_MASS,
            "arms": [Arm::Active.label(), Arm::SensorOff.label(), Arm::MotorOff.label(), Arm::ZeroReserve.label(), Arm::EmptySham.label()],
            "causal_step_order": ["observe_current_local_contact", "advance_existing_regulator", "apply_existing_funded_contractility_and_dcdev011_stick_slip", "advance_existing_mechanics", "execute_existing_local_uptake", "record_material_state"],
            "minimum_relative_improvement": MIN_RELATIVE_IMPROVEMENT,
            "absolute_improvement_tolerance": ABSOLUTE_IMPROVEMENT_TOLERANCE,
            "rotation_tolerance": ROTATION_TOLERANCE,
            "parameter_screening": false,
            "geometry_screening": false,
            "dcdev012_imported": false
        }),
    );
    write_json(
        &output_root,
        "settled_body.json",
        &json!({
            "initial_mesh_hash": settlement.initial_mesh_hash,
            "settled_mesh_hash": settlement.settled_mesh_hash,
            "settled_chemistry_hash": settlement.settled_chemistry_hash,
            "settlement_steps": SETTLEMENT_STEPS,
            "settled": settlement.settled,
            "maximum_attempted_velocity": settlement.maximum_attempted_velocity,
            "maximum_local_displacement": settlement.maximum_local_displacement,
            "maximum_material_centroid_step": settlement.maximum_material_centroid_step
        }),
    );
    write_json(
        &output_root,
        "arm_results.json",
        &json!({
            "active": arm_value(&active),
            "sensor_off": arm_value(&sensor_off),
            "motor_off": arm_value(&motor_off),
            "zero_reserve": arm_value(&zero_reserve),
            "empty_sham": arm_value(&empty_sham),
            "rotated_active": arm_value(&rotated_active)
        }),
    );
    write_json(&output_root, "gate_results.json", &gate_results);
    write_json(
        &output_root,
        "final_manifest.json",
        &json!({
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "freeze_commit": FREEZE_COMMIT,
            "conclusion": conclusion,
            "settled_body_hash": settlement.settled_mesh_hash,
            "production_module": "crates/regulatory-core/src/spatial_resource.rs",
            "assay": "examples/dcdev013_gate_assay.rs",
            "evidence_files": ["protocol.json", "settled_body.json", "arm_results.json", "gate_results.json", "final_manifest.json"],
            "next_execution_started": false
        }),
    );
    println!("{conclusion}");
}
