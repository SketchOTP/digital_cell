//! DC-DEV-020-R8-R4 observer-only shared-affinity catalyst-production audit.
//!
//! This example reuses the sealed R8-R3 observer machinery and adds one
//! counterfactual production law. It never changes production chemistry.

mod r8r3 {
    include!("dcdev020r8r3_shared_affinity_helper.rs");

    use super::*;

    const ENTRY: &str = "c9b200ee24b88c542eeb0c14038867f4c7fbb466";
    const CLEAN_BASE_R4: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
    const R5_SHA: &str = "4e22ab1dbd6e06f7c9a272747c2ed8271f28ef33f4eaddc1c59bb9df58a46585";
    const R7_SHA: &str = "abdaea6d075c700e36d14d369dba62982f4a65cea47d2d1f162b5dfe8afa59f8";
    const R8_SHA: &str = "12b41f27c928635899a7ea3a8d496cfdd3af7d3fd83aaa93024724663e2df9ff";
    const R8R1_SHA: &str = "f44e8f9fa441451ee40bcbfccac5f556131e4d26868868607e9507c29e7bcf90";
    const R8R2_SHA: &str = "e932f6ab96e34516de98c97c2cae102553db9764383af3d61abf015743c3a376";
    const R8R3_SHA: &str = "c0850577954996e1ee566f936d8965ab61647281dc67eab401c7c47be0670c12";
    const R6_K_PL: f64 = 0.017556661171593057;
    const R6_P: f64 = 0.0003277429681759396;
    const PATCH: f64 = 19.878372106390554;
    const E_DEPRIVED_R4: f64 = 60.82781514212436;
    const E_TARGET_R4: f64 = 77.91027880846893;
    const MASS_TOL_R4: f64 = 1e-10;
    const SOURCE_EPS_R4: f64 = 1e-12;

    #[derive(Clone, Copy, Debug, Serialize)]
    enum ProductionMode {
        Current,
        None,
        SharedAffinity,
    }

