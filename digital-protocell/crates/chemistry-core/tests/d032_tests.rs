//! D-032 Gate 1: activated surface assembly conservation and causality.
use chemistry_core::config::{
    EquationVersion, SimParams, SurfaceExchangeIntegrator, GRID_HEIGHT, GRID_WIDTH,
};
use chemistry_core::d029_analysis::apply_exchange_candidate;
use chemistry_core::d031_analysis::d030_identified_candidate;
use chemistry_core::d032_analysis::{
    active_material_residual, combined_boundary_inward, frozen_exchange_kinetics_ok,
    generate_active_candidates, reconstruct_active_rate, v8_passive_only_params, v9_params,
    ActiveRateEstimate, PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT, D032_ALPHA_FROZEN,
    D032_BETA_FROZEN,
};
use chemistry_core::grid::Grid;
use chemistry_core::surface_density::{
    active_assembly_rate_j, apply_active_assembly_bounded, circular_phi_profile,
    compute_interface_geometry, evolve_surface_density, seed_surface_from_gamma,
    InterfaceGeometryCell, SURFACE_EXCHANGE_INTEGRATOR_V2,
};

fn v9_test_params(k_active: f64) -> SimParams {
    let mut p = v9_params(k_active);
    p.k_gamma_decay = 0.0;
    p.d_gamma = 0.0;
    p.k_precursor = 0.0;
    p.k_precursor_decay = 0.0;
    p.reactions_enabled = false;
    p
}

fn tiny_state(
    params: &SimParams,
    theta0: f64,
    p0: f64,
    a0: f64,
) -> (
    Grid,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Vec<InterfaceGeometryCell>,
    Vec<f64>,
    Vec<f64>,
) {
    let grid = Grid::new();
    let n = GRID_WIDTH * GRID_HEIGHT;
    let mut phi = vec![0.0; n];
    circular_phi_profile(&grid, 10.0, 2.0, &mut phi);
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |_, _, _| {
        theta0 * params.gamma_max
    });
    let mut catalyst = vec![0.0; n];
    let mut activated = vec![0.0; n];
    let mut precursor = vec![0.0; n];
    let waste = vec![0.0; n];
    for idx in 0..n {
        if grid.in_dish(idx) {
            catalyst[idx] = 0.4;
            precursor[idx] = p0;
            activated[idx] = a0;
        }
    }
    (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        waste,
        geometry,
        vec![0.0; n],
        vec![0.0; n],
    )
}

#[test]
fn v9_dispatch_and_schema() {
    let p = v9_params(1.0);
    assert_eq!(
        p.equation_version.as_str(),
        "membrane_metabolism_v9_activated_surface_assembly"
    );
    assert!(p.equation_version.is_activated_surface_assembly());
    assert!(p.equation_version.is_reversible_surface_exchange());
    assert!(p.equation_version.is_surface_density());
    assert_eq!(p.equation_version.surface_exchange_schema_version(), 3);
    assert_eq!(p.equation_version.active_assembly_schema_version(), 1);
    assert_eq!(
        SurfaceExchangeIntegrator::InvariantDomainV2.as_str(),
        SURFACE_EXCHANGE_INTEGRATOR_V2
    );
    assert!(frozen_exchange_kinetics_ok());
    assert!((D032_ALPHA_FROZEN - 0.167).abs() < 5e-3);
    assert!((D032_BETA_FROZEN - 0.00334).abs() < 5e-5);
    assert_eq!(
        PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT,
        "PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT_FOR_MEMBRANE_MAINTENANCE"
    );
}

#[test]
fn active_zero_without_a_or_p_or_capacity_or_q() {
    let params = v9_test_params(1.0);
    let g = 0.3;
    assert_eq!(
        active_assembly_rate_j(1.0, 0.0, 0.4, g, &params),
        0.0,
        "no A"
    );
    assert_eq!(
        active_assembly_rate_j(0.0, 1.0, 0.4, g, &params),
        0.0,
        "no P"
    );
    assert_eq!(
        active_assembly_rate_j(1.0, 1.0, 0.4, params.gamma_max, &params),
        0.0,
        "theta=1"
    );
    assert_eq!(
        active_assembly_rate_j(1.0, 1.0, 0.0, g, &params),
        0.0,
        "q(C)=0"
    );
    assert!(active_assembly_rate_j(1.0, 1.0, 0.4, g, &params) > 0.0);
}

#[test]
fn active_stoichiometry_and_activation_accounting() {
    let params = v9_test_params(2.0);
    let p0 = 0.5;
    let a0 = 0.4;
    let s0 = 0.1;
    let w0 = 0.0;
    let d = 0.5;
    let (residual, r) = active_material_residual(p0, a0, s0, w0, d, 0.4, 0.01, &params);
    assert!(r > 0.0, "r={r}");
    assert!(residual.abs() < 1e-12, "residual={residual}");
    let (p1, a1, s1, dw, r2) = apply_active_assembly_bounded(p0, a0, s0, d, 0.4, 0.01, &params);
    assert!((r2 - r).abs() < 1e-15);
    assert!((p0 - p1 - r).abs() < 1e-12, "P loss");
    assert!((a0 - a1 - r).abs() < 1e-12, "A loss = activation");
    assert!((s1 - s0 - r).abs() < 1e-12, "S gain");
    assert!((dw - r).abs() < 1e-12, "W gain");
}

