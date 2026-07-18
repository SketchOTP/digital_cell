//! D-028 bracketed surface-renewal root recovery.
//!
//! Deterministic regula-falsi / bisection inside the frozen D-027
//! `k_ads ∈ [1×, 2×]` interval. Only scalar `k_ads` may change before Gate 9.

use serde::{Deserialize, Serialize};

/// Exact machine values from D-027 `analytical_candidates.json` / `isolated_surface.json`.
pub const D028_K_ADS_0_5X: f64 = 0.01688873302573429;
pub const D028_K_ADS_1X: f64 = 0.03377746605146858;
pub const D028_K_ADS_2X: f64 = 0.06755493210293716;

/// Frozen D-027 late-window Q at the three mandated candidates (12k isolated screen).
pub const D028_Q_0_5X: f64 = 0.6145455496162924;
pub const D028_Q_1X: f64 = 0.9064945432686394;
pub const D028_Q_2X: f64 = 1.0568472407802698;

pub const D028_Q_BALANCE_LO: f64 = 0.98;
pub const D028_Q_BALANCE_HI: f64 = 1.02;
pub const D028_G_SURFACE_MAX: f64 = 1.0e-4;
pub const D028_MAX_NEW_CANDIDATES: usize = 4;
/// Stop when bracket width < 1% of midpoint and no root found.
pub const D028_BRACKET_WIDTH_FRAC: f64 = 0.01;
/// Monotonicity: increasing k must not decrease Q beyond this absolute tolerance.
pub const D028_MONOTONIC_TOL: f64 = 1.0e-6;
/// Gate 0 reproduction tolerance on Q vs frozen D-027 artifact values.
pub const D028_GATE0_Q_REPRO_TOL: f64 = 0.05;
pub const D028_PORTABILITY_Q_LO: f64 = 0.90;
pub const D028_PORTABILITY_Q_HI: f64 = 1.10;
pub const D028_PORTABILITY_MIN_PASS: usize = 5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SurfaceBalanceMetrics {
    pub q_surface: f64,
    pub g_surface: f64,
    pub f_balance: f64,
}

impl SurfaceBalanceMetrics {
    pub fn from_rates(adsorption_rate: f64, gamma_turnover_rate: f64, mean_s_mass: f64) -> Self {
        let q = adsorption_rate / gamma_turnover_rate.max(f64::EPSILON);
        // Rates are already mass/time; g = net_S_rate / mean_S (= ΔS_net / (mean_S · Δt)).
        let g = (adsorption_rate - gamma_turnover_rate) / mean_s_mass.max(f64::EPSILON);
        Self {
            q_surface: q,
            g_surface: g,
            f_balance: q - 1.0,
        }
    }

    pub fn is_balanced(&self) -> bool {
        self.q_surface >= D028_Q_BALANCE_LO
            && self.q_surface <= D028_Q_BALANCE_HI
            && self.g_surface.abs() <= D028_G_SURFACE_MAX
    }
}

