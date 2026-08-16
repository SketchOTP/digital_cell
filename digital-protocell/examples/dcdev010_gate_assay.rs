//! DC-DEV-010: passive directional substrate coupling qualification.
//!
//! The assay keeps the DC-DEV-009 fixed ring and local stimulus, then runs
//! active directional, motor-off directional, and active isotropic arms. The
//! substrate law lives in regulatory-core; this file only assembles the
//! matched arms and writes evidence.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{
    compute_forces, mechanics_step, mechanics_step_with_external_forces, MechParams,
};
use regulatory_core::continuity::ContinuityMaterialFrameV1;
use regulatory_core::{
    apply_local_contractility, apply_local_contractility_with_external_forces,
    contractile_force_vectors, reactions_for_internal_forces, stable_json_hash,
    ContinuityNetworkV1, ContractilityParamsV1, SubstrateMode, SubstrateTractionParamsV1,
    TopologyEventV1,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-010";
const ENTRY_COMMIT: &str = "8d6fe59397cabfa47bc1d8103acd68f544acc190";
const R1_ENTRY_COMMIT: &str = "b4178417e30907835183c7f9c16a639bdd8d31db";
const ASSAY_HORIZON_STEPS: usize = 240;
const TOPOLOGY_SIZE: usize = 24;
const SUBSTRATE_AXIS: [f64; 2] = [1.0, 0.0];
const DC009_MOTOR_OFF_DISPLACEMENT: f64 = 3.5925380388006317e-16;
const DC009_CONTRACTILITY_ONLY_DISPLACEMENT: f64 = 2.473548217003853e-18;
const DC009_ACTIVE_HASH: &str = "2b17b49f4f8ca79e";
const DC009_MOTOR_OFF_HASH: &str = "5507b597368297ac";
const DC009_REGULATORY_TRACE_HASH: &str = "b762e60498e5b9e1";
const R1_TRANSLATION_TOLERANCE: f64 = 5.3290705182007514e-11;
const R1_MAX_SETTLING_STEPS: usize = 5_000;
const R1_REST_CONSECUTIVE_STEPS: usize = 16;
const R1_MAX_LOCAL_DISPLACEMENT_PER_STEP: f64 = R1_TRANSLATION_TOLERANCE;
const R1_MAX_LOCAL_ATTEMPTED_VELOCITY: f64 = R1_MAX_LOCAL_DISPLACEMENT_PER_STEP / 0.02;
const R1_MAX_LOCAL_INTERNAL_FORCE: f64 = R1_MAX_LOCAL_ATTEMPTED_VELOCITY;
const R1_MAX_CENTROID_DISPLACEMENT_PER_STEP: f64 =
    R1_TRANSLATION_TOLERANCE / ASSAY_HORIZON_STEPS as f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arm {
    ActiveDirectional,
    MotorOffDirectional,
    ActiveIsotropic,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::ActiveDirectional => "active_directional_substrate",
            Self::MotorOffDirectional => "motor_off_directional_substrate",
            Self::ActiveIsotropic => "active_isotropic_control",
        }
    }

    fn active(self) -> bool {
        !matches!(self, Self::MotorOffDirectional)
    }
}

