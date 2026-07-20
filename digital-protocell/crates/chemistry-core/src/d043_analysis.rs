//! D-043 activation-reaction capacity repair (observer-only analysis helpers).
//!
//! Production rate law (from `activated_metabolism.rs`):
//! ```text
//! B_activation(c,n,f) = max(0,c) * max(0,n) * max(0,f)
//! r_activation        = k_d008_activation * B_activation
//! ```
//! Stoichiometry N+F→A+W: extent Δ ⇒ N−, F−, A+, W+.

use crate::activated_metabolism::{
    activated_metabolism_rates, activation_isolated_delta,
};
use crate::config::SimParams;
use crate::d012_accounting::{activation_potential, E_ACTIVATED, E_FUEL};
use crate::d042_analysis::{ALedgerIntegral, ALedgerTerms, CumulativeABalance};
use serde::{Deserialize, Serialize};

pub const D043_STARTING_COMMIT: &str = "6c7328f";
pub const D043_D042_TAG: &str = "D-042-activation-buffer-feasibility";
pub const D043_AGENT_MEMORY_ID: &str =
    "D-20260719-2116-d043-activation-reaction-capacity-repair";
pub const D043_RECORD: &str = "ACTIVATION_BUFFER_BRANCH_CLOSED";
pub const D043_HISTORICAL_K_ACTIVATION: f64 = 0.020;
pub const D043_REPAIR_P_MIN: f64 = 0.020;
pub const D043_GATE0_HORIZON: u64 = 25_000;
pub const D043_MAX_ACCEPTED: u64 = 200_000;
pub const D043_MEASURE_WINDOW: u64 = 500;
pub const D043_LEDGER_REL_TOL: f64 = 1e-4;
pub const D043_LATE_BALANCE_EPS: f64 = 1e-12;
pub const D043_MAX_CANDIDATES: usize = 5;
pub const D043_PORTABLE_MIN_ESTIMATES: usize = 6;
pub const D043_PORTABLE_MAX_SPAN: f64 = 3.0;
pub const D043_PORTABLE_LOO_MEDIAN_TOL: f64 = 0.50;
pub const D043_BASIS_FLOOR: f64 = 1e-9;
pub const D043_RATE_PARITY_REL_TOL: f64 = 1e-9;

/// Selected k applied only in D-043 validation runs after Gate 9 QUALIFIED.
pub const D043_SELECTED_K_ACTIVATION: Option<f64> = None;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D043Conclusion {
    ActivationRateRepairQualified,
    D042CapacityDeficitNotReproduced,
    ActivationImplementationDefect,
    ActivationSubstrateDeliveryDefect,
    ActivationCatalystBasisDeficit,
    ActivatedResourceDecayDefect,
    ActivatedResourceDemandDefect,
    ActivationRateNotPortable,
    ActivationRateRepairNotFound,
    FoundationalActivationRegression,
    MembraneBasinNotRecovered,
    ContinuousReplacementNotRecovered,
    DamageRepairNotRecovered,
    ResourceDependenceNotEstablished,
    StageEMembraneContractFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D043Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActivationRateRepairQualified => "D043_ACTIVATION_RATE_REPAIR_QUALIFIED",
            Self::D042CapacityDeficitNotReproduced => "D043_D042_CAPACITY_DEFICIT_NOT_REPRODUCED",
            Self::ActivationImplementationDefect => "D043_ACTIVATION_IMPLEMENTATION_DEFECT",
            Self::ActivationSubstrateDeliveryDefect => "D043_ACTIVATION_SUBSTRATE_DELIVERY_DEFECT",
            Self::ActivationCatalystBasisDeficit => "D043_ACTIVATION_CATALYST_BASIS_DEFICIT",
            Self::ActivatedResourceDecayDefect => "D043_ACTIVATED_RESOURCE_DECAY_DEFECT",
            Self::ActivatedResourceDemandDefect => "D043_ACTIVATED_RESOURCE_DEMAND_DEFECT",
            Self::ActivationRateNotPortable => "D043_ACTIVATION_RATE_NOT_PORTABLE",
            Self::ActivationRateRepairNotFound => "D043_ACTIVATION_RATE_REPAIR_NOT_FOUND",
            Self::FoundationalActivationRegression => "D043_FOUNDATIONAL_ACTIVATION_REGRESSION",
            Self::MembraneBasinNotRecovered => "D043_MEMBRANE_BASIN_NOT_RECOVERED",
            Self::ContinuousReplacementNotRecovered => "D043_CONTINUOUS_REPLACEMENT_NOT_RECOVERED",
            Self::DamageRepairNotRecovered => "D043_DAMAGE_REPAIR_NOT_RECOVERED",
            Self::ResourceDependenceNotEstablished => "D043_RESOURCE_DEPENDENCE_NOT_ESTABLISHED",
            Self::StageEMembraneContractFailure => "D043_STAGE_E_MEMBRANE_CONTRACT_FAILURE",
            Self::AccountingFailure => "D043_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D043_NUMERICAL_FAILURE",
            Self::Fail => "D043_FAIL",
        }
    }
}

