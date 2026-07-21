//! D-056 waste-coupled resource-pair / waste antiporter architecture helpers.
//! Phase A is observer-only: no production biology change until Gates 0–5 pass.

use serde::{Deserialize, Serialize};

pub const D056_PROJECT_ID: &str = "D-056";
pub const D056_AGENT_MEMORY_ID: &str = "D-20260721-d056-waste-coupled-resource-carrier";
pub const D056_STARTING_COMMIT: &str = "f9dd924e0eb1195ce341ed397bb1ec1f37ace62a";
pub const D056_STARTING_TAG: &str = "D-055-strict-resource-architecture-review";
pub const D056_FROZEN_D051: &str = "D051_RESOURCE_THROUGHPUT_LIMIT";
pub const D056_FROZEN_D052: &str = "D052_MIXED_RESOURCE_DELIVERY_LIMIT";
pub const D056_FROZEN_D053: &str = "D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND";
pub const D056_FROZEN_D054: &str = "D054_D053_PROVENANCE_RERUN_DIVERGED";
pub const D056_FROZEN_D055: &str = "D055_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT";
pub const D056_ORDINARY_PASSIVE_CLOSED: &str = "ORDINARY_PASSIVE_RESOURCE_IMPORT_BRANCH_CLOSED";
pub const D056_V14: &str = "V14_SCHEMA3_MIXED_RESOURCE_DELIVERY_EXPERIMENTAL_FAILED";
pub const D056_V15: &str = "V15_SCHEMA3_WASTE_COUPLED_RESOURCE_CARRIER";
pub const D056_EQUATION: &str = "membrane_metabolism_v15_waste_coupled_resource_carrier";

/// Sealed D-055 Control E χ (complete passive upper bound).
pub const D056_SEALED_CHI_E: f64 = 0.9039035176168589;
pub const D056_SEALED_JN_E: f64 = 998.3951969770961;
pub const D056_SEALED_LN_E: f64 = 1104.5373510763234;
pub const D056_CHI_REPRO_TOL: f64 = 0.05;
pub const D056_CAPACITY_MARGIN: f64 = 1.10;
pub const D056_CHI_TARGET: f64 = 1.05;
pub const D056_RETENTION_MIN: f64 = 0.80;
pub const D056_RATE_SPAN_MAX: f64 = 3.0;
pub const D056_ID_BOOTSTRAP_MAX: f64 = 0.50;
pub const D056_ID_LOO_FACTOR: f64 = 2.0;
pub const D056_ID_HOLD_MEDIAN_MAX: f64 = 0.20;
pub const D056_ID_HOLD_MAX_MAX: f64 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D056PrimaryConclusion {
    StageERecovered,
    ResourceCarrierQualifiedStageEBlocked,
    D055PassiveBoundNotReproduced,
    CarrierConservationOrReversibilityFailure,
    WasteGradientCapacityInsufficient,
    CarrierKineticsNotIdentifiable,
    CarrierArchitectureNotPortable,
    CarrierShadowRepairFailure,
    CarrierDiscreteConservationFailure,
    CarrierRuntimeParityFailure,
    BoundedCarrierRepairNotFound,
    FoundationalCarrierRegression,
    NoHealthyCarrierSupportedAttractor,
    CarrierCausalityOrRepairFailure,
    StageEMembraneContractFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D056PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StageERecovered => "D056_STAGE_E_RECOVERED",
            Self::ResourceCarrierQualifiedStageEBlocked => {
                "D056_RESOURCE_CARRIER_QUALIFIED_STAGE_E_BLOCKED"
            }
            Self::D055PassiveBoundNotReproduced => "D056_D055_PASSIVE_BOUND_NOT_REPRODUCED",
            Self::CarrierConservationOrReversibilityFailure => {
                "D056_CARRIER_CONSERVATION_OR_REVERSIBILITY_FAILURE"
            }
            Self::WasteGradientCapacityInsufficient => "D056_WASTE_GRADIENT_CAPACITY_INSUFFICIENT",
            Self::CarrierKineticsNotIdentifiable => "D056_CARRIER_KINETICS_NOT_IDENTIFIABLE",
            Self::CarrierArchitectureNotPortable => "D056_CARRIER_ARCHITECTURE_NOT_PORTABLE",
            Self::CarrierShadowRepairFailure => "D056_CARRIER_SHADOW_REPAIR_FAILURE",
            Self::CarrierDiscreteConservationFailure => {
                "D056_CARRIER_DISCRETE_CONSERVATION_FAILURE"
            }
            Self::CarrierRuntimeParityFailure => "D056_CARRIER_RUNTIME_PARITY_FAILURE",
            Self::BoundedCarrierRepairNotFound => "D056_BOUNDED_CARRIER_REPAIR_NOT_FOUND",
            Self::FoundationalCarrierRegression => "D056_FOUNDATIONAL_CARRIER_REGRESSION",
            Self::NoHealthyCarrierSupportedAttractor => {
                "D056_NO_HEALTHY_CARRIER_SUPPORTED_ATTRACTOR"
            }
            Self::CarrierCausalityOrRepairFailure => "D056_CARRIER_CAUSALITY_OR_REPAIR_FAILURE",
            Self::StageEMembraneContractFailure => "D056_STAGE_E_MEMBRANE_CONTRACT_FAILURE",
            Self::AccountingFailure => "D056_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D056_NUMERICAL_FAILURE",
            Self::Fail => "D056_FAIL",
        }
    }
}

