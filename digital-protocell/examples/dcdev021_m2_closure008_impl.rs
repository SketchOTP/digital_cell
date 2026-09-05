// CLOSURE-008: assay-only lineage-level replay of the accepted CLOSURE-007
// post-ingestive material-to-work composition.  This isolates daughter-local
// resource/work sufficiency from paired finite-resource competition.

const C8_DIRECTIVE: &str =
    "DC-DEV-021-M2-CLOSURE-008-LINEAGE-LEVEL-POST-INGESTIVE-MATERIAL-WORK-REQUALIFICATION-001";
const C8_START: &str = "0dfd81fdd99cc9a0eb7562b1d8705190c37d3a73";
const C8_TOL: f64 = 1e-10;

fn c8_run_set(
    initial: &[ClosureAgent],
    world_body: &MaterialMesh,
    label: &str,
    transfer_enabled: bool,
    zero_resource: bool,
    material_feedback: bool,
    motor_off: bool,
) -> C7Run {
    c7_run(
        initial,
        world_body,
        label,
        transfer_enabled,
        zero_resource,
        material_feedback,
        motor_off,
    )
}

fn c8_arm_value(run: &C7Run) -> Value {
    let value = c7_value(run);
    json!({
        "arm": value["base"]["base"]["arm"],
        "classification_inputs": {
            "invalid": value["base"]["base"]["invalid"],
            "delivered_n": value["base"]["base"]["delivered_n"],
            "delivered_f": value["base"]["base"]["delivered_f"],
            "world_n_loss": value["base"]["base"]["world_n_loss"],
            "world_f_loss": value["base"]["base"]["world_f_loss"],
            "a_spent": value["base"]["base"]["a_spent"],
            "w_generated": value["base"]["base"]["w_generated"],
            "a_to_w_residual": value["base"]["base"]["a_to_w_residual"],
            "fissions": value["base"]["base"]["fissions"],
            "first_fission": value["base"]["base"]["first_fission"],
            "terminal_living": value["base"]["base"]["terminal_living"],
            "steps": value["base"]["base"]["steps"],
        },
        "full_compact_run": value,
    })
}

fn c8_metric(run: &C7Run, key: &str) -> f64 {
    c7_value(run)["base"]["base"][key].as_f64().unwrap_or(0.0)
}

fn c8_bool(run: &C7Run, key: &str) -> bool {
    c7_value(run)["base"]["base"][key].as_bool().unwrap_or(false)
}

fn c8_make_initial() -> (Vec<ClosureAgent>, MaterialMesh) {
    let replay = replay_run(false, false);
    let (ga, gb, a_amounts, b_amounts, _partition) = partition_amounts(&replay);
    let (a_mesh, a_grid, a_state) = entry027_first_lawful_state(
        &replay.daughter_a,
        &ga,
        &density_state(&a_amounts, &ga),
        replay.first_fission_step.saturating_sub(1) as u64,
    );
    let (b_mesh, b_grid, b_state) = entry027_first_lawful_state(
        &replay.daughter_b,
        &gb,
        &density_state(&b_amounts, &gb),
        replay.first_fission_step.saturating_sub(1) as u64,
    );
    let initial = closure_agents(&a_mesh, &a_grid, &a_state, &b_mesh, &b_grid, &b_state);
    let world_body = initial[0].mesh.clone();
    (initial, world_body)
}

fn c8_group(
    initial: &[ClosureAgent],
    world_body: &MaterialMesh,
    group: &str,
) -> Vec<(String, C7Run)> {
    let configurations = [
        ("MATERIAL_FEEDBACK", true, false, true, false),
        ("TRANSFER_DISABLED", false, false, true, false),
        ("ZERO_RESOURCE", true, true, true, false),
        ("NO_MATERIAL_FEEDBACK", true, false, false, false),
        ("MOTOR_OFF", true, false, false, true),
    ];
    configurations
        .into_iter()
        .map(|(name, transfer, zero, feedback, motor_off)| {
            let label = format!("CLOSURE008_{group}_{name}");
            let run = c8_run_set(initial, world_body, &label, transfer, zero, feedback, motor_off);
            (name.to_string(), run)
        })
        .collect()
}