/// Safeguarded regula-falsi proposal toward Q=1.
pub fn regula_falsi_trial(k_low: f64, q_low: f64, k_high: f64, q_high: f64) -> f64 {
    let denom = q_high - q_low;
    if !denom.is_finite() || denom.abs() < f64::EPSILON {
        return 0.5 * (k_low + k_high);
    }
    k_low + (1.0 - q_low) * (k_high - k_low) / denom
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalMethod {
    RegulaFalsi,
    Bisection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BracketEndpoints {
    pub k_low: f64,
    pub q_low: f64,
    pub k_high: f64,
    pub q_high: f64,
}

impl BracketEndpoints {
    pub fn from_d027_1x_2x() -> Self {
        Self {
            k_low: D028_K_ADS_1X,
            q_low: D028_Q_1X,
            k_high: D028_K_ADS_2X,
            q_high: D028_Q_2X,
        }
    }

    pub fn straddles_unity(&self) -> bool {
        self.q_low < 1.0 && self.q_high > 1.0
    }

    pub fn width(&self) -> f64 {
        self.k_high - self.k_low
    }

    pub fn midpoint(&self) -> f64 {
        0.5 * (self.k_low + self.k_high)
    }

    pub fn too_narrow(&self) -> bool {
        let mid = self.midpoint().abs().max(f64::EPSILON);
        self.width() < D028_BRACKET_WIDTH_FRAC * mid
    }

    /// Propose next k inside the open bracket; fall back to bisection if needed.
    pub fn propose(&self, previous_k: Option<f64>) -> (f64, ProposalMethod, Option<&'static str>) {
        let mut k = regula_falsi_trial(self.k_low, self.q_low, self.k_high, self.q_high);
        let mut method = ProposalMethod::RegulaFalsi;
        let mut reason = None;

        let repeats_endpoint = (k - self.k_low).abs() <= f64::EPSILON * self.k_low.abs().max(1.0)
            || (k - self.k_high).abs() <= f64::EPSILON * self.k_high.abs().max(1.0);
        let repeats_previous = previous_k
            .map(|p| (k - p).abs() <= 1e-15 * k.abs().max(1.0))
            .unwrap_or(false);
        let outside = k <= self.k_low || k >= self.k_high;
        let poorly_ordered = self.q_high <= self.q_low;

        if repeats_endpoint || repeats_previous || outside || poorly_ordered || !k.is_finite() {
            k = self.midpoint();
            method = ProposalMethod::Bisection;
            reason = Some(if outside {
                "interpolation_outside_bracket"
            } else if repeats_endpoint {
                "interpolation_repeats_endpoint"
            } else if repeats_previous {
                "interpolation_repeats_previous"
            } else if poorly_ordered {
                "q_high_not_greater_than_q_low"
            } else {
                "non_finite_interpolation"
            });
        }
        // Hard clamp: never evaluate outside the bracket.
        let eps = 1e-15 * self.midpoint().abs().max(1.0);
        k = k.clamp(self.k_low + eps, self.k_high - eps);
        (k, method, reason)
    }

    /// Replace only the endpoint on the same side of Q=1; preserve sign-changing bracket.
    pub fn update_with_observation(&self, k: f64, q: f64) -> Result<Self, &'static str> {
        if k <= self.k_low || k >= self.k_high {
            return Err("candidate_outside_bracket");
        }
        let mut next = *self;
        if q < 1.0 {
            next.k_low = k;
            next.q_low = q;
        } else {
            next.k_high = k;
            next.q_high = q;
        }
        if !(next.q_low < 1.0 && next.q_high > 1.0) {
            return Err("bracket_lost_sign_change");
        }
        if next.k_low >= next.k_high {
            return Err("bracket_collapsed");
        }
        Ok(next)
    }
}

/// Verify Q increases with k over the three frozen D-027 points, then optional midpoint.
pub fn adsorption_response_monotonic(q_values_ascending_k: &[f64]) -> bool {
    if q_values_ascending_k.len() < 2 {
        return true;
    }
    for w in q_values_ascending_k.windows(2) {
        if w[1] + D028_MONOTONIC_TOL < w[0] {
            return false;
        }
    }
    true
}

pub fn frozen_d027_monotonicity_holds() -> bool {
    adsorption_response_monotonic(&[D028_Q_0_5X, D028_Q_1X, D028_Q_2X])
}

/// Gate 0 acceptance on a reproduced isolated candidate.
pub fn gate0_endpoint_ok(k_ads: f64, q: f64, expect_below: bool) -> bool {
    let finite = k_ads.is_finite() && q.is_finite();
    let side = if expect_below {
        q < 0.98
    } else {
        q > 1.02
    };
    finite && side
}

/// Select smallest k among balanced candidates.
pub fn select_smallest_passing(candidates: &[(f64, bool)]) -> Option<f64> {
    let mut passing: Vec<f64> = candidates
        .iter()
        .filter(|(_, pass)| *pass)
        .map(|(k, _)| *k)
        .collect();
    if passing.is_empty() {
        return None;
    }
    passing.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(passing[0])
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RootIterationRecord {
    pub iteration: usize,
    pub bracket_before: BracketEndpoints,
    pub method: ProposalMethod,
    pub fallback_reason: Option<String>,
    pub k_trial: f64,
    pub q_surface: f64,
    pub g_surface: f64,
    pub balanced: bool,
    pub bracket_after: Option<BracketEndpoints>,
    pub termination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BracketSolveResult {
    pub initial_bracket: BracketEndpoints,
    pub iterations: Vec<RootIterationRecord>,
    pub selected_k_ads: Option<f64>,
    pub new_candidate_count: usize,
    pub conclusion: String,
    pub pass: bool,
}

/// Pure bracketed solve given an evaluator `f(k) -> (q, g)`.
///
/// Evaluates at most [`D028_MAX_NEW_CANDIDATES`] new points. Does not re-evaluate
/// the frozen D-027 endpoints (those seed the initial bracket).
pub fn solve_bracketed_root<E>(
    initial: BracketEndpoints,
    mut evaluate: E,
) -> BracketSolveResult
where
    E: FnMut(f64) -> (f64, f64),
{
    let mut bracket = initial;
    let mut iterations = Vec::new();
    let mut previous_k: Option<f64> = None;
    let mut passing: Vec<(f64, bool)> = Vec::new();

    if !bracket.straddles_unity() {
        return BracketSolveResult {
            initial_bracket: initial,
            iterations,
            selected_k_ads: None,
            new_candidate_count: 0,
            conclusion: "D028_ROOT_BRACKET_NOT_REPRODUCED".into(),
            pass: false,
        };
    }

    for i in 0..D028_MAX_NEW_CANDIDATES {
        if bracket.too_narrow() {
            let selected = select_smallest_passing(&passing);
            return BracketSolveResult {
                initial_bracket: initial,
                iterations,
                selected_k_ads: selected,
                new_candidate_count: i,
                conclusion: if selected.is_some() {
                    "D028_ISOLATED_ROOT_FOUND".into()
                } else {
                    "D028_NO_ISOLATED_SURFACE_BALANCE_ROOT".into()
                },
                pass: selected.is_some(),
            };
        }

        let bracket_before = bracket;
        let (k_trial, method, reason) = bracket.propose(previous_k);
        debug_assert!(k_trial > bracket.k_low && k_trial < bracket.k_high);

        let (q, g) = evaluate(k_trial);
        let metrics = SurfaceBalanceMetrics {
            q_surface: q,
            g_surface: g,
            f_balance: q - 1.0,
        };
        let balanced = metrics.is_balanced();
        passing.push((k_trial, balanced));
        previous_k = Some(k_trial);

        if balanced {
            iterations.push(RootIterationRecord {
                iteration: i + 1,
                bracket_before,
                method,
                fallback_reason: reason.map(str::to_string),
                k_trial,
                q_surface: q,
                g_surface: g,
                balanced: true,
                bracket_after: None,
                termination: "balanced".into(),
            });
            let selected = select_smallest_passing(&passing);
            return BracketSolveResult {
                initial_bracket: initial,
                iterations,
                selected_k_ads: selected,
                new_candidate_count: i + 1,
                conclusion: "D028_ISOLATED_ROOT_FOUND".into(),
                pass: true,
            };
        }

        match bracket.update_with_observation(k_trial, q) {
            Ok(next) => {
                iterations.push(RootIterationRecord {
                    iteration: i + 1,
                    bracket_before,
                    method,
                    fallback_reason: reason.map(str::to_string),
                    k_trial,
                    q_surface: q,
                    g_surface: g,
                    balanced: false,
                    bracket_after: Some(next),
                    termination: "continue".into(),
                });
                bracket = next;
            }
            Err(e) => {
                iterations.push(RootIterationRecord {
                    iteration: i + 1,
                    bracket_before,
                    method,
                    fallback_reason: reason.map(str::to_string),
                    k_trial,
                    q_surface: q,
                    g_surface: g,
                    balanced: false,
                    bracket_after: None,
                    termination: e.to_string(),
                });
                return BracketSolveResult {
                    initial_bracket: initial,
                    iterations,
                    selected_k_ads: select_smallest_passing(&passing),
                    new_candidate_count: i + 1,
                    conclusion: "D028_NO_ISOLATED_SURFACE_BALANCE_ROOT".into(),
                    pass: false,
                };
            }
        }
    }

    let selected = select_smallest_passing(&passing);
    let conclusion = if selected.is_some() {
        "D028_ISOLATED_ROOT_FOUND"
    } else if bracket.too_narrow() {
        "D028_NO_ISOLATED_SURFACE_BALANCE_ROOT"
    } else {
        "D028_NO_ISOLATED_SURFACE_BALANCE_ROOT"
    };
    BracketSolveResult {
        initial_bracket: initial,
        iterations,
        selected_k_ads: selected,
        new_candidate_count: D028_MAX_NEW_CANDIDATES,
        conclusion: conclusion.into(),
        pass: selected.is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_machine_candidates_match_d027_artifacts() {
        assert!((D028_K_ADS_1X - 0.03377746605146858).abs() < 1e-18);
        assert!((D028_K_ADS_2X - 0.06755493210293716).abs() < 1e-18);
        assert!((D028_K_ADS_0_5X * 2.0 - D028_K_ADS_1X).abs() < 1e-18);
        assert!((D028_K_ADS_1X * 2.0 - D028_K_ADS_2X).abs() < 1e-18);
    }

    #[test]
    fn frozen_monotonicity_and_bracket_straddle() {
        assert!(frozen_d027_monotonicity_holds());
        let b = BracketEndpoints::from_d027_1x_2x();
        assert!(b.straddles_unity());
        assert!(b.q_low < 0.98);
        assert!(b.q_high > 1.02);
    }

    #[test]
    fn regula_falsi_near_expected_trial() {
        let b = BracketEndpoints::from_d027_1x_2x();
        let k = regula_falsi_trial(b.k_low, b.q_low, b.k_high, b.q_high);
        // Rounded evidence ≈ 0.0548; exact from machine Q.
        assert!((k - 0.0548).abs() < 5e-4);
        assert!(k > b.k_low && k < b.k_high);
    }

    #[test]
    fn bisection_fallback_on_repeat_endpoint() {
        let b = BracketEndpoints {
            k_low: 0.03,
            q_low: 0.0, // forces regula to k_low + 1*(0.04)/1 = 0.07? wait
            k_high: 0.07,
            q_high: 2.0,
        };
        // q_low=0, q_high=2 → k = 0.03 + 1*0.04/2 = 0.05 (fine)
        let (k, method, _) = b.propose(Some(0.05));
        // Force repeat of previous → bisection
        assert_eq!(method, ProposalMethod::Bisection);
        assert!((k - 0.05).abs() < 1e-12 || (k - b.midpoint()).abs() < 1e-12);
    }

    #[test]
    fn bracket_update_preserves_sign_change() {
        let b = BracketEndpoints::from_d027_1x_2x();
        let next = b.update_with_observation(0.05, 0.99).unwrap();
        assert!(next.straddles_unity());
        assert!((next.k_low - 0.05).abs() < 1e-15);
        assert!((next.q_low - 0.99).abs() < 1e-15);
    }

    #[test]
    fn rejects_out_of_bracket_and_selects_smallest() {
        let b = BracketEndpoints::from_d027_1x_2x();
        assert!(b.update_with_observation(0.01, 0.5).is_err());
        assert_eq!(
            select_smallest_passing(&[(0.06, true), (0.05, true), (0.055, false)]),
            Some(0.05)
        );
    }

    #[test]
    fn q_g_common_window_balance() {
        let m = SurfaceBalanceMetrics::from_rates(1.0, 1.0, 100.0);
        assert!(m.is_balanced());
        assert!((m.g_surface).abs() < 1e-15);
        let m2 = SurfaceBalanceMetrics::from_rates(1.1, 1.0, 100.0);
        assert!(!m2.is_balanced()); // Q=1.1 out of band
    }

    #[test]
    fn solver_finds_root_on_linear_q() {
        // Synthetic: Q(k) = 0.5 + 10*(k - 0.03)  → Q=1 at k=0.08; bracket [0.05,0.09]
        // Better: use D027-like linear interpolate between endpoints.
        let initial = BracketEndpoints {
            k_low: 0.03,
            q_low: 0.90,
            k_high: 0.07,
            q_high: 1.10,
        };
        // Linear: Q = 0.90 + (k-0.03)/(0.04)*0.20 → Q=1 at k=0.05
        let result = solve_bracketed_root(initial, |k| {
            let q = 0.90 + (k - 0.03) / 0.04 * 0.20;
            let g = (q - 1.0) * 1e-6; // tiny g so balance is Q-dominated
            (q, g)
        });
        assert!(result.pass, "{:?}", result.conclusion);
        let k = result.selected_k_ads.unwrap();
        assert!((k - 0.05).abs() < 1e-6, "k={k}");
        assert!(result.new_candidate_count <= D028_MAX_NEW_CANDIDATES);
        assert!(result.iterations.iter().all(|it| {
            it.k_trial > initial.k_low - 1e-15 && it.k_trial < initial.k_high + 1e-15
        }));
    }

    #[test]
    fn solver_stops_when_narrow_without_root() {
        let initial = BracketEndpoints {
            k_low: 1.0,
            q_low: 0.5,
            k_high: 1.005, // already <1% of mid≈1.0025? width=0.005, 1% mid≈0.01 → not yet
            q_high: 1.5,
        };
        // Pathological: Q jumps over balance band without landing in [0.98,1.02]
        let result = solve_bracketed_root(initial, |k| {
            let q = if k < 1.0025 { 0.5 } else { 1.5 };
            (q, 1.0) // large |g| always fails balance
        });
        assert!(!result.pass);
        assert_eq!(result.conclusion, "D028_NO_ISOLATED_SURFACE_BALANCE_ROOT");
        assert!(result.new_candidate_count <= D028_MAX_NEW_CANDIDATES);
    }

    #[test]
    fn nonmonotonic_detection() {
        assert!(!adsorption_response_monotonic(&[0.6, 0.9, 0.8]));
        assert!(adsorption_response_monotonic(&[0.6, 0.9, 1.05]));
    }
}
