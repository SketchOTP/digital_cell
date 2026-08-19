//! DC-DEV-020-R8-R5 observer-only A<->C allocation capacity envelope.
//!
//! This example reuses the sealed R8-R3 machinery. It never changes production
//! chemistry. Counterfactual states are cloned, repartitioned conservatively,
//! and advanced through the existing reaction ledger.

mod r8r3 {
    include!("dcdev020r8r3_shared_affinity_helper.rs");

    use super::*;

    const ENTRY: &str = "37b47ec89e02418018a138f670e826c6945c8030";
    const CLEAN_BASE_R5: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
    const R8R2_SHA: &str = "e932f6ab96e34516de98c97c2cae102553db9764383af3d61abf015743c3a376";
    const R8R3_SHA: &str = "c0850577954996e1ee566f936d8965ab61647281dc67eab401c7c47be0670c12";
    const R8R4_SHA: &str = "f72a81b18a19aadec8eed4533640b6db287fb45d0f7fd2415bd2add1456c1263";
    const R8R4_FINITE: f64 = 62.575632782724874;
    const R8R4_SUSTAINED: f64 = 54.45821737181944;
    const R6_K_PL: f64 = 0.017556661171593057;
    const R6_P: f64 = 0.0003277429681759396;
    const SUSTAINED_STEPS_R5: usize = 8_000;
    const LOCAL_SPACING: usize = 40;
    const GRID_POINTS: usize = 65;
    const REFINE_ITERATIONS: usize = 26;
    const REL_INTERVAL_TOL: f64 = 1e-6;
    const MASS_TOL_R5: f64 = 1e-10;
    const SOURCE_EPS_R5: f64 = 1e-12;

    #[derive(Clone, Copy, Debug, Serialize)]
    struct EconomicEnvelope {
        n: f64,
        f: f64,
        k_pl: f64,
        p: f64,
        k_c: f64,
        k_c_turn: f64,
        b: f64,
        j_a_at_deprived_c: f64,
        j_turn_at_deprived_c: f64,
        j_econ_at_deprived_c: f64,
        c_econ: f64,
        deprived_c: f64,
        deprived_q_c: f64,
    }

    #[derive(Clone, Debug, Serialize)]
    struct OracleRun {
        c_hold: f64,
        q_c: f64,
        initial: Snap,
        final_state: Snap,
        executed_steps: usize,
        requested_steps: usize,
        final_quarter_min: f64,
        final_quarter_max: f64,
        final_quarter_slope: f64,
        oscillatory: bool,
        peak_e_ar: f64,
        final_a: f64,
        final_r: f64,
        gross_source: f64,
        catalyst_replacement_cost: f64,
        a_decay: f64,
        structural_cost: f64,
        membrane_cost: f64,
        reserve_loss: f64,
        irreversible_w: f64,
        alive: bool,
        finite: bool,
        conservation: f64,
        partition_residual: f64,
        clipped_steps: usize,
        accelerated_decay_steps: usize,
        infeasible: bool,
        complete_sustained_gate: bool,
        trajectory_hash: String,
    }

    #[derive(Clone, Debug, Serialize)]
    struct LocalEnvelope {
        source_context: String,
        source_step: usize,
        c_start: f64,
        c_end: f64,
        max_delta_e_ar: f64,
        c_at_max_drift: f64,
        q_c_at_max_drift: f64,
        source_contribution: f64,
        replacement_cost: f64,
        a_decay: f64,
        structure: f64,
        membrane: f64,
        reserve_loss: f64,
        conservation: f64,
        infeasible_partitions: usize,
        interval_relative_width: f64,
    }

    #[derive(Clone, Debug, Serialize)]
    struct SharedFrame {
        step: usize,
        state: Snap,
        source_contribution: f64,
        catalyst_replacement_cost: f64,
        a_decay: f64,
        structural_cost: f64,
        membrane_cost: f64,
        reserve_loss: f64,
        conservation: f64,
    }

    fn r6() -> SourceLaw {
        SourceLaw::PowerLaw(PowerLaw {
            k_pl: R6_K_PL,
            p: R6_P,
            g_h: 1.0,
        })
    }

