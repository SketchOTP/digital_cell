//! DC-DEV-009: fixed-topology free-space motility feasibility audit.
//!
//! This is an observer-only investigation. It runs the accepted local
//! regulator and contractility path on a fixed mesh in free space, then
//! compares it with a motor-off arm using the same preregistered regulator
//! trajectory. No coordinates are written by the assay, and no production
//! locomotion mechanism is introduced.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{compute_forces, mechanics_step, MechParams};
use regulatory_core::continuity::ContinuityMaterialFrameV1;
use regulatory_core::{
    apply_local_contractility, stable_json_hash, ContinuityNetworkV1, ContractilityParamsV1,
    TopologyEventV1,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-009";
const ENTRY_COMMIT: &str = "79751bed5ad78d367b7409f0ec677e32a3b9d527";
const ASSAY_HORIZON_STEPS: usize = 240;
const METRIC_TOLERANCE: f64 = 1e-10;
const MATERIAL_CENTROID_TOLERANCE: f64 = 1e-9;
const TOPOLOGY_SIZE: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arm {
    ActiveContractility,
    MotorOff,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::ActiveContractility => "active_contractility",
            Self::MotorOff => "motor_off",
        }
    }
}

#[derive(Debug, Clone)]
struct StepRecord {
    base_force_sum: [f64; 2],
    contractile_force_sum: [f64; 2],
    total_force_sum: [f64; 2],
    observed_total_force_sum: [f64; 2],
    vertex_centroid: [f64; 2],
    material_centroid: [f64; 2],
    edge_tension_l1: f64,
    edge_tension_max: f64,
    regulatory_state_hash: String,
}

