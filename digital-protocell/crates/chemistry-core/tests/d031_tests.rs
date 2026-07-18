//! D-031 Gate 0–1: invariant-domain exchange integrator.
use chemistry_core::config::{
    EquationVersion, SimParams, SurfaceExchangeIntegrator, GRID_HEIGHT, GRID_WIDTH,
};
use chemistry_core::d029_analysis::apply_exchange_candidate;
use chemistry_core::d031_analysis::{
    d030_identified_candidate, exchange_f_is_monotone_decreasing, D031_ALPHA_FROZEN,
    D031_BETA_FROZEN,
};
use chemistry_core::grid::Grid;
use chemistry_core::surface_density::{
    apply_turnover_exact, circular_phi_profile, classify_exchange_invariant_field,
    compute_interface_geometry, evolve_surface_density, exchange_scalar_f, propose_explicit_exchange,
    reconstruct_gamma, seed_surface_from_gamma, solve_exchange_backward_euler,
    surface_occupancy_theta, ExchangeReject, InterfaceGeometryCell, SURFACE_EXCHANGE_INTEGRATOR_V2,
};

fn identified_params(integrator: SurfaceExchangeIntegrator) -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    apply_exchange_candidate(&mut p, &d030_identified_candidate());
    p.surface_exchange_integrator = integrator;
    p.k_gamma_decay = 0.0;
    p.d_gamma = 0.0;
    p.reactions_enabled = false;
    p
}

fn tiny_state(params: &SimParams, theta0: f64, p0: f64) -> (
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
    let activated = vec![0.0; n];
    let mut precursor = vec![0.0; n];
    let waste = vec![0.0; n];
    for idx in 0..n {
        if grid.in_dish(idx) {
            catalyst[idx] = 0.4;
            precursor[idx] = p0;
        }
    }
    let gamma = vec![0.0; n];
    let diffusion = vec![0.0; n];
    (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        waste,
        geometry,
        gamma,
        diffusion,
    )
}

#[test]
fn integrator_schema_identity() {
    assert_eq!(
        SurfaceExchangeIntegrator::InvariantDomainV2.as_str(),
        SURFACE_EXCHANGE_INTEGRATOR_V2
    );
    assert!((D031_ALPHA_FROZEN - 0.167).abs() < 5e-3);
    assert!((D031_BETA_FROZEN - 0.00334).abs() < 5e-5);
}

#[test]
fn continuous_boundary_derivatives_point_inward() {
    let params = identified_params(SurfaceExchangeIntegrator::ExplicitEulerV1);
    let signs = classify_exchange_invariant_field(0.2, 0.5, 0.4, 0.8, &params);
    assert!(signs.dp_at_p0 >= -1e-14, "{signs:?}");
    assert!(signs.ds_at_s0 >= -1e-14, "{signs:?}");
    assert!(signs.ds_at_theta1 <= 1e-14, "{signs:?}");
    assert!(signs.continuous_inward, "{signs:?}");
}

#[test]
fn scalar_exchange_f_monotone_decreasing() {
    let params = identified_params(SurfaceExchangeIntegrator::InvariantDomainV2);
    assert!(exchange_f_is_monotone_decreasing(
        1.0, 0.5, 0.5, 0.7, &params, 64
    ));
}

#[test]
fn unique_backward_euler_root_and_conservation() {
    let params = identified_params(SurfaceExchangeIntegrator::InvariantDomainV2);
    let c_surface = 0.4;
    let t = 1.2;
    let s_old = 0.1;
    let info = solve_exchange_backward_euler(
        s_old,
        t,
        c_surface,
        0.4,
        0.8,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
        0.05,
    )
    .expect("solve");
    assert!(info.s_next >= 0.0 && info.s_next <= c_surface.min(t) + 1e-14);
    assert!((info.p_next + info.s_next - t).abs() < 1e-12);
    assert!(info.iterations > 0 || info.residual < 1e-12);
}

#[test]
fn exact_capacity_moves_inward() {
    let params = identified_params(SurfaceExchangeIntegrator::InvariantDomainV2);
    let c_surface = 0.5;
    let t = 2.0;
    let s_old = c_surface; // θ=1
    let f = exchange_scalar_f(
        s_old,
        t,
        c_surface,
        0.5,
        1.0,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
    );
    assert!(f <= 1e-14, "f={f}");
    let info = solve_exchange_backward_euler(
        s_old,
        t,
        c_surface,
        0.5,
        1.0,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
        1.0,
    )
    .unwrap();
    assert!(info.s_next <= s_old + 1e-12);
}

#[test]
fn zero_p_and_zero_s_move_inward() {
    let params = identified_params(SurfaceExchangeIntegrator::InvariantDomainV2);
    let c_surface = 0.5;
    // S=0, T=P>0 ⇒ adsorption
    let f0 = exchange_scalar_f(
        0.0,
        1.0,
        c_surface,
        0.5,
        1.0,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
    );
    assert!(f0 >= -1e-14);
    // P=0 ⇒ T=S
    let fs = exchange_scalar_f(
        0.2,
        0.2,
        c_surface,
        0.5,
        1.0,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
    );
    assert!(fs <= 1e-14);
}

#[test]
fn turnover_exact_conserves_s_plus_w() {
    let (s_after, dw) = apply_turnover_exact(1.25, 0.03, 0.5);
    assert!(s_after >= 0.0);
    assert!((s_after + dw - 1.25).abs() < 1e-14);
}

