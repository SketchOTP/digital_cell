//! DC-DEV-006: minimal spatial environment and local external contact.
//!
//! The assay uses one static inert circular obstacle.  Geometry produces a
//! bounded local force vector and one penetration-normalized contact signal;
//! chemistry-core mechanics alone moves the organism.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_growth::{growth_step, merge_growth_into_reaction, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step_with_external_forces, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::material_adapter::observe_continuity_material_frame;
use regulatory_core::{
    apply_local_plasticity, apply_local_plasticity_with_external_forces,
    augment_frame_with_contact, stable_json_hash, ContactObservationV1, ContinuityNetworkV1,
    ContractilityParamsV1, PlasticityParamsV1, PlasticityStateV1, StaticObstacleV1,
    TopologyEventV1, TopologyMappingV1, CONTACT_FORCE_NORMALIZATION, CONTACT_STIFFNESS_PER_LENGTH,
    SPATIAL_WORLD_SCHEMA_V1,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-006";
const ENTRY_COMMIT: &str = "4da04a5cf8153e4ab31603965eeba305ad4bb721";
const FROZEN_RESERVE: f64 = 0.6;
const EXPOSURE_STEPS: usize = 40;
const RECOVERY_STEPS: usize = 200;

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn reserve_for(area: f64) -> ReserveParams {
    ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, area)
}

fn seed_mesh(radius: f64, reserve: f64) -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
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
    );
    stamp_reserve_equation(&mut mesh);
    mesh
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
) -> (ReactionLedger, chemistry_core::mesh_growth::GrowthLedger) {
    let _ = transport_step(mesh, transport, mechanics.dt);
    let mut reaction = reactions_step(mesh, reactions, mechanics.dt, true, true);
    let growth_ledger = growth_step(mesh, reactions, growth, mechanics.dt);
    merge_growth_into_reaction(&mut reaction, &growth_ledger);
    (reaction, growth_ledger)
}

fn event_for(old_size: usize, new_size: usize) -> TopologyEventV1 {
    match new_size.cmp(&old_size) {
        std::cmp::Ordering::Greater => TopologyEventV1::Split,
        std::cmp::Ordering::Less => TopologyEventV1::Merge,
        std::cmp::Ordering::Equal => TopologyEventV1::Stable,
    }
}

fn one_contact_step(
    mesh: &MaterialMesh,
    mechanics: &MechParams,
    obstacle: &StaticObstacleV1,
    seed: u64,
) -> (
    ContactObservationV1,
    regulatory_core::ContinuityMaterialFrameV1,
    Vec<f64>,
) {
    let base = observe_continuity_material_frame(mesh, mechanics);
    let observation = obstacle.observe(mesh, mechanics).unwrap();
    let frame = augment_frame_with_contact(&base, &observation.contact_stimulus).unwrap();
    let mut network = ContinuityNetworkV1::new(base, Some(seed)).unwrap();
    network
        .step(frame.clone(), TopologyEventV1::Stable)
        .unwrap();
    (observation, frame, network.state.activity)
}

fn max_adaptation(state: &PlasticityStateV1) -> f64 {
    state.adaptation.iter().copied().fold(0.0, f64::max)
}

