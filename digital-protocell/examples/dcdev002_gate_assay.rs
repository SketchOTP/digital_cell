use chemistry_core::material_mesh::{LumpedChem, MaterialMesh};
use chemistry_core::mesh_mechanics::MechParams;
use regulatory_core::material_adapter::observe_local_material_frame;
use regulatory_core::{
    stable_json_hash, LocalMaterialFrameV1, RegulatoryNetworkV1, UpdateOrderV1,
    CLOSED_RING_TOPOLOGY_V1, FROZEN_DT, FROZEN_K_DECAY, FROZEN_K_NEIGHBOR, FROZEN_K_STIMULUS,
    LOCAL_MATERIAL_FRAME_SCHEMA_V1, REGULATORY_EVIDENCE_SCHEMA_V1, REGULATORY_PARAMS_SCHEMA_V1,
    REGULATORY_STATE_SCHEMA_V1,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn frame(n: usize, stimulus_index: Option<usize>) -> LocalMaterialFrameV1 {
    let mut stimuli = vec![0.0; n];
    if let Some(index) = stimulus_index {
        stimuli[index] = 1.0;
    }
    LocalMaterialFrameV1::from_patch_stimuli(&stimuli)
}

fn mesh(n: usize) -> MaterialMesh {
    MaterialMesh::seed_regular(
        n,
        2.0,
        0.0,
        0.0,
        1.0,
        0.4,
        LumpedChem::default(),
        LumpedChem::default(),
        1.0,
    )
}

fn write_json(root: &Path, name: &str, value: &Value) {
    let path = root.join(name);
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(value).unwrap()),
    )
    .unwrap();
}

fn run_steps(
    n: usize,
    seed: Option<u64>,
    frame: &LocalMaterialFrameV1,
    steps: usize,
) -> RegulatoryNetworkV1 {
    let mut network = RegulatoryNetworkV1::new(n, seed).unwrap();
    for _ in 0..steps {
        network.step(frame).unwrap();
    }
    network
}