/// Complete old-state reaction basis B_activation = c·n·f (no interior weighting, no saturation).
#[inline]
pub fn activation_basis(catalyst: f64, nutrient: f64, fuel: f64) -> f64 {
    catalyst.max(0.0) * nutrient.max(0.0) * fuel.max(0.0)
}

/// Observer rate r = k · B.
#[inline]
pub fn activation_rate(k: f64, catalyst: f64, nutrient: f64, fuel: f64) -> f64 {
    k * activation_basis(catalyst, nutrient, fuel)
}

/// Production runtime parity: observer basis/rate vs `activated_metabolism_rates`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ActivationParitySample {
    pub c: f64,
    pub n: f64,
    pub f: f64,
    pub a: f64,
    pub basis_observer: f64,
    pub basis_runtime: f64,
    pub rate_observer: f64,
    pub rate_runtime: f64,
    pub basis_match: bool,
    pub rate_match: bool,
}

pub fn check_activation_parity(
    k: f64,
    c: f64,
    n: f64,
    f: f64,
    a: f64,
    params: &SimParams,
) -> ActivationParitySample {
    let mut p = params.clone();
    p.k_d008_activation = k;
    let basis_obs = activation_basis(c, n, f);
    let rate_obs = activation_rate(k, c, n, f);
    let rates = activated_metabolism_rates(c, n, f, a, &p);
    // Runtime basis inferred from r/k when k>0; zero-k uses observer basis.
    let basis_runtime = if k.abs() > 1e-18 {
        rates.activation / k
    } else {
        basis_obs
    };
    let basis_match = relative_match(basis_obs, basis_runtime, D043_RATE_PARITY_REL_TOL);
    let rate_match = relative_match(rate_obs, rates.activation, D043_RATE_PARITY_REL_TOL);
    ActivationParitySample {
        c,
        n,
        f,
        a,
        basis_observer: basis_obs,
        basis_runtime,
        rate_observer: rate_obs,
        rate_runtime: rates.activation,
        basis_match,
        rate_match,
    }
}

fn relative_match(a: f64, b: f64, rel_tol: f64) -> bool {
    let scale = a.abs().max(b.abs()).max(1.0);
    (a - b).abs() / scale <= rel_tol
}

/// Stoichiometry parity for extent Δ: N−, F−, A+, W+.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ActivationStoichiometrySample {
    pub extent: f64,
    pub d_n: f64,
    pub d_f: f64,
    pub d_a: f64,
    pub d_w: f64,
    pub pass: bool,
}

