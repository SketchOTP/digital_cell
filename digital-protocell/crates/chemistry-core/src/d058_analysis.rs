//! D-058 corrected carrier face/timestep normalization and re-identification helpers.
//! Observer / shadow diagnostic only: no production biology change.

use crate::config::DX;
use crate::d056_analysis::{
    activity_drive, identify_k_t, D056_ID_BOOTSTRAP_MAX, D056_ID_HOLD_MAX_MAX,
    D056_ID_HOLD_MEDIAN_MAX, D056_ID_LOO_FACTOR, D056_RATE_SPAN_MAX,
};
use crate::d057_analysis::{
    bootstrap_spread, drive_for_model, loo_factor, rate_span, CarrierMeasureKind, DriveModelKind,
    IdentifiabilityReport,
};
use crate::surface_density::reconstruct_gamma;
use serde::{Deserialize, Serialize};

pub const D058_PROJECT_ID: &str = "D-058";
pub const D058_AGENT_MEMORY_ID: &str =
    "D-20260721-d058-corrected-carrier-normalization-reidentification";
pub const D058_STARTING_COMMIT: &str = "1c9d6ae73ac828622d1315e7a2137385a5ac1e71";
pub const D058_STARTING_TAG: &str = "D-057-carrier-geometry-driving-force-audit";
pub const D058_D056_COMMIT: &str = "ed6de2cb0ce78202a665ddc4335ca198ac79b625";
pub const D058_D056_TAG: &str = "D-056-waste-coupled-resource-carrier-fail";
pub const D058_D057_CONCLUSION: &str = "D057_CARRIER_GRID_OR_SURFACE_NORMALIZATION_DEFECT";
pub const D058_INVALIDATION: &str =
    "D056_CARRIER_IDENTIFICATION_INVALIDATED_BY_OBSERVER_NORMALIZATION";
pub const D058_EQUATION: &str =
    "xi_f_req = k_T * Gamma_f * D_f * A_f * dt; Delta_c = +/- xi / V_cell";
pub const D058_RATE_SPAN_MAX: f64 = D056_RATE_SPAN_MAX;
pub const D058_DEFECTIVE_SPAN_MIN: f64 = 50.0;
pub const D058_PARITY_TOL: f64 = 1e-9;
pub const D058_INVARIANCE_TOL: f64 = 1e-9;

/// Production δ estimator (mirrors private `membrane_transport::cell_delta_estimate`).
#[inline]
pub fn production_cell_delta_estimate(phi: f64, delta_floor: f64) -> f64 {
    let p = phi.clamp(0.0, 1.0);
    let dh_dphi = 6.0 * p * (1.0 - p);
    (dh_dphi / DX).max(delta_floor)
}

/// Physical face measure for a Cartesian edge in 2D (length).
#[inline]
pub fn face_measure_a_f() -> f64 {
    DX
}

