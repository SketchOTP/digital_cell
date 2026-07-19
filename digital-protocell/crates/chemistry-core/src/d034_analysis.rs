//! D-034 surface-bound membrane maturation: dual-surface (U/S) rate ID and helpers.
//!
//! Fields: soluble precursor `P`, immature surface `U = δΓ_U`, mature surface `S = δΓ_S`.
//! Passive exchange is `P↔U` (not `P↔S`) with the frozen D-030/D-031 α/β identity; mature
//! `S` never desorbs. Maturation `U + A → S + W` converts immature into mature in place.

use crate::config::{EquationVersion, SimParams, SurfaceExchangeIntegrator};
use crate::d029_analysis::{apply_exchange_candidate, ExchangeCandidate};
use crate::d031_analysis::{d030_identified_candidate, D031_ALPHA_FROZEN, D031_BETA_FROZEN};
use crate::surface_density::apply_maturation_bounded;

pub use crate::d031_analysis::{
    D031_ALPHA_FROZEN as D034_ALPHA_FROZEN, D031_BETA_FROZEN as D034_BETA_FROZEN,
};

/// A dissolved (bulk) activated intermediate is rejected for D-034: the intermediate is a
/// surface-bound immature membrane `U`, never a soluble species.
pub const SOLUBLE_ACTIVATED_INTERMEDIATE_REJECTED: &str =
    "SOLUBLE_ACTIVATED_INTERMEDIATE_REJECTED_USE_SURFACE_BOUND_IMMATURE_MEMBRANE";

/// Frozen D-030/D-031 exchange candidate (passive P↔U reuses the identified α, β).
pub fn d034_frozen_exchange_candidate() -> ExchangeCandidate {
    d030_identified_candidate()
}

/// Build v11 params with frozen passive exchange and the chosen maturation rate.
pub fn v11_params(k_mature: f64) -> SimParams {
    let mut p = SimParams::default();
    apply_exchange_candidate(&mut p, &d034_frozen_exchange_candidate());
    p.equation_version = EquationVersion::MembraneMetabolismV11SurfaceMaturation;
    p.surface_exchange_integrator = SurfaceExchangeIntegrator::InvariantDomainV2;
    p.a_reference = 1.0;
    p.p_reference = 1.0;
    p.k_active = 0.0;
    p.k_charge = 0.0;
    p.k_insert = 0.0;
    p.k_relax = 0.0;
    p.k_mature = k_mature;
    // Immature surface diffuses tangentially with the mature-surface coefficient by default.
    p.d_u = p.d_gamma;
    p.reactions_enabled = true;
    p
}

/// Frozen α = k_exchange·K_exchange, β = k_exchange identity check for the passive P↔U law.
pub fn d034_frozen_exchange_kinetics_ok() -> bool {
    let c = d034_frozen_exchange_candidate();
    let alpha = c.k_exchange * c.k_exchange_eq;
    let beta = c.k_exchange;
    ((alpha - D031_ALPHA_FROZEN) / D031_ALPHA_FROZEN).abs() < 1e-12
        && ((beta - D031_BETA_FROZEN) / D031_BETA_FROZEN).abs() < 1e-12
}

/// Material residual of one bounded maturation step: ΔU + ΔA + ΔS + ΔW should be 0.
///
/// Returns `(residual, r)` where `r` is the maturation extent.
pub fn maturation_material_residual(
    u0: f64,
    a0: f64,
    s0: f64,
    w0: f64,
    delta: f64,
    catalyst: f64,
    dt: f64,
    params: &SimParams,
) -> (f64, f64) {
    let (u1, a1, s1, dw, r) = apply_maturation_bounded(u0, a0, s0, delta, catalyst, dt, params);
    let w1 = w0 + dw;
    let residual = (u1 - u0) + (a1 - a0) + (s1 - s0) + (w1 - w0);
    (residual, r)
}

use crate::config::DX;
use crate::d030_analysis::{
    adsorption_matrix_specs, catalyst_for_q, desorption_matrix_specs, estimate_alpha_from_step,
    estimate_beta_from_step, isotherm_ratio, recover_exchange_parameters, robust_median,
    OrthogonalAssaySpec, D030_ADS_THETA_MAX, D030_EQ_ISOTHERM_REL_MAX, D030_MIXED_FLUX_REL_MAX,
};
use crate::grid::Grid;
use crate::membrane::membrane_catalyst_saturation;
use crate::surface_density::{
    circular_phi_profile, compute_interface_geometry, evolve_surface_density,
    reconstruct_gamma_field, seed_surface_from_gamma, surface_occupancy_theta, total_surface_mass,
    InterfaceGeometryCell, SurfaceAccountingTotals,
};
use crate::Simulation;
use serde::{Deserialize, Serialize};

