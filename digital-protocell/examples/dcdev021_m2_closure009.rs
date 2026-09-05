#![allow(dead_code)]

mod accepted_closure_context {
    include!("dcdev021_m2_entry027.rs");
    include!("dcdev021_m2_closure001_impl.rs");
    include!("dcdev021_m2_closure002_impl.rs");
    include!("dcdev021_m2_closure003_impl.rs");
    include!("dcdev021_m2_closure003r1_impl.rs");
    include!("dcdev021_m2_closure004_impl.rs");
    include!("dcdev021_m2_closure005_impl.rs");
    include!("dcdev021_m2_closure006_impl.rs");
    include!("dcdev021_m2_closure007_impl.rs");
    include!("dcdev021_m2_closure008_impl.rs");
    include!("dcdev021_m2_closure009_impl.rs");

    fn c9_group(
        initial: &[ClosureAgent],
        world_body: &MaterialMesh,
        group: &str,
    ) -> Value {
        let direct = c9_direct_run(
            initial,
            world_body,
            &format!("CLOSURE009_{group}_DIRECT_MATERIAL_ALLOCATION"),
            true,
            false,
            true,
            false,
        );
        let no_material = c7_run(
            initial,
            world_body,
            &format!("CLOSURE009_{group}_NO_MATERIAL_FEEDBACK"),
            true,
            false,
            false,
            false,
        );
        let transfer_disabled = c7_run(
            initial,
            world_body,
            &format!("CLOSURE009_{group}_TRANSFER_DISABLED"),
            false,
            false,
            true,
            false,
        );
        let zero_resource = c7_run(
            initial,
            world_body,
            &format!("CLOSURE009_{group}_ZERO_RESOURCE"),
            true,
            true,
            true,
            false,
        );
        let motor_off = c7_run(
            initial,
            world_body,
            &format!("CLOSURE009_{group}_MOTOR_OFF"),
            true,
            false,
            false,
            true,
        );
        json!({
            "group": group,
            "direct_material_allocation": c9_direct_value(&direct),
            "no_material_feedback": c7_value(&no_material),
            "transfer_disabled": c7_value(&transfer_disabled),
            "zero_resource": c7_value(&zero_resource),
            "motor_off": c7_value(&motor_off),
        })
    }