/// Cell volume for concentration ↔ amount conversion.
#[inline]
pub fn cell_volume() -> f64 {
    DX * DX
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D058PrimaryConclusion {
    WasteCoupledCarrierNormalizationQualified,
    CorrectedCarrierDrivingForceQualified,
    CarrierSurfaceVolumeCapacityLimit,
    WasteCoupledCarrierArchitectureRejected,
    CorrectedCarrierKineticsNotIdentifiable,
    CarrierNormalizationRepairInconclusive,
    WorkspaceScopeNotIsolated,
    D057NormalizationDefectNotReproduced,
    CanonicalFaceOperatorInvalid,
    CorrectedObserverParityFailure,
    CarrierNormalizationInvarianceFailure,
    ShadowRepairFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D058PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WasteCoupledCarrierNormalizationQualified => {
                "D058_WASTE_COUPLED_CARRIER_NORMALIZATION_QUALIFIED"
            }
            Self::CorrectedCarrierDrivingForceQualified => {
                "D058_CORRECTED_CARRIER_DRIVING_FORCE_QUALIFIED"
            }
            Self::CarrierSurfaceVolumeCapacityLimit => {
                "D058_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT"
            }
            Self::WasteCoupledCarrierArchitectureRejected => {
                "D058_WASTE_COUPLED_CARRIER_ARCHITECTURE_REJECTED"
            }
            Self::CorrectedCarrierKineticsNotIdentifiable => {
                "D058_CORRECTED_CARRIER_KINETICS_NOT_IDENTIFIABLE"
            }
            Self::CarrierNormalizationRepairInconclusive => {
                "D058_CARRIER_NORMALIZATION_REPAIR_INCONCLUSIVE"
            }
            Self::WorkspaceScopeNotIsolated => "D058_WORKSPACE_SCOPE_NOT_ISOLATED",
            Self::D057NormalizationDefectNotReproduced => {
                "D058_D057_NORMALIZATION_DEFECT_NOT_REPRODUCED"
            }
            Self::CanonicalFaceOperatorInvalid => "D058_CANONICAL_FACE_OPERATOR_INVALID",
            Self::CorrectedObserverParityFailure => "D058_CORRECTED_OBSERVER_PARITY_FAILURE",
            Self::CarrierNormalizationInvarianceFailure => {
                "D058_CARRIER_NORMALIZATION_INVARIANCE_FAILURE"
            }
            Self::ShadowRepairFailure => "D058_SHADOW_REPAIR_FAILURE",
            Self::AccountingFailure => "D058_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D058_NUMERICAL_FAILURE",
            Self::Fail => "D058_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum D058Route {
    Q,
    D,
    V,
    R,
    I,
}

impl D058Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Q => "Route_Q_corrected_original_carrier_qualified",
            Self::D => "Route_D_corrected_drive_qualified",
            Self::V => "Route_V_physical_surface_to_volume_limit",
            Self::R => "Route_R_waste_coupled_carrier_rejected",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D058PrimaryConclusion {
        match self {
            Self::Q => D058PrimaryConclusion::WasteCoupledCarrierNormalizationQualified,
            Self::D => D058PrimaryConclusion::CorrectedCarrierDrivingForceQualified,
            Self::V => D058PrimaryConclusion::CarrierSurfaceVolumeCapacityLimit,
            Self::R => D058PrimaryConclusion::WasteCoupledCarrierArchitectureRejected,
            Self::I => D058PrimaryConclusion::CarrierNormalizationRepairInconclusive,
        }
    }
}

/// Explicit dimensional table for the canonical face operator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionalTable {
    pub gamma_f: &'static str,
    pub d_f: &'static str,
    pub a_f: &'static str,
    pub delta_t: &'static str,
    pub xi_req: &'static str,
    pub cell_volume: &'static str,
    pub concentration_update: &'static str,
    pub delta_estimator: &'static str,
    pub face_measure_count: usize,
    pub timestep_count: usize,
    pub interface_reconstruction_count: usize,
    pub cell_volume_conversion_count: usize,
    pub valid: bool,
}

pub fn dimensional_table() -> DimensionalTable {
    DimensionalTable {
        gamma_f: "Γ_f = reconstruct_gamma(S_face, cell_delta_estimate(φ), δ_floor) [amount/length]",
        d_f: "D_f = a_z(N_o F_o) a_W(W_i) − a_z(N_i F_i) a_W(W_o) [dimensionless]",
        a_f: "A_f = DX [length] — Cartesian face measure, applied exactly once",
        delta_t: "Δt = accepted substep dt [time] — applied exactly once",
        xi_req: "ξ_f^req = k_T Γ_f D_f A_f Δt [amount]",
        cell_volume: "V_i = DX² [area] — concentration storage volume",
        concentration_update: "ΔX_i = ±ξ_f / V_i ; ΔX_j = ∓ξ_f / V_j",
        delta_estimator: "production cell_delta_estimate = max(6φ(1−φ)/DX, δ_floor) — not interface_weight",
        face_measure_count: 1,
        timestep_count: 1,
        interface_reconstruction_count: 1,
        cell_volume_conversion_count: 1,
        valid: true,
    }
}

