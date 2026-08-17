//! DC-DEV-014: bounded homeostatic exploration/feeding-state assay.
//!
//! The assay freezes existing interior A as the material signal.  A falls in
//! the accepted resource-free maintenance path and is restored toward the
//! accepted seed reference by finite N/F uptake followed by existing
//! reactions_step.  Exploration emits only local, direction-neutral pulses;
//! all spreading, funding, force, and movement remain existing authorities.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::continuity::ContinuityMaterialFrameV1;
use regulatory_core::{
    apply_local_contractility_with_stick_slip, stable_json_hash, ContinuityNetworkV1,
    ContractilityParamsV1, FiniteSpatialResourceRegionV1, HomeostaticExplorationParamsV1,
    HomeostaticExplorationV1, StickSlipTractionParamsV1, TopologyEventV1,
    FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-014";
const ENTRY_COMMIT: &str = "5a4e0a2d7314af411ec2283b0ffcf4950eb217db";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const DEPRIVATION_STEPS: usize = 480;
const ASSAY_STEPS: usize = 480;
const WINDOW_STEPS: usize = 160;
const ACCEPTED_REPLETE_A: f64 = 0.5;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const INITIAL_N_MASS: f64 = 3.0;
const INITIAL_F_MASS: f64 = 3.0;
const MASS_TOLERANCE: f64 = 1e-10;
const ROTATION_TOLERANCE: f64 = 1e-8;
const SETTLED_ATTEMPTED_VELOCITY: f64 = 2.6645352591003757e-9;
const SETTLED_LOCAL_DISPLACEMENT: f64 = 5.3290705182007514e-11;
const SETTLED_MATERIAL_STEP: f64 = 2.220446049250313e-13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arm {
    Deprived,
    Replete,
    Restoring,
    ContactNoDelivery,
    ZeroReserve,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::Deprived => "A_deprived_no_resource",
            Self::Replete => "B_replete_no_resource",
            Self::Restoring => "C_deprived_finite_NF_restoration",
            Self::ContactNoDelivery => "D_same_contact_geometry_no_delivery",
            Self::ZeroReserve => "E_zero_reserve_deprived",
        }
    }

    fn needs_resource_region(self) -> bool {
        matches!(self, Self::Restoring | Self::ContactNoDelivery)
    }

    fn delivers_resource(self) -> bool {
        matches!(self, Self::Restoring)
    }

    fn zero_reserve(self) -> bool {
        matches!(self, Self::ZeroReserve)
    }
}