pub fn check_activation_stoichiometry(extent: f64) -> ActivationStoichiometrySample {
    let d = activation_isolated_delta(extent);
    let pass = (d[2] + extent).abs() < 1e-15
        && (d[3] + extent).abs() < 1e-15
        && (d[5] - extent).abs() < 1e-15
        && (d[4] - extent).abs() < 1e-15;
    ActivationStoichiometrySample {
        extent,
        d_n: d[2],
        d_f: d[3],
        d_a: d[5],
        d_w: d[4],
        pass,
    }
}

/// Activation-potential transfer: ΔΦ = e_F·ΔF + e_A·ΔA = −e_F·Δ + e_A·Δ = (e_A − e_F)·Δ = 0 for e_F=e_A=1.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ActivationPotentialTransfer {
    pub fuel_before: f64,
    pub fuel_after: f64,
    pub activated_before: f64,
    pub activated_after: f64,
    pub potential_before: f64,
    pub potential_after: f64,
    pub chemistry_potential_change: f64,
    pub pass: bool,
}

pub fn check_activation_potential_transfer(extent: f64) -> ActivationPotentialTransfer {
    let d = activation_isolated_delta(extent);
    let fuel_before = 1.0;
    let activated_before = 0.5;
    let fuel_after = fuel_before + d[3];
    let activated_after = activated_before + d[5];
    let phi_before = activation_potential(fuel_before, activated_before);
    let phi_after = activation_potential(fuel_after, activated_after);
    let chem = E_FUEL * d[3] + E_ACTIVATED * d[5];
    ActivationPotentialTransfer {
        fuel_before,
        fuel_after,
        activated_before,
        activated_after,
        potential_before: phi_before,
        potential_after: phi_after,
        chemistry_potential_change: chem,
        pass: chem.abs() <= 1e-12,
    }
}

