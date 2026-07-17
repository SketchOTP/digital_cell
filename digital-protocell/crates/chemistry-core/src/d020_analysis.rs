//! D-020 v3 joint-rate Stage E recovery analysis.
//!
//! Mutates only the four productive rates under `membrane_metabolism_v3_structural_scaling`.
//! Topology, turnover constants, transport, yields, fields, and environment stay frozen.

use crate::d011_analysis::{
    rate_vector, sensitivity_matrix, ComponentBalance, JointBalanceMetrics, JointSolverCandidate,
    JointSolverReport, SensitivityReport, StageEReferenceRates, RATE_PARAM_NAMES, D011_G_TOL,
    D011_LOCALIZATION_MIN, D011_Q_MAX, D011_Q_MIN, D011_RETENTION_MIN,
};
use serde::{Deserialize, Serialize};

pub const D020_SENSITIVITY_PERTURB: f64 = 0.10;
pub const D020_GLOBAL_RATE_MIN_FACTOR: f64 = 0.25;
pub const D020_GLOBAL_RATE_MAX_FACTOR: f64 = 4.0;
pub const D020_ROUND_RATE_MIN_FACTOR: f64 = 0.67;
pub const D020_ROUND_RATE_MAX_FACTOR: f64 = 1.50;
pub const D020_MAX_SOLVER_ROUNDS: usize = 4;
pub const D020_MAX_CANDIDATES: usize = 6;
pub const D020_CONTAMINATION_MAX: f64 = 0.05;
pub const D020_CENTER_RADIUS: f64 = 22.0;
pub const D020_NEIGHBOR_RADII: [f64; 2] = [18.0, 26.0];
pub const D020_DIAGNOSTIC_MAX_STEPS: u64 = 5_000;
pub const D020_DIAGNOSTIC_WINDOW: u64 = 1_000;
pub const D020_FULL_MAX_STEPS: u64 = 200_000;
pub const D020_FULL_WINDOW: u64 = 10_000;

/// Analytical v3 Stage E reference: D-019 prebalance k_structure + frozen companion rates.
pub const D020_ANALYTICAL_V3_RATES: StageEReferenceRates = StageEReferenceRates {
    k_membrane: 0.5832201149734729,
    k_d008_activation: 0.07866591100881273,
    k_d008_reproduction: 0.011832646550468768,
    k_d008_structure: 0.2576268689457459,
    k_d008_activated_decay: 0.005,
    k_d008_catalyst_turnover: 0.002,
    k_structure_decay: 0.025,
};

/// Frozen (immutable under D-020) turnover / non-productive constants.
pub const D020_FROZEN_RATE_NAMES: [&str; 3] = [
    "k_d008_activated_decay",
    "k_d008_catalyst_turnover",
    "k_structure_decay",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D020Conclusion {
    D020V3StageEReferenceRecovered,
    D020NoBoundedJointRateSolution,
    D020CoupledRateSystemRankDeficient,
    D020ReferenceRemainsNonconvergent,
    D020NumericalFailure,
    D020Fail,
}

impl D020Conclusion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::D020V3StageEReferenceRecovered => "D020_V3_STAGE_E_REFERENCE_RECOVERED",
            Self::D020NoBoundedJointRateSolution => "D020_NO_BOUNDED_JOINT_RATE_SOLUTION",
            Self::D020CoupledRateSystemRankDeficient => "D020_COUPLED_RATE_SYSTEM_RANK_DEFICIENT",
            Self::D020ReferenceRemainsNonconvergent => "D020_REFERENCE_REMAINS_NONCONVERGENT",
            Self::D020NumericalFailure => "D020_NUMERICAL_FAILURE",
            Self::D020Fail => "D020_FAIL",
        }
    }
}

pub fn is_productive_rate_name(name: &str) -> bool {
    RATE_PARAM_NAMES.contains(&name)
}

pub fn is_frozen_rate_name(name: &str) -> bool {
    D020_FROZEN_RATE_NAMES.contains(&name)
}

