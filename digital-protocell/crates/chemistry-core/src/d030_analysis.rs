//! D-030 orthogonal transient identification of reversible P↔S exchange.
//!
//! Establishes adsorption-only / desorption-only / mixed transient assays that
//! independently excite forward and reverse exchange under the frozen v8 law:
//! `J = α δ q(C) Γ_max p (1−θ) − β δ q(C) Γ_max θ` with `α = k K`, `β = k`.

use crate::config::{SimParams, DX};
use crate::grid::Grid;
use crate::membrane::membrane_catalyst_saturation;
use crate::surface_density::{
    circular_phi_profile, compute_interface_geometry, evolve_surface_density,
    reconstruct_gamma_field, seed_surface_from_gamma, surface_occupancy_theta, total_surface_mass,
    InterfaceGeometryCell, SurfaceAccountingTotals,
};
use serde::{Deserialize, Serialize};

/// Relative dispersion ceiling across P or θ levels (Gate 2/3).
pub const D030_LEVEL_SPREAD_MAX: f64 = 0.10;
/// Relative dispersion ceiling after q(C) normalization.
pub const D030_Q_NORM_SPREAD_MAX: f64 = 0.10;
/// Leave-one-experiment-out relative tolerance (Gate 4).
pub const D030_LOO_REL_MAX: f64 = 0.25;
/// Bootstrap spread factor ceiling (Gate 4).
pub const D030_BOOTSTRAP_SPREAD_MAX: f64 = 1.5;
/// Condition number ceiling (Gate 4).
pub const D030_COND_MAX: f64 = 1.0e6;
/// Occupancy ceiling for adsorption identification window.
pub const D030_ADS_THETA_MAX: f64 = 0.05;
/// Mixed-state initial flux relative error ceiling (Gate 5).
pub const D030_MIXED_FLUX_REL_MAX: f64 = 0.15;
/// Mixed-state trajectory relative error ceiling (Gate 5).
pub const D030_MIXED_TRAJ_REL_MAX: f64 = 0.20;
/// Equilibrium isotherm relative error ceiling (Gate 6).
pub const D030_EQ_ISOTHERM_REL_MAX: f64 = 0.05;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExchangeObservabilitySample {
    pub label: String,
    pub accepted_substeps: u64,
    pub dt: f64,
    pub forward_exchange: f64,
    pub reverse_exchange: f64,
    pub net_exchange: f64,
    pub bulk_p: f64,
    pub surface_s: f64,
    pub mean_theta: f64,
    pub mean_one_minus_theta: f64,
    pub mean_q_c: f64,
    pub adsorption_basis: f64,
    pub desorption_basis: f64,
    pub exchange_affinity_proxy: f64,
    pub exchange_dissipation: f64,
    pub exact_dp: f64,
    pub exact_ds: f64,
    pub accounting_residual: f64,
    pub alpha_estimate: f64,
    pub beta_estimate: f64,
}

/// Integrated exchange bases on the current fields (pre-step).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ExchangeLocalBases {
    pub adsorption_basis: f64,
    pub desorption_basis: f64,
    pub mean_theta: f64,
    pub mean_one_minus_theta: f64,
    pub mean_q_c: f64,
    pub bulk_p: f64,
    pub surface_s: f64,
    pub interface_cells: usize,
}

