//! D-038 focused tests: corrected turnover transfer and renewal selection rules.

use chemistry_core::config::{SimParams, SurfaceTurnoverSchema};
use chemistry_core::d038_analysis::{
    candidate_scale_plan, gate0_preservation, gate1_corrected_bulk_surface_equivalence,
    gate1_decay_trajectories, gate2_integrator_validation, multistart_attractor_agree,
    multistart_set, route_decision, select_architecture, three_consecutive_balance,
    transient_balance_eligibility, MembraneArchitecture, TransientBalanceEligibility,
    D038_FROZEN_EPS_M, D038_K_MEMBRANE_DECAY, D038_LOSS_EQUIV_RTOL,
};
use chemistry_core::surface_density::{
    apply_surface_turnover_exact, surface_turnover_lambda, surface_turnover_protection_factor,
};

#[test]
fn turnover_schema_isolation_default_historical() {
    let p = SimParams::default();
    assert_eq!(
        p.surface_turnover_schema,
        SurfaceTurnoverSchema::HistoricalUniform
    );
    assert_eq!(
        p.surface_turnover_schema.as_str(),
        "surface_turnover_schema_1_historical_uniform"
    );
}

#[test]
fn historical_schema1_equivalence_uniform_lambda() {
    let mut p = SimParams::default();
    p.k_gamma_decay = D038_K_MEMBRANE_DECAY;
    p.surface_turnover_schema = SurfaceTurnoverSchema::HistoricalUniform;
    let lam = surface_turnover_lambda(0.5, &p);
    assert!((lam - D038_K_MEMBRANE_DECAY).abs() < 1e-15);
    let (s1, dw) = apply_surface_turnover_exact(1.0, 0.5, &p, 0.1);
    let expect = (-D038_K_MEMBRANE_DECAY * 0.1_f64).exp();
    assert!((s1 - expect).abs() < 1e-14);
    assert!((dw - (1.0 - s1)).abs() < 1e-14);
}

#[test]
fn corrected_d021_protection_multiplier() {
    let mut p = SimParams::default();
    p.surface_turnover_schema = SurfaceTurnoverSchema::D021Equivalent;
    p.eps_m = D038_FROZEN_EPS_M;
    p.k_gamma_decay = D038_K_MEMBRANE_DECAY;
    // At peak interface weight the factor approaches ε_M.
    let f = surface_turnover_protection_factor(0.5, &p);
    assert!(f >= D038_FROZEN_EPS_M - 1e-12);
    assert!(f <= 1.0 + D038_FROZEN_EPS_M + 1e-12);
    let lam = surface_turnover_lambda(0.5, &p);
    assert!((lam - D038_K_MEMBRANE_DECAY * f).abs() < 1e-15);
}

#[test]
fn no_duplicated_delta_in_schema2_loss() {
    let g1 = gate1_corrected_bulk_surface_equivalence();
    assert!(g1.no_duplicated_delta);
    for s in &g1.samples {
        let schema1 = D038_K_MEMBRANE_DECAY * s.mass_surface;
        assert!(
            s.l_surface < schema1 * 0.99,
            "schema2 must not equal full k·S (would imply missing protection or δ duplicate)"
        );
    }
}

#[test]
fn matched_bulk_surface_loss_within_5pct() {
    let g1 = gate1_corrected_bulk_surface_equivalence();
    assert!(
        g1.all_pass,
        "Gate1 failed: max_rel={} conclusion={}",
        g1.max_relative_error,
        g1.conclusion
    );
    assert!(g1.max_relative_error <= D038_LOSS_EQUIV_RTOL);
    for s in &g1.samples {
        assert!(s.pass, "fail at R={} w={} rel={}", s.radius, s.interface_width, s.relative_error);
    }
}

#[test]
fn radius_and_interface_width_invariance() {
    let g1 = gate1_corrected_bulk_surface_equivalence();
    assert!(g1.max_rel_by_radius_spread <= 0.05);
    assert!(g1.max_rel_by_width_spread <= 0.05);
}

#[test]
fn exact_s_to_w_accounting_and_trajectories() {
    let traj = gate1_decay_trajectories();
    assert!(traj.accounting_closed);
    assert!(traj.pass, "max_rel={}", traj.max_rel_mass_diff);
    assert!(traj.max_rel_mass_diff <= 0.05);
}

#[test]
fn invariant_domain_turnover_integration_schemas() {
    let g2 = gate2_integrator_validation();
    assert!(g2.schema1_historical_ok);
    assert!(g2.schema2_d021_ok);
    assert!(g2.exact_s_to_w);
    assert!(g2.schema_mismatch_rejected);
    assert!(g2.pass);
}

#[test]
fn transient_state_balance_ineligible() {
    assert_eq!(
        transient_balance_eligibility(true),
        TransientBalanceEligibility::Ineligible
    );
    assert_eq!(
        transient_balance_eligibility(false),
        TransientBalanceEligibility::EligibleAttractorWindows
    );
}