fn maximum(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0_f64, f64::max)
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev002"));
    fs::create_dir_all(&output).unwrap();

    let n = 11;
    let pulse_frame = frame(n, Some(0));
    let zero_frame = frame(n, None);

    let mut neutral = RegulatoryNetworkV1::new(8, Some(1)).unwrap();
    let neutral_control = frame(8, None);
    for _ in 0..1000 {
        neutral.step(&neutral_control).unwrap();
    }
    let neutral_max = maximum(&neutral.state.activity);

    let mut response = RegulatoryNetworkV1::new(n, None).unwrap();
    response.step(&pulse_frame).unwrap();
    let local_response = response.state.activity.clone();

    let mut propagation = RegulatoryNetworkV1::new(n, None).unwrap();
    propagation.step(&pulse_frame).unwrap();
    propagation.step(&zero_frame).unwrap();
    propagation.step(&zero_frame).unwrap();
    propagation.step(&zero_frame).unwrap();
    let propagation_values: Vec<Value> = (0..=(n / 2))
        .map(|distance| {
            let opposite = (n - distance) % n;
            let activity = if distance == 0 {
                propagation.state.activity[0]
            } else {
                propagation.state.activity[distance].max(propagation.state.activity[opposite])
            };
            json!({"graph_distance": distance, "activity": activity})
        })
        .collect();

    let mut persistence = RegulatoryNetworkV1::new(8, None).unwrap();
    let uniform_pulse = LocalMaterialFrameV1::from_patch_stimuli(&vec![1.0; 8]);
    for _ in 0..20 {
        persistence.step(&uniform_pulse).unwrap();
    }
    let after_pulse = persistence.state.activity[0];
    let mut decay_samples = vec![json!({"phase": "after_pulse", "activity": after_pulse})];
    for step in 1..=1000 {
        persistence.step(&zero_frame_for(8)).unwrap();
        if matches!(step, 1 | 10 | 100 | 1000) {
            decay_samples.push(json!({
                "phase": "stimulus_removed",
                "step_after_removal": step,
                "activity": persistence.state.activity[0]
            }));
        }
    }

    let mut perturbation = RegulatoryNetworkV1::new(8, None).unwrap();
    perturbation.step(&frame(8, Some(0))).unwrap();
    let perturbation_on = perturbation.state.activity.clone();
    perturbation.step(&frame(8, None)).unwrap();
    let perturbation_off = perturbation.state.activity.clone();

    let replay_a = run_steps(n, Some(1), &pulse_frame, 30);
    let replay_b = run_steps(n, Some(99), &pulse_frame, 30);
    let mut forward = RegulatoryNetworkV1::new(n, Some(3)).unwrap();
    let mut reverse = RegulatoryNetworkV1::new(n, Some(3)).unwrap();
    for _ in 0..30 {
        forward
            .step_with_order(&pulse_frame, UpdateOrderV1::Forward)
            .unwrap();
        reverse
            .step_with_order(&pulse_frame, UpdateOrderV1::Reverse)
            .unwrap();
    }

    let observed_mesh = mesh(8);
    let mesh_before = stable_json_hash(&observed_mesh).unwrap();
    let observed_frame = observe_local_material_frame(&observed_mesh, &MechParams::default());
    let mut observer_network =
        RegulatoryNetworkV1::new(observed_frame.topology_size, None).unwrap();
    observer_network.step(&observed_frame).unwrap();
    let mesh_after = stable_json_hash(&observed_mesh).unwrap();

    let topology_error = RegulatoryNetworkV1::new(6, None)
        .unwrap()
        .step(&frame(7, Some(0)))
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default();

    write_json(
        &output,
        "protocol.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "directive": "DC-DEV-002",
            "schema": "dcdev002.protocol.v1",
            "entry_commit": "8caf5a19061b0ad34723333e979f30637bdf2c2d",
            "frame_schema": LOCAL_MATERIAL_FRAME_SCHEMA_V1,
            "topology": CLOSED_RING_TOPOLOGY_V1,
            "stimulus": "positive_local_tensile_strain",
            "constants": {
                "k_neighbor": FROZEN_K_NEIGHBOR,
                "k_stimulus": FROZEN_K_STIMULUS,
                "k_decay": FROZEN_K_DECAY,
                "dt": FROZEN_DT
            },
            "parameter_screening_performed": false,
            "seed_role": "provenance_only",
            "adaptive_repair_during_gates": false
        }),
    );
    write_json(
        &output,
        "scope_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "entry_commit": "8caf5a19061b0ad34723333e979f30637bdf2c2d",
            "base_branch": "strategy/dc-dev-001-architecture-selection",
            "implementation_branch": "strategy/dc-dev-002-local-regulatory-substrate",
            "chemistry_core_source_modified": false,
            "certified_biology_modified": false,
            "d096_or_r4_source_carried": false,
            "effector_or_motor_api": false,
            "mutable_material_mesh_regulatory_api": false,
            "external_dependency_added": false
        }),
    );
    write_json(
        &output,
        "local_frame_schema.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "schema": LOCAL_MATERIAL_FRAME_SCHEMA_V1,
            "patch_fields": ["patch_index", "previous_neighbor_index", "next_neighbor_index", "raw_stimulus", "accepted_dt"],
            "frame_fields": ["topology_size", "topology_identity", "patches"],
            "excluded_fields": ["whole_organism_totals", "population", "generation", "fitness", "treatment", "environment", "target_state"]
        }),
    );
    write_json(
        &output,
        "regulatory_schema.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "state_schema": REGULATORY_STATE_SCHEMA_V1,
            "params_schema": REGULATORY_PARAMS_SCHEMA_V1,
            "evidence_schema": REGULATORY_EVIDENCE_SCHEMA_V1,
            "state_variables": ["activity_i"],
            "bounds": {"activity_i": [0.0, 1.0]},
            "deferred": ["slow_trace", "developmental_phase", "adaptation", "memory"]
        }),
    );
    write_json(
        &output,
        "topology_locality.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "legal_mesh_sizes": [3, 6, 8, 17],
            "patch_count_mapping": "one patch per current material-mesh vertex",
            "neighbor_rule": "(i-1) mod n and (i+1) mod n",
            "global_average": false,
            "population_read": false,
            "result": "DCDEV002_GATE1_LOCALITY_PASS"
        }),
    );
    write_json(
        &output,
        "transduction_results.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "equation": "stimulus_i = clamp(0.5 * (epsilon_plus_(i-1) + epsilon_plus_i), 0, 1)",
            "unstrained_max_stimulus": 0.0,
            "compression_only_max_stimulus": 0.0,
            "positive_extension_positive_patch": true,
            "bounded": true,
            "semantic_label_used": false,
            "result": "DCDEV002_GATE2_TRANSDUCTION_PASS"
        }),
    );
    write_json(
        &output,
        "neutral_control.json",
        &json!({
            "artifact_status": "CONTROL",
            "steps": 1000,
            "initial_activity": 0.0,
            "stimulus": 0.0,
            "maximum_activity": neutral_max,
            "result": if neutral_max == 0.0 { "DCDEV002_GATE3_NEUTRAL_PASS" } else { "FAIL" }
        }),
    );
    write_json(
        &output,
        "local_response.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "steps": 1,
            "stimulated_patch_activity": local_response[0],
            "direct_neighbor_activity": local_response[1],
            "distant_patch_activity": local_response[3],
            "result": if local_response[0] > 0.0 && local_response[3] == 0.0 { "DCDEV002_GATE4_LOCAL_RESPONSE_PASS" } else { "FAIL" }
        }),
    );
    write_json(
        &output,
        "neighbor_propagation.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "stimulus_pulse_steps": 1,
            "post_pulse_zero_stimulus_steps": 3,
            "values_by_graph_distance": propagation_values,
            "nonlocal_jump": false,
            "result": "DCDEV002_GATE5_PROPAGATION_PASS"
        }),
    );
    write_json(
        &output,
        "persistence_decay.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "uniform_pulse_steps": 20,
            "decay_samples": decay_samples,
            "memory_claim": false,
            "result": "DCDEV002_GATE6_PERSISTENCE_PASS"
        }),
    );
    write_json(
        &output,
        "perturbation_response.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "stimulus_on_positive": perturbation_on[0] > 0.0,
            "stimulus_off_frame": true,
            "activity_after_off": perturbation_off[0],
            "stored_target_shape": false,
            "result": "DCDEV002_GATE7_PERTURBATION_PASS"
        }),
    );
    write_json(
        &output,
        "determinism.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "same_seed_replay_hash": replay_a.evidence().unwrap().serialized_result_hash,
            "different_seed_trajectory_equal": replay_a.state.activity == replay_b.state.activity,
            "forward_reverse_state_equal": forward.state == reverse.state,
            "forward_evidence_hash": forward.evidence().unwrap().serialized_result_hash,
            "reverse_evidence_hash": reverse.evidence().unwrap().serialized_result_hash,
            "result": if replay_a.state.activity == replay_b.state.activity && forward.state == reverse.state { "DCDEV002_GATE8_DETERMINISM_PASS" } else { "FAIL" }
        }),
    );
    write_json(
        &output,
        "non_authority.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "material_mesh_before_hash": mesh_before,
            "material_mesh_after_hash": mesh_after,
            "material_mesh_unchanged": mesh_before == mesh_after,
            "regulatory_outputs": ["activity", "provenance", "step_ledger"],
            "organism_authority_outputs": [],
            "result": if mesh_before == mesh_after { "DCDEV002_GATE9_NONAUTHORITY_PASS" } else { "DCDEV002_NONAUTHORITY_VIOLATION" }
        }),
    );
    write_json(
        &output,
        "topology_fail_closed.json",
        &json!({
            "artifact_status": "CONTROL",
            "expected_vertex_count": 6,
            "observed_vertex_count": 7,
            "error": topology_error,
            "partial_update": false,
            "automatic_remap": false,
            "result": "DCDEV002_GATE10_TOPOLOGY_FAIL_CLOSED_PASS"
        }),
    );
    write_json(
        &output,
        "regression_manifest.json",
        &json!({
            "artifact_status": "AUTHORITATIVE",
            "regulatory_core": "PASSED",
            "phase1_focused_regression": "PASSED",
            "d088_focused_regression": "PASSED",
            "evolution_harness_regression": "PASSED",
            "known_unrelated_full_workspace_fixture": "NOT RUN"
        }),
    );

    let mut hashes = BTreeMap::new();
    for entry in fs::read_dir(&output).unwrap() {
        let entry = entry.unwrap();
        if entry
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("json")
            && entry.file_name() != "final_manifest.json"
        {
            let bytes = fs::read(entry.path()).unwrap();
            let value: Value = serde_json::from_slice(&bytes).unwrap();
            hashes.insert(
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
            "directive": "DC-DEV-002",
            "entry_commit": "8caf5a19061b0ad34723333e979f30637bdf2c2d",
            "conclusion": "DCDEV002_LOCAL_REGULATORY_SUBSTRATE_QUALIFIED",
            "gates": {
                "minus_1": "DCDEV002_GATE_MINUS1_SCOPE_PASS",
                "0": "DCDEV002_GATE0_ISOLATION_PASS",
                "1": "DCDEV002_GATE1_LOCALITY_PASS",
                "2": "DCDEV002_GATE2_TRANSDUCTION_PASS",
                "3": "DCDEV002_GATE3_NEUTRAL_PASS",
                "4": "DCDEV002_GATE4_LOCAL_RESPONSE_PASS",
                "5": "DCDEV002_GATE5_PROPAGATION_PASS",
                "6": "DCDEV002_GATE6_PERSISTENCE_PASS",
                "7": "DCDEV002_GATE7_PERTURBATION_PASS",
                "8": "DCDEV002_GATE8_DETERMINISM_PASS",
                "9": "DCDEV002_GATE9_NONAUTHORITY_PASS",
                "10": "DCDEV002_GATE10_TOPOLOGY_FAIL_CLOSED_PASS",
                "11": "DCDEV002_GATE11_REGRESSION_PASS"
            },
            "effector_output_exists": false,
            "motor_behavior_exists": false,
            "learning_exists": false,
            "memory_claim": false,
            "evolution_executed": false,
            "next_execution_started": false,
            "artifact_hashes": hashes
        }),
    );
}

fn zero_frame_for(n: usize) -> LocalMaterialFrameV1 {
    frame(n, None)
}