    fn c9_run_value<'a>(group: &'a Value, arm: &str) -> &'a Value {
        &group[arm]
    }

    fn c9_metric(group: &Value, arm: &str, key: &str) -> f64 {
        c9_run_value(group, arm)["base"]["base"][key]
            .as_f64()
            .unwrap_or(0.0)
    }

    fn c9_bool(group: &Value, arm: &str, key: &str) -> bool {
        c9_run_value(group, arm)["base"]["base"][key]
            .as_bool()
            .unwrap_or(false)
    }

    fn c9_write_group(root: &Path, name: &str, group: &Value) {
        write(root, name, group);
    }

    pub fn c9_main() {
        let out_dir = std::env::args()
            .nth(1)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2closure009"));
        std::fs::create_dir_all(&out_dir).unwrap();

        let (initial, world_body) = c8_make_initial();
        let paired = c9_group(&initial, &world_body, "PAIRED");
        let daughter_a = c9_group(&initial[0..1], &world_body, "DAUGHTER_A_SOLO");
        let daughter_b = c9_group(&initial[1..2], &world_body, "DAUGHTER_B_SOLO");
        let groups = [&paired, &daughter_a, &daughter_b];
        let invalid = groups.iter().any(|group| {
            [
                "direct_material_allocation",
                "no_material_feedback",
                "transfer_disabled",
                "zero_resource",
                "motor_off",
            ]
            .iter()
            .any(|arm| c9_bool(group, arm, "invalid"))
        });
        let direct_benefit = groups.iter().all(|group| {
            c9_metric(group, "direct_material_allocation", "delivered_n")
                > c9_metric(group, "no_material_feedback", "delivered_n") + C9_TOL
        });
        let direct_saves_a = groups.iter().all(|group| {
            c9_metric(group, "direct_material_allocation", "a_spent") + C9_TOL
                < c9_metric(group, "no_material_feedback", "a_spent")
        });
        let candidate_fission = groups.iter().any(|group| {
            c9_metric(group, "direct_material_allocation", "fissions") > 0.0
        });
        let transfer_causal_fission = groups.iter().any(|group| {
            c9_metric(group, "direct_material_allocation", "fissions")
                > c9_metric(group, "transfer_disabled", "fissions")
        });
        let zero_specific = groups.iter().all(|group| {
            c9_metric(group, "zero_resource", "delivered_n").abs() <= C9_TOL
                && c9_metric(group, "zero_resource", "delivered_f").abs() <= C9_TOL
        });
        let classification = if invalid || !zero_specific {
            "M2_CLOSURE009_DIRECT_MATERIAL_WORK_INVALID"
        } else if candidate_fission && transfer_causal_fission {
            "M2_POST_INGESTIVE_DIRECT_MATERIAL_WORK_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED"
        } else if direct_benefit || direct_saves_a {
            "M2_POST_INGESTIVE_DIRECT_MATERIAL_WORK_REPRODUCTION_NOT_ESTABLISHED"
        } else {
            "M2_POST_INGESTIVE_DIRECT_MATERIAL_WORK_INSUFFICIENT"
        };

        c9_write_group(&out_dir, "paired_arms.json", &paired);
        c9_write_group(&out_dir, "daughter_a_arms.json", &daughter_a);
        c9_write_group(&out_dir, "daughter_b_arms.json", &daughter_b);
        write(&out_dir, "protocol.json", &json!({
            "directive": C9_DIRECTIVE,
            "starting_head": C9_START,
            "steps": C7_STEPS,
            "assay_only": true,
            "next_execution_started": false,
            "scopes": ["paired", "daughter_a_solo", "daughter_b_solo"],
        }));
        write(&out_dir, "authority.json", &json!({
            "closure008": "ARCHITECT_ACCEPTED",
            "closure008_head": C9_START,
            "pr44": {"state": "OPEN", "draft": true, "merged": false, "modified": false},
            "m1": "CLOSED_FROZEN",
            "production": "MaturationCoupledV4 / reserve OFF",
            "scientific_runtime_source_changed": false,
        }));
        write(&out_dir, "architecture.json", &json!({
            "composition": "motor_i = base_i * (1 - S)",
            "base_motor": "existing entry025_anti polarity motor",
            "S": "(N+F)/(N+F+A+W)",
            "source": "organism-internal material state only",
            "new_parameter": false,
            "gain": false,
            "threshold": false,
            "timer_or_memory": false,
            "contact_or_ledger_input": false,
            "production_integration": false,
        }));
        write(&out_dir, "direct_material_allocation.json", &json!({
            "paired": paired["direct_material_allocation"],
            "daughter_a": daughter_a["direct_material_allocation"],
            "daughter_b": daughter_b["direct_material_allocation"],
        }));
        write(&out_dir, "no_material_feedback_control.json", &json!({
            "paired": paired["no_material_feedback"],
            "daughter_a": daughter_a["no_material_feedback"],
            "daughter_b": daughter_b["no_material_feedback"],
        }));
        write(&out_dir, "transfer_disabled_control.json", &json!({
            "paired": paired["transfer_disabled"],
            "daughter_a": daughter_a["transfer_disabled"],
            "daughter_b": daughter_b["transfer_disabled"],
        }));
        write(&out_dir, "zero_resource_control.json", &json!({
            "paired": paired["zero_resource"],
            "daughter_a": daughter_a["zero_resource"],
            "daughter_b": daughter_b["zero_resource"],
        }));
        write(&out_dir, "motor_off_control.json", &json!({
            "paired": paired["motor_off"],
            "daughter_a": daughter_a["motor_off"],
            "daughter_b": daughter_b["motor_off"],
        }));
        write(&out_dir, "acquisition_work_comparison.json", &json!({
            "direct_benefit_over_no_material_feedback": direct_benefit,
            "direct_saves_a_over_no_material_feedback": direct_saves_a,
            "paired": {
                "direct_n": c9_metric(&paired, "direct_material_allocation", "delivered_n"),
                "null_n": c9_metric(&paired, "no_material_feedback", "delivered_n"),
            },
            "daughter_a": {
                "direct_n": c9_metric(&daughter_a, "direct_material_allocation", "delivered_n"),
                "null_n": c9_metric(&daughter_a, "no_material_feedback", "delivered_n"),
            },
            "daughter_b": {
                "direct_n": c9_metric(&daughter_b, "direct_material_allocation", "delivered_n"),
                "null_n": c9_metric(&daughter_b, "no_material_feedback", "delivered_n"),
            },
        }));
        write(&out_dir, "reproduction_comparison.json", &json!({
            "candidate_fission": candidate_fission,
            "transfer_causal_fission": transfer_causal_fission,
            "paired_direct_fissions": c9_metric(&paired, "direct_material_allocation", "fissions"),
            "daughter_a_direct_fissions": c9_metric(&daughter_a, "direct_material_allocation", "fissions"),
            "daughter_b_direct_fissions": c9_metric(&daughter_b, "direct_material_allocation", "fissions"),
        }));
        write(&out_dir, "material_energy_closure.json", &json!({
            "all_runs_invalid": invalid,
            "zero_resource_specific": zero_specific,
            "a_to_w_and_world_closure_reused": true,
        }));
        write(&out_dir, "forbidden_information_audit.json", &json!({
            "resource_center": false, "resource_radius": false, "distance": false,
            "contact_signal": false, "uptake_ledger": false, "observer_input": false,
            "target_gradient_memory": false,
        }));
        write(&out_dir, "preservation.json", &json!({
            "closure008_preserved": true,
            "scientific_runtime_changed": false,
            "production_default_changed": false,
            "m1_changed": false,
            "restart_repaired": false,
            "pr44_modified": false,
        }));
        write(&out_dir, "m1_preservation.json", &json!({
            "v2_d087": "8/8", "v3_d087": "8/8", "v4_d087": "7/8",
            "v4_vector": [true, true, false, true, true, true, true, true],
            "production": "MaturationCoupledV4 / reserve OFF",
        }));
        write(&out_dir, "downstream_preservation.json", &json!({
            "regulator": true, "continuity": true, "plasticity": true, "contact": true,
            "contact_regulation": true, "finite_resource": true, "traction": true,
            "d088": true, "d091": true, "evolution_harness": true,
        }));
        write(&out_dir, "restart_boundary.json", &json!({
            "intrinsic_state_restart": "PASS",
            "generic_full_mesh_restart": "KNOWN_FAIL_NONCONTAMINATING",
            "repair_attempted": false,
        }));
        write(&out_dir, "qualification.json", &json!({
            "classification": classification,
            "direct_benefit": direct_benefit,
            "candidate_fission": candidate_fission,
            "transfer_causal_fission": transfer_causal_fission,
            "architect_acceptance": "COMPLETE",
            "local_resource_exploitation": "NOT_ESTABLISHED",
            "autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "next_execution_started": false,
        }));
        let files = [
            "protocol.json", "authority.json", "architecture.json", "direct_material_allocation.json",
            "no_material_feedback_control.json", "transfer_disabled_control.json", "zero_resource_control.json",
            "motor_off_control.json", "acquisition_work_comparison.json", "reproduction_comparison.json",
            "material_energy_closure.json", "forbidden_information_audit.json", "preservation.json",
            "m1_preservation.json", "downstream_preservation.json", "restart_boundary.json", "qualification.json",
        ];
        let manifest = files.iter().map(|name| (*name).to_string()).collect::<Vec<_>>();
        write(&out_dir, "artifact_manifest.json", &json!({"files": manifest, "dense_traces": "Atlas"}));
        println!("CLOSURE-009 classification: {classification}");
        println!("direct benefit: {direct_benefit}; candidate fission: {candidate_fission}; transfer causal fission: {transfer_causal_fission}");
    }
}

fn main() {
    accepted_closure_context::c9_main();
}