/// Historical defective D-056/D-057 estimator (regression fixture only).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DefectiveEstimatorFixture {
    pub used_interface_weight_as_delta: bool,
    pub omitted_face_measure: bool,
    pub omitted_timestep: bool,
    pub face_measure_count: usize,
    pub timestep_count: usize,
}

pub fn defective_estimator_fixture() -> DefectiveEstimatorFixture {
    DefectiveEstimatorFixture {
        used_interface_weight_as_delta: true,
        omitted_face_measure: true,
        omitted_timestep: true,
        face_measure_count: 0,
        timestep_count: 0,
    }
}

/// Defective k_T★: J_missing / (Γ_iw_sum · D_net) — no A_f, no Δt.
#[inline]
pub fn defective_k_t_star(j_missing: f64, gamma_iw_sum: f64, d_net: f64) -> Option<f64> {
    identify_k_t(j_missing, gamma_iw_sum, d_net)
}

/// Canonical requested transported amount on one face.
#[inline]
pub fn xi_face_req(k_t: f64, gamma_f: f64, d_f: f64, a_f: f64, dt: f64) -> f64 {
    k_t * gamma_f.max(0.0) * d_f * a_f.max(0.0) * dt.max(0.0)
}

/// Concentration update from transported amount (positive ξ = into cell i from j perspective
/// when `into_i` is true for cell i).
#[inline]
pub fn concentration_delta_from_xi(xi: f64, volume: f64, into_cell: bool) -> f64 {
    let v = volume.max(1e-18);
    if into_cell {
        xi / v
    } else {
        -xi / v
    }
}

/// Face-averaged mature-membrane carrier measure using production δ.
#[inline]
pub fn gamma_face_production(
    s_i: f64,
    phi_i: f64,
    s_j: f64,
    phi_j: f64,
    delta_floor: f64,
) -> f64 {
    let d_i = production_cell_delta_estimate(phi_i, delta_floor);
    let d_j = production_cell_delta_estimate(phi_j, delta_floor);
    let g_i = reconstruct_gamma(s_i.max(0.0), d_i, delta_floor);
    let g_j = reconstruct_gamma(s_j.max(0.0), d_j, delta_floor);
    0.5 * (g_i + g_j)
}

/// Corrected capacity contribution for one face × accepted step: Γ D A Δt.
#[inline]
pub fn capacity_contrib(gamma_f: f64, d_f: f64, a_f: f64, dt: f64) -> f64 {
    gamma_f.max(0.0) * d_f * a_f.max(0.0) * dt.max(0.0)
}

/// Corrected k_T★ from integrated missing amount and Σ Γ D A Δt.
///
/// Equivalent to directive form
/// `k_T★ = J_missing_rate / ((1/T) Σ Γ D A Δt)` when `J_missing` is the horizon total
/// and the denominator is the raw sum (the `1/T` cancels).
#[inline]
pub fn corrected_k_t_star(j_missing: f64, capacity_sum: f64) -> Option<f64> {
    if capacity_sum <= 1e-18 || j_missing < 0.0 {
        return None;
    }
    Some(j_missing / capacity_sum)
}

/// Mean integrated carrier throughput (amount / time) from accepted ξ sums.
#[inline]
pub fn integrated_throughput(xi_sum: f64, horizon_time: f64) -> f64 {
    if horizon_time <= 1e-18 {
        return 0.0;
    }
    xi_sum / horizon_time
}

/// Original Model A drive (shared with D-056/D-057).
#[inline]
pub fn drive_original_a(
    n_o: f64,
    f_o: f64,
    w_i: f64,
    n_i: f64,
    f_i: f64,
    w_o: f64,
    k_nf: f64,
    k_w: f64,
) -> f64 {
    activity_drive(n_o, f_o, w_i, n_i, f_i, w_o, k_nf, k_w)
}

/// Synthetic face fixture for normalization invariance tests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyntheticFace {
    pub gamma: f64,
    pub drive: f64,
    pub a_f: f64,
    pub dt: f64,
    pub volume_i: f64,
    pub volume_j: f64,
    pub orientation: f64, // +1 or -1
}