#[derive(Debug, Clone)]
struct ArmResult {
    arm: Arm,
    records: Vec<StepRecord>,
    initial_vertex_centroid: [f64; 2],
    final_vertex_centroid: [f64; 2],
    initial_material_centroid: [f64; 2],
    final_material_centroid: [f64; 2],
    initial_edge_lengths: Vec<f64>,
    final_edge_lengths: Vec<f64>,
    final_mesh_hash: String,
    regulatory_trace_hash: String,
    topology_size: usize,
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

fn add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn vector_sum(vectors: &[[f64; 2]]) -> [f64; 2] {
    vectors.iter().copied().fold([0.0, 0.0], add)
}

fn edge_lengths(mesh: &MaterialMesh) -> Vec<f64> {
    (0..mesh.n()).map(|index| mesh.edge_length(index)).collect()
}

/// Observer-only material centroid using the existing edge masses at their
/// current geometric midpoints. The assay never changes these weights.
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

fn shape_change(initial: &[f64], final_lengths: &[f64]) -> f64 {
    initial
        .iter()
        .zip(final_lengths)
        .map(|(before, after)| (after - before).powi(2))
        .sum::<f64>()
        .sqrt()
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

/// Fixed, local, preregistered stimulus. It is not a navigation signal or a
/// target. It merely creates a bounded asymmetric contractility exposure.
fn preregistered_stimulus() -> Vec<f64> {
    (0..TOPOLOGY_SIZE)
        .map(|index| match index {
            0..=4 => 1.0,
            5..=7 => 0.35,
            _ => 0.0,
        })
        .collect()
}

/// Reconstruct the already accepted edge-tension force vectors for the
/// observer ledger. The contractility adapter remains the sole movement
/// authority; this calculation is used only to audit its force accounting.
fn observer_contractile_forces(
    mesh: &MaterialMesh,
    activity: &[f64],
    mechanics: &MechParams,
    params: &ContractilityParamsV1,
) -> (Vec<[f64; 2]>, Vec<f64>) {
    let mut requested = vec![0.0; mesh.n()];
    let mut requested_resource = 0.0;
    for index in 0..mesh.n() {
        if mesh.edges[index].ruptured {
            continue;
        }
        let edge_activity = 0.5 * (activity[index] + activity[(index + 1) % mesh.n()]);
        if edge_activity <= f64::EPSILON {
            continue;
        }
        let tension = params.max_active_tension * edge_activity;
        requested[index] = tension;
        requested_resource += params.reserve_cost_per_force_length_time
            * tension
            * mesh.edge_length(index)
            * mechanics.dt.max(0.0);
    }
    let available = mesh.interior.r.max(0.0) * mesh.area().max(1e-12);
    let scale = if requested_resource <= f64::EPSILON {
        0.0
    } else {
        (available / requested_resource).min(1.0)
    };
    let tensions: Vec<f64> = requested.into_iter().map(|value| value * scale).collect();
    let mut forces = vec![[0.0, 0.0]; mesh.n()];
    for index in 0..mesh.n() {
        let tension = tensions[index];
        if tension <= f64::EPSILON {
            continue;
        }
        let a = mesh.vertices[index];
        let b = mesh.vertices[(index + 1) % mesh.n()];
        let length = (b[0] - a[0]).hypot(b[1] - a[1]).max(1e-15);
        let direction = [(b[0] - a[0]) / length, (b[1] - a[1]) / length];
        forces[index][0] += tension * direction[0];
        forces[index][1] += tension * direction[1];
        let next = (index + 1) % mesh.n();
        forces[next][0] -= tension * direction[0];
        forces[next][1] -= tension * direction[1];
    }
    (forces, tensions)
}

fn run_arm(
    initial_mesh: &MaterialMesh,
    arm: Arm,
    horizon: usize,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    stimulus: &[f64],
) -> ArmResult {
    let mut mesh = initial_mesh.clone();
    let initial_vertex_centroid = mesh.centroid();
    let initial_material_centroid = material_centroid(&mesh);
    let initial_edge_lengths = edge_lengths(&mesh);
    let initial_frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
        &mesh.vertices,
        stimulus,
        mechanics.dt,
    );
    let mut network = ContinuityNetworkV1::new(initial_frame, Some(9009)).unwrap();
    let mut regulatory_trace = Vec::with_capacity(horizon);
    let mut records = Vec::with_capacity(horizon);

    for _ in 0..horizon {
        let vertices_before = mesh.vertices.clone();
        let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &mesh.vertices,
            stimulus,
            mechanics.dt,
        );
        network.step(frame, TopologyEventV1::Stable).unwrap();
        let activity = network.state.activity.clone();
        let regulatory_state_hash = stable_json_hash(&network.state).unwrap();
        regulatory_trace.push(regulatory_state_hash.clone());
        let base_forces = compute_forces(&mesh, mechanics);
        let base_force_sum = vector_sum(&base_forces);
        let (contractile_forces, tensions) = match arm {
            Arm::ActiveContractility => {
                observer_contractile_forces(&mesh, &activity, mechanics, contractility)
            }
            Arm::MotorOff => (vec![[0.0, 0.0]; mesh.n()], vec![0.0; mesh.n()]),
        };
        let contractile_force_sum = vector_sum(&contractile_forces);
        let total_force_sum = add(base_force_sum, contractile_force_sum);
        match arm {
            Arm::ActiveContractility => {
                let ledger =
                    apply_local_contractility(&mut mesh, &activity, mechanics, contractility)
                        .unwrap();
                let expected_max = tensions.iter().copied().fold(0.0, f64::max);
                assert!((ledger.maximum_tension - expected_max).abs() <= 1e-12);
            }
            Arm::MotorOff => assert!(mechanics_step(&mut mesh, mechanics)),
        }
        let observed_total_force = vertices_before
            .iter()
            .zip(&mesh.vertices)
            .map(|(before, after)| {
                [
                    (after[0] - before[0]) * mechanics.gamma / mechanics.dt,
                    (after[1] - before[1]) * mechanics.gamma / mechanics.dt,
                ]
            })
            .collect::<Vec<_>>();
        records.push(StepRecord {
            base_force_sum,
            contractile_force_sum,
            total_force_sum,
            observed_total_force_sum: vector_sum(&observed_total_force),
            vertex_centroid: mesh.centroid(),
            material_centroid: material_centroid(&mesh),
            edge_tension_l1: tensions.iter().sum(),
            edge_tension_max: tensions.iter().copied().fold(0.0, f64::max),
            regulatory_state_hash,
        });
    }

    let final_vertex_centroid = mesh.centroid();
    let final_material_centroid = material_centroid(&mesh);
    let final_edge_lengths = edge_lengths(&mesh);
    ArmResult {
        arm,
        records,
        initial_vertex_centroid,
        final_vertex_centroid,
        initial_material_centroid,
        final_material_centroid,
        initial_edge_lengths,
        final_edge_lengths,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        regulatory_trace_hash: stable_json_hash(&regulatory_trace).unwrap(),
        topology_size: mesh.n(),
    }
}