/// Gate 2: α/β relative tolerance vs frozen identity.
pub const D034_EXCHANGE_REL_TOL: f64 = 0.02;
/// Gate 4: orthogonal maturation rate recovery tolerance.
pub const D034_MATURATION_RATE_TOL: f64 = 0.15;
/// Gate 6: portable k_mature span ceiling.
pub const D034_PORTABILITY_SPAN_MAX: f64 = 3.0;
/// Gate 6: leave-one-out median relative tolerance.
pub const D034_LOO_MEDIAN_REL_MAX: f64 = 0.50;
/// Gate 6: minimum valid renewal-state estimates.
pub const D034_MIN_VALID_STATES: usize = 5;
/// Gate 7: candidate scale factors (max 5 total).
pub const D034_CANDIDATE_SCALES: [f64; 3] = [0.5, 1.0, 2.0];
pub const D034_MAX_MATURATION_CANDIDATES: usize = 5;
pub const D034_BASIS_EPS: f64 = 1e-18;

/// Planted maturation rate for Gate 4 orthogonal identification assays.
pub const D034_ASSAY_K_MATURE: f64 = 2.0;

/// v11 passive P↔U exchange only (no maturation, turnover, or bulk chemistry).
pub fn v11_exchange_only_params() -> SimParams {
    let mut p = v11_params(0.0);
    p.k_gamma_decay = 0.0;
    p.d_gamma = 0.0;
    p.d_u = 0.0;
    p.k_precursor = 0.0;
    p.k_precursor_decay = 0.0;
    p.reactions_enabled = false;
    p
}