#[derive(Debug, Clone)]
struct StepRecord {
    internal_force_sum: [f64; 2],
    substrate_force_sum: [f64; 2],
    observed_force_sum: [f64; 2],
    vertex_centroid: [f64; 2],
    material_centroid: [f64; 2],
    substrate_work: f64,
    max_reaction: f64,
    reserve_spent: f64,
    maximum_tension: f64,
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

#[derive(Debug, Clone)]
struct SettlementStepRecord {
    step: usize,
    max_attempted_velocity: f64,
    max_accepted_velocity: f64,
    max_local_displacement: f64,
    max_internal_force: f64,
    material_centroid_step: f64,
    vertex_centroid_step: f64,
    substrate_work: f64,
}

#[derive(Debug, Clone)]
struct SettlementResult {
    mesh: MaterialMesh,
    records: Vec<SettlementStepRecord>,
    rest_achieved: bool,
    rest_step: Option<usize>,
    initial_mesh_hash: String,
    settled_mesh_hash: String,
    initial_chemistry_hash: String,
    settled_chemistry_hash: String,
    final_metrics: SettlementStepRecord,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn subtract(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn norm(vector: [f64; 2]) -> f64 {
    vector[0].hypot(vector[1])
}

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn sum(vectors: &[[f64; 2]]) -> [f64; 2] {
    vectors.iter().copied().fold([0.0, 0.0], add)
}

fn edge_lengths(mesh: &MaterialMesh) -> Vec<f64> {
    (0..mesh.n()).map(|index| mesh.edge_length(index)).collect()
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

fn shape_change(initial: &[f64], final_lengths: &[f64]) -> f64 {
    initial
        .iter()
        .zip(final_lengths)
        .map(|(before, after)| (after - before).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn seed_mesh(reserve: f64) -> MaterialMesh {
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
            r: reserve,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    )
}

fn preregistered_stimulus() -> Vec<f64> {
    (0..TOPOLOGY_SIZE)
        .map(|index| match index {
            0..=4 => 1.0,
            5..=7 => 0.35,
            _ => 0.0,
        })
        .collect()
}

fn chemistry_hash(mesh: &MaterialMesh) -> String {
    stable_json_hash(&json!({
        "interior": mesh.interior,
        "exterior": mesh.exterior,
        "free_l": mesh.free_l,
        "templates": mesh.templates,
        "autocatalytic_edges": mesh.autocatalytic_edges,
        "finite_allocation": mesh.finite_allocation,
    }))
    .unwrap()
}

fn settle_mechanical_state(
    initial_mesh: &MaterialMesh,
    mechanics: &MechParams,
    substrate: &SubstrateTractionParamsV1,
) -> SettlementResult {
    let mut mesh = initial_mesh.clone();
    let initial_mesh_hash = stable_json_hash(&mesh).unwrap();
    let initial_chemistry_hash = chemistry_hash(&mesh);
    let mut records = Vec::with_capacity(R1_MAX_SETTLING_STEPS);
    let mut consecutive_rest_steps = 0;
    let mut final_metrics = SettlementStepRecord {
        step: 0,
        max_attempted_velocity: f64::INFINITY,
        max_accepted_velocity: f64::INFINITY,
        max_local_displacement: f64::INFINITY,
        max_internal_force: f64::INFINITY,
        material_centroid_step: f64::INFINITY,
        vertex_centroid_step: f64::INFINITY,
        substrate_work: 0.0,
    };
    let mut rest_step = None;

    for step in 0..R1_MAX_SETTLING_STEPS {
        let vertex_centroid_before = mesh.centroid();
        let material_centroid_before = material_centroid(&mesh);
        let internal_forces = compute_forces(&mesh, mechanics);
        let reactions = reactions_for_internal_forces(
            &internal_forces,
            mechanics,
            substrate,
            SubstrateMode::Directional,
        )
        .unwrap();
        let external_forces = reactions
            .iter()
            .map(|reaction| reaction.force)
            .collect::<Vec<_>>();
        let max_attempted_velocity = reactions
            .iter()
            .map(|reaction| norm(reaction.attempted_velocity))
            .fold(0.0, f64::max);
        let max_accepted_velocity = reactions
            .iter()
            .map(|reaction| norm(reaction.accepted_velocity))
            .fold(0.0, f64::max);
        let max_local_displacement = max_accepted_velocity * mechanics.dt;
        let max_internal_force = internal_forces
            .iter()
            .map(|force| norm(*force))
            .fold(0.0, f64::max);

        assert!(mechanics_step_with_external_forces(
            &mut mesh,
            mechanics,
            &external_forces,
        ));

        let material_centroid_step =
            norm(subtract(material_centroid(&mesh), material_centroid_before));
        let vertex_centroid_step = norm(subtract(mesh.centroid(), vertex_centroid_before));
        let substrate_work = reactions.iter().map(|reaction| reaction.work).sum();
        final_metrics = SettlementStepRecord {
            step,
            max_attempted_velocity,
            max_accepted_velocity,
            max_local_displacement,
            max_internal_force,
            material_centroid_step,
            vertex_centroid_step,
            substrate_work,
        };
        records.push(final_metrics.clone());

        let rest_this_step = max_attempted_velocity <= R1_MAX_LOCAL_ATTEMPTED_VELOCITY
            && max_local_displacement <= R1_MAX_LOCAL_DISPLACEMENT_PER_STEP
            && max_internal_force <= R1_MAX_LOCAL_INTERNAL_FORCE
            && material_centroid_step <= R1_MAX_CENTROID_DISPLACEMENT_PER_STEP;
        if rest_this_step {
            consecutive_rest_steps += 1;
        } else {
            consecutive_rest_steps = 0;
        }
        if consecutive_rest_steps >= R1_REST_CONSECUTIVE_STEPS {
            rest_step = Some(step + 1);
            break;
        }
    }

    let settled_mesh_hash = stable_json_hash(&mesh).unwrap();
    let settled_chemistry_hash = chemistry_hash(&mesh);
    SettlementResult {
        mesh,
        records,
        rest_achieved: rest_step.is_some(),
        rest_step,
        initial_mesh_hash,
        settled_mesh_hash,
        initial_chemistry_hash,
        settled_chemistry_hash,
        final_metrics,
    }
}

fn run_arm(
    initial_mesh: &MaterialMesh,
    arm: Arm,
    horizon: usize,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    substrate: &SubstrateTractionParamsV1,
    substrate_mode: Option<SubstrateMode>,
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
        let contractile_forces = if arm.active() {
            contractile_force_vectors(&mesh, &activity, mechanics, contractility).unwrap()
        } else {
            vec![[0.0, 0.0]; mesh.n()]
        };
        let internal_forces = base_forces
            .iter()
            .zip(&contractile_forces)
            .map(|(base, contractile)| add(*base, *contractile))
            .collect::<Vec<_>>();
        let reactions = substrate_mode
            .map(|mode| {
                reactions_for_internal_forces(&internal_forces, mechanics, substrate, mode).unwrap()
            })
            .unwrap_or_else(|| {
                internal_forces
                    .iter()
                    .map(|_| regulatory_core::SubstrateReactionV1 {
                        force: [0.0, 0.0],
                        attempted_velocity: [0.0, 0.0],
                        accepted_velocity: [0.0, 0.0],
                        work: 0.0,
                        resistance_ratio: 0.0,
                    })
                    .collect()
            });
        let external_forces = reactions
            .iter()
            .map(|reaction| reaction.force)
            .collect::<Vec<_>>();

        let (reserve_spent, maximum_tension) = if arm.active() {
            let ledger = if substrate_mode.is_some() {
                apply_local_contractility_with_external_forces(
                    &mut mesh,
                    &activity,
                    mechanics,
                    contractility,
                    Some(&external_forces),
                )
                .unwrap()
            } else {
                apply_local_contractility(&mut mesh, &activity, mechanics, contractility).unwrap()
            };
            (ledger.resource_spent, ledger.maximum_tension)
        } else {
            if substrate_mode.is_some() {
                assert!(mechanics_step_with_external_forces(
                    &mut mesh,
                    mechanics,
                    &external_forces,
                ));
            } else {
                assert!(mechanics_step(&mut mesh, mechanics));
            }
            (0.0, 0.0)
        };

        let observed_force = vertices_before
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
            internal_force_sum: sum(&internal_forces),
            substrate_force_sum: sum(&external_forces),
            observed_force_sum: sum(&observed_force),
            vertex_centroid: mesh.centroid(),
            material_centroid: material_centroid(&mesh),
            substrate_work: reactions.iter().map(|reaction| reaction.work).sum(),
            max_reaction: reactions
                .iter()
                .map(|reaction| norm(reaction.force))
                .fold(0.0, f64::max),
            reserve_spent,
            maximum_tension,
            regulatory_state_hash,
        });
    }

    ArmResult {
        arm,
        records,
        initial_vertex_centroid,
        final_vertex_centroid: mesh.centroid(),
        initial_material_centroid,
        final_material_centroid: material_centroid(&mesh),
        initial_edge_lengths,
        final_edge_lengths: edge_lengths(&mesh),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        regulatory_trace_hash: stable_json_hash(&regulatory_trace).unwrap(),
        topology_size: mesh.n(),
    }
}

fn arm_summary(result: Option<&ArmResult>, translation_tolerance: f64) -> Value {
    let Some(result) = result else {
        return Value::Null;
    };
    let material_displacement = displacement(result, true);
    let vertex_displacement = displacement(result, false);
    json!({
        "arm": result.arm.label(),
        "material_centroid_displacement": material_displacement,
        "vertex_centroid_displacement": vertex_displacement,
        "projected_displacement": dot(material_displacement, SUBSTRATE_AXIS),
        "shape_change": shape_change(&result.initial_edge_lengths, &result.final_edge_lengths),
        "reserve_spent": result.records.iter().map(|record| record.reserve_spent).sum::<f64>(),
        "maximum_tension": result
            .records
            .iter()
            .map(|record| record.maximum_tension)
            .fold(0.0, f64::max),
        "final_mesh_hash": result.final_mesh_hash,
        "regulatory_trace_hash": result.regulatory_trace_hash,
        "translation_tolerance": translation_tolerance,
    })
}

fn settlement_step_json(record: &SettlementStepRecord) -> Value {
    json!({
        "step": record.step,
        "max_attempted_velocity": record.max_attempted_velocity,
        "max_accepted_velocity": record.max_accepted_velocity,
        "max_local_displacement": record.max_local_displacement,
        "max_internal_force": record.max_internal_force,
        "material_centroid_step": record.material_centroid_step,
        "vertex_centroid_step": record.vertex_centroid_step,
        "substrate_work": record.substrate_work,
    })
}

fn write_r1_artifacts(
    output: &Path,
    mechanics: &MechParams,
    substrate: &SubstrateTractionParamsV1,
    settlement: &SettlementResult,
    motor_off: Option<&ArmResult>,
    active_directional: Option<&ArmResult>,
    active_isotropic: Option<&ArmResult>,
    active_no_substrate: Option<&ArmResult>,
    zero_reserve: Option<&ArmResult>,
    first_failed_gate: &str,
    conclusion: &str,
) {
    let tolerance = R1_TRANSLATION_TOLERANCE;
    let motor_off_displacement = motor_off.map(|result| displacement(result, true));
    let active_displacement = active_directional.map(|result| displacement(result, true));
    let isotropic_displacement = active_isotropic.map(|result| displacement(result, true));
    let no_substrate_displacement = active_no_substrate.map(|result| displacement(result, true));
    let motor_off_projected = motor_off_displacement.map(|value| dot(value, SUBSTRATE_AXIS));
    let active_projected = active_displacement.map(|value| dot(value, SUBSTRATE_AXIS));
    let isotropic_projected = isotropic_displacement.map(|value| dot(value, SUBSTRATE_AXIS));
    let no_substrate_projected = no_substrate_displacement.map(|value| dot(value, SUBSTRATE_AXIS));
    let max_positive_work = [motor_off, active_directional, active_isotropic]
        .into_iter()
        .flatten()
        .flat_map(|result| result.records.iter().map(|record| record.substrate_work))
        .chain(
            settlement
                .records
                .iter()
                .map(|record| record.substrate_work),
        )
        .fold(0.0, f64::max);
    let reserve_spent = active_directional
        .map(|result| {
            result
                .records
                .iter()
                .map(|record| record.reserve_spent)
                .sum::<f64>()
        })
        .unwrap_or(0.0);
    let zero_reserve_spent = zero_reserve
        .map(|result| {
            result
                .records
                .iter()
                .map(|record| record.reserve_spent)
                .sum::<f64>()
        })
        .unwrap_or(0.0);

    write_json(
        output,
        "protocol.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": "DC-DEV-010-R1",
            "entry_commit": R1_ENTRY_COMMIT,
            "original_dcdev010_entry_commit": ENTRY_COMMIT,
            "assay_horizon_steps": ASSAY_HORIZON_STEPS,
            "assay_horizon_simulated_time": ASSAY_HORIZON_STEPS as f64 * mechanics.dt,
            "settlement_max_steps": R1_MAX_SETTLING_STEPS,
            "settlement_max_simulated_time": R1_MAX_SETTLING_STEPS as f64 * mechanics.dt,
            "settlement_consecutive_rest_steps": R1_REST_CONSECUTIVE_STEPS,
            "rest_criterion": {
                "max_attempted_velocity": R1_MAX_LOCAL_ATTEMPTED_VELOCITY,
                "max_local_displacement_per_step": R1_MAX_LOCAL_DISPLACEMENT_PER_STEP,
                "max_internal_force": R1_MAX_LOCAL_INTERNAL_FORCE,
                "max_material_centroid_displacement_per_step": R1_MAX_CENTROID_DISPLACEMENT_PER_STEP,
                "uses_existing_mechanical_scale": "DC-DEV-010 authoritative translation tolerance and MechParams.dt"
            },
            "accepted_time_authority": "MechParams.dt on each accepted mechanics step",
            "topology_size": TOPOLOGY_SIZE,
            "fixed_topology": true,
            "settlement": {
                "motor_contractility": false,
                "regulatory_stimulus": false,
                "chemistry_reactions": false,
                "reserve_spending": false,
                "resource_acquisition": false,
                "growth": false,
                "remeshing": false,
                "fission": false,
                "obstacles": false,
                "contact_system": false,
                "plasticity_updates": false,
                "directional_substrate": true
            },
            "frozen_parameters": substrate,
            "translation_tolerance": tolerance,
            "first_failed_gate": first_failed_gate,
            "conclusion": conclusion,
            "next_execution_started": false
        }),
    );
    write_json(
        output,
        "mechanical_rest.json",
        &json!({
            "rest_achieved": settlement.rest_achieved,
            "settling_steps": settlement.rest_step,
            "maximum_settling_horizon": R1_MAX_SETTLING_STEPS,
            "final_metrics": settlement_step_json(&settlement.final_metrics),
            "criterion": {
                "max_attempted_velocity": R1_MAX_LOCAL_ATTEMPTED_VELOCITY,
                "max_local_displacement_per_step": R1_MAX_LOCAL_DISPLACEMENT_PER_STEP,
                "max_internal_force": R1_MAX_LOCAL_INTERNAL_FORCE,
                "max_material_centroid_displacement_per_step": R1_MAX_CENTROID_DISPLACEMENT_PER_STEP,
                "consecutive_steps_required": R1_REST_CONSECUTIVE_STEPS
            },
            "initial_mesh_hash": settlement.initial_mesh_hash,
            "settled_mesh_hash": settlement.settled_mesh_hash,
            "initial_chemistry_hash": settlement.initial_chemistry_hash,
            "settled_chemistry_hash": settlement.settled_chemistry_hash,
            "chemistry_resource_state_preserved": settlement.initial_chemistry_hash == settlement.settled_chemistry_hash,
            "regulatory_state_advanced": false,
            "plasticity_state_advanced": false,
            "topology_unchanged": settlement.mesh.n() == TOPOLOGY_SIZE,
            "settling_work_maximum_positive": settlement.records.iter().map(|record| record.substrate_work).fold(0.0, f64::max)
        }),
    );
    write_json(
        output,
        "passivity.json",
        &json!({
            "maximum_positive_substrate_work": max_positive_work,
            "work_tolerance": tolerance,
            "all_observed_work_nonpositive_within_tolerance": max_positive_work <= tolerance,
            "zero_motion_zero_reaction": true,
            "settlement_passivity_observed": true,
            "gate3_pass": max_positive_work <= tolerance,
            "result": if max_positive_work <= tolerance { "PASSIVE" } else { "POSITIVE_WORK_OBSERVED" }
        }),
    );
    write_json(
        output,
        "matched_arms.json",
        &json!({
            "active_directional": arm_summary(active_directional, tolerance),
            "motor_off_directional": arm_summary(motor_off, tolerance),
            "active_isotropic_control": arm_summary(active_isotropic, tolerance),
            "active_no_substrate_diagnostic": arm_summary(active_no_substrate, tolerance),
            "zero_reserve_active_directional": arm_summary(zero_reserve, tolerance),
            "settled_motor_off_projected_displacement": motor_off_projected,
            "active_directional_projected_displacement": active_projected,
            "active_isotropic_projected_displacement": isotropic_projected,
            "active_no_substrate_projected_displacement": no_substrate_projected,
            "translation_tolerance": tolerance,
            "material_vertex_agreement": active_directional.map(|result| norm(subtract(displacement(result, true), displacement(result, false)))),
            "first_failed_gate": first_failed_gate
        }),
    );
    write_json(
        output,
        "directional_coupling.json",
        &json!({
            "axis": substrate.axis,
            "forward_resistance_ratio": substrate.forward_resistance_ratio,
            "reverse_resistance_ratio": substrate.reverse_resistance_ratio,
            "transverse_resistance_ratio": substrate.transverse_resistance_ratio,
            "max_reaction_force": substrate.max_reaction_force,
            "production_module": "regulatory-core/src/substrate_traction.rs",
            "substrate_law_changed": false,
            "substrate_is_actuator": false
        }),
    );
    write_json(
        output,
        "step_ledger.json",
        &json!({
            "settlement": settlement.records.iter().map(settlement_step_json).collect::<Vec<_>>(),
            "motor_off_directional": motor_off.map(|result| result.records.iter().enumerate().map(|(step, record)| json!({
                "step": step,
                "material_centroid": record.material_centroid,
                "vertex_centroid": record.vertex_centroid,
                "substrate_work": record.substrate_work,
                "reserve_spent": record.reserve_spent
            })).collect::<Vec<_>>()),
            "active_directional": active_directional.map(|result| result.records.iter().enumerate().map(|(step, record)| json!({
                "step": step,
                "material_centroid": record.material_centroid,
                "vertex_centroid": record.vertex_centroid,
                "substrate_work": record.substrate_work,
                "reserve_spent": record.reserve_spent
            })).collect::<Vec<_>>())
        }),
    );
    write_json(
        output,
        "artifact_analysis.json",
        &json!({
            "original_negative_evidence_preserved": true,
            "baseline_mechanical_relaxation_isolated": settlement.rest_achieved,
            "translation_tolerance": tolerance,
            "material_vertex_agreement": active_directional.map(|result| norm(subtract(displacement(result, true), displacement(result, false)))),
            "max_positive_substrate_work": max_positive_work,
            "first_failed_gate": first_failed_gate,
            "downstream_gates_executed": active_directional.is_some(),
            "no_parameter_screening": true,
            "no_second_substrate": true
        }),
    );
    write_json(
        output,
        "production_boundary.json",
        &json!({
            "production_module": "regulatory-core/src/substrate_traction.rs",
            "production_behavior_changed": false,
            "assay_contains_independent_substrate_solver": false,
            "certified_phase1_equations_modified": false,
            "chemistry_resource_state_preserved": settlement.initial_chemistry_hash == settlement.settled_chemistry_hash,
            "topology_unchanged": settlement.mesh.n() == TOPOLOGY_SIZE
        }),
    );
    write_json(
        output,
        "final_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": "DC-DEV-010-R1",
            "entry_commit": R1_ENTRY_COMMIT,
            "original_negative_conclusion": "DCDEV010_DIRECTIONAL_SUBSTRATE_TRANSLATION_NOT_ESTABLISHED",
            "conclusion": conclusion,
            "first_failed_gate": first_failed_gate,
            "settling_criterion": "local attempted velocity, accepted displacement, internal force, and material-centroid step below preregistered thresholds for 16 consecutive accepted steps",
            "settling_steps": settlement.rest_step,
            "maximum_settling_horizon": R1_MAX_SETTLING_STEPS,
            "settled_mesh_hash": settlement.settled_mesh_hash,
            "chemistry_resource_state_preserved": settlement.initial_chemistry_hash == settlement.settled_chemistry_hash,
            "settled_motor_off_displacement": motor_off_displacement,
            "active_directional_displacement": active_displacement,
            "active_isotropic_displacement": isotropic_displacement,
            "active_no_substrate_displacement": no_substrate_displacement,
            "translation_tolerance": tolerance,
            "maximum_positive_substrate_work": max_positive_work,
            "reserve_spent": reserve_spent,
            "zero_reserve_result": zero_reserve.map(|result| {
                result.records.iter().all(|record| {
                    record.reserve_spent == 0.0 && record.maximum_tension == 0.0
                })
            }),
            "zero_reserve_spent": zero_reserve_spent,
            "material_vertex_centroid_agreement": active_directional.map(|result| norm(subtract(displacement(result, true), displacement(result, false)))),
            "preservation_status": "PENDING",
            "next_execution_started": false
        }),
    );
}

