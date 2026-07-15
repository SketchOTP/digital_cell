//! D-011 transport-coupled constrained-radius balance analysis.

use crate::activated_metabolism::ActivatedMetabolismCumulativeAccounting;
use crate::config::SimParams;
use crate::constraint_accounting::StructureConstraintCumulative;
use crate::membrane_accounting::{MembraneCumulativeAccounting, TransportStepAccounting};
use serde::{Deserialize, Serialize};

pub const D011_SLOPE_TOL: f64 = 1e-4;
pub const D011_TOTAL_REL_TOL: f64 = 0.05;
pub const D011_Q_MIN: f64 = 0.98;
pub const D011_Q_MAX: f64 = 1.02;
pub const D011_G_TOL: f64 = 1e-4;
pub const D011_RETENTION_MIN: f64 = 0.80;
pub const D011_LOCALIZATION_MIN: f64 = 0.90;
pub const D011_SENSITIVITY_PERTURB: f64 = 0.05;
pub const D011_GLOBAL_RATE_MIN_FACTOR: f64 = 0.5;
pub const D011_GLOBAL_RATE_MAX_FACTOR: f64 = 2.0;
pub const D011_ROUND_RATE_MIN_FACTOR: f64 = 0.75;
pub const D011_ROUND_RATE_MAX_FACTOR: f64 = 1.33;
pub const D011_MAX_SOLVER_ROUNDS: usize = 4;
pub const D011_MAX_CANDIDATES: usize = 5;
pub const D011_DEFAULT_WINDOW: u64 = 10_000;
pub const D011_TEST_WINDOW: u64 = 1_000;

pub const STAGE_E_FAILED_RATES: StageEReferenceRates = StageEReferenceRates {
    k_membrane: 0.23697878259991778,
    k_d008_activation: 0.024,
    k_d008_reproduction: 0.032,
    k_d008_structure: 0.6788558775098147,
    k_d008_activated_decay: 0.005,
    k_d008_catalyst_turnover: 0.002,
    k_structure_decay: 0.025,
};

pub const D011_REPLAY_RADII: [f64; 6] = [14.0, 18.0, 22.0, 26.0, 30.0, 34.0];
pub const D011_HORIZON_RADII: [f64; 3] = [18.0, 24.0, 30.0];
pub const D011_HORIZONS: [u64; 4] = [20_000, 50_000, 100_000, 200_000];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct StageEReferenceRates {
    pub k_membrane: f64,
    pub k_d008_activation: f64,
    pub k_d008_reproduction: f64,
    pub k_d008_structure: f64,
    pub k_d008_activated_decay: f64,
    pub k_d008_catalyst_turnover: f64,
    pub k_structure_decay: f64,
}