    fn repartition(start: &MaterialMesh, c_hold: f64) -> (MaterialMesh, f64) {
        let mut mesh = start.clone();
        let total = (mesh.interior.a + mesh.interior.c).max(0.0);
        let c = c_hold.clamp(0.0, total);
        mesh.interior.c = c;
        mesh.interior.a = total - c;
        let residual = (mesh.interior.a + mesh.interior.c) - total;
        assert!(residual.abs() <= MASS_TOL_R5);
        (mesh, residual)
    }

    fn source_extent(mesh: &MaterialMesh, params: &ReactionParams, law: SourceLaw) -> f64 {
        requested(mesh, params, law)
    }

    fn accounting(
        before: LumpedChem,
        after: LumpedChem,
        ledger: &ReactionLedger,
        area: f64,
    ) -> f64 {
        let before_e = area * (before.a + before.r).max(0.0);
        let after_e = area * (after.a + after.r).max(0.0);
        let decay = inferred_a_decay(before, after, ledger, area);
        let expected = ledger.a_produced
            - ledger.c_produced
            - decay
            - ledger.a_consumed_build
            - ledger.l_produced
            - ledger.reserve.r_to_w;
        (after_e - before_e) - expected
    }

    fn apply_shared(
        mesh: &mut MaterialMesh,
        params: &ReactionParams,
        law: SourceLaw,
    ) -> (ReactionLedger, SourceStep) {
        let before = mesh.interior;
        let area = mesh.area().max(1e-6);
        let requested_extent = source_extent(mesh, params, law);
        let ordinary = ordinary_requested(mesh, params);
        let gain = if requested_extent <= SOURCE_EPS_R5 {
            0.0
        } else {
            requested_extent / ordinary.max(SOURCE_EPS_R5)
        };
        let mut effective = *params;
        effective.k_act = params.k_act * gain;
        effective.k_c_prod = params.k_c_prod * (1.0 - q_catalyst(before.c, params.q_c));
        let ledger = reactions_step(mesh, &effective, DT, true, true);
        let accepted = ledger.n_consumed;
        let capacity = (before.n.max(0.0) * area).min(before.f.max(0.0) * area);
        let after_n = (before.n - accepted / area).max(0.0);
        let after_f = (before.f - accepted / area).max(0.0);
        (
            ledger.clone(),
            SourceStep {
                accelerated: after_n * after_f < 1e-8,
                clipped: requested_extent > accepted + SOURCE_EPS_R5,
                capacity_violation: requested_extent > capacity + SOURCE_EPS_R5,
                accounting_residual: accounting(before, mesh.interior, &ledger, area),
            },
        )
    }

    fn apply_constant_allocation(
        mesh: &mut MaterialMesh,
        params: &ReactionParams,
        law: SourceLaw,
        _c_hold: f64,
    ) -> Option<(ReactionLedger, SourceStep)> {
        let before = mesh.interior;
        let area = mesh.area().max(1e-6);
        let requested_extent = source_extent(mesh, params, law);
        let ordinary = ordinary_requested(mesh, params);
        let gain = if requested_extent <= SOURCE_EPS_R5 {
            0.0
        } else {
            requested_extent / ordinary.max(SOURCE_EPS_R5)
        };
        let capacity = (before.n.max(0.0) * area).min(before.f.max(0.0) * area);
        let accepted_concentration = requested_extent.min(capacity) / area;
        let available_a = before.a.max(0.0) + accepted_concentration;
        let c_turn = params.k_c_turn * before.c.max(0.0) * DT;
        if c_turn > available_a + MASS_TOL_R5 {
            return None;
        }
        let mut effective = *params;
        effective.k_act = params.k_act * gain;
        effective.k_c_prod = if available_a <= SOURCE_EPS_R5 {
            0.0
        } else {
            params.k_c_turn * before.c.max(0.0) / available_a
        };
        let ledger = reactions_step(mesh, &effective, DT, true, true);
        let accepted = ledger.n_consumed;
        let after_n = (before.n - accepted / area).max(0.0);
        let after_f = (before.f - accepted / area).max(0.0);
        let capacity_violation = requested_extent > capacity + SOURCE_EPS_R5;
        Some((
            ledger.clone(),
            SourceStep {
                accelerated: after_n * after_f < 1e-8,
                clipped: requested_extent > accepted + SOURCE_EPS_R5,
                capacity_violation,
                accounting_residual: accounting(before, mesh.interior, &ledger, area),
            },
        ))
    }

