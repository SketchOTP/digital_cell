//! D-041 structural A-retention basin-accessibility helpers.
//!
//! Does not alter α, β, K, or the schema-3 exchange integrator.
//! Adds only bounded structural-interface attenuation of A transport (ρ_A).

use crate::config::{
    SimParams, MEMBRANE_TRANSPORT_SCHEMA_3_STRUCTURAL_A_RETENTION, TRANSPORT_SCHEMA_VERSION_V3,
};
use crate::d039_analysis::v8_schema3_params;
use crate::membrane_transport::mature_a_permeability;
use serde::{Deserialize, Serialize};

pub const D041_STARTING_COMMIT: &str = "e05564b";
pub const D041_D040_TAG: &str = "D-040-exchange-precursor-decomposition";
pub const D041_AGENT_MEMORY_ID: &str =
    "D-20260719-d041-structural-a-retention-basin-accessibility";
pub const D041_RECORD_EXCHANGE: &str = "VALIDATED_EXCHANGE_LAW_FROZEN";
pub const D041_ARCHITECTURE_PASS: &str = "MEMBRANE_ARCHITECTURE_V8_SCHEMA3_STRUCTURAL_A_BOOTSTRAP";

/// D-040 frozen repair threshold (θ≈0.5 isotherm activity).
pub const D041_REPAIR_P_MIN: f64 = 0.020;
/// Maximum ρ_A screen candidates (historical + five + one bracket).
pub const D041_MAX_RHO_CANDIDATES: usize = 6;
/// Mature-membrane nonredundancy: ρ_A ≥ 4 Π_A,healthy.
pub const D041_NONREDUNDANCY_FACTOR: f64 = 4.0;
pub const D041_HEALTHY_THETA: f64 = 1.0;
pub const D041_NET_S_FLOW_MAX: f64 = 1e-4;
pub const D041_REPLACEMENT_MIN: f64 = 0.10;
pub const D041_S_DRIFT_MAX: f64 = 0.05;
pub const D041_GATE0_HORIZON: u64 = 25_000;
pub const D041_MAX_ACCEPTED: u64 = 200_000;

/// Ordered ρ_A screen (largest = weakest retention first).
pub const D041_RHO_SCREEN: [f64; 5] = [1.00, 0.80, 0.60, 0.40, 0.20];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D041Conclusion {
    BasinAccessibleMembraneMaintenanceQualified,
    D040RouteNotReproduced,
    StructuralRetentionImplementationFailure,
    StructuralARetentionNotSufficient,
    MembraneCausalityLost,
    BasinAccessibilityNotRecovered,
    ContinuousReplacementNotRecovered,
    DamageRepairNotRecovered,
    ResourceDependenceNotEstablished,
    FoundationalRegression,
    StageEMembraneContractFailure,
    AccountingFailure,
    NumericalFailure,
    Fail,
}

impl D041Conclusion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BasinAccessibleMembraneMaintenanceQualified => {
                "D041_BASIN_ACCESSIBLE_MEMBRANE_MAINTENANCE_QUALIFIED"
            }
            Self::D040RouteNotReproduced => "D041_D040_ROUTE_NOT_REPRODUCED",
            Self::StructuralRetentionImplementationFailure => {
                "D041_STRUCTURAL_RETENTION_IMPLEMENTATION_FAILURE"
            }
            Self::StructuralARetentionNotSufficient => "D041_STRUCTURAL_A_RETENTION_NOT_SUFFICIENT",
            Self::MembraneCausalityLost => "D041_MEMBRANE_CAUSALITY_LOST",
            Self::BasinAccessibilityNotRecovered => "D041_BASIN_ACCESSIBILITY_NOT_RECOVERED",
            Self::ContinuousReplacementNotRecovered => "D041_CONTINUOUS_REPLACEMENT_NOT_RECOVERED",
            Self::DamageRepairNotRecovered => "D041_DAMAGE_REPAIR_NOT_RECOVERED",
            Self::ResourceDependenceNotEstablished => "D041_RESOURCE_DEPENDENCE_NOT_ESTABLISHED",
            Self::FoundationalRegression => "D041_FOUNDATIONAL_REGRESSION",
            Self::StageEMembraneContractFailure => "D041_STAGE_E_MEMBRANE_CONTRACT_FAILURE",
            Self::AccountingFailure => "D041_ACCOUNTING_FAILURE",
            Self::NumericalFailure => "D041_NUMERICAL_FAILURE",
            Self::Fail => "D041_FAIL",
        }
    }
}