pub fn compute_exchange_local_bases(
    grid: &Grid,
    precursor: &[f64],
    catalyst: &[f64],
    s: &[f64],
    geometry: &[InterfaceGeometryCell],
    gamma: &[f64],
    params: &SimParams,
) -> ExchangeLocalBases {
    let gamma_max = params.gamma_max.max(0.0);
    let pref = if params.p_reference > 0.0 {
        params.p_reference
    } else {
        1.0
    };
    let mut bases = ExchangeLocalBases::default();
    let mut theta_w = 0.0;
    let mut sat_w = 0.0;
    let mut q_w = 0.0;
    let mut wsum = 0.0;
    let mut p_sum = 0.0;
    for idx in 0..s.len() {
        if !grid.in_dish(idx) {
            continue;
        }
        p_sum += precursor[idx].max(0.0);
        let delta = geometry[idx].delta;
        if delta <= params.delta_floor {
            continue;
        }
        let g = gamma[idx].max(0.0);
        let p = precursor[idx].max(0.0) / pref;
        let q_c = membrane_catalyst_saturation(catalyst[idx].max(0.0), params);
        let theta = surface_occupancy_theta(g, gamma_max);
        let sat = (1.0 - theta).max(0.0);
        bases.adsorption_basis += delta * gamma_max * q_c * p * sat;
        bases.desorption_basis += delta * gamma_max * q_c * theta;
        theta_w += delta * theta;
        sat_w += delta * sat;
        q_w += delta * q_c;
        wsum += delta;
        bases.interface_cells += 1;
    }
    bases.bulk_p = p_sum * DX * DX;
    bases.surface_s = total_surface_mass(grid, s);
    if wsum > 0.0 {
        bases.mean_theta = theta_w / wsum;
        bases.mean_one_minus_theta = sat_w / wsum;
        bases.mean_q_c = q_w / wsum;
    }
    bases
}

/// First-substep α estimator: `α = J_net / (dt · A)` with A = ∫ δ Γ_max q p (1−θ).
#[inline]
pub fn estimate_alpha_from_step(net_exchange: f64, dt: f64, adsorption_basis: f64) -> f64 {
    if !(dt > 0.0) || !(adsorption_basis > 0.0) || !net_exchange.is_finite() {
        return f64::NAN;
    }
    net_exchange / (dt * adsorption_basis)
}

/// First-substep β estimator: `β = −J_net / (dt · B)` with B = ∫ δ Γ_max q θ.
#[inline]
pub fn estimate_beta_from_step(net_exchange: f64, dt: f64, desorption_basis: f64) -> f64 {
    if !(dt > 0.0) || !(desorption_basis > 0.0) || !net_exchange.is_finite() {
        return f64::NAN;
    }
    (-net_exchange) / (dt * desorption_basis)
}

pub fn sample_from_step(
    label: &str,
    accepted_substeps: u64,
    dt: f64,
    bases_before: &ExchangeLocalBases,
    totals: &SurfaceAccountingTotals,
    p_before: f64,
    s_before: f64,
    p_after: f64,
    s_after: f64,
) -> ExchangeObservabilitySample {
    let exact_dp = p_after - p_before;
    let exact_ds = s_after - s_before;
    // Exchange-only: ΔP + ΔS should close (cell volume already in p totals).
    let accounting_residual = (exact_dp + exact_ds).abs();
    let alpha_estimate =
        estimate_alpha_from_step(totals.exchange_net, dt, bases_before.adsorption_basis);
    let beta_estimate =
        estimate_beta_from_step(totals.exchange_net, dt, bases_before.desorption_basis);
    let aff = if bases_before.desorption_basis > 0.0 && bases_before.adsorption_basis > 0.0 {
        (bases_before.adsorption_basis / bases_before.desorption_basis).ln()
    } else if bases_before.adsorption_basis > 0.0 {
        f64::INFINITY
    } else if bases_before.desorption_basis > 0.0 {
        f64::NEG_INFINITY
    } else {
        0.0
    };
    ExchangeObservabilitySample {
        label: label.to_string(),
        accepted_substeps,
        dt,
        forward_exchange: totals.exchange_forward,
        reverse_exchange: totals.exchange_reverse,
        net_exchange: totals.exchange_net,
        bulk_p: bases_before.bulk_p,
        surface_s: bases_before.surface_s,
        mean_theta: bases_before.mean_theta,
        mean_one_minus_theta: bases_before.mean_one_minus_theta,
        mean_q_c: bases_before.mean_q_c,
        adsorption_basis: bases_before.adsorption_basis,
        desorption_basis: bases_before.desorption_basis,
        exchange_affinity_proxy: aff,
        exchange_dissipation: totals.exchange_dissipation,
        exact_dp,
        exact_ds,
        accounting_residual,
        alpha_estimate,
        beta_estimate,
    }
}

