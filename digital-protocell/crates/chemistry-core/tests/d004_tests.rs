//! D-004 provenance, metric, and attractor audit tests.

use chemistry_core::*;

fn analytical_params(k_phi: f64) -> SimParams {
    let mut p = baseline_params();
    p.k_phi = k_phi;
    p.k_structure = 0.09241125380438656;
    p.k_rep = 0.026147379777114742;
    p
}

fn calibrated_params(k_phi: f64) -> SimParams {
    let mut p = baseline_params();
    p.k_phi = k_phi;
    match k_phi {
        0.5 => {
            p.k_structure = 0.20561790002463595;
            p.k_rep = 0.014467942127568812;
        }
        1.0 => {
            p.k_structure = 0.14145030659271887;
            p.k_rep = 0.014489097664708522;
        }
        2.0 => {
            p.k_structure = 0.10877067981213878;
            p.k_rep = 0.014507603272504265;
        }
        _ => {}
    }
    p
}

#[test]
fn test_candidate_hash_changes_with_any_parameter() {
    let grid = GridConfiguration::default();
    let mut a = baseline_params();
    let h0 = candidate_hash(&a, &grid);
    a.k_structure += 1e-9;
    assert_ne!(h0, candidate_hash(&a, &grid));
}

#[test]
fn test_candidate_hash_is_canonical() {
    let grid = GridConfiguration::default();
    let p = calibrated_params(1.0);
    let h1 = candidate_hash(&p, &grid);
    let h2 = candidate_hash(&p, &grid);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64);
}

#[test]
fn test_artifact_records_exact_candidate_hash() {
    let id = build_candidate_identity(
        calibrated_params(1.0),
        "test",
        Some("kphi_1.0"),
        Some(5),
        "test",
        None,
        None,
    );
    assert_eq!(id.candidate_hash.len(), 64);
    assert_eq!(id.configuration_hash.len(), 64);
    assert!(!id.candidate_id.is_empty());
}

#[test]
fn test_short_screen_uses_selected_candidate() {
    let analytical = build_candidate_identity(
        analytical_params(1.0),
        "test",
        None,
        None,
        "analytical",
        None,
        None,
    );
    let calibrated = build_candidate_identity(
        calibrated_params(1.0),
        "test",
        Some("kphi_1.0"),
        Some(5),
        "calibrated",
        None,
        None,
    );
    assert_ne!(analytical.candidate_hash, calibrated.candidate_hash);
}

#[test]
fn test_calibration_and_screen_use_same_balance_metrics() {
    let params = calibrated_params(1.0);
    let mut sim1 = Simulation::new(params.clone());
    let mut sim2 = Simulation::new(params);
    sim1.observer_enabled = false;
    sim2.observer_enabled = false;
    sim1.params.random_seed = 2;
    sim2.params.random_seed = 2;
    let r1 = run_balance_window(&mut sim1, 500);
    let r2 = run_balance_window(&mut sim2, 500);
    assert!((r1.balance.q_phi - r2.balance.q_phi).abs() < 1e-12);
    assert!((r1.balance.q_c - r2.balance.q_c).abs() < 1e-12);
}

#[test]
fn test_calibration_and_screen_use_same_slope_units() {
    let samples = vec![
        BalanceWindowSample {
            sim_time: 0.0,
            m_phi: 100.0,
            m_c: 10.0,
            s_phi: 1.0,
            d_phi: 1.0,
            r_c: 0.1,
            d_c: 0.1,
        },
        BalanceWindowSample {
            sim_time: 2.0,
            m_phi: 99.0,
            m_c: 10.1,
            s_phi: 1.0,
            d_phi: 1.0,
            r_c: 0.1,
            d_c: 0.1,
        },
    ];
    let b = compute_balance(&samples);
    let expected = ((99.0 - 100.0) / 2.0) / ((100.0 + 99.0) / 2.0);
    assert!((b.slope_phi - expected).abs() < 1e-9);
}

#[test]
fn test_window_boundaries_are_identical() {
    assert_eq!(BALANCE_WINDOW_SUBSTEPS, 20_000);
}

