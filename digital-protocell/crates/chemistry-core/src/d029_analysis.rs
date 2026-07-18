//! D-029 reversible bulk–surface exchange: identification, candidates, gates.

use crate::config::SimParams;
use crate::membrane::membrane_catalyst_saturation;
use crate::simulation::Simulation;
use crate::surface_density::{
    compute_interface_geometry, reconstruct_gamma_field, surface_occupancy_theta,
    InterfaceGeometryCell,
};
use serde::{Deserialize, Serialize};

/// Maximum exchange candidates (fitted center + four axis perturbations).
pub const D029_MAX_CANDIDATES: usize = 5;
/// Identifiability: condition number ceiling.
pub const D029_COND_MAX: f64 = 1.0e6;
/// Median relative prediction error ceiling.
pub const D029_MEDIAN_REL_ERR_MAX: f64 = 0.15;
/// Maximum relative prediction error ceiling.
pub const D029_MAX_REL_ERR_MAX: f64 = 0.35;
/// Leave-one-out must stay within this factor of the full fit.
pub const D029_LOO_FACTOR_MAX: f64 = 2.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExchangeBasisRow {
    pub label: String,
    /// A_i = ∫ δ Γ_max q(C) p (1−θ) dV
    pub a_integral: f64,
    /// B_i = ∫ δ Γ_max q(C) θ dV
    pub b_integral: f64,
    /// L_i = biological Γ turnover ∫ δ k_Γ_decay Γ dV
    pub l_turnover: f64,
    pub finite: bool,
}