/// Fixed-interface exchange-only assay configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrthogonalAssaySpec {
    pub label: String,
    pub theta0: f64,
    pub precursor0: f64,
    pub catalyst0: f64,
    pub radius: f64,
    pub dt: f64,
    pub max_steps: u64,
    pub theta_stop: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrthogonalAssayResult {
    pub spec: OrthogonalAssaySpec,
    pub q_c: f64,
    pub first: ExchangeObservabilitySample,
    pub window_10: Option<ExchangeObservabilitySample>,
    pub window_100: Option<ExchangeObservabilitySample>,
    pub trajectory_p: Vec<f64>,
    pub trajectory_s: Vec<f64>,
    pub trajectory_theta: Vec<f64>,
    pub trajectory_net: Vec<f64>,
    pub pass_gates: bool,
    pub notes: Vec<String>,
}

fn v8_exchange_only_params(k_exchange: f64, k_eq: f64) -> SimParams {
    let mut p = SimParams::default();
    p.equation_version =
        crate::config::EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    p.k_exchange = k_exchange;
    p.k_exchange_eq = k_eq;
    p.p_reference = 1.0;
    p.k_ads = 0.0;
    p.gamma_max = 1.0;
    p.gamma_reference = 1.0;
    p.k_gamma_decay = 0.0;
    p.d_gamma = 0.0;
    p.k_precursor = 0.0;
    p.k_precursor_decay = 0.0;
    p.k_rep = 0.0;
    p.k_structure = 0.0;
    p.k_structure_decay = 0.0;
    p.k_d008_activation = 0.0;
    p.reactions_enabled = false;
    p
}

/// Build a circular fixed-interface state at uniform θ and precursor/catalyst.
pub fn build_fixed_interface_state(
    params: &SimParams,
    radius: f64,
    theta: f64,
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
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |_, _, _| {
        theta * params.gamma_max
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
        grid, phi, cat, act, prec, s, waste, geometry, gamma, diffusion,
    )
}

/// Catalyst level producing approximately the requested q(C) = C/(k_c+C).
pub fn catalyst_for_q(params: &SimParams, q_target: f64) -> f64 {
    let q = q_target.clamp(1e-6, 1.0 - 1e-6);
    params.k_c_membrane * q / (1.0 - q)
}

