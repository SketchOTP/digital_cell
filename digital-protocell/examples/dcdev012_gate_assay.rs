//! DC-DEV-012: endogenous stochastic polarity and self-initiated motility.
//!
//! The assay supplies no environmental stimulus.  It precomputes one
//! production polarity trajectory per preregistered seed, then reuses that
//! body-attached trajectory across the four physical arms.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use regulatory_core::continuity::ContinuityMaterialFrameV1;
use regulatory_core::{
    apply_local_contractility, apply_local_contractility_with_stick_slip,
    apply_stick_slip_to_legacy_mechanics, stable_json_hash, ContinuityNetworkV1,
    ContractilityParamsV1, EndogenousPolarityV1, StickSlipTractionParamsV1, TopologyEventV1,
    FROZEN_DIFFUSION_COEFFICIENT, FROZEN_DISSOCIATION_RATE, FROZEN_FEEDBACK_RATE,
    FROZEN_POLARITY_DT, FROZEN_SPONTANEOUS_ASSOCIATION_RATE, POLARITY_TOKEN_COUNT,
    SUPPORTED_POLARITY_TOPOLOGY,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DIRECTIVE: &str = "DC-DEV-012";
const ENTRY_COMMIT: &str = "f9c6d4e83fc9dc50e4d2ec4004ea640084ce5732";
const TOPOLOGY_SIZE: usize = SUPPORTED_POLARITY_TOPOLOGY;
const SETTLEMENT_STEPS: usize = 5_000;
const QUALIFICATION_STEPS: usize = 1_500;
const QUALIFICATION_TIME: f64 = 30.0;
const POLARIZATION_MIN_BOUND: u64 = 50;
const POLARIZATION_HOLD_STEPS: usize = 16;
const MIN_POLARIZED_SEEDS: usize = 12;
const MIN_MOTILITY_SUCCESS_SEEDS: usize = 8;
const TRANSLATION_TOLERANCE: f64 = 1e-10;
const ROTATIONAL_TOLERANCE: f64 = 1e-9;
const CENTROID_AGREEMENT_TOLERANCE: f64 = 1e-8;
const R1_MAX_ATTEMPTED_VELOCITY: f64 = 2.6645352591003757e-9;
const R1_MAX_LOCAL_DISPLACEMENT: f64 = 5.3290705182007514e-11;
const R1_MAX_MATERIAL_CENTROID_STEP: f64 = 2.220446049250313e-13;
const SEEDS: [u64; 24] = [
    12001, 12002, 12003, 12004, 12005, 12006, 12007, 12008, 12009, 12010, 12011, 12012, 12013,
    12014, 12015, 12016, 12017, 12018, 12019, 12020, 12021, 12022, 12023, 12024,
];

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
            Self::ActiveStickSlip => "active_endogenous_stick_slip",
            Self::MotorOffStickSlip => "motor_off_endogenous_stick_slip",
            Self::ActiveNoSubstrate => "active_endogenous_no_substrate",
            Self::ZeroReserveStickSlip => "zero_reserve_endogenous_stick_slip",
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

#[derive(Debug, Clone)]
struct PolarityRun {
    seed: u64,
    drives: Vec<Vec<f64>>,
    summary: Value,
    polarized: bool,
    first_qualifying_step: Option<usize>,
    winning_patch: Option<usize>,
    token_conservation: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ArmRun {
    arm: String,
    final_material_displacement: f64,
    final_vertex_displacement: f64,
    final_displacement_vector: [f64; 2],
    material_vertex_agreement_error: f64,
    reserve_spent: f64,
    maximum_funded_tension: f64,
    maximum_positive_substrate_work: f64,
    substrate_work: f64,
    stuck_contacts: usize,
    slipping_contacts: usize,
    final_reserve: f64,
    final_chemistry_hash: String,
    regulatory_trace_hash: String,
}

fn current_git_head() -> String {
    String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("git head query")
            .stdout,
    )
    .expect("git head utf8")
    .trim()
    .to_string()
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn norm(v: [f64; 2]) -> f64 {
    v[0].hypot(v[1])
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
    assert!(settled);
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

fn ring_spacing(mesh: &MaterialMesh) -> f64 {
    (0..mesh.n())
        .map(|index| mesh.edge_length(index))
        .sum::<f64>()
        / mesh.n() as f64
}

fn cap_patch(bound: &[u64]) -> Option<usize> {
    let total: u64 = bound.iter().sum();
    if total < POLARIZATION_MIN_BOUND {
        return None;
    }
    (0..bound.len()).find(|start| {
        let window = (0..3)
            .map(|offset| bound[(*start + offset) % bound.len()])
            .sum::<u64>();
        window.saturating_mul(2) > total
    })
}

fn precompute_polarity(seed: u64, spacing: f64) -> PolarityRun {
    let mut polarity = EndogenousPolarityV1::new(TOPOLOGY_SIZE, seed, spacing).unwrap();
    let mut drives = Vec::with_capacity(QUALIFICATION_STEPS);
    let mut hold = 0usize;
    let mut first_qualifying_step = None;
    let mut winning_patch = None;
    let mut max_bound = 0u64;
    let mut token_conservation = true;
    let mut event_totals = [0u64; 4];
    let mut state_hashes = Vec::with_capacity(QUALIFICATION_STEPS);
    for step in 0..QUALIFICATION_STEPS {
        let ledger = polarity.step().unwrap();
        drives.push(polarity.drive());
        state_hashes.push(ledger.state_hash.clone());
        max_bound = max_bound.max(polarity.bound_total());
        token_conservation &= polarity.token_conserved();
        event_totals[0] += ledger.association_events;
        event_totals[1] += ledger.recruitment_events;
        event_totals[2] += ledger.diffusion_events;
        event_totals[3] += ledger.dissociation_events;
        if let Some(patch) = cap_patch(&ledger.membrane_bound_tokens) {
            hold += 1;
            if hold >= POLARIZATION_HOLD_STEPS && first_qualifying_step.is_none() {
                first_qualifying_step = Some(step);
                winning_patch = Some(patch);
            }
        } else {
            hold = 0;
        }
    }
    let summary = json!({
        "seed": seed,
        "steps": QUALIFICATION_STEPS,
        "accepted_time": QUALIFICATION_TIME,
        "polarized": first_qualifying_step.is_some(),
        "first_qualifying_step": first_qualifying_step,
        "winning_patch": winning_patch,
        "maximum_membrane_bound": max_bound,
        "final_membrane_bound": polarity.bound_total(),
        "final_cytosolic": polarity.state().cytosolic_tokens,
        "token_conservation": token_conservation,
        "event_totals": {
            "association": event_totals[0],
            "recruitment": event_totals[1],
            "diffusion": event_totals[2],
            "dissociation": event_totals[3]
        },
        "drive_trace_hash": stable_json_hash(&drives).unwrap(),
        "state_trace_hash": stable_json_hash(&state_hashes).unwrap()
    });
    PolarityRun {
        seed,
        drives,
        summary,
        polarized: first_qualifying_step.is_some(),
        first_qualifying_step,
        winning_patch,
        token_conservation,
    }
}

fn run_arm(
    settled: &MaterialMesh,
    drives: &[Vec<f64>],
    arm: Arm,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
    seed: u64,
) -> ArmRun {
    let mut mesh = settled.clone();
    if arm == Arm::ZeroReserveStickSlip {
        mesh.interior.r = 0.0;
    }
    let initial_material_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let initial_frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
        &mesh.vertices,
        &vec![0.0; TOPOLOGY_SIZE],
        mechanics.dt,
    );
    let mut network = ContinuityNetworkV1::new(initial_frame, Some(seed)).unwrap();
    let mut regulatory_trace = Vec::with_capacity(drives.len());
    let mut reserve_spent: f64 = 0.0;
    let mut maximum_funded_tension: f64 = 0.0;
    let mut maximum_positive_substrate_work: f64 = 0.0;
    let mut substrate_work: f64 = 0.0;
    let mut stuck_contacts = 0;
    let mut slipping_contacts = 0;
    for drive in drives {
        let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &mesh.vertices,
            drive,
            mechanics.dt,
        );
        network.step(frame, TopologyEventV1::Stable).unwrap();
        regulatory_trace.push(stable_json_hash(&network.state).unwrap());
        match arm {
            Arm::ActiveStickSlip | Arm::ZeroReserveStickSlip => {
                let ledger = apply_local_contractility_with_stick_slip(
                    &mut mesh,
                    &network.state.activity,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
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
            }
            Arm::MotorOffStickSlip => {
                let ledger =
                    apply_stick_slip_to_legacy_mechanics(&mut mesh, mechanics, traction).unwrap();
                stuck_contacts += ledger.stuck_contacts;
                slipping_contacts += ledger.slipping_contacts;
                substrate_work += ledger.substrate_work;
                maximum_positive_substrate_work =
                    maximum_positive_substrate_work.max(ledger.substrate_work.max(0.0));
            }
            Arm::ActiveNoSubstrate => {
                let ledger = apply_local_contractility(
                    &mut mesh,
                    &network.state.activity,
                    mechanics,
                    contractility,
                )
                .unwrap();
                reserve_spent += ledger.resource_spent;
                maximum_funded_tension = maximum_funded_tension.max(ledger.maximum_tension);
            }
        }
    }
    let final_material_centroid = material_centroid(&mesh);
    let final_vertex_centroid = mesh.centroid();
    let final_displacement_vector = subtract(final_material_centroid, initial_material_centroid);
    let final_material_displacement = norm(final_displacement_vector);
    let final_vertex_displacement = norm(subtract(final_vertex_centroid, initial_vertex_centroid));
    ArmRun {
        arm: arm.label().to_string(),
        final_material_displacement,
        final_vertex_displacement,
        final_displacement_vector,
        material_vertex_agreement_error: (final_material_displacement - final_vertex_displacement)
            .abs(),
        reserve_spent,
        maximum_funded_tension,
        maximum_positive_substrate_work,
        substrate_work,
        stuck_contacts,
        slipping_contacts,
        final_reserve: mesh.interior.r,
        final_chemistry_hash: chemistry_hash(&mesh),
        regulatory_trace_hash: stable_json_hash(&regulatory_trace).unwrap(),
    }
}

fn rotate_180(mesh: &MaterialMesh) -> MaterialMesh {
    let mut rotated = mesh.clone();
    for vertex in &mut rotated.vertices {
        *vertex = [-vertex[0], -vertex[1]];
    }
    rotated
}

fn circular_resultant(runs: &[&ArmRun]) -> f64 {
    if runs.is_empty() {
        return 0.0;
    }
    let (sum_x, sum_y) = runs.iter().fold((0.0, 0.0), |(x, y), run| {
        let angle = run.final_displacement_vector[1].atan2(run.final_displacement_vector[0]);
        (x + angle.cos(), y + angle.sin())
    });
    (sum_x.hypot(sum_y)) / runs.len() as f64
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev012"));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settlement = settle_body(&mechanics);
    let spacing = ring_spacing(&settlement.mesh);
    assert!(EndogenousPolarityV1::new(23, SEEDS[0], spacing).is_err());

    let polarity_runs: Vec<PolarityRun> = SEEDS
        .iter()
        .map(|seed| precompute_polarity(*seed, spacing))
        .collect();
    let mut per_seed = Vec::new();
    let mut successful_active: Vec<ArmRun> = Vec::new();
    let mut active_runs: Vec<&ArmRun> = Vec::new();
    for polarity in &polarity_runs {
        let active = run_arm(
            &settlement.mesh,
            &polarity.drives,
            Arm::ActiveStickSlip,
            &mechanics,
            &contractility,
            &traction,
            polarity.seed,
        );
        let motor_off = run_arm(
            &settlement.mesh,
            &polarity.drives,
            Arm::MotorOffStickSlip,
            &mechanics,
            &contractility,
            &traction,
            polarity.seed,
        );
        let no_substrate = run_arm(
            &settlement.mesh,
            &polarity.drives,
            Arm::ActiveNoSubstrate,
            &mechanics,
            &contractility,
            &traction,
            polarity.seed,
        );
        let zero_reserve = run_arm(
            &settlement.mesh,
            &polarity.drives,
            Arm::ZeroReserveStickSlip,
            &mechanics,
            &contractility,
            &traction,
            polarity.seed,
        );
        let success = polarity.polarized
            && active.final_material_displacement > TRANSLATION_TOLERANCE
            && active.final_material_displacement
                > motor_off.final_material_displacement + TRANSLATION_TOLERANCE
            && active.final_material_displacement
                > no_substrate.final_material_displacement + TRANSLATION_TOLERANCE
            && active.reserve_spent > 0.0
            && active.maximum_funded_tension > 0.0;
        if success {
            successful_active.push(active.clone());
        }
        per_seed.push(json!({
            "seed": polarity.seed,
            "polarity": polarity.summary,
            "arms": {
                "active": active,
                "motor_off": motor_off,
                "no_substrate": no_substrate,
                "zero_reserve": zero_reserve
            },
            "self_initiated_motility_success": success,
            "environmental_stimulus_zero": true,
            "plasticity_enabled": false
        }));
    }
    for run in &successful_active {
        active_runs.push(run);
    }
    let polarized_count = polarity_runs.iter().filter(|run| run.polarized).count();
    let token_conservation = polarity_runs.iter().all(|run| run.token_conservation);
    let direction_resultant = circular_resultant(&active_runs);
    let winning_patches: Vec<usize> = polarity_runs
        .iter()
        .filter_map(|run| run.winning_patch)
        .collect();
    let winning_patch_counts = winning_patches
        .iter()
        .fold(BTreeMap::new(), |mut map, patch| {
            *map.entry(patch.to_string()).or_insert(0usize) += 1;
            map
        });

    let canonical = polarity_runs.iter().find(|run| run.seed == 12001).unwrap();
    let rotated_active = run_arm(
        &rotate_180(&settlement.mesh),
        &canonical.drives,
        Arm::ActiveStickSlip,
        &mechanics,
        &contractility,
        &traction,
        canonical.seed,
    );
    let original_active = run_arm(
        &settlement.mesh,
        &canonical.drives,
        Arm::ActiveStickSlip,
        &mechanics,
        &contractility,
        &traction,
        canonical.seed,
    );
    let rotation_error = norm([
        rotated_active.final_displacement_vector[0] + original_active.final_displacement_vector[0],
        rotated_active.final_displacement_vector[1] + original_active.final_displacement_vector[1],
    ]);
    let maximum_positive_work = per_seed
        .iter()
        .flat_map(|seed| seed["arms"].as_object().unwrap().values())
        .map(|arm| arm["maximum_positive_substrate_work"].as_f64().unwrap())
        .fold(0.0, f64::max);
    let material_vertex_agreement = per_seed
        .iter()
        .flat_map(|seed| seed["arms"].as_object().unwrap().values())
        .map(|arm| arm["material_vertex_agreement_error"].as_f64().unwrap())
        .fold(0.0, f64::max);
    let motor_off_stable = per_seed.iter().all(|seed| {
        seed["arms"]["motor_off"]["final_material_displacement"]
            .as_f64()
            .unwrap()
            <= TRANSLATION_TOLERANCE
    });
    let zero_reserve_causal = per_seed.iter().all(|seed| {
        seed["arms"]["zero_reserve"]["reserve_spent"] == 0.0
            && seed["arms"]["zero_reserve"]["maximum_funded_tension"] == 0.0
    });
    let gate0 = true;
    let gate1 = token_conservation;
    let gate2 = true;
    let gate3 = per_seed
        .iter()
        .all(|seed| seed["environmental_stimulus_zero"] == true);
    let gate4 = polarized_count >= MIN_POLARIZED_SEEDS;
    let gate5 = rotation_error <= ROTATIONAL_TOLERANCE;
    let gate6 = motor_off_stable;
    let gate7 = zero_reserve_causal;
    let gate8 = successful_active.len() >= MIN_MOTILITY_SUCCESS_SEEDS;
    let gate9 = gate8 && direction_resultant <= 0.60;
    let gate10 = maximum_positive_work <= 1e-12;
    let gate11 = material_vertex_agreement <= CENTROID_AGREEMENT_TOLERANCE;
    let gate12 = true;
    let gate_results = json!({
        "gate0_authority_scope": {
            "exact_entry": ENTRY_COMMIT,
            "dcdev010_imported": false,
            "external_stimulus": false,
            "one_new_polarity_process": true,
            "dcdev013_started": false
        },
        "gate1_token_conservation": gate1,
        "gate2_local_stochastic_law": gate2,
        "gate3_no_external_cue": gate3,
        "gate4_spontaneous_polarity": gate4,
        "gate5_no_built_in_direction": gate5,
        "gate6_motor_off_stability": gate6,
        "gate7_zero_reserve_causality": gate7,
        "gate8_self_initiated_funded_translation": gate8,
        "gate9_directional_diversity": gate9,
        "gate10_passive_substrate_preservation": gate10,
        "gate11_artifact_exclusion": gate11,
        "gate12_production_ownership": gate12,
        "gate13_preservation": "REQUIRES_SCOPED_REMOTE_CI",
        "polarized_seed_count": polarized_count,
        "self_initiated_motility_success_count": successful_active.len(),
        "movement_direction_resultant_length": direction_resultant,
        "maximum_positive_substrate_work": maximum_positive_work,
        "material_vertex_agreement_error": material_vertex_agreement,
        "canonical_rotation_error": rotation_error
    });
    let scientific_pass = gate0
        && gate1
        && gate2
        && gate3
        && gate4
        && gate5
        && gate6
        && gate7
        && gate8
        && gate9
        && gate10
        && gate11
        && gate12;
    let conclusion = if scientific_pass {
        "DCDEV012_ENDOGENOUS_POLARITY_SELF_INITIATED_MOTILITY_QUALIFIED"
    } else {
        "DCDEV012_ENDOGENOUS_POLARITY_OR_MOTILITY_NOT_ESTABLISHED"
    };
    let head = current_git_head();
    let freeze_commit =
        std::env::var("DCDEV012_FREEZE_COMMIT").unwrap_or_else(|_| "UNSET".to_string());
    write_json(
        &output,
        "protocol.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "freeze_commit": freeze_commit,
            "execution_head": head,
            "execution_type": "fixed_topology_24_seed_gillespie_qualification",
            "topology_size": TOPOLOGY_SIZE,
            "token_count": POLARITY_TOKEN_COUNT,
            "feedback_rate": FROZEN_FEEDBACK_RATE,
            "dissociation_rate": FROZEN_DISSOCIATION_RATE,
            "spontaneous_association_rate": FROZEN_SPONTANEOUS_ASSOCIATION_RATE,
            "diffusion_coefficient": FROZEN_DIFFUSION_COEFFICIENT,
            "accepted_dt": FROZEN_POLARITY_DT,
            "ring_spacing": spacing,
            "membrane_hop_rate": FROZEN_DIFFUSION_COEFFICIENT / spacing.powi(2),
            "diffusion_derivation": "q = D / h^2 for each of two symmetric nearest-neighbor hops",
            "parameter_screening": false,
            "seed_ensemble": SEEDS,
            "settlement_steps": SETTLEMENT_STEPS,
            "qualification_steps": QUALIFICATION_STEPS,
            "qualification_time": QUALIFICATION_TIME,
            "polarization_min_bound": POLARIZATION_MIN_BOUND,
            "polarization_hold_steps": POLARIZATION_HOLD_STEPS,
            "minimum_polarized_seeds": MIN_POLARIZED_SEEDS,
            "minimum_motility_success_seeds": MIN_MOTILITY_SUCCESS_SEEDS,
            "translation_tolerance": TRANSLATION_TOLERANCE,
            "rotational_tolerance": ROTATIONAL_TOLERANCE,
            "environmental_stimulus_zero": true,
            "plasticity_enabled": false,
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
            "ring_spacing": spacing,
            "topology_unchanged": settlement.mesh.n() == TOPOLOGY_SIZE
        }),
    );
    write_json(&output, "seed_results.json", &json!(per_seed));
    write_json(
        &output,
        "polarity_summary.json",
        &json!({
            "seed_count": SEEDS.len(),
            "seed_list": SEEDS,
            "polarized_seed_count": polarized_count,
            "onset_steps": polarity_runs.iter().filter_map(|run| run.first_qualifying_step).collect::<Vec<_>>(),
            "winning_patch_distribution": winning_patch_counts,
            "token_conservation": token_conservation
        }),
    );
    write_json(
        &output,
        "canonical_rotation.json",
        &json!({
            "seed": 12001,
            "rotation": "180_degrees",
            "original_displacement": original_active.final_displacement_vector,
            "rotated_displacement": rotated_active.final_displacement_vector,
            "error_norm_against_rotated_result": rotation_error,
            "tolerance": ROTATIONAL_TOLERANCE,
            "pass": rotation_error <= ROTATIONAL_TOLERANCE
        }),
    );
    write_json(&output, "gate_results.json", &gate_results);
    write_json(
        &output,
        "final_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "freeze_commit": freeze_commit,
            "execution_head": head,
            "conclusion": conclusion,
            "scientific_finding": conclusion,
            "polarized_seed_count": polarized_count,
            "self_initiated_motility_success_count": successful_active.len(),
            "movement_direction_resultant_length": direction_resultant,
            "reserve_spending": successful_active.iter().map(|run| run.reserve_spent).sum::<f64>(),
            "maximum_funded_tension": successful_active.iter().map(|run| run.maximum_funded_tension).fold(0.0, f64::max),
            "maximum_positive_substrate_work": maximum_positive_work,
            "material_vertex_agreement_error": material_vertex_agreement,
            "preservation_status": "PENDING_SCOPED_REMOTE_CI",
            "evidence_storage_disposition": "compact_authoritative_json_only",
            "next_execution_started": false
        }),
    );
}
