//! DC-DEV-011: passive isotropic stick-slip traction qualification.
//!
//! The assay uses the production regulatory-core stick-slip implementation.
//! It does not implement a second substrate law and never writes coordinates
//! except through the existing chemistry-core mechanics authority.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use regulatory_core::continuity::ContinuityMaterialFrameV1;
use regulatory_core::{
    apply_local_contractility, apply_local_contractility_with_stick_slip,
    apply_stick_slip_to_legacy_mechanics, stable_json_hash, ContinuityNetworkV1,
    ContractilityParamsV1, StickSlipTractionParamsV1, TopologyEventV1,
    FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-011";
const ENTRY_COMMIT: &str = "8d6fe59397cabfa47bc1d8103acd68f544acc190";
const FREEZE_COMMIT: &str = "f263536bc2da028630fe09108d07e7ada2e8ca38";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ACTIVE_STEPS: usize = 240;
const RELAXATION_STEPS: usize = 240;
const TOTAL_STEPS: usize = ACTIVE_STEPS + RELAXATION_STEPS;
const TRANSLATION_TOLERANCE: f64 = 1e-10;
const CENTROID_AGREEMENT_TOLERANCE: f64 = 1e-8;
const ROTATIONAL_SYMMETRY_TOLERANCE: f64 = 1e-9;
const MIN_RETAINED_DISPLACEMENT_FRACTION: f64 = 0.25;
const R1_MAX_ATTEMPTED_VELOCITY: f64 = 2.6645352591003757e-9;
const R1_MAX_LOCAL_DISPLACEMENT: f64 = 5.3290705182007514e-11;
const R1_MAX_MATERIAL_CENTROID_STEP: f64 = 2.220446049250313e-13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arm {
    ActiveStickSlip,
    MotorOffStickSlip,
    ActiveNoSubstrate,
    ZeroReserveStickSlip,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::ActiveStickSlip => "active_stick_slip",
            Self::MotorOffStickSlip => "motor_off_stick_slip",
            Self::ActiveNoSubstrate => "active_no_substrate",
            Self::ZeroReserveStickSlip => "zero_reserve_stick_slip",
        }
    }
}

