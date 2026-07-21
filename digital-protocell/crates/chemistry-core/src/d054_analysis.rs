//! D-054 dynamic resource-geometry / passive-transport upper-bound audit helpers.
//! Diagnostic only: no production biology change.

use serde::{Deserialize, Serialize};

pub const D054_PROJECT_ID: &str = "D-054";
pub const D054_AGENT_MEMORY_ID: &str =
    "D-20260721-d054-dynamic-resource-geometry-passive-upper-bound";
pub const D054_D053_SOURCE_COMMIT: &str = "76c0898e297b0abf04362df3e848e32c9d228b15";
pub const D054_D053_SOURCE_SUBJECT: &str =
    "D-053: Add combined exterior and membrane resource delivery";
pub const D054_D053_RESULT_TAG: &str = "D-053-combined-resource-delivery-fail";
pub const D054_V14_EXPERIMENTAL_FAILED: &str =
    "V14_SCHEMA3_MIXED_RESOURCE_DELIVERY_EXPERIMENTAL_FAILED";
pub const D054_BOUNDED_MIXED_DELIVERY_REPAIR_EXHAUSTED: &str =
    "BOUNDED_MIXED_DELIVERY_REPAIR_EXHAUSTED";
pub const D054_CHI_MIN: f64 = 1.05;
pub const D054_FROZEN_M_EXT: f64 = 4.0;
pub const D054_FROZEN_M_BETA: f64 = 0.5776226504666211;

/// Informal pre-seal primary vs sealed-source primary.
pub const D054_INFORMAL_D053_PRIMARY: &str = "D053_NO_HEALTHY_RESOURCE_REPAIRED_ATTRACTOR";
pub const D054_SEALED_D053_PRIMARY: &str = "D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum D054Conclusion {
    D053FixedCompartmentGateDefect,
    FixedDynamicAssayMismatch,
    ResourceSurfaceVolumeLimit,
    StageANfUpperBandUnsupported,
    EnvironmentalResourceGeometryLimit,
    EnvironmentalResourceConcentrationLimit,
    DynamicProductiveDemandScalingFailure,
    PassiveResourceTransportArchitectureInsufficient,
    ResourceGeometryReviewInconclusive,
    D053ProvenanceRerunDiverged,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D054Conclusion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::D053FixedCompartmentGateDefect => "D054_D053_FIXED_COMPARTMENT_GATE_DEFECT",
            Self::FixedDynamicAssayMismatch => "D054_FIXED_DYNAMIC_ASSAY_MISMATCH",
            Self::ResourceSurfaceVolumeLimit => "D054_RESOURCE_SURFACE_VOLUME_LIMIT",
            Self::StageANfUpperBandUnsupported => "D054_STAGE_A_NF_UPPER_BAND_UNSUPPORTED",
            Self::EnvironmentalResourceGeometryLimit => "D054_ENVIRONMENTAL_RESOURCE_GEOMETRY_LIMIT",
            Self::EnvironmentalResourceConcentrationLimit => {
                "D054_ENVIRONMENTAL_RESOURCE_CONCENTRATION_LIMIT"
            }
            Self::DynamicProductiveDemandScalingFailure => {
                "D054_DYNAMIC_PRODUCTIVE_DEMAND_SCALING_FAILURE"
            }
            Self::PassiveResourceTransportArchitectureInsufficient => {
                "D054_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT"
            }
            Self::ResourceGeometryReviewInconclusive => "D054_RESOURCE_GEOMETRY_REVIEW_INCONCLUSIVE",
            Self::D053ProvenanceRerunDiverged => "D054_D053_PROVENANCE_RERUN_DIVERGED",
            Self::AccountingFailure => "D054_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D054_NUMERICAL_FAILURE",
            Self::Fail => "D054_FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum D054Route {
    F,
    Q,
    R,
    B,
    E,
    C,
    D,
    P,
    I,
}

impl D054Route {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::F => "Route_F",
            Self::Q => "Route_Q",
            Self::R => "Route_R",
            Self::B => "Route_B",
            Self::E => "Route_E",
            Self::C => "Route_C",
            Self::D => "Route_D",
            Self::P => "Route_P",
            Self::I => "Route_I",
        }
    }
}

/// Gate −1: sealed D-053 primary must match the informal claimed primary.
pub fn provenance_rerun_diverged(informal_primary: &str, sealed_primary: &str) -> bool {
    informal_primary != sealed_primary
}

/// Fixed-compartment Gate 8 threshold check (stated contract χ≥1.05).
pub fn fixed_compartment_chi_meets_contract(chi_n: f64, chi_f: f64) -> bool {
    chi_n >= D054_CHI_MIN && chi_f >= D054_CHI_MIN
}

/// Gate 8 case table: all radii must meet χ contract when short-horizon relax is disallowed.
pub fn fixed_compartment_gate_defect(
    cases: &[(f64, f64)],
    short_horizon_relaxed: bool,
) -> bool {
    if short_horizon_relaxed {
        // Silent threshold weaken relative to stated χ≥1.05 contract.
        return true;
    }
    cases
        .iter()
        .any(|&(chi_n, chi_f)| !fixed_compartment_chi_meets_contract(chi_n, chi_f))
}

