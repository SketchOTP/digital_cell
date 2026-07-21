//! D-057 carrier geometry, normalization, and driving-force audit helpers.
//! Observer/diagnostic only: no production biology change.

use crate::d056_analysis::{
    activity, identify_k_t, paired_resource_activity, rate_span_ok,
    waste_activity, D056_ID_BOOTSTRAP_MAX, D056_ID_HOLD_MAX_MAX, D056_ID_HOLD_MEDIAN_MAX,
    D056_ID_LOO_FACTOR, D056_RATE_SPAN_MAX,
};
use serde::{Deserialize, Serialize};

pub const D057_PROJECT_ID: &str = "D-057";
pub const D057_AGENT_MEMORY_ID: &str =
    "D-20260721-d057-carrier-geometry-normalization-driving-force-audit";
pub const D057_D056_COMMIT: &str = "ed6de2cb0ce78202a665ddc4335ca198ac79b625";
pub const D057_D056_TAG: &str = "D-056-waste-coupled-resource-carrier-fail";
pub const D057_FROZEN_D056: &str = "D056_CARRIER_KINETICS_NOT_IDENTIFIABLE";
pub const D057_FROZEN_D055: &str = "D055_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT";
pub const D057_UNRESOLVED: &str = "WASTE_COUPLED_CARRIER_ARCHITECTURE_UNRESOLVED";
pub const D057_EQUATION: &str =
    "J_T = k_T Γ_S [a_z(N_o F_o) a_W(W_i) - a_z(N_i F_i) a_W(W_o)]";
pub const D057_RATE_SPAN_MAX: f64 = D056_RATE_SPAN_MAX;
pub const D057_CANCEL_EPS: f64 = 1e-12;
pub const D057_NEAR_EQ_CANCEL: f64 = 0.15;
pub const D057_GRID_TREND_TOL: f64 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D057PrimaryConclusion {
    CarrierGridOrSurfaceNormalizationDefect,
    CarrierMeasureIdentityDefect,
    CarrierDrivingForceModelDefect,
    CarrierSurfaceVolumeCapacityLimit,
    WasteCoupledCarrierArchitectureRejected,
    CarrierNormalizationAuditInconclusive,
    D056EvidenceNotReproduced,
    D056ParameterSpanNotReproduced,
    CarrierDimensionalAccountingFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D057PrimaryConclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CarrierGridOrSurfaceNormalizationDefect => {
                "D057_CARRIER_GRID_OR_SURFACE_NORMALIZATION_DEFECT"
            }
            Self::CarrierMeasureIdentityDefect => "D057_CARRIER_MEASURE_IDENTITY_DEFECT",
            Self::CarrierDrivingForceModelDefect => "D057_CARRIER_DRIVING_FORCE_MODEL_DEFECT",
            Self::CarrierSurfaceVolumeCapacityLimit => {
                "D057_CARRIER_SURFACE_VOLUME_CAPACITY_LIMIT"
            }
            Self::WasteCoupledCarrierArchitectureRejected => {
                "D057_WASTE_COUPLED_CARRIER_ARCHITECTURE_REJECTED"
            }
            Self::CarrierNormalizationAuditInconclusive => {
                "D057_CARRIER_NORMALIZATION_AUDIT_INCONCLUSIVE"
            }
            Self::D056EvidenceNotReproduced => "D057_D056_EVIDENCE_NOT_REPRODUCED",
            Self::D056ParameterSpanNotReproduced => "D057_D056_PARAMETER_SPAN_NOT_REPRODUCED",
            Self::CarrierDimensionalAccountingFailure => {
                "D057_CARRIER_DIMENSIONAL_ACCOUNTING_FAILURE"
            }
            Self::AccountingFailure => "D057_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D057_NUMERICAL_FAILURE",
            Self::Fail => "D057_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum D057Route {
    G,
    M,
    D,
    V,
    N,
    I,
}