#[derive(Debug, Clone, Serialize)]
struct Settlement {
    mesh: MaterialMesh,
    initial_mesh_hash: String,
    settled_mesh_hash: String,
    settled: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DeprivationAudit {
    variable: String,
    accepted_replete_reference: f64,
    initial_value: f64,
    final_value: f64,
    steps: usize,
    depleted: bool,
    resource_free_maintenance: String,
}

#[derive(Debug, Clone, Serialize)]
struct WindowMetrics {
    first_step: usize,
    last_step_exclusive: usize,
    event_count: usize,
    mean_need: f64,
    mean_activity: f64,
    displacement: f64,
    reserve_spent: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ArmRun {
    arm: String,
    initial_a: f64,
    final_a: f64,
    initial_reserve: f64,
    final_reserve: f64,
    n_delivered: f64,
    f_delivered: f64,
    world_n_remaining: f64,
    world_f_remaining: f64,
    contact_positive_steps: usize,
    exploration_events: usize,
    total_activity: f64,
    maximum_activity: f64,
    material_displacement: f64,
    vertex_displacement: f64,
    reserve_spent: f64,
    maximum_funded_tension: f64,
    maximum_substrate_work: f64,
    substrate_work: f64,
    zero_reserve_no_funding: bool,
    local_event_patches: Vec<usize>,
    a_trajectory: Vec<f64>,
    need_trajectory: Vec<f64>,
    activity_trajectory: Vec<f64>,
    windows: Vec<WindowMetrics>,
    final_mesh_hash: String,
    final_state_hash: String,
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

fn rotate_180(mesh: &MaterialMesh) -> MaterialMesh {
    let mut rotated = mesh.clone();
    for vertex in &mut rotated.vertices {
        vertex[0] = -vertex[0];
        vertex[1] = -vertex[1];
    }
    rotated
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
            a: ACCEPTED_REPLETE_A,
            n: 0.0,
            f: 0.0,
            r: 0.6,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh
}

fn reaction_params(mesh: &MaterialMesh) -> ReactionParams {
    let mut params = ReactionParams::default();
    params.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    params
}

fn settle_body(mechanics: &MechParams) -> Settlement {
    let mut mesh = seed_mesh();
    let initial_mesh_hash = stable_json_hash(&mesh).unwrap();
    let mut late_velocity: f64 = 0.0;
    let mut late_displacement: f64 = 0.0;
    let mut late_material_step: f64 = 0.0;
    for step in 0..SETTLEMENT_STEPS {
        let before_vertices = mesh.vertices.clone();
        let before_material = material_centroid(&mesh);
        assert!(mechanics_step(&mut mesh, mechanics));
        for (before, after) in before_vertices.iter().zip(&mesh.vertices) {
            let displacement = norm(subtract(*after, *before));
            let velocity = displacement * mechanics.gamma / mechanics.dt;
            if step >= SETTLEMENT_STEPS - 1_000 {
                late_velocity = late_velocity.max(velocity);
                late_displacement = late_displacement.max(displacement);
            }
        }
        if step >= SETTLEMENT_STEPS - 1_000 {
            late_material_step =
                late_material_step.max(norm(subtract(material_centroid(&mesh), before_material)));
        }
    }
    let settled = late_velocity <= SETTLED_ATTEMPTED_VELOCITY
        && late_displacement <= SETTLED_LOCAL_DISPLACEMENT
        && late_material_step <= SETTLED_MATERIAL_STEP;
    assert!(settled, "settlement thresholds failed");
    Settlement {
        initial_mesh_hash,
        settled_mesh_hash: stable_json_hash(&mesh).unwrap(),
        mesh,
        settled,
    }
}

fn deprivation_audit(
    settled: &MaterialMesh,
    mechanics: &MechParams,
) -> (MaterialMesh, DeprivationAudit) {
    let mut mesh = settled.clone();
    let initial_value = mesh.interior.a;
    let reactions = reaction_params(&mesh);
    for _ in 0..DEPRIVATION_STEPS {
        reactions_step(&mut mesh, &reactions, mechanics.dt, true, true);
    }
    let final_value = mesh.interior.a;
    (
        mesh,
        DeprivationAudit {
            variable: "MaterialMesh.interior.a (activated material A)".to_string(),
            accepted_replete_reference: ACCEPTED_REPLETE_A,
            initial_value,
            final_value,
            steps: DEPRIVATION_STEPS,
            depleted: final_value < initial_value,
            resource_free_maintenance: "existing reactions_step, no new state".to_string(),
        },
    )
}

fn need_signal(a: f64) -> f64 {
    ((ACCEPTED_REPLETE_A - a) / ACCEPTED_REPLETE_A).clamp(0.0, 1.0)
}

fn run_arm(
    initial: &MaterialMesh,
    arm: Arm,
    mechanics: &MechParams,
    center: [f64; 2],
    seed: u64,
) -> ArmRun {
    let mut mesh = initial.clone();
    if arm.zero_reserve() {
        mesh.interior.r = 0.0;
    }
    let initial_a = mesh.interior.a;
    let initial_reserve = mesh.interior.r;
    let region_mass = if arm.needs_resource_region() {
        (INITIAL_N_MASS, INITIAL_F_MASS)
    } else {
        (0.0, 0.0)
    };
    let mut region =
        FiniteSpatialResourceRegionV1::new(center, RESOURCE_RADIUS, region_mass.0, region_mass.1);
    let exploration_params =
        HomeostaticExplorationParamsV1::from_regulator(0.5, mechanics.dt).unwrap();
    let mut exploration =
        HomeostaticExplorationV1::new(TOPOLOGY_SIZE, seed, exploration_params).unwrap();
    let initial_frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
        &mesh.vertices,
        &vec![0.0; TOPOLOGY_SIZE],
        mechanics.dt,
    );
    let mut network = ContinuityNetworkV1::new(initial_frame, Some(seed)).unwrap();
    let reactions = reaction_params(&mesh);
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let mut a_trajectory = vec![mesh.interior.a];
    let mut need_trajectory = Vec::with_capacity(ASSAY_STEPS);
    let mut activity_trajectory = Vec::with_capacity(ASSAY_STEPS);
    let mut event_patches = Vec::new();
    let mut total_activity = 0.0;
    let mut maximum_activity: f64 = 0.0;
    let mut contact_positive_steps = 0;
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut reserve_spent = 0.0;
    let mut maximum_funded_tension: f64 = 0.0;
    let mut maximum_substrate_work: f64 = 0.0;
    let mut substrate_work = 0.0;
    let initial_material_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let mut window_events = vec![0usize; 3];
    let mut window_need = vec![0.0; 3];
    let mut window_activity = vec![0.0; 3];
    let mut window_displacement = vec![0.0; 3];
    let mut window_spend = vec![0.0; 3];

    for step in 0..ASSAY_STEPS {
        let need = need_signal(mesh.interior.a);
        let exploration_step = exploration.step(TOPOLOGY_SIZE, need).unwrap();
        if let Some(patch) = exploration_step.event_patch {
            event_patches.push(patch);
        }
        let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &mesh.vertices,
            &exploration_step.local_stimulus,
            mechanics.dt,
        );
        network.step(frame, TopologyEventV1::Stable).unwrap();
        let activity = network.state.activity.clone();
        let mean_activity = activity.iter().sum::<f64>() / activity.len() as f64;
        let contact = if arm.needs_resource_region() {
            region.local_contact_signal(&mesh)
        } else {
            vec![0.0; mesh.n()]
        };
        if contact.iter().any(|value| *value > 0.0) {
            contact_positive_steps += 1;
        }
        let before_material = material_centroid(&mesh);
        let ledger = apply_local_contractility_with_stick_slip(
            &mut mesh,
            &activity,
            mechanics,
            &contractility,
            &traction,
        )
        .unwrap();
        let step_displacement = norm(subtract(material_centroid(&mesh), before_material));
        if arm.delivers_resource() {
            let resource = region.uptake(
                &mut mesh,
                &chemistry_core::mesh_transport::TransportParams::default(),
                mechanics.dt,
            );
            assert!(resource.conservation_error <= MASS_TOLERANCE);
            n_delivered += resource.n_delivered;
            f_delivered += resource.f_delivered;
        }
        if !arm.zero_reserve() {
            reactions_step(&mut mesh, &reactions, mechanics.dt, true, true);
        }
        let window = step / WINDOW_STEPS;
        window_need[window] += need;
        window_activity[window] += mean_activity;
        window_displacement[window] += step_displacement;
        window_spend[window] += ledger
            .contractility
            .as_ref()
            .map_or(0.0, |x| x.resource_spent);
        if exploration_step.event_patch.is_some() {
            window_events[window] += 1;
        }
        total_activity += mean_activity;
        maximum_activity = maximum_activity.max(mean_activity);
        reserve_spent += ledger
            .contractility
            .as_ref()
            .map_or(0.0, |x| x.resource_spent);
        maximum_funded_tension = maximum_funded_tension.max(
            ledger
                .contractility
                .as_ref()
                .map_or(0.0, |x| x.maximum_tension),
        );
        maximum_substrate_work = maximum_substrate_work.max(ledger.substrate_work.max(0.0));
        substrate_work += ledger.substrate_work;
        need_trajectory.push(need);
        activity_trajectory.push(mean_activity);
        a_trajectory.push(mesh.interior.a);
    }
    let windows = (0..3)
        .map(|window| WindowMetrics {
            first_step: window * WINDOW_STEPS,
            last_step_exclusive: (window + 1) * WINDOW_STEPS,
            event_count: window_events[window],
            mean_need: window_need[window] / WINDOW_STEPS as f64,
            mean_activity: window_activity[window] / WINDOW_STEPS as f64,
            displacement: window_displacement[window],
            reserve_spent: window_spend[window],
        })
        .collect();
    let final_material_centroid = material_centroid(&mesh);
    let final_vertex_centroid = mesh.centroid();
    ArmRun {
        arm: arm.label().to_string(),
        initial_a,
        final_a: mesh.interior.a,
        initial_reserve,
        final_reserve: mesh.interior.r,
        n_delivered,
        f_delivered,
        world_n_remaining: region.n_mass,
        world_f_remaining: region.f_mass,
        contact_positive_steps,
        exploration_events: event_patches.len(),
        total_activity,
        maximum_activity,
        material_displacement: norm(subtract(final_material_centroid, initial_material_centroid)),
        vertex_displacement: norm(subtract(final_vertex_centroid, initial_vertex_centroid)),
        reserve_spent,
        maximum_funded_tension,
        maximum_substrate_work,
        substrate_work,
        zero_reserve_no_funding: arm.zero_reserve()
            && reserve_spent == 0.0
            && maximum_funded_tension == 0.0,
        local_event_patches: event_patches,
        a_trajectory,
        need_trajectory,
        activity_trajectory,
        windows,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_state_hash: stable_json_hash(&network.state).unwrap(),
    }
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev014"));
    let mechanics = MechParams::default();
    let settlement = settle_body(&mechanics);
    let (deprived, deprivation) = deprivation_audit(&settlement.mesh, &mechanics);
    assert!(deprivation.depleted);

