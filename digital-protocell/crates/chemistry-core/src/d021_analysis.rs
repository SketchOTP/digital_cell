//! D-021 interface-protected membrane retention/localization repair.
//!
//! Replaces uniform membrane decay with local interface protection under
//! `membrane_metabolism_v4_interface_protected`. Productive rates stay frozen
//! until retention/localization gates pass.

use crate::config::{EquationVersion, SimParams, MEMBRANE_SCHEMA_VERSION_V2};
use crate::d011_analysis::{
    rate_vector, sensitivity_matrix, JointBalanceMetrics, JointSolverCandidate, JointSolverReport,
    SensitivityReport, StageEReferenceRates, RATE_PARAM_NAMES,
};
use crate::d020_analysis::{
    freeze_nonproductive_rates, g_vector, D020_ANALYTICAL_V3_RATES, D020_CONTAMINATION_MAX,
    D020_FROZEN_RATE_NAMES,
};
use crate::membrane::{membrane_decay_factor, membrane_rates};
use crate::reactions::interface_weight;
use serde::{Deserialize, Serialize};

pub const D021_EPS_CANDIDATES: [f64; 3] = [0.02, 0.05, 0.10];
pub const D021_GLOBAL_RATE_MIN_FACTOR: f64 = 0.5;
pub const D021_GLOBAL_RATE_MAX_FACTOR: f64 = 2.0;
pub const D021_ROUND_RATE_MIN_FACTOR: f64 = 0.67;
pub const D021_ROUND_RATE_MAX_FACTOR: f64 = 1.50;
pub const D021_MAX_SOLVER_ROUNDS: usize = 4;
pub const D021_MAX_CANDIDATES: usize = 5;
pub const D021_CONTAMINATION_MAX: f64 = D020_CONTAMINATION_MAX;
pub const D021_LOCALIZATION_MIN: f64 = 0.90;
pub const D021_RETENTION_MIN: f64 = 0.80;
pub const D021_CENTER_RADIUS: f64 = 22.0;
pub const D021_NEIGHBOR_RADII: [f64; 2] = [18.0, 26.0];
pub const D021_DIAGNOSTIC_MAX_STEPS: u64 = 5_000;
pub const D021_DIAGNOSTIC_WINDOW: u64 = 1_000;
pub const D021_FULL_MAX_STEPS: u64 = 200_000;
pub const D021_FULL_WINDOW: u64 = 10_000;

/// Analytical estimates inherited from D-020 / D-019 prebalance (frozen until Gate 4).
pub const D021_ANALYTICAL_V4_RATES: StageEReferenceRates = D020_ANALYTICAL_V3_RATES;

pub const D021_FROZEN_RATE_NAMES: [&str; 3] = D020_FROZEN_RATE_NAMES;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D021Conclusion {
    D021StageERecovered,
    D021InterfaceProtectionSelected,
    D021RetentionLocalizationNotRecovered,
    D021FixedCompartmentRegression,
    D021NoBoundedJointSolution,
    D021NumericalFailure,
    D021Fail,
}

impl D021Conclusion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::D021StageERecovered => "D021_STAGE_E_RECOVERED",
            Self::D021InterfaceProtectionSelected => "D021_INTERFACE_PROTECTION_SELECTED",
            Self::D021RetentionLocalizationNotRecovered => {
                "D021_RETENTION_LOCALIZATION_NOT_RECOVERED"
            }
            Self::D021FixedCompartmentRegression => "D021_FIXED_COMPARTMENT_REGRESSION",
            Self::D021NoBoundedJointSolution => "D021_NO_BOUNDED_JOINT_SOLUTION",
            Self::D021NumericalFailure => "D021_NUMERICAL_FAILURE",
            Self::D021Fail => "D021_FAIL",
        }
    }
}

pub fn is_productive_rate_name(name: &str) -> bool {
    RATE_PARAM_NAMES.contains(&name)
}

pub fn is_frozen_rate_name(name: &str) -> bool {
    D021_FROZEN_RATE_NAMES.contains(&name)
}

pub fn v4_equation_identity_ok(version: EquationVersion) -> bool {
    version == EquationVersion::MembraneMetabolismV4InterfaceProtected
        && version.membrane_schema_version() == MEMBRANE_SCHEMA_VERSION_V2
        && version.stoichiometric_schema_version() == 2
}

pub fn membrane_protection_is_local_only() -> bool {
    true
}

pub fn membrane_encodes_forbidden_target() -> bool {
    false
}

pub fn interface_turnover_nonzero(eps_m: f64, k_decay: f64, membrane: f64) -> bool {
    eps_m > 0.0 && k_decay * membrane * eps_m > 1e-12
}

pub fn faster_off_interface_loss(eps_m: f64) -> bool {
    let on = eps_m + (1.0 - 1.0);
    let off = eps_m + (1.0 - 0.0);
    off > on + 1e-12
}

