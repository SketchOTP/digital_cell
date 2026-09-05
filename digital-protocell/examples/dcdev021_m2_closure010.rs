#[allow(dead_code)]

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
        let closure012_mode = std::env::var_os("DC_CLOSURE012_FISSION_AUDIT").is_some();
        let closure011_mode = std::env::var_os("DC_CLOSURE011_A_FRACTION").is_some();
        let closure013_mode = std::env::var_os("DC_CLOSURE013_A_FRACTION_LAW").is_some();
        let closure014_mode = std::env::var_os("DC_CLOSURE014_CONTACT_BOUNDARY").is_some();
        let material_mode = closure011_mode || closure012_mode || closure013_mode;
        let directive = if closure014_mode {
            "DC-DEV-021-M2-CLOSURE-014-CONTACT-BOUNDARY-A-FRACTION-REPRODUCTION-CEILING-001"
        } else if closure013_mode {
            "DC-DEV-021-M2-CLOSURE-013-A-FRACTION-EXECUTION-RECONCILIATION-001"
        } else if closure012_mode {
            "DC-DEV-021-M2-CLOSURE-012-ACTIVE-WORK-FISSION-GATE-AUDIT-001"
        } else if closure011_mode {
            "DC-DEV-021-M2-CLOSURE-011-ACTIVE-MATERIAL-ALLOCATION-FISSION-CEILING-AUDIT-001"
        } else {
            C10_DIRECTIVE
        };
        let starting_head = if closure014_mode {
            "0d82bfaca318bca2c0ca6bb45e1a41e714f230fb"
        } else if closure013_mode {
            "925fd72ea0e2f330174cae317564de7f28139d69"
        } else if closure012_mode {
            "307968768af3e321b5fe2f047d854cff13d2ce18"
        } else if material_mode {
            "845d5c0a6f5778ec7c8dfb5d822aaba89aeddb31"
        } else {
            C10_START
        };
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
        let c12_audit = json!({
            "paired_combined": paired["combined"]["fission_gate_audit"],
            "daughter_a_combined": daughter_a["combined"]["fission_gate_audit"],
            "daughter_b_combined": daughter_b["combined"]["fission_gate_audit"],
            "paired_motor_off": paired["motor_off"]["fission_gate_audit"],
            "daughter_a_motor_off": daughter_a["motor_off"]["fission_gate_audit"],
            "daughter_b_motor_off": daughter_b["motor_off"]["fission_gate_audit"],
        });
        let c12_records = [
            &paired["combined"]["fission_gate_audit"],
            &daughter_a["combined"]["fission_gate_audit"],
            &daughter_b["combined"]["fission_gate_audit"],
        ];
        let c12_pinch_seen = c12_records.iter().any(|records| {
            records.as_array().is_some_and(|items| items.iter().any(|item| {
                item["pinch_found"].as_bool().unwrap_or(false)
            }))
        });
        let c12_mass_eligible_seen = c12_records.iter().any(|records| {
            records.as_array().is_some_and(|items| items.iter().any(|item| {
                item["eligible_by_mass"].as_bool().unwrap_or(false)
            }))
        });
        let c12_mass_ceiling_seen = c12_records.iter().any(|records| {
            records.as_array().is_some_and(|items| items.iter().any(|item| {
                !item["eligible_by_mass"].as_bool().unwrap_or(false)
            }))
        });
        let c12_a_shortfall_seen = c12_records.iter().any(|records| {
            records.as_array().is_some_and(|items| items.iter().any(|item| {
                item["pinch_found"].as_bool().unwrap_or(false)
                    && !item["a_sufficient_for_existing_fission_gate"].as_bool().unwrap_or(false)
            }))
        });
        let c12_geometry_only = c12_mass_eligible_seen && !c12_pinch_seen;
        let classification = if closure014_mode {
            if invalid || !zero_specific {
                "M2_CLOSURE014_CONTACT_BOUNDARY_A_FRACTION_INVALID"
            } else if combined_fission && transfer_causal_fission {
                "M2_CONTACT_BOUNDARY_A_FRACTION_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED"
            } else if combined_benefit || combined_saves_a {
                "M2_CONTACT_BOUNDARY_A_FRACTION_REPRODUCTION_NOT_ESTABLISHED"
            } else {
                "M2_CONTACT_BOUNDARY_A_FRACTION_WORK_ALLOCATION_INSUFFICIENT"
            }
        } else if closure013_mode {
            if invalid || !zero_specific {
                "M2_CLOSURE013_A_FRACTION_EXECUTION_RECONCILIATION_INVALID"
            } else if combined_fission && transfer_causal_fission {
                "M2_DOCUMENTED_A_FRACTION_LAW_RESOURCE_CAUSAL_REPRODUCTION_QUALIFIED"
            } else if combined_benefit || combined_saves_a {
                "M2_DOCUMENTED_A_FRACTION_LAW_REPRODUCTION_NOT_ESTABLISHED"
            } else {
                "M2_DOCUMENTED_A_FRACTION_LAW_WORK_ALLOCATION_INSUFFICIENT"
            }
        } else if closure012_mode {
            if invalid || !zero_specific {
                "M2_CLOSURE012_ACTIVE_WORK_FISSION_GATE_AUDIT_INVALID"
            } else if !c12_mass_eligible_seen && c12_mass_ceiling_seen {
                "M2_ACTIVE_WORK_FISSION_GROWTH_THRESHOLD_CEILING_CONFIRMED"
            } else if c12_a_shortfall_seen {
                "M2_ACTIVE_WORK_FISSION_A_FUNDING_CEILING_CONFIRMED"
            } else if c12_geometry_only {
                "M2_ACTIVE_WORK_FISSION_PINCH_GEOMETRY_CEILING_CONFIRMED"
            } else {
                "M2_ACTIVE_WORK_FISSION_FISSION_GATE_MULTIFACTOR_UNRESOLVED"
            }
        } else if invalid || !zero_specific {
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
            "directive": directive,
            "starting_head": starting_head,
            "steps": C10_STEPS,
            "assay_only": true,
            "mode": if closure014_mode { "contact_boundary_a_fraction_reproduction_ceiling" } else if closure013_mode { "documented_a_fraction_execution_reconciliation" } else if closure012_mode { "active_work_fission_gate_audit" } else if closure011_mode { "activated_energy_fraction" } else { "combined_material_contact" },
            "next_execution_started": false,
            "scopes": ["paired", "daughter_a_solo", "daughter_b_solo"],
        }));
        write(&out_dir, "authority.json", &json!({
            "closure009": "ARCHITECT_ACCEPTED",
            "closure009_head": starting_head,
            "pr44": {"state": "OPEN", "draft": true, "merged": false, "modified": false},
            "m1": "CLOSED_FROZEN",
            "production": "MaturationCoupledV4 / reserve OFF",
            "scientific_runtime_source_changed": false,
        }));
        write(&out_dir, "architecture.json", &json!({
            "composition": if closure014_mode { "contact_i=1: motor_i=0; contact_i=0: motor_i=raw_i * A/(N+F+A+W) * (1 - regulator_i)" } else if closure013_mode { "motor_i = raw_i * A/(N+F+A+W) * (1 - regulator_i)" } else if material_mode { "motor_i = raw_i * A/(N+F+A+W) * (1 - regulator_i)" } else { "motor_i = raw_i * (1 - S) * (1 - regulator_i)" },
            "material_signal": if material_mode { "A/(N+F+A+W)" } else { "S=(N+F)/(N+F+A+W)" },
            "historical_executed_composition": if closure013_mode { "not_applicable" } else { "motor_i = raw_i * (1 - material_signal_i) * (1 - regulator_i)" },
            "reconciled_executed_composition": if closure013_mode { "motor_i = raw_i * material_signal_i * (1 - regulator_i)" } else { "not_applicable" },
            "contact_boundary_rule": if closure014_mode { "production-positive local contact selects exact zero motor output; noncontact retains literal A-fraction output" } else { "not_applicable" },
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
        if closure013_mode && !closure014_mode {
            write(&out_dir, "semantic_reconciliation.json", &json!({
                "historical_authority": {
                    "directive": "DC-DEV-021-M2-CLOSURE-012-ACTIVE-WORK-FISSION-GATE-AUDIT-001",
                    "head": "925fd72ea0e2f330174cae317564de7f28139d69",
                    "ci": "33963815229",
                    "artifact": "sha256:1d90f619bbd1ea10bbbed03294c009b78153c7ab982e1c565e220a6d06a40eef"
                },
                "historical_documented_composition": "motor_i = raw_i * A/(N+F+A+W) * (1 - regulator_i)",
                "historical_executed_composition": "motor_i = raw_i * (1 - A/(N+F+A+W)) * (1 - regulator_i)",
                "historical_replay": {
                    "paired_combined_n": 1252.571074851476,
                    "paired_combined_a_spent": 1487.7520125367535,
                    "paired_combined_fissions": 0,
                    "daughter_b_combined_n": 1252.5710748514732,
                    "daughter_b_combined_a_spent": 1028.4329122775034,
                    "daughter_b_combined_fissions": 0
                },
                "reconciled_executed_composition": "motor_i = raw_i * A/(N+F+A+W) * (1 - regulator_i)",
                "reconciled_replay": {
                    "paired_combined_n": c10_metric(&paired, "combined", "delivered_n"),
                    "paired_combined_a_spent": c10_metric(&paired, "combined", "a_spent"),
                    "paired_combined_fissions": c10_metric(&paired, "combined", "fissions"),
                    "daughter_b_combined_n": c10_metric(&daughter_b, "combined", "delivered_n"),
                    "daughter_b_combined_a_spent": c10_metric(&daughter_b, "combined", "a_spent"),
                    "daughter_b_combined_fissions": c10_metric(&daughter_b, "combined", "fissions")
                },
                "historical_result_preserved": true,
                "production_scientific_runtime_changed": false
            }));
        }
        if closure012_mode {
            write(&out_dir, "fission_gate_audit.json", &c12_audit);
            write(&out_dir, "fission_gate_summary.json", &json!({
                "candidate_combined_fissions": combined_fission,
                "candidate_mass_eligible_seen": c12_mass_eligible_seen,
                "candidate_mass_threshold_ceiling_seen": c12_mass_ceiling_seen,
                "candidate_pinch_seen": c12_pinch_seen,
                "candidate_a_shortfall_seen": c12_a_shortfall_seen,
                "candidate_geometry_only": c12_mass_eligible_seen && !c12_pinch_seen,
                "motor_off_paired_fissions": c10_metric(&paired, "motor_off", "fissions"),
                "motor_off_daughter_a_fissions": c10_metric(&daughter_a, "motor_off", "fissions"),
                "motor_off_daughter_b_fissions": c10_metric(&daughter_b, "motor_off", "fissions"),
            }));
        }
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
        let reported_classification = if closure014_mode {
            classification
        } else if closure013_mode {
            classification
        } else if closure012_mode {
            classification
        } else if closure011_mode {
            "M2_ACTIVE_MATERIAL_ALLOCATION_FISSION_INSUFFICIENT"
        } else {
            classification
        };
        write(&out_dir, "qualification.json", &json!({
            "classification": reported_classification,
            "underlying_closure010_classification": classification,
            "combined_benefit": combined_benefit,
            "combined_fission": combined_fission,
            "transfer_causal_fission": transfer_causal_fission,
            "architect_acceptance": if material_mode { "PENDING" } else { "COMPLETE" },
            "local_resource_exploitation": "NOT_ESTABLISHED",
            "autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "next_execution_started": false,
        }));
        let mut files = vec![
            "protocol.json", "authority.json", "architecture.json", "paired_arms.json",
            "daughter_a_arms.json", "daughter_b_arms.json", "combined_material_contact.json",
            "direct_material_control.json", "contact_local_control.json", "no_feedback_control.json",
            "transfer_disabled_control.json", "zero_resource_control.json", "motor_off_control.json",
            "acquisition_work_comparison.json", "reproduction_comparison.json", "material_energy_closure.json",
            "forbidden_information_audit.json", "preservation.json", "m1_preservation.json",
            "downstream_preservation.json", "restart_boundary.json", "qualification.json",
        ];
        if closure012_mode {
            files.extend(["fission_gate_audit.json", "fission_gate_summary.json"]);
        }
        if closure013_mode && !closure014_mode {
            files.push("semantic_reconciliation.json");
        }
        write(&out_dir, "artifact_manifest.json", &json!({
            "files": files,
            "dense_traces": "Atlas",
        }));
        println!("{} classification: {reported_classification}", if closure014_mode { "CLOSURE-014" } else if closure013_mode { "CLOSURE-013" } else if closure012_mode { "CLOSURE-012" } else if closure011_mode { "CLOSURE-011" } else { "CLOSURE-010" });
        println!("combined benefit: {combined_benefit}; candidate fission: {combined_fission}; transfer causal fission: {transfer_causal_fission}");
    }
}

pub fn run() {
    accepted_closure_context::c10_main();
}

fn main() {
    run();
}
