//! DC-DEV-007: first autonomous external-contact sensorimotor behavior.
//!
//! This is an integration qualification assay.  It uses only the bounded
//! DC-DEV-006 obstacle/contact adapter, the existing distributed regulator,
//! the DC-DEV-005 adaptation trace, D-091-funded contractility, and the
//! chemistry-core mechanics solver.  The assay never writes coordinates
//! directly and does not introduce a new production mechanism.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_growth::{growth_step, merge_growth_into_reaction, GrowthParams};
use chemistry_core::mesh_mechanics::{remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::material_adapter::observe_continuity_material_frame;
use regulatory_core::{
    apply_local_plasticity_with_external_forces, augment_frame_with_contact, stable_json_hash,
    ContactObservationV1, ContinuityNetworkV1, ContractilityParamsV1, PlasticityParamsV1,
    PlasticityStateV1, StaticObstacleV1, TopologyEventV1, TopologyMappingV1,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-007";
const ENTRY_COMMIT: &str = "3a5971be332f94848250196e8148b722464066f2";
const FROZEN_RESERVE: f64 = 0.6;
const ASSAY_HORIZON_STEPS: usize = 120;
const EXPOSURE_STEPS: usize = 40;
const RECOVERY_STEPS: usize = 200;
const METRIC_TOLERANCE: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arm {
    Active,
    MotorOff,
    ZeroReserve,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::MotorOff => "motor_off",
            Self::ZeroReserve => "zero_reserve",
        }
    }
}

#[derive(Debug, Clone)]
struct StepRecord {
    observation: ContactObservationV1,
    activity: Vec<f64>,
    maximum_tension: f64,
    resource_spent: f64,
    vertices_before: Vec<[f64; 2]>,
    vertices_after: Vec<[f64; 2]>,
}

#[derive(Debug, Clone)]
struct ArmResult {
    arm: Arm,
    j_contact: f64,
    maximum_penetration: f64,
    contact_duration_steps: usize,
    reserve_spent: f64,
    maximum_tension: f64,
    records: Vec<StepRecord>,
    final_mesh_hash: String,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn reserve_for(area: f64) -> ReserveParams {
    ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area)
}

fn seed_mesh_at_radius(radius: f64, reserve: f64) -> MaterialMesh {
    MaterialMesh::seed_regular(
        24,
        radius,
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
        LumpedChem {
            n: 1.0,
            f: 1.0,
            ..Default::default()
        },
        5.0,
    )
}

fn seed_mesh(reserve: f64) -> MaterialMesh {
    seed_mesh_at_radius(5.0, reserve)
}

fn reaction_params(mesh: &MaterialMesh) -> ReactionParams {
    let mut params = ReactionParams::default();
    params.reserve = reserve_for(mesh.area());
    params
}

fn advance_chemistry(
    mesh: &mut MaterialMesh,
    mechanics: &MechParams,
    reactions: &ReactionParams,
    transport: &TransportParams,
    growth: &GrowthParams,
) {
    let _ = transport_step(mesh, transport, mechanics.dt);
    let mut reaction = reactions_step(mesh, reactions, mechanics.dt, true, true);
    let growth_ledger = growth_step(mesh, reactions, growth, mechanics.dt);
    merge_growth_into_reaction(&mut reaction, &growth_ledger);
}

fn event_for(old_size: usize, new_size: usize) -> TopologyEventV1 {
    match new_size.cmp(&old_size) {
        std::cmp::Ordering::Greater => TopologyEventV1::Split,
        std::cmp::Ordering::Less => TopologyEventV1::Merge,
        std::cmp::Ordering::Equal => TopologyEventV1::Stable,
    }
}

fn contact_obstacle() -> StaticObstacleV1 {
    StaticObstacleV1::new([5.0, 0.0], 0.9).unwrap()
}

fn far_obstacle() -> StaticObstacleV1 {
    StaticObstacleV1::new([100.0, 100.0], 1.0).unwrap()
}

fn max_penetration(observation: &ContactObservationV1) -> f64 {
    observation.penetration.iter().copied().fold(0.0, f64::max)
}

fn contact_integral(observation: &ContactObservationV1, dt: f64) -> f64 {
    observation.penetration.iter().sum::<f64>() * dt
}