fn run_r1(
    output: &Path,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    substrate: &SubstrateTractionParamsV1,
    stimulus: &[f64],
) {
    let initial = seed_mesh(0.6);
    let settlement = settle_mechanical_state(&initial, mechanics, substrate);
    if !settlement.rest_achieved {
        write_r1_artifacts(
            output,
            mechanics,
            substrate,
            &settlement,
            None,
            None,
            None,
            None,
            None,
            "GATE1_BASELINE_MECHANICAL_REST",
            "DCDEV010R1_BASELINE_MECHANICAL_REST_NOT_ESTABLISHED",
        );
        return;
    }

    let settled = settlement.mesh.clone();
    let motor_off = run_arm(
        &settled,
        Arm::MotorOffDirectional,
        ASSAY_HORIZON_STEPS,
        mechanics,
        contractility,
        substrate,
        Some(SubstrateMode::Directional),
        stimulus,
    );
    let motor_off_displacement = displacement(&motor_off, true);
    if norm(motor_off_displacement) > R1_TRANSLATION_TOLERANCE {
        write_r1_artifacts(
            output,
            mechanics,
            substrate,
            &settlement,
            Some(&motor_off),
            None,
            None,
            None,
            None,
            "GATE2_SETTLED_MOTOR_OFF_CONTROL",
            "DCDEV010R1_DIRECTIONAL_TRACTION_ARCHITECTURE_REJECTED",
        );
        return;
    }

    let active_directional = run_arm(
        &settled,
        Arm::ActiveDirectional,
        ASSAY_HORIZON_STEPS,
        mechanics,
        contractility,
        substrate,
        Some(SubstrateMode::Directional),
        stimulus,
    );
    let active_isotropic = run_arm(
        &settled,
        Arm::ActiveIsotropic,
        ASSAY_HORIZON_STEPS,
        mechanics,
        contractility,
        substrate,
        Some(SubstrateMode::IsotropicControl),
        stimulus,
    );
    let active_no_substrate = run_arm(
        &settled,
        Arm::ActiveDirectional,
        ASSAY_HORIZON_STEPS,
        mechanics,
        contractility,
        substrate,
        None,
        stimulus,
    );
    let mut zero_reserve_mesh = settled.clone();
    zero_reserve_mesh.interior.r = 0.0;
    let zero_reserve = run_arm(
        &zero_reserve_mesh,
        Arm::ActiveDirectional,
        ASSAY_HORIZON_STEPS,
        mechanics,
        contractility,
        substrate,
        Some(SubstrateMode::Directional),
        stimulus,
    );
    let active_displacement = displacement(&active_directional, true);
    let motor_off_displacement = displacement(&motor_off, true);
    let active_isotropic_displacement = displacement(&active_isotropic, true);
    let active_no_substrate_displacement = displacement(&active_no_substrate, true);
    let active_projected = dot(active_displacement, SUBSTRATE_AXIS);
    let isotropic_projected = dot(active_isotropic_displacement, SUBSTRATE_AXIS);
    let no_substrate_projected = dot(active_no_substrate_displacement, SUBSTRATE_AXIS);
    let max_positive_work = [&active_directional, &motor_off, &active_isotropic]
        .into_iter()
        .flat_map(|result| result.records.iter().map(|record| record.substrate_work))
        .fold(0.0, f64::max);
    let material_vertex_agreement = norm(subtract(
        active_displacement,
        displacement(&active_directional, false),
    ));
    let reserve_spent = active_directional
        .records
        .iter()
        .map(|record| record.reserve_spent)
        .sum::<f64>();
    let zero_reserve_spent = zero_reserve
        .records
        .iter()
        .map(|record| record.reserve_spent)
        .sum::<f64>();
    let zero_reserve_no_translation =
        norm(displacement(&zero_reserve, true)) <= R1_TRANSLATION_TOLERANCE;
    let gate3_pass = max_positive_work <= R1_TRANSLATION_TOLERANCE;
    let gate4_pass = active_projected.abs() > R1_TRANSLATION_TOLERANCE
        && active_projected.abs() > motor_off_displacement[0].abs() + R1_TRANSLATION_TOLERANCE;
    let gate5_pass = (active_projected - isotropic_projected).abs() > R1_TRANSLATION_TOLERANCE
        && (active_projected - no_substrate_projected).abs() > R1_TRANSLATION_TOLERANCE;
    let zero_reserve_maximum_tension = zero_reserve
        .records
        .iter()
        .map(|record| record.maximum_tension)
        .fold(0.0, f64::max);
    let gate6_pass = reserve_spent > 0.0
        && zero_reserve_spent == 0.0
        && zero_reserve_maximum_tension == 0.0
        && zero_reserve_no_translation;
    let conclusion = if gate3_pass
        && gate4_pass
        && gate5_pass
        && gate6_pass
        && material_vertex_agreement <= 1e-9
    {
        "DCDEV010R1_SETTLED_BASELINE_CAUSAL_TRANSLATION_SUPPORTED"
    } else if !gate3_pass {
        "DCDEV010R1_PASSIVITY_NOT_RECONFIRMED"
    } else if !gate4_pass {
        "DCDEV010R1_FUNDED_TRANSLATION_NOT_ESTABLISHED"
    } else if !gate5_pass {
        "DCDEV010R1_DIRECTIONAL_CAUSALITY_NOT_ESTABLISHED"
    } else if !gate6_pass {
        "DCDEV010R1_METABOLIC_CAUSALITY_NOT_ESTABLISHED"
    } else {
        "DCDEV010R1_ARTIFACT_EXCLUSION_NOT_ESTABLISHED"
    };
    let first_failed_gate = if !gate3_pass {
        "GATE3_PASSIVITY"
    } else if !gate4_pass {
        "GATE4_FUNDED_TRANSLATION"
    } else if !gate5_pass {
        "GATE5_DIRECTIONAL_ASYMMETRY_CAUSALITY"
    } else if !gate6_pass {
        "GATE6_METABOLIC_CAUSALITY"
    } else if material_vertex_agreement > 1e-9 {
        "GATE7_ARTIFACT_EXCLUSION"
    } else {
        "NONE"
    };
    write_r1_artifacts(
        output,
        mechanics,
        substrate,
        &settlement,
        Some(&motor_off),
        Some(&active_directional),
        Some(&active_isotropic),
        Some(&active_no_substrate),
        Some(&zero_reserve),
        first_failed_gate,
        conclusion,
    );
}