/// Run one orthogonal exchange-only transient (no turnover, no productive chemistry).
pub fn run_orthogonal_assay(
    k_exchange: f64,
    k_eq: f64,
    spec: &OrthogonalAssaySpec,
) -> Result<OrthogonalAssayResult, String> {
    let params = v8_exchange_only_params(k_exchange, k_eq);
    let q_c = membrane_catalyst_saturation(spec.catalyst0, &params);
    let (
        grid,
        phi,
        catalyst,
        activated,
        mut precursor,
        mut s,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
    ) = build_fixed_interface_state(
        &params,
        spec.radius,
        spec.theta0,
        spec.precursor0,
        spec.catalyst0,
    );
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let mut notes = Vec::new();
    let mut trajectory_p = Vec::new();
    let mut trajectory_s = Vec::new();
    let mut trajectory_theta = Vec::new();
    let mut trajectory_net = Vec::new();
    let mut first: Option<ExchangeObservabilitySample> = None;
    let mut window_10: Option<ExchangeObservabilitySample> = None;
    let mut window_100: Option<ExchangeObservabilitySample> = None;
    let mut cum = SurfaceAccountingTotals::default();
    let mut accepted = 0u64;
    let p0 = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX;
    let s0 = total_surface_mass(&grid, &s);
    trajectory_p.push(p0);
    trajectory_s.push(s0);
    trajectory_theta.push(spec.theta0);
    trajectory_net.push(0.0);

    for step in 0..spec.max_steps {
        reconstruct_gamma_field(
            &grid,
            &s,
            &geometry,
            params.delta_floor,
            &mut gamma,
        );
        let bases = compute_exchange_local_bases(
            &grid, &precursor, &catalyst, &s, &geometry, &gamma, &params,
        );
        if bases.mean_theta > spec.theta_stop && spec.theta0 <= D030_ADS_THETA_MAX {
            notes.push(format!("stopped_theta={:.4}", bases.mean_theta));
            break;
        }
        let p_before = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX;
        let s_before = total_surface_mass(&grid, &s);
        // Exchange only: synthesis/decay/turnover/diffusion off.
        let totals = evolve_surface_density(
            &grid,
            &phi,
            &catalyst,
            &activated,
            &precursor,
            &s,
            &params,
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
        )
        .map_err(|e| format!("evolve reject: {e:?}"))?;
        s.copy_from_slice(&s_next);
        precursor.copy_from_slice(&p_next);
        cum.accumulate(totals.clone());
        accepted += 1;
        let p_after = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX;
        let s_after = total_surface_mass(&grid, &s);
        reconstruct_gamma_field(
            &grid,
            &s,
            &geometry,
            params.delta_floor,
            &mut gamma,
        );
        let bases_after = compute_exchange_local_bases(
            &grid, &precursor, &catalyst, &s, &geometry, &gamma, &params,
        );
        trajectory_p.push(p_after);
        trajectory_s.push(s_after);
        trajectory_theta.push(bases_after.mean_theta);
        trajectory_net.push(totals.exchange_net);

        if first.is_none() {
            first = Some(sample_from_step(
                &format!("{}:first", spec.label),
                1,
                spec.dt,
                &bases,
                &totals,
                p_before,
                s_before,
                p_after,
                s_after,
            ));
        }
        if accepted == 10 {
            window_10 = Some(sample_from_step(
                &format!("{}:w10", spec.label),
                10,
                spec.dt * 10.0,
                &bases, // ponytail: window sample uses last-step bases; upgrade: baseline snapshot
                &cum,
                p0,
                s0,
                p_after,
                s_after,
            ));
        }
        if accepted == 100 {
            window_100 = Some(sample_from_step(
                &format!("{}:w100", spec.label),
                100,
                spec.dt * 100.0,
                &bases,
                &cum,
                p0,
                s0,
                p_after,
                s_after,
            ));
        }
        let _ = step;
    }

    let first = first.ok_or_else(|| "no accepted substep".to_string())?;
    let pass_gates = first.accounting_residual < 1e-9
        && first.exchange_dissipation >= -1e-12
        && first.net_exchange.is_finite();
    Ok(OrthogonalAssayResult {
        spec: spec.clone(),
        q_c,
        first,
        window_10,
        window_100,
        trajectory_p,
        trajectory_s,
        trajectory_theta,
        trajectory_net,
        pass_gates,
        notes,
    })
}