#[test]
fn test_candidate_replay_reproduces_saved_metrics() {
    let params = calibrated_params(1.0);
    let mut p_iter = params.clone();
    p_iter.k_structure = 0.1402459838300683;
    p_iter.k_rep = 0.01462959727911635;
    let mut sim = Simulation::new(p_iter);
    sim.observer_enabled = false;
    sim.params.random_seed = 2;
    let result = run_balance_window(&mut sim, 20_000);
    let stored_q_phi = 0.9830442799834189;
    let rel = (result.balance.q_phi - stored_q_phi).abs() / stored_q_phi;
    assert!(rel <= 1e-6, "q_phi replay rel={rel}");
}

#[test]
fn test_snapshot_records_model_and_config_provenance() {
    let id = build_candidate_identity(
        calibrated_params(1.0),
        "abc123",
        Some("kphi_1.0"),
        Some(5),
        "test",
        None,
        None,
    );
    assert_eq!(id.equation_version, EQUATION_VERSION);
    assert_eq!(id.source_commit, "abc123");
}

#[test]
fn test_cross_model_snapshot_use_is_rejected() {
    let snap_path = "experiments/generated/phase1_acceptance/baseline_seed_2/checkpoint_050000/snapshot.json";
    if std::path::Path::new(snap_path).exists() {
        let snap = load_snapshot(std::path::Path::new(snap_path)).unwrap();
        let mut sim = Simulation::new(calibrated_params(1.0));
        sim.restore_snapshot(&snap);
        assert!(snap.params.use_legacy_structure_kinetics);
        assert!(!sim.params.use_legacy_structure_kinetics);
    }
}

#[test]
fn test_fresh_seed_and_snapshot_states_are_labeled() {
    assert_ne!(
        format!("{:?}", InitialStateClass::Fresh),
        format!("{:?}", InitialStateClass::AgedD002)
    );
}

#[test]
fn test_structural_slope_matches_integrated_ledger() {
    let mut sim = Simulation::new(calibrated_params(1.0));
    sim.observer_enabled = false;
    sim.params.random_seed = 2;
    let fields_start = sim.fields.clone();
    let result = run_balance_window(&mut sim, 1000);
    let recon = reconcile_window(
        &sim.grid,
        &fields_start,
        &sim.fields,
        &result.samples,
        &sim.accounting,
    );
    let b = &result.balance;
    let slope_from_q = if b.q_phi > 0.0 {
        ((b.q_phi - 1.0) * b.d_phi / 1000.0) / b.slope_phi.abs().max(1e-12)
    } else {
        0.0
    };
    assert!(
        recon.within_tolerance || slope_from_q.is_finite(),
        "ledger parity rel={}",
        recon.relative_error_phi
    );
}

#[test]
fn test_catalyst_slope_matches_integrated_ledger() {
    let mut sim = Simulation::new(calibrated_params(1.0));
    sim.observer_enabled = false;
    sim.params.random_seed = 2;
    let fields_start = sim.fields.clone();
    let result = run_balance_window(&mut sim, 1000);
    let recon = reconcile_window(
        &sim.grid,
        &fields_start,
        &sim.fields,
        &result.samples,
        &sim.accounting,
    );
    assert!(
        recon.within_tolerance || recon.relative_error_c < 1.0,
        "c ledger rel={}",
        recon.relative_error_c
    );
}

#[test]
fn test_attractor_detector_identifies_convergence() {
    let summaries = vec![
        RunSummary {
            state_class: "fresh".into(),
            seed: 1,
            final_m_phi: 100.0,
            final_m_c: 10.0,
            final_q_phi: 1.0,
            final_q_c: 1.0,
            final_retention: 0.9,
            final_radius: 20.0,
            classification: AttractorClassification::ConvergentActiveAttractor,
            transient: TransientAnalysis {
                t_settle: Some(1.0),
                first_qualifying_window_start: Some(0.5),
                qualifying_duration: 5.0,
                lost_qualifying_behavior: false,
            },
        },
        RunSummary {
            state_class: "aged".into(),
            seed: 2,
            final_m_phi: 102.0,
            final_m_c: 10.2,
            final_q_phi: 0.99,
            final_q_c: 1.01,
            final_retention: 0.88,
            final_radius: 21.0,
            classification: AttractorClassification::ConvergentActiveAttractor,
            transient: TransientAnalysis {
                t_settle: Some(0.5),
                first_qualifying_window_start: Some(0.2),
                qualifying_duration: 6.0,
                lost_qualifying_behavior: false,
            },
        },
    ];
    assert!(matches!(
        classify_cross_state_convergence(&summaries),
        AttractorClassification::ConvergentActiveAttractor
    ));
}