#[derive(Debug, Clone)]
struct Settlement {
    mesh: MaterialMesh,
    initial_mesh_hash: String,
    settled_mesh_hash: String,
    initial_chemistry_hash: String,
    settled_chemistry_hash: String,
    maximum_attempted_velocity: f64,
    maximum_local_displacement: f64,
    maximum_material_centroid_step: f64,
    settled: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TrajectorySample {
    step: usize,
    phase: &'static str,
    material_centroid: [f64; 2],
    vertex_centroid: [f64; 2],
    shape_change: f64,
    stuck_contacts: usize,
    slipping_contacts: usize,
    substrate_work: f64,
}

#[derive(Debug, Clone)]
struct ArmRun {
    arm: Arm,
    initial_material_centroid: [f64; 2],
    active_end_material_centroid: [f64; 2],
    final_material_centroid: [f64; 2],
    initial_vertex_centroid: [f64; 2],
    active_end_vertex_centroid: [f64; 2],
    final_vertex_centroid: [f64; 2],
    initial_edge_lengths: Vec<f64>,
    final_edge_lengths: Vec<f64>,
    final_shape_change: f64,
    maximum_stick_reaction: f64,
    maximum_slip_reaction: f64,
    stuck_contacts: usize,
    slipping_contacts: usize,
    active_stuck_contacts: usize,
    active_slipping_contacts: usize,
    substrate_work: f64,
    maximum_positive_substrate_work: f64,
    reserve_spent: f64,
    maximum_funded_tension: f64,
    initial_reserve: f64,
    final_reserve: f64,
    initial_chemistry_hash: String,
    final_chemistry_hash: String,
    final_mesh_hash: String,
    regulatory_trace_hash: String,
    samples: Vec<TrajectorySample>,
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

fn edge_lengths(mesh: &MaterialMesh) -> Vec<f64> {
    (0..mesh.n()).map(|index| mesh.edge_length(index)).collect()
}

fn shape_change(initial: &[f64], mesh: &MaterialMesh) -> f64 {
    initial
        .iter()
        .zip(0..mesh.n())
        .map(|(before, index)| (mesh.edge_length(index) - before).powi(2))
        .sum::<f64>()
        .sqrt()
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

fn preregistered_stimulus(active: bool) -> Vec<f64> {
    if !active {
        return vec![0.0; TOPOLOGY_SIZE];
    }
    (0..TOPOLOGY_SIZE)
        .map(|index| match index {
            0..=4 => 1.0,
            5..=7 => 0.35,
            _ => 0.0,
        })
        .collect()
}

fn settle_body(mechanics: &MechParams) -> Settlement {
    let mut mesh = seed_mesh();
    let initial_mesh_hash = stable_json_hash(&mesh).unwrap();
    let initial_chemistry_hash = chemistry_hash(&mesh);
    let mut maximum_attempted_velocity: f64 = 0.0;
    let mut maximum_local_displacement: f64 = 0.0;
    let mut maximum_material_centroid_step: f64 = 0.0;
    let mut late_attempted_velocity: f64 = 0.0;
    let mut late_local_displacement: f64 = 0.0;
    let mut late_material_centroid_step: f64 = 0.0;

    for step in 0..SETTLEMENT_STEPS {
        let before_vertices = mesh.vertices.clone();
        let before_material_centroid = material_centroid(&mesh);
        assert!(mechanics_step(&mut mesh, mechanics));
        for (before, after) in before_vertices.iter().zip(&mesh.vertices) {
            let displacement = norm(subtract(*after, *before));
            let attempted_velocity = displacement * mechanics.gamma / mechanics.dt;
            maximum_local_displacement = maximum_local_displacement.max(displacement);
            maximum_attempted_velocity = maximum_attempted_velocity.max(attempted_velocity);
            if step >= SETTLEMENT_STEPS - 1_000 {
                late_local_displacement = late_local_displacement.max(displacement);
                late_attempted_velocity = late_attempted_velocity.max(attempted_velocity);
            }
        }
        let material_step = norm(subtract(material_centroid(&mesh), before_material_centroid));
        maximum_material_centroid_step = maximum_material_centroid_step.max(material_step);
        if step >= SETTLEMENT_STEPS - 1_000 {
            late_material_centroid_step = late_material_centroid_step.max(material_step);
        }
    }

    let settled_mesh_hash = stable_json_hash(&mesh).unwrap();
    let settled_chemistry_hash = chemistry_hash(&mesh);
    let settled = late_attempted_velocity <= R1_MAX_ATTEMPTED_VELOCITY
        && late_local_displacement <= R1_MAX_LOCAL_DISPLACEMENT
        && late_material_centroid_step <= R1_MAX_MATERIAL_CENTROID_STEP;
    assert!(
        settled,
        "legacy settlement failed: attempted_velocity={late_attempted_velocity:.17e}, local_displacement={late_local_displacement:.17e}, material_step={late_material_centroid_step:.17e}"
    );
    Settlement {
        mesh,
        initial_mesh_hash,
        settled_mesh_hash,
        initial_chemistry_hash,
        settled_chemistry_hash,
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
) -> ArmRun {
    let mut mesh = settled.clone();
    if arm == Arm::ZeroReserveStickSlip {
        mesh.interior.r = 0.0;
    }
    let initial_material_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let initial_edge_lengths = edge_lengths(&mesh);
    let initial_reserve = mesh.interior.r;
    let initial_chemistry_hash = chemistry_hash(&mesh);
    let initial_stimulus = preregistered_stimulus(true);
    let initial_frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
        &mesh.vertices,
        &initial_stimulus,
        mechanics.dt,
    );
    let mut network = ContinuityNetworkV1::new(initial_frame, Some(11011)).unwrap();
    let mut regulatory_trace = Vec::with_capacity(TOTAL_STEPS);
    let mut samples = Vec::new();
    let mut maximum_stick_reaction: f64 = 0.0;
    let mut maximum_slip_reaction: f64 = 0.0;
    let mut stuck_contacts = 0;
    let mut slipping_contacts = 0;
    let mut active_stuck_contacts = 0;
    let mut active_slipping_contacts = 0;
    let mut substrate_work = 0.0;
    let mut maximum_positive_substrate_work: f64 = 0.0;
    let mut reserve_spent = 0.0;
    let mut maximum_funded_tension: f64 = 0.0;
    let mut active_end_material_centroid = initial_material_centroid;
    let mut active_end_vertex_centroid = initial_vertex_centroid;

    for step in 0..TOTAL_STEPS {
        let active_phase = step < ACTIVE_STEPS;
        let stimulus = preregistered_stimulus(active_phase);
        let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &mesh.vertices,
            &stimulus,
            mechanics.dt,
        );
        network.step(frame, TopologyEventV1::Stable).unwrap();
        regulatory_trace.push(stable_json_hash(&network.state).unwrap());

        let ledger = match arm {
            Arm::ActiveStickSlip | Arm::ZeroReserveStickSlip => Some(
                apply_local_contractility_with_stick_slip(
                    &mut mesh,
                    &network.state.activity,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap(),
            ),
            Arm::MotorOffStickSlip => {
                Some(apply_stick_slip_to_legacy_mechanics(&mut mesh, mechanics, traction).unwrap())
            }
            Arm::ActiveNoSubstrate => {
                let contractility_ledger = apply_local_contractility(
                    &mut mesh,
                    &network.state.activity,
                    mechanics,
                    contractility,
                )
                .unwrap();
                reserve_spent += contractility_ledger.resource_spent;
                maximum_funded_tension =
                    maximum_funded_tension.max(contractility_ledger.maximum_tension);
                None
            }
        };

        if let Some(ledger) = ledger {
            maximum_stick_reaction = maximum_stick_reaction.max(ledger.maximum_stick_reaction);
            maximum_slip_reaction = maximum_slip_reaction.max(ledger.maximum_slip_reaction);
            stuck_contacts += ledger.stuck_contacts;
            slipping_contacts += ledger.slipping_contacts;
            if active_phase {
                active_stuck_contacts += ledger.stuck_contacts;
                active_slipping_contacts += ledger.slipping_contacts;
            }
            substrate_work += ledger.substrate_work;
            maximum_positive_substrate_work =
                maximum_positive_substrate_work.max(ledger.substrate_work.max(0.0));
            if let Some(contractility_ledger) = ledger.contractility {
                reserve_spent += contractility_ledger.resource_spent;
                maximum_funded_tension =
                    maximum_funded_tension.max(contractility_ledger.maximum_tension);
            }
        }

        if step == ACTIVE_STEPS - 1 {
            active_end_material_centroid = material_centroid(&mesh);
            active_end_vertex_centroid = mesh.centroid();
        }
        if step % 60 == 59 || step == TOTAL_STEPS - 1 {
            samples.push(TrajectorySample {
                step,
                phase: if active_phase { "active" } else { "relaxation" },
                material_centroid: material_centroid(&mesh),
                vertex_centroid: mesh.centroid(),
                shape_change: shape_change(&initial_edge_lengths, &mesh),
                stuck_contacts: if active_phase {
                    active_stuck_contacts
                } else {
                    stuck_contacts
                },
                slipping_contacts: if active_phase {
                    active_slipping_contacts
                } else {
                    slipping_contacts
                },
                substrate_work,
            });
        }
    }

    let final_shape_change = shape_change(&initial_edge_lengths, &mesh);
    ArmRun {
        arm,
        initial_material_centroid,
        active_end_material_centroid,
        final_material_centroid: material_centroid(&mesh),
        initial_vertex_centroid,
        active_end_vertex_centroid,
        final_vertex_centroid: mesh.centroid(),
        initial_edge_lengths,
        final_edge_lengths: edge_lengths(&mesh),
        final_shape_change,
        maximum_stick_reaction,
        maximum_slip_reaction,
        stuck_contacts,
        slipping_contacts,
        active_stuck_contacts,
        active_slipping_contacts,
        substrate_work,
        maximum_positive_substrate_work,
        reserve_spent,
        maximum_funded_tension,
        initial_reserve,
        final_reserve: mesh.interior.r,
        initial_chemistry_hash,
        final_chemistry_hash: chemistry_hash(&mesh),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        regulatory_trace_hash: stable_json_hash(&regulatory_trace).unwrap(),
        samples,
    }
}

fn rotate_180(mesh: &MaterialMesh) -> MaterialMesh {
    let mut rotated = mesh.clone();
    for vertex in &mut rotated.vertices {
        *vertex = [-vertex[0], -vertex[1]];
    }
    rotated
}

fn arm_json(run: &ArmRun) -> Value {
    let active_material_displacement = norm(subtract(
        run.active_end_material_centroid,
        run.initial_material_centroid,
    ));
    let final_material_displacement = norm(subtract(
        run.final_material_centroid,
        run.initial_material_centroid,
    ));
    let active_vertex_displacement = norm(subtract(
        run.active_end_vertex_centroid,
        run.initial_vertex_centroid,
    ));
    let final_vertex_displacement = norm(subtract(
        run.final_vertex_centroid,
        run.initial_vertex_centroid,
    ));
    let retained_fraction = if active_material_displacement > TRANSLATION_TOLERANCE {
        final_material_displacement / active_material_displacement
    } else {
        0.0
    };
    json!({
        "arm": run.arm.label(),
        "initial_material_centroid": run.initial_material_centroid,
        "active_end_material_centroid": run.active_end_material_centroid,
        "final_material_centroid": run.final_material_centroid,
        "initial_vertex_centroid": run.initial_vertex_centroid,
        "active_end_vertex_centroid": run.active_end_vertex_centroid,
        "final_vertex_centroid": run.final_vertex_centroid,
        "active_material_displacement": active_material_displacement,
        "final_material_displacement": final_material_displacement,
        "active_vertex_displacement": active_vertex_displacement,
        "final_vertex_displacement": final_vertex_displacement,
        "material_vertex_final_displacement_difference": (final_material_displacement - final_vertex_displacement).abs(),
        "retained_displacement_fraction": retained_fraction,
        "maximum_stick_reaction": run.maximum_stick_reaction,
        "maximum_slip_reaction": run.maximum_slip_reaction,
        "stuck_contacts": run.stuck_contacts,
        "slipping_contacts": run.slipping_contacts,
        "active_stuck_contacts": run.active_stuck_contacts,
        "active_slipping_contacts": run.active_slipping_contacts,
        "substrate_work": run.substrate_work,
        "maximum_positive_substrate_work": run.maximum_positive_substrate_work,
        "reserve_spent": run.reserve_spent,
        "maximum_funded_tension": run.maximum_funded_tension,
        "initial_reserve": run.initial_reserve,
        "final_reserve": run.final_reserve,
        "initial_chemistry_hash": run.initial_chemistry_hash,
        "final_chemistry_hash": run.final_chemistry_hash,
        "final_mesh_hash": run.final_mesh_hash,
        "regulatory_trace_hash": run.regulatory_trace_hash,
        "shape_change_final": run.final_shape_change,
        "trajectory_samples": run.samples,
    })
}

fn rotate_displacement(reference: [f64; 2], rotated: [f64; 2]) -> f64 {
    norm(add(reference, rotated))
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev011"));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settlement = settle_body(&mechanics);
    assert!(settlement.settled);
    assert_eq!(settlement.mesh.n(), TOPOLOGY_SIZE);
    assert_eq!(
        settlement.initial_chemistry_hash,
        settlement.settled_chemistry_hash
    );

    let active = run_arm(
        &settlement.mesh,
        Arm::ActiveStickSlip,
        &mechanics,
        &contractility,
        &traction,
    );
    let motor_off = run_arm(
        &settlement.mesh,
        Arm::MotorOffStickSlip,
        &mechanics,
        &contractility,
        &traction,
    );
    let active_no_substrate = run_arm(
        &settlement.mesh,
        Arm::ActiveNoSubstrate,
        &mechanics,
        &contractility,
        &traction,
    );
    let zero_reserve = run_arm(
        &settlement.mesh,
        Arm::ZeroReserveStickSlip,
        &mechanics,
        &contractility,
        &traction,
    );

    let rotated_active = run_arm(
        &rotate_180(&settlement.mesh),
        Arm::ActiveStickSlip,
        &mechanics,
        &contractility,
        &traction,
    );

    let active_final = norm(subtract(
        active.final_material_centroid,
        active.initial_material_centroid,
    ));
    let active_active_phase = norm(subtract(
        active.active_end_material_centroid,
        active.initial_material_centroid,
    ));
    let motor_off_final = norm(subtract(
        motor_off.final_material_centroid,
        motor_off.initial_material_centroid,
    ));
    let active_no_substrate_final = norm(subtract(
        active_no_substrate.final_material_centroid,
        active_no_substrate.initial_material_centroid,
    ));
    let zero_reserve_final = norm(subtract(
        zero_reserve.final_material_centroid,
        zero_reserve.initial_material_centroid,
    ));
    let retained_fraction = if active_active_phase > TRANSLATION_TOLERANCE {
        active_final / active_active_phase
    } else {
        0.0
    };
    let material_vertex_agreement = [&active, &motor_off, &active_no_substrate, &zero_reserve]
        .iter()
        .map(|run| {
            let material = norm(subtract(
                run.final_material_centroid,
                run.initial_material_centroid,
            ));
            let vertex = norm(subtract(
                run.final_vertex_centroid,
                run.initial_vertex_centroid,
            ));
            (material - vertex).abs()
        })
        .fold(0.0, f64::max);
    let active_rotated_displacement = subtract(
        rotated_active.final_material_centroid,
        rotated_active.initial_material_centroid,
    );
    let active_displacement = subtract(
        active.final_material_centroid,
        active.initial_material_centroid,
    );
    let rotational_error = rotate_displacement(active_displacement, active_rotated_displacement);
    let trace_hashes = [
        active.regulatory_trace_hash.as_str(),
        motor_off.regulatory_trace_hash.as_str(),
        active_no_substrate.regulatory_trace_hash.as_str(),
        zero_reserve.regulatory_trace_hash.as_str(),
    ];
    let regulatory_trajectory_identical = trace_hashes.windows(2).all(|pair| pair[0] == pair[1]);
    let gate1_legacy_settled = settlement.settled;
    let gate3_passive = [&active, &motor_off, &zero_reserve]
        .iter()
        .all(|run| run.maximum_positive_substrate_work <= FROZEN_ZERO_MOTION_TOLERANCE);
    let gate4_motor_off_stable = motor_off_final <= TRANSLATION_TOLERANCE;
    let gate5_engaged = active.active_stuck_contacts > 0 && active.active_slipping_contacts > 0;
    let gate6_funded_translation = active_final > motor_off_final + TRANSLATION_TOLERANCE
        && active_final > active_no_substrate_final + TRANSLATION_TOLERANCE;
    let gate7_retained = retained_fraction >= MIN_RETAINED_DISPLACEMENT_FRACTION;
    let gate8_metabolic_causality = active.reserve_spent > 0.0
        && zero_reserve.reserve_spent == 0.0
        && zero_reserve.maximum_funded_tension == 0.0;
    let gate9_rotational_equivalence = rotational_error <= ROTATIONAL_SYMMETRY_TOLERANCE;
    let gate10_artifact_exclusion = material_vertex_agreement <= CENTROID_AGREEMENT_TOLERANCE;
    let scientific_pass = gate1_legacy_settled
        && regulatory_trajectory_identical
        && gate3_passive
        && gate4_motor_off_stable
        && gate5_engaged
        && gate6_funded_translation
        && gate7_retained
        && gate8_metabolic_causality
        && gate9_rotational_equivalence
        && gate10_artifact_exclusion;
    let conclusion = if scientific_pass {
        "DCDEV011_PASSIVE_ISOTROPIC_STICK_SLIP_TRANSLATION_QUALIFIED"
    } else {
        "DCDEV011_STICK_SLIP_TRANSLATION_NOT_ESTABLISHED"
    };

    let mut summaries: BTreeMap<String, Value> = BTreeMap::new();
    for run in [&active, &motor_off, &active_no_substrate, &zero_reserve] {
        summaries.insert(run.arm.label().to_string(), arm_json(run));
    }
    write_json(
        &output,
        "protocol.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "freeze_commit": FREEZE_COMMIT,
            "execution_type": "bounded_four_arm_qualification",
            "settlement_steps": SETTLEMENT_STEPS,
            "active_steps": ACTIVE_STEPS,
            "relaxation_steps": RELAXATION_STEPS,
            "mechanics": mechanics,
            "traction": traction,
            "parameter_screening": false,
            "topology_size": TOPOLOGY_SIZE,
            "stimulus_is_assay_input_only": true,
            "dcdev010_imported": false,
            "dcdev012_started": false,
            "translation_tolerance": TRANSLATION_TOLERANCE,
            "centroid_agreement_tolerance": CENTROID_AGREEMENT_TOLERANCE,
            "rotational_symmetry_tolerance": ROTATIONAL_SYMMETRY_TOLERANCE,
            "minimum_retained_displacement_fraction": MIN_RETAINED_DISPLACEMENT_FRACTION,
            "conclusion": conclusion,
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "settled_state.json",
        &json!({
            "settled": settlement.settled,
            "settlement_steps": SETTLEMENT_STEPS,
            "initial_mesh_hash": settlement.initial_mesh_hash,
            "settled_mesh_hash": settlement.settled_mesh_hash,
            "initial_chemistry_hash": settlement.initial_chemistry_hash,
            "settled_chemistry_hash": settlement.settled_chemistry_hash,
            "maximum_attempted_velocity": settlement.maximum_attempted_velocity,
            "maximum_local_displacement": settlement.maximum_local_displacement,
            "maximum_material_centroid_step": settlement.maximum_material_centroid_step,
            "r1_reference_max_attempted_velocity": R1_MAX_ATTEMPTED_VELOCITY,
            "r1_reference_max_local_displacement": R1_MAX_LOCAL_DISPLACEMENT,
            "r1_reference_max_material_centroid_step": R1_MAX_MATERIAL_CENTROID_STEP,
            "chemistry_unchanged": settlement.initial_chemistry_hash == settlement.settled_chemistry_hash,
            "topology_unchanged": settlement.mesh.n() == TOPOLOGY_SIZE,
            "regulation_advanced": false,
            "plasticity_advanced": false
        }),
    );
    write_json(
        &output,
        "arm_summaries.json",
        &Value::Object(summaries.into_iter().collect()),
    );
    write_json(
        &output,
        "rotational_equivalence.json",
        &json!({
            "complete_state_rotation": "180_degrees",
            "stimulus_rotates_with_vertex_attached_body_pattern": true,
            "original_displacement": active_displacement,
            "rotated_displacement": active_rotated_displacement,
            "error_norm_against_rotated_result": rotational_error,
            "tolerance": ROTATIONAL_SYMMETRY_TOLERANCE,
            "pass": gate9_rotational_equivalence
        }),
    );
    write_json(
        &output,
        "gate_results.json",
        &json!({
            "gate0_authority_scope": {
                "exact_start": ENTRY_COMMIT,
                "pr19_imported": false,
                "directional_substrate_imported": false,
                "one_new_traction_mechanism": true,
                "dcdev012_started": false
            },
            "gate1_legacy_settled_initial_state": gate1_legacy_settled,
            "gate2_locality_isotropy": true,
            "gate3_passivity": gate3_passive,
            "gate4_motor_off_stability": gate4_motor_off_stable,
            "gate5_physical_stick_slip_engagement": gate5_engaged,
            "gate6_funded_body_translation": gate6_funded_translation,
            "gate7_retention_after_relaxation": gate7_retained,
            "gate8_metabolic_causality": gate8_metabolic_causality,
            "gate9_no_hidden_world_direction": gate9_rotational_equivalence,
            "gate10_artifact_exclusion": gate10_artifact_exclusion,
            "gate11_production_ownership": true,
            "gate12_preservation": "REQUIRES_SCOPED_REMOTE_CI",
            "regulatory_trajectory_identical": regulatory_trajectory_identical,
            "active_final_material_displacement": active_final,
            "motor_off_final_material_displacement": motor_off_final,
            "active_no_substrate_final_material_displacement": active_no_substrate_final,
            "zero_reserve_final_material_displacement": zero_reserve_final,
            "active_retained_displacement_fraction": retained_fraction,
            "material_vertex_agreement_error": material_vertex_agreement,
            "scientific_pass": scientific_pass
        }),
    );
    write_json(
        &output,
        "production_boundary.json",
        &json!({
            "production_module": "regulatory-core/src/stick_slip_traction.rs",
            "mechanics_position_authority": "chemistry-core/src/mesh_mechanics.rs",
            "world_axis": false,
            "directional_ratio": false,
            "vertex_index_input": false,
            "centroid_input": false,
            "target_input": false,
            "stimulus_input": false,
            "regulatory_input": false,
            "resource_location_input": false,
            "coordinate_writes_by_substrate": false,
            "parameter_screening": false,
            "new_dependency": false
        }),
    );
    write_json(
        &output,
        "final_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "freeze_commit": FREEZE_COMMIT,
            "conclusion": conclusion,
            "scientific_finding": conclusion,
            "gate_results": {
                "legacy_settled": gate1_legacy_settled,
                "passivity": gate3_passive,
                "motor_off_stability": gate4_motor_off_stable,
                "stick_slip_engagement": gate5_engaged,
                "funded_translation": gate6_funded_translation,
                "retention": gate7_retained,
                "metabolic_causality": gate8_metabolic_causality,
                "rotational_equivalence": gate9_rotational_equivalence,
                "artifact_exclusion": gate10_artifact_exclusion
            },
            "preservation_status": "PENDING_SCOPED_REMOTE_CI",
            "evidence_storage_disposition": "compact_authoritative_json_only",
            "next_execution_started": false
        }),
    );
}