/// Interface-to-area ratio σ = interface_length / interior_area.
pub fn interface_to_area(interface_length: f64, interior_area: f64) -> f64 {
    interface_length / interior_area.max(1e-30)
}

/// Demand density d_A = L_A / interior_area.
pub fn demand_density(l_a: f64, interior_area: f64) -> f64 {
    l_a / interior_area.max(1e-30)
}

/// Flux density j = J / interface_length.
pub fn flux_density(j: f64, interface_length: f64) -> f64 {
    j / interface_length.max(1e-30)
}

/// χ_A = min(J_N, J_F) / L_A.
pub fn chi_a(j_n: f64, j_f: f64, l_a: f64) -> f64 {
    j_n.min(j_f) / l_a.max(1e-30)
}

/// Power-law exponent from two positive observations: y ∝ R^p.
pub fn scaling_exponent(r1: f64, y1: f64, r2: f64, y2: f64) -> Option<f64> {
    if r1 <= 0.0 || r2 <= 0.0 || y1 <= 0.0 || y2 <= 0.0 || (r1 - r2).abs() < 1e-30 {
        return None;
    }
    Some((y2.ln() - y1.ln()) / (r2.ln() - r1.ln()))
}

/// Estimate R_critical where χ_A(R)=1 by log-linear interpolation between two radii.
pub fn critical_radius_from_chi_a(r_lo: f64, chi_lo: f64, r_hi: f64, chi_hi: f64) -> Option<f64> {
    if chi_lo <= 0.0 || chi_hi <= 0.0 || r_lo <= 0.0 || r_hi <= 0.0 {
        return None;
    }
    if (chi_lo - 1.0).abs() < 1e-12 {
        return Some(r_lo);
    }
    if (chi_hi - 1.0).abs() < 1e-12 {
        return Some(r_hi);
    }
    // Require a crossing of 1.
    if (chi_lo - 1.0) * (chi_hi - 1.0) > 0.0 {
        return None;
    }
    let p = scaling_exponent(r_lo, chi_lo, r_hi, chi_hi)?;
    // χ(R) = χ_lo * (R/r_lo)^p = 1 → R = r_lo * (1/χ_lo)^(1/p)
    if p.abs() < 1e-12 {
        return None;
    }
    Some(r_lo * (1.0 / chi_lo).powf(1.0 / p))
}

/// Selectivity ratios.
pub fn selectivity_n_over_c(pi_n: f64, pi_c: f64) -> f64 {
    pi_n / pi_c.max(1e-30)
}

pub fn selectivity_f_over_a(pi_f: f64, pi_a: f64) -> f64 {
    pi_f / pi_a.max(1e-30)
}

/// Route selection from ordered stop conditions (Gate −1 / Gate 0 take priority).
pub fn select_route(
    provenance_diverged: bool,
    fixed_gate_defect: bool,
    assay_mismatch: bool,
    surface_volume_limit: bool,
    stage_a_band_unsupported: bool,
    env_geometry_limit: bool,
    env_concentration_limit: bool,
    demand_scaling_failure: bool,
    passive_architecture_insufficient: bool,
) -> (D054Route, D054Conclusion) {
    if provenance_diverged {
        return (
            D054Route::I,
            D054Conclusion::D053ProvenanceRerunDiverged,
        );
    }
    if fixed_gate_defect {
        return (
            D054Route::F,
            D054Conclusion::D053FixedCompartmentGateDefect,
        );
    }
    if assay_mismatch {
        return (D054Route::Q, D054Conclusion::FixedDynamicAssayMismatch);
    }
    if surface_volume_limit {
        return (D054Route::R, D054Conclusion::ResourceSurfaceVolumeLimit);
    }
    if stage_a_band_unsupported {
        return (D054Route::B, D054Conclusion::StageANfUpperBandUnsupported);
    }
    if env_geometry_limit {
        return (
            D054Route::E,
            D054Conclusion::EnvironmentalResourceGeometryLimit,
        );
    }
    if env_concentration_limit {
        return (
            D054Route::C,
            D054Conclusion::EnvironmentalResourceConcentrationLimit,
        );
    }
    if demand_scaling_failure {
        return (
            D054Route::D,
            D054Conclusion::DynamicProductiveDemandScalingFailure,
        );
    }
    if passive_architecture_insufficient {
        return (
            D054Route::P,
            D054Conclusion::PassiveResourceTransportArchitectureInsufficient,
        );
    }
    (
        D054Route::I,
        D054Conclusion::ResourceGeometryReviewInconclusive,
    )
}

/// Gate 5 admission under sealed source contract (must not silently accept χ-rise alone).
pub fn gate5_candidate_admitted(capacity: bool, a_rise: bool, chi_rise: bool, a_retention: f64) -> bool {
    capacity || a_rise || (chi_rise && a_retention >= 0.5)
}