/// Robust median of finite samples.
pub fn robust_median(values: &[f64]) -> f64 {
    let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

/// Max/min relative spread about the median: (max−min)/median.
pub fn relative_spread(values: &[f64]) -> f64 {
    let med = robust_median(values);
    if !(med.abs() > 0.0) {
        return f64::INFINITY;
    }
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &x in values {
        if x.is_finite() {
            lo = lo.min(x);
            hi = hi.max(x);
        }
    }
    if !lo.is_finite() {
        return f64::INFINITY;
    }
    (hi - lo) / med.abs()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectParameterRecovery {
    pub alpha_estimates: Vec<f64>,
    pub beta_estimates: Vec<f64>,
    pub alpha_direct: f64,
    pub beta_direct: f64,
    pub k_exchange: f64,
    pub k_exchange_eq: f64,
    pub alpha_spread: f64,
    pub beta_spread: f64,
    pub alpha_q_norm_spread: f64,
    pub beta_q_norm_spread: f64,
    pub bootstrap_alpha: Vec<f64>,
    pub bootstrap_beta: Vec<f64>,
    pub bootstrap_spread_factor_alpha: f64,
    pub bootstrap_spread_factor_beta: f64,
    pub loo_alpha: Vec<f64>,
    pub loo_beta: f64,
    pub loo_ok: bool,
    pub condition_number: f64,
    pub covariance: [f64; 3],
    pub correlation: f64,
    pub identifiable: bool,
    pub conclusion: String,
}

fn bootstrap_medians(samples: &[f64], rounds: usize, seed: u64) -> Vec<f64> {
    if samples.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(rounds);
    let mut state = seed;
    for _ in 0..rounds {
        let mut draw = Vec::with_capacity(samples.len());
        for _ in 0..samples.len() {
            // xorshift64*
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let idx = (state as usize) % samples.len();
            draw.push(samples[idx]);
        }
        out.push(robust_median(&draw));
    }
    out
}

fn spread_factor(samples: &[f64]) -> f64 {
    let med = robust_median(samples);
    if !(med.abs() > 0.0) {
        return f64::INFINITY;
    }
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &x in samples {
        if x.is_finite() {
            lo = lo.min(x);
            hi = hi.max(x);
        }
    }
    if !(lo > 0.0) {
        return f64::INFINITY;
    }
    (hi / lo).max(lo / hi)
}

/// Recover (k, K) from direct α/β estimates with bootstrap + LOO checks.
pub fn recover_exchange_parameters(
    alpha_estimates: &[f64],
    beta_estimates: &[f64],
    alpha_by_q_group: &[Vec<f64>],
    beta_by_q_group: &[Vec<f64>],
) -> DirectParameterRecovery {
    let alpha_direct = robust_median(alpha_estimates);
    let beta_direct = robust_median(beta_estimates);
    let alpha_spread = relative_spread(alpha_estimates);
    let beta_spread = relative_spread(beta_estimates);
    let alpha_q_norm_spread = {
        let meds: Vec<f64> = alpha_by_q_group.iter().map(|g| robust_median(g)).collect();
        relative_spread(&meds)
    };
    let beta_q_norm_spread = {
        let meds: Vec<f64> = beta_by_q_group.iter().map(|g| robust_median(g)).collect();
        relative_spread(&meds)
    };
    let bootstrap_alpha = bootstrap_medians(alpha_estimates, 64, 0xD030_A11A);
    let bootstrap_beta = bootstrap_medians(beta_estimates, 64, 0xD030_BE7A);
    let bootstrap_spread_factor_alpha = spread_factor(&bootstrap_alpha);
    let bootstrap_spread_factor_beta = spread_factor(&bootstrap_beta);

    let mut loo_alpha = Vec::new();
    for i in 0..alpha_estimates.len() {
        let mut rest = alpha_estimates.to_vec();
        rest.remove(i);
        loo_alpha.push(robust_median(&rest));
    }
    let mut loo_beta_vals = Vec::new();
    for i in 0..beta_estimates.len() {
        let mut rest = beta_estimates.to_vec();
        rest.remove(i);
        loo_beta_vals.push(robust_median(&rest));
    }
    let loo_beta = robust_median(&loo_beta_vals);
    let loo_ok = loo_alpha.iter().all(|a| {
        a.is_finite()
            && alpha_direct > 0.0
            && ((a / alpha_direct) - 1.0).abs() <= D030_LOO_REL_MAX
    }) && loo_beta_vals.iter().all(|b| {
        b.is_finite()
            && beta_direct > 0.0
            && ((b / beta_direct) - 1.0).abs() <= D030_LOO_REL_MAX
    });

    // 2×2 design covariance on stacked [α samples; β samples] via simple moments.
    let n_a = alpha_estimates.len().max(1) as f64;
    let n_b = beta_estimates.len().max(1) as f64;
    let var_a = alpha_estimates
        .iter()
        .map(|x| (x - alpha_direct).powi(2))
        .sum::<f64>()
        / n_a;
    let var_b = beta_estimates
        .iter()
        .map(|x| (x - beta_direct).powi(2))
        .sum::<f64>()
        / n_b;
    let cov_ab = 0.0; // independent assay families
    let corr = 0.0;
    let cond = if var_b > 0.0 && var_a > 0.0 {
        (var_a.max(var_b)) / (var_a.min(var_b))
    } else if alpha_direct > 0.0 && beta_direct > 0.0 {
        1.0
    } else {
        f64::INFINITY
    };

    let k_exchange = beta_direct;
    let k_exchange_eq = if beta_direct > 0.0 {
        alpha_direct / beta_direct
    } else {
        f64::NAN
    };

    let ci_excludes_zero = bootstrap_alpha.iter().all(|&x| x > 0.0)
        && bootstrap_beta.iter().all(|&x| x > 0.0)
        && alpha_direct > 0.0
        && beta_direct > 0.0;

    let identifiable = alpha_direct.is_finite()
        && beta_direct.is_finite()
        && alpha_direct > 0.0
        && beta_direct > 0.0
        && k_exchange_eq.is_finite()
        && k_exchange_eq > 0.0
        && alpha_spread <= D030_LEVEL_SPREAD_MAX
        && beta_spread <= D030_LEVEL_SPREAD_MAX
        && alpha_q_norm_spread <= D030_Q_NORM_SPREAD_MAX
        && beta_q_norm_spread <= D030_Q_NORM_SPREAD_MAX
        && bootstrap_spread_factor_alpha <= D030_BOOTSTRAP_SPREAD_MAX
        && bootstrap_spread_factor_beta <= D030_BOOTSTRAP_SPREAD_MAX
        && loo_ok
        && cond < D030_COND_MAX
        && ci_excludes_zero;

    let conclusion = if identifiable {
        "D030_EXCHANGE_PARAMETERS_IDENTIFIED".to_string()
    } else if alpha_direct.is_finite()
        && beta_direct.is_finite()
        && alpha_direct > 0.0
        && beta_direct > 0.0
        && !(k_exchange_eq.is_finite() && k_exchange_eq > 0.0 && loo_ok)
    {
        "D030_EXCHANGE_PARAMETER_INCONSISTENCY".to_string()
    } else if !(beta_direct.is_finite() && beta_direct > 0.0) {
        "D030_REVERSE_EXCHANGE_NOT_IDENTIFIABLE".to_string()
    } else if !(alpha_direct.is_finite() && alpha_direct > 0.0) {
        "D030_FORWARD_EXCHANGE_NOT_IDENTIFIABLE".to_string()
    } else {
        "D030_EXCHANGE_PARAMETER_INCONSISTENCY".to_string()
    };

    DirectParameterRecovery {
        alpha_estimates: alpha_estimates.to_vec(),
        beta_estimates: beta_estimates.to_vec(),
        alpha_direct,
        beta_direct,
        k_exchange,
        k_exchange_eq,
        alpha_spread,
        beta_spread,
        alpha_q_norm_spread,
        beta_q_norm_spread,
        bootstrap_alpha,
        bootstrap_beta,
        bootstrap_spread_factor_alpha,
        bootstrap_spread_factor_beta,
        loo_alpha,
        loo_beta,
        loo_ok,
        condition_number: cond,
        covariance: [var_a, cov_ab, var_b],
        correlation: corr,
        identifiable,
        conclusion,
    }
}

/// Build the Gate 2 adsorption matrix specs (θ=0).
pub fn adsorption_matrix_specs(params: &SimParams) -> Vec<OrthogonalAssaySpec> {
    let p_levels = [0.25, 0.50, 1.00];
    let q_levels = [0.2, 0.5, 0.8];
    let mut out = Vec::new();
    for &q in &q_levels {
        let c = catalyst_for_q(params, q);
        for &p in &p_levels {
            out.push(OrthogonalAssaySpec {
                label: format!("ads_p{p:.2}_q{q:.1}"),
                theta0: 0.0,
                precursor0: p, // p_reference = 1
                catalyst0: c,
                radius: 10.0,
                dt: 1e-3,
                max_steps: 20,
                theta_stop: D030_ADS_THETA_MAX,
            });
        }
    }
    out
}

/// Build the Gate 3 desorption matrix specs (P=0).
pub fn desorption_matrix_specs(params: &SimParams) -> Vec<OrthogonalAssaySpec> {
    let theta_levels = [0.25, 0.50, 0.75];
    let q_levels = [0.2, 0.5, 0.8];
    let mut out = Vec::new();
    for &q in &q_levels {
        let c = catalyst_for_q(params, q);
        for &theta in &theta_levels {
            out.push(OrthogonalAssaySpec {
                label: format!("des_th{theta:.2}_q{q:.1}"),
                theta0: theta,
                precursor0: 0.0,
                catalyst0: c,
                radius: 10.0,
                dt: 1e-3,
                max_steps: 20,
                theta_stop: 1.0, // unused for desorption stop
            });
        }
    }
    out
}

/// Redistribute a fixed total material budget between bulk P and interfacial S.
///
/// `surface_fraction` ∈ (0,1) is the fraction of total mass placed on the surface.
pub fn seed_fixed_inventory_partition(
    params: &SimParams,
    radius: f64,
    total_mass: f64,
    surface_fraction: f64,
    catalyst: f64,
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
    f64,
    f64,
) {
    let grid = Grid::new();
    let n = grid.width * grid.height;
    let mut phi = vec![0.0; n];
    circular_phi_profile(&grid, radius, 2.0, &mut phi);
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    compute_interface_geometry(&grid, &phi, params.eta_n, &mut geometry);
    let mut delta_sum = 0.0;
    let mut dish_cells = 0usize;
    for idx in 0..n {
        if !grid.in_dish(idx) {
            continue;
        }
        dish_cells += 1;
        if geometry[idx].delta > params.delta_floor {
            delta_sum += geometry[idx].delta;
        }
    }
    let cell = DX * DX;
    let target_s = (surface_fraction.clamp(0.01, 0.99) * total_mass).max(0.0);
    let target_p_mass = (total_mass - target_s).max(0.0);
    let gamma = if delta_sum > 0.0 {
        (target_s / delta_sum).min(params.gamma_max * 0.99)
    } else {
        0.0
    };
    let mut s = vec![0.0; n];
    seed_surface_from_gamma(&grid, &geometry, params.delta_floor, &mut s, |_, _, _| gamma);
    let p_field = if dish_cells > 0 {
        target_p_mass / (dish_cells as f64 * cell)
    } else {
        0.0
    };
    let mut cat = vec![0.0; n];
    let mut act = vec![0.0; n];
    let mut prec = vec![0.0; n];
    let mut waste = vec![0.0; n];
    for idx in 0..n {
        if grid.in_dish(idx) {
            cat[idx] = catalyst;
            prec[idx] = p_field;
        }
    }
    let gbuf = vec![0.0; n];
    let diffusion = vec![0.0; n];
    let theta0 = surface_occupancy_theta(gamma, params.gamma_max);
    (
        grid, phi, cat, act, prec, s, waste, geometry, gbuf, diffusion, p_field, theta0,
    )
}

/// Run exchange-only equilibration from a fixed-inventory partition.
pub fn run_equilibrium_partition_assay(
    k_exchange: f64,
    k_eq: f64,
    radius: f64,
    total_mass: f64,
    surface_fraction: f64,
    catalyst: f64,
    dt: f64,
    max_steps: u64,
) -> Result<(f64, f64, f64, f64), String> {
    let params = v8_exchange_only_params(k_exchange, k_eq);
    let (
        grid,
        phi,
        catalyst_f,
        activated,
        mut precursor,
        mut s,
        mut waste,
        mut geometry,
        mut gamma,
        mut diffusion,
        _p0,
        _th0,
    ) = seed_fixed_inventory_partition(&params, radius, total_mass, surface_fraction, catalyst);
    let mut s_next = s.clone();
    let mut a_next = activated.clone();
    let mut p_next = precursor.clone();
    let total0 = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX
        + total_surface_mass(&grid, &s);
    for _ in 0..max_steps {
        let _ = evolve_surface_density(
            &grid,
            &phi,
            &catalyst_f,
            &activated,
            &precursor,
            &s,
            &params,
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
        )
        .map_err(|e| format!("evolve reject: {e:?}"))?;
        s.copy_from_slice(&s_next);
        precursor.copy_from_slice(&p_next);
    }
    reconstruct_gamma_field(&grid, &s, &geometry, params.delta_floor, &mut gamma);
    let bases = compute_exchange_local_bases(
        &grid, &precursor, &catalyst_f, &s, &geometry, &gamma, &params,
    );
    let total1 = precursor.iter().map(|v| v.max(0.0)).sum::<f64>() * DX * DX
        + total_surface_mass(&grid, &s);
    Ok((bases.mean_theta, bases.bulk_p, total0, total1))
}

/// Equilibrium isotherm check: θ/(1−θ) ≈ K p.
#[inline]
pub fn isotherm_ratio(theta: f64, p: f64, k_eq: f64) -> f64 {
    let lhs = theta / (1.0 - theta).max(1e-30);
    let rhs = k_eq * p;
    if rhs.abs() < 1e-30 {
        return f64::INFINITY;
    }
    ((lhs - rhs) / rhs).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_estimator_recovers_planted() {
        let k = 0.2;
        let k_eq = 2.5;
        let alpha = k * k_eq;
        let params = v8_exchange_only_params(k, k_eq);
        let spec = OrthogonalAssaySpec {
            label: "unit_ads".into(),
            theta0: 0.0,
            precursor0: 0.5,
            catalyst0: catalyst_for_q(&params, 0.5),
            radius: 10.0,
            dt: 1e-3,
            max_steps: 1,
            theta_stop: D030_ADS_THETA_MAX,
        };
        let r = run_orthogonal_assay(k, k_eq, &spec).expect("assay");
        assert!(r.first.net_exchange > 0.0, "{:?}", r.first);
        assert!(r.first.reverse_exchange.abs() < 1e-14, "{:?}", r.first);
        assert!(
            (r.first.alpha_estimate - alpha).abs() / alpha < 0.02,
            "got {} want {alpha}",
            r.first.alpha_estimate
        );
        assert!(r.first.accounting_residual < 1e-10);
    }

    #[test]
    fn beta_estimator_recovers_planted() {
        let k = 0.2;
        let k_eq = 2.5;
        let params = v8_exchange_only_params(k, k_eq);
        let spec = OrthogonalAssaySpec {
            label: "unit_des".into(),
            theta0: 0.5,
            precursor0: 0.0,
            catalyst0: catalyst_for_q(&params, 0.5),
            radius: 10.0,
            dt: 1e-3,
            max_steps: 1,
            theta_stop: 1.0,
        };
        let r = run_orthogonal_assay(k, k_eq, &spec).expect("assay");
        assert!(r.first.net_exchange < 0.0, "{:?}", r.first);
        assert!(r.first.forward_exchange.abs() < 1e-14, "{:?}", r.first);
        assert!(
            (r.first.beta_estimate - k).abs() / k < 0.02,
            "got {} want {k}",
            r.first.beta_estimate
        );
        assert!(r.first.accounting_residual < 1e-10);
    }
}