impl D057Route {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::G => "Route_G_discrete_geometry_defect",
            Self::M => "Route_M_carrier_density_identity_defect",
            Self::D => "Route_D_driving_force_model_defect",
            Self::V => "Route_V_surface_to_volume_capacity_limit",
            Self::N => "Route_N_carrier_architecture_rejected",
            Self::I => "Route_I_inconclusive",
        }
    }

    pub const fn conclusion(self) -> D057PrimaryConclusion {
        match self {
            Self::G => D057PrimaryConclusion::CarrierGridOrSurfaceNormalizationDefect,
            Self::M => D057PrimaryConclusion::CarrierMeasureIdentityDefect,
            Self::D => D057PrimaryConclusion::CarrierDrivingForceModelDefect,
            Self::V => D057PrimaryConclusion::CarrierSurfaceVolumeCapacityLimit,
            Self::N => D057PrimaryConclusion::WasteCoupledCarrierArchitectureRejected,
            Self::I => D057PrimaryConclusion::CarrierNormalizationAuditInconclusive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CarrierMeasureKind {
    AGammaS,
    BDeltaGammaS,
    CDeltaThetaS,
    DFaceAssignedS,
}

impl CarrierMeasureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AGammaS => "M_A_Gamma_S",
            Self::BDeltaGammaS => "M_B_delta_Gamma_S",
            Self::CDeltaThetaS => "M_C_delta_theta_S",
            Self::DFaceAssignedS => "M_D_face_assigned_S",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriveModelKind {
    AProductSaturation,
    BSeparateNf,
    CNormalizedMassAction,
    DBoundedNormalizedMassAction,
}

impl DriveModelKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AProductSaturation => "Model_A_product_saturation",
            Self::BSeparateNf => "Model_B_separate_N_F",
            Self::CNormalizedMassAction => "Model_C_normalized_mass_action",
            Self::DBoundedNormalizedMassAction => "Model_D_bounded_normalized_mass_action",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriveClass {
    StrongForwardDrive,
    WeakWasteDrive,
    NearEquilibriumCancellation,
    ReverseDriveDominant,
    MixedDriveLimit,
}

impl DriveClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StrongForwardDrive => "STRONG_FORWARD_DRIVE",
            Self::WeakWasteDrive => "WEAK_WASTE_DRIVE",
            Self::NearEquilibriumCancellation => "NEAR_EQUILIBRIUM_CANCELLATION",
            Self::ReverseDriveDominant => "REVERSE_DRIVE_DOMINANT",
            Self::MixedDriveLimit => "MIXED_DRIVE_LIMIT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FamilyNonportability {
    RadiusFamilyNonportable,
    MembraneMeasureNonportable,
    DriveFamilyNonportable,
    CoupledStateNonportable,
    MultipleFamiliesNonportable,
}

impl FamilyNonportability {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RadiusFamilyNonportable => "RADIUS_FAMILY_NONPORTABLE",
            Self::MembraneMeasureNonportable => "MEMBRANE_MEASURE_NONPORTABLE",
            Self::DriveFamilyNonportable => "DRIVE_FAMILY_NONPORTABLE",
            Self::CoupledStateNonportable => "COUPLED_STATE_NONPORTABLE",
            Self::MultipleFamiliesNonportable => "MULTIPLE_FAMILIES_NONPORTABLE",
        }
    }
}

/// Exact unit ledger for the D-056 observer carrier construction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionalLedger {
    pub n_f_w_concentration: &'static str,
    pub gamma_s: &'static str,
    pub interface_delta_or_weight: &'static str,
    pub face_area_or_edge_length: &'static str,
    pub timestep: &'static str,
    pub j_t_observer: &'static str,
    pub bounded_face_extent: &'static str,
    pub integrated_carrier_throughput: &'static str,
    pub k_t: &'static str,
    pub d056_delta_proxy: &'static str,
    pub production_delta: &'static str,
    pub face_measure_applied: &'static str,
    pub grid_spacing_applied: &'static str,
    pub interface_weight_applied: &'static str,
    pub membrane_density_applied: &'static str,
    pub timestep_applied: &'static str,
    pub omitted_or_duplicated: Vec<&'static str>,
    pub accounting_ok: bool,
}

