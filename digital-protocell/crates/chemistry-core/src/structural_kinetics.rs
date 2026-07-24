//! D-019 structural production/decay kinetics and mechanism comparison primitives.
//!
//! Parent v2 behavior remains `k_d008_structure * A * I(φ)` production and
//! `k_structure_decay * φ` decay. V3 selects one local scaling repair.

use crate::config::{
    EquationVersion, SimParams, StructuralScalingMechanism, DX, GRID_HEIGHT, GRID_WIDTH,
};
use crate::d018_analysis::{
    fit_radius_scaling, g_structure_at, required_k_structure, restoring_crossing_signs,
    StructureBasisPoint, D018_RADII,
};
use crate::d008_analysis::{PrescribedInterior, STAGE_E_INTERFACE_WIDTH};
use crate::fields::interior_weight;
use crate::reactions::{catalyst_activation, interface_weight};
use serde::{Deserialize, Serialize};

/// Structural kinetics schema for `membrane_metabolism_v3_structural_scaling`.
pub const STRUCTURAL_SCHEMA_VERSION_V3: u32 = 1;

/// Floor on interface exposure so deep interior retains nonzero turnover (mechanism B).
/// ponytail: frozen constant (not a free knob); ceiling = make exposure fully local via neighbors.
pub const STRUCTURAL_EXPOSURE_FLOOR: f64 = 0.05;

/// Curvature-maintenance floor for mechanism C comparison (same turnover rationale).
pub const STRUCTURAL_CURVATURE_FLOOR: f64 = 0.05;

/// V3 ships with the selected mechanism baked in (not a runtime controller).
pub const V3_SELECTED_MECHANISM: StructuralScalingMechanism =
    StructuralScalingMechanism::InterfaceLimitedTurnover;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum D019PrimaryConclusion {
    D019StructuralScalingRepairPass,
    D019SelectPhaseVolumeSynthesis,
    D019SelectInterfaceLimitedTurnover,
    D019SelectLocalCurvatureMaintenance,
    D019NoDefensibleStructuralScalingRepair,
    D019ConservationFailure,
    D019StageDRegression,
    D019NoRestoringNullcline,
    D019NumericalFailure,
    D019Fail,
}

pub fn d019_primary_conclusion_tag(c: D019PrimaryConclusion) -> &'static str {
    match c {
        D019PrimaryConclusion::D019StructuralScalingRepairPass => {
            "D019_STRUCTURAL_SCALING_REPAIR_PASS"
        }
        D019PrimaryConclusion::D019SelectPhaseVolumeSynthesis => {
            "D019_SELECT_PHASE_VOLUME_SYNTHESIS"
        }
        D019PrimaryConclusion::D019SelectInterfaceLimitedTurnover => {
            "D019_SELECT_INTERFACE_LIMITED_TURNOVER"
        }
        D019PrimaryConclusion::D019SelectLocalCurvatureMaintenance => {
            "D019_SELECT_LOCAL_CURVATURE_MAINTENANCE"
        }
        D019PrimaryConclusion::D019NoDefensibleStructuralScalingRepair => {
            "D019_NO_DEFENSIBLE_STRUCTURAL_SCALING_REPAIR"
        }
        D019PrimaryConclusion::D019ConservationFailure => "D019_CONSERVATION_FAILURE",
        D019PrimaryConclusion::D019StageDRegression => "D019_STAGE_D_REGRESSION",
        D019PrimaryConclusion::D019NoRestoringNullcline => "D019_NO_RESTORING_NULLCLINE",
        D019PrimaryConclusion::D019NumericalFailure => "D019_NUMERICAL_FAILURE",
        D019PrimaryConclusion::D019Fail => "D019_FAIL",
    }
}

/// Resolve active structural mechanism for rate evaluation.
pub fn active_structural_mechanism(params: &SimParams) -> Option<StructuralScalingMechanism> {
    if let Some(m) = params.d019_mechanism_probe {
        return Some(m);
    }
    match params.equation_version {
        EquationVersion::MembraneMetabolismV3StructuralScaling
        | EquationVersion::MembraneMetabolismV4InterfaceProtected
        | EquationVersion::MembraneMetabolismV5InterfaceAffinity
        | EquationVersion::MembraneMetabolismV6PrecursorAssembly
        | EquationVersion::MembraneMetabolismV7SurfaceDensity | EquationVersion::MembraneMetabolismV8ReversibleSurfaceExchange
                | EquationVersion::MembraneMetabolismV9ActivatedSurfaceAssembly
                | EquationVersion::MembraneMetabolismV10ActivatedIntermediate
                | EquationVersion::MembraneMetabolismV11SurfaceMaturation | EquationVersion::MembraneMetabolismV12MembraneCatalyticAssembly
                | EquationVersion::MembraneMetabolismV13CatalystSaturatingActivation => Some(V3_SELECTED_MECHANISM),
        _ => None,
    }
}