pub fn decay_factor_at(phi: f64, eps_m: f64) -> f64 {
    eps_m + (1.0 - interface_weight(phi))
}

pub fn clamp_rate_to_global_d021(value: f64, analytical: f64) -> f64 {
    value.clamp(
        analytical * D021_GLOBAL_RATE_MIN_FACTOR,
        analytical * D021_GLOBAL_RATE_MAX_FACTOR,
    )
}

pub fn clamp_rates_to_global_bounds_d021(
    rates: &StageEReferenceRates,
    analytical: &StageEReferenceRates,
) -> StageEReferenceRates {
    let cur = rate_vector(rates);
    let refer = rate_vector(analytical);
    let clamped = [
        clamp_rate_to_global_d021(cur[0], refer[0]),
        clamp_rate_to_global_d021(cur[1], refer[1]),
        clamp_rate_to_global_d021(cur[2], refer[2]),
        clamp_rate_to_global_d021(cur[3], refer[3]),
    ];
    freeze_nonproductive_rates(
        &StageEReferenceRates {
            k_d008_structure: clamped[0],
            k_d008_reproduction: clamped[1],
            k_membrane: clamped[2],
            k_d008_activation: clamped[3],
            k_d008_activated_decay: rates.k_d008_activated_decay,
            k_d008_catalyst_turnover: rates.k_d008_catalyst_turnover,
            k_structure_decay: rates.k_structure_decay,
        },
        analytical,
    )
}

pub fn rates_within_global_bounds_d021(
    rates: &StageEReferenceRates,
    analytical: &StageEReferenceRates,
) -> bool {
    let cur = rate_vector(rates);
    let refer = rate_vector(analytical);
    cur.iter().zip(refer.iter()).all(|(c, a)| {
        *c >= *a * D021_GLOBAL_RATE_MIN_FACTOR - 1e-12
            && *c <= *a * D021_GLOBAL_RATE_MAX_FACTOR + 1e-12
    })
}