/// Enable schema-3 structural A retention with the given ρ_A.
pub fn apply_structural_a_retention(params: &mut SimParams, rho_a: f64) {
    params.transport_schema_version = TRANSPORT_SCHEMA_VERSION_V3;
    params.rho_a = rho_a.max(0.0);
}

/// Schema-3 organism params with structural A retention enabled.
pub fn v8_schema3_structural_a_params(rho_a: f64) -> SimParams {
    let mut params = v8_schema3_params();
    apply_structural_a_retention(&mut params, rho_a);
    params
}

#[inline]
pub fn transport_schema_name(params: &SimParams) -> &'static str {
    if params.transport_schema_version == TRANSPORT_SCHEMA_VERSION_V3 {
        MEMBRANE_TRANSPORT_SCHEMA_3_STRUCTURAL_A_RETENTION
    } else {
        "historical"
    }
}

/// Healthy mature-membrane A permeability Π_A,healthy = exp(−β_A θ_healthy).
#[inline]
pub fn pi_a_healthy(beta_a: f64, theta: f64) -> f64 {
    mature_a_permeability(theta, beta_a)
}

/// Nonredundancy: ρ_A ≥ 4 Π_A,healthy.
#[inline]
pub fn mature_membrane_nonredundant(rho_a: f64, beta_a: f64, theta_healthy: f64) -> bool {
    let pi = pi_a_healthy(beta_a, theta_healthy);
    rho_a >= D041_NONREDUNDANCY_FACTOR * pi
}

/// Select largest passing ρ_A (weakest retention). Returns None if none pass.
pub fn select_weakest_passing_rho(passing: &[(f64, bool)]) -> Option<f64> {
    let mut best: Option<f64> = None;
    for &(rho, ok) in passing {
        if ok {
            best = Some(match best {
                Some(b) => b.max(rho),
                None => rho,
            });
        }
    }
    best
}

/// One optional bracket midpoint between weakest pass and strongest fail.
pub fn bracket_intermediate(failing: f64, passing: f64) -> f64 {
    0.5 * (failing + passing)
}

/// Build ≤6 candidates: historical screen order + at most one bracket.
pub fn build_rho_candidates(screen: &[f64], bracket: Option<f64>) -> Vec<f64> {
    let mut out: Vec<f64> = screen.iter().copied().take(5).collect();
    if let Some(b) = bracket {
        if out.len() < D041_MAX_RHO_CANDIDATES && !out.iter().any(|r| (r - b).abs() < 1e-12) {
            out.push(b);
        }
    }
    out.truncate(D041_MAX_RHO_CANDIDATES);
    out
}

/// Gate-2 candidate pass predicate inputs (observer-only classification).
#[derive(Debug, Clone, Copy)]
pub struct RetentionCandidateMetrics {
    pub a_decline_precedes_collapse: bool,
    pub endogenous_p: f64,
    pub s_toward_healthy: bool,
    pub accounting_ok: bool,
    pub numerical_ok: bool,
}

#[inline]
pub fn retention_candidate_passes(m: RetentionCandidateMetrics) -> bool {
    !m.a_decline_precedes_collapse
        && m.endogenous_p >= D041_REPAIR_P_MIN
        && m.s_toward_healthy
        && m.accounting_ok
        && m.numerical_ok
}

#[cfg(test)]
mod inline_tests {
    use super::*;

    #[test]
    fn nonredundancy_holds_for_screen_at_beta_4_6() {
        let beta = 4.6;
        let pi = pi_a_healthy(beta, 1.0);
        assert!(pi < 0.02);
        for &rho in &D041_RHO_SCREEN {
            assert!(mature_membrane_nonredundant(rho, beta, 1.0));
        }
    }

    #[test]
    fn select_largest_passing() {
        let rows = [(1.0, false), (0.8, false), (0.6, true), (0.4, true)];
        assert!((select_weakest_passing_rho(&rows).unwrap() - 0.6).abs() < 1e-15);
    }
}