#[test]
fn multistart_attractor_classification() {
    assert!(multistart_attractor_agree(
        &[1.0, 1.02, 0.99],
        &[0.5, 0.51, 0.49],
        &[0.4, 0.41, 0.39],
        &[0.01, 0.0105, 0.0096],
    ));
    assert!(!multistart_attractor_agree(
        &[1.0, 2.0, 1.0],
        &[0.5, 0.5, 0.5],
        &[0.4, 0.4, 0.4],
        &[0.01, 0.01, 0.01],
    ));
    assert_eq!(multistart_set().len(), 6);
}

#[test]
fn candidate_count_limits() {
    let plan = candidate_scale_plan(true);
    assert!(plan.scales.len() <= 5);
    assert_eq!(plan.max_candidates, 5);
    assert!(plan.scales.contains(&1.0));
}

#[test]
fn simplest_valid_architecture_selection() {
    assert_eq!(
        select_architecture(true, true, true),
        MembraneArchitecture::V8PassiveRenewal
    );
    assert_eq!(
        select_architecture(false, true, true),
        MembraneArchitecture::V11LinearMaturation
    );
    assert_eq!(
        select_architecture(false, false, true),
        MembraneArchitecture::V12CatalyticMaturation
    );
    assert_eq!(
        select_architecture(false, false, false),
        MembraneArchitecture::None
    );
}

#[test]
fn three_consecutive_window_rule() {
    assert!(!three_consecutive_balance(&[(1.0, 0.0), (1.0, 0.0)]));
    assert!(three_consecutive_balance(&[
        (1.0, 1e-5),
        (0.99, 0.0),
        (1.01, -1e-5),
    ]));
    // A single bad window does not invalidate later consecutive good triples.
    assert!(three_consecutive_balance(&[
        (1.0, 0.0),
        (0.90, 0.0),
        (1.0, 0.0),
        (1.0, 0.0),
        (1.0, 0.0),
    ]));
    assert!(!three_consecutive_balance(&[
        (0.90, 0.0),
        (0.91, 0.0),
        (0.92, 0.0),
    ]));
}

#[test]
fn route_selection_keeps_stage_e_blocked() {
    let g0 = gate0_preservation();
    assert!(g0.pass);
    assert!(g0.surface_turnover_transfer_defect_confirmed);
    let r = route_decision(MembraneArchitecture::V8PassiveRenewal, true, true, true);
    assert_eq!(r.stage_e_status, "BLOCKED_NOT_RECOVERED");
    assert_eq!(r.production_verdict, "REQUIRES_REMEDIATION");
    assert_eq!(r.primary_conclusion, "D038_PASSIVE_RENEWAL_RECOVERED");
}

#[test]
fn v8_schema2_records_nonzero_turnover_and_exchange() {
    use chemistry_core::d038_analysis::v8_schema2_params;
    use chemistry_core::surface_density::total_surface_mass;
    use chemistry_core::Simulation;

    let params = v8_schema2_params();
    assert_eq!(
        params.d008_stage_mode,
        chemistry_core::config::D008StageMode::ConstrainedRadius
    );
    assert_eq!(
        params.surface_turnover_schema,
        chemistry_core::config::SurfaceTurnoverSchema::D021Equivalent
    );
    assert!(params.k_gamma_decay > 0.0);
    assert!(params.k_exchange > 0.0);
    let mut sim = Simulation::new(params);
    sim.enforce_structure_constraint = true;
    sim.dt_cap = 0.005;
    let n = sim.grid.width * sim.grid.height;
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let i = idx % sim.grid.width;
        let j = idx / sim.grid.width;
        let x = i as f64 - sim.grid.cx;
        let y = j as f64 - sim.grid.cy;
        let r = (x * x + y * y).sqrt();
        let phi = 0.5 * (1.0 - ((r - 22.0) / 2.0).tanh());
        sim.fields.structure[idx] = phi;
        if phi >= 0.5 {
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.activated[idx] = 0.5;
            sim.fields.precursor[idx] = 0.05;
        }
    }
    let mut geometry =
        vec![chemistry_core::surface_density::InterfaceGeometryCell::default(); n];
    chemistry_core::surface_density::compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    chemistry_core::surface_density::seed_surface_from_gamma(
        &sim.grid,
        &geometry,
        sim.params.delta_floor,
        &mut sim.fields.membrane,
        |_, _, _| 0.4,
    );

    let s0 = total_surface_mass(&sim.grid, &sim.fields.membrane);
    sim.surface_accounting
        .begin_window_local(sim.substep, sim.sim_time);
    let mut ok = 0u64;
    for _ in 0..500 {
        if sim.step() {
            ok += 1;
        }
    }
    let wl = sim.surface_accounting.window_local();
    assert!(ok > 100);
    assert!(
        wl.gamma_decay_delta > 1e-12,
        "expected nonzero turnover under ConstrainedRadius+schema2"
    );
    assert!(s0 > 0.0);
}