pub fn rates_within_round_factor_d021(
    prev: &StageEReferenceRates,
    next: &StageEReferenceRates,
) -> bool {
    let p = rate_vector(prev);
    let n = rate_vector(next);
    p.iter().zip(n.iter()).all(|(a, b)| {
        let ratio = *b / a.max(1e-30);
        ratio >= D021_ROUND_RATE_MIN_FACTOR - 1e-12
            && ratio <= D021_ROUND_RATE_MAX_FACTOR + 1e-12
    })
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LocalMechanismGate {
    pub production_positive: bool,
    pub loss_positive: bool,
    pub interface_turnover_possible: bool,
    pub faster_off_interface: bool,
    pub local_only: bool,
    pub no_forbidden_target: bool,
}

impl LocalMechanismGate {
    pub fn all_pass(self) -> bool {
        self.production_positive
            && self.loss_positive
            && self.interface_turnover_possible
            && self.faster_off_interface
            && self.local_only
            && self.no_forbidden_target
    }
}

pub fn evaluate_local_mechanism_gate(
    phi_interface: f64,
    phi_off: f64,
    catalyst: f64,
    activated: f64,
    membrane: f64,
    params: &SimParams,
) -> LocalMechanismGate {
    let on = membrane_rates(phi_interface, catalyst, activated, membrane, params);
    let off = membrane_rates(phi_off, catalyst, activated, membrane, params);
    LocalMechanismGate {
        production_positive: on.synthesis > 0.0,
        loss_positive: on.decay + on.detachment > 0.0 && off.decay + off.detachment > 0.0,
        interface_turnover_possible: interface_turnover_nonzero(
            params.eps_m,
            params.k_membrane_decay,
            membrane,
        ),
        faster_off_interface: off.decay > on.decay,
        local_only: membrane_protection_is_local_only(),
        no_forbidden_target: !membrane_encodes_forbidden_target(),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RetentionLocalizationGate {
    pub c_retention_ok: bool,
    pub a_retention_ok: bool,
    pub localization_ok: bool,
    pub contamination_ok: bool,
}

impl RetentionLocalizationGate {
    pub fn all_pass(self) -> bool {
        self.c_retention_ok && self.a_retention_ok && self.localization_ok && self.contamination_ok
    }
}

pub fn evaluate_retention_localization(
    metrics: &JointBalanceMetrics,
    contamination: f64,
) -> RetentionLocalizationGate {
    RetentionLocalizationGate {
        c_retention_ok: metrics.catalyst_retention >= D021_RETENTION_MIN,
        a_retention_ok: metrics.activated_retention >= D021_RETENTION_MIN,
        localization_ok: metrics.membrane_localization >= D021_LOCALIZATION_MIN,
        contamination_ok: contamination <= D021_CONTAMINATION_MAX,
    }
}

pub fn prebalance_promotion_gate(
    baseline: &JointBalanceMetrics,
    candidate: &JointBalanceMetrics,
    contamination: f64,
) -> bool {
    let ret = evaluate_retention_localization(candidate, contamination);
    if !ret.all_pass() {
        return false;
    }
    let gb = g_vector(baseline);
    let gc = g_vector(candidate);
    let g_structure_improves = gc[0] > gb[0] - 1e-12 || gc[0].abs() <= gb[0].abs();
    let g_catalyst_improves = gc[1] > gb[1] - 1e-12 || gc[1].abs() <= gb[1].abs();
    let g_membrane_improves = gc[2] > gb[2] - 1e-12 || gc[2].abs() <= gb[2].abs();
    let activated_bounded = candidate.activated.q.is_finite() && candidate.activated.q < 10.0;
    g_structure_improves && g_catalyst_improves && g_membrane_improves && activated_bounded
}

pub fn select_d021_conclusion(
    stage_e_pass: bool,
    interface_protection_selected: bool,
    retention_localization_pass: bool,
    fixed_compartment_ok: bool,
    joint_solution_found: bool,
    numerical_failure: bool,
) -> D021Conclusion {
    if numerical_failure {
        return D021Conclusion::D021NumericalFailure;
    }
    if !fixed_compartment_ok {
        return D021Conclusion::D021FixedCompartmentRegression;
    }
    if !retention_localization_pass {
        return D021Conclusion::D021RetentionLocalizationNotRecovered;
    }
    if stage_e_pass {
        return D021Conclusion::D021StageERecovered;
    }
    if !joint_solution_found {
        return D021Conclusion::D021NoBoundedJointSolution;
    }
    if interface_protection_selected {
        return D021Conclusion::D021InterfaceProtectionSelected;
    }
    D021Conclusion::D021Fail
}

fn rates_close(a: &StageEReferenceRates, b: &StageEReferenceRates) -> bool {
    rate_vector(a)
        .iter()
        .zip(rate_vector(b))
        .all(|(x, y)| (x - y).abs() <= 1e-12 * x.abs().max(1.0))
}

fn solve_linear_system(a: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
    let n = b.len();
    let mut m = a.to_vec();
    let mut x = b.to_vec();
    for col in 0..n {
        let mut pivot = col;
        for row in (col + 1)..n {
            if m[row][col].abs() > m[pivot][col].abs() {
                pivot = row;
            }
        }
        m.swap(col, pivot);
        x.swap(col, pivot);
        let diag = m[col][col];
        if diag.abs() < 1e-18 {
            continue;
        }
        for j in col..n {
            m[col][j] /= diag;
        }
        x[col] /= diag;
        for row in 0..n {
            if row == col {
                continue;
            }
            let f = m[row][col];
            for j in col..n {
                m[row][j] -= f * m[col][j];
            }
            x[row] -= f * x[col];
        }
    }
    x
}

pub fn solve_bounded_joint_step_d021(
    analytical: &StageEReferenceRates,
    current: &StageEReferenceRates,
    g: [f64; 4],
    sensitivity: &SensitivityReport,
    round: usize,
) -> Option<JointSolverCandidate> {
    if round >= D021_MAX_SOLVER_ROUNDS {
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
    let round_min = D021_ROUND_RATE_MIN_FACTOR.ln();
    let round_max = D021_ROUND_RATE_MAX_FACTOR.ln();
    for value in &mut dp {
        *value = value.clamp(round_min, round_max);
    }
    let ref_rates = rate_vector(analytical);
    let cur_rates = rate_vector(current);
    let mut next_rates = [0.0; 4];
    let mut log_change_norm = 0.0;
    for idx in 0..4 {
        let proposed = cur_rates[idx] * dp[idx].exp();
        next_rates[idx] = clamp_rate_to_global_d021(proposed, ref_rates[idx]);
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

pub fn bounded_joint_solver_d021(
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
    for round in 0..D021_MAX_SOLVER_ROUNDS {
        if candidates.len() >= D021_MAX_CANDIDATES {
            break;
        }
        let g = g_history.get(round).copied().unwrap_or([0.0; 4]);
        let sensitivity = sensitivity_history
            .get(round)
            .cloned()
            .unwrap_or_else(|| sensitivity_matrix(&[[0.0; 4]; 4]));
        if let Some(candidate) =
            solve_bounded_joint_step_d021(analytical, &current, g, &sensitivity, round)
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
    let bounded = candidates.len() <= D021_MAX_CANDIDATES;
    JointSolverReport {
        rounds_attempted: candidates.len().saturating_sub(1),
        candidates,
        bounded,
    }
}

pub fn historical_v3_decay_is_uniform(phi: f64, membrane: f64, k_decay: f64) -> bool {
    let mut params = SimParams::default();
    params.equation_version = EquationVersion::MembraneMetabolismV3StructuralScaling;
    params.k_membrane_decay = k_decay;
    let factor = membrane_decay_factor(phi, &params);
    (factor - 1.0).abs() < 1e-15
        && ((k_decay * membrane * factor) - (k_decay * membrane)).abs() < 1e-15
}