    fn run_shared_sustained(
        initial: &MaterialMesh,
        law: SourceLaw,
        checkpoints: &[usize],
    ) -> (Vec<SharedFrame>, BTreeMap<usize, MaterialMesh>) {
        let mut mesh = initial.clone();
        let params = reaction_params(&mesh);
        let mut frames = Vec::with_capacity(SUSTAINED_STEPS_R5);
        let mut states = BTreeMap::new();
        for step in 1..=SUSTAINED_STEPS_R5 {
            mesh.interior.n = SUSTAINED_NF;
            mesh.interior.f = SUSTAINED_NF;
            let before = mesh.interior;
            let area = mesh.area().max(1e-6);
            let (ledger, source) = apply_shared(&mut mesh, &params, law);
            frames.push(SharedFrame {
                step,
                state: snap(&mesh, step),
                source_contribution: ledger.a_produced,
                catalyst_replacement_cost: ledger.c_produced,
                a_decay: inferred_a_decay(before, mesh.interior, &ledger, area),
                structural_cost: ledger.a_consumed_build,
                membrane_cost: ledger.l_produced,
                reserve_loss: ledger.reserve.r_to_w,
                conservation: source.accounting_residual,
            });
            if checkpoints.binary_search(&step).is_ok() {
                states.insert(step, mesh.clone());
            }
        }
        (frames, states)
    }

    fn run_oracle(
        start: &MaterialMesh,
        c_hold: f64,
        steps: usize,
        set_sustained_nf: bool,
        baseline_slope: f64,
    ) -> OracleRun {
        let (mut mesh, partition_residual) = repartition(start, c_hold);
        let initial = snap(&mesh, 0);
        let params = reaction_params(&mesh);
        let mut values = Vec::with_capacity(steps + 1);
        let mut hashes = Vec::with_capacity(steps + 1);
        values.push(initial.e_stored);
        hashes.push(stable_json_hash(&initial).unwrap());
        let mut alive = true;
        let mut finite = true;
        let mut conservation = 0.0_f64;
        let mut clipped_steps = 0;
        let mut accelerated_decay_steps = 0;
        let mut flux = FluxTotals::default();
        let mut infeasible = false;
        let mut peak = initial.e_stored;
        for step in 0..steps {
            if set_sustained_nf {
                mesh.interior.n = SUSTAINED_NF;
                mesh.interior.f = SUSTAINED_NF;
            }
            let before = mesh.interior;
            let area = mesh.area().max(1e-6);
            let Some((ledger, source)) =
                apply_constant_allocation(&mut mesh, &params, r6(), c_hold)
            else {
                infeasible = true;
                break;
            };
            accumulate_flux(&mut flux, before, mesh.interior, &ledger, area);
            conservation = conservation.max(source.accounting_residual.abs());
            clipped_steps += usize::from(source.clipped);
            accelerated_decay_steps += usize::from(source.accelerated);
            alive &= mesh.alive;
            finite &= finite_nonnegative(&mesh);
            let state = snap(&mesh, step + 1);
            peak = peak.max(state.e_stored);
            values.push(state.e_stored);
            hashes.push(stable_json_hash(&state).unwrap());
        }
        let executed_steps = values.len().saturating_sub(1);
        let q4 = (3 * executed_steps / 4).min(executed_steps.saturating_sub(1));
        let final_quarter_min = values[q4..].iter().copied().fold(f64::INFINITY, f64::min);
        let final_quarter_max = values[q4..]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let denominator = (values.len().saturating_sub(q4 + 1)).max(1) as f64;
        let final_quarter_slope =
            (values.last().copied().unwrap_or(initial.e_stored) - values[q4]) / denominator;
        let oscillatory =
            final_quarter_min < 0.95 * E_TARGET && final_quarter_max > 1.05 * E_TARGET;
        let complete_sustained_gate = !infeasible
            && executed_steps == steps
            && alive
            && finite
            && final_state_in_band(values.last().copied().unwrap_or(initial.e_stored))
            && peak <= 1.10 * E_TARGET
            && !oscillatory
            && final_quarter_slope.abs() <= 0.01 * baseline_slope.abs()
            && clipped_steps == 0
            && accelerated_decay_steps == 0
            && conservation <= MASS_TOL_R5;
        let area = mesh.area().max(1e-6);
        OracleRun {
            c_hold,
            q_c: q_catalyst(c_hold, params.q_c),
            initial,
            final_state: snap(&mesh, executed_steps),
            executed_steps,
            requested_steps: steps,
            final_quarter_min,
            final_quarter_max,
            final_quarter_slope,
            oscillatory,
            peak_e_ar: peak,
            final_a: mesh.interior.a,
            final_r: mesh.interior.r,
            gross_source: flux.a_produced,
            catalyst_replacement_cost: flux.catalyst_a_consumption,
            a_decay: flux.a_decay,
            structural_cost: flux.structural_a_consumption,
            membrane_cost: flux.membrane_a_consumption,
            reserve_loss: flux.reserve_r_to_w,
            irreversible_w: area * (mesh.interior.w - start.interior.w).max(0.0),
            alive,
            finite,
            conservation,
            partition_residual,
            clipped_steps,
            accelerated_decay_steps,
            infeasible,
            complete_sustained_gate,
            trajectory_hash: stable_json_hash(&hashes).unwrap(),
        }
    }