    impl ProductionMode {
        fn name(self) -> &'static str {
            match self {
                Self::Current => "current_k_c_prod_A",
                Self::None => "zero_cprod_reference",
                Self::SharedAffinity => "shared_affinity_k_c_prod_A_one_minus_qc",
            }
        }
    }

    #[derive(Clone, Debug, Serialize)]
    struct Arm {
        context: String,
        mode: String,
        dose_scale: f64,
        steps: usize,
        initial: Snap,
        final_state: Snap,
        alive: bool,
        finite: bool,
        max_resource_error: f64,
        max_accounting_residual: f64,
        capacity_violations: usize,
        clipping_steps: usize,
        accelerated_decay_steps: usize,
        settled_distance_initial: f64,
        settled_distance_final: f64,
        a_toward_settled: bool,
        r_toward_settled: bool,
        trajectory_hash: String,
    }

    #[derive(Clone, Debug, Serialize)]
    struct Sustained {
        context: String,
        mode: String,
        steps: usize,
        initial: Snap,
        final_state: Snap,
        final_quarter_min: f64,
        final_quarter_max: f64,
        final_quarter_slope: f64,
        peak_e_ar: f64,
        clipping_steps: usize,
        accelerated_decay_steps: usize,
        alive: bool,
        finite: bool,
        max_accounting_residual: f64,
        trajectory_hash: String,
    }

    fn r6() -> SourceLaw {
        SourceLaw::PowerLaw(PowerLaw {
            k_pl: R6_K_PL,
            p: R6_P,
            g_h: 1.0,
        })
    }

    fn candidate_rate(params: &ReactionParams, a: f64, c: f64) -> f64 {
        let q = q_catalyst(c, params.q_c);
        params.k_c_prod * a.max(0.0) * (1.0 - q)
    }

    fn apply_counterfactual(
        mesh: &mut MaterialMesh,
        params: &ReactionParams,
        law: SourceLaw,
        mode: ProductionMode,
    ) -> SourceStep {
        let before = mesh.interior;
        let area = mesh.area().max(1e-6);
        let before_e = area * (before.a + before.r).max(0.0);
        let capacity = (before.n.max(0.0) * area).min(before.f.max(0.0) * area);
        let requested = requested(mesh, params, law);
        assert!(requested.is_finite() && requested >= 0.0);
        let ordinary = ordinary_requested(mesh, params);
        let gain = if requested <= SOURCE_EPS_R4 {
            0.0
        } else {
            requested / ordinary.max(SOURCE_EPS_R4)
        };
        let mut effective = *params;
        effective.k_act = params.k_act * gain;
        effective.k_c_prod = match mode {
            ProductionMode::Current => params.k_c_prod,
            ProductionMode::None => 0.0,
            ProductionMode::SharedAffinity => {
                params.k_c_prod * (1.0 - q_catalyst(before.c, params.q_c))
            }
        };
        let ledger = reactions_step(mesh, &effective, DT, true, true);
        let accepted = ledger.n_consumed;
        let after_e = area * (mesh.interior.a + mesh.interior.r).max(0.0);
        let decay = inferred_a_decay(before, mesh.interior, &ledger, area);
        let expected = ledger.a_produced
            - ledger.c_produced
            - decay
            - ledger.a_consumed_build
            - ledger.l_produced
            - ledger.reserve.r_to_w;
        let after_n = (before.n - accepted / area).max(0.0);
        let after_f = (before.f - accepted / area).max(0.0);
        SourceStep {
            accelerated: after_n * after_f < 1e-8,
            clipped: requested > accepted + SOURCE_EPS_R4,
            capacity_violation: requested > capacity + SOURCE_EPS_R4,
            accounting_residual: (after_e - before_e) - expected,
        }
    }

    fn run_phase(
        start: &MaterialMesh,
        settled: &MaterialMesh,
        law: SourceLaw,
        mode: ProductionMode,
        steps: usize,
        patch: Option<f64>,
        dose_scale: f64,
        context: &str,
        start_step: usize,
    ) -> Arm {
        let params = reaction_params(start);
        let mut mesh = start.clone();
        let mut region = patch.map(|mass| {
            FiniteSpatialResourceRegionV1::new(
                RESOURCE_CENTER,
                RESOURCE_RADIUS,
                mass * dose_scale,
                mass * dose_scale,
            )
        });
        let initial = snap(&mesh, start_step);
        let initial_distance = settled_distance(&mesh, settled);
        let mut max_resource_error: f64 = 0.0;
        let mut max_accounting: f64 = 0.0;
        let mut capacity_violations = 0;
        let mut clipping_steps = 0;
        let mut accelerated_decay_steps = 0;
        let mut alive = true;
        let mut finite = true;
        let mut hashes = vec![stable_json_hash(&initial).unwrap()];
        for step in 0..steps {
            if let Some(resource) = region.as_mut() {
                let uptake = resource.uptake(&mut mesh, &TransportParams::default(), DT);
                max_resource_error = max_resource_error.max(uptake.conservation_error.abs());
            }
            let source = apply_counterfactual(&mut mesh, &params, law, mode);
            max_accounting = max_accounting.max(source.accounting_residual.abs());
            capacity_violations += usize::from(source.capacity_violation);
            clipping_steps += usize::from(source.clipped);
            accelerated_decay_steps += usize::from(source.accelerated);
            alive &= mesh.alive;
            finite &= finite_nonnegative(&mesh);
            hashes.push(stable_json_hash(&snap(&mesh, start_step + step + 1)).unwrap());
        }
        let final_state = snap(&mesh, start_step + steps);
        Arm {
            context: context.into(),
            mode: mode.name().into(),
            dose_scale,
            steps,
            initial,
            final_state,
            alive,
            finite,
            max_resource_error,
            max_accounting_residual: max_accounting,
            capacity_violations,
            clipping_steps,
            accelerated_decay_steps,
            settled_distance_initial: initial_distance,
            settled_distance_final: settled_distance(&mesh, settled),
            a_toward_settled: (settled.interior.a - mesh.interior.a).abs()
                < (settled.interior.a - initial.a).abs(),
            r_toward_settled: (settled.interior.r - mesh.interior.r).abs()
                < (settled.interior.r - initial.r).abs(),
            trajectory_hash: stable_json_hash(&hashes).unwrap(),
        }
    }

    fn run_sustained_candidate(
        start: &MaterialMesh,
        law: SourceLaw,
        mode: ProductionMode,
        context: &str,
        steps: usize,
    ) -> Sustained {
        let params = reaction_params(start);
        let mut mesh = start.clone();
        let initial = snap(&mesh, 0);
        let mut values = Vec::with_capacity(steps);
        let mut hashes = Vec::with_capacity(steps);
        let mut clipping_steps = 0;
        let mut accelerated_decay_steps = 0;
        let mut max_accounting: f64 = 0.0;
        let mut alive = true;
        let mut finite = true;
        let mut peak = initial.e_stored;
        for step in 0..steps {
            mesh.interior.n = SUSTAINED_NF;
            mesh.interior.f = SUSTAINED_NF;
            let source = apply_counterfactual(&mut mesh, &params, law, mode);
            let state = snap(&mesh, step + 1);
            values.push(state.e_stored);
            hashes.push(stable_json_hash(&state).unwrap());
            peak = peak.max(state.e_stored);
            clipping_steps += usize::from(source.clipped);
            accelerated_decay_steps += usize::from(source.accelerated);
            max_accounting = max_accounting.max(source.accounting_residual.abs());
            alive &= mesh.alive;
            finite &= finite_nonnegative(&mesh);
        }
        let q4 = 3 * steps / 4;
        Sustained {
            context: context.into(),
            mode: mode.name().into(),
            steps,
            initial,
            final_state: snap(&mesh, steps),
            final_quarter_min: values[q4..].iter().copied().fold(f64::INFINITY, f64::min),
            final_quarter_max: values[q4..]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max),
            final_quarter_slope: (values.last().unwrap() - values[q4])
                / (values.len() - q4 - 1) as f64,
            peak_e_ar: peak,
            clipping_steps,
            accelerated_decay_steps,
            alive,
            finite,
            max_accounting_residual: max_accounting,
            trajectory_hash: stable_json_hash(&hashes).unwrap(),
        }
    }

    fn candidate_semantics(params: &ReactionParams) -> bool {
        let samples = [0.0, 1e-12, 0.05, 0.3, 1.0, 10.0, 1e6];
        let rates: Vec<f64> = samples
            .iter()
            .map(|c| candidate_rate(params, 0.7, *c))
            .collect();
        rates.iter().all(|x| x.is_finite() && *x >= 0.0)
            && (candidate_rate(params, 0.0, 0.0) == 0.0)
            && (rates[0] - params.k_c_prod * 0.7).abs() <= 1e-12
            && rates.windows(2).all(|w| w[1] <= w[0] + 1e-15)
            && rates.last().unwrap() < &rates[0]
    }

    fn finite_gate(arm: &Arm) -> bool {
        arm.alive
            && arm.finite
            && arm.max_resource_error <= MASS_TOL_R4
            && arm.max_accounting_residual <= MASS_TOL_R4
            && arm.capacity_violations == 0
            && arm.final_state.e_stored > E_DEPRIVED_R4
            && (arm.a_toward_settled || arm.r_toward_settled)
    }

    fn dose_monotonic(arms: &[Arm]) -> bool {
        arms.len() == 3
            && arms.iter().all(|x| {
                x.alive
                    && x.finite
                    && x.max_resource_error <= MASS_TOL_R4
                    && x.max_accounting_residual <= MASS_TOL_R4
                    && x.capacity_violations == 0
            })
            && arms[0].final_state.e_stored <= arms[1].final_state.e_stored + MASS_TOL_R4
            && arms[1].final_state.e_stored <= arms[2].final_state.e_stored + MASS_TOL_R4
    }

    fn baseline_final_quarter_slope(run: &SustainedRunR8R3) -> f64 {
        let q4 = 3 * run.horizon / 4;
        (run.frames.last().unwrap().e_ar - run.frames[q4].e_ar) / (run.frames.len() - q4 - 1) as f64
    }

    fn sustained_gate(run: &Sustained, baseline_slope: f64) -> bool {
        let low = 0.95 * E_TARGET_R4;
        let high = 1.05 * E_TARGET_R4;
        run.alive
            && run.finite
            && run.final_state.e_stored >= low
            && run.final_state.e_stored <= high
            && run.peak_e_ar <= 1.10 * E_TARGET_R4
            && run.final_quarter_slope.abs() <= 0.01 * baseline_slope.abs()
            && run.clipping_steps == 0
            && run.accelerated_decay_steps == 0
            && run.max_accounting_residual <= MASS_TOL_R4
    }

    pub fn run() {
        let output = std::env::var_os("DCDEV020R8R4_OUTPUT_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r8r4"));
        let dense_path = std::env::var_os("DCDEV020R8R4_DENSE_LEDGER")
            .map(PathBuf::from)
            .unwrap_or_else(|| output.join("shared_affinity_dense_ledger.json"));
        let external_location = std::env::var("DCDEV020R8R4_EXTERNAL_LOCATION")
            .unwrap_or_else(|_| "UNRECORDED_EXTERNAL_LOCATION".into());
        let external_sha = std::env::var("DCDEV020R8R4_EXTERNAL_SHA256")
            .unwrap_or_else(|_| "COMPUTED_AFTER_RUN".into());
        let result_commit =
            std::env::var("DCDEV020R8R4_RESULT_COMMIT").unwrap_or_else(|_| "PENDING".into());

        let settled = settle();
        let deprived = deprive(&settled);
        let params = reaction_params(&deprived);
        assert!(candidate_semantics(&params));
        assert!((params.q_c - ReactionParams::default().q_c).abs() <= 1e-15);

        let r6_law = r6();
        let acute = run_shadow_r8r2(&deprived, r6_law, true);
        let acute_deferred = run_shadow_r8r2(&deprived, r6_law, false);
        assert!((acute.final_state.e_stored - 60.0620310117838).abs() <= MASS_TOL_R4);
        assert!((acute_deferred.final_state.e_stored - 63.645566711951915).abs() <= MASS_TOL_R4);

        let sustained_d016_current =
            run_sustained_r8r3(&deprived, SourceLaw::Baseline, true, 8_000, &[]);
        let sustained_d016_none =
            run_sustained_r8r3(&deprived, SourceLaw::Baseline, false, 8_000, &[]);
        let sustained_r6_current = run_sustained_r8r3(&deprived, r6_law, true, 8_000, &[]);
        let sustained_r6_none = run_sustained_r8r3(&deprived, r6_law, false, 8_000, &[]);
        assert!(sustained_d016_current.alive && sustained_d016_current.finite);
        assert!(sustained_d016_none.alive && sustained_d016_none.finite);
        assert!(sustained_r6_current.alive && sustained_r6_current.finite);
        assert!(sustained_r6_none.alive && sustained_r6_none.finite);

        let finite_current = run_phase(
            &deprived,
            &settled,
            r6_law,
            ProductionMode::Current,
            WINDOW,
            Some(PATCH),
            1.0,
            "R6",
            0,
        );
        let finite_none = run_phase(
            &deprived,
            &settled,
            r6_law,
            ProductionMode::None,
            WINDOW,
            Some(PATCH),
            1.0,
            "R6",
            0,
        );
        let finite_shared = run_phase(
            &deprived,
            &settled,
            r6_law,
            ProductionMode::SharedAffinity,
            WINDOW,
            Some(PATCH),
            1.0,
            "R6",
            0,
        );
        let gate3 = finite_gate(&finite_shared);

        let doses: Vec<Arm> = [0.75, 1.0, 1.25]
            .into_iter()
            .map(|dose| {
                run_phase(
                    &deprived,
                    &settled,
                    r6_law,
                    ProductionMode::SharedAffinity,
                    WINDOW,
                    Some(PATCH),
                    dose,
                    "R6",
                    0,
                )
            })
            .collect();
        let gate4 = gate3 && dose_monotonic(&doses);

        let sustained_shared = run_sustained_candidate(
            &deprived,
            r6_law,
            ProductionMode::SharedAffinity,
            "R6",
            8_000,
        );
        let baseline_slope = baseline_final_quarter_slope(&sustained_r6_current);
        let gate5 = gate4 && sustained_gate(&sustained_shared, baseline_slope);

        let mut cycles = Vec::new();
        let mut cycle_state = sustained_shared.final_state;
        let mut cycle_mesh = deprived.clone();
        // Reconstruct the qualified-state entry deterministically, then keep it
        // continuous through all three deprivation/refeed cycles. The cycle
        // protocol is not executed when Gate 5 fails.
        let mut tmp = deprived.clone();
        let cycle_params = reaction_params(&tmp);
        if gate5 {
            for _ in 0..8_000 {
                tmp.interior.n = SUSTAINED_NF;
                tmp.interior.f = SUSTAINED_NF;
                apply_counterfactual(
                    &mut tmp,
                    &cycle_params,
                    r6_law,
                    ProductionMode::SharedAffinity,
                );
            }
            cycle_mesh = tmp;
            cycle_state = snap(&cycle_mesh, 8_000);
        }
        for cycle in 1..=3 {
            if !gate5 {
                break;
            }
            let deprived_phase = run_phase(
                &cycle_mesh,
                &settled,
                r6_law,
                ProductionMode::SharedAffinity,
                WINDOW,
                None,
                1.0,
                "R6",
                (cycle - 1) * 2 * WINDOW,
            );
            let fed_phase = run_phase(
                &deprived_phase_mesh(
                    &cycle_mesh,
                    &deprived_phase,
                    r6_law,
                    ProductionMode::SharedAffinity,
                ),
                &settled,
                r6_law,
                ProductionMode::SharedAffinity,
                WINDOW,
                Some(PATCH),
                1.0,
                "R6",
                cycle * 2 * WINDOW - WINDOW,
            );
            let recovery = fed_phase.final_state.e_stored - deprived_phase.final_state.e_stored;
            cycles.push(json!({
                "cycle": cycle,
                "deprived": deprived_phase,
                "fed": fed_phase,
                "recovery": recovery
            }));
            // Re-run the two phases from the actual previous state for continuity.
            cycle_mesh = phase_mesh(
                &cycle_mesh,
                &settled,
                r6_law,
                ProductionMode::SharedAffinity,
                WINDOW,
                None,
                1.0,
            );
            cycle_mesh = phase_mesh(
                &cycle_mesh,
                &settled,
                r6_law,
                ProductionMode::SharedAffinity,
                WINDOW,
                Some(PATCH),
                1.0,
            );
            cycle_state = snap(&cycle_mesh, cycle * 2 * WINDOW);
        }
        let recoveries: Vec<f64> = cycles
            .iter()
            .map(|x| x["recovery"].as_f64().unwrap())
            .collect();
        let gate6 = gate5
            && recoveries.len() == 3
            && recoveries.iter().all(|x| *x > 0.0)
            && recoveries[2] >= 0.90 * recoveries[0];

        let d016_finite = run_phase(
            &deprived,
            &settled,
            SourceLaw::Baseline,
            ProductionMode::SharedAffinity,
            WINDOW,
            Some(PATCH),
            1.0,
            "D016",
            0,
        );
        let d016_shared = run_sustained_candidate(
            &deprived,
            SourceLaw::Baseline,
            ProductionMode::SharedAffinity,
            "D016",
            8_000,
        );
        let gate7 = d016_finite.alive
            && d016_finite.finite
            && d016_shared.alive
            && d016_shared.finite
            && d016_finite.max_accounting_residual <= MASS_TOL_R4
            && d016_shared.max_accounting_residual <= MASS_TOL_R4;
        let classification = if !gate3 {
            "DCDEV020R8R4_SHARED_AFFINITY_FINITE_FEED_FAILURE"
        } else if !gate4 {
            "DCDEV020R8R4_SHARED_AFFINITY_DOSE_ROBUSTNESS_FAILURE"
        } else if !gate5 {
            "DCDEV020R8R4_SHARED_AFFINITY_NO_STABLE_HOMEOSTASIS"
        } else if !gate6 {
            "DCDEV020R8R4_SHARED_AFFINITY_REPEATABILITY_FAILURE"
        } else if !gate7 {
            "DCDEV020R8R4_D016_PRESERVATION_FAILURE"
        } else {
            "DCDEV020R8R4_SHARED_AFFINITY_JOINT_OBSERVER_QUALIFIED"
        };

        let dense = json!({
            "directive": "DC-DEV-020-R8-R4",
            "entry_head": ENTRY,
            "finite": [finite_current, finite_none, finite_shared],
            "dose": doses,
            "sustained_shared": {"candidate": sustained_shared, "frozen_r6_baseline_final_quarter_slope": baseline_slope},
            "d016_finite": d016_finite,
            "d016_sustained": d016_shared,
            "cycles": cycles,
            "cycle_final_state": cycle_state,
        });
        fs::create_dir_all(dense_path.parent().unwrap()).unwrap();
        fs::write(&dense_path, serde_json::to_vec(&dense).unwrap()).unwrap();

        let compact = json!({
            "directive": "DC-DEV-020-R8-R4",
            "entry_head": ENTRY,
            "clean_scientific_base": CLEAN_BASE_R4,
            "seals": {"R5": R5_SHA, "R7": R7_SHA, "R8": R8_SHA, "R8-R1": R8R1_SHA, "R8-R2": R8R2_SHA, "R8-R3": R8R3_SHA},
            "candidate": {"K_C": params.q_c, "formula": "k_c_prod*A*(1-q_c(C))", "new_parameters": false, "observer_only": true},
            "acute": {"normal": acute.final_state, "deferred": acute_deferred.final_state},
            "finite_current": finite_current,
            "finite_none": finite_none,
            "finite_shared": finite_shared,
            "dose": doses,
            "sustained": {"candidate": sustained_shared, "frozen_r6_baseline_final_quarter_slope": baseline_slope},
            "cycles": cycles,
            "d016_finite": d016_finite,
            "d016_sustained": d016_shared,
            "qualification": {"gate_0": true, "gate_1": candidate_semantics(&params), "gate_2": true, "gate_3": gate3, "gate_4": gate4, "gate_5": gate5, "gate_6": gate6, "gate_7": gate7, "classification": classification, "production_chemistry_changed": false, "production_behavior_changed": false, "dc_dev_021_authorized": false, "architect_acceptance": "PENDING", "next_execution_started": false},
            "external_evidence": {"location": external_location, "sha256": external_sha},
            "result_commit": result_commit,
        });
        write_json(
            &output,
            "protocol.json",
            &json!({"directive": "DC-DEV-020-R8-R4", "entry_head": ENTRY, "clean_scientific_base": CLEAN_BASE_R4, "R6_source": {"K_PL": R6_K_PL, "p": R6_P}, "K_C": params.q_c, "shared_affinity_formula": "k_c_prod*A*(1-q_c(C))", "finite_feed_patch": PATCH, "finite_feed_steps": WINDOW, "sustained_steps": 8_000, "E_target": E_TARGET_R4, "observer_only": true, "production_chemistry_changed": false, "production_behavior_changed": false, "dc_dev_021_authorized": false}),
        );
        write_json(
            &output,
            "acute_reproduction.json",
            &json!({"normal": acute.final_state, "deferred": acute_deferred.final_state, "normal_expected": 60.0620310117838, "deferred_expected": 63.645566711951915}),
        );
        write_json(
            &output,
            "finite_feed_summary.json",
            &json!({"current": finite_current, "zero_cprod": finite_none, "shared_affinity": finite_shared}),
        );
        write_json(
            &output,
            "dose_summary.json",
            &json!({"scales": [0.75, 1.0, 1.25], "arms": doses, "monotonic": dose_monotonic(&doses)}),
        );
        write_json(
            &output,
            "sustained_summary.json",
            &json!({"r6_shared_affinity": sustained_shared, "d016_shared_affinity": d016_shared, "frozen_r6_baseline_final_quarter_slope": baseline_slope, "target": E_TARGET_R4}),
        );
        write_json(
            &output,
            "cycle_summary.json",
            &json!({"cycles": cycles, "recoveries": recoveries, "cycle3_over_cycle1": recoveries.get(2).zip(recoveries.first()).map(|(a,b)| a/b)}),
        );
        write_json(&output, "qualification.json", &compact["qualification"]);
        write_json(
            &output,
            "literature_review.json",
            &json!({"dispositions": ["ADAPTABLE_COUPLED_CONTROL_METHOD", "ADAPTABLE_AUTOGENOUS_FEEDBACK_TOPOLOGY", "REFERENCE_DYNAMIC_RESERVE"], "external_values_imported": false, "sources": [{"pmid": "10878248", "use": "supply-demand coupling architecture only"}, {"pmc": "PMC210154", "use": "negative autogenous topology only"}, {"doi": "10.1038/s41564-022-01310-w", "use": "dynamic reserve reference only"}]}),
        );
        write_json(
            &output,
            "external_evidence_manifest.json",
            &json!({"dense_artifact": dense_path.display().to_string(), "external_location": external_location, "sha256": external_sha, "compact_git_artifacts": ["protocol.json", "acute_reproduction.json", "finite_feed_summary.json", "dose_summary.json", "sustained_summary.json", "cycle_summary.json", "qualification.json", "literature_review.json", "external_evidence_manifest.json", "manifest.json"]}),
        );
        write_json(
            &output,
            "manifest.json",
            &json!({"directive": "DC-DEV-020-R8-R4", "classification": classification, "source_commit": ENTRY, "result_commit": result_commit, "dense_location": external_location, "dense_sha256": external_sha, "preservation": ["DC-DEV-002", "DC-DEV-003", "DC-DEV-004", "DC-DEV-005", "DC-DEV-006", "DC-DEV-007", "DC-DEV-008", "DC-DEV-009", "DC-DEV-010-R1", "DC-DEV-010-R2", "DC-DEV-011", "DC-DEV-012", "DC-DEV-013", "DC-DEV-014", "DC-DEV-015", "DC-DEV-016", "DC-DEV-017", "DC-DEV-018", "DC-DEV-018-R1", "DC-DEV-019", "DC-DEV-019-R1", "DC-DEV-020-R1", "DC-DEV-020-R2", "DC-DEV-020-R3", "DC-DEV-020-R4", "DC-DEV-020-R5", "DC-DEV-020-R6", "DC-DEV-020-R7", "DC-DEV-020-R8", "DC-DEV-020-R8-R1", "DC-DEV-020-R8-R2", "DC-DEV-020-R8-R3", "Phase-1", "D-088", "evolution-harness", "governance"]}),
        );
        println!("DCDEV020R8R4_SHARED_AFFINITY_AUDIT_COMPLETE");
        println!("classification={classification}");
        println!("finite_shared_e_ar={}", finite_shared.final_state.e_stored);
        println!(
            "sustained_shared_e_ar={}",
            sustained_shared.final_state.e_stored
        );
        println!(
            "cycle3_over_cycle1={:?}",
            recoveries
                .get(2)
                .zip(recoveries.first())
                .map(|(a, b)| a / b)
        );
        println!("next_execution_started=false");
    }

    fn phase_mesh(
        start: &MaterialMesh,
        settled: &MaterialMesh,
        law: SourceLaw,
        mode: ProductionMode,
        steps: usize,
        patch: Option<f64>,
        dose: f64,
    ) -> MaterialMesh {
        let params = reaction_params(start);
        let mut mesh = start.clone();
        let mut region = patch.map(|mass| {
            FiniteSpatialResourceRegionV1::new(
                RESOURCE_CENTER,
                RESOURCE_RADIUS,
                mass * dose,
                mass * dose,
            )
        });
        for _ in 0..steps {
            if let Some(resource) = region.as_mut() {
                resource.uptake(&mut mesh, &TransportParams::default(), DT);
            }
            apply_counterfactual(&mut mesh, &params, law, mode);
        }
        let _ = settled;
        mesh
    }

    fn deprived_phase_mesh(
        start: &MaterialMesh,
        _summary: &Arm,
        law: SourceLaw,
        mode: ProductionMode,
    ) -> MaterialMesh {
        phase_mesh(start, &settle(), law, mode, WINDOW, None, 1.0)
    }
}

fn main() {
    r8r3::run();
}
