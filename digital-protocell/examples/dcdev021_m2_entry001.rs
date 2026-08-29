//! DC-DEV-021 ENTRY-001: opt-in A-funded contractility feasibility.
//!
//! This assay is deliberately separate from the production selector. It
//! compares the new V4 A-funded adapter with the frozen R-funded oracle and
//! composes it with the existing DC-DEV-011 stick-slip substrate.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use regulatory_core::{
    apply_local_activated_energy_contractility,
    apply_local_activated_energy_contractility_with_stick_slip, apply_local_contractility,
    apply_stick_slip_to_legacy_mechanics, stable_json_hash, ContractilityParamsV1,
    StickSlipTractionParamsV1, FROZEN_ZERO_MOTION_TOLERANCE,
    ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-021-M2-ENTRY-001-ACTIVATED-ENERGY-CONTRACTILITY-FEASIBILITY-001";
const ENTRY_HEAD: &str = "d76481c785e9eec361df3fa0cd03c512b521639c";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ACTIVE_STEPS: usize = 240;
const RELAXATION_STEPS: usize = 240;
const TOTAL_STEPS: usize = ACTIVE_STEPS + RELAXATION_STEPS;
const CENTROID_TOLERANCE: f64 = 1e-8;
const ROTATION_TOLERANCE: f64 = 1e-9;

#[derive(Clone, Copy)]
enum Arm {
    AActiveStickSlip,
    MotorOffStickSlip,
    AActiveNoSubstrate,
    ZeroAStickSlip,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::AActiveStickSlip => "A_ACTIVE_STICK_SLIP",
            Self::MotorOffStickSlip => "MOTOR_OFF_STICK_SLIP",
            Self::AActiveNoSubstrate => "A_ACTIVE_NO_SUBSTRATE",
            Self::ZeroAStickSlip => "ZERO_A_STICK_SLIP",
        }
    }
}

#[derive(Debug, Clone)]
struct ArmResult {
    arm: &'static str,
    initial_material_centroid: [f64; 2],
    final_material_centroid: [f64; 2],
    initial_vertex_centroid: [f64; 2],
    final_vertex_centroid: [f64; 2],
    a_spent: f64,
    w_generated: f64,
    maximum_tension: f64,
    stuck_contacts: usize,
    slipping_contacts: usize,
    substrate_work: f64,
    accepted_steps: usize,
    final_mesh_hash: String,
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
        return mesh.centroid();
    }
    [weighted[0] / total, weighted[1] / total]
}

fn activity(active: bool) -> Vec<f64> {
    if !active {
        return vec![0.0; TOPOLOGY_SIZE];
    }
    (0..TOPOLOGY_SIZE)
        .map(|i| match i {
            0..=4 => 1.0,
            5..=7 => 0.35,
            _ => 0.0,
        })
        .collect()
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
            a: 0.6,
            r: 0.6,
            c: 0.8,
            n: 0.4,
            f: 0.4,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    mesh
}

fn settle(mut mesh: MaterialMesh, mechanics: &MechParams) -> MaterialMesh {
    for _ in 0..SETTLEMENT_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    assert!(mesh.area().is_finite() && mesh.area() > 0.0);
    assert!(mesh.lifecycle_invariants_hold());
    mesh
}