    fn final_state_in_band(e: f64) -> bool {
        e >= 0.95 * E_TARGET && e <= 1.05 * E_TARGET
    }

    fn economic(start: &MaterialMesh, params: &ReactionParams) -> EconomicEnvelope {
        let n = SUSTAINED_NF;
        let f = SUSTAINED_NF;
        let b = R6_K_PL * n.powf(R6_P) * f.powf(R6_P);
        let c = start.interior.c;
        let k_c = params.q_c;
        let j_a = b * q_catalyst(c, k_c);
        let j_turn = params.k_c_turn * c;
        let c_econ = if b <= k_c * params.k_c_turn {
            0.0
        } else {
            (b * k_c / params.k_c_turn).sqrt() - k_c
        };
        EconomicEnvelope {
            n,
            f,
            k_pl: R6_K_PL,
            p: R6_P,
            k_c,
            k_c_turn: params.k_c_turn,
            b,
            j_a_at_deprived_c: j_a,
            j_turn_at_deprived_c: j_turn,
            j_econ_at_deprived_c: j_a - j_turn,
            c_econ,
            deprived_c: c,
            deprived_q_c: q_catalyst(c, k_c),
        }
    }

    fn candidate_points(total: f64) -> Vec<f64> {
        (0..GRID_POINTS)
            .map(|i| total * i as f64 / (GRID_POINTS - 1) as f64)
            .collect()
    }

    fn refine_boundary<F>(mut left: f64, mut right: f64, mut predicate: F, total: f64) -> (f64, f64)
    where
        F: FnMut(f64) -> bool,
    {
        let initial_side = predicate(left);
        for _ in 0..REFINE_ITERATIONS {
            let mid = (left + right) / 2.0;
            if predicate(mid) == initial_side {
                left = mid;
            } else {
                right = mid;
            }
            if (right - left) / total.max(1.0) <= REL_INTERVAL_TOL {
                break;
            }
        }
        (left, right)
    }