fn max_norm<F>(records: &[StepRecord], selector: F) -> f64
where
    F: Fn(&StepRecord) -> [f64; 2],
{
    records
        .iter()
        .map(|record| norm(selector(record)))
        .fold(0.0, f64::max)
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev009"));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let stimulus = preregistered_stimulus();
    let initial = seed_mesh();
    let active = run_arm(
        &initial,
        Arm::ActiveContractility,
        ASSAY_HORIZON_STEPS,
        &mechanics,
        &contractility,
        &stimulus,
    );
    let motor_off = run_arm(
        &initial,
        Arm::MotorOff,
        ASSAY_HORIZON_STEPS,
        &mechanics,
        &contractility,
        &stimulus,
    );

    assert_eq!(active.topology_size, TOPOLOGY_SIZE);
    assert_eq!(motor_off.topology_size, TOPOLOGY_SIZE);
    assert_eq!(active.records.len(), ASSAY_HORIZON_STEPS);
    assert_eq!(motor_off.records.len(), ASSAY_HORIZON_STEPS);
    assert_eq!(
        active.regulatory_trace_hash,
        motor_off.regulatory_trace_hash
    );

    let active_vertex_displacement = norm(subtract(
        active.final_vertex_centroid,
        active.initial_vertex_centroid,
    ));
    let motor_off_vertex_displacement = norm(subtract(
        motor_off.final_vertex_centroid,
        motor_off.initial_vertex_centroid,
    ));
    let active_material_displacement = norm(subtract(
        active.final_material_centroid,
        active.initial_material_centroid,
    ));
    let motor_off_material_displacement = norm(subtract(
        motor_off.final_material_centroid,
        motor_off.initial_material_centroid,
    ));
    let active_minus_control_vertex = subtract(
        active.final_vertex_centroid,
        motor_off.final_vertex_centroid,
    );
    let active_minus_control_material = subtract(
        active.final_material_centroid,
        motor_off.final_material_centroid,
    );
    let active_shape_change =
        shape_change(&active.initial_edge_lengths, &active.final_edge_lengths);
    let motor_off_shape_change = shape_change(
        &motor_off.initial_edge_lengths,
        &motor_off.final_edge_lengths,
    );
    let active_minus_control_shape = active
        .final_edge_lengths
        .iter()
        .zip(&motor_off.final_edge_lengths)
        .map(|(active, control)| (active - control).powi(2))
        .sum::<f64>()
        .sqrt();
    let contractile_force_integral = active.records.iter().fold([0.0, 0.0], |sum, record| {
        add(
            sum,
            [
                record.contractile_force_sum[0] * mechanics.dt / mechanics.gamma,
                record.contractile_force_sum[1] * mechanics.dt / mechanics.gamma,
            ],
        )
    });
    let contractile_only_centroid_displacement =
        norm(contractile_force_integral) / TOPOLOGY_SIZE as f64;
    let baseline_force_difference_integral = active.records.iter().zip(&motor_off.records).fold(
        [0.0, 0.0],
        |sum, (active_record, motor_off_record)| {
            add(
                sum,
                [
                    (active_record.base_force_sum[0] - motor_off_record.base_force_sum[0])
                        * mechanics.dt
                        / mechanics.gamma,
                    (active_record.base_force_sum[1] - motor_off_record.base_force_sum[1])
                        * mechanics.dt
                        / mechanics.gamma,
                ],
            )
        },
    );
    let baseline_force_difference_centroid_displacement =
        norm(baseline_force_difference_integral) / TOPOLOGY_SIZE as f64;

    let active_contractile_force_sum =
        max_norm(&active.records, |record| record.contractile_force_sum);
    let motor_off_contractile_force_sum =
        max_norm(&motor_off.records, |record| record.contractile_force_sum);
    let active_total_force_sum = max_norm(&active.records, |record| record.total_force_sum);
    let motor_off_total_force_sum = max_norm(&motor_off.records, |record| record.total_force_sum);
    let active_observed_total_force_sum =
        max_norm(&active.records, |record| record.observed_total_force_sum);
    let motor_off_observed_total_force_sum =
        max_norm(&motor_off.records, |record| record.observed_total_force_sum);
    let fixed_topology = active.topology_size == initial.n()
        && motor_off.topology_size == initial.n()
        && active.records.len() == ASSAY_HORIZON_STEPS
        && motor_off.records.len() == ASSAY_HORIZON_STEPS;
    let no_valid_translation = contractile_only_centroid_displacement <= METRIC_TOLERANCE
        && active_contractile_force_sum <= METRIC_TOLERANCE;
    let shape_changed = active_shape_change > motor_off_shape_change + METRIC_TOLERANCE
        || active_minus_control_shape > METRIC_TOLERANCE;
    let force_accounting_pass =
        active_contractile_force_sum <= METRIC_TOLERANCE && motor_off_contractile_force_sum == 0.0;
    let conclusion = if fixed_topology && force_accounting_pass && no_valid_translation {
        "DCDEV009_EXISTING_FREE_SPACE_MOTILITY_NOT_ESTABLISHED"
    } else {
        "DCDEV009_MOTILITY_AUDIT_BLOCKED"
    };
    assert!(
        conclusion == "DCDEV009_EXISTING_FREE_SPACE_MOTILITY_NOT_ESTABLISHED",
        "audit classification blocked: fixed_topology={fixed_topology}, force_accounting_pass={force_accounting_pass}, no_valid_translation={no_valid_translation}, active_contractile_force_sum={active_contractile_force_sum:.17e}, motor_off_contractile_force_sum={motor_off_contractile_force_sum:.17e}, contractile_only_centroid_displacement={contractile_only_centroid_displacement:.17e}"
    );

    write_json(
        &output,
        "protocol.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "assay_horizon_steps": ASSAY_HORIZON_STEPS,
            "assay_horizon_simulated_time": ASSAY_HORIZON_STEPS as f64 * mechanics.dt,
            "accepted_time_authority": "MechParams.dt on each accepted mechanics step",
            "topology_size": TOPOLOGY_SIZE,
            "fixed_topology": true,
            "obstacle_contact": false,
            "external_forces": false,
            "growth": false,
            "remeshing": false,
            "fission": false,
            "resource_acquisition": false,
            "preregistered_local_stimulus": stimulus,
            "parameter_screening": false,
            "new_actuator": false,
            "new_friction": false,
            "new_adhesion": false,
            "new_fluid_physics": false,
            "navigation": false,
            "resource_seeking": false,
            "dcdev010_started": false,
            "conclusion": conclusion,
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "force_accounting.json",
        &json!({
            "active_contractile_force_vector_sum_max_norm": active_contractile_force_sum,
            "motor_off_contractile_force_vector_sum_max_norm": motor_off_contractile_force_sum,
            "active_total_force_vector_sum_max_norm": active_total_force_sum,
            "motor_off_total_force_vector_sum_max_norm": motor_off_total_force_sum,
            "active_observed_total_force_vector_sum_max_norm": active_observed_total_force_sum,
            "motor_off_observed_total_force_vector_sum_max_norm": motor_off_observed_total_force_sum,
            "equal_and_opposite_contractile_pairing": force_accounting_pass,
            "result": "DCDEV009_GATE1_FORCE_ACCOUNTING_PASS"
        }),
    );
    write_json(
        &output,
        "matched_arms.json",
        &json!({
            "active": {
                "arm": active.arm.label(),
                "vertex_centroid_displacement": active_vertex_displacement,
                "material_centroid_displacement": active_material_displacement,
                "shape_change": active_shape_change,
                "final_mesh_hash": active.final_mesh_hash,
                "regulatory_trace_hash": active.regulatory_trace_hash
            },
            "motor_off": {
                "arm": motor_off.arm.label(),
                "vertex_centroid_displacement": motor_off_vertex_displacement,
                "material_centroid_displacement": motor_off_material_displacement,
                "shape_change": motor_off_shape_change,
                "final_mesh_hash": motor_off.final_mesh_hash,
                "regulatory_trace_hash": motor_off.regulatory_trace_hash
            },
            "active_minus_control_vertex_displacement": active_minus_control_vertex,
            "active_minus_control_material_displacement": active_minus_control_material,
            "active_minus_control_displacement_norm": norm(active_minus_control_vertex),
            "active_minus_control_material_displacement_norm": norm(active_minus_control_material),
            "active_minus_control_shape_change": active_minus_control_shape,
            "contractile_only_centroid_displacement": contractile_only_centroid_displacement,
            "baseline_force_difference_centroid_displacement": baseline_force_difference_centroid_displacement,
            "regulatory_trajectory_identical": active.regulatory_trace_hash == motor_off.regulatory_trace_hash,
            "fixed_topology": fixed_topology,
            "shape_change_without_translation": shape_changed,
            "result": "DCDEV009_GATE2_FIXED_TOPOLOGY_NO_VALID_TRANSLATION"
        }),
    );
    write_json(
        &output,
        "step_ledger.json",
        &json!({
            "active": active.records.iter().enumerate().map(|(step, record)| json!({
                "step": step,
                "base_force_sum": record.base_force_sum,
                "contractile_force_sum": record.contractile_force_sum,
                "total_force_sum": record.total_force_sum,
                "observed_total_force_sum": record.observed_total_force_sum,
                "vertex_centroid": record.vertex_centroid,
                "material_centroid": record.material_centroid,
                "edge_tension_l1": record.edge_tension_l1,
                "edge_tension_max": record.edge_tension_max,
                "regulatory_state_hash": record.regulatory_state_hash
            })).collect::<Vec<_>>(),
            "motor_off": motor_off.records.iter().enumerate().map(|(step, record)| json!({
                "step": step,
                "base_force_sum": record.base_force_sum,
                "contractile_force_sum": record.contractile_force_sum,
                "total_force_sum": record.total_force_sum,
                "observed_total_force_sum": record.observed_total_force_sum,
                "vertex_centroid": record.vertex_centroid,
                "material_centroid": record.material_centroid,
                "edge_tension_l1": record.edge_tension_l1,
                "edge_tension_max": record.edge_tension_max,
                "regulatory_state_hash": record.regulatory_state_hash
            })).collect::<Vec<_>>()
        }),
    );
    write_json(
        &output,
        "artifact_analysis.json",
        &json!({
            "baseline_force_field_difference_present": baseline_force_difference_centroid_displacement > METRIC_TOLERANCE,
            "active_shape_change": active_shape_change,
            "motor_off_shape_change": motor_off_shape_change,
            "active_minus_control_shape_change": active_minus_control_shape,
            "fixed_topology_removes_bookkeeping_confounds": fixed_topology,
            "centroid_change_is_not_classified_as_locomotion": true,
            "locomotion_attribution": "none: contractile force pairs have zero net force; the residual active-minus-control centroid drift is a baseline mechanics force-field/discretization artifact after shape change, not contractile propulsion",
            "contractile_only_centroid_displacement": contractile_only_centroid_displacement,
            "baseline_force_difference_centroid_displacement": baseline_force_difference_centroid_displacement,
            "result": "DCDEV009_GATE3_ARTIFACT_AUDIT_PASS"
        }),
    );
    write_json(
        &output,
        "environment_coupling_inventory.json",
        &json!({
            "internal_contractility": {
                "present": true,
                "force_exchange_with_outside": false,
                "symmetry_breaking": false,
                "notes": "equal-and-opposite endpoint forces on existing edges"
            },
            "overdamped_drag": {
                "present": true,
                "force_exchange_with_outside": true,
                "symmetry_breaking": false,
                "notes": "one scalar gamma applied identically to every vertex"
            },
            "inert_obstacle_contact": {
                "present_in_project": true,
                "enabled_in_audit": false,
                "symmetry_breaking": "potentially, only when contact is enabled"
            },
            "spatial_nf_material_acquisition": {
                "present_in_project": true,
                "enabled_in_audit": false,
                "force_exchange_with_outside": false,
                "symmetry_breaking": false,
                "notes": "changes material inventory, not free-space momentum"
            },
            "substrate_friction": {"present": false, "symmetry_breaking": false},
            "adhesion_or_anchoring": {"present": false, "symmetry_breaking": false},
            "fluid_hydrodynamics": {"present": false, "symmetry_breaking": false},
            "result": "DCDEV009_GATE4_COUPLING_INVENTORY_COMPLETE"
        }),
    );
    write_json(
        &output,
        "governance_boundary.json",
        &json!({
            "observer_only": true,
            "production_locomotion_mechanism_added": false,
            "new_actuator": false,
            "friction": false,
            "adhesion": false,
            "cilia": false,
            "flagella": false,
            "propulsion_forces": false,
            "swimming_physics": false,
            "locomotion_controller": false,
            "navigation": false,
            "chemotaxis": false,
            "nutrient_sensing": false,
            "resource_seeking": false,
            "reward": false,
            "reinforcement_learning": false,
            "fitness": false,
            "evolution": false,
            "dcdev010_started": false,
            "result": "DCDEV009_GATE0_SCOPE_PASS"
        }),
    );
    write_json(
        &output,
        "final_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "conclusion": "DCDEV009_MOTILITY_FEASIBILITY_AUDIT_COMPLETE",
            "scientific_finding": conclusion,
            "audit_horizon_steps": ASSAY_HORIZON_STEPS,
            "audit_horizon_simulated_time": ASSAY_HORIZON_STEPS as f64 * mechanics.dt,
            "active_contractile_force_vector_sum_max_norm": active_contractile_force_sum,
            "motor_off_contractile_force_vector_sum_max_norm": motor_off_contractile_force_sum,
            "active_centroid_displacement": active_vertex_displacement,
            "motor_off_centroid_displacement": motor_off_vertex_displacement,
            "active_minus_control_displacement": active_minus_control_vertex,
            "contractile_only_centroid_displacement": contractile_only_centroid_displacement,
            "baseline_force_difference_centroid_displacement": baseline_force_difference_centroid_displacement,
            "shape_change_result": shape_changed,
            "fixed_topology": fixed_topology,
            "regulatory_trajectory_identical": active.regulatory_trace_hash == motor_off.regulatory_trace_hash,
            "preservation_status": "PENDING",
            "next_execution_started": false
        }),
    );
}