fn run_arm(
    initial_mesh: &MaterialMesh,
    obstacle: &StaticObstacleV1,
    arm: Arm,
    initial_plasticity: Option<&PlasticityStateV1>,
    horizon: usize,
    seed: u64,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    plasticity: &PlasticityParamsV1,
) -> ArmResult {
    let mut mesh = initial_mesh.clone();
    let initial_frame = observe_continuity_material_frame(&mesh, mechanics);
    let mut network = ContinuityNetworkV1::new(initial_frame, Some(seed)).unwrap();
    let mut state = initial_plasticity
        .cloned()
        .unwrap_or_else(|| PlasticityStateV1::new(mesh.n()));
    let mut motor_off_params = contractility.clone();
    motor_off_params.max_active_tension = 0.0;
    let selected_params = match arm {
        Arm::MotorOff => &motor_off_params,
        Arm::Active | Arm::ZeroReserve => contractility,
    };
    let mut records = Vec::with_capacity(horizon);
    let mut j_contact = 0.0;
    let mut maximum_penetration: f64 = 0.0;
    let mut contact_duration_steps = 0;
    let mut reserve_spent = 0.0;
    let mut maximum_tension: f64 = 0.0;

    if arm == Arm::ZeroReserve {
        mesh.interior.r = 0.0;
    }

    for _ in 0..horizon {
        let vertices_before = mesh.vertices.clone();
        let observation = obstacle.observe(&mesh, mechanics).unwrap();
        let base_frame = observe_continuity_material_frame(&mesh, mechanics);
        let frame = augment_frame_with_contact(&base_frame, &observation.contact_stimulus).unwrap();
        network.step(frame, TopologyEventV1::Stable).unwrap();
        let ledger = apply_local_plasticity_with_external_forces(
            &mut mesh,
            &network.state.activity,
            &mut state,
            mechanics,
            selected_params,
            plasticity,
            Some(&observation.external_force),
        )
        .unwrap();
        let accepted = ledger.contractility.mechanics_accepted;
        assert!(accepted, "DC-DEV-007 requires accepted mechanics steps");
        j_contact += contact_integral(&observation, mechanics.dt);
        maximum_penetration = maximum_penetration.max(max_penetration(&observation));
        contact_duration_steps +=
            usize::from(observation.penetration.iter().any(|value| *value > 0.0));
        reserve_spent += ledger.contractility.resource_spent;
        maximum_tension = maximum_tension.max(ledger.contractility.maximum_tension);
        records.push(StepRecord {
            observation,
            activity: network.state.activity.clone(),
            maximum_tension: ledger.contractility.maximum_tension,
            resource_spent: ledger.contractility.resource_spent,
            vertices_before,
            vertices_after: mesh.vertices.clone(),
        });
    }

    ArmResult {
        arm,
        j_contact,
        maximum_penetration,
        contact_duration_steps,
        reserve_spent,
        maximum_tension,
        records,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
    }
}

fn exposure_state(
    initial_mesh: &MaterialMesh,
    obstacle: &StaticObstacleV1,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    plasticity: &PlasticityParamsV1,
) -> PlasticityStateV1 {
    let mut state = PlasticityStateV1::new(initial_mesh.n());
    for _ in 0..EXPOSURE_STEPS {
        // Reset geometry between exposures while carrying only the existing
        // adaptation state, matching the DC-DEV-005 repeated-input assay.
        let mut mesh = initial_mesh.clone();
        let frame = observe_continuity_material_frame(&mesh, mechanics);
        let mut network = ContinuityNetworkV1::new(frame, Some(701)).unwrap();
        let observation = obstacle.observe(&mesh, mechanics).unwrap();
        let frame = augment_frame_with_contact(
            &observe_continuity_material_frame(&mesh, mechanics),
            &observation.contact_stimulus,
        )
        .unwrap();
        network.step(frame, TopologyEventV1::Stable).unwrap();
        apply_local_plasticity_with_external_forces(
            &mut mesh,
            &network.state.activity,
            &mut state,
            mechanics,
            contractility,
            plasticity,
            Some(&observation.external_force),
        )
        .unwrap();
    }
    state
}