#[test]
fn active_respects_available_bounds() {
    let params = v9_test_params(1000.0);
    // Tiny P and A, near capacity — transfer must not exceed any bound.
    let d = 0.5;
    let s0 = d * params.gamma_max - 1e-4;
    let (p1, a1, s1, _, r) =
        apply_active_assembly_bounded(1e-5, 1e-5, s0, d, 0.4, 1.0, &params);
    assert!(p1 >= -1e-15 && a1 >= -1e-15 && s1 <= d * params.gamma_max + 1e-12);
    assert!(r <= 1e-5 + 1e-18);
    assert!(r <= (d * params.gamma_max - s0) + 1e-18);
}

#[test]
fn passive_desorption_returns_s_to_p_only_and_turnover_to_w() {
    // Passive desorption: S→P via reverse exchange; no W from exchange.
    let mut params = v8_passive_only_params();
    params.k_gamma_decay = 0.0;
    params.d_gamma = 0.0;
    params.reactions_enabled = false;
    let (grid, phi, catalyst, activated, precursor, s, mut waste, mut geometry, mut gamma, mut diffusion) =
        tiny_state(&params, 0.9, 0.0, 0.0);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let w0: f64 = waste.iter().sum();
    let totals = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params,
        0.01,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diffusion,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut waste,
        None,
        None,
    )
    .expect("passive step");
    let w1: f64 = waste.iter().sum();
    assert!(totals.exchange_reverse > 0.0 || totals.exchange_net < 0.0);
    assert!((w1 - w0).abs() < 1e-12, "passive creates no W");

    // Turnover-only: S→W, no P gain from turnover.
    let mut params_t = v9_test_params(0.0);
    params_t.equation_version = EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    apply_exchange_candidate(&mut params_t, &d030_identified_candidate());
    params_t.k_exchange = 0.0;
    params_t.k_gamma_decay = 0.1;
    let (grid, phi, catalyst, activated, precursor, s, mut waste, mut geometry, mut gamma, mut diffusion) =
        tiny_state(&params_t, 0.6, 0.0, 0.0);
    let p0: f64 = precursor.iter().sum();
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let totals = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params_t,
        0.01,
        false,
        false,
        false,
        true,
        false,
        &mut geometry,
        &mut gamma,
        &mut diffusion,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut waste,
        None,
        None,
    )
    .expect("turnover");
    let p1: f64 = p_next.iter().sum();
    assert!(totals.gamma_decay_delta > 0.0);
    assert!((p1 - p0).abs() < 1e-12, "turnover does not create P");
}

#[test]
fn combined_field_inward_at_physical_boundaries() {
    let params = v9_test_params(1.0);
    assert!(chemistry_core::d032_analysis::combined_domain_corners_ok(
        &params
    ));
    assert!(combined_boundary_inward(0.0, 1.0, 0.2, 0.5, 0.4, &params));
    assert!(combined_boundary_inward(1.0, 0.0, 0.2, 0.5, 0.4, &params));
    assert!(combined_boundary_inward(1.0, 1.0, 0.0, 0.5, 0.4, &params));
    assert!(combined_boundary_inward(
        1.0,
        1.0,
        0.5,
        0.5,
        0.4,
        &params
    ));
}

#[test]
fn v9_evolve_consumes_a_and_conserves_paw() {
    let params = v9_test_params(1.0);
    let (grid, phi, catalyst, activated, precursor, s, mut waste, mut geometry, mut gamma, mut diffusion) =
        tiny_state(&params, 0.3, 0.5, 0.5);
    let p0: f64 = precursor.iter().sum();
    let a0: f64 = activated.iter().sum();
    let s0: f64 = s.iter().sum();
    let w0: f64 = waste.iter().sum();
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let totals = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params,
        0.005,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diffusion,
        &mut s_next,
        &mut a_next,
        &mut p_next,
        &mut waste,
        None,
        None,
    )
    .expect("v9 step");
    assert!(totals.active_assembly > 0.0, "active assembly active");
    assert!(
        (totals.active_assembly - totals.active_assembly_activation).abs() < 1e-12
    );
    let p1: f64 = p_next.iter().sum();
    let a1: f64 = a_next.iter().sum();
    let s1: f64 = s_next.iter().sum();
    let w1: f64 = waste.iter().sum();
    // Net Δ from active + passive exchange: P+S conserved by exchange; active adds W and removes A.
    let material = (p1 - p0) + (a1 - a0) + (s1 - s0) + (w1 - w0);
    assert!(material.abs() < 1e-9, "material residual {material}");
    assert!((a0 - a1 - totals.active_assembly_activation).abs() < 1e-9);
}

#[test]
fn d031_passive_only_reproducible_on_v8() {
    let p = v8_passive_only_params();
    assert_eq!(
        p.equation_version,
        EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange
    );
    assert_eq!(p.k_active, 0.0);
    assert!(p.k_exchange > 0.0);
}

#[test]
fn candidate_generation_and_reconstruction_helpers() {
    let cands = generate_active_candidates(0.2);
    assert_eq!(cands.len(), 3);
    assert!((cands[0].k_active - 0.1).abs() < 1e-12);
    assert!((cands[1].k_active - 0.2).abs() < 1e-12);
    assert!((cands[2].k_active - 0.4).abs() < 1e-12);

    let estimates: Vec<ActiveRateEstimate> = (0..6)
        .map(|i| ActiveRateEstimate {
            state_id: format!("s{i}"),
            accepted_steps: 10_000 * (i as u64 + 1),
            biological_turnover: 1.0,
            passive_net_exchange: -0.5,
            r_required: 1.5,
            b_active: 1.0,
            k_active_required: 0.2 * (1.0 + 0.05 * (i as f64 - 2.5)),
            valid: true,
            reject_reason: String::new(),
        })
        .collect();
    let rec = reconstruct_active_rate(estimates);
    assert!(rec.portable, "{rec:?}");
    assert!(rec.median_k_active.is_finite());
}