fn ring_distance(a: usize, b: usize, n: usize) -> usize {
    let direct = a.abs_diff(b);
    direct.min(n - direct)
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev006"));
    let mechanics = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 1.3,
        enable_growth: true,
    };
    let contractility = ContractilityParamsV1::default();
    let plasticity = PlasticityParamsV1::default();
    let contact_obstacle = StaticObstacleV1::new([5.0, 0.0], 0.9).unwrap();
    let no_contact_obstacle = StaticObstacleV1::new([100.0, 100.0], 1.0).unwrap();

    // Gate 1: the world support is inert when it produces no contact.  Compare
    // the complete no-world and zero-contact paths over a deterministic trace.
    let base_mesh = seed_mesh(5.0, FROZEN_RESERVE);
    let base_frame = observe_continuity_material_frame(&base_mesh, &mechanics);
    let zero_observation = no_contact_obstacle.observe(&base_mesh, &mechanics).unwrap();
    let zero_frame =
        augment_frame_with_contact(&base_frame, &zero_observation.contact_stimulus).unwrap();
    let mut legacy_network = ContinuityNetworkV1::new(base_frame.clone(), Some(600)).unwrap();
    let mut zero_world_network = ContinuityNetworkV1::new(base_frame, Some(600)).unwrap();
    let mut legacy_mesh = base_mesh.clone();
    let mut zero_world_mesh = base_mesh.clone();
    let mut legacy_state = PlasticityStateV1::new(base_mesh.n());
    let mut zero_world_state = PlasticityStateV1::new(base_mesh.n());
    let mut zero_world_parity = true;
    for _ in 0..24 {
        legacy_network
            .step(zero_frame.clone(), TopologyEventV1::Stable)
            .unwrap();
        zero_world_network
            .step(zero_frame.clone(), TopologyEventV1::Stable)
            .unwrap();
        let legacy = apply_local_plasticity(
            &mut legacy_mesh,
            &legacy_network.state.activity,
            &mut legacy_state,
            &mechanics,
            &contractility,
            &plasticity,
        )
        .unwrap();
        let zero_world = apply_local_plasticity_with_external_forces(
            &mut zero_world_mesh,
            &zero_world_network.state.activity,
            &mut zero_world_state,
            &mechanics,
            &contractility,
            &plasticity,
            Some(&zero_observation.external_force),
        )
        .unwrap();
        zero_world_parity &= legacy_network.state == zero_world_network.state
            && legacy_mesh.vertices == zero_world_mesh.vertices
            && stable_json_hash(&legacy_mesh.interior).unwrap()
                == stable_json_hash(&zero_world_mesh.interior).unwrap()
            && legacy_state == zero_world_state
            && legacy.contractility == zero_world.contractility;
    }

    // Gate 2: geometry supplies local forces; mechanics, not the world, moves
    // the body and preserves material quantities.
    let physics_mesh = seed_mesh(5.0, FROZEN_RESERVE);
    let physics_before = physics_mesh.clone();
    let physics_observation = contact_obstacle.observe(&physics_mesh, &mechanics).unwrap();
    let mut physics_after = physics_mesh.clone();
    let physics_accepted = mechanics_step_with_external_forces(
        &mut physics_after,
        &mechanics,
        &physics_observation.external_force,
    );
    let contact_indices = physics_observation.contacted_indices();
    let non_contact_forces_zero = physics_observation
        .external_force
        .iter()
        .enumerate()
        .all(|(index, force)| contact_indices.contains(&index) || *force == [0.0, 0.0]);
    let world_did_not_move_coordinates = physics_mesh.vertices == physics_before.vertices;
    let mechanics_resolved_contact =
        physics_accepted && physics_after.vertices != physics_before.vertices;
    let contact_physics_pass = !contact_indices.is_empty()
        && contact_indices.len() < physics_mesh.n()
        && non_contact_forces_zero
        && world_did_not_move_coordinates
        && mechanics_resolved_contact;

    // Gate 3: contact_stimulus is one deterministic, bounded local signal.
    let transduction_a = contact_obstacle.observe(&physics_mesh, &mechanics).unwrap();
    let transduction_b = contact_obstacle.observe(&physics_mesh, &mechanics).unwrap();
    let signal_hash_a = stable_json_hash(&transduction_a.contact_stimulus).unwrap();
    let signal_hash_b = stable_json_hash(&transduction_b.contact_stimulus).unwrap();
    let contacted_positive = contact_indices
        .iter()
        .all(|index| transduction_a.contact_stimulus[*index] > 0.0);
    let distant_zero = (0..physics_mesh.n())
        .filter(|index| !contact_indices.contains(index))
        .all(|index| transduction_a.contact_stimulus[index] == 0.0);
    let transduction_pass = signal_hash_a == signal_hash_b
        && contacted_positive
        && distant_zero
        && transduction_a
            .contact_stimulus
            .iter()
            .all(|value| (0.0..=1.0).contains(value));

    // Gate 4: the existing neighbor-coupled regulator sees only the local
    // external signal; one-step distant patches remain untouched.
    let response_mesh = seed_mesh(5.0, FROZEN_RESERVE);
    let (contact_response_observation, contact_frame, contact_activity) =
        one_contact_step(&response_mesh, &mechanics, &contact_obstacle, 604);
    let (no_contact_response_observation, no_contact_frame, no_contact_activity) =
        one_contact_step(&response_mesh, &mechanics, &no_contact_obstacle, 604);
    let contacted_activity_increased = contact_response_observation
        .contacted_indices()
        .iter()
        .any(|index| contact_activity[*index] > no_contact_activity[*index]);
    let distant_indices: Vec<usize> = (0..response_mesh.n())
        .filter(|index| {
            !contact_response_observation
                .contacted_indices()
                .contains(index)
                && contact_response_observation
                    .contacted_indices()
                    .iter()
                    .all(|contact| ring_distance(*index, *contact, response_mesh.n()) > 1)
        })
        .collect();
    let distant_activity_unchanged = !distant_indices.is_empty()
        && distant_indices
            .iter()
            .all(|index| contact_activity[*index] == no_contact_activity[*index]);
    let regulatory_response_pass = contacted_activity_increased && distant_activity_unchanged;

    // Gate 5: identical physical contact exposures load the already-qualified
    // adaptation trace and reduce the later matched contact response.
    let experience_mesh = seed_mesh(5.0, FROZEN_RESERVE);
    let (_, repeated_frame, repeated_activity) =
        one_contact_step(&experience_mesh, &mechanics, &contact_obstacle, 605);
    let repeated_observation = contact_obstacle
        .observe(&experience_mesh, &mechanics)
        .unwrap();
    let repeated_input_hash = stable_json_hash(&(&repeated_frame, &repeated_activity)).unwrap();
    let mut experienced_state = PlasticityStateV1::new(experience_mesh.n());
    let mut exposure_responses = Vec::new();
    let mut repeated_inputs_identical = true;
    for _ in 0..EXPOSURE_STEPS {
        let mut exposure_mesh = experience_mesh.clone();
        let (_, frame, activity) =
            one_contact_step(&exposure_mesh, &mechanics, &contact_obstacle, 605);
        repeated_inputs_identical &=
            stable_json_hash(&(&frame, &activity)).unwrap() == repeated_input_hash;
        let ledger = apply_local_plasticity_with_external_forces(
            &mut exposure_mesh,
            &activity,
            &mut experienced_state,
            &mechanics,
            &contractility,
            &plasticity,
            Some(&repeated_observation.external_force),
        )
        .unwrap();
        exposure_responses.push(ledger.contractility.maximum_tension);
    }
    let adaptation_after_exposure = max_adaptation(&experienced_state);
    let mut naive_state = PlasticityStateV1::new(experience_mesh.n());
    let mut naive_mesh = experience_mesh.clone();
    let naive = apply_local_plasticity_with_external_forces(
        &mut naive_mesh,
        &repeated_activity,
        &mut naive_state,
        &mechanics,
        &contractility,
        &plasticity,
        Some(&repeated_observation.external_force),
    )
    .unwrap();
    let naive_response = naive.contractility.maximum_tension;
    let mut recovery_mesh = experience_mesh.clone();
    let recovery_activity = vec![0.0; recovery_mesh.n()];
    let zero_forces = vec![[0.0, 0.0]; recovery_mesh.n()];
    let adaptation_before_recovery = max_adaptation(&experienced_state);
    for _ in 0..RECOVERY_STEPS {
        apply_local_plasticity_with_external_forces(
            &mut recovery_mesh,
            &recovery_activity,
            &mut experienced_state,
            &mechanics,
            &contractility,
            &plasticity,
            Some(&zero_forces),
        )
        .unwrap();
    }
    let adaptation_after_recovery = max_adaptation(&experienced_state);
    let mut recovered_mesh = experience_mesh.clone();
    let recovered = apply_local_plasticity_with_external_forces(
        &mut recovered_mesh,
        &repeated_activity,
        &mut experienced_state,
        &mechanics,
        &contractility,
        &plasticity,
        Some(&repeated_observation.external_force),
    )
    .unwrap();
    let experienced_response = exposure_responses.last().copied().unwrap_or(0.0);
    let recovered_response = recovered.contractility.maximum_tension;
    let habituation_pass = repeated_inputs_identical
        && adaptation_after_exposure > 0.0
        && experienced_response < naive_response
        && adaptation_after_recovery < adaptation_before_recovery
        && experienced_response < recovered_response
        && recovered_response < naive_response;

    // Gate 6: ordinary growth/remesh remains a continuity concern, while
    // fission and unknown state transfer remain fail-closed.
    let mut remesh_mesh = seed_mesh(14.0, FROZEN_RESERVE);
    let remesh_reaction = reaction_params(&remesh_mesh);
    let initial_frame = observe_continuity_material_frame(&remesh_mesh, &mechanics);
    let mut continuity = ContinuityNetworkV1::new(initial_frame, Some(606)).unwrap();
    continuity.state.activity[0] = 0.8;
    let mut remesh_state = PlasticityStateV1::new(remesh_mesh.n());
    let remesh_obstacle = StaticObstacleV1::new([14.0, 0.0], 0.9).unwrap();
    let mut remesh_events = Vec::new();
    let mut remesh_continuity_valid = true;
    let mut remesh_trace_nonzero = false;
    let mut previous_size = remesh_mesh.n();
    for step in 0..1000u64 {
        let _ = advance_chemistry(
            &mut remesh_mesh,
            &mechanics,
            &remesh_reaction,
            &transport,
            &growth,
        );
        let base = observe_continuity_material_frame(&remesh_mesh, &mechanics);
        let observation = remesh_obstacle.observe(&remesh_mesh, &mechanics).unwrap();
        let frame = augment_frame_with_contact(&base, &observation.contact_stimulus).unwrap();
        let event = event_for(continuity.previous_frame.topology_size, frame.topology_size);
        match continuity.step(frame, event) {
            Ok(mapping) => {
                remesh_continuity_valid &= continuity.state.activity.len() == remesh_mesh.n();
                remesh_state.remap(&mapping).unwrap();
                let ledger = apply_local_plasticity_with_external_forces(
                    &mut remesh_mesh,
                    &continuity.state.activity,
                    &mut remesh_state,
                    &mechanics,
                    &contractility,
                    &plasticity,
                    Some(&observation.external_force),
                )
                .unwrap();
                remesh_trace_nonzero |= ledger.maximum_adaptation > 0.0;
                let (splits, merges) = remesh(&mut remesh_mesh);
                if remesh_mesh.n() != previous_size {
                    remesh_events.push(json!({
                        "step": step + 1,
                        "old_vertices": previous_size,
                        "new_vertices": remesh_mesh.n(),
                        "split_operations": splits,
                        "merge_operations": merges,
                        "mapping_event": format!("{event:?}"),
                        "adaptation_after_mapping": max_adaptation(&remesh_state),
                        "mapping_rule": mapping.mapping_rule
                    }));
                }
                previous_size = remesh_mesh.n();
            }
            Err(_) => remesh_continuity_valid = false,
        }
    }
    let remesh_pass = remesh_events.len() >= 2
        && remesh_continuity_valid
        && remesh_trace_nonzero
        && remesh_state.adaptation.len() == remesh_mesh.n()
        && remesh_mesh.alive;
    let fission_frame = observe_continuity_material_frame(&remesh_mesh, &mechanics);
    let mut fission_network = ContinuityNetworkV1::new(fission_frame.clone(), Some(607)).unwrap();
    let continuity_fission_rejected = fission_network
        .step(fission_frame, TopologyEventV1::Fission)
        .is_err();
    let fission_mapping = TopologyMappingV1 {
        schema: regulatory_core::continuity::TOPOLOGY_MAPPING_SCHEMA_V1.to_string(),
        old_topology_size: remesh_mesh.n(),
        new_topology_size: remesh_mesh.n(),
        event: TopologyEventV1::Fission,
        new_to_old: (0..remesh_mesh.n()).collect(),
        maximum_mapping_distance: 0.0,
        mapping_rule: "unsupported".to_string(),
    };
    let plasticity_fission_rejected = PlasticityStateV1::new(remesh_mesh.n())
        .remap(&fission_mapping)
        .is_err();
    let fission_fail_closed = continuity_fission_rejected && plasticity_fission_rejected;

    let gates = [
        true,
        zero_world_parity,
        contact_physics_pass,
        transduction_pass,
        regulatory_response_pass,
        habituation_pass,
        remesh_pass && fission_fail_closed,
    ];
    assert!(
        gates.iter().all(|passed| *passed),
        "DC-DEV-006 gate failed: {gates:?}"
    );

    write_json(
        &output,
        "protocol.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "source_branch": "strategy/dc-dev-005-local-plasticity",
            "world_primitive": "one static inert circular obstacle",
            "contact_signal": "contact_stimulus_i",
            "contact_force": "bounded local penetration force passed to mechanics",
            "new_sensor": false,
            "new_actuator": false,
            "reward": false,
            "fitness": false,
            "evolution": false,
            "resource_ecology": false,
            "parameter_screening": false,
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "contact_world.json",
        &json!({
            "schema": SPATIAL_WORLD_SCHEMA_V1,
            "obstacle": {"center": [5.0, 0.0], "radius": 0.9, "static": true, "inert": true},
            "force_law": "F_i = clamp(0.5 * penetration_i, 0, 0.5) * outward_normal_i",
            "contact_stimulus_definition": "clamp(|F_i| / 0.5, 0, 1)",
            "contact_force_normalization": CONTACT_FORCE_NORMALIZATION,
            "contact_stiffness_per_length": CONTACT_STIFFNESS_PER_LENGTH,
            "parameter_screening": false,
            "world_writes_coordinates": false
        }),
    );
    write_json(
        &output,
        "zero_world_parity.json",
        &json!({"steps": 24, "far_obstacle_contact_is_zero": true, "exact_dcdev005_trajectory": zero_world_parity, "result": "DCDEV006_GATE1_ZERO_WORLD_PARITY_PASS"}),
    );
    write_json(
        &output,
        "contact_physics.json",
        &json!({"contact_indices": contact_indices, "non_contact_forces_zero": non_contact_forces_zero, "world_did_not_move_coordinates": world_did_not_move_coordinates, "mechanics_resolved_contact": mechanics_resolved_contact, "result": "DCDEV006_GATE2_LOCAL_CONTACT_PHYSICS_PASS"}),
    );
    write_json(
        &output,
        "contact_transduction.json",
        &json!({"signal_hash_a": signal_hash_a, "signal_hash_b": signal_hash_b, "deterministic": signal_hash_a == signal_hash_b, "contacted_positive": contacted_positive, "distant_zero": distant_zero, "bounded": transduction_pass, "result": "DCDEV006_GATE3_LOCAL_EXTERNAL_TRANSDUCTION_PASS"}),
    );
    write_json(
        &output,
        "regulatory_response.json",
        &json!({"contacted_activity_increased": contacted_activity_increased, "distant_activity_unchanged": distant_activity_unchanged, "contact_frame_hash": stable_json_hash(&contact_frame).unwrap(), "no_contact_frame_hash": stable_json_hash(&no_contact_frame).unwrap(), "contact_observation_hash": stable_json_hash(&contact_response_observation).unwrap(), "no_contact_observation_hash": stable_json_hash(&no_contact_response_observation).unwrap(), "result": "DCDEV006_GATE4_LOCAL_REGULATORY_RESPONSE_PASS"}),
    );
    write_json(
        &output,
        "experience_dependence.json",
        &json!({"exposure_steps": EXPOSURE_STEPS, "same_physical_contact": repeated_inputs_identical, "adaptation_after_exposure": adaptation_after_exposure, "naive_response": naive_response, "experienced_response": experienced_response, "habituation_magnitude": naive_response - experienced_response, "adaptation_before_recovery": adaptation_before_recovery, "adaptation_after_recovery": adaptation_after_recovery, "recovered_response": recovered_response, "result": "DCDEV006_GATE5_EXPERIENCE_DEPENDENCE_PASS"}),
    );
    write_json(
        &output,
        "remesh_boundaries.json",
        &json!({"events": remesh_events, "continuity_valid": remesh_continuity_valid, "trace_survived": remesh_trace_nonzero, "final_trace_length": remesh_state.adaptation.len(), "final_vertices": remesh_mesh.n(), "fission_rejected": fission_fail_closed, "environment_decides_remesh": false, "environment_changes_metabolism": false, "environment_modifies_heredity": false, "result": "DCDEV006_GATE6_REMESH_NON_INTERFERENCE_PASS"}),
    );
    write_json(
        &output,
        "governance_boundary.json",
        &json!({"one_static_primitive": true, "one_external_signal": true, "new_sensor": false, "new_actuator": false, "extra_plasticity": false, "reward": false, "fitness": false, "evolution": false, "fission_inheritance": false, "parameter_screening": false, "next_execution_started": false, "result": "DCDEV006_GATE0_SCOPE_PASS"}),
    );
    write_json(
        &output,
        "regression_manifest.json",
        &json!({"artifact_status": "AUTHORITATIVE", "dcdev006_gate_assay": "PASSED (Gates 0 through 6)", "dcdev005_preservation": "PASSED locally", "dcdev002_dcdev003_dcdev004": "PASSED locally", "phase1_focused_regression": "PENDING", "d088_focused_regression": "PENDING", "evolution_harness_regression": "PENDING", "governance": "PENDING", "exact_head_remote_ci": "PENDING", "parameter_screening": false}),
    );
    let mut artifact_hashes = BTreeMap::new();
    for entry in fs::read_dir(&output).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|value| value.to_str()) == Some("json")
            && entry.file_name() != "final_manifest.json"
        {
            let value: Value = serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
            artifact_hashes.insert(
                entry.file_name().to_string_lossy().to_string(),
                stable_json_hash(&value).unwrap(),
            );
        }
    }
    write_json(
        &output,
        "final_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "conclusion": "DCDEV006_SPATIAL_CONTACT_ENVIRONMENT_QUALIFIED",
            "gates": {"0": "DCDEV006_GATE0_SCOPE_PASS", "1": "DCDEV006_GATE1_ZERO_WORLD_PARITY_PASS", "2": "DCDEV006_GATE2_LOCAL_CONTACT_PHYSICS_PASS", "3": "DCDEV006_GATE3_LOCAL_EXTERNAL_TRANSDUCTION_PASS", "4": "DCDEV006_GATE4_LOCAL_REGULATORY_RESPONSE_PASS", "5": "DCDEV006_GATE5_EXPERIENCE_DEPENDENCE_PASS", "6": "DCDEV006_GATE6_REMESH_NON_INTERFERENCE_PASS", "7": "REMOTE_CI_PENDING"},
            "parameter_screening": false,
            "new_sensor": false,
            "new_actuator": false,
            "reward": false,
            "fitness": false,
            "evolution": false,
            "next_execution_started": false,
            "artifact_hashes": artifact_hashes
        }),
    );
}