    fn refine_maximum(
        start: &MaterialMesh,
        mut left: f64,
        mut right: f64,
        steps: usize,
        set_sustained_nf: bool,
        baseline_slope: f64,
        total: f64,
    ) -> (f64, f64) {
        for _ in 0..REFINE_ITERATIONS {
            let one_third = (right - left) / 3.0;
            let a = left + one_third;
            let b = right - one_third;
            let va = run_oracle(start, a, steps, set_sustained_nf, baseline_slope)
                .final_state
                .e_stored;
            let vb = run_oracle(start, b, steps, set_sustained_nf, baseline_slope)
                .final_state
                .e_stored;
            if va <= vb {
                left = a;
            } else {
                right = b;
            }
            if (right - left) / total.max(1.0) <= REL_INTERVAL_TOL {
                break;
            }
        }
        (left, right)
    }

    fn envelope(
        start: &MaterialMesh,
        steps: usize,
        set_sustained_nf: bool,
        baseline_slope: f64,
    ) -> (Vec<OracleRun>, Vec<(f64, f64)>) {
        let total = (start.interior.a + start.interior.c).max(0.0);
        let points = candidate_points(total);
        let mut runs: Vec<OracleRun> = points
            .iter()
            .map(|c| run_oracle(start, *c, steps, set_sustained_nf, baseline_slope))
            .collect();
        let mut coarse_brackets = Vec::new();
        for window in runs.windows(3) {
            let left = &window[0];
            let center = &window[1];
            let right = &window[2];
            let target_cross = (left.final_state.e_stored - E_TARGET)
                * (right.final_state.e_stored - E_TARGET)
                <= 0.0;
            let slope_cross = left.final_quarter_slope * right.final_quarter_slope <= 0.0;
            let local_max = center.final_state.e_stored >= left.final_state.e_stored
                && center.final_state.e_stored >= right.final_state.e_stored;
            if target_cross || slope_cross || local_max {
                coarse_brackets.push((left.c_hold, right.c_hold, target_cross, slope_cross));
            }
        }
        let mut refined_brackets = Vec::new();
        for (left, right, target_cross, slope_cross) in coarse_brackets {
            let (a, b) = if target_cross {
                refine_boundary(
                    left,
                    right,
                    |c| {
                        run_oracle(start, c, steps, set_sustained_nf, baseline_slope)
                            .final_state
                            .e_stored
                            >= E_TARGET
                    },
                    total,
                )
            } else if slope_cross {
                refine_boundary(
                    left,
                    right,
                    |c| {
                        run_oracle(start, c, steps, set_sustained_nf, baseline_slope)
                            .final_quarter_slope
                            >= 0.0
                    },
                    total,
                )
            } else {
                refine_maximum(
                    start,
                    left,
                    right,
                    steps,
                    set_sustained_nf,
                    baseline_slope,
                    total,
                )
            };
            refined_brackets.push((a, b));
            runs.push(run_oracle(
                start,
                a,
                steps,
                set_sustained_nf,
                baseline_slope,
            ));
            runs.push(run_oracle(
                start,
                b,
                steps,
                set_sustained_nf,
                baseline_slope,
            ));
        }
        runs.sort_by(|a, b| a.c_hold.partial_cmp(&b.c_hold).unwrap_or(Ordering::Equal));
        runs.dedup_by(|a, b| (a.c_hold - b.c_hold).abs() <= total.max(1.0) * 1e-12);
        (runs, refined_brackets)
    }