pub fn g_vector(metrics: &JointBalanceMetrics) -> [f64; 4] {
    [
        metrics.structure.g,
        metrics.catalyst.g,
        metrics.membrane.g,
        metrics.activated.g,
    ]
}

pub fn q_vector(metrics: &JointBalanceMetrics) -> [f64; 4] {
    [
        metrics.structure.q,
        metrics.catalyst.q,
        metrics.membrane.q,
        metrics.activated.q,
    ]
}

/// Joint-flow score: ‖g‖₂ over the four constitutive flows.
pub fn joint_flow_score(metrics: &JointBalanceMetrics) -> f64 {
    g_vector(metrics).iter().map(|g| g * g).sum::<f64>().sqrt()
}

pub fn log_central_difference_with_perturb(g_up: f64, g_down: f64, perturb: f64) -> f64 {
    let ln_up = (1.0 + perturb).ln();
    let ln_down = (1.0 - perturb).ln();
    (g_up - g_down) / (ln_up - ln_down)
}

pub fn clamp_rate_to_global(value: f64, analytical: f64) -> f64 {
    let lo = analytical * D020_GLOBAL_RATE_MIN_FACTOR;
    let hi = analytical * D020_GLOBAL_RATE_MAX_FACTOR;
    value.clamp(lo, hi)
}

pub fn clamp_rates_to_global_bounds(
    rates: &StageEReferenceRates,
    analytical: &StageEReferenceRates,
) -> StageEReferenceRates {
    let mut out = *rates;
    let cur = rate_vector(rates);
    let refer = rate_vector(analytical);
    let clamped = [
        clamp_rate_to_global(cur[0], refer[0]),
        clamp_rate_to_global(cur[1], refer[1]),
        clamp_rate_to_global(cur[2], refer[2]),
        clamp_rate_to_global(cur[3], refer[3]),
    ];
    out.k_d008_structure = clamped[0];
    out.k_d008_reproduction = clamped[1];
    out.k_membrane = clamped[2];
    out.k_d008_activation = clamped[3];
    // Frozen turnovers must remain identical to analytical.
    out.k_d008_activated_decay = analytical.k_d008_activated_decay;
    out.k_d008_catalyst_turnover = analytical.k_d008_catalyst_turnover;
    out.k_structure_decay = analytical.k_structure_decay;
    out
}

pub fn rates_within_global_bounds(
    rates: &StageEReferenceRates,
    analytical: &StageEReferenceRates,
) -> bool {
    let cur = rate_vector(rates);
    let refer = rate_vector(analytical);
    cur.iter().zip(refer.iter()).all(|(v, a)| {
        *v >= *a * D020_GLOBAL_RATE_MIN_FACTOR - 1e-15
            && *v <= *a * D020_GLOBAL_RATE_MAX_FACTOR + 1e-15
    })
}

pub fn rates_within_round_factor(prev: &StageEReferenceRates, next: &StageEReferenceRates) -> bool {
    rate_vector(prev)
        .iter()
        .zip(rate_vector(next).iter())
        .all(|(p, n)| {
            let factor = *n / p.max(f64::EPSILON);
            factor >= D020_ROUND_RATE_MIN_FACTOR - 1e-12
                && factor <= D020_ROUND_RATE_MAX_FACTOR + 1e-12
        })
}

pub fn freeze_nonproductive_rates(
    rates: &StageEReferenceRates,
    analytical: &StageEReferenceRates,
) -> StageEReferenceRates {
    let mut out = *rates;
    out.k_d008_activated_decay = analytical.k_d008_activated_decay;
    out.k_d008_catalyst_turnover = analytical.k_d008_catalyst_turnover;
    out.k_structure_decay = analytical.k_structure_decay;
    out
}

pub fn only_productive_rates_differ(
    a: &StageEReferenceRates,
    b: &StageEReferenceRates,
) -> bool {
    a.k_d008_activated_decay == b.k_d008_activated_decay
        && a.k_d008_catalyst_turnover == b.k_d008_catalyst_turnover
        && a.k_structure_decay == b.k_structure_decay
}