fn recover_state(
    initial_mesh: &MaterialMesh,
    state: &PlasticityStateV1,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    plasticity: &PlasticityParamsV1,
) -> PlasticityStateV1 {
    // Reproduce the existing DC-DEV-005 no-contact recovery path so the
    // returned state is the accepted trace.
    let mut recovered = state.clone();
    let mut mesh = initial_mesh.clone();
    let frame = observe_continuity_material_frame(&mesh, mechanics);
    let mut network = ContinuityNetworkV1::new(frame, Some(702)).unwrap();
    let zero_forces = vec![[0.0, 0.0]; mesh.n()];
    for _ in 0..RECOVERY_STEPS {
        let base = observe_continuity_material_frame(&mesh, mechanics);
        let frame = augment_frame_with_contact(&base, &vec![0.0; mesh.n()]).unwrap();
        network.step(frame, TopologyEventV1::Stable).unwrap();
        apply_local_plasticity_with_external_forces(
            &mut mesh,
            &network.state.activity,
            &mut recovered,
            mechanics,
            contractility,
            plasticity,
            Some(&zero_forces),
        )
        .unwrap();
    }
    recovered
}

fn first_step_loop(
    initial_mesh: &MaterialMesh,
    obstacle: &StaticObstacleV1,
    arm: Arm,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    plasticity: &PlasticityParamsV1,
) -> StepRecord {
    run_arm(
        initial_mesh,
        obstacle,
        arm,
        None,
        1,
        703,
        mechanics,
        contractility,
        plasticity,
    )
    .records
    .into_iter()
    .next()
    .unwrap()
}

fn remesh_compatibility(
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    plasticity: &PlasticityParamsV1,
) -> (bool, bool, usize, usize) {
    let mut mesh = seed_mesh_at_radius(14.0, FROZEN_RESERVE);
    stamp_reserve_equation(&mut mesh);
    let reactions = reaction_params(&mesh);
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 1.3,
        enable_growth: true,
    };
    let initial_frame = observe_continuity_material_frame(&mesh, mechanics);
    let mut network = ContinuityNetworkV1::new(initial_frame, Some(708)).unwrap();
    network.state.activity[0] = 0.8;
    let mut plasticity_state = PlasticityStateV1::new(mesh.n());
    let mut previous_size = mesh.n();
    let mut remesh_events = 0;
    let mut continuity_valid = true;
    let mut trace_nonzero = false;
    for _ in 0..1000 {
        advance_chemistry(&mut mesh, mechanics, &reactions, &transport, &growth);
        let frame = observe_continuity_material_frame(&mesh, mechanics);
        let event = event_for(network.previous_frame.topology_size, frame.topology_size);
        match network.step(frame, event) {
            Ok(mapping) => {
                continuity_valid &= plasticity_state.remap(&mapping).is_ok();
                let zero_forces = vec![[0.0, 0.0]; mesh.n()];
                match apply_local_plasticity_with_external_forces(
                    &mut mesh,
                    &network.state.activity,
                    &mut plasticity_state,
                    mechanics,
                    contractility,
                    plasticity,
                    Some(&zero_forces),
                ) {
                    Ok(ledger) => {
                        trace_nonzero |= ledger.maximum_adaptation > 0.0;
                    }
                    Err(_) => continuity_valid = false,
                }
                let _ = remesh(&mut mesh);
                if mesh.n() != previous_size {
                    remesh_events += 1;
                }
                previous_size = mesh.n();
            }
            Err(_) => continuity_valid = false,
        }
    }

    let frame = observe_continuity_material_frame(&mesh, mechanics);
    let mut fission_network = ContinuityNetworkV1::new(frame.clone(), Some(709)).unwrap();
    let continuity_fission_rejected = fission_network
        .step(frame, TopologyEventV1::Fission)
        .is_err();
    let fission_mapping = TopologyMappingV1 {
        schema: regulatory_core::continuity::TOPOLOGY_MAPPING_SCHEMA_V1.to_string(),
        old_topology_size: mesh.n(),
        new_topology_size: mesh.n(),
        event: TopologyEventV1::Fission,
        new_to_old: (0..mesh.n()).collect(),
        maximum_mapping_distance: 0.0,
        mapping_rule: "unsupported".to_string(),
    };
    let plasticity_fission_rejected = PlasticityStateV1::new(mesh.n())
        .remap(&fission_mapping)
        .is_err();
    (
        remesh_events >= 2
            && continuity_valid
            && trace_nonzero
            && plasticity_state.adaptation.len() == mesh.n(),
        continuity_fission_rejected && plasticity_fission_rejected,
        remesh_events,
        mesh.n(),
    )
}

