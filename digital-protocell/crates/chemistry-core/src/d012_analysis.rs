//! D-012 v2 transport-coupled Stage E analysis (constrained-radius balance).

use crate::d011_analysis::{
    joint_overlap_pass, log_central_difference, rate_vector, sensitivity_matrix,
    ComponentBalance, ConvergenceClassification, JointBalanceMetrics, JointSolverCandidate,
    JointSolverReport, QuasiSteadyReport, SensitivityReport, StageEReferenceRates, D011_G_TOL,
    D011_LOCALIZATION_MIN, D011_MAX_CANDIDATES, D011_MAX_SOLVER_ROUNDS, D011_Q_MAX, D011_Q_MIN,
    D011_RETENTION_MIN, D011_SENSITIVITY_PERTURB,
};
use crate::d012_accounting::{material_step_closes, MaterialEquivalentStep};
use crate::stage_d_gate::ordered_restoring_crossing;
use serde::{Deserialize, Serialize};

pub const D012_V2_STAGE_E_RADII: [f64; 6] = [14.0, 18.0, 22.0, 26.0, 30.0, 34.0];
pub const D012_V2_CENTER_RADIUS: f64 = 22.0;
pub const D012_V2_NEIGHBOR_RADII: [f64; 2] = [18.0, 26.0];
pub const D012_CALIBRATION_FACTORS: [f64; 3] = [0.75, 1.0, 1.25];
pub const D012_V2_MAX_STEPS: u64 = 200_000;
pub const D012_V2_WINDOW: u64 = 10_000;
pub const D012_V2_REQUIRED_WINDOWS: u64 = 3;
pub const D012_DIAGNOSTIC_MAX_STEPS: u64 = 5_000;
pub const D012_DIAGNOSTIC_WINDOW: u64 = 1_000;

pub const D012_GLOBAL_RATE_MIN_FACTOR: f64 = 0.25;
pub const D012_GLOBAL_RATE_MAX_FACTOR: f64 = 4.0;
pub const D012_ROUND_RATE_MIN_FACTOR: f64 = 0.67;
pub const D012_ROUND_RATE_MAX_FACTOR: f64 = 1.50;

pub const D012_YIELD_CANDIDATES: [f64; 3] = [1.0, 17.0 / 20.0, 7.0 / 10.0];
pub const D012_MAX_YIELD_CANDIDATES: usize = 3;

pub const D012_RATE_PERTURB: f64 = 0.02;
pub const D012_INITIAL_CAM_PERTURB: f64 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum D012V2CalibrationParameter {
    Activation,
    Reproduction,
    MembraneProduction,
    StructureProduction,
}

impl D012V2CalibrationParameter {
    pub const ORDER: [Self; 4] = [
        Self::Activation,
        Self::Reproduction,
        Self::MembraneProduction,
        Self::StructureProduction,
    ];

    pub fn apply_factor(&self, rates: &mut StageEReferenceRates, factor: f64) {
        match self {
            Self::Activation => rates.k_d008_activation *= factor,
            Self::Reproduction => rates.k_d008_reproduction *= factor,
            Self::MembraneProduction => rates.k_membrane *= factor,
            Self::StructureProduction => rates.k_d008_structure *= factor,
        }
    }

    pub fn baseline_value(&self, rates: &StageEReferenceRates) -> f64 {
        match self {
            Self::Activation => rates.k_d008_activation,
            Self::Reproduction => rates.k_d008_reproduction,
            Self::MembraneProduction => rates.k_membrane,
            Self::StructureProduction => rates.k_d008_structure,
        }
    }