pub fn q_corrected_rates(
    analytical: &StageEReferenceRates,
    metrics: &JointBalanceMetrics,
) -> StageEReferenceRates {
    let mut rates = *analytical;
    rates.k_d008_structure /= metrics.structure.q.max(1e-6);
    rates.k_d008_reproduction /= metrics.catalyst.q.max(1e-6);
    rates.k_membrane /= metrics.membrane.q.max(1e-6);
    rates.k_d008_activation /= metrics.activated.q.max(1e-6);
    clamp_rates_to_global_bounds(&rates, analytical)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CandidateHardGates {
    pub catalyst_retention_ok: bool,
    pub activated_retention_ok: bool,
    pub membrane_localization_ok: bool,
    pub constraint_contamination_ok: bool,
    pub no_extinction: bool,
    pub no_concentration_ceiling: bool,
    pub accounting_valid: bool,
}

impl CandidateHardGates {
    pub fn all_pass(self) -> bool {
        self.catalyst_retention_ok
            && self.activated_retention_ok
            && self.membrane_localization_ok
            && self.constraint_contamination_ok
            && self.no_extinction
            && self.no_concentration_ceiling
            && self.accounting_valid
    }
}

pub fn evaluate_hard_gates(
    metrics: &JointBalanceMetrics,
    constraint_contamination: f64,
    extinct: bool,
    concentration_ceiling: bool,
    accounting_valid: bool,
) -> CandidateHardGates {
    CandidateHardGates {
        catalyst_retention_ok: metrics.catalyst_retention >= D011_RETENTION_MIN,
        activated_retention_ok: metrics.activated_retention >= D011_RETENTION_MIN,
        membrane_localization_ok: metrics.membrane_localization >= D011_LOCALIZATION_MIN,
        constraint_contamination_ok: constraint_contamination <= D020_CONTAMINATION_MAX,
        no_extinction: !extinct,
        no_concentration_ceiling: !concentration_ceiling,
        accounting_valid,
    }
}

pub fn all_abs_g_improved(baseline: &[f64; 4], candidate: &[f64; 4]) -> bool {
    baseline
        .iter()
        .zip(candidate.iter())
        .all(|(b, c)| c.abs() < b.abs())
}

pub fn q_moving_toward_one(baseline: &[f64; 4], candidate: &[f64; 4]) -> bool {
    baseline
        .iter()
        .zip(candidate.iter())
        .all(|(b, c)| (c - 1.0).abs() <= (b - 1.0).abs() + 1e-12)
}

pub fn promotion_gate(
    baseline: &JointBalanceMetrics,
    candidate: &JointBalanceMetrics,
    hard: CandidateHardGates,
) -> bool {
    if !hard.all_pass() {
        return false;
    }
    all_abs_g_improved(&g_vector(baseline), &g_vector(candidate))
        && q_moving_toward_one(&q_vector(baseline), &q_vector(candidate))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RestoringRadiusSigns {
    pub r18_positive: bool,
    pub r22_near_zero: bool,
    pub r26_negative: bool,
}

impl RestoringRadiusSigns {
    pub fn pass(self) -> bool {
        self.r18_positive && self.r22_near_zero && self.r26_negative
    }
}

pub fn restoring_radius_signs(g18: f64, g22: f64, g26: f64) -> RestoringRadiusSigns {
    RestoringRadiusSigns {
        r18_positive: g18 > 0.0,
        r22_near_zero: g22.abs() <= D011_G_TOL.max(0.05 * g18.abs().max(g26.abs()).max(1e-6)),
        r26_negative: g26 < 0.0,
    }
}

/// Loose Stage-E restoring check used when full joint overlap is already established:
/// require sign pattern R18>0, R22≈0, R26<0 with R22 the smallest |g|.
pub fn restoring_sign_pattern_pass(g18: f64, g22: f64, g26: f64) -> bool {
    g18 > 0.0 && g26 < 0.0 && g22.abs() <= g18.abs() && g22.abs() <= g26.abs()
}

pub fn stage_e_q_gate(metrics: &JointBalanceMetrics) -> bool {
    q_vector(metrics)
        .iter()
        .all(|q| *q >= D011_Q_MIN && *q <= D011_Q_MAX)
}

pub fn stage_e_flow_gate(metrics: &JointBalanceMetrics) -> bool {
    g_vector(metrics).iter().all(|g| g.abs() <= D011_G_TOL)
}

fn solve_linear_system(matrix: &[Vec<f64>], rhs: &[f64]) -> Vec<f64> {
    let n = matrix.len();
    if n == 0 {
        return Vec::new();
    }
    let mut a = matrix.to_vec();
    let mut b = rhs.to_vec();
    for col in 0..n {
        let mut pivot = col;
        let mut max_val = 0.0;
        for row in col..n {
            let val = a[row][col].abs();
            if val > max_val {
                max_val = val;
                pivot = row;
            }
        }
        if max_val <= 1e-15 {
            continue;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        let pivot_val = a[col][col];
        for c in col..n {
            a[col][c] /= pivot_val;
        }
        b[col] /= pivot_val;
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = a[row][col];
            if factor.abs() <= 1e-15 {
                continue;
            }
            for c in col..n {
                a[row][c] -= factor * a[col][c];
            }
            b[row] -= factor * b[col];
        }
    }
    b
}

fn rates_close(a: &StageEReferenceRates, b: &StageEReferenceRates) -> bool {
    rate_vector(a)
        .iter()
        .zip(rate_vector(b))
        .all(|(x, y)| (x - y).abs() <= 1e-12 * x.abs().max(1.0))
}

pub fn solve_bounded_joint_step_d020(
    analytical: &StageEReferenceRates,
    current: &StageEReferenceRates,
    g: [f64; 4],
    sensitivity: &SensitivityReport,
    round: usize,
) -> Option<JointSolverCandidate> {
    if round >= D020_MAX_SOLVER_ROUNDS {
        return None;
    }
    let cols = sensitivity.matrix.first().map(|r| r.len()).unwrap_or(0);
    if cols == 0 {
        return None;
    }
    let rows = sensitivity.matrix.len();
    let mut ata = vec![vec![0.0; cols]; cols];
    let mut atg = vec![0.0; cols];
    for i in 0..cols {
        for j in 0..cols {
            let mut sum = 0.0;
            for row in &sensitivity.matrix {
                sum += row[i] * row[j];
            }
            ata[i][j] = sum;
        }
        let mut sum = 0.0;
        for (r, row) in sensitivity.matrix.iter().enumerate().take(rows) {
            sum += row[i] * g[r];
        }
        atg[i] = sum;
    }
    let mut dp = solve_linear_system(&ata, &atg.iter().map(|v| -*v).collect::<Vec<_>>());
    let round_min = D020_ROUND_RATE_MIN_FACTOR.ln();
    let round_max = D020_ROUND_RATE_MAX_FACTOR.ln();
    for value in &mut dp {
        *value = value.clamp(round_min, round_max);
    }
    let ref_rates = rate_vector(analytical);
    let cur_rates = rate_vector(current);
    let mut next_rates = [0.0; 4];
    let mut log_change_norm = 0.0;
    for idx in 0..4 {
        let proposed = cur_rates[idx] * dp[idx].exp();
        next_rates[idx] = clamp_rate_to_global(proposed, ref_rates[idx]);
        let delta = (next_rates[idx] / cur_rates[idx].max(f64::EPSILON)).ln();
        log_change_norm += delta * delta;
    }
    log_change_norm = log_change_norm.sqrt();
    let mut dp_arr = [0.0; 4];
    for (idx, value) in dp.iter().take(4).enumerate() {
        dp_arr[idx] = *value;
    }
    Some(JointSolverCandidate {
        round,
        rate_deltas_log: dp_arr,
        rates: freeze_nonproductive_rates(
            &StageEReferenceRates {
                k_d008_structure: next_rates[0],
                k_d008_reproduction: next_rates[1],
                k_membrane: next_rates[2],
                k_d008_activation: next_rates[3],
                k_d008_activated_decay: current.k_d008_activated_decay,
                k_d008_catalyst_turnover: current.k_d008_catalyst_turnover,
                k_structure_decay: current.k_structure_decay,
            },
            analytical,
        ),
        log_change_norm,
    })
}

pub fn bounded_joint_solver_d020(
    analytical: &StageEReferenceRates,
    start: &StageEReferenceRates,
    g_history: &[[f64; 4]],
    sensitivity_history: &[SensitivityReport],
) -> JointSolverReport {
    let mut candidates = Vec::new();
    candidates.push(JointSolverCandidate {
        round: 0,
        rate_deltas_log: [0.0; 4],
        rates: freeze_nonproductive_rates(start, analytical),
        log_change_norm: 0.0,
    });
    let mut current = freeze_nonproductive_rates(start, analytical);
    for round in 0..D020_MAX_SOLVER_ROUNDS {
        if candidates.len() >= D020_MAX_CANDIDATES {
            break;
        }
        let g = g_history.get(round).copied().unwrap_or([0.0; 4]);
        let sensitivity = sensitivity_history
            .get(round)
            .cloned()
            .unwrap_or_else(|| sensitivity_matrix(&[[0.0; 4]; 4]));
        if let Some(candidate) =
            solve_bounded_joint_step_d020(analytical, &current, g, &sensitivity, round)
        {
            if candidates
                .iter()
                .any(|c| rates_close(&c.rates, &candidate.rates))
            {
                break;
            }
            current = candidate.rates;
            candidates.push(candidate);
        } else {
            break;
        }
    }
    let bounded = candidates.len() <= D020_MAX_CANDIDATES;
    JointSolverReport {
        rounds_attempted: candidates.len().saturating_sub(1),
        candidates,
        bounded,
    }
}

pub fn select_d020_conclusion(
    rank_deficient: bool,
    numerical_failure: bool,
    any_bounded_feasible: bool,
    recovered: bool,
    reference_nonconvergent: bool,
) -> D020Conclusion {
    if numerical_failure {
        return D020Conclusion::D020NumericalFailure;
    }
    if rank_deficient && !any_bounded_feasible {
        return D020Conclusion::D020CoupledRateSystemRankDeficient;
    }
    if recovered {
        return D020Conclusion::D020V3StageEReferenceRecovered;
    }
    if !any_bounded_feasible {
        return D020Conclusion::D020NoBoundedJointRateSolution;
    }
    if reference_nonconvergent {
        return D020Conclusion::D020ReferenceRemainsNonconvergent;
    }
    D020Conclusion::D020Fail
}

/// Historical equivalence: productive rate vector layout matches D-011/D-012 ordering.
pub fn productive_rate_layout_matches_historical() -> bool {
    RATE_PARAM_NAMES
        == [
            "k_d008_structure",
            "k_d008_reproduction",
            "k_membrane",
            "k_d008_activation",
        ]
}

pub fn placeholder_metrics(g: [f64; 4], q: [f64; 4]) -> JointBalanceMetrics {
    JointBalanceMetrics {
        structure: ComponentBalance {
            q: q[0],
            g: g[0],
            production: 0.0,
            loss: 0.0,
        },
        catalyst: ComponentBalance {
            q: q[1],
            g: g[1],
            production: 0.0,
            loss: 0.0,
        },
        membrane: ComponentBalance {
            q: q[2],
            g: g[2],
            production: 0.0,
            loss: 0.0,
        },
        activated: ComponentBalance {
            q: q[3],
            g: g[3],
            production: 0.0,
            loss: 0.0,
        },
        catalyst_retention: 0.9,
        activated_retention: 0.9,
        membrane_localization: 0.95,
        nutrient_influx: 1.0,
        fuel_influx: 1.0,
        waste_efflux: 1.0,
    }
}