#[test]
fn large_dt_invariant_remains_bounded_no_clip() {
    let params = identified_params(SurfaceExchangeIntegrator::InvariantDomainV2);
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
    ) = tiny_state(&params, 0.99, 10.0);
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
    )
    .expect("V2 must accept large dt");
    for idx in 0..s_next.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= params.delta_floor {
            continue;
        }
        let g = reconstruct_gamma(s_next[idx], d, params.delta_floor);
        let th = surface_occupancy_theta(g, params.gamma_max);
        assert!(p_next[idx] >= -1e-12, "P={}", p_next[idx]);
        assert!(s_next[idx] >= -1e-12, "S={}", s_next[idx]);
        assert!(th <= 1.0 + 1e-12, "θ={th}");
    }
    assert!((totals.exchange_net).is_finite());
}

#[test]
fn explicit_v1_oversized_rejects_capacity() {
    let mut params = identified_params(SurfaceExchangeIntegrator::ExplicitEulerV1);
    params.k_exchange = 1000.0;
    let (p_n, s_n, ..) = propose_explicit_exchange(10.0, 0.49, 0.5, 0.4, 10.0, &params);
    let g_n = reconstruct_gamma(s_n, 0.5, params.delta_floor);
    let th = surface_occupancy_theta(g_n, params.gamma_max);
    assert!(th > 1.0 + 1e-12 || p_n < 0.0, "θ={th} P={p_n}");
}

#[test]
fn small_dt_v2_matches_explicit_v1() {
    let p_v1 = identified_params(SurfaceExchangeIntegrator::ExplicitEulerV1);
    let p_v2 = identified_params(SurfaceExchangeIntegrator::InvariantDomainV2);
    let dt = 1e-5;
    let (
        grid,
        phi,
        catalyst,
        activated,
        precursor,
        s,
        mut waste1,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = tiny_state(&p_v1, 0.3, 0.5);
    let mut waste2 = waste1.clone();
    let mut geom2 = geometry.clone();
    let mut gamma2 = gamma.clone();
    let mut diff2 = diffusion.clone();
    let mut s1 = s.clone();
    let mut s2 = s.clone();
    let mut a1 = activated.clone();
    let mut a2 = activated.clone();
    let mut pr1 = precursor.clone();
    let mut pr2 = precursor.clone();
    evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &p_v1,
        dt,
        false,
        true,
        false,
        false,
        false,
        &mut geometry,
        &mut gamma,
        &mut diffusion,
        &mut s1,
        &mut a1,
        &mut pr1,
        &mut waste1,
    )
    .unwrap();
    evolve_surface_density(
        &grid,
        &phi,
        &catalyst,
        &activated,
        &precursor,
        &s,
        &p_v2,
        dt,
        false,
        true,
        false,
        false,
        false,
        &mut geom2,
        &mut gamma2,
        &mut diff2,
        &mut s2,
        &mut a2,
        &mut pr2,
        &mut waste2,
    )
    .unwrap();
    let mut max_ds: f64 = 0.0;
    for i in 0..s.len() {
        max_ds = max_ds.max((s1[i] - s2[i]).abs());
        max_ds = max_ds.max((pr1[i] - pr2[i]).abs());
    }
    assert!(max_ds < 1e-6, "max_ds={max_ds}");
}

#[test]
fn strang_turnover_plus_exchange_conserves_psw() {
    let mut params = identified_params(SurfaceExchangeIntegrator::InvariantDomainV2);
    params.k_gamma_decay = 0.02;
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
    ) = tiny_state(&params, 0.4, 0.6);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let p0: f64 = precursor.iter().sum();
    let s0: f64 = s.iter().sum();
    let w0: f64 = waste.iter().sum();
    evolve_surface_density(
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
    )
    .unwrap();
    let p1: f64 = p_next.iter().sum();
    let s1: f64 = s_next.iter().sum();
    let w1: f64 = waste.iter().sum();
    assert!((p1 + s1 + w1 - (p0 + s0 + w0)).abs() < 1e-9);
}

#[test]
fn equilibrium_stationary_under_invariant_solve() {
    let params = identified_params(SurfaceExchangeIntegrator::InvariantDomainV2);
    // θ_eq from K p (1-θ)=θ ⇒ θ = Kp/(1+Kp)
    let p0 = 0.1;
    let k = params.k_exchange_eq;
    let theta_eq = (k * p0) / (1.0 + k * p0);
    let c_surface = 0.5;
    let s_eq = theta_eq * c_surface;
    let t = p0 + s_eq;
    let info = solve_exchange_backward_euler(
        s_eq,
        t,
        c_surface,
        0.5,
        1.0,
        params.k_exchange,
        params.k_exchange_eq,
        params.p_reference,
        params.gamma_max,
        1.0,
    )
    .unwrap();
    assert!((info.s_next - s_eq).abs() < 1e-9, "{info:?}");
}

#[test]
fn atomic_reject_still_available_on_v1() {
    let mut params = identified_params(SurfaceExchangeIntegrator::ExplicitEulerV1);
    params.k_exchange = 1000.0;
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
    ) = tiny_state(&params, 0.99, 10.0);
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
    );
    assert_eq!(result, Err(ExchangeReject::CapacityExceeded));
    // Atomic: buffers unchanged on reject path is enforced by caller; here we only require Err.
}