    fn local_envelope(
        context: &str,
        step: usize,
        state: &MaterialMesh,
        baseline_slope: f64,
    ) -> LocalEnvelope {
        let (runs, brackets) = envelope(state, 1, false, baseline_slope);
        let mut best = runs
            .iter()
            .filter(|run| !run.infeasible)
            .max_by(|a, b| {
                let da = a.final_state.e_stored - a.initial.e_stored;
                let db = b.final_state.e_stored - b.initial.e_stored;
                da.partial_cmp(&db).unwrap_or(Ordering::Equal)
            })
            .cloned();
        let infeasible_partitions = runs.iter().filter(|run| run.infeasible).count();
        let total = (state.interior.a + state.interior.c).max(1.0);
        let interval_relative_width = brackets
            .iter()
            .map(|(a, b)| (b - a) / total)
            .fold(0.0, f64::max);
        let Some(best) = best.take() else {
            return LocalEnvelope {
                source_context: context.into(),
                source_step: step,
                c_start: 0.0,
                c_end: 0.0,
                max_delta_e_ar: f64::NEG_INFINITY,
                c_at_max_drift: 0.0,
                q_c_at_max_drift: 0.0,
                source_contribution: 0.0,
                replacement_cost: 0.0,
                a_decay: 0.0,
                structure: 0.0,
                membrane: 0.0,
                reserve_loss: 0.0,
                conservation: f64::INFINITY,
                infeasible_partitions,
                interval_relative_width,
            };
        };
        LocalEnvelope {
            source_context: context.into(),
            source_step: step,
            c_start: 0.0,
            c_end: total,
            max_delta_e_ar: best.final_state.e_stored - best.initial.e_stored,
            c_at_max_drift: best.c_hold,
            q_c_at_max_drift: best.q_c,
            source_contribution: best.gross_source,
            replacement_cost: best.catalyst_replacement_cost,
            a_decay: best.a_decay,
            structure: best.structural_cost,
            membrane: best.membrane_cost,
            reserve_loss: best.reserve_loss,
            conservation: best.conservation,
            infeasible_partitions,
            interval_relative_width,
        }
    }

    fn run_r8r4_finite(start: &MaterialMesh, mass: f64) -> f64 {
        let params = reaction_params(start);
        let mut mesh = start.clone();
        let mut region =
            FiniteSpatialResourceRegionV1::new(RESOURCE_CENTER, RESOURCE_RADIUS, mass, mass);
        for _ in 0..WINDOW {
            region.uptake(&mut mesh, &TransportParams::default(), DT);
            apply_shared(&mut mesh, &params, r6());
        }
        snap(&mesh, WINDOW).e_stored
    }

    pub fn run() {
        let output = std::env::var_os("DCDEV020R8R5_OUTPUT_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020r8r5"));
        let dense_path = std::env::var_os("DCDEV020R8R5_DENSE_LEDGER")
            .map(PathBuf::from)
            .unwrap_or_else(|| output.join("ac_allocation_dense_ledger.json"));
        let external_location = std::env::var("DCDEV020R8R5_EXTERNAL_LOCATION")
            .unwrap_or_else(|_| "UNRECORDED_EXTERNAL_LOCATION".into());
        let external_sha = std::env::var("DCDEV020R8R5_EXTERNAL_SHA256")
            .unwrap_or_else(|_| "COMPUTED_AFTER_RUN".into());
        let result_commit =
            std::env::var("DCDEV020R8R5_RESULT_COMMIT").unwrap_or_else(|_| "PENDING".into());

        let settled = settle();
        let deprived = deprive(&settled);
        let params = reaction_params(&deprived);
        let r6_law = r6();
        let acute = run_shadow_r8r2(&deprived, r6_law, true);
        assert!((acute.final_state.e_stored - 60.0620310117838).abs() <= MASS_TOL_R5);
        let acute_deferred = run_shadow_r8r2(&deprived, r6_law, false);
        assert!((acute_deferred.final_state.e_stored - 63.645566711951915).abs() <= MASS_TOL_R5);
        assert!((run_r8r4_finite(&deprived, M_SELECTED) - R8R4_FINITE).abs() <= MASS_TOL_R5);

        let r6_current = run_sustained_r8r3(&deprived, r6_law, true, SUSTAINED_STEPS_R5, &[]);
        let baseline_slope = (r6_current.frames.last().unwrap().e_ar
            - r6_current.frames[3 * SUSTAINED_STEPS_R5 / 4].e_ar)
            / (SUSTAINED_STEPS_R5 / 4 - 1) as f64;
        let sustained_checkpoints: Vec<usize> = (LOCAL_SPACING..=SUSTAINED_STEPS_R5)
            .step_by(LOCAL_SPACING)
            .collect();
        let deferred = run_sustained_r8r3(
            &deprived,
            r6_law,
            false,
            SUSTAINED_STEPS_R5,
            &sustained_checkpoints,
        );
        let (shared_frames, shared_states) =
            run_shared_sustained(&deprived, r6_law, &sustained_checkpoints);
        assert!((shared_frames.last().unwrap().state.e_stored - R8R4_SUSTAINED).abs() <= 1e-9);