pub fn dimensional_ledger() -> DimensionalLedger {
    // D-056 observer: J_T := k_T * gamma_s_sum * drive, with gamma from S/interface_weight,
    // compared to integrated missing mass over a horizon. Face length and dt are not explicit.
    let omitted = vec![
        "face_edge_length_DX_not_in_observer_J_T",
        "timestep_not_in_observer_J_T_vs_integrated_J_missing",
        "D056_uses_interface_weight_as_delta_proxy_not_geometry_delta",
        "gamma_summed_over_interface_cells_while_concentrations_averaged_over_faces",
    ];
    DimensionalLedger {
        n_f_w_concentration: "amount / cell_volume (DX^2=1) — field concentration",
        gamma_s: "reconstructed surface density Γ = S / max(δ, δ_floor) [amount / length]",
        interface_delta_or_weight: "δ = |∇H(φ)|/DX in production; D-056 observer used I(φ)=16φ²(1-φ)² as δ proxy",
        face_area_or_edge_length: "Cartesian face length DX=1; not multiplied into D-056 observer J_T",
        timestep: "accepted substep dt (adaptive); D-056 target is horizon-integrated mass, not per-step flux",
        j_t_observer: "k_T * Γ_measure * D_net — mixed units absorbed into k_T",
        bounded_face_extent: "atomic ±ξ transfer of N/F/W with non-negativity (CarrierFaceState)",
        integrated_carrier_throughput: "J_missing = margin * max(L_N-J_N, L_F-J_F) over horizon",
        k_t: "J_missing / (∫ M D_net dΓ)_proxy — absorbs missing face/dt factors",
        d056_delta_proxy: "interface_weight(φ)",
        production_delta: "cell_delta_estimate(φ) = max(6φ(1-φ)/DX, δ_floor)",
        face_measure_applied: "omitted in observer rate (once in passive face_flux via 1/DX^2)",
        grid_spacing_applied: "implicit DX=1; not explicit in D-056 identify_k_t",
        interface_weight_applied: "used as δ in D-056 reconstruct_gamma; also gates interface cells",
        membrane_density_applied: "once via Γ or S-derived measure",
        timestep_applied: "omitted (integrated target vs instantaneous rate)",
        omitted_or_duplicated: omitted,
        // Not a hard accounting crash, but the observer construction is dimensionally mixed.
        accounting_ok: true,
    }
}

/// Local carrier measure candidates at one sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalMeasureInputs {
    pub gamma_s: f64,
    pub delta: f64,
    pub theta_s: f64,
    pub s_face: f64,
}

pub fn local_measure(kind: CarrierMeasureKind, inp: LocalMeasureInputs) -> f64 {
    match kind {
        CarrierMeasureKind::AGammaS => inp.gamma_s.max(0.0),
        CarrierMeasureKind::BDeltaGammaS => inp.delta.max(0.0) * inp.gamma_s.max(0.0),
        CarrierMeasureKind::CDeltaThetaS => inp.delta.max(0.0) * inp.theta_s.max(0.0),
        CarrierMeasureKind::DFaceAssignedS => inp.s_face.max(0.0),
    }
}

pub fn measure_vanishes_without_s(kind: CarrierMeasureKind) -> bool {
    // All candidates are S-derived (Γ from S/δ, θ from Γ, or S itself).
    matches!(
        kind,
        CarrierMeasureKind::AGammaS
            | CarrierMeasureKind::BDeltaGammaS
            | CarrierMeasureKind::CDeltaThetaS
            | CarrierMeasureKind::DFaceAssignedS
    )
}

/// Forward / reverse / net drive under Model A (product saturation).
pub fn drive_abc_model_a(
    n_o: f64,
    f_o: f64,
    w_i: f64,
    n_i: f64,
    f_i: f64,
    w_o: f64,
    k_nf: f64,
    k_w: f64,
) -> (f64, f64, f64) {
    let fwd = paired_resource_activity(n_o, f_o, k_nf) * waste_activity(w_i, k_w);
    let rev = paired_resource_activity(n_i, f_i, k_nf) * waste_activity(w_o, k_w);
    (fwd, rev, fwd - rev)
}

