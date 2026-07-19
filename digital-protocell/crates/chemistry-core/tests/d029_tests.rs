//! D-029 Gate 1 unit tests: schema, conservation, dissipation, atomicity.
use chemistry_core::config::{EquationVersion, SimParams, DX, GRID_HEIGHT, GRID_WIDTH};
use chemistry_core::d029_analysis::{
    fit_exchange_nnls, generate_exchange_candidates, leave_one_out_stable, ExchangeBasisRow,
    D029_MAX_CANDIDATES,
};
use chemistry_core::grid::Grid;
use chemistry_core::surface_density::{
    circular_phi_profile, compute_interface_geometry, evolve_surface_density, exchange_activities,
    exchange_affinity, exchange_dissipation_density, exchange_mobility, exchange_rate_j,
    seed_surface_from_gamma, total_surface_mass, validate_exchange_cell, ExchangeReject,
    InterfaceGeometryCell, EXCHANGE_BOUND_TOLERANCE,
};
use chemistry_core::Simulation;

fn v8_params(k_exchange: f64, k_eq: f64) -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    p.k_exchange = k_exchange;
    p.k_exchange_eq = k_eq;
    p.p_reference = 1.0;
    p.k_ads = 0.0;
    p.gamma_max = 1.0;
    p.gamma_reference = 1.0;
    p.k_gamma_decay = 0.05;
    p.d_gamma = 0.0;
    p.k_precursor = 0.0;
    p.k_precursor_decay = 0.0;
    p
}

fn v7_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    p.k_ads = 0.2;
    p.gamma_max = 1.0;
    p.gamma_reference = 1.0;
    p
}

fn tiny_interface(
    params: &SimParams,
    theta: f64,
    precursor: f64,
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
        theta * params.gamma_max
    });
    let mut catalyst = vec![0.0; n];
    let mut activated = vec![0.0; n];
    let mut precursor_f = vec![0.0; n];
    let mut waste = vec![0.0; n];
    for idx in 0..n {
        if grid.in_dish(idx) {
            catalyst[idx] = 0.4;
            precursor_f[idx] = precursor;
        }
    }
    let gamma = vec![0.0; n];
    let diffusion = vec![0.0; n];
    (
        grid,
        phi,
        catalyst,
        activated,
        precursor_f,
        s,
        waste,
        geometry,
        gamma,
        diffusion,
    )
}

#[test]
fn v8_equation_dispatch_and_schema() {
    let v7 = EquationVersion::MembraneMetabolismV7SurfaceDensity;
    let v8 = EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    assert_eq!(
        v8.as_str(),
        "membrane_metabolism_v8_reversible_surface_exchange"
    );
    assert!(v8.is_surface_density());
    assert!(v8.is_reversible_surface_exchange());
    assert!(!v7.is_reversible_surface_exchange());
    assert_eq!(v7.surface_exchange_schema_version(), 1);
    assert_eq!(v8.surface_exchange_schema_version(), 2);
    assert_eq!(v8.surface_density_schema_version(), 1);
}

#[test]
fn v7_cannot_silently_resume_as_v8() {
    let sim7 = Simulation::new(v7_params());
    let snap7 = sim7.snapshot();
    assert!(snap7.can_resume_into(&v7_params()).is_ok());
    assert!(snap7.can_resume_into(&v8_params(0.1, 1.0)).is_err());

    let sim8 = Simulation::new(v8_params(0.1, 1.0));
    let snap8 = sim8.snapshot();
    assert!(snap8.can_resume_into(&v8_params(0.1, 1.0)).is_ok());
    assert!(snap8.can_resume_into(&v7_params()).is_err());
}

#[test]
fn forward_adsorption_conserves_p_to_s() {
    let params = v8_params(0.5, 2.0);
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_interface(&params, 0.1, 1.0);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let p_before: f64 = precursor.iter().sum();
    let s_before = total_surface_mass(&grid, &s);
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
    .expect("evolve");
    let p_after: f64 = p_next.iter().sum();
    let s_after = total_surface_mass(&grid, &s_next);
    let cell = DX * DX;
    let dp = (p_after - p_before) * cell;
    let ds = s_after - s_before;
    assert!((dp + ds).abs() < 1e-9, "dp={dp} ds={ds}");
    assert!(totals.exchange_net > 0.0, "expected adsorption");
    assert!((totals.exchange_net - totals.precursor_to_surface).abs() < 1e-12);
    assert_eq!(totals.surface_to_waste, 0.0);
}

#[test]
fn reverse_desorption_conserves_s_to_p() {
    let params = v8_params(0.5, 0.1);
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_interface(&params, 0.9, 0.05);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let p_before: f64 = precursor.iter().sum();
    let s_before = total_surface_mass(&grid, &s);
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
    .expect("evolve");
    let p_after: f64 = p_next.iter().sum();
    let s_after = total_surface_mass(&grid, &s_next);
    let cell = DX * DX;
    let dp = (p_after - p_before) * cell;
    let ds = s_after - s_before;
    assert!((dp + ds).abs() < 1e-9, "dp={dp} ds={ds}");
    assert!(
        totals.exchange_net < 0.0,
        "expected desorption, got {}",
        totals.exchange_net
    );
    assert_eq!(totals.surface_to_waste, 0.0);
}