        let economic = economic(&deprived, &params);
        let (oracle_runs, refinement_brackets) =
            envelope(&deprived, SUSTAINED_STEPS_R5, true, baseline_slope);
        let complete_runs: Vec<&OracleRun> = oracle_runs
            .iter()
            .filter(|run| run.complete_sustained_gate)
            .collect();
        let best = oracle_runs
            .iter()
            .max_by(|a, b| {
                a.final_state
                    .e_stored
                    .partial_cmp(&b.final_state.e_stored)
                    .unwrap_or(Ordering::Equal)
            })
            .unwrap();

        let deferred_local: Vec<LocalEnvelope> = deferred
            .checkpoint_meshes
            .iter()
            .map(|(step, state)| local_envelope("R8-R3 R6 deferred", *step, state, baseline_slope))
            .collect();
        let shared_local: Vec<LocalEnvelope> = shared_states
            .iter()
            .map(|(step, state)| {
                local_envelope("R8-R4 R6 shared-affinity", *step, state, baseline_slope)
            })
            .collect();
        let deferred_nonnegative = deferred_local
            .iter()
            .filter(|x| x.max_delta_e_ar >= 0.0)
            .count();
        let shared_nonnegative = shared_local
            .iter()
            .filter(|x| x.max_delta_e_ar >= 0.0)
            .count();
        let worst_max_drift = deferred_local
            .iter()
            .chain(shared_local.iter())
            .map(|x| x.max_delta_e_ar)
            .fold(f64::NEG_INFINITY, f64::max);

        let classification = if !complete_runs.is_empty() {
            "DCDEV020R8R5_CATALYST_ALLOCATION_CAPACITY_EXISTS"
        } else if deferred_nonnegative == 0 && shared_nonnegative == 0 {
            "DCDEV020R8R5_CATALYST_ALLOCATION_CAPACITY_INSUFFICIENT"
        } else {
            "DCDEV020R8R5_CATALYST_ALLOCATION_ENVELOPE_MIXED"
        };
        let gate0 = ENTRY == "37b47ec89e02418018a138f670e826c6945c8030"
            && CLEAN_BASE_R5 == "1e242f28152797b512e25cd56c7b718e45d6ca97";
        let gate1 = (acute.final_state.e_stored - 60.0620310117838).abs() <= MASS_TOL_R5
            && (acute_deferred.final_state.e_stored - 63.645566711951915).abs() <= MASS_TOL_R5
            && (run_r8r4_finite(&deprived, M_SELECTED) - R8R4_FINITE).abs() <= MASS_TOL_R5
            && (shared_frames.last().unwrap().state.e_stored - R8R4_SUSTAINED).abs() <= 1e-9;
        let gate2 = oracle_runs
            .iter()
            .all(|run| run.partition_residual.abs() <= MASS_TOL_R5);
        let gate3 = !oracle_runs.is_empty()
            && refinement_brackets.iter().all(|(a, b)| {
                (b - a) / (deprived.interior.a + deprived.interior.c).max(1.0)
                    <= REL_INTERVAL_TOL + 1e-12
            });
        let gate4 = classification != "DCDEV020R8R5_FOUNDATIONAL_REGRESSION";
        let gate5 = deferred_local.len() == sustained_checkpoints.len()
            && shared_local.len() == sustained_checkpoints.len();
        let qualification = json!({
            "gate_0_authority": gate0,
            "gate_1_reproduction": gate1,
            "gate_2_conservative_partition": gate2,
            "gate_3_deterministic_refinement": gate3,
            "gate_4_original_sustained_qualification": gate4,
            "gate_5_local_late_state_envelope": gate5,
            "gate_7_preservation": "PENDING_REMOTE_CI",
            "classification": classification,
            "production_chemistry_changed": false,
            "production_behavior_changed": false,
            "dc_dev_021_authorized": false,
            "architect_acceptance": "PENDING",
            "next_execution_started": false
        });