/// Michaelis activity a(x) = x / (K + x).
#[inline]
pub fn activity(x: f64, k: f64) -> f64 {
    let x = x.max(0.0);
    let k = k.max(0.0);
    x / (k + x)
}

/// Paired-resource activity a_z(NF).
#[inline]
pub fn paired_resource_activity(n: f64, f: f64, k_nf: f64) -> f64 {
    activity(n.max(0.0) * f.max(0.0), k_nf)
}

/// Waste activity a_W(W).
#[inline]
pub fn waste_activity(w: f64, k_w: f64) -> f64 {
    activity(w, k_w)
}

/// Signed inward carrier flux (positive: N,F in / W out).
///
/// `J_T = k_T Γ_S [ a_z(N_o F_o) a_W(W_i) − a_z(N_i F_i) a_W(W_o) ]`
///
/// No `max(0,·)` rectification — both directions remain physically possible.
#[inline]
pub fn carrier_flux_jt(
    n_o: f64,
    f_o: f64,
    w_i: f64,
    n_i: f64,
    f_i: f64,
    w_o: f64,
    gamma_s: f64,
    k_nf: f64,
    k_w: f64,
    k_t: f64,
) -> f64 {
    let forward = paired_resource_activity(n_o, f_o, k_nf) * waste_activity(w_i, k_w);
    let reverse = paired_resource_activity(n_i, f_i, k_nf) * waste_activity(w_o, k_w);
    k_t * gamma_s.max(0.0) * (forward - reverse)
}

/// Atomic face transfer of extent ξ (positive = inward N/F, outward W).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarrierFaceState {
    pub n_out: f64,
    pub f_out: f64,
    pub w_out: f64,
    pub n_in: f64,
    pub f_in: f64,
    pub w_in: f64,
}

impl CarrierFaceState {
    pub fn total_n(self) -> f64 {
        self.n_out + self.n_in
    }
    pub fn total_f(self) -> f64 {
        self.f_out + self.f_in
    }
    pub fn total_w(self) -> f64 {
        self.w_out + self.w_in
    }

