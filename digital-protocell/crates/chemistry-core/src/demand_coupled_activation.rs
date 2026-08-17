//! Bounded DC-DEV-017 demand-coupled N/F activation.
//!
//! This is an opt-in adapter over the existing N+F -> A reaction. It uses
//! the existing reserve demand signal and does not write A, R, N, or F
//! directly. The frozen reference gain is the DC-DEV-016 sink/source ratio.

use crate::metabolic_reserve::ReserveParams;

pub const VERSION: &str = "dcdev017_demand_coupled_activation_v1";
pub const REFERENCE_A: f64 = 0.303630027599798;
pub const REFERENCE_R: f64 = 0.5551860064286098;
pub const REFERENCE_GAIN: f64 = 8.58379474604017;
const EPS: f64 = 1e-12;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct DemandCoupledActivationParams {
    pub enable: bool,
    pub reference_a: f64,
    pub reference_r: f64,
    pub reference_gain: f64,
}

impl Default for DemandCoupledActivationParams {
    fn default() -> Self {
        Self {
            enable: false,
            reference_a: REFERENCE_A,
            reference_r: REFERENCE_R,
            reference_gain: REFERENCE_GAIN,
        }
    }
}

impl DemandCoupledActivationParams {
    pub fn frozen_reference() -> Self {
        Self {
            enable: true,
            ..Self::default()
        }
    }

    fn frozen_values_match(&self) -> bool {
        (self.reference_a - REFERENCE_A).abs() <= EPS
            && (self.reference_r - REFERENCE_R).abs() <= EPS
            && (self.reference_gain - REFERENCE_GAIN).abs() <= EPS
            && self.reference_a.is_finite()
            && self.reference_r.is_finite()
            && self.reference_gain.is_finite()
            && self.reference_gain >= 1.0
    }
}

/// Existing reserve demand composition: low A and available R.
pub fn demand(a: f64, r: f64, reserve: &ReserveParams) -> f64 {
    let a = a.max(0.0);
    let r = r.max(0.0);
    let low_a = reserve.k_low / (reserve.k_low + a + EPS);
    let r_term = r / (reserve.k_r + r + EPS);
    (low_a * r_term).clamp(0.0, 1.0)
}

/// Return a multiplicative activation gain, failing closed to legacy behavior.
pub fn activation_multiplier(
    a: f64,
    r: f64,
    reserve: &ReserveParams,
    params: &DemandCoupledActivationParams,
) -> f64 {
    if !params.enable || !params.frozen_values_match() {
        return 1.0;
    }
    let reference_demand = demand(params.reference_a, params.reference_r, reserve);
    let current_demand = demand(a, r, reserve);
    if !reference_demand.is_finite() || reference_demand <= EPS || !current_demand.is_finite() {
        return 1.0;
    }
    (1.0 + (params.reference_gain - 1.0) * current_demand / reference_demand)
        .clamp(1.0, params.reference_gain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_exact_legacy_gain() {
        let params = DemandCoupledActivationParams::default();
        let reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, 1.0);
        assert_eq!(
            activation_multiplier(REFERENCE_A, REFERENCE_R, &reserve, &params),
            1.0
        );
    }

    #[test]
    fn frozen_reference_reaches_frozen_gain() {
        let params = DemandCoupledActivationParams::frozen_reference();
        let reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, 1.0);
        let gain = activation_multiplier(REFERENCE_A, REFERENCE_R, &reserve, &params);
        assert!((gain - REFERENCE_GAIN).abs() < 1e-12);
    }

    #[test]
    fn invalid_reference_fails_closed() {
        let mut params = DemandCoupledActivationParams::frozen_reference();
        params.reference_gain = 2.0;
        let reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, 1.0);
        assert_eq!(activation_multiplier(0.0, 1.0, &reserve, &params), 1.0);
    }
}