#[test]
fn turnover_s_to_w_separate_from_exchange() {
    let params = v8_params(0.0, 1.0);
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_interface(&params, 0.5, 0.5);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let w_before: f64 = waste.iter().sum();
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
    .expect("evolve");
    let w_after: f64 = waste.iter().sum();
    assert!(totals.exchange_net.abs() < 1e-15);
    assert!(totals.surface_to_waste > 0.0);
    assert!((w_after - w_before) > 0.0);
}

#[test]
fn exchange_equilibrium_zero_net() {
    let k_eq = 2.0;
    let p = 0.5;
    let theta = (k_eq * p) / (1.0 + k_eq * p);
    let params = v8_params(1.0, k_eq);
    let (a_f, a_r) = exchange_activities(p, theta * params.gamma_max, &params);
    assert!((a_f - a_r).abs() < 1e-12, "a_f={a_f} a_r={a_r}");
    let (j, ..) = exchange_rate_j(p, 0.4, theta * params.gamma_max, &params);
    assert!(j.abs() < 1e-12);
}

#[test]
fn affinity_sign_and_nonnegative_dissipation() {
    let params = v8_params(1.0, 2.0);
    let (af, ar) = exchange_activities(1.0, 0.1, &params);
    let aff = exchange_affinity(af, ar);
    let (j, ..) = exchange_rate_j(1.0, 0.4, 0.1, &params);
    assert!(aff > 0.0 && j > 0.0);
    assert!(j * aff >= -1e-12);
    let m = exchange_mobility(0.4, &params);
    assert!(exchange_dissipation_density(af, ar, m) >= -1e-12);

    let (af2, ar2) = exchange_activities(0.05, 0.9, &params);
    let aff2 = exchange_affinity(af2, ar2);
    let (j2, ..) = exchange_rate_j(0.05, 0.4, 0.9, &params);
    assert!(aff2 < 0.0 && j2 < 0.0);
    assert!(j2 * aff2 >= -1e-12);
}

#[test]
fn zero_mobility_no_exchange() {
    let params = v8_params(0.0, 5.0);
    let (j, jf, jr, ..) = exchange_rate_j(1.0, 0.4, 0.1, &params);
    assert_eq!(j, 0.0);
    assert_eq!(jf, 0.0);
    assert_eq!(jr, 0.0);
}

#[test]
fn positivity_and_capacity_rejection() {
    assert_eq!(
        validate_exchange_cell(-1e-6, 0.1, 0.5, 1.0, 1e-12, 1.0, 1.0, 0.0),
        Err(ExchangeReject::NegPrecursor)
    );
    assert_eq!(
        validate_exchange_cell(0.1, -1e-6, 0.5, 1.0, 1e-12, 1.0, 1.0, 0.0),
        Err(ExchangeReject::NegSurface)
    );
    assert_eq!(
        validate_exchange_cell(0.1, 1.1, 0.5, 1.0, 1e-12, 1.0, 1.0, 0.0),
        Err(ExchangeReject::CapacityExceeded)
    );
    let _ = EXCHANGE_BOUND_TOLERANCE;
}

#[test]
fn atomic_reject_on_oversized_transfer() {
    // Historical V1 explicit Euler rejects oversized proposals; V2 keeps them bounded.
    let mut params = v8_params(1000.0, 10.0);
    params.surface_exchange_integrator =
        chemistry_core::config::SurfaceExchangeIntegrator::ExplicitEulerV1;
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_interface(&params, 0.99, 10.0);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let result = evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &params,
        10.0,
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
    );
    assert!(result.is_err(), "expected reject on oversized transfer");
}

#[test]
fn two_parameter_fit_rank_conditioning_loo() {
    let rows: Vec<ExchangeBasisRow> = (0..6)
        .map(|i| {
            let a = 1.0 + 0.3 * i as f64;
            let b = 0.4 + 0.15 * i as f64;
            ExchangeBasisRow {
                label: format!("s{i}"),
                a_integral: a,
                b_integral: b,
                l_turnover: 3.0 * a - 1.5 * b,
                finite: true,
            }
        })
        .collect();
    let fit = fit_exchange_nnls(&rows);
    assert_eq!(fit.rank, 2);
    assert!(fit.condition_number < 1e6);
    assert!(fit.identifiable, "{fit:?}");
    let (loo_ok, _) = leave_one_out_stable(&rows, &fit);
    assert!(loo_ok);
    assert_eq!(generate_exchange_candidates(&fit).len(), D029_MAX_CANDIDATES);
}