/// Production rate density (before η_φ); excludes `dt`.
#[inline]
pub fn structure_production_rate(phi: f64, activated: f64, catalyst: f64, params: &SimParams) -> f64 {
    let a = activated.max(0.0);
    match active_structural_mechanism(params) {
        Some(StructuralScalingMechanism::PhaseVolumeSynthesis) => {
            let act = catalyst_activation(catalyst, params.k_c_structure);
            params.k_d008_structure * a * act * interior_weight(phi)
        }
        Some(StructuralScalingMechanism::InterfaceLimitedTurnover)
        | Some(StructuralScalingMechanism::LocalCurvatureMaintenance)
        | None => params.k_d008_structure * a * interface_weight(phi),
    }
}

/// Mixed bulk/interface structural loss density (D-084):
/// `k * φ * [η + (1−η) I(φ)]`. η=0 is interface-only; η=1 is bulk.
#[inline]
pub fn mixed_structure_loss_density(phi: f64, k_decay: f64, eta: f64) -> f64 {
    let phi = phi.max(0.0);
    let eta = eta.clamp(0.0, 1.0);
    let i = interface_weight(phi);
    k_decay.max(0.0) * phi * (eta + (1.0 - eta) * i)
}

/// Enable D-084 mixed turnover on `params` with global `(η, k_φ,-)`.
pub fn apply_mixed_turnover_params(params: &mut SimParams, eta: f64, k_decay: f64) {
    params.use_mixed_structure_turnover = true;
    params.structure_turnover_eta = eta.clamp(0.0, 1.0);
    params.k_structure_decay = k_decay.max(0.0);
}

pub fn legacy_exposure_floor() -> f64 {
    STRUCTURAL_EXPOSURE_FLOOR
}

/// Decay rate density; `lap_abs` used only by curvature mechanism (0 otherwise).
#[inline]
pub fn structure_decay_rate(phi: f64, lap_abs: f64, params: &SimParams) -> f64 {
    let phi = phi.max(0.0);
    match active_structural_mechanism(params) {
        Some(StructuralScalingMechanism::InterfaceLimitedTurnover) => {
            if params.use_mixed_structure_turnover {
                mixed_structure_loss_density(phi, params.k_structure_decay, params.structure_turnover_eta)
            } else {
                // Frozen D-019/D-083 legacy: ε + I(φ) floor (not the D-084 convex mix).
                params.k_structure_decay
                    * phi
                    * (STRUCTURAL_EXPOSURE_FLOOR + interface_weight(phi))
            }
        }
        Some(StructuralScalingMechanism::LocalCurvatureMaintenance) => {
            params.k_structure_decay * phi * (STRUCTURAL_CURVATURE_FLOOR + lap_abs.max(0.0))
        }
        Some(StructuralScalingMechanism::PhaseVolumeSynthesis) | None => {
            params.k_structure_decay * phi
        }
    }
}

/// Production basis density B such that rate = k_d008_structure * B (η applied elsewhere).
#[inline]
pub fn structure_production_basis_density(
    phi: f64,
    activated: f64,
    catalyst: f64,
    params: &SimParams,
) -> f64 {
    let a = activated.max(0.0);
    match active_structural_mechanism(params) {
        Some(StructuralScalingMechanism::PhaseVolumeSynthesis) => {
            let act = catalyst_activation(catalyst, params.k_c_structure);
            a * act * interior_weight(phi)
        }
        Some(StructuralScalingMechanism::InterfaceLimitedTurnover)
        | Some(StructuralScalingMechanism::LocalCurvatureMaintenance)
        | None => a * interface_weight(phi),
    }
}

/// True when mechanism uses only local fields (no target radius/mass / observer feedback).
pub fn mechanism_is_local_only(mechanism: StructuralScalingMechanism) -> bool {
    match mechanism {
        StructuralScalingMechanism::PhaseVolumeSynthesis
        | StructuralScalingMechanism::InterfaceLimitedTurnover
        | StructuralScalingMechanism::LocalCurvatureMaintenance => true,
    }
}

/// Reject curvature maintenance if a target radius/mass knob is present (none are).
pub fn mechanism_encodes_forbidden_target(mechanism: StructuralScalingMechanism) -> bool {
    let _ = mechanism;
    false
}

fn circular_phi(x: f64, y: f64, cx: f64, cy: f64, radius: f64) -> f64 {
    let distance = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
    0.5 * (1.0 - ((distance - radius) / STAGE_E_INTERFACE_WIDTH).tanh())
}