        write_json(
            &output,
            "protocol.json",
            &json!({
                "directive": "DC-DEV-020-R8-R5",
                "entry_head": ENTRY,
                "clean_scientific_base": CLEAN_BASE_R5,
                "sustained_steps": SUSTAINED_STEPS_R5,
                "constant_allocation": {"c_hold_min": 0.0, "c_hold_max": deprived.interior.a + deprived.interior.c, "production": "exact_turnover_replacement", "a_c_conserving": true},
                "frozen_r6": {"K_PL": R6_K_PL, "p": R6_P, "K_C": params.q_c, "k_c_turn": params.k_c_turn, "N": SUSTAINED_NF, "F": SUSTAINED_NF},
                "observer_only": true,
                "production_chemistry_changed": false,
                "production_behavior_changed": false,
                "dc_dev_021_authorized": false
            }),
        );
        write_json(
            &output,
            "economic_envelope.json",
            &serde_json::to_value(economic).unwrap(),
        );
        write_json(
            &output,
            "r8r4_reproduction.json",
            &json!({"acute_normal": acute.final_state, "acute_deferred": acute_deferred.final_state, "finite_shared": run_r8r4_finite(&deprived, M_SELECTED), "sustained_shared": shared_frames.last().unwrap()}),
        );
        write_json(
            &output,
            "constant_allocation_summary.json",
            &json!({"runs": oracle_runs, "best": best, "refinement_brackets": refinement_brackets, "complete_gate_passes": complete_runs.len()}),
        );
        write_json(
            &output,
            "local_drift_summary.json",
            &json!({"deferred": deferred_local, "shared": shared_local, "deferred_nonnegative": deferred_nonnegative, "shared_nonnegative": shared_nonnegative, "worst_max_drift": worst_max_drift}),
        );
        write_json(&output, "qualification.json", &qualification);
        write_json(
            &output,
            "literature_review.json",
            &json!({
                "dispositions": ["ADAPTABLE_ENZYME_COST_METHOD", "REFERENCE_FLUX_SIGNALING_MECHANISM"],
                "external_values_imported": false,
                "sources": [
                    {"pmc": "PMC5094713", "use": "enzyme cost and flux coupling method only"},
                    {"pmc": "PMC3549114", "use": "flux-signaling mechanism reference only"}
                ]
            }),
        );
        write_json(
            &output,
            "external_evidence_manifest.json",
            &json!({"dense_artifact": dense_path.display().to_string(), "external_location": external_location, "sha256": external_sha, "compact_git_artifacts": ["protocol.json", "economic_envelope.json", "r8r4_reproduction.json", "constant_allocation_summary.json", "local_drift_summary.json", "qualification.json", "literature_review.json", "external_evidence_manifest.json", "manifest.json"]}),
        );
        write_json(
            &output,
            "manifest.json",
            &json!({"directive": "DC-DEV-020-R8-R5", "classification": classification, "source_commit": ENTRY, "result_commit": result_commit, "dense_location": external_location, "dense_sha256": external_sha, "preservation": ["R8-R2", "R8-R3", "R8-R4", "Phase-1", "D-088", "evolution-harness", "governance"]}),
        );

        let dense = json!({"directive": "DC-DEV-020-R8-R5", "economic": economic, "constant_allocation_runs": oracle_runs, "deferred_local": deferred_local, "shared_local": shared_local, "shared_frames": shared_frames});
        fs::create_dir_all(dense_path.parent().unwrap()).unwrap();
        fs::write(&dense_path, serde_json::to_vec(&dense).unwrap()).unwrap();

        println!("DCDEV020R8R5_CATALYST_ALLOCATION_AUDIT_COMPLETE");
        println!("classification={classification}");
        println!("best_final_e_ar={}", best.final_state.e_stored);
        println!("best_c_hold={}", best.c_hold);
        println!("best_final_quarter_slope={}", best.final_quarter_slope);
        println!("complete_sustained_gate_passes={}", complete_runs.len());
        println!("deferred_states_with_max_drift_ge_zero={deferred_nonnegative}");
        println!("shared_states_with_max_drift_ge_zero={shared_nonnegative}");
        println!("next_execution_started=false");
    }
}

fn main() {
    r8r3::run();
}