/// Compute A_i, B_i, L_i for one state under the reversible-exchange basis.
pub fn compute_exchange_basis_labeled(sim: &Simulation, label: &str) -> ExchangeBasisRow {
    let n = sim.grid.width * sim.grid.height;
    let mut geometry = vec![InterfaceGeometryCell::default(); n];
    let mut gamma = vec![0.0; n];
    compute_interface_geometry(
        &sim.grid,
        &sim.fields.structure,
        sim.params.eta_n,
        &mut geometry,
    );
    reconstruct_gamma_field(
        &sim.grid,
        &sim.fields.membrane,
        &geometry,
        sim.params.delta_floor,
        &mut gamma,
    );
    let gamma_max = sim.params.gamma_max.max(0.0);
    let pref = if sim.params.p_reference > 0.0 {
        sim.params.p_reference
    } else {
        1.0
    };
    let k_decay = sim.params.k_gamma_decay;
    let mut a = 0.0;
    let mut b = 0.0;
    let mut l = 0.0;
    for idx in 0..n {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let delta = geometry[idx].delta;
        if delta <= sim.params.delta_floor {
            continue;
        }
        let g = gamma[idx].max(0.0);
        let p = sim.fields.precursor[idx].max(0.0) / pref;
        let c = sim.fields.catalyst[idx].max(0.0);
        let q_c = membrane_catalyst_saturation(c, &sim.params);
        let theta = surface_occupancy_theta(g, gamma_max);
        let sat = (1.0 - theta).max(0.0);
        a += delta * gamma_max * q_c * p * sat;
        b += delta * gamma_max * q_c * theta;
        l += delta * k_decay * g;
    }
    let finite = a.is_finite() && b.is_finite() && l.is_finite();
    ExchangeBasisRow {
        label: label.to_string(),
        a_integral: a,
        b_integral: b,
        l_turnover: l,
        finite,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExchangeFitResult {
    pub alpha: f64,
    pub beta: f64,
    pub k_exchange: f64,
    pub k_exchange_eq: f64,
    pub rank: usize,
    pub singular_values: [f64; 2],
    pub condition_number: f64,
    pub residuals: Vec<f64>,
    pub relative_errors: Vec<f64>,
    pub weighted_residual_norm: f64,
    pub median_rel_err: f64,
    pub max_rel_err: f64,
    pub direction_correct_count: usize,
    pub identifiable: bool,
    pub conclusion: String,
}

/// Weight that prevents large-interface states from dominating solely by scale.
pub fn exchange_row_weight(row: &ExchangeBasisRow) -> f64 {
    let scale = (row.a_integral.abs() + row.b_integral.abs()).max(1e-30);
    1.0 / scale.sqrt()
}

/// Weighted nonnegative least squares for L ≈ α A − β B with α,β ≥ 0.
///
/// Solved in unconstrained LS on design [wA, −wB] then projected to the nonnegative orthant.
/// If a coefficient goes negative, refit with that column removed (active-set of size 1).
pub fn fit_exchange_nnls(rows: &[ExchangeBasisRow]) -> ExchangeFitResult {
    let n = rows.len();
    let mut a_col = Vec::with_capacity(n);
    let mut b_col = Vec::with_capacity(n);
    let mut l_col = Vec::with_capacity(n);
    let mut weights = Vec::with_capacity(n);
    for r in rows {
        let w = exchange_row_weight(r);
        weights.push(w);
        a_col.push(w * r.a_integral);
        b_col.push(w * (-r.b_integral));
        l_col.push(w * r.l_turnover);
    }

    let (alpha0, beta0, sv, cond, rank) = solve_2col_ls(&a_col, &b_col, &l_col);
    let (alpha, beta) = project_nonnegative_2(alpha0, beta0, &a_col, &b_col, &l_col);

    let mut residuals = Vec::with_capacity(n);
    let mut rel_errs = Vec::with_capacity(n);
    let mut wres2 = 0.0;
    let mut dir_ok = 0usize;
    for (i, r) in rows.iter().enumerate() {
        let pred = alpha * r.a_integral - beta * r.b_integral;
        let res = pred - r.l_turnover;
        residuals.push(res);
        let denom = r.l_turnover.abs().max(1e-30);
        rel_errs.push((res / denom).abs());
        let w = weights[i];
        wres2 += (w * res).powi(2);
        // Predicted net exchange direction vs turnover demand sign.
        let net_pred = pred;
        if net_pred.signum() == r.l_turnover.signum()
            || (net_pred.abs() < 1e-18 && r.l_turnover.abs() < 1e-18)
        {
            dir_ok += 1;
        }
    }
    let mut sorted_rel = rel_errs.clone();
        sorted_rel.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted_rel.is_empty() {
        f64::INFINITY
    } else if sorted_rel.len() % 2 == 1 {
        sorted_rel[sorted_rel.len() / 2]
    } else {
        0.5 * (sorted_rel[sorted_rel.len() / 2 - 1] + sorted_rel[sorted_rel.len() / 2])
    };
    let max_rel = rel_errs.iter().copied().fold(0.0_f64, f64::max);

    let k_exchange = beta;
    let k_exchange_eq = if beta > 0.0 { alpha / beta } else { f64::NAN };

    let identifiable = rank == 2
        && alpha.is_finite()
        && beta.is_finite()
        && alpha > 0.0
        && beta > 0.0
        && k_exchange_eq.is_finite()
        && k_exchange_eq > 0.0
        && cond.is_finite()
        && cond < D029_COND_MAX
        && median <= D029_MEDIAN_REL_ERR_MAX
        && max_rel <= D029_MAX_REL_ERR_MAX
        && dir_ok >= 5;

    let conclusion = if identifiable {
        "D029_EXCHANGE_IDENTIFIABLE".into()
    } else if rank < 2
        || !cond.is_finite()
        || cond >= D029_COND_MAX
        || !(alpha > 0.0 && beta > 0.0)
        || !k_exchange_eq.is_finite()
        || !(k_exchange_eq > 0.0)
    {
        "D029_REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE".into()
    } else {
        "D029_REVERSIBLE_EXCHANGE_NOT_PORTABLE".into()
    };

    ExchangeFitResult {
        alpha,
        beta,
        k_exchange,
        k_exchange_eq,
        rank,
        singular_values: sv,
        condition_number: cond,
        residuals,
        relative_errors: rel_errs,
        weighted_residual_norm: wres2.sqrt(),
        median_rel_err: median,
        max_rel_err: max_rel,
        direction_correct_count: dir_ok,
        identifiable,
        conclusion,
    }
}

fn solve_2col_ls(a: &[f64], b: &[f64], y: &[f64]) -> (f64, f64, [f64; 2], f64, usize) {
    // Normal equations: G x = g with G = XᵀX, g = Xᵀy, X = [a b].
    let mut gaa = 0.0;
    let mut gab = 0.0;
    let mut gbb = 0.0;
    let mut ga = 0.0;
    let mut gb = 0.0;
    for i in 0..a.len() {
        gaa += a[i] * a[i];
        gab += a[i] * b[i];
        gbb += b[i] * b[i];
        ga += a[i] * y[i];
        gb += b[i] * y[i];
    }
    let det = gaa * gbb - gab * gab;
    let (sv1, sv2) = eig2_symm(gaa, gab, gbb);
    let smax = sv1.max(sv2).max(0.0);
    let smin = sv1.min(sv2).max(0.0);
    let rank = if smax <= 0.0 {
        0
    } else if smin / smax < 1e-14 {
        1
    } else {
        2
    };
    let cond = if smin > 0.0 {
        smax / smin
    } else {
        f64::INFINITY
    };
    if det.abs() < 1e-30 {
        return (0.0, 0.0, [sv1, sv2], cond, rank);
    }
    let alpha = (gbb * ga - gab * gb) / det;
    let beta = (gaa * gb - gab * ga) / det;
    (alpha, beta, [sv1, sv2], cond, rank)
}

fn eig2_symm(a: f64, c: f64, b: f64) -> (f64, f64) {
    // Eigenvalues of [[a,c],[c,b]]
    let tr = a + b;
    let det = a * b - c * c;
    let disc = (tr * tr - 4.0 * det).max(0.0).sqrt();
    ((tr + disc) * 0.5, (tr - disc) * 0.5)
}

fn project_nonnegative_2(alpha0: f64, beta0: f64, a: &[f64], b: &[f64], y: &[f64]) -> (f64, f64) {
    if alpha0 >= 0.0 && beta0 >= 0.0 {
        return (alpha0, beta0);
    }
    // Try single-column nonnegative fits.
    let fit_a = nn_single(a, y);
    let fit_b = nn_single(b, y);
    let err = |alpha: f64, beta: f64| -> f64 {
        let mut e = 0.0;
        for i in 0..a.len() {
            let r = alpha * a[i] + beta * b[i] - y[i];
            e += r * r;
        }
        e
    };
    let candidates = [
        (alpha0.max(0.0), beta0.max(0.0)),
        (fit_a, 0.0),
        (0.0, fit_b),
        (0.0, 0.0),
    ];
    let mut best = candidates[0];
    let mut best_e = err(best.0, best.1);
    for c in &candidates[1..] {
        let e = err(c.0, c.1);
        if e < best_e {
            best = *c;
            best_e = e;
        }
    }
    best
}

fn nn_single(x: &[f64], y: &[f64]) -> f64 {
    let mut xx = 0.0;
    let mut xy = 0.0;
    for i in 0..x.len() {
        xx += x[i] * x[i];
        xy += x[i] * y[i];
    }
    if xx <= 0.0 {
        0.0
    } else {
        (xy / xx).max(0.0)
    }
}

/// Leave-one-out stability: each LOO (α,β) within factor of full fit.
pub fn leave_one_out_stable(rows: &[ExchangeBasisRow], full: &ExchangeFitResult) -> (bool, Vec<(f64, f64)>) {
    let mut loo = Vec::new();
    let mut ok = true;
    for i in 0..rows.len() {
        let subset: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, r)| r.clone())
            .collect();
        let fit = fit_exchange_nnls(&subset);
        loo.push((fit.k_exchange, fit.k_exchange_eq));
        if !fit.k_exchange.is_finite() || !fit.k_exchange_eq.is_finite() {
            ok = false;
            continue;
        }
        let f_k = full.k_exchange.max(1e-30);
        let f_K = full.k_exchange_eq.max(1e-30);
        let rk = (fit.k_exchange / f_k).max(f_k / fit.k_exchange);
        let rK = (fit.k_exchange_eq / f_K).max(f_K / fit.k_exchange_eq);
        if rk > D029_LOO_FACTOR_MAX || rK > D029_LOO_FACTOR_MAX {
            ok = false;
        }
    }
    (ok, loo)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExchangeCandidate {
    pub identity: String,
    pub k_exchange: f64,
    pub k_exchange_eq: f64,
}

/// Generate ≤5 candidates: center + ±20% K + ±25% k (no Cartesian product).
pub fn generate_exchange_candidates(fit: &ExchangeFitResult) -> Vec<ExchangeCandidate> {
    let k = fit.k_exchange;
    let k_eq = fit.k_exchange_eq;
    vec![
        ExchangeCandidate {
            identity: "fitted_center".into(),
            k_exchange: k,
            k_exchange_eq: k_eq,
        },
        ExchangeCandidate {
            identity: "K_minus_20".into(),
            k_exchange: k,
            k_exchange_eq: k_eq * 0.80,
        },
        ExchangeCandidate {
            identity: "K_plus_20".into(),
            k_exchange: k,
            k_exchange_eq: k_eq * 1.20,
        },
        ExchangeCandidate {
            identity: "k_minus_25".into(),
            k_exchange: k * 0.75,
            k_exchange_eq: k_eq,
        },
        ExchangeCandidate {
            identity: "k_plus_25".into(),
            k_exchange: k * 1.25,
            k_exchange_eq: k_eq,
        },
    ]
}

/// Log-parameter distance to fitted center (tie-break: lower k_exchange).
pub fn candidate_log_distance(c: &ExchangeCandidate, fit: &ExchangeFitResult) -> f64 {
    let dk = (c.k_exchange.max(1e-30).ln() - fit.k_exchange.max(1e-30).ln()).abs();
    let dK = (c.k_exchange_eq.max(1e-30).ln() - fit.k_exchange_eq.max(1e-30).ln()).abs();
    dk + dK
}

/// Apply exchange params onto a SimParams (v8).
pub fn apply_exchange_candidate(params: &mut SimParams, c: &ExchangeCandidate) {
    params.equation_version =
        crate::config::EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange;
    params.k_exchange = c.k_exchange;
    params.k_exchange_eq = c.k_exchange_eq;
    params.p_reference = 1.0;
    // k_ads unused under v8; keep zero to avoid silent irreversible fallback.
    params.k_ads = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nnls_recovers_known_alpha_beta() {
        // L = 2 A − 1 B
        let rows: Vec<ExchangeBasisRow> = (0..6)
            .map(|i| {
                let a = 1.0 + 0.2 * i as f64;
                let b = 0.5 + 0.1 * i as f64;
                ExchangeBasisRow {
                    label: format!("s{i}"),
                    a_integral: a,
                    b_integral: b,
                    l_turnover: 2.0 * a - 1.0 * b,
                    finite: true,
                }
            })
            .collect();
        let fit = fit_exchange_nnls(&rows);
        assert!(fit.identifiable, "{fit:?}");
        assert!((fit.alpha - 2.0).abs() < 1e-9);
        assert!((fit.beta - 1.0).abs() < 1e-9);
        assert!((fit.k_exchange_eq - 2.0).abs() < 1e-9);
        assert_eq!(generate_exchange_candidates(&fit).len(), D029_MAX_CANDIDATES);
    }

    #[test]
    fn candidate_generation_is_not_cartesian() {
        let fit = ExchangeFitResult {
            alpha: 2.0,
            beta: 1.0,
            k_exchange: 1.0,
            k_exchange_eq: 2.0,
            rank: 2,
            singular_values: [1.0, 1.0],
            condition_number: 1.0,
            residuals: vec![],
            relative_errors: vec![],
            weighted_residual_norm: 0.0,
            median_rel_err: 0.0,
            max_rel_err: 0.0,
            direction_correct_count: 6,
            identifiable: true,
            conclusion: "ok".into(),
        };
        let c = generate_exchange_candidates(&fit);
        assert_eq!(c.len(), 5);
        assert_eq!(c[1].k_exchange, 1.0);
        assert!((c[1].k_exchange_eq - 1.6).abs() < 1e-12);
    }
}