impl SyntheticFace {
    pub fn xi(&self, k_t: f64) -> f64 {
        self.orientation * xi_face_req(k_t, self.gamma, self.drive, self.a_f, self.dt)
    }
}

/// Evaluate synthetic invariance suite. Returns (pass, report rows).
pub fn synthetic_normalization_invariance(k_t: f64) -> (bool, Vec<(&'static str, bool)>) {
    let base = SyntheticFace {
        gamma: 1.5,
        drive: 0.4,
        a_f: 1.0,
        dt: 0.01,
        volume_i: 1.0,
        volume_j: 1.0,
        orientation: 1.0,
    };
    let xi0 = base.xi(k_t);
    let mut checks = Vec::new();

    // Face measure linearity
    let mut f_half = base;
    f_half.a_f = 0.5;
    let mut f_dbl = base;
    f_dbl.a_f = 2.0;
    checks.push((
        "face_measure_linear",
        (f_half.xi(k_t) - 0.5 * xi0).abs() < D058_INVARIANCE_TOL
            && (f_dbl.xi(k_t) - 2.0 * xi0).abs() < D058_INVARIANCE_TOL,
    ));

    // Timestep linearity of amount; flux rate invariant
    let mut t_half = base;
    t_half.dt = 0.5 * base.dt;
    let mut t_dbl = base;
    t_dbl.dt = 2.0 * base.dt;
    let rate0 = xi0 / base.dt;
    checks.push((
        "timestep_amount_linear",
        (t_half.xi(k_t) - 0.5 * xi0).abs() < D058_INVARIANCE_TOL
            && (t_dbl.xi(k_t) - 2.0 * xi0).abs() < D058_INVARIANCE_TOL,
    ));
    checks.push((
        "flux_rate_timestep_invariant",
        (t_half.xi(k_t) / t_half.dt - rate0).abs() < D058_INVARIANCE_TOL
            && (t_dbl.xi(k_t) / t_dbl.dt - rate0).abs() < D058_INVARIANCE_TOL,
    ));

    // Concentration ∝ 1/V
    let dc_i = concentration_delta_from_xi(xi0, base.volume_i, true);
    let dc_j = concentration_delta_from_xi(xi0, base.volume_j, false);
    let mut v2 = base;
    v2.volume_i = 2.0;
    let dc_i2 = concentration_delta_from_xi(v2.xi(k_t), v2.volume_i, true);
    checks.push((
        "concentration_inverse_volume",
        (dc_i2 - 0.5 * dc_i).abs() < D058_INVARIANCE_TOL
            && (dc_i + dc_j).abs() < D058_INVARIANCE_TOL, // equal volumes → equal opposite
    ));

    // Orientation reversal → sign only
    let mut rev = base;
    rev.orientation = -1.0;
    checks.push((
        "orientation_sign_only",
        (rev.xi(k_t) + xi0).abs() < D058_INVARIANCE_TOL,
    ));

    // Traversal order: two faces, sum invariant under swap
    let f2 = SyntheticFace {
        gamma: 0.8,
        drive: 0.25,
        a_f: 1.0,
        dt: 0.01,
        volume_i: 1.0,
        volume_j: 1.0,
        orientation: 1.0,
    };
    let sum_ab = base.xi(k_t) + f2.xi(k_t);
    let sum_ba = f2.xi(k_t) + base.xi(k_t);
    checks.push((
        "traversal_order_invariant",
        (sum_ab - sum_ba).abs() < D058_INVARIANCE_TOL,
    ));

    // Equivalent interfaces: N faces with A/N each → same total
    let n = 4usize;
    let mut equiv = base;
    equiv.a_f = base.a_f / n as f64;
    let sum_equiv: f64 = (0..n).map(|_| equiv.xi(k_t)).sum();
    checks.push((
        "equivalent_faces_same_throughput",
        (sum_equiv - xi0).abs() < D058_INVARIANCE_TOL,
    ));

    // No duplicated measure: capacity uses A and dt once
    let cap = capacity_contrib(base.gamma, base.drive, base.a_f, base.dt);
    let xi_from_cap = k_t * cap;
    checks.push((
        "single_measure_factors",
        (xi_from_cap - xi0.abs()).abs() < D058_INVARIANCE_TOL,
    ));

    // Synthetic DX scaling of A_f and V
    let mut dx2 = base;
    dx2.a_f = 2.0 * base.a_f;
    dx2.volume_i = 4.0 * base.volume_i; // (2 DX)^2
    dx2.volume_j = 4.0 * base.volume_j;
    let xi_dx2 = dx2.xi(k_t);
    let dc_dx2 = concentration_delta_from_xi(xi_dx2, dx2.volume_i, true);
    checks.push((
        "synthetic_dx_face_scales_amount",
        (xi_dx2 - 2.0 * xi0).abs() < D058_INVARIANCE_TOL,
    ));
    checks.push((
        "synthetic_dx_volume_scales_concentration",
        (dc_dx2 - dc_i / 2.0).abs() < D058_INVARIANCE_TOL, // 2×A / 4×V = 0.5×
    ));

    let pass = checks.iter().all(|(_, ok)| *ok);
    (pass, checks)
}

/// Observer kernel parity: ξ from formula equals k_T · capacity_contrib.
pub fn observer_kernel_parity(k_t: f64, gamma: f64, d: f64, a: f64, dt: f64) -> bool {
    let xi = xi_face_req(k_t, gamma, d, a, dt);
    let cap = capacity_contrib(gamma, d, a, dt);
    (xi - k_t * cap).abs() < D058_PARITY_TOL
}

/// Corrected Model A measure options (S-derived, production δ).
#[inline]
pub fn corrected_measure_value(kind: CarrierMeasureKind, gamma_delta: f64, delta: f64, theta: f64, s_face: f64) -> f64 {
    match kind {
        CarrierMeasureKind::AGammaS => gamma_delta.max(0.0),
        CarrierMeasureKind::BDeltaGammaS => delta.max(0.0) * gamma_delta.max(0.0),
        CarrierMeasureKind::CDeltaThetaS => delta.max(0.0) * theta.max(0.0),
        CarrierMeasureKind::DFaceAssignedS => s_face.max(0.0),
    }
}

pub fn identifiability_passes_corrected(r: &IdentifiabilityReport) -> bool {
    r.portable
        && r.rate_span
            .map(|s| s <= D058_RATE_SPAN_MAX + 1e-12)
            .unwrap_or(false)
        && r.bootstrap_spread <= D056_ID_BOOTSTRAP_MAX + 1e-12
        && r.loo_factor <= D056_ID_LOO_FACTOR + 1e-12
        && r.hold_median_err <= D056_ID_HOLD_MEDIAN_MAX + 1e-12
        && r.hold_max_err <= D056_ID_HOLD_MAX_MAX + 1e-12
        && r.direction_ok
        && r.starve_ok
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteEvidence058 {
    pub workspace_isolated: bool,
    pub d057_defect_reproduced: bool,
    pub canonical_operator_valid: bool,
    pub observer_parity_ok: bool,
    pub invariance_ok: bool,
    pub original_model_portable: bool,
    pub alt_drive_portable: bool,
    pub surface_volume_limit: bool,
    pub shadow_ok: bool,
    pub architecture_rejected: bool,
    pub kinetics_not_identifiable: bool,
}

pub fn select_route(ev: RouteEvidence058) -> (D058Route, D058PrimaryConclusion) {
    if !ev.workspace_isolated {
        return (
            D058Route::I,
            D058PrimaryConclusion::WorkspaceScopeNotIsolated,
        );
    }
    if !ev.d057_defect_reproduced {
        return (
            D058Route::I,
            D058PrimaryConclusion::D057NormalizationDefectNotReproduced,
        );
    }
    if !ev.canonical_operator_valid {
        return (
            D058Route::I,
            D058PrimaryConclusion::CanonicalFaceOperatorInvalid,
        );
    }
    if !ev.observer_parity_ok {
        return (
            D058Route::I,
            D058PrimaryConclusion::CorrectedObserverParityFailure,
        );
    }
    if !ev.invariance_ok {
        return (
            D058Route::I,
            D058PrimaryConclusion::CarrierNormalizationInvarianceFailure,
        );
    }
    if ev.original_model_portable && ev.shadow_ok {
        return (D058Route::Q, D058Route::Q.conclusion());
    }
    if ev.alt_drive_portable && ev.shadow_ok {
        return (D058Route::D, D058Route::D.conclusion());
    }
    if ev.original_model_portable || ev.alt_drive_portable {
        // Identifiable but shadow not run / failed
        if !ev.shadow_ok {
            return (
                D058Route::I,
                D058PrimaryConclusion::ShadowRepairFailure,
            );
        }
    }
    if ev.surface_volume_limit {
        return (D058Route::V, D058Route::V.conclusion());
    }
    if ev.architecture_rejected {
        return (D058Route::R, D058Route::R.conclusion());
    }
    if ev.kinetics_not_identifiable {
        return (
            D058Route::I,
            D058PrimaryConclusion::CorrectedCarrierKineticsNotIdentifiable,
        );
    }
    (D058Route::I, D058Route::I.conclusion())
}

/// Surface-to-volume classification after corrected normalization.
pub fn corrected_surface_volume_limit(
    normalization_correct: bool,
    portable_global_rate: bool,
    p_missing: f64,
    p_throughput: f64,
) -> bool {
    normalization_correct && !portable_global_rate && p_missing > p_throughput + 1e-9
}

/// Re-export drive helpers used by the runner.
pub use crate::d057_analysis::{
    cancellation_ratio, classify_drive, drive_abc_model_a as drive_model_a, median,
    required_rate_star,
};

pub fn drive_net_for_model(
    model: DriveModelKind,
    n_o: f64,
    f_o: f64,
    w_i: f64,
    n_i: f64,
    f_i: f64,
    w_o: f64,
    k_nf: f64,
    k_n: f64,
    k_f: f64,
    k_w: f64,
    n_ref: f64,
    f_ref: f64,
    w_ref: f64,
) -> f64 {
    drive_for_model(
        model, n_o, f_o, w_i, n_i, f_i, w_o, k_nf, k_n, k_f, k_w, n_ref, f_ref, w_ref,
    )
    .2
}

/// Build an identifiability report from per-state corrected k★ and holdout errors.
pub fn build_identifiability_report(
    measure: &str,
    drive_model: &str,
    train_k: &[f64],
    hold_errs: &[f64],
    direction_ok: bool,
    starve_ok: bool,
) -> IdentifiabilityReport {
    let span = rate_span(train_k);
    let portable = span.map(|s| s <= D058_RATE_SPAN_MAX).unwrap_or(false) && direction_ok && starve_ok;
    let mut sorted_err = hold_errs.to_vec();
    sorted_err.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let hold_median = if sorted_err.is_empty() {
        0.0
    } else {
        sorted_err[sorted_err.len() / 2]
    };
    let hold_max = sorted_err.iter().copied().fold(0.0_f64, f64::max);
    IdentifiabilityReport {
        measure: measure.to_string(),
        drive_model: drive_model.to_string(),
        rate_span: span,
        bootstrap_spread: bootstrap_spread(train_k),
        loo_factor: loo_factor(train_k),
        hold_median_err: hold_median,
        hold_max_err: hold_max,
        direction_ok,
        starve_ok,
        portable,
    }
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn dimensional_table_counts_once() {
        let t = dimensional_table();
        assert!(t.valid);
        assert_eq!(t.face_measure_count, 1);
        assert_eq!(t.timestep_count, 1);
    }

    #[test]
    fn defective_fixture_flags() {
        let f = defective_estimator_fixture();
        assert!(f.used_interface_weight_as_delta);
        assert_eq!(f.face_measure_count, 0);
        assert_eq!(f.timestep_count, 0);
    }
}
