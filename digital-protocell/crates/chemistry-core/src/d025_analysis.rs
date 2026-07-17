//! D-025 v7 surface-density Stage E joint-balance analysis and bounded solver.

use crate::d011_analysis::{
    sensitivity_matrix, ComponentBalance, JointBalanceMetrics, JointSolverCandidate,
    JointSolverReport, SensitivityReport, StageEReferenceRates, D011_G_TOL, D011_Q_MAX,
    D011_Q_MIN, D011_RETENTION_MIN,
};
use crate::d013_harness::{ScientificClassification, TerminationReason};
use crate::d020_analysis::{log_central_difference_with_perturb, g_vector, q_vector};
use serde::{Deserialize, Serialize};

pub const D025_FROZEN_K_ADS: f64 = 0.0011111111111111111;
pub const D025_CENTER_RADIUS: f64 = 22.0;
pub const D025_NEIGHBOR_RADII: [f64; 2] = [18.0, 26.0];
pub const D025_WINDOW: u64 = 10_000;
pub const D025_REQUIRED_WINDOWS: u64 = 3;
pub const D025_DIAGNOSTIC_MAX_STEPS: u64 = 50_000;
pub const D025_DIAGNOSTIC_WINDOW: u64 = 5_000;
pub const D025_FULL_MAX_STEPS: u64 = 200_000;
pub const D025_SENSITIVITY_PERTURB: f64 = 0.10;
pub const D025_GLOBAL_RATE_MIN_FACTOR: f64 = 0.25;
pub const D025_GLOBAL_RATE_MAX_FACTOR: f64 = 4.0;
pub const D025_ROUND_RATE_MIN_FACTOR: f64 = 0.67;
pub const D025_ROUND_RATE_MAX_FACTOR: f64 = 1.50;
pub const D025_MAX_SOLVER_ROUNDS: usize = 4;
pub const D025_MAX_CANDIDATES: usize = 5;
pub const D025_LOCALIZATION_MIN: f64 = 0.95;
pub const D025_CONTAMINATION_MAX: f64 = 0.05;

