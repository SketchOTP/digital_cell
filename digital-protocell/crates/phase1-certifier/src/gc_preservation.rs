//! Versioned preservation predicates for the GeometryConservativeV3 contract.
//!
//! This module does not alter D-087. It only makes the replacement causal
//! starvation qualification explicit and fail closed.

pub const STARVATION_EXTENSION_BOUND: usize = 150_000;
pub const NUMERIC_TOLERANCE: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CausalStarvationEvidence {
    pub post_switch_n_delivery: f64,
    pub organized_material_entry: f64,
    pub organized_material_late: f64,
    pub late_organized_material_max: f64,
    pub observer_viability_loss_step: Option<usize>,
    pub extension_bound: usize,
}

/// Qualification for the new material contract's causal starvation gate.
///
/// There is no concentration threshold here. The only numeric comparison is
/// the preregistered material deterioration relation, plus floating-point
/// tolerance. The observer failure bound is the preregistered extension bound.
pub fn causal_starvation_passes(evidence: CausalStarvationEvidence) -> bool {
    evidence.post_switch_n_delivery.abs() <= NUMERIC_TOLERANCE
        && evidence.organized_material_late < evidence.organized_material_entry - NUMERIC_TOLERANCE
        && evidence.late_organized_material_max
            <= evidence.organized_material_entry + NUMERIC_TOLERANCE
        && evidence
            .observer_viability_loss_step
            .is_some_and(|step| step <= evidence.extension_bound)
        && evidence.extension_bound == STARVATION_EXTENSION_BOUND
}

/// The historical D-087 result is isolated only when Gate 2 is the sole
/// failure and it is the unchanged historical starvation surrogate failure.
pub fn historical_failure_is_isolated(gates: [bool; 8], gate2_failure: Option<&str>) -> bool {
    gates[0]
        && !gates[1..2].contains(&false)
        && !gates[3..].contains(&false)
        && gate2_failure == Some("D087_D086_REPRODUCTION_FAILURE")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passing() -> CausalStarvationEvidence {
        CausalStarvationEvidence {
            post_switch_n_delivery: 0.0,
            organized_material_entry: 100.0,
            organized_material_late: 80.0,
            late_organized_material_max: 99.0,
            observer_viability_loss_step: Some(10_383),
            extension_bound: STARVATION_EXTENSION_BOUND,
        }
    }

    #[test]
    fn causal_gate_accepts_only_the_preregistered_evidence_shape() {
        assert!(causal_starvation_passes(passing()));
    }

    #[test]
    fn causal_gate_rejects_hidden_n_delivery() {
        let mut evidence = passing();
        evidence.post_switch_n_delivery = 1e-9;
        assert!(!causal_starvation_passes(evidence));
    }

    #[test]
    fn causal_gate_rejects_material_recovery() {
        let mut evidence = passing();
        evidence.late_organized_material_max = 101.0;
        assert!(!causal_starvation_passes(evidence));
    }

    #[test]
    fn causal_gate_rejects_missing_observer_failure() {
        let mut evidence = passing();
        evidence.observer_viability_loss_step = None;
        assert!(!causal_starvation_passes(evidence));
    }

    #[test]
    fn historical_gate2_failure_must_be_the_sole_known_failure() {
        assert!(historical_failure_is_isolated(
            [true, true, false, true, true, true, true, true],
            Some("D087_D086_REPRODUCTION_FAILURE")
        ));
        assert!(!historical_failure_is_isolated(
            [true, false, false, true, true, true, true, true],
            Some("D087_D086_REPRODUCTION_FAILURE")
        ));
    }
}