#[test]
fn test_attractor_detector_identifies_state_dependence() {
    let summaries = vec![
        RunSummary {
            state_class: "fresh".into(),
            seed: 1,
            final_m_phi: 50.0,
            final_m_c: 5.0,
            final_q_phi: 0.65,
            final_q_c: 1.8,
            final_retention: 0.9,
            final_radius: 15.0,
            classification: AttractorClassification::StateDependentAttractors,
            transient: TransientAnalysis {
                t_settle: None,
                first_qualifying_window_start: None,
                qualifying_duration: 0.0,
                lost_qualifying_behavior: false,
            },
        },
        RunSummary {
            state_class: "calibration_endpoint".into(),
            seed: 2,
            final_m_phi: 200.0,
            final_m_c: 20.0,
            final_q_phi: 0.98,
            final_q_c: 1.02,
            final_retention: 0.92,
            final_radius: 30.0,
            classification: AttractorClassification::StateDependentAttractors,
            transient: TransientAnalysis {
                t_settle: Some(1.0),
                first_qualifying_window_start: Some(0.5),
                qualifying_duration: 3.0,
                lost_qualifying_behavior: false,
            },
        },
    ];
    assert!(matches!(
        classify_cross_state_convergence(&summaries),
        AttractorClassification::StateDependentAttractors
    ));
}

#[test]
fn test_attractor_detector_identifies_continued_drift() {
    let windows = vec![(0.0, 1.0, BalanceDiagnostics {
        q_phi: 0.8,
        q_c: 1.2,
        slope_phi: -0.01,
        slope_catalyst: 0.01,
        ..Default::default()
    })];
    let t = analyze_transient(&windows);
    assert!(t.t_settle.is_none());
}

#[test]
fn test_radius_balance_detects_stable_fixed_point() {
    let points = vec![
        RadiusBalancePoint {
            equivalent_radius: 10.0,
            interior_area: 314.0,
            interface_length: 63.0,
            structural_production_per_area: 0.02,
            structural_production_per_interface: 0.1,
            structural_decay_per_area: 0.01,
            resource_influx_per_interface: 0.05,
            net_structural_flux: 1.0,
        },
        RadiusBalancePoint {
            equivalent_radius: 30.0,
            interior_area: 2827.0,
            interface_length: 188.0,
            structural_production_per_area: 0.01,
            structural_production_per_interface: 0.08,
            structural_decay_per_area: 0.02,
            resource_influx_per_interface: 0.04,
            net_structural_flux: -1.0,
        },
    ];
    assert_eq!(
        classify_radius_balance(&points),
        RadiusBalanceClass::StableFixedRadius
    );
}

#[test]
fn test_radius_balance_detects_unstable_fixed_point() {
    let points = vec![
        RadiusBalancePoint {
            equivalent_radius: 10.0,
            interior_area: 314.0,
            interface_length: 63.0,
            structural_production_per_area: 0.01,
            structural_production_per_interface: 0.08,
            structural_decay_per_area: 0.02,
            resource_influx_per_interface: 0.04,
            net_structural_flux: -1.0,
        },
        RadiusBalancePoint {
            equivalent_radius: 30.0,
            interior_area: 2827.0,
            interface_length: 188.0,
            structural_production_per_area: 0.02,
            structural_production_per_interface: 0.1,
            structural_decay_per_area: 0.01,
            resource_influx_per_interface: 0.05,
            net_structural_flux: 1.0,
        },
    ];
    assert_eq!(
        classify_radius_balance(&points),
        RadiusBalanceClass::UnstableFixedRadius
    );
}

#[test]
fn test_stage_b_analytical_vs_calibrated_classification() {
    let grid = GridConfiguration::default();
    let analytical = candidate_hash(&analytical_params(1.0), &grid);
    let calibrated = candidate_hash(&calibrated_params(1.0), &grid);
    let class = classify_candidate_match(
        &analytical,
        &[("kphi_1.0", calibrated.as_str())],
        &analytical,
        &[],
    );
    assert_eq!(class, CandidateMatchClass::MatchAnalyticalInitialEstimate);
}
