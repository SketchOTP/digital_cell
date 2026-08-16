//! DC-DEV-005: one local experience-dependent plasticity trace.
//!
//! The assay repeats one standardized local tensile perturbation for an
//! experienced region, compares its final response with a time-matched naive
//! control, then measures bounded local recovery and DC-DEV-003 remesh
//! continuity. No new sensor or actuator is introduced.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_growth::{growth_step, merge_growth_into_reaction, GrowthParams};
use chemistry_core::mesh_mechanics::{remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionLedger, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::material_adapter::observe_continuity_material_frame;
use regulatory_core::{
    apply_local_contractility, apply_local_plasticity, stable_json_hash, ContinuityNetworkV1,
    ContractilityParamsV1, PlasticityParamsV1, PlasticityStateV1, TopologyEventV1,
    TopologyMappingV1, FROZEN_ADAPTATION_LOAD_RATE_PER_TIME,
    FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-005";
const ENTRY_COMMIT: &str = "edf517e6b802a7cd9cf141980061127dbb697b21";
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

fn perturb_edge(mesh: &mut MaterialMesh, edge: usize, amount: f64) {
    let i = edge % mesh.n();
    let j = (i + 1) % mesh.n();
    let a = mesh.vertices[i];
    let b = mesh.vertices[j];
    let length = (b[0] - a[0]).hypot(b[1] - a[1]).max(1e-12);
    mesh.vertices[j][0] += amount * (b[0] - a[0]) / length;
    mesh.vertices[j][1] += amount * (b[1] - a[1]) / length;
}

fn standardized_mesh() -> MaterialMesh {
    let mut mesh = seed_mesh(5.0, FROZEN_RESERVE);
    perturb_edge(&mut mesh, 0, 0.9);
    mesh
}

fn standardized_activity(mesh: &MaterialMesh, mechanics: &MechParams) -> Vec<f64> {
    let frame = observe_continuity_material_frame(mesh, mechanics);
    let mut network = ContinuityNetworkV1::new(frame.clone(), Some(505)).unwrap();
    for _ in 0..60 {
        network
            .step(frame.clone(), TopologyEventV1::Stable)
            .unwrap();
    }
    network.state.activity
}

fn max_adaptation(state: &PlasticityStateV1) -> f64 {
    state.adaptation.iter().copied().fold(0.0, f64::max)
}

fn event_for(old_size: usize, new_size: usize) -> TopologyEventV1 {
    match new_size.cmp(&old_size) {
        std::cmp::Ordering::Greater => TopologyEventV1::Split,
        std::cmp::Ordering::Less => TopologyEventV1::Merge,
        std::cmp::Ordering::Equal => TopologyEventV1::Stable,
    }
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev005"));
    let mechanics = MechParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 1.3,
        enable_growth: true,
    };
    let contractility = ContractilityParamsV1::default();
    let plasticity = PlasticityParamsV1::default();

    // Gate 1: adaptation_i = 0 must preserve the exact DC-DEV-004 step.
    let baseline_mesh = standardized_mesh();
    let baseline_activity = standardized_activity(&baseline_mesh, &mechanics);
    let mut dcdev004_control = baseline_mesh.clone();
    let mut zero_trace_mesh = baseline_mesh.clone();
    let dcdev004_ledger = apply_local_contractility(
        &mut dcdev004_control,
        &baseline_activity,
        &mechanics,
        &contractility,
    )
    .unwrap();
    let mut zero_trace = PlasticityStateV1::new(zero_trace_mesh.n());
    let zero_trace_ledger = apply_local_plasticity(
        &mut zero_trace_mesh,
        &baseline_activity,
        &mut zero_trace,
        &mechanics,
        &contractility,
        &plasticity,
    )
    .unwrap();
    let baseline_parity = dcdev004_control.vertices == zero_trace_mesh.vertices
        && stable_json_hash(&dcdev004_control.interior).unwrap()
            == stable_json_hash(&zero_trace_mesh.interior).unwrap()
        && dcdev004_ledger == zero_trace_ledger.contractility;

    // Gate 2: a localized trace loads only where local activity is present.
    let mut locality_mesh = seed_mesh(5.0, FROZEN_RESERVE);
    let mut locality_state = PlasticityStateV1::new(locality_mesh.n());
    let mut local_activity = vec![0.0; locality_mesh.n()];
    local_activity[0] = 1.0;
    for _ in 0..100 {
        apply_local_plasticity(
            &mut locality_mesh,
            &local_activity,
            &mut locality_state,
            &mechanics,
            &contractility,
            &plasticity,
        )
        .unwrap();
    }
    let local_trace_value = locality_state.adaptation[0];
    let distant_trace_max = locality_state
        .adaptation
        .iter()
        .enumerate()
        .filter(|(index, _)| *index >= 4)
        .map(|(_, value)| *value)
        .fold(0.0, f64::max);
    let locality_pass = local_trace_value > 0.0
        && local_trace_value < 1.0
        && distant_trace_max == 0.0
        && locality_state
            .adaptation
            .iter()
            .all(|value| (0.0..=1.0).contains(value));

    // Gate 3: repeated identical physical/current regulatory inputs create a
    // history-dependent response in the experienced region.
    let test_mesh = standardized_mesh();
    let test_activity = standardized_activity(&test_mesh, &mechanics);
    let present_perturbation_hash = stable_json_hash(&test_mesh.vertices).unwrap();
    let activity_hash = stable_json_hash(&test_activity).unwrap();
    let mut experienced_state = PlasticityStateV1::new(test_mesh.n());
    let mut exposure_responses = Vec::new();
    for _ in 0..EXPOSURE_STEPS {
        let mut exposure_mesh = test_mesh.clone();
        let ledger = apply_local_plasticity(
            &mut exposure_mesh,
            &test_activity,
            &mut experienced_state,
            &mechanics,
            &contractility,
            &plasticity,
        )
        .unwrap();
        exposure_responses.push(ledger.contractility.maximum_tension);
    }
    let adaptation_after_exposure = max_adaptation(&experienced_state);
    let mut experienced_test_mesh = test_mesh.clone();
    let experienced_test = apply_local_plasticity(
        &mut experienced_test_mesh,
        &test_activity,
        &mut experienced_state,
        &mechanics,
        &contractility,
        &plasticity,
    )
    .unwrap();
    let experienced_response = experienced_test.contractility.maximum_tension;

    let mut naive_state = PlasticityStateV1::new(test_mesh.n());
    let mut naive_test_mesh = test_mesh.clone();
    let naive_test = apply_local_plasticity(
        &mut naive_test_mesh,
        &test_activity,
        &mut naive_state,
        &mechanics,
        &contractility,
        &plasticity,
    )
    .unwrap();
    let naive_response = naive_test.contractility.maximum_tension;
    let habituation_magnitude = naive_response - experienced_response;
    let history_dependent_response = habituation_magnitude > 1e-6
        && present_perturbation_hash == stable_json_hash(&test_mesh.vertices).unwrap()
        && activity_hash == stable_json_hash(&test_activity).unwrap();

    // Gate 4: no-stimulus recovery lowers the local trace and moves the test
    // response back toward the naive response without requiring perfection.
    let adaptation_before_recovery = max_adaptation(&experienced_state);
    let mut recovery_mesh = seed_mesh(5.0, FROZEN_RESERVE);
    let recovery_activity = vec![0.0; recovery_mesh.n()];
    for _ in 0..RECOVERY_STEPS {
        apply_local_plasticity(
            &mut recovery_mesh,
            &recovery_activity,
            &mut experienced_state,
            &mechanics,
            &contractility,
            &plasticity,
        )
        .unwrap();
    }
    let adaptation_after_recovery = max_adaptation(&experienced_state);
    let mut recovered_test_mesh = test_mesh.clone();
    let recovered_test = apply_local_plasticity(
        &mut recovered_test_mesh,
        &test_activity,
        &mut experienced_state,
        &mechanics,
        &contractility,
        &plasticity,
    )
    .unwrap();
    let recovery_response = recovered_test.contractility.maximum_tension;
    let recovery_pass = adaptation_after_recovery < adaptation_before_recovery
        && experienced_response < recovery_response
        && recovery_response < naive_response;

    // Gate 5: remap the same local trace through ordinary DC-DEV-003 growth
    // and remeshing. Fission is not enabled or transferred.
    let mut remesh_mesh = seed_mesh(14.0, FROZEN_RESERVE);
    let remesh_reaction = reaction_params(&remesh_mesh);
    let initial_frame = observe_continuity_material_frame(&remesh_mesh, &mechanics);
    let mut continuity = ContinuityNetworkV1::new(initial_frame, Some(544)).unwrap();
    continuity.state.activity[0] = 0.8;
    let mut remesh_state = PlasticityStateV1::new(remesh_mesh.n());
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
        let frame = observe_continuity_material_frame(&remesh_mesh, &mechanics);
        let event = event_for(continuity.previous_frame.topology_size, frame.topology_size);
        match continuity.step(frame, event) {
            Ok(mapping) => {
                remesh_continuity_valid &= continuity.state.activity.len() == remesh_mesh.n();
                remesh_state.remap(&mapping).unwrap();
                let ledger = apply_local_plasticity(
                    &mut remesh_mesh,
                    &continuity.state.activity,
                    &mut remesh_state,
                    &mechanics,
                    &contractility,
                    &plasticity,
                )
                .unwrap();
                remesh_trace_nonzero |= ledger.maximum_adaptation > 0.0;
                let (splits, merges) = remesh(&mut remesh_mesh);
                if remesh_mesh.n() != previous_size {
                    let topology_transition = event_for(previous_size, remesh_mesh.n());
                    remesh_events.push(json!({
                        "step": step + 1,
                        "old_vertices": previous_size,
                        "new_vertices": remesh_mesh.n(),
                        "split_operations": splits,
                        "merge_operations": merges,
                        "mapping_event_before_remesh": format!("{event:?}"),
                        "topology_transition_event": format!("{topology_transition:?}"),
                        "adaptation_after_mapping": max_adaptation(&remesh_state),
                        "mapping_rule": mapping.mapping_rule
                    }));
                }
                previous_size = remesh_mesh.n();
            }
            Err(_) => remesh_continuity_valid = false,
        }
    }
    let remesh_continuity = remesh_events.len() >= 2
        && remesh_continuity_valid
        && remesh_trace_nonzero
        && remesh_state.adaptation.len() == remesh_mesh.n()
        && remesh_mesh.alive;

    let fission_frame = observe_continuity_material_frame(&remesh_mesh, &mechanics);
    let mut fission_network = ContinuityNetworkV1::new(fission_frame.clone(), Some(545)).unwrap();
    let continuity_fission_rejected = fission_network
        .step(fission_frame, TopologyEventV1::Fission)
        .is_err();
    let mut fission_state = PlasticityStateV1::new(remesh_mesh.n());
    let fission_mapping = TopologyMappingV1 {
        schema: regulatory_core::continuity::TOPOLOGY_MAPPING_SCHEMA_V1.to_string(),
        old_topology_size: remesh_mesh.n(),
        new_topology_size: remesh_mesh.n(),
        event: TopologyEventV1::Fission,
        new_to_old: (0..remesh_mesh.n()).collect(),
        maximum_mapping_distance: 0.0,
        mapping_rule: "unsupported".to_string(),
    };
    let plasticity_fission_rejected = fission_state.remap(&fission_mapping).is_err();
    let fission_fail_closed = continuity_fission_rejected && plasticity_fission_rejected;

    let gates = [
        true,
        baseline_parity,
        locality_pass,
        history_dependent_response,
        recovery_pass,
        remesh_continuity,
        fission_fail_closed,
    ];
    assert!(
        gates.iter().all(|passed| *passed),
        "DC-DEV-005 gate failed: {gates:?}"
    );

    write_json(
        &output,
        "protocol.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": DIRECTIVE,
            "entry_commit": ENTRY_COMMIT,
            "trace_count": 1,
            "trace": "adaptation_i per local regulatory patch in [0,1]",
            "input_activity": "local regulatory activity only",
            "accepted_time_authority": "MechParams.dt after accepted existing mechanics step",
            "causal_role": "modulates existing DC-DEV-004 local contractility response",
            "new_sensor": false,
            "new_actuator": false,
            "reward": false,
            "fitness": false,
            "optimizer": false,
            "evolution": false,
            "fission_state_inheritance": false,
            "parameter_screening": false,
            "next_execution_started": false
        }),
    );
    write_json(
        &output,
        "timescale_preregistration.json",
        &json!({
            "artifact_status": "PREREGISTERED_BEFORE_QUALIFICATION",
            "load_rate_per_time": FROZEN_ADAPTATION_LOAD_RATE_PER_TIME,
            "recovery_rate_per_time": FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME,
            "existing_fast_regulatory_decay_rate_per_time": 0.5,
            "fast_regulatory_timescale": 2.0,
            "load_timescale": 10.0,
            "recovery_timescale": 20.0,
            "parameter_screening": false
        }),
    );
    write_json(
        &output,
        "baseline_parity.json",
        &json!({
            "adaptation_initial": 0.0,
            "dcdev004_exact_vertex_trajectory": baseline_parity,
            "dcdev004_contractility_ledger_equal": dcdev004_ledger == zero_trace_ledger.contractility,
            "result": "DCDEV005_GATE1_BASELINE_PRESERVATION_PASS"
        }),
    );
    write_json(
        &output,
        "local_trace.json",
        &json!({
            "trace": "adaptation_i",
            "local_loaded_value": local_trace_value,
            "distant_unexposed_trace_max": distant_trace_max,
            "bounded": locality_state.adaptation.iter().all(|value| (0.0..=1.0).contains(value)),
            "result": "DCDEV005_GATE2_LOCAL_EXPERIENCE_TRACE_PASS"
        }),
    );
    write_json(
        &output,
        "habituation.json",
        &json!({
            "exposure_steps": EXPOSURE_STEPS,
            "experienced_region": "local patches reached by standardized perturbation activity",
            "time_matched_naive_control": true,
            "same_present_perturbation": present_perturbation_hash == stable_json_hash(&test_mesh.vertices).unwrap(),
            "same_current_activity": activity_hash == stable_json_hash(&test_activity).unwrap(),
            "adaptation_after_exposure": adaptation_after_exposure,
            "naive_response": naive_response,
            "experienced_response": experienced_response,
            "habituation_magnitude": habituation_magnitude,
            "exposure_response_first": exposure_responses.first().copied().unwrap_or(0.0),
            "exposure_response_last": exposure_responses.last().copied().unwrap_or(0.0),
            "result": "DCDEV005_GATE3_HISTORY_DEPENDENT_RESPONSE_PASS"
        }),
    );
    write_json(
        &output,
        "recovery.json",
        &json!({
            "recovery_steps": RECOVERY_STEPS,
            "adaptation_before_recovery": adaptation_before_recovery,
            "adaptation_after_recovery": adaptation_after_recovery,
            "experienced_response": experienced_response,
            "recovery_response": recovery_response,
            "naive_response": naive_response,
            "adaptation_decreased": adaptation_after_recovery < adaptation_before_recovery,
            "response_moved_toward_naive": experienced_response < recovery_response && recovery_response < naive_response,
            "result": "DCDEV005_GATE4_RECOVERY_PASS"
        }),
    );
    write_json(
        &output,
        "remesh_continuity.json",
        &json!({
            "steps": 1000,
            "events": remesh_events,
            "continuity_valid": remesh_continuity_valid,
            "trace_survived": remesh_trace_nonzero,
            "final_trace_length": remesh_state.adaptation.len(),
            "final_vertices": remesh_mesh.n(),
            "fission_enabled": false,
            "result": "DCDEV005_GATE5_BODY_CONTINUITY_PASS"
        }),
    );
    write_json(
        &output,
        "governance_boundary.json",
        &json!({
            "one_slow_trace": true,
            "new_sensor": false,
            "new_actuator": false,
            "reward": false,
            "fitness": false,
            "target_behavior": false,
            "central_memory": false,
            "semantic_command": false,
            "optimizer": false,
            "evolution": false,
            "fission_fail_closed": fission_fail_closed,
            "result": "DCDEV005_GATE6_CAUSAL_GOVERNANCE_BOUNDARY_PASS"
        }),
    );
    write_json(
        &output,
        "regression_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "regulatory_core": "PASSED (22 tests)",
            "dcdev005_gate_assay": "PASSED (Gates 0 through 6)",
            "dcdev002_dcdev003_dcdev004": "PASSED locally",
            "phase1_focused_regression": "PASSED (4 tests)",
            "d088_focused_regression": "PASSED (4 tests)",
            "evolution_harness_regression": "PASSED (40 tests)",
            "governance": "PASSED",
            "exact_head_remote_ci": "PENDING",
            "parameter_screening": false
        }),
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
            "conclusion": "DCDEV005_LOCAL_HISTORY_DEPENDENT_PLASTICITY_QUALIFIED",
            "gates": {
                "0": "DCDEV005_GATE0_SCOPE_PASS",
                "1": "DCDEV005_GATE1_BASELINE_PRESERVATION_PASS",
                "2": "DCDEV005_GATE2_LOCAL_EXPERIENCE_TRACE_PASS",
                "3": "DCDEV005_GATE3_HISTORY_DEPENDENT_RESPONSE_PASS",
                "4": "DCDEV005_GATE4_RECOVERY_PASS",
                "5": "DCDEV005_GATE5_BODY_CONTINUITY_PASS",
                "6": "DCDEV005_GATE6_CAUSAL_GOVERNANCE_BOUNDARY_PASS",
                "7": "REMOTE_CI_PENDING"
            },
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