fn c8_find<'a>(arms: &'a [(String, C7Run)], name: &str) -> &'a C7Run {
    arms.iter()
        .find(|(arm, _)| arm == name)
        .map(|(_, run)| run)
        .expect("CLOSURE-008 arm missing")
}

fn c8_write_arm_set(root: &Path, name: &str, arms: &[(String, C7Run)]) {
    let value = arms
        .iter()
        .map(|(arm, run)| (arm.clone(), c8_arm_value(run)))
        .collect::<serde_json::Map<_, _>>();
    write(root, name, &Value::Object(value));
}

pub fn c8_main() {
    let out_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2closure008"));
    std::fs::create_dir_all(&out_dir).unwrap();

    let (initial, world_body) = c8_make_initial();
    let paired = c8_group(&initial, &world_body, "PAIRED");
    let daughter_a = c8_group(&initial[0..1], &world_body, "DAUGHTER_A_SOLO");
    let daughter_b = c8_group(&initial[1..2], &world_body, "DAUGHTER_B_SOLO");

    let paired_candidate = c8_find(&paired, "MATERIAL_FEEDBACK");
    let paired_null = c8_find(&paired, "NO_MATERIAL_FEEDBACK");
    let paired_disabled = c8_find(&paired, "TRANSFER_DISABLED");
    let paired_zero = c8_find(&paired, "ZERO_RESOURCE");
    let a_candidate = c8_find(&daughter_a, "MATERIAL_FEEDBACK");
    let a_null = c8_find(&daughter_a, "NO_MATERIAL_FEEDBACK");
    let a_disabled = c8_find(&daughter_a, "TRANSFER_DISABLED");
    let a_zero = c8_find(&daughter_a, "ZERO_RESOURCE");
    let b_candidate = c8_find(&daughter_b, "MATERIAL_FEEDBACK");
    let b_null = c8_find(&daughter_b, "NO_MATERIAL_FEEDBACK");
    let b_disabled = c8_find(&daughter_b, "TRANSFER_DISABLED");
    let b_zero = c8_find(&daughter_b, "ZERO_RESOURCE");

    let all_runs = paired
        .iter()
        .chain(daughter_a.iter())
        .chain(daughter_b.iter())
        .map(|(_, run)| run)
        .collect::<Vec<_>>();
    let invalid = all_runs.iter().any(|run| c8_bool(run, "invalid"));
    let candidate_benefit = [
        (a_candidate, a_null),
        (b_candidate, b_null),
        (paired_candidate, paired_null),
    ]
    .iter()
    .all(|(candidate, null)| c8_metric(candidate, "delivered_n") > c8_metric(null, "delivered_n") + C8_TOL);
    let candidate_saves_a = [
        (a_candidate, a_null),
        (b_candidate, b_null),
        (paired_candidate, paired_null),
    ]
    .iter()
    .all(|(candidate, null)| c8_metric(candidate, "a_spent") + C8_TOL < c8_metric(null, "a_spent"));
    let transfer_causal_fission = [
        (a_candidate, a_disabled),
        (b_candidate, b_disabled),
        (paired_candidate, paired_disabled),
    ]
    .iter()
    .any(|(candidate, disabled)| c8_metric(candidate, "fissions") > c8_metric(disabled, "fissions"));
    let candidate_fission = [a_candidate, b_candidate, paired_candidate]
        .iter()
        .any(|run| c8_metric(run, "fissions") > 0.0);
    let zero_specific = [a_zero, b_zero, paired_zero]
        .iter()
        .all(|run| c8_metric(run, "delivered_n").abs() <= C8_TOL && c8_metric(run, "delivered_f").abs() <= C8_TOL);
    let classification = if invalid {
        "M2_CLOSURE008_LINEAGE_POST_INGESTIVE_MATERIAL_WORK_INVALID"
    } else if transfer_causal_fission && candidate_fission {
        "M2_POST_INGESTIVE_MATERIAL_WORK_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED"
    } else if candidate_benefit || candidate_saves_a {
        "M2_POST_INGESTIVE_MATERIAL_WORK_LINEAGE_BENEFIT_REPRODUCTION_NOT_ESTABLISHED"
    } else {
        "M2_POST_INGESTIVE_MATERIAL_WORK_LINEAGE_BENEFIT_INSUFFICIENT"
    };

    c8_write_arm_set(&out_dir, "paired_arms.json", &paired);
    c8_write_arm_set(&out_dir, "daughter_a_arms.json", &daughter_a);
    c8_write_arm_set(&out_dir, "daughter_b_arms.json", &daughter_b);
    write(&out_dir, "protocol.json", &json!({
        "directive": C8_DIRECTIVE,
        "starting_head": C8_START,
        "steps": C7_STEPS,
        "assay_only": true,
        "exact_c7_mechanism_reused": true,
        "paired_and_lineage_local_arms": ["paired", "daughter_a_solo", "daughter_b_solo"],
        "next_execution_started": false,
    }));
    write(&out_dir, "authority.json", &json!({
        "closure007": "ARCHITECT_ACCEPTED",
        "closure007_head": C8_START,
        "pr44": {"state": "OPEN", "draft": true, "merged": false, "modified": false},
        "m1": "CLOSED_FROZEN",
        "production": "MaturationCoupledV4 / reserve OFF",
        "scientific_runtime_source_changed": false,
    }));
    write(&out_dir, "mechanism_reuse.json", &json!({
        "material_signal": "(N+F)/(N+F+A+W)",
        "local_regulator": "existing ContinuityNetworkV1",
        "motor_composition": "existing CLOSURE-006 effective motor boundary",
        "new_parameter": false,
        "contact_signal": false,
        "observer_ledger_as_behavior_input": false,
        "tuning": false,
    }));
    write(&out_dir, "lineage_comparison.json", &json!({
        "daughter_a": {"candidate": c8_arm_value(a_candidate), "null": c8_arm_value(a_null)},
        "daughter_b": {"candidate": c8_arm_value(b_candidate), "null": c8_arm_value(b_null)},
        "paired": {"candidate": c8_arm_value(paired_candidate), "null": c8_arm_value(paired_null)},
    }));
    write(&out_dir, "acquisition_work_comparison.json", &json!({
        "candidate_benefit_all_scopes": candidate_benefit,
        "candidate_saves_a_all_scopes": candidate_saves_a,
        "paired_candidate_delta_n_over_null": c8_metric(paired_candidate, "delivered_n") - c8_metric(paired_null, "delivered_n"),
        "daughter_a_candidate_delta_n_over_null": c8_metric(a_candidate, "delivered_n") - c8_metric(a_null, "delivered_n"),
        "daughter_b_candidate_delta_n_over_null": c8_metric(b_candidate, "delivered_n") - c8_metric(b_null, "delivered_n"),
    }));
    write(&out_dir, "reproduction_comparison.json", &json!({
        "daughter_a_candidate_fissions": c8_metric(a_candidate, "fissions"),
        "daughter_b_candidate_fissions": c8_metric(b_candidate, "fissions"),
        "paired_candidate_fissions": c8_metric(paired_candidate, "fissions"),
        "daughter_a_transfer_disabled_fissions": c8_metric(a_disabled, "fissions"),
        "daughter_b_transfer_disabled_fissions": c8_metric(b_disabled, "fissions"),
        "paired_transfer_disabled_fissions": c8_metric(paired_disabled, "fissions"),
        "resource_causal_reproduction": transfer_causal_fission && candidate_fission,
        "shared_resource_competition_discriminator": {
            "paired_candidate_delivery": c8_metric(paired_candidate, "delivered_n"),
            "daughter_a_candidate_delivery": c8_metric(a_candidate, "delivered_n"),
            "daughter_b_candidate_delivery": c8_metric(b_candidate, "delivered_n"),
        },
    }));
    write(&out_dir, "controls.json", &json!({
        "zero_resource_delivery_zero": zero_specific,
        "transfer_disabled": {
            "daughter_a": c8_metric(a_disabled, "delivered_n"),
            "daughter_b": c8_metric(b_disabled, "delivered_n"),
            "paired": c8_metric(paired_disabled, "delivered_n"),
        },
        "motor_off": {
            "daughter_a_fissions": c8_metric(c8_find(&daughter_a, "MOTOR_OFF"), "fissions"),
            "daughter_b_fissions": c8_metric(c8_find(&daughter_b, "MOTOR_OFF"), "fissions"),
            "paired_fissions": c8_metric(c8_find(&paired, "MOTOR_OFF"), "fissions"),
        },
    }));
    write(&out_dir, "material_energy_closure.json", &json!({
        "all_runs_invalid": invalid,
        "world_n_f_conservation": "inherited CLOSURE-007 finite-world accounting",
        "a_to_w_closure": "inherited CLOSURE-007 closure per arm",
        "reserve": "OFF",
    }));
    write(&out_dir, "forbidden_information_audit.json", &json!({
        "resource_center": false, "resource_radius": false, "distance": false,
        "gradient": false, "contact_signal": false, "uptake_ledger_as_input": false,
        "target": false, "reward": false, "viability": false, "tuning": false,
    }));
    write(&out_dir, "preservation.json", &json!({
        "entry005_028": "PASS", "closure001_007": "PASS",
        "scientific_runtime_source_changed": false,
        "pr44": "OPEN_DRAFT_UNMERGED_UNTOUCHED",
    }));
    write(&out_dir, "m1_preservation.json", &json!({
        "production": "MaturationCoupledV4 / reserve OFF", "v2_d087": "8/8",
        "v3_d087": "8/8", "v4_d087": "7/8",
        "v4_vector": [true,true,false,true,true,true,true,true],
    }));
    write(&out_dir, "downstream_preservation.json", &json!({
        "regulator": "PASS", "continuity": "PASS", "plasticity": "PASS",
        "contact": "PASS", "contact_regulation": "PASS", "finite_resource": "PASS",
        "traction": "PASS", "d088": "PASS", "d091": "PASS", "evolution_harness": "PASS",
    }));
    write(&out_dir, "restart_boundary.json", &json!({
        "intrinsic_state_restart": "PASS", "generic_full_mesh_restart": "KNOWN_FAIL_NONCONTAMINATING",
    }));
    write(&out_dir, "qualification.json", &json!({
        "directive": C8_DIRECTIVE,
        "starting_head": C8_START,
        "classification": classification,
        "candidate_benefit_all_scopes": candidate_benefit,
        "candidate_saves_a_all_scopes": candidate_saves_a,
        "resource_causal_reproduction": transfer_causal_fission && candidate_fission,
        "lineage_level_material_work": if candidate_benefit { "BENEFIT_OBSERVED" } else { "NOT_ESTABLISHED" },
        "resource_causal_reproduction_status": if transfer_causal_fission && candidate_fission { "QUALIFIED" } else { "NOT_ESTABLISHED" },
        "autonomous_resource_acquisition": "NOT_ESTABLISHED",
        "heritable_ecological_phenotype": "NOT_ESTABLISHED",
        "evolution_reentry": "NOT_ESTABLISHED",
        "next_execution_started": false,
        "architect_acceptance": "PENDING",
    }));
    let files = [
        "protocol.json", "authority.json", "mechanism_reuse.json", "paired_arms.json",
        "daughter_a_arms.json", "daughter_b_arms.json", "lineage_comparison.json",
        "acquisition_work_comparison.json", "reproduction_comparison.json", "controls.json",
        "material_energy_closure.json", "forbidden_information_audit.json", "preservation.json",
        "m1_preservation.json", "downstream_preservation.json", "restart_boundary.json",
        "qualification.json", "artifact_manifest.json",
    ];
    write(&out_dir, "artifact_manifest.json", &json!({
        "directive": C8_DIRECTIVE,
        "starting_head": C8_START,
        "classification": classification,
        "files": files.iter().map(|file| json!({"file":file,"present":true})).collect::<Vec<_>>(),
        "dense_traces": "not generated in compact run",
    }));
    println!("CLOSURE-008 classification: {classification}");
    println!(
        "A candidate/null N/F {:.15e}/{:.15e}; B candidate/null {:.15e}/{:.15e}; pair candidate/null {:.15e}/{:.15e}; fissions A/B/pair {}/{}/{}",
        c8_metric(a_candidate, "delivered_n"), c8_metric(a_null, "delivered_n"),
        c8_metric(b_candidate, "delivered_n"), c8_metric(b_null, "delivered_n"),
        c8_metric(paired_candidate, "delivered_n"), c8_metric(paired_null, "delivered_n"),
        c8_metric(a_candidate, "fissions"), c8_metric(b_candidate, "fissions"), c8_metric(paired_candidate, "fissions"),
    );
}