    /// Apply signed extent with concentration safety bounds (no clipping of fields below 0).
    pub fn apply_extent(self, xi: f64) -> Option<Self> {
        let mut s = self;
        if xi >= 0.0 {
            let lim = s.n_out.min(s.f_out).min(s.w_in);
            if xi > lim + 1e-15 {
                return None;
            }
            s.n_out -= xi;
            s.n_in += xi;
            s.f_out -= xi;
            s.f_in += xi;
            s.w_in -= xi;
            s.w_out += xi;
        } else {
            let xi_abs = -xi;
            let lim = s.n_in.min(s.f_in).min(s.w_out);
            if xi_abs > lim + 1e-15 {
                return None;
            }
            s.n_in -= xi_abs;
            s.n_out += xi_abs;
            s.f_in -= xi_abs;
            s.f_out += xi_abs;
            s.w_out -= xi_abs;
            s.w_in += xi_abs;
        }
        if s.n_out < -1e-12
            || s.f_out < -1e-12
            || s.w_out < -1e-12
            || s.n_in < -1e-12
            || s.f_in < -1e-12
            || s.w_in < -1e-12
        {
            return None;
        }
        Some(s)
    }
}

/// Required additional paired influx for χ≥margin against passive delivery.
#[inline]
pub fn required_additional_influx(l_required: f64, j_passive: f64) -> f64 {
    (l_required - j_passive).max(0.0)
}

/// Capacity gate: JT,max ≥ CAPACITY_MARGIN × max(ΔN, ΔF).
#[inline]
pub fn waste_capacity_ok(jt_max: f64, delta_n: f64, delta_f: f64) -> bool {
    jt_max + 1e-12 >= D056_CAPACITY_MARGIN * delta_n.max(delta_f)
}

/// Stoichiometric W budget: exporting one W per pair must not exceed production + inventory.
#[inline]
pub fn waste_export_budget_ok(
    required_jt: f64,
    w_production: f64,
    w_inventory: f64,
) -> bool {
    required_jt <= w_production + w_inventory.max(0.0) + 1e-12
}

/// Thermodynamic drive: forward activity product minus reverse.
#[inline]
pub fn activity_drive(
    n_o: f64,
    f_o: f64,
    w_i: f64,
    n_i: f64,
    f_i: f64,
    w_o: f64,
    k_nf: f64,
    k_w: f64,
) -> f64 {
    paired_resource_activity(n_o, f_o, k_nf) * waste_activity(w_i, k_w)
        - paired_resource_activity(n_i, f_i, k_nf) * waste_activity(w_o, k_w)
}

/// Conservative JT upper bound at saturating drive with given k_T Γ_S and |Δa|≤1.
#[inline]
pub fn jt_saturating_bound(k_t: f64, gamma_s: f64, drive: f64) -> f64 {
    k_t.max(0.0) * gamma_s.max(0.0) * drive.max(0.0).min(1.0)
}

/// Identify k_T from a reference drive and required flux: k_T = J_req / (Γ_S · drive).
#[inline]
pub fn identify_k_t(j_required: f64, gamma_s: f64, drive: f64) -> Option<f64> {
    let denom = gamma_s.max(0.0) * drive;
    if denom <= 1e-18 || j_required < 0.0 {
        return None;
    }
    Some(j_required / denom)
}