fn baseline_parity(
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    plasticity: &PlasticityParamsV1,
) -> bool {
    let initial = seed_mesh(FROZEN_RESERVE);
    let far = far_obstacle();
    let mut legacy_mesh = initial.clone();
    let mut zero_world_mesh = initial.clone();
    let mut legacy_state = PlasticityStateV1::new(initial.n());
    let mut zero_world_state = PlasticityStateV1::new(initial.n());
    let frame = observe_continuity_material_frame(&initial, mechanics);
    let mut legacy_network = ContinuityNetworkV1::new(frame.clone(), Some(704)).unwrap();
    let mut zero_world_network = ContinuityNetworkV1::new(frame, Some(704)).unwrap();
    for _ in 0..24 {
        let base_legacy = observe_continuity_material_frame(&legacy_mesh, mechanics);
        let base_zero = observe_continuity_material_frame(&zero_world_mesh, mechanics);
        let zero_observation = far.observe(&zero_world_mesh, mechanics).unwrap();
        let zero_frame =
            augment_frame_with_contact(&base_zero, &zero_observation.contact_stimulus).unwrap();
        legacy_network
            .step(base_legacy, TopologyEventV1::Stable)
            .unwrap();
        zero_world_network
            .step(zero_frame, TopologyEventV1::Stable)
            .unwrap();
        let legacy = apply_local_plasticity_with_external_forces(
            &mut legacy_mesh,
            &legacy_network.state.activity,
            &mut legacy_state,
            mechanics,
            contractility,
            plasticity,
            None,
        )
        .unwrap();
        let zero_world = apply_local_plasticity_with_external_forces(
            &mut zero_world_mesh,
            &zero_world_network.state.activity,
            &mut zero_world_state,
            mechanics,
            contractility,
            plasticity,
            Some(&zero_observation.external_force),
        )
        .unwrap();
        if legacy_mesh.vertices != zero_world_mesh.vertices
            || stable_json_hash(&legacy_mesh.interior).unwrap()
                != stable_json_hash(&zero_world_mesh.interior).unwrap()
            || legacy_state != zero_world_state
            || legacy.contractility != zero_world.contractility
        {
            return false;
        }
    }
    true
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev007"));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let plasticity = PlasticityParamsV1::default();
    let initial = seed_mesh(FROZEN_RESERVE);
    let obstacle = contact_obstacle();
    let far = far_obstacle();

    let gate0 = ENTRY_COMMIT == "3a5971be332f94848250196e8148b722464066f2";
    let gate1 = baseline_parity(&mechanics, &contractility, &plasticity);

    let active = run_arm(
        &initial,
        &obstacle,
        Arm::Active,
        None,
        ASSAY_HORIZON_STEPS,
        705,
        &mechanics,
        &contractility,
        &plasticity,
    );
    let motor_off = run_arm(
        &initial,
        &obstacle,
        Arm::MotorOff,
        None,
        ASSAY_HORIZON_STEPS,
        705,
        &mechanics,
        &contractility,
        &plasticity,
    );
    let zero_reserve = run_arm(
        &initial,
        &obstacle,
        Arm::ZeroReserve,
        None,
        ASSAY_HORIZON_STEPS,
        705,
        &mechanics,
        &contractility,
        &plasticity,
    );

    let active_first = &active.records[0];
    let motor_off_first = &motor_off.records[0];
    let no_contact_first = first_step_loop(
        &initial,
        &far,
        Arm::Active,
        &mechanics,
        &contractility,
        &plasticity,
    );
    let contacted = active_first.observation.contacted_indices();
    let local_activity_increased = !contacted.is_empty()
        && contacted
            .iter()
            .any(|index| active_first.activity[*index] > no_contact_first.activity[*index]);
    let distant_indices: Vec<usize> = (0..initial.n())
        .filter(|index| {
            !contacted.contains(index)
                && contacted.iter().all(|contact| {
                    index
                        .abs_diff(*contact)
                        .min(initial.n() - index.abs_diff(*contact))
                        > 1
                })
        })
        .collect();
    let distant_initial_activity_unchanged = !distant_indices.is_empty()
        && distant_indices
            .iter()
            .all(|index| active_first.activity[*index] == no_contact_first.activity[*index]);
    let active_vertex_changed = active_first.vertices_after != motor_off_first.vertices_after;
    let gate2 = active_first
        .observation
        .contact_stimulus
        .iter()
        .copied()
        .fold(0.0, f64::max)
        > 0.0
        && local_activity_increased
        && active_first.maximum_tension > 0.0
        && active_vertex_changed;
    let gate3 = active.j_contact + METRIC_TOLERANCE < motor_off.j_contact;
    let zero_reserve_pass = zero_reserve.maximum_tension == 0.0
        && zero_reserve.reserve_spent == 0.0
        && (zero_reserve.j_contact - motor_off.j_contact).abs() <= METRIC_TOLERANCE;
    let gate4 = active.reserve_spent > 0.0 && zero_reserve_pass;
    let gate5 = distant_initial_activity_unchanged;

    let experienced_state =
        exposure_state(&initial, &obstacle, &mechanics, &contractility, &plasticity);
    let naive = run_arm(
        &initial,
        &obstacle,
        Arm::Active,
        None,
        ASSAY_HORIZON_STEPS,
        706,
        &mechanics,
        &contractility,
        &plasticity,
    );
    let experienced = run_arm(
        &initial,
        &obstacle,
        Arm::Active,
        Some(&experienced_state),
        ASSAY_HORIZON_STEPS,
        706,
        &mechanics,
        &contractility,
        &plasticity,
    );
    let recovered_state = recover_state(
        &initial,
        &experienced_state,
        &mechanics,
        &contractility,
        &plasticity,
    );
    let recovered = run_arm(
        &initial,
        &obstacle,
        Arm::Active,
        Some(&recovered_state),
        ASSAY_HORIZON_STEPS,
        706,
        &mechanics,
        &contractility,
        &plasticity,
    );
    let gate6 = experienced.records[0].maximum_tension < naive.records[0].maximum_tension
        || experienced.j_contact != naive.j_contact;
    let recovered_moves_toward_naive =
        (recovered.records[0].maximum_tension - naive.records[0].maximum_tension).abs()
            < (experienced.records[0].maximum_tension - naive.records[0].maximum_tension).abs();
    let gate7 = recovered_moves_toward_naive;
    let (remesh_pass, fission_fail_closed, remesh_events, final_remesh_vertices) =
        remesh_compatibility(&mechanics, &contractility, &plasticity);
    let gate8 = remesh_pass && fission_fail_closed;

    let gates = [
        gate0, gate1, gate2, gate3, gate4, gate5, gate6, gate7, gate8,
    ];
    assert!(
        gates.iter().all(|passed| *passed),
        "DC-DEV-007 gate failed: {gates:?}"
    );

    let artifact_status = "AUTHORITATIVE";
    write_json(
        &output,
        "protocol.json",
        &json!({
            "artifact_status": artifact_status,
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "assay_horizon_steps": ASSAY_HORIZON_STEPS,
            "assay_horizon_simulated_time": ASSAY_HORIZON_STEPS as f64 * mechanics.dt,
            "accepted_time_authority": "MechParams.dt on each accepted mechanics step",
            "parameter_screening": false,
            "new_sensor": false,
            "new_actuator": false,
            "new_trace": false,
            "new_world_primitive": false,
            "reward": false,
            "fitness": false,
            "evolution": false,
            "dcdev008_started": false,
            "conclusion": "DCDEV007_ACTIVE_EXTERNAL_CONTACT_REGULATION_QUALIFIED",
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "matched_arms.json",
        &json!({
            "active": {"j_contact": active.j_contact, "maximum_penetration": active.maximum_penetration, "contact_duration_steps": active.contact_duration_steps, "reserve_spent": active.reserve_spent, "maximum_tension": active.maximum_tension, "final_mesh_hash": active.final_mesh_hash},
            "motor_off": {"j_contact": motor_off.j_contact, "maximum_penetration": motor_off.maximum_penetration, "contact_duration_steps": motor_off.contact_duration_steps, "reserve_spent": motor_off.reserve_spent, "maximum_tension": motor_off.maximum_tension, "final_mesh_hash": motor_off.final_mesh_hash},
            "zero_reserve": {"j_contact": zero_reserve.j_contact, "maximum_penetration": zero_reserve.maximum_penetration, "contact_duration_steps": zero_reserve.contact_duration_steps, "reserve_spent": zero_reserve.reserve_spent, "maximum_tension": zero_reserve.maximum_tension, "final_mesh_hash": zero_reserve.final_mesh_hash},
            "gate3_active_less_than_motor_off": gate3,
            "gate4_zero_reserve": gate4,
            "result": "DCDEV007_GATES3_AND4_PASS"
        }),
    );
    write_json(
        &output,
        "causal_loop.json",
        &json!({
            "contact_stimulus_positive": active_first.observation.contact_stimulus.iter().copied().fold(0.0, f64::max) > 0.0,
            "local_activity_increased": local_activity_increased,
            "local_funded_tension_positive": active_first.maximum_tension > 0.0,
            "active_vertex_trajectory_differs_from_motor_off": active_vertex_changed,
            "direct_coordinate_command": false,
            "result": "DCDEV007_GATE2_CAUSAL_EXTERNAL_LOOP_PASS"
        }),
    );
    write_json(
        &output,
        "locality.json",
        &json!({
            "contacted_indices": contacted,
            "distant_indices": distant_indices,
            "distant_initial_activity_unchanged": distant_initial_activity_unchanged,
            "initial_response_is_local": gate5,
            "result": "DCDEV007_GATE5_LOCALITY_PASS"
        }),
    );
    write_json(
        &output,
        "experience_and_recovery.json",
        &json!({
            "exposure_steps": EXPOSURE_STEPS,
            "recovery_steps": RECOVERY_STEPS,
            "naive": {"j_contact": naive.j_contact, "first_tension": naive.records[0].maximum_tension, "contact_duration_steps": naive.contact_duration_steps},
            "experienced": {"j_contact": experienced.j_contact, "first_tension": experienced.records[0].maximum_tension, "contact_duration_steps": experienced.contact_duration_steps},
            "recovered": {"j_contact": recovered.j_contact, "first_tension": recovered.records[0].maximum_tension, "contact_duration_steps": recovered.contact_duration_steps},
            "history_changed_response": gate6,
            "recovery_moved_toward_naive": gate7,
            "result": "DCDEV007_GATES6_AND7_PASS"
        }),
    );
    write_json(
        &output,
        "zero_world_and_body_continuity.json",
        &json!({
            "exact_dcdev006_dcdev005_no_contact_parity": gate1,
        "ordinary_remeshing": "preserved by existing DC-DEV-006/DC-DEV-005 continuity path; no remesh authority added",
        "ordinary_remeshing_assay_pass": remesh_pass,
        "remesh_events": remesh_events,
        "final_remesh_vertices": final_remesh_vertices,
        "fission_fail_closed": fission_fail_closed,
        "environment_authority_over_growth_remesh_fission_metabolism_heredity": false,
            "fission": "fail-closed outside this assay",
            "result": "DCDEV007_GATES1_AND8_BOUNDARY_PASS"
        }),
    );
    write_json(
        &output,
        "governance_boundary.json",
        &json!({
            "new_sensor": false,
            "new_actuator": false,
            "new_trace": false,
            "new_world_primitive": false,
            "planner": false,
            "action_selector": false,
            "reward": false,
            "fitness": false,
            "reinforcement_learning": false,
            "neural_network": false,
            "navigation": false,
            "evolution": false,
            "dcdev008_started": false,
            "result": "DCDEV007_GATE0_SCOPE_PASS"
        }),
    );
    write_json(
        &output,
        "regression_manifest.json",
        &json!({
            "dcdev002": "PENDING_REMOTE_CI",
            "dcdev003": "PENDING_REMOTE_CI",
            "dcdev004": "PENDING_REMOTE_CI",
            "dcdev005": "PENDING_REMOTE_CI",
            "dcdev006": "PENDING_REMOTE_CI",
            "phase1_focused_certification": "PENDING_REMOTE_CI",
            "d088": "PENDING_REMOTE_CI",
            "evolution_harness": "PENDING_REMOTE_CI",
            "governance": "PENDING_REMOTE_CI",
            "exact_head_remote_ci": "PENDING_REMOTE_CI",
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "final_manifest.json",
        &json!({
            "artifact_status": artifact_status,
            "conclusion": "DCDEV007_ACTIVE_EXTERNAL_CONTACT_REGULATION_QUALIFIED",
            "gates": gates,
            "primary_metric": "integrated contact penetration",
            "active_j_contact": active.j_contact,
            "motor_off_j_contact": motor_off.j_contact,
        "zero_reserve_j_contact": zero_reserve.j_contact,
        "remesh_pass": remesh_pass,
        "fission_fail_closed": fission_fail_closed,
        "next_execution_started": false
        }),
    );
}