/// v11 maturation-only params (passive exchange off).
pub fn v11_maturation_only_params(k_mature: f64) -> SimParams {
    let mut p = v11_params(k_mature);
    p.k_exchange = 0.0;
    p.k_gamma_decay = 0.0;
    p.d_gamma = 0.0;
    p.d_u = 0.0;
    p.k_precursor = 0.0;
    p.k_precursor_decay = 0.0;
    p.reactions_enabled = false;
    p
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DualExchangeLocalBases {
    pub adsorption_basis: f64,
    pub desorption_basis: f64,
    pub mean_theta_u: f64,
    pub mean_theta_s: f64,
    pub mean_theta_total: f64,
    pub mean_one_minus_theta_total: f64,
    pub mean_q_c: f64,
    pub bulk_p: f64,
    pub surface_u: f64,
    pub surface_s: f64,
    pub interface_cells: usize,
}

pub fn compute_dual_exchange_local_bases(
    grid: &Grid,
    precursor: &[f64],
    catalyst: &[f64],
    u: &[f64],
    s: &[f64],
    geometry: &[InterfaceGeometryCell],
    params: &SimParams,
) -> DualExchangeLocalBases {
    let gamma_max = params.gamma_max.max(0.0);
    let pref = if params.p_reference > 0.0 {
        params.p_reference
    } else {
        1.0
    };
    let mut bases = DualExchangeLocalBases::default();
    let mut theta_u_w = 0.0;
    let mut theta_s_w = 0.0;
    let mut sat_w = 0.0;
    let mut q_w = 0.0;
    let mut wsum = 0.0;
    let mut p_sum = 0.0;
    for idx in 0..u.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        p_sum += precursor[idx].max(0.0);
        let delta = geometry[idx].delta;
        if delta <= params.delta_floor {
            continue;
        }
        let gamma_u = (u[idx].max(0.0) / delta).max(0.0);
        let gamma_s = (s[idx].max(0.0) / delta).max(0.0);
        let p = precursor[idx].max(0.0) / pref;
        let q_c = membrane_catalyst_saturation(catalyst[idx].max(0.0), params);
        let theta_u = surface_occupancy_theta(gamma_u, gamma_max);
        let theta_s = surface_occupancy_theta(gamma_s, gamma_max);
        let theta_total = (theta_u + theta_s).min(1.0);
        let sat = (1.0 - theta_total).max(0.0);
        bases.adsorption_basis += delta * gamma_max * q_c * p * sat;
        bases.desorption_basis += delta * gamma_max * q_c * theta_u;
        theta_u_w += delta * theta_u;
        theta_s_w += delta * theta_s;
        sat_w += delta * sat;
        q_w += delta * q_c;
        wsum += delta;
        bases.interface_cells += 1;
    }
    bases.bulk_p = p_sum * DX * DX;
    bases.surface_u = total_surface_mass(grid, u);
    bases.surface_s = total_surface_mass(grid, s);
    if wsum > 0.0 {
        bases.mean_theta_u = theta_u_w / wsum;
        bases.mean_theta_s = theta_s_w / wsum;
        bases.mean_theta_total = bases.mean_theta_u + bases.mean_theta_s;
        bases.mean_one_minus_theta_total = sat_w / wsum;
        bases.mean_q_c = q_w / wsum;
    }
    bases
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DualExchangeObservabilitySample {
    pub label: String,
    pub accepted_substeps: u64,
    pub dt: f64,
    pub forward_exchange: f64,
    pub reverse_exchange: f64,
    pub net_exchange: f64,
    pub bulk_p: f64,
    pub surface_u: f64,
    pub surface_s: f64,
    pub mean_theta_u: f64,
    pub mean_theta_total: f64,
    pub adsorption_basis: f64,
    pub desorption_basis: f64,
    pub exchange_dissipation: f64,
    pub exact_dp: f64,
    pub exact_du: f64,
    pub exact_ds: f64,
    pub accounting_residual: f64,
    pub alpha_estimate: f64,
    pub beta_estimate: f64,
}

pub fn sample_from_dual_step(
    label: &str,
    accepted_substeps: u64,
    dt: f64,
    bases_before: &DualExchangeLocalBases,
    totals: &SurfaceAccountingTotals,
    p_before: f64,
    u_before: f64,
    s_before: f64,
    p_after: f64,
    u_after: f64,
    s_after: f64,
) -> DualExchangeObservabilitySample {
    let exact_dp = p_after - p_before;
    let exact_du = u_after - u_before;
    let exact_ds = s_after - s_before;
    let accounting_residual = (exact_dp + exact_du).abs();
    DualExchangeObservabilitySample {
        label: label.to_string(),
        accepted_substeps,
        dt,
        forward_exchange: totals.exchange_forward,
        reverse_exchange: totals.exchange_reverse,
        net_exchange: totals.exchange_net,
        bulk_p: bases_before.bulk_p,
        surface_u: bases_before.surface_u,
        surface_s: bases_before.surface_s,
        mean_theta_u: bases_before.mean_theta_u,
        mean_theta_total: bases_before.mean_theta_total,
        adsorption_basis: bases_before.adsorption_basis,
        desorption_basis: bases_before.desorption_basis,
        exchange_dissipation: totals.exchange_dissipation,
        exact_dp,
        exact_du,
        exact_ds,
        accounting_residual,
        alpha_estimate: estimate_alpha_from_step(totals.exchange_net, dt, bases_before.adsorption_basis),
        beta_estimate: estimate_beta_from_step(totals.exchange_net, dt, bases_before.desorption_basis),
    }
}

/// Build fixed-interface dual-surface state `(grid, …, u, s, …)`.
pub fn build_dual_fixed_interface_state(
    params: &SimParams,
    radius: f64,
    theta_u: f64,
    theta_s: f64,
    precursor: f64,
    catalyst: f64,
) -> (
    Grid,
    Vec<f64>,
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
    let n = grid.width * grid.height;
    let mut phi = vec![0.0; n];
    circular_phi_profile(&grid, radius, 2.0, &mut phi);
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    let mut s = vec![0.0; n];
    let mut u = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |_, _, _| {
        theta_s * params.gamma_max
    });
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut u, |_, _, _| {
        theta_u * params.gamma_max
    });
    let mut cat = vec![0.0; n];
    let mut act = vec![0.0; n];
    let mut prec = vec![0.0; n];
    let mut waste = vec![0.0; n];
    for idx in 0..n {
        if grid.in_dish(idx) {
            cat[idx] = catalyst;
            prec[idx] = precursor;
        }
    }
    let gamma = vec![0.0; n];
    let diffusion = vec![0.0; n];
    (
        grid, phi, cat, act, prec, s, u, waste, geometry, gamma, diffusion,
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DualOrthogonalAssayResult {
    pub spec: OrthogonalAssaySpec,
    pub theta_s_fixed: f64,
    pub q_c: f64,
    pub first: DualExchangeObservabilitySample,
    pub pass_gates: bool,
    pub notes: Vec<String>,
}

/// Run one P↔U exchange-only assay on a fixed dual interface (S held fixed, no maturation).
pub fn run_dual_orthogonal_assay(
    spec: &OrthogonalAssaySpec,
    theta_s_fixed: f64,
) -> Result<DualOrthogonalAssayResult, String> {
    let params = v11_exchange_only_params();
    let c = d034_frozen_exchange_candidate();
    let mut p = params.clone();
    p.k_exchange = c.k_exchange;
    p.k_exchange_eq = c.k_exchange_eq;
    let q_c = membrane_catalyst_saturation(spec.catalyst0, &p);
    let (
        grid,
        phi,
        catalyst,
        activated,
        mut precursor,
        mut s,
        mut u,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = build_dual_fixed_interface_state(
        &p,
        spec.radius,
        spec.theta0,
        theta_s_fixed,
        spec.precursor0,
        spec.catalyst0,
    );
    let mut s_next = s.clone();
    let mut u_next = u.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut notes = Vec::new();
    let mut first: Option<DualExchangeObservabilitySample> = None;
    let p0 = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX;
    let u0 = total_surface_mass(&grid, &u);
    let _s0 = total_surface_mass(&grid, &s);

    for step in 0..spec.max_steps {
        reconstruct_gamma_field(&grid, &s, &geometry, p.delta_floor, &mut gamma);
        let bases = compute_dual_exchange_local_bases(
            &grid, &precursor, &catalyst, &u, &s, &geometry, &p,
        );
        if bases.mean_theta_u > spec.theta_stop && spec.theta0 <= D030_ADS_THETA_MAX {
            notes.push(format!("stopped_theta_u={:.4}", bases.mean_theta_u));
            break;
        }
        let p_before = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX;
        let u_before = total_surface_mass(&grid, &u);
        let s_before = total_surface_mass(&grid, &s);
        u_next.copy_from_slice(&u);
        let totals = evolve_surface_density(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            &p,
            spec.dt,
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
            Some(&u),
            Some(&mut u_next),
        )
        .map_err(|e| format!("evolve reject: {e:?}"))?;
        s.copy_from_slice(&s_next);
        u.copy_from_slice(&u_next);
        precursor.copy_from_slice(&p_next);
        let p_after = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX;
        let u_after = total_surface_mass(&grid, &u);
        let s_after = total_surface_mass(&grid, &s);
        if first.is_none() {
            first = Some(sample_from_dual_step(
                &format!("{}:first", spec.label),
                1,
                spec.dt,
                &bases,
                &totals,
                p_before,
                u_before,
                s_before,
                p_after,
                u_after,
                s_after,
            ));
        }
        let _ = step;
    }

    let first = first.ok_or_else(|| "no accepted substep".to_string())?;
    let pass_gates = first.accounting_residual < 1e-9
        && first.exchange_dissipation >= -1e-12
        && first.net_exchange.is_finite()
        && first.exact_ds.abs() < 1e-12;
    Ok(DualOrthogonalAssayResult {
        spec: spec.clone(),
        theta_s_fixed,
        q_c,
        first,
        pass_gates,
        notes,
    })
}

/// Equilibrate P↔U partition (S fixed at 0).
pub fn run_dual_equilibrium_assay(
    k_exchange: f64,
    k_eq: f64,
    radius: f64,
    total_mass: f64,
    u_fraction: f64,
    catalyst: f64,
    dt: f64,
    max_steps: u64,
) -> Result<(f64, f64, f64, f64), String> {
    let mut p = v11_exchange_only_params();
    p.k_exchange = k_exchange;
    p.k_exchange_eq = k_eq;
    let (
        grid,
        phi,
        catalyst_f,
        activated,
        mut precursor,
        mut s,
        mut u,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = build_dual_fixed_interface_state(&p, radius, 0.0, 0.0, 0.0, catalyst);
    let n = grid.width * grid.height;
    let mut delta_sum = 0.0;
    let mut dish_cells = 0usize;
    for idx in 0..n {
        if grid.in_dish(idx) {
            dish_cells += 1;
            if geometry[idx].delta > p.delta_floor {
                delta_sum += geometry[idx].delta;
            }
        }
    }
    let cell = DX * DX;
    let target_u = (u_fraction.clamp(0.01, 0.99) * total_mass).max(0.0);
    let target_p_mass = (total_mass - target_u).max(0.0);
    let gamma_u = if delta_sum > 0.0 {
        (target_u / delta_sum).min(p.gamma_max * 0.99)
    } else {
        0.0
    };
    u.fill(0.0);
    seed_surface_from_gamma(&grid, &geometry, p.delta_floor, &mut u, |_, _, _| gamma_u);
    let p_field = if dish_cells > 0 {
        target_p_mass / (dish_cells as f64 * cell)
    } else {
        0.0
    };
    for idx in 0..n {
        if grid.in_dish(idx) {
            precursor[idx] = p_field;
        }
    }
    let mut s_next = s.clone();
    let mut u_next = u.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let total0 = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX
        + total_surface_mass(&grid, &u)
        + total_surface_mass(&grid, &s);
    for _ in 0..max_steps {
        u_next.copy_from_slice(&u);
        let _ = evolve_surface_density(
            &grid,
            &phi,
            &catalyst_f,
            &activated,
            &precursor,
            &s,
            &p,
            dt,
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
            Some(&u),
            Some(&mut u_next),
        )
        .map_err(|e| format!("evolve reject: {e:?}"))?;
        s.copy_from_slice(&s_next);
        u.copy_from_slice(&u_next);
        precursor.copy_from_slice(&p_next);
    }
    let bases = compute_dual_exchange_local_bases(
        &grid, &precursor, &catalyst_f, &u, &s, &geometry, &p,
    );
    let total1 = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX
        + total_surface_mass(&grid, &u)
        + total_surface_mass(&grid, &s);
    Ok((bases.mean_theta_u, bases.bulk_p, total0, total1))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PassiveUExchangeRegression {
    pub alpha_estimates: Vec<f64>,
    pub beta_estimates: Vec<f64>,
    pub alpha_median: f64,
    pub beta_median: f64,
    pub alpha_rel_err: f64,
    pub beta_rel_err: f64,
    pub recovery: crate::d030_analysis::DirectParameterRecovery,
    pub mixed_flux_rel_err: f64,
    pub equilibrium_isotherm_err: f64,
    pub conservation_ok: bool,
    pub dissipation_ok: bool,
    pub pass: bool,
    pub conclusion: String,
}

/// Gate 2 — passive P↔U exchange regression (D-030-style assays on v11 with U as exchange state).
pub fn passive_u_exchange_regression() -> Result<PassiveUExchangeRegression, String> {
    let params = v11_exchange_only_params();
    let ads = adsorption_matrix_specs(&params);
    let des = desorption_matrix_specs(&params);
    let mut alphas = Vec::new();
    let mut betas = Vec::new();
    let mut a_by_q = vec![Vec::new(); 3];
    let mut b_by_q = vec![Vec::new(); 3];
    let mut conservation_ok = true;
    let mut dissipation_ok = true;

    for (i, spec) in ads.iter().enumerate() {
        let r = run_dual_orthogonal_assay(spec, 0.0)?;
        alphas.push(r.first.alpha_estimate);
        a_by_q[i / 3].push(r.first.alpha_estimate);
        if r.first.accounting_residual >= 1e-9 {
            conservation_ok = false;
        }
        if r.first.exchange_dissipation < -1e-12 {
            dissipation_ok = false;
        }
    }
    for (i, spec) in des.iter().enumerate() {
        let r = run_dual_orthogonal_assay(spec, 0.0)?;
        betas.push(r.first.beta_estimate);
        b_by_q[i / 3].push(r.first.beta_estimate);
        if r.first.accounting_residual >= 1e-9 {
            conservation_ok = false;
        }
        if r.first.exchange_dissipation < -1e-12 {
            dissipation_ok = false;
        }
    }

    // Mixed assay with fixed S=0.2 (S unused for exchange dynamics but occupies capacity).
    let mixed_spec = OrthogonalAssaySpec {
        label: "mixed_u".into(),
        theta0: 0.3,
        precursor0: 0.5,
        catalyst0: catalyst_for_q(&params, 0.5),
        radius: 10.0,
        dt: 1e-3,
        max_steps: 5,
        theta_stop: 0.95,
    };
    let mixed = run_dual_orthogonal_assay(&mixed_spec, 0.2)?;
    let c = d034_frozen_exchange_candidate();
    let alpha_pred = c.k_exchange * c.k_exchange_eq;
    let mixed_flux_rel_err = if alpha_pred > 0.0 && mixed.first.net_exchange.abs() > 1e-30 {
        let expected_sign = mixed.first.net_exchange.signum();
        let _ = expected_sign;
        (mixed.first.alpha_estimate - alpha_pred).abs() / alpha_pred
    } else {
        0.0
    };

    // Equilibrium partition on U with S=0.
    let (theta_eq, _p_eq, total0, total1) = run_dual_equilibrium_assay(
        c.k_exchange,
        c.k_exchange_eq,
        10.0,
        2.0,
        0.4,
        catalyst_for_q(&params, 0.5),
        1e-3,
        200,
    )?;
    let conservation_total = ((total1 - total0) / total0.max(1e-12)).abs();
    if conservation_total >= 1e-9 {
        conservation_ok = false;
    }
    let dish_cells = grid_cell_count();
    let p_eq = _p_eq / (dish_cells as f64 * DX * DX).max(1e-12);
    let equilibrium_isotherm_err = isotherm_ratio(theta_eq, p_eq, c.k_exchange_eq);

    let recovery = recover_exchange_parameters(&alphas, &betas, &a_by_q, &b_by_q);
    let alpha_median = robust_median(&alphas);
    let beta_median = robust_median(&betas);
    let alpha_rel_err = ((alpha_median - D034_ALPHA_FROZEN) / D034_ALPHA_FROZEN).abs();
    let beta_rel_err = ((beta_median - D034_BETA_FROZEN) / D034_BETA_FROZEN).abs();

    let pass = alpha_rel_err <= D034_EXCHANGE_REL_TOL
        && beta_rel_err <= D034_EXCHANGE_REL_TOL
        && conservation_ok
        && dissipation_ok;

    Ok(PassiveUExchangeRegression {
        alpha_estimates: alphas,
        beta_estimates: betas,
        alpha_median,
        beta_median,
        alpha_rel_err,
        beta_rel_err,
        recovery,
        mixed_flux_rel_err,
        equilibrium_isotherm_err,
        conservation_ok,
        dissipation_ok,
        pass,
        conclusion: if pass {
            "D034_PASSIVE_U_EXCHANGE_PASS".into()
        } else {
            "D034_PASSIVE_EXCHANGE_REGRESSION".into()
        },
    })
}

fn grid_cell_count() -> usize {
    Grid::new().width * Grid::new().height
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaturationAssayRow {
    pub u0: f64,
    pub a0: f64,
    pub q_target: f64,
    pub k_estimate: f64,
    pub u_loss: f64,
    pub s_gain: f64,
    pub a_loss: f64,
    pub w_gain: f64,
    pub stoichiometry_ok: bool,
    pub no_a_ok: bool,
    pub capacity_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrthogonalMaturationId {
    pub k_mature_true: f64,
    pub estimates: Vec<f64>,
    pub median: f64,
    pub relative_error: f64,
    pub rows: Vec<MaturationAssayRow>,
    pub identifiable: bool,
    pub conclusion: String,
}

fn maturation_basis_integrated(
    u: f64,
    a: f64,
    catalyst: f64,
    _delta: f64,
    params: &SimParams,
) -> f64 {
    let q = membrane_catalyst_saturation(catalyst, params);
    let a_act = if params.a_reference > 0.0 {
        a / params.a_reference
    } else {
        a
    };
    // ∫ δ q(C) a Γ_U dV with U = δ Γ_U ⇒ integrand q·a·U per interface cell.
    q * a_act * u.max(0.0)
}

fn median_sorted(vals: &mut [f64]) -> f64 {
    if vals.is_empty() {
        return f64::NAN;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = vals.len();
    if n % 2 == 1 {
        vals[n / 2]
    } else {
        0.5 * (vals[n / 2 - 1] + vals[n / 2])
    }
}

/// Gate 4 — orthogonal maturation identification on fixed interface.
pub fn identify_orthogonal_maturation(k_mature_true: f64) -> OrthogonalMaturationId {
    let params = v11_maturation_only_params(k_mature_true);
    let dt = 0.01_f64;
    let delta = 0.5_f64;
    let mut estimates = Vec::new();
    let mut rows = Vec::new();

    let cases: [(f64, f64, f64); 5] = [
        (0.4, 0.8, 0.4),
        (0.3, 0.6, 0.2),
        (0.5, 1.0, 0.6),
        (0.2, 0.5, 0.8),
        (0.35, 0.4, 0.5),
    ];

    for &(u0, a0, q_target) in &cases {
        let catalyst = catalyst_for_q(&params, q_target);
        let s0 = 0.1_f64;
        let w0 = 0.0_f64;
        let (u1, a1, s1, w1, r) =
            apply_maturation_bounded(u0, a0, s0, delta, catalyst, dt, &params);
        let basis = maturation_basis_integrated(u0, a0, catalyst, delta, &params);
        let k_est = if basis > D034_BASIS_EPS && r > 0.0 {
            r / (basis * dt)
        } else {
            f64::NAN
        };
        if k_est.is_finite() {
            estimates.push(k_est);
        }
        let u_loss = u0 - u1;
        let s_gain = s1 - s0;
        let a_loss = a0 - a1;
        let w_gain = w1 - w0;
        let stoichiometry_ok = (u_loss - r).abs() < 1e-11
            && (s_gain - r).abs() < 1e-11
            && (a_loss - r).abs() < 1e-11
            && (w_gain - r).abs() < 1e-11;
        let (_, _, _, _, r_no_a) = apply_maturation_bounded(u0, 0.0, s0, delta, catalyst, dt, &params);
        let no_a_ok = r_no_a == 0.0;
        let theta_before =
            surface_occupancy_theta(u0 / delta, params.gamma_max)
                + surface_occupancy_theta(s0 / delta, params.gamma_max);
        let theta_after =
            surface_occupancy_theta(u1 / delta, params.gamma_max)
                + surface_occupancy_theta(s1 / delta, params.gamma_max);
        let capacity_ok = (theta_after - theta_before).abs() < 1e-11;
        rows.push(MaturationAssayRow {
            u0,
            a0,
            q_target,
            k_estimate: k_est,
            u_loss,
            s_gain,
            a_loss,
            w_gain,
            stoichiometry_ok,
            no_a_ok,
            capacity_ok,
        });
    }

    let mut sorted = estimates.clone();
    let median = median_sorted(&mut sorted);
    let relative_error = if k_mature_true > 0.0 && median.is_finite() {
        ((median - k_mature_true) / k_mature_true).abs()
    } else {
        f64::INFINITY
    };
    let rows_ok = rows.iter().all(|r| r.stoichiometry_ok && r.no_a_ok && r.capacity_ok);
    let identifiable = rows_ok
        && estimates.len() >= 4
        && median.is_finite()
        && median > 0.0
        && relative_error <= D034_MATURATION_RATE_TOL;

    OrthogonalMaturationId {
        k_mature_true,
        estimates,
        median,
        relative_error,
        rows,
        identifiable,
        conclusion: if identifiable {
            "D034_MATURATION_KINETICS_IDENTIFIABLE".into()
        } else {
            "D034_MATURATION_KINETICS_NOT_IDENTIFIABLE".into()
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaturationRateEstimate {
    pub state_id: String,
    pub l_s: f64,
    pub b_mature: f64,
    pub k_mature_required: f64,
    pub valid: bool,
    pub reject_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaturationRateReconstruction {
    pub estimates: Vec<MaturationRateEstimate>,
    pub valid_count: usize,
    pub median_k_mature: f64,
    pub span_factor: f64,
    pub loo_medians: Vec<f64>,
    pub loo_ok: bool,
    pub portable: bool,
    pub conclusion: String,
}

/// Instantaneous maturation basis ∫ δ q(C) a Γ_U dV on a simulation snapshot.
pub fn integrate_maturation_basis(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let dx2 = DX * DX;
    let mut b = 0.0;
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= sim.params.delta_floor {
            continue;
        }
        let u = sim.fields.immature_membrane[idx].max(0.0);
        let q = membrane_catalyst_saturation(sim.fields.catalyst[idx].max(0.0), &sim.params);
        let a = sim.fields.activated[idx].max(0.0) / sim.params.a_reference.max(1e-30);
        b += q * a * u * dx2;
    }
    b
}

/// S turnover load ∫ δ k_γ Γ_S dV.
pub fn integrate_s_turnover_load(sim: &Simulation) -> f64 {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    let dx2 = DX * DX;
    let mut l = 0.0;
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let d = geometry[idx].delta;
        if d <= sim.params.delta_floor {
            continue;
        }
        let s = sim.fields.membrane[idx].max(0.0);
        let gamma_s = s / d;
        l += d * sim.params.k_gamma_decay * gamma_s.max(0.0) * dx2;
    }
    l
}

pub fn estimate_k_mature_required(state_id: &str, sim: &Simulation) -> MaturationRateEstimate {
    let l_s = integrate_s_turnover_load(sim);
    let b_mature = integrate_maturation_basis(sim);
    let mut valid = true;
    let mut reject = String::new();
    if !(l_s > 0.0 && l_s.is_finite()) {
        valid = false;
        reject = "l_s_nonpositive".into();
    } else if !(b_mature > D034_BASIS_EPS && b_mature.is_finite()) {
        valid = false;
        reject = "b_mature_underflow".into();
    }
    let k = if valid { l_s / b_mature } else { f64::NAN };
    if valid && !(k.is_finite() && k > 0.0) {
        valid = false;
        reject = "k_nonfinite".into();
    }
    MaturationRateEstimate {
        state_id: state_id.into(),
        l_s,
        b_mature,
        k_mature_required: k,
        valid,
        reject_reason: reject,
    }
}

/// Build a tiny v11 sim at a fixed renewal state for rate reconstruction.
pub fn build_renewal_state_sim(
    state_id: &str,
    theta_u: f64,
    theta_s: f64,
    precursor: f64,
    activated: f64,
    q_target: f64,
    k_mature: f64,
) -> Simulation {
    let mut params = v11_params(k_mature);
    params.k_gamma_decay = params.k_gamma_decay.max(1e-6);
    params.reactions_enabled = false;
    let mut sim = Simulation::new(params);
    let n = sim.grid.width * sim.grid.height;
    let mut phi = sim.fields.structure.clone();
    circular_phi_profile(&sim.grid, 10.0, 2.0, &mut phi);
    sim.fields.structure.copy_from_slice(&phi);
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(&sim.grid, &phi, sim.params.eta_n, &mut geometry);
    sim.fields.membrane.fill(0.0);
    sim.fields.immature_membrane.fill(0.0);
    seed_surface_from_gamma(
        &sim.grid,
        &geometry,
        sim.params.delta_floor,
        &mut sim.fields.membrane,
        |_, _, _| theta_s * sim.params.gamma_max,
    );
    seed_surface_from_gamma(
        &sim.grid,
        &geometry,
        sim.params.delta_floor,
        &mut sim.fields.immature_membrane,
        |_, _, _| theta_u * sim.params.gamma_max,
    );
    let catalyst = catalyst_for_q(&sim.params, q_target);
    for idx in 0..n {
        if sim.grid.in_dish(idx) {
            sim.fields.catalyst[idx] = catalyst;
            sim.fields.precursor[idx] = precursor;
            sim.fields.activated[idx] = activated;
        }
    }
    let _ = state_id;
    sim
}

/// Gate 6 — analytical k_mature reconstruction across renewal states.
pub fn reconstruct_maturation_rate() -> MaturationRateReconstruction {
    let k_ref = D034_ASSAY_K_MATURE;
    let states = [
        ("highU_lowS", 0.5_f64, 0.1_f64, 0.6_f64, 0.8_f64, 0.5_f64),
        ("balanced", 0.25, 0.25, 0.5, 0.6, 0.5),
        ("lowU_highS", 0.1, 0.5, 0.4, 0.6, 0.5),
        ("lowA", 0.3, 0.2, 0.5, 0.2, 0.5),
        ("medA", 0.3, 0.2, 0.5, 0.6, 0.5),
        ("highA", 0.3, 0.2, 0.5, 1.2, 0.5),
    ];
    let estimates: Vec<MaturationRateEstimate> = states
        .iter()
        .map(|(id, tu, ts, p, a, q)| {
            let sim = build_renewal_state_sim(id, *tu, *ts, *p, *a, *q, k_ref);
            estimate_k_mature_required(id, &sim)
        })
        .collect();
    let valid: Vec<f64> = estimates
        .iter()
        .filter(|e| e.valid)
        .map(|e| e.k_mature_required)
        .collect();
    let valid_count = valid.len();
    let mut sorted = valid.clone();
    let median = median_sorted(&mut sorted);
    let (span, loo, loo_ok, portable, conclusion) = if valid_count < D034_MIN_VALID_STATES {
        (
            f64::NAN,
            Vec::new(),
            false,
            false,
            "D034_MATURATION_LAW_NOT_PORTABLE".to_string(),
        )
    } else {
        let min_k = valid.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_k = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let span = if min_k > 0.0 {
            max_k / min_k
        } else {
            f64::INFINITY
        };
        let mut loo_medians = Vec::new();
        let mut loo_ok = true;
        for i in 0..valid.len() {
            let mut others: Vec<f64> = valid
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, v)| *v)
                .collect();
            let m = median_sorted(&mut others);
            loo_medians.push(m);
            if !m.is_finite()
                || median <= 0.0
                || ((m - median).abs() / median) > D034_LOO_MEDIAN_REL_MAX
            {
                loo_ok = false;
            }
        }
        let portable = span.is_finite()
            && span <= D034_PORTABILITY_SPAN_MAX
            && loo_ok
            && median.is_finite()
            && median > 0.0;
        let conclusion = if portable {
            "D034_MATURATION_RATE_PORTABLE".to_string()
        } else {
            "D034_MATURATION_LAW_NOT_PORTABLE".to_string()
        };
        (span, loo_medians, loo_ok, portable, conclusion)
    };
    MaturationRateReconstruction {
        estimates,
        valid_count,
        median_k_mature: median,
        span_factor: span,
        loo_medians: loo,
        loo_ok,
        portable,
        conclusion,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaturationCandidate {
    pub identity: String,
    pub k_mature: f64,
    pub scale: f64,
}

pub fn generate_maturation_candidates(median_k: f64) -> Vec<MaturationCandidate> {
    let mut out = Vec::new();
    // Center (1.0×) first per directive.
    for &scale in &[1.0, 0.5, 2.0] {
        out.push(MaturationCandidate {
            identity: format!("k_mature_{scale}x"),
            k_mature: median_k * scale,
            scale,
        });
    }
    out.truncate(D034_MAX_MATURATION_CANDIDATES);
    out
}

/// Safeguarded bracketed interpolation between below- and above-balance candidates.
pub fn bracketed_maturation_interpolate(k_lo: f64, k_hi: f64, q_lo: f64, q_hi: f64) -> Option<f64> {
    if !(k_lo < k_hi && q_lo.is_finite() && q_hi.is_finite()) {
        return None;
    }
    if (q_lo - 1.0) * (q_hi - 1.0) >= 0.0 {
        return None;
    }
    let denom = q_hi - q_lo;
    if denom.abs() < 1e-30 {
        return None;
    }
    let k = k_lo + (1.0 - q_lo) * (k_hi - k_lo) / denom;
    if k.is_finite() && k > k_lo && k < k_hi {
        Some(k)
    } else {
        None
    }
}