fn run_arm(
    settled: &MaterialMesh,
    arm: Arm,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> ArmResult {
    let mut mesh = settled.clone();
    if matches!(arm, Arm::ZeroAStickSlip) {
        mesh.interior.a = 0.0;
    }
    let initial_a = mesh.interior.a * mesh.area();
    let initial_w = mesh.interior.w * mesh.area();
    let initial_material_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let mut maximum_tension: f64 = 0.0;
    let mut stuck_contacts = 0;
    let mut slipping_contacts = 0;
    let mut substrate_work = 0.0;
    let mut a_spent = 0.0;
    let mut accepted_steps = 0;

    for step in 0..TOTAL_STEPS {
        let input = activity(step < ACTIVE_STEPS);
        match arm {
            Arm::AActiveStickSlip | Arm::ZeroAStickSlip => {
                let ledger = apply_local_activated_energy_contractility_with_stick_slip(
                    &mut mesh,
                    &input,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
                if let Some(contractility) = ledger.contractility {
                    a_spent += contractility.resource_spent;
                    maximum_tension = maximum_tension.max(contractility.maximum_tension);
                }
                stuck_contacts += ledger.stuck_contacts;
                slipping_contacts += ledger.slipping_contacts;
                substrate_work += ledger.substrate_work;
            }
            Arm::MotorOffStickSlip => {
                let ledger =
                    apply_stick_slip_to_legacy_mechanics(&mut mesh, mechanics, traction).unwrap();
                stuck_contacts += ledger.stuck_contacts;
                slipping_contacts += ledger.slipping_contacts;
                substrate_work += ledger.substrate_work;
            }
            Arm::AActiveNoSubstrate => {
                let ledger = apply_local_activated_energy_contractility(
                    &mut mesh,
                    &input,
                    mechanics,
                    contractility,
                )
                .unwrap();
                a_spent += ledger.resource_spent;
                maximum_tension = maximum_tension.max(ledger.maximum_tension);
            }
        }
        accepted_steps += 1;
        assert!(mesh.lifecycle_invariants_hold());
    }

    let final_area = mesh.area();
    let final_a = mesh.interior.a * final_area;
    let final_w = mesh.interior.w * final_area;
    assert!((initial_a - final_a - a_spent).abs() <= 1e-8);
    assert!((final_w - initial_w - a_spent).abs() <= 1e-8);
    ArmResult {
        arm: arm.label(),
        initial_material_centroid,
        final_material_centroid: material_centroid(&mesh),
        initial_vertex_centroid,
        final_vertex_centroid: mesh.centroid(),
        a_spent,
        w_generated: final_w - initial_w,
        maximum_tension,
        stuck_contacts,
        slipping_contacts,
        substrate_work,
        accepted_steps,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
    }
}

fn result_json(result: &ArmResult) -> Value {
    let material_displacement = norm(sub(
        result.final_material_centroid,
        result.initial_material_centroid,
    ));
    let vertex_displacement = norm(sub(
        result.final_vertex_centroid,
        result.initial_vertex_centroid,
    ));
    json!({
        "arm": result.arm,
        "initial_material_centroid": result.initial_material_centroid,
        "final_material_centroid": result.final_material_centroid,
        "initial_vertex_centroid": result.initial_vertex_centroid,
        "final_vertex_centroid": result.final_vertex_centroid,
        "material_centroid_displacement": material_displacement,
        "vertex_centroid_displacement": vertex_displacement,
        "material_vertex_centroid_difference": (material_displacement - vertex_displacement).abs(),
        "a_spent": result.a_spent,
        "w_generated": result.w_generated,
        "maximum_active_tension": result.maximum_tension,
        "stuck_contacts": result.stuck_contacts,
        "slipping_contacts": result.slipping_contacts,
        "substrate_work": result.substrate_work,
        "accepted_steps": result.accepted_steps,
        "final_mesh_hash": result.final_mesh_hash,
    })
}

fn rotate_180(mut mesh: MaterialMesh) -> MaterialMesh {
    for vertex in &mut mesh.vertices {
        *vertex = [-vertex[0], -vertex[1]];
    }
    mesh
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry001"));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settled = settle(seed_mesh(), &mechanics);

    let mut oracle_r = settled.clone();
    let mut oracle_a = settled.clone();
    let oracle_activity = vec![1.0; TOPOLOGY_SIZE];
    let r_ledger = apply_local_contractility(
        &mut oracle_r,
        &oracle_activity,
        &mechanics,
        &contractility,
    )
    .unwrap();
    let a_ledger = apply_local_activated_energy_contractility(
        &mut oracle_a,
        &oracle_activity,
        &mechanics,
        &contractility,
    )
    .unwrap();
    let oracle_vertex_parity = oracle_r.vertices == oracle_a.vertices;
    let oracle_resource_parity =
        (r_ledger.resource_spent - a_ledger.resource_spent).abs() <= 1e-12;

    let mut zero_a = settled.clone();
    zero_a.interior.a = 0.0;
    let mut zero_a_control = zero_a.clone();
    assert!(mechanics_step(&mut zero_a_control, &mechanics));
    let zero_a_ledger = apply_local_activated_energy_contractility(
        &mut zero_a,
        &oracle_activity,
        &mechanics,
        &contractility,
    )
    .unwrap();
    let zero_a_geometry_parity = zero_a.vertices == zero_a_control.vertices;
    let zero_a_passive_parity = zero_a_ledger.resource_spent == 0.0
        && zero_a_ledger.maximum_tension == 0.0
        && zero_a.interior.w == zero_a_control.interior.w;

    let mut zero_activity = settled.clone();
    let mut zero_activity_control = zero_activity.clone();
    assert!(mechanics_step(&mut zero_activity_control, &mechanics));
    let zero_activity_ledger = apply_local_activated_energy_contractility(
        &mut zero_activity,
        &vec![0.0; TOPOLOGY_SIZE],
        &mechanics,
        &contractility,
    )
    .unwrap();
    let zero_activity_parity = zero_activity.vertices == zero_activity_control.vertices
        && zero_activity_ledger.resource_spent == 0.0;

    let active = run_arm(
        &settled,
        Arm::AActiveStickSlip,
        &mechanics,
        &contractility,
        &traction,
    );
    let motor_off = run_arm(
        &settled,
        Arm::MotorOffStickSlip,
        &mechanics,
        &contractility,
        &traction,
    );
    let no_substrate = run_arm(
        &settled,
        Arm::AActiveNoSubstrate,
        &mechanics,
        &contractility,
        &traction,
    );
    let zero_a_stick = run_arm(
        &settled,
        Arm::ZeroAStickSlip,
        &mechanics,
        &contractility,
        &traction,
    );
    let rotated = run_arm(
        &rotate_180(settled.clone()),
        Arm::AActiveStickSlip,
        &mechanics,
        &contractility,
        &traction,
    );

    let active_displacement = sub(active.final_material_centroid, active.initial_material_centroid);
    let rotated_displacement = sub(rotated.final_material_centroid, rotated.initial_material_centroid);
    let rotational_error = norm([
        active_displacement[0] + rotated_displacement[0],
        active_displacement[1] + rotated_displacement[1],
    ]);
    let active_distance = norm(active_displacement);
    let motor_distance = norm(sub(motor_off.final_material_centroid, motor_off.initial_material_centroid));
    let no_substrate_distance = norm(sub(no_substrate.final_material_centroid, no_substrate.initial_material_centroid));
    let zero_a_distance = norm(sub(zero_a_stick.final_material_centroid, zero_a_stick.initial_material_centroid));
    let active_centroid_agreement = (active_distance
        - norm(sub(active.final_vertex_centroid, active.initial_vertex_centroid)))
        .abs();
    let active_displacement_benefit = active_distance > motor_distance + FROZEN_ZERO_MOTION_TOLERANCE
        && active_distance > no_substrate_distance + FROZEN_ZERO_MOTION_TOLERANCE;
    let zero_a_stick_slip_parity = zero_a_distance <= FROZEN_ZERO_MOTION_TOLERANCE
        && zero_a_stick.a_spent == 0.0
        && zero_a_stick.maximum_tension == 0.0;
    let material_closure = active.a_spent > 0.0
        && (active.w_generated - active.a_spent).abs() <= 1e-8
        && active.substrate_work <= FROZEN_ZERO_MOTION_TOLERANCE;
    let scientific_pass = oracle_vertex_parity
        && oracle_resource_parity
        && zero_a_geometry_parity
        && zero_a_passive_parity
        && zero_activity_parity
        && active_displacement_benefit
        && active.a_spent > 0.0
        && active.w_generated > 0.0
        && active.maximum_tension > 0.0
        && active.slipping_contacts > 0
        && active_centroid_agreement <= CENTROID_TOLERANCE
        && rotational_error <= ROTATION_TOLERANCE
        && zero_a_stick_slip_parity
        && material_closure;

    let mut arms = BTreeMap::new();
    for result in [&active, &motor_off, &no_substrate, &zero_a_stick] {
        arms.insert(result.arm.to_string(), result_json(result));
    }
    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE,
            "entry_head": ENTRY_HEAD,
            "schema": ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1,
            "execution": "bounded_opt_in_feasibility_assay",
            "settlement_steps": SETTLEMENT_STEPS,
            "active_steps": ACTIVE_STEPS,
            "relaxation_steps": RELAXATION_STEPS,
            "topology_size": TOPOLOGY_SIZE,
            "resource_acquisition": false,
            "parameter_search": false,
            "production_invocation": false,
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "funding_semantics.json",
        &json!({
            "schema": ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1,
            "funding_pool": "interior_A_absolute_amount",
            "cost_parameter_source": "FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME",
            "maximum_tension_source": "FROZEN_MAX_ACTIVE_TENSION",
            "a_to_w": true,
            "r_changed": false,
            "area_semantics": "actual_positive_v4_area",
            "direct_coordinate_writes": false,
            "observer_feedback": false,
            "target_or_gradient": false
        }),
    );
    write_json(
        &output,
        "r_funded_oracle_comparison.json",
        &json!({
            "r_funded_schema": regulatory_core::CONTRACTILITY_SCHEMA_V1,
            "a_funded_schema": ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1,
            "same_funded_tension": (r_ledger.maximum_tension - a_ledger.maximum_tension).abs() <= 1e-12,
            "same_resource_spent": oracle_resource_parity,
            "same_vertices": oracle_vertex_parity,
            "r_resource_spent": r_ledger.resource_spent,
            "a_resource_spent": a_ledger.resource_spent,
            "r_maximum_tension": r_ledger.maximum_tension,
            "a_maximum_tension": a_ledger.maximum_tension
        }),
    );
    write_json(
        &output,
        "zero_a_parity.json",
        &json!({
            "passive_geometry_parity": zero_a_geometry_parity,
            "passive_material_parity": zero_a_passive_parity,
            "maximum_tension": zero_a_ledger.maximum_tension,
            "resource_spent": zero_a_ledger.resource_spent
        }),
    );
    write_json(
        &output,
        "zero_activity_parity.json",
        &json!({ "passive_geometry_parity": zero_activity_parity, "resource_spent": zero_activity_ledger.resource_spent }),
    );
    write_json(
        &output,
        "stick_slip_feasibility.json",
        &json!({
            "arms": Value::Object(arms.into_iter().collect()),
            "active_displacement": active_distance,
            "motor_off_displacement": motor_distance,
            "active_no_substrate_displacement": no_substrate_distance,
            "zero_a_displacement": zero_a_distance,
            "active_displacement_benefit": active_displacement_benefit,
            "material_vertex_centroid_agreement": active_centroid_agreement,
            "pass": scientific_pass
        }),
    );
    write_json(
        &output,
        "rotation_check.json",
        &json!({
            "rotation": "180_degrees",
            "displacement": active_displacement,
            "rotated_displacement": rotated_displacement,
            "error": rotational_error,
            "tolerance": ROTATION_TOLERANCE,
            "pass": rotational_error <= ROTATION_TOLERANCE
        }),
    );
    write_json(
        &output,
        "material_closure.json",
        &json!({
            "a_spent": active.a_spent,
            "w_generated": active.w_generated,
            "residual": active.w_generated - active.a_spent,
            "pass": material_closure
        }),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({
            "production_contract": "MaturationCoupledV4",
            "production_reserve": false,
            "production_invocation": false,
            "scientific_core_changed": "regulatory-core only"
        }),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({
            "dcdev002_regulator": "REQUIRES_SCOPED_CI",
            "dcdev005_plasticity": "REQUIRES_SCOPED_CI",
            "dcdev006_contact": "REQUIRES_SCOPED_CI",
            "dcdev007_contact_regulation": "REQUIRES_SCOPED_CI",
            "dcdev008_finite_resource": "REQUIRES_SCOPED_CI",
            "dcdev011_historical_r_funded_traction": "REQUIRES_SCOPED_CI",
            "d088": "REQUIRES_SCOPED_CI",
            "d091": "REQUIRES_SCOPED_CI",
            "dcdev013_historical_classification_changed": false
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "directive": DIRECTIVE,
            "classification": if scientific_pass { "M2_ACTIVATED_ENERGY_CONTRACTILITY_FEASIBILITY_QUALIFIED" } else { "M2_ACTIVATED_ENERGY_CONTRACTILITY_FEASIBILITY_INSUFFICIENT" },
            "m2_actuator": if scientific_pass { "QUALIFIED" } else { "FAIL" },
            "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "production_default_changed": false,
            "reserve_enabled": false,
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({
            "directive": DIRECTIVE,
            "entry_head": ENTRY_HEAD,
            "schema": ACTIVATED_ENERGY_CONTRACTILITY_SCHEMA_V1,
            "files": ["protocol.json", "funding_semantics.json", "r_funded_oracle_comparison.json", "zero_a_parity.json", "zero_activity_parity.json", "stick_slip_feasibility.json", "rotation_check.json", "material_closure.json", "m1_preservation.json", "downstream_preservation.json", "qualification.json"],
            "dense_evidence_external": true,
            "scientific_pass": scientific_pass
        }),
    );
}