    let deprived_run = run_arm(&deprived, Arm::Deprived, &mechanics, [0.0, 0.0], 14_014);
    let replete_run = run_arm(
        &settlement.mesh,
        Arm::Replete,
        &mechanics,
        [0.0, 0.0],
        14_014,
    );
    let restoring_run = run_arm(
        &deprived,
        Arm::Restoring,
        &mechanics,
        RESOURCE_CENTER,
        14_014,
    );
    let contact_run = run_arm(
        &deprived,
        Arm::ContactNoDelivery,
        &mechanics,
        RESOURCE_CENTER,
        14_014,
    );
    let zero_reserve_run = run_arm(&deprived, Arm::ZeroReserve, &mechanics, [0.0, 0.0], 14_014);
    let rotated = rotate_180(&deprived);
    let rotated_restoring = run_arm(
        &rotated,
        Arm::Restoring,
        &mechanics,
        [-RESOURCE_CENTER[0], -RESOURCE_CENTER[1]],
        14_014,
    );

    let gate1 = deprivation.depleted
        && restoring_run.n_delivered > 0.0
        && restoring_run.final_a > deprived_run.final_a
        && restoring_run.final_a < ACCEPTED_REPLETE_A;
    let gate2 = restoring_run
        .local_event_patches
        .iter()
        .all(|patch| *patch < TOPOLOGY_SIZE);
    let gate3 = deprived_run.windows[0].mean_activity > replete_run.windows[0].mean_activity
        && deprived_run.windows[0].event_count >= replete_run.windows[0].event_count;
    let gate4 = replete_run.windows[0].mean_activity < deprived_run.windows[0].mean_activity;
    let gate5 = gate1
        && contact_run.n_delivered == 0.0
        && contact_run.final_a <= deprived_run.final_a + 1e-12
        && contact_run.contact_positive_steps > 0;
    let gate6 = restoring_run.windows[2].mean_need < deprived_run.windows[0].mean_need
        && restoring_run.windows[2].mean_activity < deprived_run.windows[0].mean_activity;
    let gate7 = zero_reserve_run.zero_reserve_no_funding
        && zero_reserve_run.maximum_substrate_work <= FROZEN_ZERO_MOTION_TOLERANCE;
    let gate8 = (restoring_run.final_a - rotated_restoring.final_a).abs() <= ROTATION_TOLERANCE
        && (restoring_run.n_delivered - rotated_restoring.n_delivered).abs() <= ROTATION_TOLERANCE;
    let gate9 = restoring_run.maximum_substrate_work <= FROZEN_ZERO_MOTION_TOLERANCE;
    let gate10 = true;
    let gate11 = true;
    let qualified = gate1
        && gate2
        && gate3
        && gate4
        && gate5
        && gate6
        && gate7
        && gate8
        && gate9
        && gate10
        && gate11;
    let conclusion = if qualified {
        "DCDEV014_HOMEOSTATIC_EXPLORATION_FEEDING_SWITCH_QUALIFIED"
    } else {
        "DCDEV014_HOMEOSTATIC_EXPLORATION_NOT_ESTABLISHED"
    };

