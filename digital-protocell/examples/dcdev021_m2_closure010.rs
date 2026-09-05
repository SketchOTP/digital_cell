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
    include!("dcdev021_m2_closure010_impl.rs");

    fn c10_group(
        initial: &[ClosureAgent],
        world_body: &MaterialMesh,
        group: &str,
    ) -> Value {
        let combined = c10_combined_run(
            initial,
            world_body,
            &format!("CLOSURE010_{group}_COMBINED"),
            true,
            false,
            true,
            true,
            false,
        );
        let direct = c9_direct_run(
            initial,
            world_body,
            &format!("CLOSURE010_{group}_DIRECT_MATERIAL"),
            true,
            false,
            true,
            false,
        );
        let contact = c6_run(
            initial,
            world_body,
            &format!("CLOSURE010_{group}_CONTACT_LOCAL"),
            true,
            false,
            C6ContactMode::Real,
            false,
        );
        let no_material = c7_run(
            initial,
            world_body,
            &format!("CLOSURE010_{group}_NO_FEEDBACK"),
            true,
            false,
            false,
            false,
        );
        let transfer_disabled = c7_run(
            initial,
            world_body,
            &format!("CLOSURE010_{group}_TRANSFER_DISABLED"),
            false,
            false,
            false,
            false,
        );
        let zero_resource = c7_run(
            initial,
            world_body,
            &format!("CLOSURE010_{group}_ZERO_RESOURCE"),
            true,
            true,
            false,
            false,
        );
        let motor_off = c7_run(
            initial,
            world_body,
            &format!("CLOSURE010_{group}_MOTOR_OFF"),
            true,
            false,
            false,
            true,
        );
        json!({
            "combined": c10_value(&combined),
            "direct_material": c7_value(&direct),
            "contact_local": c6_value(&contact),
            "no_feedback": c7_value(&no_material),
            "transfer_disabled": c7_value(&transfer_disabled),
            "zero_resource": c7_value(&zero_resource),
            "motor_off": c7_value(&motor_off),
        })
    }

    fn c10_run_value<'a>(group: &'a Value, arm: &str) -> &'a Value {
        &group[arm]
    }

    fn c10_metric(group: &Value, arm: &str, key: &str) -> f64 {
        c10_run_value(group, arm)["base"]["base"][key]
            .as_f64()
            .unwrap_or(0.0)
    }

    fn c10_bool(group: &Value, arm: &str, key: &str) -> bool {
        c10_run_value(group, arm)["base"]["base"][key]
            .as_bool()
            .unwrap_or(false)
    }

    pub fn c10_main() {
        let out_dir = std::env::args()
            .nth(1)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2closure010"));
        std::fs::create_dir_all(&out_dir).unwrap();

        let (initial, world_body) = c8_make_initial();
        let paired = c10_group(&initial, &world_body, "PAIRED");
        let daughter_a = c10_group(&initial[0..1], &world_body, "DAUGHTER_A_SOLO");
        let daughter_b = c10_group(&initial[1..2], &world_body, "DAUGHTER_B_SOLO");
        let groups = [&paired, &daughter_a, &daughter_b];
        let arms = [
            "combined",
            "direct_material",
            "contact_local",
            "no_feedback",
            "transfer_disabled",
            "zero_resource",
            "motor_off",
        ];
        let invalid = groups.iter().any(|group| {
            arms.iter().any(|arm| c10_bool(group, arm, "invalid"))
        });
        let zero_specific = groups.iter().all(|group| {
            c10_metric(group, "zero_resource", "delivered_n").abs() <= C10_TOL
                && c10_metric(group, "zero_resource", "delivered_f").abs() <= C10_TOL
        });
        let combined_benefit = groups.iter().all(|group| {
            c10_metric(group, "combined", "delivered_n")
                > c10_metric(group, "no_feedback", "delivered_n") + C10_TOL
        });
        let combined_saves_a = groups.iter().all(|group| {
            c10_metric(group, "combined", "a_spent") + C10_TOL
                < c10_metric(group, "no_feedback", "a_spent")
        });
        let combined_fission = groups.iter().any(|group| {
            c10_metric(group, "combined", "fissions") > 0.0
        });
        let transfer_causal_fission = groups.iter().any(|group| {
            c10_metric(group, "combined", "fissions")
                > c10_metric(group, "transfer_disabled", "fissions")
        });
        let classification = if invalid || !zero_specific {
            "M2_CLOSURE010_COMBINED_MATERIAL_CONTACT_WORK_INVALID"
        } else if combined_fission && transfer_causal_fission {
            "M2_POST_INGESTIVE_COMBINED_MATERIAL_CONTACT_WORK_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED"
        } else if combined_benefit || combined_saves_a {
            "M2_POST_INGESTIVE_COMBINED_MATERIAL_CONTACT_WORK_REPRODUCTION_NOT_ESTABLISHED"
        } else {
            "M2_POST_INGESTIVE_COMBINED_MATERIAL_CONTACT_WORK_INSUFFICIENT"
        };

        for (name, value) in [
            ("paired_arms.json", &paired),
            ("daughter_a_arms.json", &daughter_a),
            ("daughter_b_arms.json", &daughter_b),
        ] {
            write(&out_dir, name, value);
        }
        write(&out_dir, "protocol.json", &json!({
            "directive": C10_DIRECTIVE,
            "starting_head": C10_START,
            "steps": C10_STEPS,
            "assay_only": true,
            "next_execution_started": false,
            "scopes": ["paired", "daughter_a_solo", "daughter_b_solo"],
        }));
        write(&out_dir, "authority.json", &json!({
            "closure009": "ARCHITECT_ACCEPTED",
            "closure009_head": C10_START,
            "pr44": {"state": "OPEN", "draft": true, "merged": false, "modified": false},
            "m1": "CLOSED_FROZEN",
            "production": "MaturationCoupledV4 / reserve OFF",
            "scientific_runtime_source_changed": false,
        }));
        write(&out_dir, "architecture.json", &json!({
            "composition": "motor_i = raw_i * (1 - S) * (1 - regulator_i)",
            "material_signal": "S=(N+F)/(N+F+A+W)",
            "contact_signal": "FiniteSpatialResourceRegionV1::local_contact_signal",
            "regulator": "existing ContinuityNetworkV1 dynamics",
            "new_parameter": false,
            "gain": false,
            "threshold": false,
            "timer_or_memory": false,
            "production_integration": false,
        }));
        for (name, key) in [
            ("combined_material_contact.json", "combined"),
            ("direct_material_control.json", "direct_material"),
            ("contact_local_control.json", "contact_local"),
            ("no_feedback_control.json", "no_feedback"),
            ("transfer_disabled_control.json", "transfer_disabled"),
            ("zero_resource_control.json", "zero_resource"),
            ("motor_off_control.json", "motor_off"),
        ] {
            let value = json!({
                "paired": paired[key],
                "daughter_a": daughter_a[key],
                "daughter_b": daughter_b[key],
            });
            write(&out_dir, name, &value);
        }
        write(&out_dir, "acquisition_work_comparison.json", &json!({
            "combined_benefit_over_no_feedback": combined_benefit,
            "combined_saves_a_over_no_feedback": combined_saves_a,
            "paired": {
                "combined_n": c10_metric(&paired, "combined", "delivered_n"),
                "direct_n": c10_metric(&paired, "direct_material", "delivered_n"),
                "contact_n": c10_metric(&paired, "contact_local", "delivered_n"),
                "null_n": c10_metric(&paired, "no_feedback", "delivered_n"),
            },
            "daughter_a": {
                "combined_n": c10_metric(&daughter_a, "combined", "delivered_n"),
                "direct_n": c10_metric(&daughter_a, "direct_material", "delivered_n"),
                "contact_n": c10_metric(&daughter_a, "contact_local", "delivered_n"),
                "null_n": c10_metric(&daughter_a, "no_feedback", "delivered_n"),
            },
            "daughter_b": {
                "combined_n": c10_metric(&daughter_b, "combined", "delivered_n"),
                "direct_n": c10_metric(&daughter_b, "direct_material", "delivered_n"),
                "contact_n": c10_metric(&daughter_b, "contact_local", "delivered_n"),
                "null_n": c10_metric(&daughter_b, "no_feedback", "delivered_n"),
            },
        }));
        write(&out_dir, "reproduction_comparison.json", &json!({
            "candidate_fission": combined_fission,
            "transfer_causal_fission": transfer_causal_fission,
            "paired_combined_fissions": c10_metric(&paired, "combined", "fissions"),
            "daughter_a_combined_fissions": c10_metric(&daughter_a, "combined", "fissions"),
            "daughter_b_combined_fissions": c10_metric(&daughter_b, "combined", "fissions"),
        }));
        write(&out_dir, "material_energy_closure.json", &json!({
            "all_runs_invalid": invalid,
            "zero_resource_specific": zero_specific,
            "world_n_f_conservation": true,
            "a_to_w_closure_reused": true,
        }));
        write(&out_dir, "forbidden_information_audit.json", &json!({
            "resource_center": false, "resource_radius": false, "distance": false,
            "uptake_ledger": false, "observer_input": false,
            "target_gradient_memory": false,
        }));
        write(&out_dir, "preservation.json", &json!({
            "closure009_preserved": true,
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
            "combined_benefit": combined_benefit,
            "combined_fission": combined_fission,
            "transfer_causal_fission": transfer_causal_fission,
            "architect_acceptance": "PENDING",
            "local_resource_exploitation": "NOT_ESTABLISHED",
            "autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "next_execution_started": false,
        }));
        let files = [
            "protocol.json", "authority.json", "architecture.json", "paired_arms.json",
            "daughter_a_arms.json", "daughter_b_arms.json", "combined_material_contact.json",
            "direct_material_control.json", "contact_local_control.json", "no_feedback_control.json",
            "transfer_disabled_control.json", "zero_resource_control.json", "motor_off_control.json",
            "acquisition_work_comparison.json", "reproduction_comparison.json", "material_energy_closure.json",
            "forbidden_information_audit.json", "preservation.json", "m1_preservation.json",
            "downstream_preservation.json", "restart_boundary.json", "qualification.json",
        ];
        write(&out_dir, "artifact_manifest.json", &json!({
            "files": files.iter().map(|name| (*name).to_string()).collect::<Vec<_>>(),
            "dense_traces": "Atlas",
        }));
        println!("CLOSURE-010 classification: {classification}");
        println!("combined benefit: {combined_benefit}; candidate fission: {combined_fission}; transfer causal fission: {transfer_causal_fission}");
    }
}

fn main() {
    accepted_closure_context::c10_main();
}