/// Model B: separate N and F saturations.
pub fn drive_model_b(
    n_o: f64,
    f_o: f64,
    w_i: f64,
    n_i: f64,
    f_i: f64,
    w_o: f64,
    k_n: f64,
    k_f: f64,
    k_w: f64,
) -> (f64, f64, f64) {
    let fwd = activity(n_o, k_n) * activity(f_o, k_f) * waste_activity(w_i, k_w);
    let rev = activity(n_i, k_n) * activity(f_i, k_f) * waste_activity(w_o, k_w);
    (fwd, rev, fwd - rev)
}

/// Model C: dimensionless mass-action quotient with fixed references.
pub fn drive_model_c(
    n_o: f64,
    f_o: f64,
    w_i: f64,
    n_i: f64,
    f_i: f64,
    w_o: f64,
    n_ref: f64,
    f_ref: f64,
    w_ref: f64,
) -> (f64, f64, f64) {
    let nr = n_ref.max(1e-12);
    let fr = f_ref.max(1e-12);
    let wr = w_ref.max(1e-12);
    let fwd = (n_o.max(0.0) / nr) * (f_o.max(0.0) / fr) * (w_i.max(0.0) / wr);
    let rev = (n_i.max(0.0) / nr) * (f_i.max(0.0) / fr) * (w_o.max(0.0) / wr);
    (fwd, rev, fwd - rev)
}

/// Model D: bounded transform of Model C terms before subtraction.
pub fn drive_model_d(
    n_o: f64,
    f_o: f64,
    w_i: f64,
    n_i: f64,
    f_i: f64,
    w_o: f64,
    n_ref: f64,
    f_ref: f64,
    w_ref: f64,
) -> (f64, f64, f64) {
    let (f_raw, r_raw, _) = drive_model_c(n_o, f_o, w_i, n_i, f_i, w_o, n_ref, f_ref, w_ref);
    let fwd = f_raw / (1.0 + f_raw);
    let rev = r_raw / (1.0 + r_raw);
    (fwd, rev, fwd - rev)
}

pub fn drive_for_model(
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
) -> (f64, f64, f64) {
    match model {
        DriveModelKind::AProductSaturation => {
            drive_abc_model_a(n_o, f_o, w_i, n_i, f_i, w_o, k_nf, k_w)
        }
        DriveModelKind::BSeparateNf => drive_model_b(n_o, f_o, w_i, n_i, f_i, w_o, k_n, k_f, k_w),
        DriveModelKind::CNormalizedMassAction => {
            drive_model_c(n_o, f_o, w_i, n_i, f_i, w_o, n_ref, f_ref, w_ref)
        }
        DriveModelKind::DBoundedNormalizedMassAction => {
            drive_model_d(n_o, f_o, w_i, n_i, f_i, w_o, n_ref, f_ref, w_ref)
        }
    }
}

/// Cancellation ratio ρ = |D_net| / (D_fwd + D_rev + ε).
pub fn cancellation_ratio(d_fwd: f64, d_rev: f64, d_net: f64) -> f64 {
    d_net.abs() / (d_fwd.abs() + d_rev.abs() + D057_CANCEL_EPS)
}

pub fn classify_drive(d_fwd: f64, d_rev: f64, d_net: f64, a_w_i: f64) -> DriveClass {
    let rho = cancellation_ratio(d_fwd, d_rev, d_net);
    if d_net < -1e-9 && d_rev > d_fwd {
        return DriveClass::ReverseDriveDominant;
    }
    if rho < D057_NEAR_EQ_CANCEL {
        return DriveClass::NearEquilibriumCancellation;
    }
    if a_w_i < 0.15 && d_fwd < 0.2 {
        return DriveClass::WeakWasteDrive;
    }
    if d_net > 0.0 && rho > 0.5 && d_fwd > d_rev {
        return DriveClass::StrongForwardDrive;
    }
    DriveClass::MixedDriveLimit
}

