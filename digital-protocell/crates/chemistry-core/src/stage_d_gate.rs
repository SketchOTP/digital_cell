//! Stage D scientific gate helpers (pure analysis; no chemistry changes).

/// Ordered restoring-radius crossing: sorted by R, velocities switch from
/// non-negative to non-positive exactly once, with at least one strictly
/// positive lower-R and one strictly negative higher-R median.
pub fn ordered_restoring_crossing(median_by_r: &[(f64, f64)]) -> bool {
    let mut pts = median_by_r.to_vec();
    pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    if pts.len() < 2 {
        return false;
    }
    let mut saw_pos = false;
    let mut saw_neg = false;
    let mut crossed = false;
    for (_, v) in &pts {
        if *v > 0.0 {
            if crossed || saw_neg {
                return false; // + after − ⇒ unordered / random flip
            }
            saw_pos = true;
        } else if *v < 0.0 {
            if !saw_pos {
                return false; // all-negative or starts negative
            }
            saw_neg = true;
            crossed = true;
        }
        // exact zeros are allowed as the crossing neighborhood
    }
    saw_pos && saw_neg && crossed
}

/// At least `require` seeds share the same velocity sign (ignores exact zeros).
pub fn seed_sign_agreement(velocities: &[f64], require: usize) -> bool {
    let pos = velocities.iter().filter(|v| **v > 0.0).count();
    let neg = velocities.iter().filter(|v| **v < 0.0).count();
    pos.max(neg) >= require
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidStabilization {
    CatalystExtinctionStall,
    ResourceExhaustionStall,
    FragmentationStall,
    DishBoundaryStall,
    NumericalStall,
    CollapseStall,
}

impl InvalidStabilization {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CatalystExtinctionStall => "CATALYST_EXTINCTION_STALL",
            Self::ResourceExhaustionStall => "RESOURCE_EXHAUSTION_STALL",
            Self::FragmentationStall => "FRAGMENTATION_STALL",
            Self::DishBoundaryStall => "DISH_BOUNDARY_STALL",
            Self::NumericalStall => "NUMERICAL_STALL",
            Self::CollapseStall => "COLLAPSE_STALL",
        }
    }
}

/// Classify invalid apparent stabilization from coarse Stage D diagnostics.
pub fn invalid_stabilization_flags(
    retention: f64,
    final_catalyst_mass: f64,
    final_radius: f64,
    q_phi: f64,
    radial_velocity: f64,
    classification: &str,
    mean_internal_nutrient: Option<f64>,
    mean_internal_fuel: Option<f64>,
    dish_half_width: f64,
) -> Vec<InvalidStabilization> {
    let mut flags = Vec::new();
    if retention < 0.05 || final_catalyst_mass < 1e-3 {
        flags.push(InvalidStabilization::CatalystExtinctionStall);
    }
    if let (Some(n), Some(f)) = (mean_internal_nutrient, mean_internal_fuel) {
        if n < 1e-4 || f < 1e-4 {
            flags.push(InvalidStabilization::ResourceExhaustionStall);
        }
    }
    if classification.contains("Fragment") {
        flags.push(InvalidStabilization::FragmentationStall);
    }
    if classification.contains("Clip") || classification.contains("Numerical") {
        flags.push(InvalidStabilization::NumericalStall);
    }
    if final_radius >= dish_half_width - 1.0 {
        flags.push(InvalidStabilization::DishBoundaryStall);
    }
    if final_radius <= 1.0 || (q_phi < 0.15 && radial_velocity > -1e-6) {
        flags.push(InvalidStabilization::CollapseStall);
    }
    flags.sort_by_key(|f| f.as_str());
    flags.dedup();
    flags
}

use crate::nullcline::{classify_jacobian, FixedPointClass};

/// Fixed-point stability from Jacobian eigenvalues (2×2). Advances only if max Re(λ) < 0.
pub fn classify_fixed_point_2x2(j00: f64, j01: f64, j10: f64, j11: f64) -> FixedPointClass {
    classify_jacobian(&[[j00, j01], [j10, j11]]).0
}

/// Radius + catalyst balance are both required before claiming a fixed point.
pub fn fixed_point_requires_radius_and_catalyst_balance(
    radius_nullcline_hit: bool,
    catalyst_nullcline_hit: bool,
) -> bool {
    radius_nullcline_hit && catalyst_nullcline_hit
}

/// Select at most one Stage D candidate. `ranked_passing` must be best-first.
pub fn select_at_most_one_candidate<'a>(ranked_passing: &[&'a str]) -> Option<&'a str> {
    ranked_passing.first().copied()
}

/// Progressive Stage E: center, neighbors, contiguous patch, and 4/5 seed agreement.
pub fn refined_basin_may_advance(
    center_pass: bool,
    neighbor_pass: bool,
    contiguous_patch: bool,
    four_of_five_seeds: bool,
) -> bool {
    center_pass && neighbor_pass && contiguous_patch && four_of_five_seeds
}

/// Full acceptance opens only after D/E/noise/controls/puncture gates.
pub fn full_acceptance_may_run(
    stage_d_pass: bool,
    stage_e_pass: bool,
    noise_pass: bool,
    controls_pass: bool,
    puncture_pass: bool,
) -> bool {
    stage_d_pass && stage_e_pass && noise_pass && controls_pass && puncture_pass
}

/// Acceptance runs must start from analytic fresh seed, never calibration snapshots.
pub fn accepts_only_fresh_seed(fresh_seed: bool, snapshot_init: bool) -> bool {
    fresh_seed && !snapshot_init
}