    pub fn set_value(&self, rates: &mut StageEReferenceRates, value: f64) {
        match self {
            Self::Activation => rates.k_d008_activation = value,
            Self::Reproduction => rates.k_d008_reproduction = value,
            Self::MembraneProduction => rates.k_membrane = value,
            Self::StructureProduction => rates.k_d008_structure = value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D012StageEClassification {
    JointBalancePass,
    NotConverged,
    LongTransientUnresolved,
    NoRestoringRadius,
    ResourceThroughputFail,
    AccountingFailure,
    NumericalFailure,
    NoJointOverlap,
    SolverDomainExhausted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct D012RadiusBalancePoint {
    pub radius: f64,
    pub g_structure: f64,
    pub joint_overlap: bool,
    pub quasi_steady: bool,
}

pub fn balance_calibration_score(metrics: &JointBalanceMetrics) -> f64 {
    let q_penalty = [metrics.structure.q, metrics.catalyst.q, metrics.membrane.q, metrics.activated.q]
        .iter()
        .map(|q| (q - 1.0).abs())
        .sum::<f64>();
    let g_penalty = [
        metrics.structure.g,
        metrics.catalyst.g,
        metrics.membrane.g,
        metrics.activated.g,
    ]
    .iter()
    .map(|g| g.abs())
    .sum::<f64>();
    q_penalty + g_penalty
}

pub fn estimate_rates_from_metrics(
    base: &StageEReferenceRates,
    metrics: &JointBalanceMetrics,
) -> StageEReferenceRates {
    let mut rates = *base;
    rates.k_d008_structure /= metrics.structure.q.max(1e-6);
    rates.k_d008_reproduction /= metrics.catalyst.q.max(1e-6);
    rates.k_membrane /= metrics.membrane.q.max(1e-6);
    rates.k_d008_activation /= metrics.activated.q.max(1e-6);
    rates
}

pub fn resource_throughput_pass(metrics: &JointBalanceMetrics) -> bool {
    metrics.nutrient_influx > 0.0 && metrics.fuel_influx > 0.0 && metrics.waste_efflux > 0.0
}

pub fn retention_pass(metrics: &JointBalanceMetrics) -> bool {
    metrics.catalyst_retention >= D011_RETENTION_MIN
        && metrics.activated_retention >= D011_RETENTION_MIN
        && metrics.membrane_localization >= D011_LOCALIZATION_MIN
}

pub fn all_four_balances_pass(metrics: &JointBalanceMetrics) -> bool {
    let qs = [
        metrics.structure.q,
        metrics.catalyst.q,
        metrics.membrane.q,
        metrics.activated.q,
    ];
    let gs = [
        metrics.structure.g,
        metrics.catalyst.g,
        metrics.membrane.g,
        metrics.activated.g,
    ];
    qs.iter().all(|q| *q >= D011_Q_MIN && *q <= D011_Q_MAX)
        && gs.iter().all(|g| g.abs() <= D011_G_TOL)
}

pub fn restoring_radius_from_g_structure(points: &[D012RadiusBalancePoint]) -> bool {
    let mut pairs: Vec<(f64, f64)> = points
        .iter()
        .map(|p| (p.radius, p.g_structure))
        .collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    ordered_restoring_crossing(&pairs)
}

pub fn v2_stage_e_pass(
    quasi_steady: &QuasiSteadyReport,
    metrics: &JointBalanceMetrics,
    material: &MaterialEquivalentStep,
    restoring_points: &[D012RadiusBalancePoint],
) -> bool {
    quasi_steady.converged
        && joint_overlap_pass(metrics)
        && resource_throughput_pass(metrics)
        && retention_pass(metrics)
        && material_step_closes(material)
        && restoring_radius_from_g_structure(restoring_points)
}

pub fn classify_v2_stage_e(
    quasi_steady: &QuasiSteadyReport,
    metrics: &JointBalanceMetrics,
    material_closed: bool,
    restoring_points: &[D012RadiusBalancePoint],
    convergence: ConvergenceClassification,
    max_steps_reached: bool,
) -> D012StageEClassification {
    if convergence == ConvergenceClassification::NumericalFailure {
        return D012StageEClassification::NumericalFailure;
    }
    if !material_closed {
        return D012StageEClassification::AccountingFailure;
    }
    if !resource_throughput_pass(metrics) {
        return D012StageEClassification::ResourceThroughputFail;
    }
    if quasi_steady.converged
        && joint_overlap_pass(metrics)
        && restoring_radius_from_g_structure(restoring_points)
    {
        return D012StageEClassification::JointBalancePass;
    }
    if quasi_steady.converged && joint_overlap_pass(metrics) {
        return D012StageEClassification::NoRestoringRadius;
    }
    if !quasi_steady.converged && max_steps_reached {
        return D012StageEClassification::LongTransientUnresolved;
    }
    if quasi_steady.converged {
        return D012StageEClassification::NoJointOverlap;
    }
    D012StageEClassification::NotConverged
}

pub fn select_calibration_factor(scores: [f64; 3]) -> usize {
    scores
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx)
        .unwrap_or(1)
}

pub fn perturb_v2_rates(base: &StageEReferenceRates, idx: usize, factor: f64) -> StageEReferenceRates {
    let mut rates = *base;
    let mut values = rate_vector(&rates);
    values[idx] *= factor;
    rates.k_d008_structure = values[0];
    rates.k_d008_reproduction = values[1];
    rates.k_membrane = values[2];
    rates.k_d008_activation = values[3];
    rates
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

pub fn solve_bounded_joint_step_v2(
    reference: &StageEReferenceRates,
    current: &StageEReferenceRates,
    g: [f64; 4],
    sensitivity: &SensitivityReport,
    round: usize,
) -> Option<JointSolverCandidate> {
    if round >= D011_MAX_SOLVER_ROUNDS {
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
    let round_min = D012_ROUND_RATE_MIN_FACTOR.ln();
    let round_max = D012_ROUND_RATE_MAX_FACTOR.ln();
    for value in &mut dp {
        *value = value.clamp(round_min, round_max);
    }
    let ref_rates = rate_vector(reference);
    let cur_rates = rate_vector(current);
    let mut next_rates = [0.0; 4];
    let mut log_change_norm = 0.0;
    for idx in 0..4 {
        let proposed = cur_rates[idx] * dp[idx].exp();
        let global_min = ref_rates[idx] * D012_GLOBAL_RATE_MIN_FACTOR;
        let global_max = ref_rates[idx] * D012_GLOBAL_RATE_MAX_FACTOR;
        next_rates[idx] = proposed.clamp(global_min, global_max);
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
        rates: StageEReferenceRates {
            k_d008_structure: next_rates[0],
            k_d008_reproduction: next_rates[1],
            k_membrane: next_rates[2],
            k_d008_activation: next_rates[3],
            k_d008_activated_decay: current.k_d008_activated_decay,
            k_d008_catalyst_turnover: current.k_d008_catalyst_turnover,
            k_structure_decay: current.k_structure_decay,
        },
        log_change_norm,
    })
}

fn rates_close_v2(a: &StageEReferenceRates, b: &StageEReferenceRates) -> bool {
    rate_vector(a)
        .iter()
        .zip(rate_vector(b))
        .all(|(x, y)| (x - y).abs() <= 1e-12 * x.abs().max(1.0))
}

pub fn bounded_joint_solver_v2(
    reference: &StageEReferenceRates,
    start: &StageEReferenceRates,
    g_history: &[[f64; 4]],
    sensitivity_history: &[SensitivityReport],
) -> JointSolverReport {
    let mut candidates = Vec::new();
    candidates.push(JointSolverCandidate {
        round: 0,
        rate_deltas_log: [0.0; 4],
        rates: *start,
        log_change_norm: 0.0,
    });
    let mut current = *start;
    for round in 0..D011_MAX_SOLVER_ROUNDS {
        if candidates.len() >= D011_MAX_CANDIDATES {
            break;
        }
        let g = g_history.get(round).copied().unwrap_or([0.0; 4]);
        let sensitivity = sensitivity_history.get(round).cloned().unwrap_or_else(|| {
            sensitivity_matrix(&[[0.0; 4]; 4])
        });
        if let Some(candidate) =
            solve_bounded_joint_step_v2(reference, &current, g, &sensitivity, round)
        {
            if candidates
                .iter()
                .any(|c| rates_close_v2(&c.rates, &candidate.rates))
            {
                break;
            }
            current = candidate.rates;
            candidates.push(candidate);
        } else {
            break;
        }
    }
    let rounds_attempted = candidates.len().saturating_sub(1);
    let bounded = candidates.len() <= D011_MAX_CANDIDATES;
    JointSolverReport {
        rounds_attempted,
        candidates,
        bounded,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YieldComponent {
    Catalyst,
    Structure,
    Membrane,
}

pub fn apply_yield_change(
    params: &mut crate::config::SimParams,
    component: YieldComponent,
    eta: f64,
) -> Result<(), String> {
    crate::config::validate_v2_yields(
        if component == YieldComponent::Catalyst {
            eta
        } else {
            params.eta_c
        },
        if component == YieldComponent::Structure {
            eta
        } else {
            params.eta_phi
        },
        if component == YieldComponent::Membrane {
            eta
        } else {
            params.eta_m
        },
    )?;
    match component {
        YieldComponent::Catalyst => params.eta_c = eta,
        YieldComponent::Structure => params.eta_phi = eta,
        YieldComponent::Membrane => params.eta_m = eta,
    }
    Ok(())
}

/// Yield branch may only reduce yield for persistently overproduced components (Q > Q_max).
pub fn yield_adjustment_allowed(component: ComponentBalance, current_eta: f64, proposed_eta: f64) -> bool {
    if component.q <= D011_Q_MAX {
        return false;
    }
    proposed_eta <= current_eta && proposed_eta < 1.0
}

pub fn count_yield_changes(before: (f64, f64, f64), after: (f64, f64, f64)) -> usize {
    let deltas = [
        (before.0 - after.0).abs(),
        (before.1 - after.1).abs(),
        (before.2 - after.2).abs(),
    ];
    deltas.iter().filter(|d| **d > 1e-12).count()
}

pub fn expansion_radii_after_center(center_pass: bool) -> Vec<f64> {
    if center_pass {
        D012_V2_STAGE_E_RADII.to_vec()
    } else {
        D012_V2_NEIGHBOR_RADII.to_vec()
    }
}