/// Discrete |∇²φ| at grid index for curvature comparison (local stencil only).
pub fn local_abs_laplacian(phi: &[f64], width: usize, height: usize, idx: usize) -> f64 {
    let x = idx % width;
    let y = idx / width;
    if x == 0 || y == 0 || x + 1 >= width || y + 1 >= height {
        return 0.0;
    }
    let c = phi[idx];
    let lap = phi[idx - 1] + phi[idx + 1] + phi[idx - width] + phi[idx + width] - 4.0 * c;
    lap.abs()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismComparisonPoint {
    pub radius: f64,
    pub b_structure: f64,
    pub l_structure: f64,
    pub k_required: f64,
    pub g_at_center_k: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismComparisonResult {
    pub mechanism: StructuralScalingMechanism,
    pub points: Vec<MechanismComparisonPoint>,
    pub production_exponent_p: f64,
    pub decay_exponent_q: f64,
    pub required_rate_exponent: f64,
    pub k_center: f64,
    pub g_below: f64,
    pub g_center: f64,
    pub g_above: f64,
    pub restoring_crossing: bool,
    pub full_turnover_at_phi1: bool,
    pub new_parameter_count: u32,
    pub changed_equation_count: u32,
    pub passes_selection_gate: bool,
    pub reject_reason: Option<String>,
}

fn prescribed_basis_for_mechanism(
    mechanism: StructuralScalingMechanism,
    radius: f64,
    interior: &PrescribedInterior,
    k_decay: f64,
) -> (f64, f64) {
    let cx = (GRID_WIDTH as f64) * 0.5;
    let cy = (GRID_HEIGHT as f64) * 0.5;
    let cell_area = DX * DX;
    let mut b = 0.0;
    let mut l = 0.0;
    let mut phi_field = vec![0.0; GRID_WIDTH * GRID_HEIGHT];
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            phi_field[idx] = circular_phi(x as f64 + 0.5, y as f64 + 0.5, cx, cy, radius);
        }
    }
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let phi = phi_field[idx];
            let a = if phi >= 0.5 {
                interior.activated
            } else {
                0.0
            };
            let c = if phi >= 0.5 {
                interior.catalyst
            } else {
                0.0
            };
            let b_cell = match mechanism {
                StructuralScalingMechanism::PhaseVolumeSynthesis => {
                    a * catalyst_activation(c, 0.10) * interior_weight(phi)
                }
                StructuralScalingMechanism::InterfaceLimitedTurnover
                | StructuralScalingMechanism::LocalCurvatureMaintenance => {
                    a * interface_weight(phi)
                }
            };
            let l_cell = match mechanism {
                StructuralScalingMechanism::PhaseVolumeSynthesis => k_decay * phi.max(0.0),
                StructuralScalingMechanism::InterfaceLimitedTurnover => {
                    k_decay * phi.max(0.0) * (STRUCTURAL_EXPOSURE_FLOOR + interface_weight(phi))
                }
                StructuralScalingMechanism::LocalCurvatureMaintenance => {
                    let lap = local_abs_laplacian(&phi_field, GRID_WIDTH, GRID_HEIGHT, idx);
                    k_decay * phi.max(0.0) * (STRUCTURAL_CURVATURE_FLOOR + lap)
                }
            };
            b += b_cell * cell_area;
            l += l_cell * cell_area;
        }
    }
    (b, l)
}

fn turnover_at_saturated_phi(mechanism: StructuralScalingMechanism, k_decay: f64) -> bool {
    let phi = 1.0;
    let rate = match mechanism {
        StructuralScalingMechanism::PhaseVolumeSynthesis => k_decay * phi,
        StructuralScalingMechanism::InterfaceLimitedTurnover => {
            k_decay * phi * (STRUCTURAL_EXPOSURE_FLOOR + interface_weight(phi))
        }
        StructuralScalingMechanism::LocalCurvatureMaintenance => {
            k_decay * phi * (STRUCTURAL_CURVATURE_FLOOR + 0.0)
        }
    };
    rate > 1e-12
}