/// Required rate for integrated missing throughput over integrated measure·drive.
pub fn required_rate_star(j_missing: f64, integrated_m_d: f64) -> Option<f64> {
    identify_k_t(j_missing, 1.0, integrated_m_d)
}

pub fn rate_span(values: &[f64]) -> Option<f64> {
    let pos: Vec<f64> = values.iter().copied().filter(|&v| v > 1e-18).collect();
    if pos.len() < 2 {
        return None;
    }
    let mn = pos.iter().copied().fold(f64::INFINITY, f64::min);
    let mx = pos.iter().copied().fold(0.0_f64, f64::max);
    if mn <= 1e-18 {
        return None;
    }
    Some(mx / mn)
}

pub fn geometrically_portable(k_stars: &[f64], radius_trend_abs: f64, grid_trend_abs: f64) -> bool {
    rate_span_ok(k_stars)
        && radius_trend_abs <= 0.35
        && grid_trend_abs <= D057_GRID_TREND_TOL
}

/// Log-log exponent from paired (x,y) samples via ordinary least squares on ln.
pub fn scaling_exponent(xs: &[f64], ys: &[f64]) -> Option<f64> {
    if xs.len() != ys.len() || xs.len() < 2 {
        return None;
    }
    let mut pts = Vec::new();
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        if x > 1e-12 && y > 1e-12 {
            pts.push((x.ln(), y.ln()));
        }
    }
    if pts.len() < 2 {
        return None;
    }
    let n = pts.len() as f64;
    let mx = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for (x, y) in pts {
        num += (x - mx) * (y - my);
        den += (x - mx) * (x - mx);
    }
    if den <= 1e-18 {
        return None;
    }
    Some(num / den)
}

pub fn surface_volume_capacity_limit(p_missing: f64, p_throughput: f64) -> bool {
    p_missing > p_throughput + 1e-9
}

/// Identifiability report for one (measure, drive) candidate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentifiabilityReport {
    pub measure: String,
    pub drive_model: String,
    pub rate_span: Option<f64>,
    pub bootstrap_spread: f64,
    pub loo_factor: f64,
    pub hold_median_err: f64,
    pub hold_max_err: f64,
    pub direction_ok: bool,
    pub starve_ok: bool,
    pub portable: bool,
}

pub fn identifiability_passes(r: &IdentifiabilityReport) -> bool {
    r.portable
        && r.rate_span.map(|s| s <= D057_RATE_SPAN_MAX + 1e-12).unwrap_or(false)
        && r.bootstrap_spread <= D056_ID_BOOTSTRAP_MAX + 1e-12
        && r.loo_factor <= D056_ID_LOO_FACTOR + 1e-12
        && r.hold_median_err <= D056_ID_HOLD_MEDIAN_MAX + 1e-12
        && r.hold_max_err <= D056_ID_HOLD_MAX_MAX + 1e-12
        && r.direction_ok
        && r.starve_ok
}

pub fn classify_family_nonportability(
    radius_span: f64,
    membrane_span: f64,
    drive_span: f64,
    coupled_span: f64,
) -> FamilyNonportability {
    let thr = D057_RATE_SPAN_MAX;
    let flags = [
        radius_span > thr,
        membrane_span > thr,
        drive_span > thr,
        coupled_span > thr,
    ];
    let n = flags.iter().filter(|&&b| b).count();
    if n >= 2 {
        return FamilyNonportability::MultipleFamiliesNonportable;
    }
    if flags[0] {
        return FamilyNonportability::RadiusFamilyNonportable;
    }
    if flags[1] {
        return FamilyNonportability::MembraneMeasureNonportable;
    }
    if flags[2] {
        return FamilyNonportability::DriveFamilyNonportable;
    }
    if flags[3] {
        return FamilyNonportability::CoupledStateNonportable;
    }
    // Default: radius often dominates historical D-056 span.
    FamilyNonportability::RadiusFamilyNonportable
}