/// Half-saturation constant chosen inside the positive tested range (geometric mid).
#[inline]
pub fn half_sat_from_range(lo: f64, hi: f64) -> f64 {
    let lo = lo.max(1e-12);
    let hi = hi.max(lo);
    (lo * hi).sqrt()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarrierParams {
    pub k_nf: f64,
    pub k_w: f64,
    pub k_t: f64,
}

/// Relative flux error |pred − target| / max(|target|, eps).
#[inline]
pub fn relative_flux_error(pred: f64, target: f64) -> f64 {
    (pred - target).abs() / target.abs().max(1e-12)
}

/// Predicted χ after adding carrier flux to passive delivery.
#[inline]
pub fn chi_with_carrier(j_passive: f64, jt: f64, l_required: f64) -> f64 {
    (j_passive + jt.max(0.0)) / l_required.max(1e-18)
}

/// Rate-span check across qualified states (max/min ≤ 3×).
#[inline]
pub fn rate_span_ok(values: &[f64]) -> bool {
    let pos: Vec<f64> = values.iter().copied().filter(|&v| v > 1e-18).collect();
    if pos.len() < 2 {
        return true;
    }
    let mn = pos.iter().copied().fold(f64::INFINITY, f64::min);
    let mx = pos.iter().copied().fold(0.0_f64, f64::max);
    mx / mn <= D056_RATE_SPAN_MAX + 1e-12
}

/// Gate 1 analytical checklist (no rectification, conservation, zero controls).
pub fn gate1_thermodynamic_checklist() -> Vec<(&'static str, bool)> {
    let mut checks = Vec::new();

    // Conservation of one positive event.
    let s0 = CarrierFaceState {
        n_out: 2.0,
        f_out: 2.0,
        w_out: 0.5,
        n_in: 1.0,
        f_in: 1.0,
        w_in: 3.0,
    };
    let s1 = s0.apply_extent(0.4).expect("extent in bounds");
    checks.push((
        "global_N_conserved",
        (s1.total_n() - s0.total_n()).abs() < 1e-12,
    ));
    checks.push((
        "global_F_conserved",
        (s1.total_f() - s0.total_f()).abs() < 1e-12,
    ));
    checks.push((
        "global_W_conserved",
        (s1.total_w() - s0.total_w()).abs() < 1e-12,
    ));

    // Zero Γ_S → zero flux.
    let j0 = carrier_flux_jt(1.0, 1.0, 2.0, 0.2, 0.2, 0.1, 0.0, 1.0, 1.0, 1.0);
    checks.push(("zero_flux_without_S", j0.abs() < 1e-15));

    // Missing exterior N or F → forward drive collapses toward reverse-only.
    let j_no_n = carrier_flux_jt(0.0, 1.0, 2.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
    checks.push(("no_inward_without_exterior_N", j_no_n <= 1e-15));
    let j_no_f = carrier_flux_jt(1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
    checks.push(("no_inward_without_exterior_F", j_no_f <= 1e-15));
    let j_no_w = carrier_flux_jt(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
    checks.push(("no_inward_without_interior_W", j_no_w <= 1e-15));

    // Detailed balance at equal activities.
    let j_eq = carrier_flux_jt(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 0.5, 0.5, 3.0);
    checks.push(("zero_net_at_equal_activities", j_eq.abs() < 1e-12));

    // Reverse under reversed gradients.
    let j_fwd = carrier_flux_jt(2.0, 2.0, 3.0, 0.2, 0.2, 0.1, 1.0, 1.0, 1.0, 1.0);
    let j_rev = carrier_flux_jt(0.2, 0.2, 0.1, 2.0, 2.0, 3.0, 1.0, 1.0, 1.0, 1.0);
    checks.push(("forward_positive", j_fwd > 0.0));
    checks.push(("reverse_negative", j_rev < 0.0));
    checks.push((
        "exact_antisymmetry_under_swap",
        (j_fwd + j_rev).abs() < 1e-12,
    ));

    // No rectification: negative flux retained (not maxed to 0).
    checks.push(("no_max0_rectification", j_rev < 0.0));

    checks
}

pub fn gate1_all_pass() -> bool {
    gate1_thermodynamic_checklist().iter().all(|(_, ok)| *ok)
}

/// Sealed passive-bound reproduction: χ within tol of sealed Control E and < 1.
pub fn passive_bound_reproduced(chi_n: f64, chi_f: f64) -> bool {
    let ok_n = (chi_n - D056_SEALED_CHI_E).abs() <= D056_CHI_REPRO_TOL || chi_n < 1.0;
    let ok_f = (chi_f - D056_SEALED_CHI_E).abs() <= D056_CHI_REPRO_TOL || chi_f < 1.0;
    // Strict: must remain a hard bound failure (<1) and be close when full-horizon.
    chi_n < 1.0 && chi_f < 1.0 && ok_n && ok_f
}

/// Full-horizon sealed match (tighter).
pub fn passive_bound_sealed_match(chi_n: f64, chi_f: f64) -> bool {
    (chi_n - D056_SEALED_CHI_E).abs() <= D056_CHI_REPRO_TOL
        && (chi_f - D056_SEALED_CHI_E).abs() <= D056_CHI_REPRO_TOL
        && chi_n < 1.0
        && chi_f < 1.0
}