pub fn compare_mechanism_prescribed(
    mechanism: StructuralScalingMechanism,
    interior: &PrescribedInterior,
    k_decay: f64,
) -> MechanismComparisonResult {
    let mut points = Vec::new();
    for &radius in &D018_RADII {
        let (b, l) = prescribed_basis_for_mechanism(mechanism, radius, interior, k_decay);
        points.push(MechanismComparisonPoint {
            radius,
            b_structure: b,
            l_structure: l,
            k_required: required_k_structure(b, l),
            g_at_center_k: 0.0,
        });
    }
    let basis: Vec<StructureBasisPoint> = points
        .iter()
        .map(|p| StructureBasisPoint {
            radius: p.radius,
            b_structure: p.b_structure,
            l_structure: p.l_structure,
            k_required: p.k_required,
            k_current: 0.0,
            required_over_current: 0.0,
            authorized_min: 0.0,
            authorized_max: 0.0,
            inside_authorized_domain: true,
            sampling_window_steps: 0,
            constraint_fraction_of_total_w: 0.0,
            window_usable: true,
        })
        .collect();
    let fit = fit_radius_scaling(&basis).expect("six usable radii");
    let center = points
        .iter()
        .find(|p| (p.radius - 22.0).abs() < 1e-9)
        .expect("R22");
    let k_center = center.k_required;
    for p in &mut points {
        p.g_at_center_k = g_structure_at(k_center, p.b_structure, p.l_structure);
    }
    let g_below = points
        .iter()
        .find(|p| (p.radius - 18.0).abs() < 1e-9)
        .map(|p| p.g_at_center_k)
        .unwrap_or(0.0);
    let g_center = points
        .iter()
        .find(|p| (p.radius - 22.0).abs() < 1e-9)
        .map(|p| p.g_at_center_k)
        .unwrap_or(0.0);
    let g_above = points
        .iter()
        .find(|p| (p.radius - 26.0).abs() < 1e-9)
        .map(|p| p.g_at_center_k)
        .unwrap_or(0.0);
    let restoring = restoring_crossing_signs(g_below, g_center, g_above);
    let full_turnover = turnover_at_saturated_phi(mechanism, k_decay);
    let (new_parameter_count, changed_equation_count) = match mechanism {
        StructuralScalingMechanism::PhaseVolumeSynthesis => (0, 1),
        StructuralScalingMechanism::InterfaceLimitedTurnover => (1, 1),
        StructuralScalingMechanism::LocalCurvatureMaintenance => (1, 1),
    };
    let mut reject_reason = None;
    if mechanism_encodes_forbidden_target(mechanism) {
        reject_reason = Some("encodes forbidden target".into());
    } else if !full_turnover {
        reject_reason = Some("saturated interior has zero turnover".into());
    } else if !restoring {
        reject_reason = Some("no restoring g_structure crossing at R18/22/26".into());
    }
    let passes = restoring && full_turnover && reject_reason.is_none();
    MechanismComparisonResult {
        mechanism,
        points,
        production_exponent_p: fit.production_exponent_p,
        decay_exponent_q: fit.decay_exponent_q,
        required_rate_exponent: fit.required_rate_exponent,
        k_center,
        g_below,
        g_center,
        g_above,
        restoring_crossing: restoring,
        full_turnover_at_phi1: full_turnover,
        new_parameter_count,
        changed_equation_count,
        passes_selection_gate: passes,
        reject_reason,
    }
}

pub fn compare_all_mechanisms_prescribed(
    interior: &PrescribedInterior,
    k_decay: f64,
) -> Vec<MechanismComparisonResult> {
    [
        StructuralScalingMechanism::PhaseVolumeSynthesis,
        StructuralScalingMechanism::InterfaceLimitedTurnover,
        StructuralScalingMechanism::LocalCurvatureMaintenance,
    ]
    .into_iter()
    .map(|m| compare_mechanism_prescribed(m, interior, k_decay))
    .collect()
}

/// Selection priority: fewest changed equations, strongest local causal, widest restore, lowest params.
pub fn select_mechanism(
    results: &[MechanismComparisonResult],
) -> Result<StructuralScalingMechanism, D019PrimaryConclusion> {
    let mut passers: Vec<&MechanismComparisonResult> = results
        .iter()
        .filter(|r| r.passes_selection_gate)
        .collect();
    if passers.is_empty() {
        return Err(D019PrimaryConclusion::D019NoDefensibleStructuralScalingRepair);
    }
    passers.sort_by(|a, b| {
        a.changed_equation_count
            .cmp(&b.changed_equation_count)
            .then_with(|| {
                let rank = |m: StructuralScalingMechanism| match m {
                    StructuralScalingMechanism::InterfaceLimitedTurnover => 0,
                    StructuralScalingMechanism::PhaseVolumeSynthesis => 1,
                    StructuralScalingMechanism::LocalCurvatureMaintenance => 2,
                };
                rank(a.mechanism).cmp(&rank(b.mechanism))
            })
            .then_with(|| {
                let span = |r: &MechanismComparisonResult| {
                    let gs: Vec<f64> = r.points.iter().map(|p| p.g_at_center_k).collect();
                    let mut lo = None;
                    let mut hi = None;
                    for (i, g) in gs.iter().enumerate() {
                        if *g > 0.0 {
                            lo = Some(i);
                            break;
                        }
                    }
                    for (i, g) in gs.iter().enumerate().rev() {
                        if *g < 0.0 {
                            hi = Some(i);
                            break;
                        }
                    }
                    match (lo, hi) {
                        (Some(a), Some(b)) if b >= a => (b - a) as i32,
                        _ => 0,
                    }
                };
                span(b).cmp(&span(a))
            })
            .then_with(|| a.new_parameter_count.cmp(&b.new_parameter_count))
    });
    Ok(passers[0].mechanism)
}