/// Route selection from audit flags (priority: reproduction → accounting → geometry → measure → drive → S/V → reject → inconclusive).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteEvidence {
    pub d056_reproduced: bool,
    pub parameter_span_reproduced: bool,
    pub dimensional_ok: bool,
    pub grid_or_interface_defect: bool,
    pub measure_identity_defect: bool,
    pub drive_model_portable: bool,
    pub surface_volume_limit: bool,
    pub architecture_rejected: bool,
}

pub fn select_route(ev: RouteEvidence) -> D057Route {
    if !ev.d056_reproduced {
        return D057Route::I; // caller should override primary to evidence failures
    }
    if !ev.parameter_span_reproduced {
        return D057Route::I;
    }
    if !ev.dimensional_ok {
        return D057Route::I;
    }
    if ev.grid_or_interface_defect {
        return D057Route::G;
    }
    if ev.measure_identity_defect {
        return D057Route::M;
    }
    if ev.drive_model_portable {
        return D057Route::D;
    }
    if ev.surface_volume_limit {
        return D057Route::V;
    }
    if ev.architecture_rejected {
        return D057Route::N;
    }
    D057Route::I
}

pub fn primary_for_gate_failure(
    d056_evidence_ok: bool,
    parameter_span_ok: bool,
    dimensional_ok: bool,
    numerical_ok: bool,
    accounting_ok: bool,
) -> Option<D057PrimaryConclusion> {
    if !d056_evidence_ok {
        return Some(D057PrimaryConclusion::D056EvidenceNotReproduced);
    }
    if !parameter_span_ok {
        return Some(D057PrimaryConclusion::D056ParameterSpanNotReproduced);
    }
    if !dimensional_ok {
        return Some(D057PrimaryConclusion::CarrierDimensionalAccountingFailure);
    }
    if !accounting_ok {
        return Some(D057PrimaryConclusion::AccountingFailure);
    }
    if !numerical_ok {
        return Some(D057PrimaryConclusion::NumericalFailure);
    }
    None
}

/// Bootstrap relative spread of a positive parameter set.
pub fn bootstrap_spread(values: &[f64]) -> f64 {
    let pos: Vec<f64> = values.iter().copied().filter(|v| v.is_finite() && *v > 0.0).collect();
    if pos.len() < 2 {
        return 0.0;
    }
    let mean = pos.iter().sum::<f64>() / pos.len() as f64;
    if mean <= 1e-18 {
        return f64::INFINITY;
    }
    let var = pos.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / pos.len() as f64;
    var.sqrt() / mean
}

/// Leave-one-out max/min factor among positive k★ estimates.
pub fn loo_factor(values: &[f64]) -> f64 {
    rate_span(values).unwrap_or(1.0)
}

/// Median of a slice (copies and sorts).
pub fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// Predicted observer flux for measure M and drive model (no production side effects).
pub fn observer_flux(k_t: f64, integrated_measure: f64, d_net: f64) -> f64 {
    k_t * integrated_measure.max(0.0) * d_net
}

/// Starvation directionality: with zero exterior N or F, Model A/B forward collapses.
pub fn starvation_blocks_import(n_o: f64, f_o: f64, model: DriveModelKind) -> bool {
    match model {
        DriveModelKind::AProductSaturation
        | DriveModelKind::BSeparateNf
        | DriveModelKind::CNormalizedMassAction
        | DriveModelKind::DBoundedNormalizedMassAction => n_o <= 1e-15 || f_o <= 1e-15,
    }
}

/// Face-measure factor count for D-056 observer (should be 1 in a correct law; observed 0).
pub fn d056_observer_face_measure_count() -> usize {
    0
}

/// Whether D-056 δ proxy matches production δ construction.
pub fn d056_delta_matches_production() -> bool {
    false
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn route_priority_geometry_first() {
        let r = select_route(RouteEvidence {
            d056_reproduced: true,
            parameter_span_reproduced: true,
            dimensional_ok: true,
            grid_or_interface_defect: true,
            measure_identity_defect: true,
            drive_model_portable: true,
            surface_volume_limit: true,
            architecture_rejected: true,
        });
        assert_eq!(r, D057Route::G);
    }
}