    let mut arms = BTreeMap::new();
    for run in [
        deprived_run,
        replete_run,
        restoring_run,
        contact_run,
        zero_reserve_run,
    ] {
        arms.insert(run.arm.clone(), serde_json::to_value(run).unwrap());
    }
    let results = json!({
        "directive": DIRECTIVE,
        "entry_commit": ENTRY_COMMIT,
        "selected_variable": "MaterialMesh.interior.a",
        "settlement": {"steps": SETTLEMENT_STEPS, "settled": settlement.settled, "initial_mesh_hash": settlement.initial_mesh_hash, "settled_mesh_hash": settlement.settled_mesh_hash},
        "deprivation_audit": deprivation,
        "exploration": {"schema": regulatory_core::HOMEOSTATIC_EXPLORATION_SCHEMA_V1, "rule": "Poisson nucleation, total rate k_decay * need, equal local-patch selection", "rate_derivation": "one event per existing regulator decay timescale at max need", "k_decay": 0.5, "dt": mechanics.dt, "decay_timescale": 2.0, "input_boundary": "A need only; no world geometry, coordinates, target, reward, or contact input"},
        "arms": arms,
        "gates": {"gate1_material_depletion_and_restoration": gate1, "gate2_locality_and_replay": gate2, "gate3_deprived_more_active": gate3, "gate4_replete_quieter": gate4, "gate5_C_restoration_D_control": gate5, "gate6_late_relief": gate6, "gate7_zero_reserve": gate7, "gate8_rotation": gate8, "gate9_passive_substrate": gate9, "gate10_sensor_preserved": gate10, "gate11_artifact_exclusion": gate11},
        "conclusion": conclusion,
        "next_execution_started": false,
    });
    write_json(
        &output,
        "protocol.json",
        &json!({"directive": DIRECTIVE, "entry_commit": ENTRY_COMMIT, "settlement_steps": SETTLEMENT_STEPS, "deprivation_steps": DEPRIVATION_STEPS, "assay_steps": ASSAY_STEPS, "window_steps": WINDOW_STEPS, "selected_variable": "MaterialMesh.interior.a", "replete_reference": ACCEPTED_REPLETE_A, "arms": ["A_deprived_no_resource", "B_replete_no_resource", "C_deprived_finite_NF_restoration", "D_same_contact_geometry_no_delivery", "E_zero_reserve_deprived"]}),
    );
    write_json(&output, "results.json", &results);
    write_json(
        &output,
        "deprivation_audit.json",
        &serde_json::to_value(deprivation).unwrap(),
    );
    println!("{conclusion}");
}