pub const D025_PRODUCTIVE_NAMES: [&str; 4] = [
    "k_d008_activation",
    "k_d008_reproduction",
    "k_precursor",
    "k_d008_structure",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct D025ProductiveRates {
    pub k_activation: f64,
    pub k_rep: f64,
    pub k_precursor: f64,
    pub k_structure: f64,
}

impl D025ProductiveRates {
    pub fn from_legacy(rates: &StageEReferenceRates, k_precursor: f64) -> Self {
        Self {
            k_activation: rates.k_d008_activation,
            k_rep: rates.k_d008_reproduction,
            k_precursor,
            k_structure: rates.k_d008_structure,
        }
    }

    pub fn to_vector(self) -> [f64; 4] {
        [
            self.k_structure,
            self.k_rep,
            self.k_precursor,
            self.k_activation,
        ]
    }

    pub fn from_vector(v: [f64; 4]) -> Self {
        Self {
            k_structure: v[0],
            k_rep: v[1],
            k_precursor: v[2],
            k_activation: v[3],
        }
    }

    pub fn apply_to_params(&self, params: &mut crate::config::SimParams) {
        params.k_d008_activation = self.k_activation;
        params.k_d008_reproduction = self.k_rep;
        params.k_precursor = self.k_precursor;
        params.k_d008_structure = self.k_structure;
    }
}

pub fn clamp_productive_to_global(
    rates: &D025ProductiveRates,
    reference: &D025ProductiveRates,
) -> D025ProductiveRates {
    let ref_v = reference.to_vector();
    let cur = rates.to_vector();
    let mut out = [0.0; 4];
    for idx in 0..4 {
        let lo = ref_v[idx] * D025_GLOBAL_RATE_MIN_FACTOR;
        let hi = ref_v[idx] * D025_GLOBAL_RATE_MAX_FACTOR;
        out[idx] = cur[idx].clamp(lo, hi);
    }
    D025ProductiveRates::from_vector(out)
}

pub fn perturb_productive(
    base: &D025ProductiveRates,
    idx: usize,
    factor: f64,
) -> D025ProductiveRates {
    let mut v = base.to_vector();
    v[idx] *= factor;
    D025ProductiveRates::from_vector(v)
}

pub fn productive_rates_close(a: &D025ProductiveRates, b: &D025ProductiveRates) -> bool {
    a.to_vector()
        .iter()
        .zip(b.to_vector())
        .all(|(x, y)| (x - y).abs() <= 1e-12 * x.abs().max(1.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D025Conclusion {
    D025StageERecovered,
    D025StageENoJointFixedPoint,
    D025StageELongTransientUnresolved,
    D025NumericalFailure,
    D025AccountingFailure,
    D025Fail,
}

impl D025Conclusion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::D025StageERecovered => "D025_STAGE_E_RECOVERED",
            Self::D025StageENoJointFixedPoint => "D025_STAGE_E_NO_JOINT_FIXED_POINT",
            Self::D025StageELongTransientUnresolved => "D025_STAGE_E_LONG_TRANSIENT_UNRESOLVED",
            Self::D025NumericalFailure => "D025_NUMERICAL_FAILURE",
            Self::D025AccountingFailure => "D025_ACCOUNTING_FAILURE",
            Self::D025Fail => "D025_FAIL",
        }
    }
}

pub fn d025_center_flow_gate(metrics: &JointBalanceMetrics) -> bool {
    g_vector(metrics).iter().all(|g| g.abs() <= D011_G_TOL)
}

pub fn d025_center_q_gate(metrics: &JointBalanceMetrics) -> bool {
    q_vector(metrics)
        .iter()
        .all(|q| *q >= D011_Q_MIN && *q <= D011_Q_MAX)
}

pub fn d025_center_retention_gate(metrics: &JointBalanceMetrics) -> bool {
    metrics.catalyst_retention >= D011_RETENTION_MIN
        && metrics.activated_retention >= D011_RETENTION_MIN
        && metrics.membrane_localization >= D025_LOCALIZATION_MIN
}

pub fn d025_resource_flux_gate(metrics: &JointBalanceMetrics) -> bool {
    metrics.nutrient_influx > 0.0
        && metrics.fuel_influx > 0.0
        && metrics.waste_efflux > 0.0
}

pub fn d025_joint_balance_pass(metrics: &JointBalanceMetrics) -> bool {
    d025_center_q_gate(metrics)
        && d025_center_flow_gate(metrics)
        && d025_center_retention_gate(metrics)
        && d025_resource_flux_gate(metrics)
}

pub fn converged_three_windows(consecutive: u64) -> bool {
    consecutive >= D025_REQUIRED_WINDOWS
}

pub fn is_numerical_termination(reason: TerminationReason) -> bool {
    matches!(
        reason,
        TerminationReason::TimestepFloorFailure
            | TerminationReason::NumericalFailure
            | TerminationReason::UnboundedAccumulation
    )
}

pub fn is_biological_termination(reason: TerminationReason) -> bool {
    matches!(
        reason,
        TerminationReason::CatalystExtinction
            | TerminationReason::ActivatedExtinction
            | TerminationReason::MembraneExtinction
            | TerminationReason::ResourceExhaustion
    )
}

pub fn stage_e_recovered(
    converged: bool,
    metrics: &JointBalanceMetrics,
    accounting_closed: bool,
    material_closed: bool,
    contamination: f64,
    restoring: bool,
) -> bool {
    converged
        && d025_joint_balance_pass(metrics)
        && accounting_closed
        && material_closed
        && contamination <= D025_CONTAMINATION_MAX
        && restoring
}

pub fn select_stage_e_conclusion(
    numerical_failure: bool,
    accounting_failure: bool,
    converged: bool,
    joint_pass: bool,
    max_steps_reached: bool,
    any_solver_candidate_pass: bool,
    restoring: bool,
) -> D025Conclusion {
    if numerical_failure {
        return D025Conclusion::D025NumericalFailure;
    }
    if accounting_failure {
        return D025Conclusion::D025AccountingFailure;
    }
    if any_solver_candidate_pass && converged && joint_pass && restoring {
        return D025Conclusion::D025StageERecovered;
    }
    if converged && joint_pass && restoring {
        return D025Conclusion::D025StageERecovered;
    }
    if converged && !joint_pass {
        return D025Conclusion::D025StageENoJointFixedPoint;
    }
    if max_steps_reached && !converged {
        return D025Conclusion::D025StageELongTransientUnresolved;
    }
    if !joint_pass {
        return D025Conclusion::D025StageENoJointFixedPoint;
    }
    D025Conclusion::D025Fail
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

fn clamp_rate_global(proposed: f64, reference: f64) -> f64 {
    proposed.clamp(
        reference * D025_GLOBAL_RATE_MIN_FACTOR,
        reference * D025_GLOBAL_RATE_MAX_FACTOR,
    )
}

pub fn apply_log_deltas(
    analytical: &D025ProductiveRates,
    current: &D025ProductiveRates,
    dp: &[f64; 4],
) -> D025ProductiveRates {
    let ref_rates = analytical.to_vector();
    let cur_rates = current.to_vector();
    let mut next_rates = [0.0; 4];
    for idx in 0..4 {
        next_rates[idx] = clamp_rate_global(cur_rates[idx] * dp[idx].exp(), ref_rates[idx]);
    }
    D025ProductiveRates::from_vector(next_rates)
}

pub fn solve_bounded_joint_step_d025(
    analytical: &D025ProductiveRates,
    current: &D025ProductiveRates,
    g: [f64; 4],
    sensitivity: &SensitivityReport,
    round: usize,
) -> Option<JointSolverCandidate> {
    if round >= D025_MAX_SOLVER_ROUNDS {
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
    let round_min = D025_ROUND_RATE_MIN_FACTOR.ln();
    let round_max = D025_ROUND_RATE_MAX_FACTOR.ln();
    for value in &mut dp {
        *value = value.clamp(round_min, round_max);
    }
    let mut dp_arr = [0.0; 4];
    for (idx, value) in dp.iter().take(4).enumerate() {
        dp_arr[idx] = *value;
    }
    let productive = apply_log_deltas(analytical, current, &dp_arr);
    let mut log_change_norm = 0.0;
    for idx in 0..4 {
        let cur = current.to_vector()[idx];
        let nxt = productive.to_vector()[idx];
        let delta = (nxt / cur.max(f64::EPSILON)).ln();
        log_change_norm += delta * delta;
    }
    log_change_norm = log_change_norm.sqrt();
    let legacy = StageEReferenceRates {
        k_membrane: 0.0,
        k_d008_activation: productive.k_activation,
        k_d008_reproduction: productive.k_rep,
        k_d008_structure: productive.k_structure,
        k_d008_activated_decay: 0.005,
        k_d008_catalyst_turnover: 0.002,
        k_structure_decay: 0.025,
    };
    Some(JointSolverCandidate {
        round,
        rate_deltas_log: dp_arr,
        rates: legacy,
        log_change_norm,
    })
}

pub fn bounded_joint_solver_d025(
    analytical: &D025ProductiveRates,
    start: &D025ProductiveRates,
    g_history: &[[f64; 4]],
    sensitivity_history: &[SensitivityReport],
) -> JointSolverReport {
    let mut candidates = Vec::new();
    candidates.push(JointSolverCandidate {
        round: 0,
        rate_deltas_log: [0.0; 4],
        rates: StageEReferenceRates {
            k_membrane: 0.0,
            k_d008_activation: start.k_activation,
            k_d008_reproduction: start.k_rep,
            k_d008_structure: start.k_structure,
            k_d008_activated_decay: 0.005,
            k_d008_catalyst_turnover: 0.002,
            k_structure_decay: 0.025,
        },
        log_change_norm: 0.0,
    });
    let mut current = *start;
    for round in 0..D025_MAX_SOLVER_ROUNDS {
        if candidates.len() >= D025_MAX_CANDIDATES {
            break;
        }
        let g = g_history.get(round).copied().unwrap_or([0.0; 4]);
        let sensitivity = sensitivity_history
            .get(round)
            .cloned()
            .unwrap_or_else(|| sensitivity_matrix(&[[0.0; 4]; 4]));
        let Some(step) =
            solve_bounded_joint_step_d025(analytical, &current, g, &sensitivity, round)
        else {
            break;
        };
        let next = apply_log_deltas(analytical, &current, &step.rate_deltas_log);
        if productive_rates_close(&current, &next) {
            break;
        }
        current = next;
        candidates.push(step);
    }
    let bounded = candidates.len() <= D025_MAX_CANDIDATES;
    JointSolverReport {
        rounds_attempted: candidates.len().saturating_sub(1),
        candidates,
        bounded,
    }
}

pub fn sensitivity_from_perturbations(
    g_up_rows: &[[f64; 4]; 4],
    g_down_rows: &[[f64; 4]; 4],
) -> SensitivityReport {
    let mut rows = [[0.0; 4]; 4];
    for idx in 0..4 {
        for row in 0..4 {
            rows[row][idx] = log_central_difference_with_perturb(
                g_up_rows[idx][row],
                g_down_rows[idx][row],
                D025_SENSITIVITY_PERTURB,
            );
        }
    }
    sensitivity_matrix(&rows)
}

/// Placeholder metrics for tests; production runs use governed balance metrics.
pub fn placeholder_joint_metrics(g: [f64; 4], q: [f64; 4]) -> JointBalanceMetrics {
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
        catalyst_retention: 0.95,
        activated_retention: 0.95,
        membrane_localization: 0.96,
        nutrient_influx: 1.0,
        fuel_influx: 1.0,
        waste_efflux: 1.0,
    }
}

pub fn legacy_rates_unchanged_except_productive(
    base: &StageEReferenceRates,
    candidate: &StageEReferenceRates,
) -> bool {
    base.k_membrane == candidate.k_membrane
        && base.k_d008_activated_decay == candidate.k_d008_activated_decay
        && base.k_d008_catalyst_turnover == candidate.k_d008_catalyst_turnover
        && base.k_structure_decay == candidate.k_structure_decay
}

pub fn productive_layout_matches_d020_order() -> bool {
    StageEReferenceRates {
        k_membrane: 1.0,
        k_d008_activation: 4.0,
        k_d008_reproduction: 2.0,
        k_d008_structure: 3.0,
        k_d008_activated_decay: 0.0,
        k_d008_catalyst_turnover: 0.0,
        k_structure_decay: 0.0,
    };
    D025ProductiveRates {
        k_structure: 3.0,
        k_rep: 2.0,
        k_precursor: 1.0,
        k_activation: 4.0,
    }
    .to_vector()
        == [3.0, 2.0, 1.0, 4.0]
}

pub fn scientific_is_numerical(class: ScientificClassification) -> bool {
    matches!(
        class,
        ScientificClassification::NumericalFailure
            | ScientificClassification::UnboundedAccumulation
            | ScientificClassification::InvalidArtifact
    )
}