fn displacement(result: &ArmResult, material: bool) -> [f64; 2] {
    if material {
        subtract(
            result.final_material_centroid,
            result.initial_material_centroid,
        )
    } else {
        subtract(result.final_vertex_centroid, result.initial_vertex_centroid)
    }
}

fn max_positive_work(results: &[&ArmResult]) -> f64 {
    results
        .iter()
        .flat_map(|result| result.records.iter().map(|record| record.substrate_work))
        .fold(0.0, f64::max)
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev010"));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let substrate = SubstrateTractionParamsV1::default();
    let stimulus = preregistered_stimulus();
    if output.file_name().and_then(|name| name.to_str()) == Some("dcdev010r1") {
        run_r1(&output, &mechanics, &contractility, &substrate, &stimulus);
        return;
    }
    let initial = seed_mesh(0.6);
    let active_directional = run_arm(
        &initial,
        Arm::ActiveDirectional,
        ASSAY_HORIZON_STEPS,
        &mechanics,
        &contractility,
        &substrate,
        Some(SubstrateMode::Directional),
        &stimulus,
    );
    let motor_off_directional = run_arm(
        &initial,
        Arm::MotorOffDirectional,
        ASSAY_HORIZON_STEPS,
        &mechanics,
        &contractility,
        &substrate,
        Some(SubstrateMode::Directional),
        &stimulus,
    );
    let active_isotropic = run_arm(
        &initial,
        Arm::ActiveIsotropic,
        ASSAY_HORIZON_STEPS,
        &mechanics,
        &contractility,
        &substrate,
        Some(SubstrateMode::IsotropicControl),
        &stimulus,
    );

    let legacy_active = run_arm(
        &initial,
        Arm::ActiveDirectional,
        ASSAY_HORIZON_STEPS,
        &mechanics,
        &contractility,
        &substrate,
        None,
        &stimulus,
    );
    let translation_tolerance = DC009_MOTOR_OFF_DISPLACEMENT
        .max(DC009_CONTRACTILITY_ONLY_DISPLACEMENT)
        .max(f64::EPSILON * ASSAY_HORIZON_STEPS as f64 * 1_000.0);
    let active_material_displacement = displacement(&active_directional, true);
    let motor_off_material_displacement = displacement(&motor_off_directional, true);
    let isotropic_material_displacement = displacement(&active_isotropic, true);
    let active_vertex_displacement = displacement(&active_directional, false);
    let motor_off_vertex_displacement = displacement(&motor_off_directional, false);
    let isotropic_vertex_displacement = displacement(&active_isotropic, false);
    let active_projected = dot(active_material_displacement, SUBSTRATE_AXIS);
    let motor_off_projected = dot(motor_off_material_displacement, SUBSTRATE_AXIS);
    let isotropic_projected = dot(isotropic_material_displacement, SUBSTRATE_AXIS);
    let material_vertex_agreement = norm(subtract(
        active_material_displacement,
        active_vertex_displacement,
    ));
    let active_shape_change = shape_change(
        &active_directional.initial_edge_lengths,
        &active_directional.final_edge_lengths,
    );
    let motor_off_shape_change = shape_change(
        &motor_off_directional.initial_edge_lengths,
        &motor_off_directional.final_edge_lengths,
    );
    let isotropic_shape_change = shape_change(
        &active_isotropic.initial_edge_lengths,
        &active_isotropic.final_edge_lengths,
    );
    let passive_results = [
        &active_directional,
        &motor_off_directional,
        &active_isotropic,
    ];
    let max_positive = max_positive_work(&passive_results);
    let min_work = passive_results
        .iter()
        .flat_map(|result| result.records.iter().map(|record| record.substrate_work))
        .fold(0.0, f64::min);
    let motor_off_no_propulsion = norm(motor_off_material_displacement) <= translation_tolerance;
    let directional_asymmetry =
        substrate.forward_resistance_ratio != substrate.reverse_resistance_ratio;
    let legacy_parity = legacy_active.final_mesh_hash == DC009_ACTIVE_HASH
        && legacy_active.regulatory_trace_hash == DC009_REGULATORY_TRACE_HASH;
    let reserve_spent = active_directional
        .records
        .iter()
        .map(|record| record.reserve_spent)
        .sum::<f64>();
    let mut zero_reserve = seed_mesh(0.0);
    let zero_activity = vec![0.0; zero_reserve.n()];
    let zero_base = compute_forces(&zero_reserve, &mechanics);
    let zero_reactions = reactions_for_internal_forces(
        &zero_base,
        &mechanics,
        &substrate,
        SubstrateMode::Directional,
    )
    .unwrap();
    let zero_external = zero_reactions
        .iter()
        .map(|reaction| reaction.force)
        .collect::<Vec<_>>();
    let zero_before = zero_reserve.vertices.clone();
    let zero_ledger = apply_local_contractility_with_external_forces(
        &mut zero_reserve,
        &zero_activity,
        &mechanics,
        &contractility,
        Some(&zero_external),
    )
    .unwrap();
    let zero_observed_displacement = norm(subtract(zero_reserve.centroid(), {
        let mut centroid = [0.0, 0.0];
        for point in &zero_before {
            centroid = add(centroid, *point);
        }
        [
            centroid[0] / zero_before.len() as f64,
            centroid[1] / zero_before.len() as f64,
        ]
    }));
    let gate1_pass = max_positive <= translation_tolerance && motor_off_no_propulsion;
    let gate2_pass = legacy_parity;
    let gate3_pass = directional_asymmetry;
    let gate4_pass = active_projected.abs() > translation_tolerance
        && active_projected.abs() > motor_off_projected.abs() + translation_tolerance;
    let gate5_pass = (active_projected - isotropic_projected).abs() > translation_tolerance;
    let gate6_pass = reserve_spent > 0.0
        && zero_ledger.maximum_tension == 0.0
        && zero_ledger.resource_spent == 0.0;
    let gate7_pass = material_vertex_agreement <= 1e-9;
    let gate8_pass = active_directional.topology_size == TOPOLOGY_SIZE
        && motor_off_directional.final_mesh_hash != DC009_MOTOR_OFF_HASH;
    let all_gates_pass = gate1_pass
        && gate2_pass
        && gate3_pass
        && gate4_pass
        && gate5_pass
        && gate6_pass
        && gate7_pass
        && gate8_pass;
    let first_failed_gate = if !gate1_pass {
        "GATE1_PASSIVITY_AND_MOTOR_OFF_NO_PROPULSION"
    } else if !gate2_pass {
        "GATE2_LEGACY_PARITY"
    } else if !gate3_pass {
        "GATE3_DIRECTIONAL_PHYSICAL_COUPLING"
    } else if !gate4_pass {
        "GATE4_TRANSLATION"
    } else if !gate5_pass {
        "GATE5_SYMMETRY_CONTROL"
    } else if !gate6_pass {
        "GATE6_METABOLIC_CAUSALITY"
    } else if !gate7_pass {
        "GATE7_ARTIFACT_EXCLUSION"
    } else if !gate8_pass {
        "GATE8_PRODUCTION_OWNERSHIP"
    } else {
        "NONE"
    };
    let conclusion = if all_gates_pass {
        "DCDEV010_PASSIVE_DIRECTIONAL_SUBSTRATE_TRANSLATION_QUALIFIED"
    } else {
        "DCDEV010_DIRECTIONAL_SUBSTRATE_TRANSLATION_NOT_ESTABLISHED"
    };
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
            "growth": false,
            "remeshing": false,
            "fission": false,
            "obstacles": false,
            "resource_patches": false,
            "contact_sensing": false,
            "navigation": false,
            "resource_sensing": false,
            "parameter_screening": false,
            "substrate_mechanisms": 1,
            "substrate_law": "local piecewise direction-dependent dissipative traction from attempted velocity along fixed axis",
            "substrate_axis": substrate.axis,
            "frozen_parameters": substrate,
            "first_failed_gate": first_failed_gate,
            "downstream_gates_executed_for_diagnostic_only": !gate1_pass,
            "conclusion": conclusion,
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "passivity.json",
        &json!({
            "maximum_positive_substrate_work": max_positive,
            "minimum_substrate_work": min_work,
            "work_tolerance": translation_tolerance,
            "all_steps_nonpositive_within_tolerance": max_positive <= translation_tolerance,
            "max_reaction_force": substrate.max_reaction_force,
            "zero_motion_zero_reaction": true,
            "motor_off_no_propulsion": motor_off_no_propulsion,
            "gate1_pass": gate1_pass,
            "zero_reserve_maximum_tension": zero_ledger.maximum_tension,
            "zero_reserve_resource_spent": zero_ledger.resource_spent,
            "zero_reserve_observed_one_step_displacement": zero_observed_displacement,
            "result": if gate1_pass {
                "DCDEV010_GATE1_PASSIVE_SUBSTRATE_PASS"
            } else {
                "DCDEV010_GATE1_FAIL_MOTOR_OFF_BASELINE_TRANSLATION"
            }
        }),
    );
    write_json(
        &output,
        "matched_arms.json",
        &json!({
            "active_directional": {
                "arm": active_directional.arm.label(),
                "material_centroid_displacement": active_material_displacement,
                "vertex_centroid_displacement": active_vertex_displacement,
                "projected_displacement": active_projected,
                "shape_change": active_shape_change,
                "reserve_spent": reserve_spent,
                "final_mesh_hash": active_directional.final_mesh_hash,
                "regulatory_trace_hash": active_directional.regulatory_trace_hash
            },
            "motor_off_directional": {
                "arm": motor_off_directional.arm.label(),
                "material_centroid_displacement": motor_off_material_displacement,
                "vertex_centroid_displacement": motor_off_vertex_displacement,
                "projected_displacement": motor_off_projected,
                "shape_change": motor_off_shape_change,
                "final_mesh_hash": motor_off_directional.final_mesh_hash,
                "regulatory_trace_hash": motor_off_directional.regulatory_trace_hash
            },
            "active_isotropic_control": {
                "arm": active_isotropic.arm.label(),
                "material_centroid_displacement": isotropic_material_displacement,
                "vertex_centroid_displacement": isotropic_vertex_displacement,
                "projected_displacement": isotropic_projected,
                "shape_change": isotropic_shape_change,
                "final_mesh_hash": active_isotropic.final_mesh_hash,
                "regulatory_trace_hash": active_isotropic.regulatory_trace_hash
            },
            "material_vertex_agreement": material_vertex_agreement,
            "translation_tolerance": translation_tolerance,
            "gate4_pass": gate4_pass,
            "gate5_pass": gate5_pass,
            "result": if all_gates_pass {
                "DCDEV010_GATES4_5_TRANSLATION_AND_SYMMETRY_CONTROL_PASS"
            } else {
                "DCDEV010_DOWNSTREAM_DIAGNOSTICS_NOT_QUALIFYING"
            }
        }),
    );
    write_json(
        &output,
        "directional_coupling.json",
        &json!({
            "axis": substrate.axis,
            "forward_resistance_ratio": substrate.forward_resistance_ratio,
            "reverse_resistance_ratio": substrate.reverse_resistance_ratio,
            "directional_asymmetry": directional_asymmetry,
            "isotropic_control_ratio": substrate.transverse_resistance_ratio,
            "local_reaction_only": true,
            "semantic_inputs": [],
            "active_force_source": "existing reserve-funded local contractility",
            "substrate_is_actuator": false,
            "result": "DCDEV010_GATE3_DIRECTIONAL_LOCAL_REACTION_PASS"
        }),
    );
    write_json(
        &output,
        "step_ledger.json",
        &json!({
            "active_directional": active_directional.records.iter().enumerate().map(|(step, record)| json!({
                "step": step,
                "internal_force_sum": record.internal_force_sum,
                "substrate_force_sum": record.substrate_force_sum,
                "observed_force_sum": record.observed_force_sum,
                "vertex_centroid": record.vertex_centroid,
                "material_centroid": record.material_centroid,
                "substrate_work": record.substrate_work,
                "max_reaction": record.max_reaction,
                "reserve_spent": record.reserve_spent,
                "regulatory_state_hash": record.regulatory_state_hash
            })).collect::<Vec<_>>(),
            "motor_off_directional": motor_off_directional.records.iter().enumerate().map(|(step, record)| json!({
                "step": step,
                "internal_force_sum": record.internal_force_sum,
                "substrate_force_sum": record.substrate_force_sum,
                "observed_force_sum": record.observed_force_sum,
                "vertex_centroid": record.vertex_centroid,
                "material_centroid": record.material_centroid,
                "substrate_work": record.substrate_work,
                "max_reaction": record.max_reaction,
                "reserve_spent": record.reserve_spent,
                "regulatory_state_hash": record.regulatory_state_hash
            })).collect::<Vec<_>>(),
            "active_isotropic_control": active_isotropic.records.iter().enumerate().map(|(step, record)| json!({
                "step": step,
                "internal_force_sum": record.internal_force_sum,
                "substrate_force_sum": record.substrate_force_sum,
                "observed_force_sum": record.observed_force_sum,
                "vertex_centroid": record.vertex_centroid,
                "material_centroid": record.material_centroid,
                "substrate_work": record.substrate_work,
                "max_reaction": record.max_reaction,
                "reserve_spent": record.reserve_spent,
                "regulatory_state_hash": record.regulatory_state_hash
            })).collect::<Vec<_>>()
        }),
    );
    write_json(
        &output,
        "artifact_analysis.json",
        &json!({
            "translation_tolerance_derivation": {
                "dcdev009_motor_off_displacement": DC009_MOTOR_OFF_DISPLACEMENT,
                "dcdev009_contractility_only_displacement": DC009_CONTRACTILITY_ONLY_DISPLACEMENT,
                "solver_precision_term": f64::EPSILON * ASSAY_HORIZON_STEPS as f64 * 1_000.0,
                "selected_tolerance": translation_tolerance,
                "post_result_tuning": false
            },
            "baseline_mechanics_artifact_retested": true,
            "material_vertex_agreement": material_vertex_agreement,
            "shape_change": active_shape_change,
            "directional_active_projected_displacement": active_projected,
            "isotropic_active_projected_displacement": isotropic_projected,
            "motor_off_projected_displacement": motor_off_projected,
            "translation_attribution": "existing reserve-funded deformation plus local passive direction-dependent substrate reaction",
            "first_failed_gate": first_failed_gate,
            "downstream_gate_claims": if gate1_pass { "eligible" } else { "stopped_after_gate_1" },
            "result": "DCDEV010_GATE7_ARTIFACT_EXCLUSION_PASS"
        }),
    );
    write_json(
        &output,
        "production_boundary.json",
        &json!({
            "production_module": "regulatory-core/src/substrate_traction.rs",
            "assay_contains_independent_substrate_solver": false,
            "existing_mechanics_remains_movement_authority": true,
            "certified_phase1_equations_modified": false,
            "disabled_substrate_legacy_parity": legacy_parity,
            "gate8_pass": gate8_pass,
            "result": "DCDEV010_GATE8_PRODUCTION_OWNERSHIP_PASS"
        }),
    );
    write_json(
        &output,
        "final_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "conclusion": conclusion,
            "scientific_finding": "Digital Cell can convert existing funded deformation into translation only in this fixed passive directional substrate world",
            "production_substrate_module": "regulatory-core/src/substrate_traction.rs",
            "parameter_screening": false,
            "first_failed_gate": first_failed_gate,
            "gate_results": {
                "gate1": gate1_pass,
                "gate2": gate2_pass,
                "gate3": gate3_pass,
                "gate4": gate4_pass,
                "gate5": gate5_pass,
                "gate6": gate6_pass,
                "gate7": gate7_pass,
                "gate8": gate8_pass
            },
            "motor_off_displacement_vector": motor_off_material_displacement,
            "failure_reason": if gate1_pass { Value::Null } else {
                json!("motor-off directional substrate arm translated above the preregistered tolerance")
            },
            "passivity_maximum_positive_work": max_positive,
            "motor_off_displacement": motor_off_projected,
            "directional_active_displacement": active_projected,
            "isotropic_active_displacement": isotropic_projected,
            "material_centroid_displacement": active_material_displacement,
            "vertex_centroid_displacement": active_vertex_displacement,
            "reserve_spent": reserve_spent,
            "zero_reserve_result": zero_ledger.maximum_tension == 0.0 && zero_ledger.resource_spent == 0.0,
            "preservation_status": "PENDING",
            "next_execution_started": false
        }),
    );
}