impl StageEReferenceRates {
    pub fn apply_to(&self, params: &mut SimParams) {
        params.k_membrane = self.k_membrane;
        params.k_d008_activation = self.k_d008_activation;
        params.k_d008_reproduction = self.k_d008_reproduction;
        params.k_d008_structure = self.k_d008_structure;
        params.k_d008_activated_decay = self.k_d008_activated_decay;
        params.k_d008_catalyst_turnover = self.k_d008_catalyst_turnover;
        params.k_structure_decay = self.k_structure_decay;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConvergenceClassification {
    JointOverlapPass,
    NotConverged,
    ResourceExhaustion,
    CatalystExtinction,
    ActivatedExtinction,
    MembraneExtinction,
    UnboundedAccumulation,
    OscillatoryUnresolved,
    NumericalFailure,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ComponentBalance {
    pub q: f64,
    pub g: f64,
    pub production: f64,
    pub loss: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct JointBalanceMetrics {
    pub structure: ComponentBalance,
    pub catalyst: ComponentBalance,
    pub membrane: ComponentBalance,
    pub activated: ComponentBalance,
    pub catalyst_retention: f64,
    pub activated_retention: f64,
    pub membrane_localization: f64,
    pub nutrient_influx: f64,
    pub fuel_influx: f64,
    pub waste_efflux: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteadyWindowSnapshot {
    pub start_step: u64,
    pub end_step: u64,
    pub simulated_time_start: f64,
    pub simulated_time_end: f64,
    pub mass_c: f64,
    pub mass_a: f64,
    pub mass_m: f64,
    pub mean_n_interior: f64,
    pub mean_f_interior: f64,
    pub mean_w_interior: f64,
    pub structure_production: f64,
    pub structure_decay: f64,
    pub catalyst_reproduction: f64,
    pub catalyst_turnover: f64,
    pub membrane_synthesis: f64,
    pub membrane_loss: f64,
    pub activation: f64,
    pub activated_loss: f64,
    pub nutrient_transport_interior: f64,
    pub fuel_transport_interior: f64,
    pub waste_transport_interior: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuasiSteadyReport {
    pub window_size: u64,
    pub converged_windows: u64,
    pub required_windows: u64,
    pub converged: bool,
    pub window_slopes: Vec<WindowSlopes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSlopes {
    pub slope_c: f64,
    pub slope_a: f64,
    pub slope_m: f64,
    pub slope_n: f64,
    pub slope_f: f64,
    pub slope_w: f64,
    pub totals_within_tolerance: bool,
}

pub const RATE_PARAM_NAMES: [&str; 4] = [
    "k_d008_structure",
    "k_d008_reproduction",
    "k_membrane",
    "k_d008_activation",
];

pub const G_COMPONENT_NAMES: [&str; 4] = [
    "g_structure",
    "g_catalyst",
    "g_membrane",
    "g_activated",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityReport {
    pub matrix: Vec<Vec<f64>>,
    pub singular_values: Vec<f64>,
    pub rank: usize,
    pub condition_number: f64,
    pub rank_deficient: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointSolverCandidate {
    pub round: usize,
    pub rate_deltas_log: [f64; 4],
    pub rates: StageEReferenceRates,
    pub log_change_norm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointSolverReport {
    pub rounds_attempted: usize,
    pub candidates: Vec<JointSolverCandidate>,
    pub bounded: bool,
}

pub fn rate_vector(rates: &StageEReferenceRates) -> [f64; 4] {
    [
        rates.k_d008_structure,
        rates.k_d008_reproduction,
        rates.k_membrane,
        rates.k_d008_activation,
    ]
}

pub fn window_time(snapshot: &SteadyWindowSnapshot) -> f64 {
    (snapshot.simulated_time_end - snapshot.simulated_time_start).max(f64::EPSILON)
}

pub fn window_slope(end: f64, start: f64, dt: f64) -> f64 {
    (end - start) / dt
}

pub fn totals_within_tolerance(prev: &SteadyWindowSnapshot, curr: &SteadyWindowSnapshot) -> bool {
    let pairs = [
        (
            curr.structure_production - prev.structure_production,
            prev.structure_production,
        ),
        (
            curr.catalyst_reproduction - prev.catalyst_reproduction,
            prev.catalyst_reproduction,
        ),
        (
            curr.membrane_synthesis - prev.membrane_synthesis,
            prev.membrane_synthesis,
        ),
        (curr.activation - prev.activation, prev.activation),
        (
            curr.nutrient_transport_interior - prev.nutrient_transport_interior,
            prev.nutrient_transport_interior,
        ),
        (
            curr.fuel_transport_interior - prev.fuel_transport_interior,
            prev.fuel_transport_interior,
        ),
        (
            curr.waste_transport_interior - prev.waste_transport_interior,
            prev.waste_transport_interior,
        ),
    ];
    pairs.iter().all(|(delta, base)| {
        let denom = base.abs().max(1e-12);
        delta.abs() / denom <= D011_TOTAL_REL_TOL
    })
}

pub fn window_slopes_converged(slopes: &WindowSlopes) -> bool {
    [
        slopes.slope_c,
        slopes.slope_a,
        slopes.slope_m,
        slopes.slope_n,
        slopes.slope_f,
        slopes.slope_w,
    ]
    .iter()
    .all(|s| s.abs() <= D011_SLOPE_TOL)
        && slopes.totals_within_tolerance
}

pub fn quasi_steady_report(
    windows: &[SteadyWindowSnapshot],
    window_size: u64,
    required_windows: u64,
) -> QuasiSteadyReport {
    let mut slopes = Vec::new();
    let mut converged_windows = 0u64;
    if windows.len() >= 2 {
        for pair in windows.windows(2) {
            let dt = window_time(&pair[1]);
            let slope = WindowSlopes {
                slope_c: window_slope(pair[1].mass_c, pair[0].mass_c, dt),
                slope_a: window_slope(pair[1].mass_a, pair[0].mass_a, dt),
                slope_m: window_slope(pair[1].mass_m, pair[0].mass_m, dt),
                slope_n: window_slope(pair[1].mean_n_interior, pair[0].mean_n_interior, dt),
                slope_f: window_slope(pair[1].mean_f_interior, pair[0].mean_f_interior, dt),
                slope_w: window_slope(pair[1].mean_w_interior, pair[0].mean_w_interior, dt),
                totals_within_tolerance: totals_within_tolerance(&pair[0], &pair[1]),
            };
            if window_slopes_converged(&slope) {
                converged_windows += 1;
            } else {
                converged_windows = 0;
            }
            slopes.push(slope);
        }
    }
    let converged = converged_windows >= required_windows.saturating_sub(1);
    QuasiSteadyReport {
        window_size,
        converged_windows,
        required_windows,
        converged,
        window_slopes: slopes,
    }
}

pub fn q_ratio(production: f64, loss: f64) -> f64 {
    production / loss.max(f64::EPSILON)
}

pub fn build_balance_metrics(
    sim_time: f64,
    constraint: &StructureConstraintCumulative,
    metabolism: &ActivatedMetabolismCumulativeAccounting,
    membrane: &MembraneCumulativeAccounting,
    transport: &TransportStepAccounting,
    catalyst_retention: f64,
    activated_retention: f64,
    membrane_localization: f64,
) -> JointBalanceMetrics {
    let dt = sim_time.max(f64::EPSILON);
    let structure_production = constraint.virtual_production / dt;
    let structure_decay = constraint.virtual_decay / dt;
    let catalyst_production = metabolism.reproduction / dt;
    let catalyst_loss = metabolism.catalyst_turnover / dt;
    let membrane_production = membrane.synthesis / dt;
    let membrane_loss = (membrane.decay + membrane.detachment) / dt;
    let activation = metabolism.activation / dt;
    let activated_loss =
        (metabolism.activated_decay + constraint.virtual_production) / dt;
    JointBalanceMetrics {
        structure: ComponentBalance {
            q: q_ratio(structure_production, structure_decay),
            g: constraint.virtual_structure_flow / dt,
            production: structure_production,
            loss: structure_decay,
        },
        catalyst: ComponentBalance {
            q: q_ratio(catalyst_production, catalyst_loss),
            g: metabolism.catalyst_reaction_delta / dt,
            production: catalyst_production,
            loss: catalyst_loss,
        },
        membrane: ComponentBalance {
            q: q_ratio(membrane_production, membrane_loss),
            g: (membrane.synthesis - membrane.decay - membrane.detachment) / dt,
            production: membrane_production,
            loss: membrane_loss,
        },
        activated: ComponentBalance {
            q: q_ratio(activation, activated_loss),
            g: metabolism.activated_reaction_delta / dt,
            production: activation,
            loss: activated_loss,
        },
        catalyst_retention,
        activated_retention,
        membrane_localization,
        nutrient_influx: transport.nutrient.interior_net_flux_rate,
        fuel_influx: transport.fuel.interior_net_flux_rate,
        waste_efflux: -transport.waste.interior_net_flux_rate,
    }
}

pub fn joint_overlap_pass(metrics: &JointBalanceMetrics) -> bool {
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
        && metrics.catalyst_retention >= D011_RETENTION_MIN
        && metrics.activated_retention >= D011_RETENTION_MIN
        && metrics.membrane_localization >= D011_LOCALIZATION_MIN
        && metrics.nutrient_influx > 0.0
        && metrics.fuel_influx > 0.0
        && metrics.waste_efflux > 0.0
}

pub fn classify_convergence(
    quasi_steady: &QuasiSteadyReport,
    metrics: &JointBalanceMetrics,
    catalyst_mass: f64,
    activated_mass: f64,
    membrane_mass: f64,
    nutrient_interior: f64,
    fuel_interior: f64,
    max_concentration: f64,
    accounting_closed: bool,
    rejection_ratio: f64,
) -> ConvergenceClassification {
    if !accounting_closed || rejection_ratio > 0.01 {
        return ConvergenceClassification::NumericalFailure;
    }
    if max_concentration > crate::config::CONC_SAFETY_LIMIT {
        return ConvergenceClassification::UnboundedAccumulation;
    }
    if catalyst_mass <= 1e-6 {
        return ConvergenceClassification::CatalystExtinction;
    }
    if activated_mass <= 1e-6 {
        return ConvergenceClassification::ActivatedExtinction;
    }
    if membrane_mass <= 1e-6 {
        return ConvergenceClassification::MembraneExtinction;
    }
    if nutrient_interior <= 1e-6 || fuel_interior <= 1e-6 {
        return ConvergenceClassification::ResourceExhaustion;
    }
    if quasi_steady.converged && joint_overlap_pass(metrics) {
        return ConvergenceClassification::JointOverlapPass;
    }
    if !quasi_steady.converged
        && quasi_steady.window_slopes.len() >= 6
        && quasi_steady.converged_windows == 0
    {
        let oscillatory = quasi_steady.window_slopes.windows(2).any(|pair| {
            pair[0].slope_c.signum() != pair[1].slope_c.signum()
                && pair[0].slope_a.signum() != pair[1].slope_a.signum()
        });
        if oscillatory {
            return ConvergenceClassification::OscillatoryUnresolved;
        }
    }
    ConvergenceClassification::NotConverged
}

pub fn log_central_difference(g_up: f64, g_down: f64) -> f64 {
    let ln_up = (1.0 + D011_SENSITIVITY_PERTURB).ln();
    let ln_down = (1.0 - D011_SENSITIVITY_PERTURB).ln();
    (g_up - g_down) / (ln_up - ln_down)
}

pub fn sensitivity_matrix<const N: usize>(rows: &[[f64; N]; 4]) -> SensitivityReport {
    let matrix: Vec<Vec<f64>> = rows.iter().map(|row| row.to_vec()).collect();
    let (singular_values, rank, condition_number) = analyze_matrix(&matrix);
    SensitivityReport {
        matrix,
        singular_values,
        rank,
        condition_number,
        rank_deficient: rank < 4,
    }
}

fn analyze_matrix(matrix: &[Vec<f64>]) -> (Vec<f64>, usize, f64) {
    let rows = matrix.len();
    let cols = matrix.first().map(|r| r.len()).unwrap_or(0);
    if rows == 0 || cols == 0 {
        return (Vec::new(), 0, f64::INFINITY);
    }
    let mut ata = vec![vec![0.0; cols]; cols];
    for i in 0..cols {
        for j in 0..cols {
            let mut sum = 0.0;
            for row in matrix {
                sum += row[i] * row[j];
            }
            ata[i][j] = sum;
        }
    }
    let singular_values = symmetric_eigenvalues(&ata);
    let rank = matrix_rank(matrix, 1e-8);
    let cond = if singular_values.last().copied().unwrap_or(0.0) > 1e-15 {
        singular_values.first().copied().unwrap_or(0.0)
            / singular_values.last().copied().unwrap_or(1.0)
    } else {
        f64::INFINITY
    };
    (singular_values, rank, cond)
}

fn matrix_rank(matrix: &[Vec<f64>], tol: f64) -> usize {
    let rows = matrix.len();
    let cols = matrix.first().map(|r| r.len()).unwrap_or(0);
    let mut a = matrix.to_vec();
    let mut rank = 0usize;
    let mut row = 0usize;
    for col in 0..cols {
        let mut pivot = row;
        let mut max_val = 0.0;
        for r in row..rows {
            let val = a[r][col].abs();
            if val > max_val {
                max_val = val;
                pivot = r;
            }
        }
        if max_val <= tol {
            continue;
        }
        a.swap(row, pivot);
        let pivot_val = a[row][col];
        for c in col..cols {
            a[row][c] /= pivot_val;
        }
        for r in 0..rows {
            if r == row {
                continue;
            }
            let factor = a[r][col];
            if factor.abs() <= tol {
                continue;
            }
            for c in col..cols {
                a[r][c] -= factor * a[row][c];
            }
        }
        rank += 1;
        row += 1;
        if row >= rows {
            break;
        }
    }
    rank
}

fn symmetric_eigenvalues(matrix: &[Vec<f64>]) -> Vec<f64> {
    let n = matrix.len();
    if n == 0 {
        return Vec::new();
    }
    let mut a = matrix.to_vec();
    for _ in 0..64 {
        let mut off = 0.0;
        for i in 0..n {
            for j in (i + 1)..n {
                off += a[i][j].abs();
            }
        }
        if off < 1e-12 {
            break;
        }
        let t = (off / (n as f64 * n as f64)).max(1e-12);
        for i in 0..(n - 1) {
            for j in (i + 1)..n {
                if a[i][j].abs() <= t {
                    continue;
                }
                let theta = 0.5 * (a[j][j] - a[i][i]) / a[i][j];
                let t_sign = theta.signum();
                let t_abs = theta.abs();
                let tan = t_sign / (t_abs + (1.0 + theta * theta).sqrt());
                let cos = 1.0 / (1.0 + tan * tan).sqrt();
                let sin = tan * cos;
                let aii = a[i][i];
                let ajj = a[j][j];
                let aij = a[i][j];
                a[i][i] = cos * cos * aii - 2.0 * sin * cos * aij + sin * sin * ajj;
                a[j][j] = sin * sin * aii + 2.0 * sin * cos * aij + cos * cos * ajj;
                a[i][j] = 0.0;
                a[j][i] = 0.0;
                for k in 0..n {
                    if k == i || k == j {
                        continue;
                    }
                    let aik = a[i][k];
                    let ajk = a[j][k];
                    a[i][k] = cos * aik - sin * ajk;
                    a[j][k] = sin * aik + cos * ajk;
                    a[k][i] = a[i][k];
                    a[k][j] = a[j][k];
                }
            }
        }
    }
    let mut values: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
    values.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    values.iter().map(|v| v.max(0.0).sqrt()).collect()
}

pub fn solve_bounded_joint_step(
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
        for (r, row) in sensitivity.matrix.iter().enumerate() {
            sum += row[i] * g[r];
        }
        atg[i] = sum;
    }
    let mut dp = solve_linear_system(&ata, &atg.iter().map(|v| -*v).collect::<Vec<_>>());
    let round_min = D011_ROUND_RATE_MIN_FACTOR.ln();
    let round_max = D011_ROUND_RATE_MAX_FACTOR.ln();
    for value in &mut dp {
        *value = value.clamp(round_min, round_max);
    }
    let ref_rates = rate_vector(reference);
    let cur_rates = rate_vector(current);
    let mut next_rates = [0.0; 4];
    let mut log_change_norm = 0.0;
    for idx in 0..4 {
        let proposed = cur_rates[idx] * dp[idx].exp();
        let global_min = ref_rates[idx] * D011_GLOBAL_RATE_MIN_FACTOR;
        let global_max = ref_rates[idx] * D011_GLOBAL_RATE_MAX_FACTOR;
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

pub fn bounded_joint_solver(
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
            solve_bounded_joint_step(reference, &current, g, &sensitivity, round)
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
    let bounded = candidates.len() <= D011_MAX_CANDIDATES;
    JointSolverReport {
        rounds_attempted: candidates.len().saturating_sub(1),
        candidates,
        bounded,
    }
}

fn rates_close(a: &StageEReferenceRates, b: &StageEReferenceRates) -> bool {
    rate_vector(a)
        .iter()
        .zip(rate_vector(b))
        .all(|(x, y)| (x - y).abs() <= 1e-12 * x.abs().max(1.0))
}

pub fn scientific_conclusion(
    any_joint_pass: bool,
    stage_e_revised: bool,
) -> &'static str {
    if any_joint_pass {
        if stage_e_revised {
            "D011_TRANSPORT_COUPLED_JOINT_BALANCE_PASS"
        } else {
            "D011_TRANSPORT_COUPLED_JOINT_BALANCE_PASS"
        }
    } else {
        "D011_TRANSPORT_COUPLED_BALANCE_NO_SOLUTION"
    }
}

pub fn stage_e_can_revise_to_pass(any_joint_pass: bool) -> bool {
    any_joint_pass
}