/// Zero controls: no C, N, or F ⇒ zero activation.
pub fn zero_control_passes(k: f64, params: &SimParams) -> bool {
    let samples = [
        (0.0, 1.0, 1.0),
        (1.0, 0.0, 1.0),
        (1.0, 1.0, 0.0),
        (0.0, 0.0, 0.0),
    ];
    samples.iter().all(|&(c, n, f)| {
        let r = check_activation_parity(k, c, n, f, 0.0, params);
        r.rate_runtime.abs() <= 1e-15 && r.rate_observer.abs() <= 1e-15
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapacityClassification {
    SubstrateDelivery,
    CatalystBasis,
    RateCapacity,
    DecayDefect,
    DemandDefect,
}

impl CapacityClassification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubstrateDelivery => "SUBSTRATE_DELIVERY",
            Self::CatalystBasis => "CATALYST_BASIS",
            Self::RateCapacity => "RATE_CAPACITY",
            Self::DecayDefect => "DECAY_DEFECT",
            Self::DemandDefect => "DEMAND_DEFECT",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityDecompositionRow {
    pub name: String,
    pub integrated_mean_r_a: f64,
    pub activation_basis: f64,
    pub activation_rate: f64,
    pub valid: bool,
}

/// Classify activation-capacity deficit from diagnostic control balances.
pub fn classify_capacity_deficit(
    historical_r_a: f64,
    healthy_n_r_a: f64,
    healthy_f_r_a: f64,
    healthy_c_r_a: f64,
    healthy_nf_r_a: f64,
    healthy_cnf_r_a: f64,
    no_decay_r_a: f64,
    demands_disabled_r_a: f64,
    eps: f64,
) -> (CapacityClassification, Option<String>) {
    if no_decay_r_a >= -eps {
        return (CapacityClassification::DecayDefect, Some("a_decay".into()));
    }
    if demands_disabled_r_a >= -eps {
        return (
            CapacityClassification::DemandDefect,
            Some("productive_demands".into()),
        );
    }
    // Substrate delivery: N and/or F restoration alone closes the sustained deficit.
    if healthy_nf_r_a >= -eps || healthy_n_r_a >= -eps || healthy_f_r_a >= -eps {
        return (CapacityClassification::SubstrateDelivery, None);
    }
    // Catalyst basis: healthy C is required (alone or with N/F) while N/F alone do not close.
    if healthy_c_r_a >= -eps || healthy_cnf_r_a >= -eps {
        return (CapacityClassification::CatalystBasis, None);
    }
    // Rate capacity: even governed healthy C/N/F leave a persistent deficit.
    if historical_r_a < -eps && healthy_cnf_r_a < -eps {
        return (CapacityClassification::RateCapacity, None);
    }
    (CapacityClassification::RateCapacity, None)
}

/// One portable-rate estimate from a balance-eligible state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateEstimate {
    pub label: String,
    pub basis: f64,
    pub l_a: f64,
    pub k_required: f64,
    pub c: f64,
    pub n: f64,
    pub f: f64,
    pub valid: bool,
    pub dominated_by_near_zero: bool,
}

pub fn required_k_activation(l_a: f64, basis: f64) -> f64 {
    if basis <= D043_BASIS_FLOOR {
        return f64::INFINITY;
    }
    l_a / basis
}

pub fn sustained_a_loss(terms: &ALedgerTerms) -> f64 {
    terms.j_demands() + terms.j_decay + terms.j_out - terms.j_in
}

/// Build a rate estimate from domain-total reaction basis.
///
/// `total_basis` must be the production old-state integral Σ(C·N·F) over the domain
/// (equivalently `j_activation / k` when measured under historical k). Do not use
/// mean-cell basis against domain-total L_A — that inflates k_required by ~interior volume.
pub fn build_rate_estimate(
    label: &str,
    c: f64,
    n: f64,
    f: f64,
    total_basis: f64,
    terms: &ALedgerTerms,
    min_species: f64,
) -> RateEstimate {
    let l_a = sustained_a_loss(terms);
    let dominated =
        c < min_species || n < min_species || f < min_species || total_basis < D043_BASIS_FLOOR;
    let valid = total_basis >= D043_BASIS_FLOOR
        && l_a.is_finite()
        && l_a >= 0.0
        && !dominated;
    RateEstimate {
        label: label.to_string(),
        basis: total_basis,
        l_a,
        k_required: if valid {
            required_k_activation(l_a, total_basis)
        } else {
            f64::INFINITY
        },
        c,
        n,
        f,
        valid,
        dominated_by_near_zero: dominated,
    }
}

/// Production-exact domain basis from measured activation flux and the k used to produce it.
#[inline]
pub fn total_basis_from_activation_flux(j_activation: f64, k: f64) -> f64 {
    if k.abs() <= 1e-18 {
        return 0.0;
    }
    (j_activation / k).max(0.0)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortableRateReport {
    pub estimates: Vec<RateEstimate>,
    pub valid_count: usize,
    pub k_min: f64,
    pub k_max: f64,
    pub k_median: f64,
    pub span: f64,
    pub loo_median_max_deviation: f64,
    pub pass: bool,
}

/// Relative basis floor vs median: estimates with B ≪ peers are treated as
/// near-zero-substrate dominated (cubic mass-action product collapse).
pub const D043_RELATIVE_BASIS_FLOOR: f64 = 0.25;

pub fn evaluate_portable_rate(estimates: &[RateEstimate]) -> PortableRateReport {
    // First pass: species-valid estimates define the median basis peer group.
    let species_ok: Vec<&RateEstimate> = estimates
        .iter()
        .filter(|e| e.valid && !e.dominated_by_near_zero)
        .collect();
    let bases: Vec<f64> = species_ok.iter().map(|e| e.basis).collect();
    let median_basis = median(&bases);
    let basis_floor = (D043_RELATIVE_BASIS_FLOOR * median_basis).max(D043_BASIS_FLOOR);

    // Second pass: invalidate relative-basis collapses so they cannot inflate span.
    let mut adjusted = estimates.to_vec();
    for e in &mut adjusted {
        if e.valid && e.basis < basis_floor {
            e.valid = false;
            e.dominated_by_near_zero = true;
            e.k_required = f64::INFINITY;
        }
    }

    let valid: Vec<&RateEstimate> = adjusted.iter().filter(|e| e.valid).collect();
    let ks: Vec<f64> = valid.iter().map(|e| e.k_required).collect();
    let valid_count = ks.len();
    let mut k_min = f64::INFINITY;
    let mut k_max = 0.0_f64;
    for &k in &ks {
        k_min = k_min.min(k);
        k_max = k_max.max(k);
    }
    let k_median = median(&ks);
    let span = if k_min.is_finite() && k_min > 0.0 {
        k_max / k_min
    } else {
        f64::INFINITY
    };
    let loo_max = leave_one_out_median_max_deviation(&ks);
    let pass = valid_count >= D043_PORTABLE_MIN_ESTIMATES
        && span <= D043_PORTABLE_MAX_SPAN
        && loo_max <= D043_PORTABLE_LOO_MEDIAN_TOL
        && ks.iter().all(|k| k.is_finite() && *k > 0.0);
    PortableRateReport {
        estimates: adjusted,
        valid_count,
        k_min,
        k_max,
        k_median,
        span,
        loo_median_max_deviation: loo_max,
        pass,
    }
}

fn median(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = s.len();
    if n % 2 == 1 {
        s[n / 2]
    } else {
        (s[n / 2 - 1] + s[n / 2]) / 2.0
    }
}

fn leave_one_out_median_max_deviation(ks: &[f64]) -> f64 {
    if ks.len() < 2 {
        return 0.0;
    }
    let full_median = median(ks);
    let mut max_dev = 0.0_f64;
    for i in 0..ks.len() {
        let subset: Vec<f64> = ks
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, &k)| k)
            .collect();
        let m = median(&subset);
        if full_median.abs() > 1e-18 {
            let dev = (m - full_median).abs() / full_median.abs();
            max_dev = max_dev.max(dev);
        }
    }
    max_dev
}

/// Candidate screen outcome for one k value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateScreenRow {
    pub k: f64,
    pub factor: f64,
    pub integrated_r_a: f64,
    pub free_a: f64,
    pub p_activity: f64,
    pub theta: f64,
    pub n_available: f64,
    pub f_available: f64,
    pub c_available: f64,
    pub pass: bool,
    pub reject_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateScreenReport {
    pub reconstructed_k: f64,
    pub candidates: Vec<CandidateScreenRow>,
    pub selected_k: Option<f64>,
    pub pass: bool,
}

/// Build ≤5 candidates from reconstructed median k.
pub fn build_activation_candidates(reconstructed_k: f64) -> Vec<f64> {
    let hist = D043_HISTORICAL_K_ACTIVATION;
    let mut raw = vec![
        hist,
        0.75 * reconstructed_k,
        1.00 * reconstructed_k,
        1.25 * reconstructed_k,
    ];
    // Bracketed intermediate when historical sits outside the 0.75–1.25 band.
    if hist < 0.75 * reconstructed_k || hist > 1.25 * reconstructed_k {
        raw.push(0.875 * reconstructed_k);
    }
    raw.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    raw.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
    raw.truncate(D043_MAX_CANDIDATES);
    raw
}

/// Evaluate short-screen pass criteria for one candidate row.
pub fn evaluate_candidate_row(
    k: f64,
    reconstructed_k: f64,
    integrated_r_a: f64,
    free_a: f64,
    p_activity: f64,
    theta: f64,
    n_available: f64,
    f_available: f64,
    c_available: f64,
    ledger_closes: bool,
    exhaustion: bool,
    accumulation: bool,
    clipping: bool,
) -> CandidateScreenRow {
    let factor = k / reconstructed_k.max(1e-18);
    let mut reject = None;
    if exhaustion {
        reject = Some("resource_exhaustion".into());
    } else if accumulation {
        reject = Some("a_accumulation".into());
    } else if clipping {
        reject = Some("numerical_clipping".into());
    } else if !ledger_closes {
        reject = Some("ledger_failure".into());
    } else if integrated_r_a < -D043_LATE_BALANCE_EPS {
        reject = Some("negative_integrated_balance".into());
    } else if p_activity < D043_REPAIR_P_MIN {
        reject = Some("p_below_repair_min".into());
    } else if n_available <= D043_BASIS_FLOOR || f_available <= D043_BASIS_FLOOR {
        reject = Some("substrate_depleted".into());
    } else if c_available <= D043_BASIS_FLOOR {
        reject = Some("catalyst_extinct".into());
    } else if !free_a.is_finite() || free_a > 1e6 {
        reject = Some("unbounded_free_a".into());
    }
    let pass = reject.is_none();
    CandidateScreenRow {
        k,
        factor,
        integrated_r_a,
        free_a,
        p_activity,
        theta,
        n_available,
        f_available,
        c_available,
        pass,
        reject_reason: reject,
    }
}

/// Select smallest passing candidate.
pub fn select_smallest_passing(candidates: &[CandidateScreenRow]) -> Option<f64> {
    candidates
        .iter()
        .filter(|c| c.pass)
        .min_by(|a, b| a.k.partial_cmp(&b.k).unwrap_or(std::cmp::Ordering::Equal))
        .map(|c| c.k)
}

pub fn screen_candidates(
    reconstructed_k: f64,
    rows: Vec<CandidateScreenRow>,
) -> CandidateScreenReport {
    let selected = select_smallest_passing(&rows);
    CandidateScreenReport {
        reconstructed_k,
        pass: selected.is_some(),
        selected_k: selected,
        candidates: rows,
    }
}

/// D-042 capacity deficit reproduction check from integrated ledger.
pub fn d042_capacity_deficit_reproduced(integ: &ALedgerIntegral, windows: usize) -> bool {
    integ.integrated_r_a < -D043_LATE_BALANCE_EPS
        && integ.closes(D043_LEDGER_REL_TOL)
        && windows >= 3
}

pub fn a_decline_precedes_demand(cumulative: &CumulativeABalance, demand_series: &[f64]) -> bool {
    if cumulative.e.len() < 3 || demand_series.len() < 3 {
        return false;
    }
    let a_trough_idx = cumulative
        .e
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let dem_trough_idx = demand_series
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(demand_series.len());
    a_trough_idx <= dem_trough_idx.saturating_add(1)
}

pub fn parity_suite_passes(k: f64, params: &SimParams) -> bool {
    if !zero_control_passes(k, params) {
        return false;
    }
    let grid = [
        (0.5, 0.5, 0.5, 0.2),
        (1.0, 0.8, 0.6, 0.3),
        (0.2, 1.0, 1.0, 0.1),
    ];
    grid.iter().all(|&(c, n, f, a)| {
        let p = check_activation_parity(k, c, n, f, a, params);
        p.basis_match && p.rate_match
    }) && check_activation_stoichiometry(0.05).pass
        && check_activation_potential_transfer(0.05).pass
}
#[cfg(test)]
mod inline_tests {
    use super::*;
    use crate::config::SimParams;

    #[test]
    fn basis_and_rate_law() {
        assert!((activation_basis(2.0, 3.0, 4.0) - 24.0).abs() < 1e-15);
        assert!((activation_rate(0.02, 2.0, 3.0, 4.0) - 0.48).abs() < 1e-15);
        assert_eq!(activation_basis(-1.0, 2.0, 3.0), 0.0);
    }

    #[test]
    fn stoichiometry_and_potential() {
        assert!(check_activation_stoichiometry(0.1).pass);
        assert!(check_activation_potential_transfer(0.1).pass);
    }
}
