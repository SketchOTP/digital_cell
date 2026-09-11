// DC-FINAL-001-R5: production-V4 neck-generation diagnosis and bounded
// existing-mechanism mechanochemical reproduction qualification.
//
// This is an assay harness. It does not add a division command, target neck,
// cleavage axis, body-size controller, or a new physical parameter.

use chemistry_core::d096_allocation::{
    catalytic_gain, expression_step, expression_step_activated_material_v2,
    expression_step_activated_material_v3, expression_step_activated_material_v4,
    AllocationGenotype, AllocationParams, ExpressionLedger,
};
use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{
    find_local_segment_apposition, segment_apposition_stress_audit, topology_step,
    try_local_fission, try_local_segment_fission, try_local_segment_fission_at_observed_apposition,
    try_local_segment_fission_legacy_tensile_only, FissionParams,
};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{
    mechanics_step_with_reference_lengths, MechParams, MAX_EXTERNAL_FORCE_PER_VERTEX,
};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_self_contact::{mechanics_step_with_local_self_contact, polygon_simple};
use chemistry_core::mesh_topology::{find_local_pinch, local_rebond_range};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use chemistry_core::metabolic_reserve::ReserveParams;
use chemistry_core::planar_ring_topology::{remesh_preserving_simple, PlanarRingTopology};
use regulatory_core::continuity::{derive_local_mapping, ContinuityNetworkV1, TopologyEventV1};
use regulatory_core::contractility::{
    apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact,
    ContractilityParamsV1, FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME,
};
use regulatory_core::material_adapter::observe_continuity_material_frame;
use regulatory_core::{
    advance_local_plasticity_trace, PlasticityParamsV1, PlasticityStateV1,
    FROZEN_ADAPTATION_LOAD_RATE_PER_TIME, FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME,
    FROZEN_STATIC_TRACTION_LIMIT,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::{env, fs, path::PathBuf};

const DIRECTIVE: &str = "DC-FINAL-001-R5-V4-MATURATION-NECK-GENERATION-MECHANOCHEMICAL-REPRODUCTION-EVOLUTION-AND-FINAL-GOAL-CLOSURE-001";
const LEGACY_HORIZON: usize = 12_000;
const DAUGHTER_STEPS: usize = 3_000;
const CLASSIFICATION_TOLERANCE: f64 = 1e-12;

const PERTURBATIONS: [(&str, f64); 10] = [
    ("rotate", 0.3),
    ("vertex", 0.12),
    ("c", 0.08),
    ("a", 0.08),
    ("env", 0.1),
    ("l", 0.1),
    ("rotate", -0.5),
    ("vertex", -0.1),
    ("c", -0.05),
    ("env", -0.08),
];

fn r10r9r1_reserve_enabled() -> bool {
    matches!(
        env::var("DCFINAL001_R10R9R1_RESERVE").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

fn r10r9r2_growth_path_surplus() -> bool {
    matches!(
        env::var("DCFINAL001_R10R9R2_GROWTH_PATH").ok().as_deref(),
        Some("surplus") | Some("SURPLUS")
    )
}

fn r10r9r3_buffered_reserve_enabled() -> bool {
    matches!(
        env::var("DCFINAL001_R10R9R3_RESERVE").ok().as_deref(),
        Some("1") | Some("on") | Some("ON") | Some("true")
    )
}

fn configured_r10_reserve(mesh: &MaterialMesh) -> ReserveParams {
    if r10r9r3_buffered_reserve_enabled() {
        ReserveParams::derived_buffered(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area())
    } else {
        ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum Mode {
    Passive,
    RegulatorMotor,
    RegulatorOffMotorOnZero,
    RegulatorOnMotorOff,
    ZeroA,
    ContrastFallback,
    ContrastMotorOff,
    ContrastZeroA,
    CurvatureNormal,
    CurvatureNormalMotorOff,
    CurvatureNormalZeroA,
    CurvatureNormalTangential,
    ContrastNormal,
    ContrastNormalMotorOff,
    ContrastNormalZeroA,
    ContrastNormalTangential,
    RefractoryCurvatureNormal,
    RefractoryCurvatureNormalMotorOff,
    RefractoryCurvatureNormalAdaptationDisabled,
    RefractoryCurvatureNormalZeroA,
    RefractoryCurvatureNormalTangential,
    RefractoryCurvatureNormalSignedAudit,
    RefractoryCurvatureNormalLegacyControl,
    RefractoryCurvatureNormalSignedStress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum ExpressionPath {
    Off,
    D096V1,
    D096V2ActivatedMaterial,
    D096V3IntensiveGain,
    D096V4CenteredGain,
}

/// R10R4 observer-only gain substitutions. Catalyst synthesis, maintenance,
/// turnover, activated-material cost, and stored catalyst state remain the
/// exact D096-v2 production values. Only the values exposed to the existing
/// reaction/growth gain readers are substituted on a clone-run step and then
/// restored immediately after those readers finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum DiagnosticGainPolicy {
    ProductionAmount,
    CostOnly,
    ProcessingActivationOnly,
    StructuralBuildOnly,
    IntensiveConcentration,
}

impl DiagnosticGainPolicy {
    fn label(self) -> &'static str {
        match self {
            Self::ProductionAmount => "D096_V2_ABSOLUTE_AMOUNT_GAIN",
            Self::CostOnly => "D096_V2_COST_ONLY_GAIN_DISABLED",
            Self::ProcessingActivationOnly => "D096_V2_FUNCTIONS_0_1_ONLY",
            Self::StructuralBuildOnly => "D096_V2_FUNCTION_2_ONLY",
            Self::IntensiveConcentration => "D096_V2_COST_WITH_INTENSIVE_GAIN_COUNTERFACTUAL",
        }
    }
}

fn substitute_diagnostic_gain_inputs(
    mesh: &mut MaterialMesh,
    policy: DiagnosticGainPolicy,
) -> Option<[f64; 4]> {
    let saved = mesh.finite_allocation.map(|state| state.catalysts)?;
    if policy == DiagnosticGainPolicy::ProductionAmount {
        return Some(saved);
    }
    let area = mesh.area();
    let state = mesh.finite_allocation.as_mut()?;
    match policy {
        DiagnosticGainPolicy::ProductionAmount => {}
        DiagnosticGainPolicy::CostOnly => state.catalysts = [0.0; 4],
        DiagnosticGainPolicy::ProcessingActivationOnly => {
            state.catalysts[2] = 0.0;
            state.catalysts[3] = 0.0;
        }
        DiagnosticGainPolicy::StructuralBuildOnly => {
            state.catalysts[0] = 0.0;
            state.catalysts[1] = 0.0;
            state.catalysts[3] = 0.0;
        }
        DiagnosticGainPolicy::IntensiveConcentration => {
            assert!(
                area.is_finite() && area > 0.0,
                "valid mesh must have positive area"
            );
            for catalyst in &mut state.catalysts {
                *catalyst /= area;
            }
        }
    }
    Some(saved)
}

fn restore_diagnostic_gain_inputs(mesh: &mut MaterialMesh, saved: Option<[f64; 4]>) {
    if let (Some(state), Some(catalysts)) = (mesh.finite_allocation.as_mut(), saved) {
        state.catalysts = catalysts;
    }
}

fn apply_integrated_expression(
    mesh: &mut MaterialMesh,
    path: ExpressionPath,
    dt: f64,
) -> Result<ExpressionLedger, ()> {
    let params = AllocationParams::default();
    match path {
        ExpressionPath::Off => Ok(ExpressionLedger::default()),
        ExpressionPath::D096V1 => expression_step(mesh, &params, dt).map_err(|_| ()),
        ExpressionPath::D096V2ActivatedMaterial => {
            expression_step_activated_material_v2(mesh, &params, dt).map_err(|_| ())
        }
        ExpressionPath::D096V3IntensiveGain => {
            expression_step_activated_material_v3(mesh, &params, dt).map_err(|_| ())
        }
        ExpressionPath::D096V4CenteredGain => {
            expression_step_activated_material_v4(mesh, &params, dt).map_err(|_| ())
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct IntegratedExpressionBudget {
    initial_structural_mass: f64,
    expression_structural_draw: f64,
    expression_catalyst_precursor_a: f64,
    expression_activation_and_maintenance_a: f64,
    expression_turnover_waste: f64,
    m1_structural_build: f64,
    m1_structural_turnover: f64,
    surplus_growth_structural_production: f64,
    growth_a_consumed: f64,
    reserve_a_to_r: f64,
    reserve_r_to_a: f64,
    reserve_r_to_w: f64,
    reserve_r_to_m: f64,
    active_motor_a: f64,
    active_motor_w: f64,
    remesh_topology_structural_net: f64,
    terminal_structural_mass: f64,
    terminal_catalyst_mass: f64,
    expression_failures: usize,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Self::Passive => "PASSIVE_V4",
            Self::RegulatorMotor => "REGULATOR_ON_MOTOR_ON",
            Self::RegulatorOffMotorOnZero => "REGULATOR_OFF_MOTOR_ON_ZERO_ACTIVITY",
            Self::RegulatorOnMotorOff => "REGULATOR_ON_MOTOR_OFF",
            Self::ZeroA => "ZERO_A",
            Self::ContrastFallback => "ZERO_PARAMETER_MEAN_RELATIVE_STRAIN",
            Self::ContrastMotorOff => "ZERO_PARAMETER_CONTRAST_MOTOR_OFF",
            Self::ContrastZeroA => "ZERO_PARAMETER_CONTRAST_ZERO_A",
            Self::CurvatureNormal => "CURVATURE_GATED_NORMAL_ONLY",
            Self::CurvatureNormalMotorOff => "CURVATURE_NORMAL_MOTOR_OFF",
            Self::CurvatureNormalZeroA => "CURVATURE_NORMAL_ZERO_A",
            Self::CurvatureNormalTangential => "CURVATURE_NORMAL_PLUS_TANGENTIAL",
            Self::ContrastNormal => "STRAIN_CONTRAST_NORMAL_ONLY",
            Self::ContrastNormalMotorOff => "STRAIN_CONTRAST_NORMAL_MOTOR_OFF",
            Self::ContrastNormalZeroA => "STRAIN_CONTRAST_NORMAL_ZERO_A",
            Self::ContrastNormalTangential => "STRAIN_CONTRAST_NORMAL_PLUS_TANGENTIAL",
            Self::RefractoryCurvatureNormal => "REFRACTORY_CURVATURE_NORMAL",
            Self::RefractoryCurvatureNormalMotorOff => "REFRACTORY_CURVATURE_NORMAL_MOTOR_OFF",
            Self::RefractoryCurvatureNormalAdaptationDisabled => {
                "REFRACTORY_CURVATURE_NORMAL_ADAPTATION_DISABLED"
            }
            Self::RefractoryCurvatureNormalZeroA => "REFRACTORY_CURVATURE_NORMAL_ZERO_A",
            Self::RefractoryCurvatureNormalTangential => {
                "REFRACTORY_CURVATURE_NORMAL_PLUS_TANGENTIAL"
            }
            Self::RefractoryCurvatureNormalSignedAudit => {
                "REFRACTORY_CURVATURE_NORMAL_SIGNED_STRESS_OBSERVER"
            }
            Self::RefractoryCurvatureNormalLegacyControl => {
                "REFRACTORY_CURVATURE_NORMAL_LEGACY_STRESS_CONTROL"
            }
            Self::RefractoryCurvatureNormalSignedStress => {
                "REFRACTORY_CURVATURE_NORMAL_SIGNED_STRESS"
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct RefractoryAudit {
    enabled: bool,
    accepted_updates: usize,
    remesh_mapping_count: usize,
    remesh_continuity_failures: usize,
    maximum_mapping_distance: f64,
    maximum_adaptation: f64,
    adaptation_variance_sum: f64,
    effective_drive_maximum: f64,
    first_step_effective_drive_error: f64,
}

#[derive(Clone, Debug, Default, Serialize)]
struct FunctionalLocalization {
    steps: usize,
    activity_mean_sum: f64,
    activity_variance_sum: f64,
    active_perimeter_fraction_sum: f64,
    largest_contiguous_active_arc_fraction_sum: f64,
    active_patch_count_sum: usize,
    max_active_patch_count: usize,
    strain_activity_sum_x: f64,
    strain_activity_sum_y: f64,
    strain_activity_sum_x2: f64,
    strain_activity_sum_y2: f64,
    strain_activity_sum_xy: f64,
    strain_activity_n: usize,
    activity_contraction_sum_x: f64,
    activity_contraction_sum_y: f64,
    activity_contraction_sum_x2: f64,
    activity_contraction_sum_y2: f64,
    activity_contraction_sum_xy: f64,
    activity_contraction_n: usize,
    sampled_nearest_activity: Vec<f64>,
    sampled_outside_activity: Vec<f64>,
    sampled_distance_over_range: Vec<f64>,
    best_apposition: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ActuatorGeometryAudit {
    sampled_steps: usize,
    inward_normal_component_sum: f64,
    outward_normal_component_sum: f64,
    absolute_normal_component_sum: f64,
    absolute_tangential_component_sum: f64,
    best_apposition: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct Attribution {
    activity_steps: usize,
    activity_maximum: f64,
    activity_mean_sum: f64,
    activity_variance_sum: f64,
    active_perimeter_fraction_sum: f64,
    strain_activity_sum_x: f64,
    strain_activity_sum_y: f64,
    strain_activity_sum_x2: f64,
    strain_activity_sum_y2: f64,
    strain_activity_sum_xy: f64,
    strain_activity_n: usize,
    activity_neck_x: Vec<f64>,
    activity_neck_y: Vec<f64>,
    requested_a: f64,
    spent_a: f64,
    produced_w: f64,
    zero_a_spent: f64,
    continuity_failures: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
struct FailureCounts {
    no_nonadjacent_apposition: usize,
    apposition_outside_local_range: usize,
    apposition_present_but_stress_condition_false: usize,
    pinch_present_insufficient_a: usize,
    scission_proposed_invalid_geometry: usize,
    scission_proposed_partition_failure: usize,
    valid_fission_daughter_nonviability: usize,
    valid_fission: usize,
}

#[derive(Clone, Debug, Serialize)]
struct RunResult {
    name: String,
    mode: Mode,
    start_step: usize,
    terminal_step: usize,
    birth_mass: f64,
    max_mass_over_birth: f64,
    physical_fission: bool,
    both_daughters_viable: bool,
    full_state_daughters_viable: Option<bool>,
    daughter_diagnostics: Option<Value>,
    fission_step: Option<usize>,
    first_invalid: Option<Value>,
    all_simple: bool,
    all_runtime_valid: bool,
    all_lifecycle_valid: bool,
    failure_counts: FailureCounts,
    deepest_failure: String,
    attempts: Vec<Value>,
    checkpoints: Vec<Value>,
    attribution: Attribution,
    final_geometry: Value,
    #[serde(skip)]
    functional_localization: FunctionalLocalization,
    actuator_geometry: ActuatorGeometryAudit,
    patch_dynamics: Vec<Value>,
    refractory_audit: RefractoryAudit,
    signed_stress_attempts: Vec<Value>,
    signed_stress_counterfactual: Option<Value>,
    #[serde(skip)]
    expression_budget: IntegratedExpressionBudget,
    #[serde(skip)]
    final_mesh: MaterialMesh,
    #[serde(skip)]
    final_plasticity: PlasticityStateV1,
}

fn circular_lag_one_autocorrelation(values: &[f64]) -> Option<f64> {
    if values.len() < 3 {
        return None;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>();
    if variance <= CLASSIFICATION_TOLERANCE {
        return None;
    }
    Some(
        values
            .iter()
            .enumerate()
            .map(|(index, value)| (value - mean) * (values[(index + 1) % values.len()] - mean))
            .sum::<f64>()
            / variance,
    )
}

fn patch_dynamics_row(
    mesh: &MaterialMesh,
    raw_drive: &[f64],
    effective_drive: &[f64],
    adaptation: Option<&[f64]>,
    fission: &FissionParams,
    step: usize,
) -> Value {
    let (mean, variance) = mean_variance(raw_drive);
    let (effective_mean, effective_variance) = mean_variance(effective_drive);
    let dominant_index = raw_drive
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(right.1))
        .map(|(index, _)| index)
        .unwrap_or(0);
    let center = mesh.centroid();
    let dominant_position = mesh.vertices[dominant_index];
    let (active_perimeter_fraction, largest_contiguous_active_arc, active_patches) =
        active_arc_metrics(mesh, raw_drive);
    let effective_dominant_index = effective_drive
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(right.1))
        .map(|(index, _)| index)
        .unwrap_or(0);
    let (effective_active_fraction, effective_largest_arc, effective_active_patches) =
        active_arc_metrics(mesh, effective_drive);
    let (adaptation_mean, adaptation_variance) =
        adaptation.map(mean_variance).unwrap_or((0.0, 0.0));
    let pair = pair_observer(mesh, fission);
    json!({
        "step": step,
        "minimum_distance_over_range": pair["minimum_distance_over_range"],
        "within_range_pair_count": pair["within_range_pairs"],
        "stress_qualified_pair_count": pair["stressed_within_range_pairs"],
        "curvature_drive_maximum": raw_drive.iter().copied().fold(0.0_f64, f64::max),
        "curvature_drive_mean": mean,
        "curvature_drive_variance": variance,
        "largest_contiguous_active_arc_fraction": largest_contiguous_active_arc,
        "active_perimeter_fraction": active_perimeter_fraction,
        "active_patch_count": active_patches,
        "dominant_patch_index": dominant_index,
        "dominant_patch_index_fraction": dominant_index as f64 / mesh.n().max(1) as f64,
        "dominant_patch_position_relative_to_centroid": [
            dominant_position[0] - center[0],
            dominant_position[1] - center[1],
        ],
        "spatial_lag_one_autocorrelation": circular_lag_one_autocorrelation(raw_drive),
        "adaptation_maximum": adaptation
            .map(|values| values.iter().copied().fold(0.0_f64, f64::max))
            .unwrap_or(0.0),
        "adaptation_mean": adaptation_mean,
        "adaptation_variance": adaptation_variance,
        "effective_drive_maximum": effective_drive.iter().copied().fold(0.0_f64, f64::max),
        "effective_drive_mean": effective_mean,
        "effective_drive_variance": effective_variance,
        "effective_dominant_patch_index": effective_dominant_index,
        "effective_dominant_patch_index_fraction": effective_dominant_index as f64 / mesh.n().max(1) as f64,
        "effective_active_perimeter_fraction": effective_active_fraction,
        "effective_largest_contiguous_active_arc_fraction": effective_largest_arc,
        "effective_active_patch_count": effective_active_patches,
        "actual_perimeter": mesh.perimeter(),
        "mature_rest_perimeter": (0..mesh.n()).map(|index| mesh.rest_length(index)).sum::<f64>(),
        "compression_ratio": mesh.perimeter()
            / (0..mesh.n()).map(|index| mesh.rest_length(index)).sum::<f64>().max(1e-300),
        "absolute_a": mesh.interior.a.max(0.0) * mesh.area().max(1e-300),
        "structural_mass": mesh.total_structural_mass(),
        "vertex_count": mesh.n(),
    })
}

fn local_inward_normal_and_tangent(mesh: &MaterialMesh, vertex: usize) -> ([f64; 2], [f64; 2]) {
    let n = mesh.n();
    let prev = mesh.vertices[(vertex + n - 1) % n];
    let here = mesh.vertices[vertex];
    let next = mesh.vertices[(vertex + 1) % n];
    let incoming = [here[0] - prev[0], here[1] - prev[1]];
    let outgoing = [next[0] - here[0], next[1] - here[1]];
    let incoming_length = incoming[0].hypot(incoming[1]).max(1e-15);
    let outgoing_length = outgoing[0].hypot(outgoing[1]).max(1e-15);
    let tangent = [
        incoming[0] / incoming_length + outgoing[0] / outgoing_length,
        incoming[1] / incoming_length + outgoing[1] / outgoing_length,
    ];
    let tangent_length = tangent[0].hypot(tangent[1]).max(1e-15);
    let tangent = [tangent[0] / tangent_length, tangent[1] / tangent_length];
    let orientation = if mesh.signed_area() >= 0.0 { 1.0 } else { -1.0 };
    let inward = [-orientation * tangent[1], orientation * tangent[0]];
    (inward, tangent)
}

fn curvature_components(mesh: &MaterialMesh) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = mesh.n();
    let orientation = if mesh.signed_area() >= 0.0 { 1.0 } else { -1.0 };
    let signed_turn = (0..n)
        .map(|vertex| {
            let prev = mesh.vertices[(vertex + n - 1) % n];
            let here = mesh.vertices[vertex];
            let next = mesh.vertices[(vertex + 1) % n];
            let incoming = [here[0] - prev[0], here[1] - prev[1]];
            let outgoing = [next[0] - here[0], next[1] - here[1]];
            let incoming_length = incoming[0].hypot(incoming[1]).max(1e-15);
            let outgoing_length = outgoing[0].hypot(outgoing[1]).max(1e-15);
            let incoming = [incoming[0] / incoming_length, incoming[1] / incoming_length];
            let outgoing = [outgoing[0] / outgoing_length, outgoing[1] / outgoing_length];
            let cross = incoming[0] * outgoing[1] - incoming[1] * outgoing[0];
            let dot = (incoming[0] * outgoing[0] + incoming[1] * outgoing[1]).clamp(-1.0, 1.0);
            orientation * cross.atan2(dot)
        })
        .collect::<Vec<_>>();
    let concavity = signed_turn
        .iter()
        .map(|turn| (-turn / std::f64::consts::PI).max(0.0).min(1.0))
        .collect::<Vec<_>>();
    let deficit = (0..n)
        .map(|vertex| {
            let neighbor =
                0.5 * (signed_turn[(vertex + n - 1) % n] + signed_turn[(vertex + 1) % n]);
            ((neighbor - signed_turn[vertex]) / std::f64::consts::PI)
                .max(0.0)
                .min(1.0)
        })
        .collect::<Vec<_>>();
    let drive = concavity
        .iter()
        .zip(&deficit)
        .map(|(left, right)| left.max(*right).min(1.0))
        .collect::<Vec<_>>();
    (signed_turn, concavity, deficit, drive)
}

fn inward_normal_request(mesh: &MaterialMesh, drive: &[f64], dt: f64) -> (Vec<[f64; 2]>, f64) {
    let force_budget = (MAX_EXTERNAL_FORCE_PER_VERTEX - FROZEN_STATIC_TRACTION_LIMIT).max(0.0);
    let forces = (0..mesh.n())
        .map(|vertex| {
            let (inward, _) = local_inward_normal_and_tangent(mesh, vertex);
            let magnitude = force_budget * drive[vertex].clamp(0.0, 1.0);
            [magnitude * inward[0], magnitude * inward[1]]
        })
        .collect::<Vec<_>>();
    let requested = forces
        .iter()
        .enumerate()
        .map(|(vertex, force)| {
            let local_length = 0.5
                * (mesh.edge_length(vertex) + mesh.edge_length((vertex + mesh.n() - 1) % mesh.n()));
            FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME * force[0].hypot(force[1]) * local_length * dt
        })
        .sum();
    (forces, requested)
}

fn active_edge_tension_forces(
    mesh: &MaterialMesh,
    activity: &[f64],
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
) -> Vec<[f64; 2]> {
    let mut tensions = vec![0.0; mesh.n()];
    let mut requested = 0.0;
    for edge in 0..mesh.n() {
        if mesh.edges[edge].ruptured {
            continue;
        }
        let edge_activity = 0.5 * (activity[edge] + activity[(edge + 1) % mesh.n()]);
        let tension = contractility.max_active_tension * edge_activity;
        tensions[edge] = tension;
        requested += contractility.reserve_cost_per_force_length_time
            * tension
            * mesh.edge_length(edge)
            * mechanics.dt.max(0.0);
    }
    let available = mesh.interior.a.max(0.0) * mesh.area().max(1e-300);
    let scale = if requested <= f64::EPSILON {
        0.0
    } else {
        (available / requested).min(1.0)
    };
    let mut forces = vec![[0.0, 0.0]; mesh.n()];
    for edge in 0..mesh.n() {
        let tension = tensions[edge] * scale;
        if tension <= 0.0 {
            continue;
        }
        let next = (edge + 1) % mesh.n();
        let delta = [
            mesh.vertices[next][0] - mesh.vertices[edge][0],
            mesh.vertices[next][1] - mesh.vertices[edge][1],
        ];
        let length = delta[0].hypot(delta[1]).max(1e-15);
        let tangent = [delta[0] / length, delta[1] / length];
        forces[edge][0] += tension * tangent[0];
        forces[edge][1] += tension * tangent[1];
        forces[next][0] -= tension * tangent[0];
        forces[next][1] -= tension * tangent[1];
    }
    forces
}

fn segment_pair_distance(mesh: &MaterialMesh, i: usize, j: usize) -> f64 {
    closest_segment_distance(
        mesh.vertices[i],
        mesh.vertices[(i + 1) % mesh.n()],
        mesh.vertices[j],
        mesh.vertices[(j + 1) % mesh.n()],
    )
}

fn actuator_geometry_before(
    mesh: &MaterialMesh,
    activity: &[f64],
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    fission: &FissionParams,
) -> Value {
    let pair = pair_observer(mesh, fission);
    let indices = pair["nearest_pair"].as_array().unwrap();
    let i = indices[0].as_u64().unwrap() as usize;
    let j = indices[1].as_u64().unwrap() as usize;
    let forces = active_edge_tension_forces(mesh, activity, mechanics, contractility);
    let vertices = [i, (i + 1) % mesh.n(), j, (j + 1) % mesh.n()];
    let mut inward = 0.0;
    let mut outward = 0.0;
    let mut absolute_normal = 0.0;
    let mut absolute_tangential = 0.0;
    let rows = vertices
        .iter()
        .map(|vertex| {
            let (normal, tangent) = local_inward_normal_and_tangent(mesh, *vertex);
            let normal_component = forces[*vertex][0] * normal[0] + forces[*vertex][1] * normal[1];
            let tangential_component =
                forces[*vertex][0] * tangent[0] + forces[*vertex][1] * tangent[1];
            inward += normal_component.max(0.0);
            outward += (-normal_component).max(0.0);
            absolute_normal += normal_component.abs();
            absolute_tangential += tangential_component.abs();
            json!({
                "vertex": vertex,
                "active_edge_tension_force": forces[*vertex],
                "inward_normal": normal,
                "local_tangent": tangent,
                "normal_component": normal_component,
                "tangential_component": tangential_component,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "nearest_pair": [i,j],
        "distance_before": segment_pair_distance(mesh,i,j),
        "distance_over_range_before": pair["minimum_distance_over_range"],
        "inward_normal_component": inward,
        "outward_normal_component": outward,
        "absolute_normal_component": absolute_normal,
        "absolute_tangential_component": absolute_tangential,
        "vertices": rows,
    })
}

fn update_actuator_geometry_after(
    audit: &mut ActuatorGeometryAudit,
    mesh: &MaterialMesh,
    mut row: Value,
) {
    let indices = row["nearest_pair"].as_array().unwrap();
    let i = indices[0].as_u64().unwrap() as usize;
    let j = indices[1].as_u64().unwrap() as usize;
    if i >= mesh.n() || j >= mesh.n() {
        return;
    }
    let after = segment_pair_distance(mesh, i, j);
    let before = row["distance_before"].as_f64().unwrap();
    row["distance_after"] = json!(after);
    row["next_step_apposition_change"] = json!(before - after);
    audit.sampled_steps += 1;
    audit.inward_normal_component_sum += row["inward_normal_component"].as_f64().unwrap();
    audit.outward_normal_component_sum += row["outward_normal_component"].as_f64().unwrap();
    audit.absolute_normal_component_sum += row["absolute_normal_component"].as_f64().unwrap();
    audit.absolute_tangential_component_sum +=
        row["absolute_tangential_component"].as_f64().unwrap();
    let replace = audit
        .best_apposition
        .as_ref()
        .and_then(|value| value["distance_over_range_before"].as_f64())
        .map(|best| row["distance_over_range_before"].as_f64().unwrap() < best)
        .unwrap_or(true);
    if replace {
        audit.best_apposition = Some(row);
    }
}

fn perturb(mesh: &mut MaterialMesh, kind: &str, magnitude: f64) {
    match kind {
        "rotate" => {
            let center = mesh.centroid();
            let (sine, cosine) = magnitude.sin_cos();
            for point in &mut mesh.vertices {
                let x = point[0] - center[0];
                let y = point[1] - center[1];
                point[0] = center[0] + cosine * x - sine * y;
                point[1] = center[1] + sine * x + cosine * y;
            }
        }
        "vertex" => {
            for (index, point) in mesh.vertices.iter_mut().enumerate() {
                let fraction = (((index as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
                point[0] += magnitude * (fraction - 0.5);
                point[1] += magnitude * ((fraction * 7.13).fract() - 0.5);
            }
        }
        "c" => mesh.interior.c = (mesh.interior.c * (1.0 + magnitude)).max(0.0),
        "a" => mesh.interior.a = (mesh.interior.a * (1.0 + magnitude)).max(0.0),
        "l" => mesh.free_l = (mesh.free_l * (1.0 + magnitude)).max(0.0),
        "env" => {
            mesh.exterior.n = (mesh.exterior.n * (1.0 + magnitude)).max(0.0);
            mesh.exterior.f = (mesh.exterior.f * (1.0 + magnitude)).max(0.0);
        }
        _ => unreachable!(),
    }
}

fn fixture(index: usize) -> MaterialMesh {
    let (kind, magnitude) = PERTURBATIONS[index];
    let mut mesh =
        chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, (index + 1) as u64, 2.2)
            .individuals
            .remove(0)
            .mesh;
    perturb(&mut mesh, kind, magnitude);
    perturb(&mut mesh, "vertex", 0.35);
    let center = mesh.centroid();
    for point in &mut mesh.vertices {
        point[0] = center[0] + (point[0] - center[0]) * 1.25;
    }
    mesh.stamp_maturation_coupled_schema();
    mesh
}

/// Expose the exact frozen ten-arm production fixtures to the shared R10
/// lifecycle harness.  The fixture construction is unchanged; this is an
/// execution-boundary adapter, not a new biological input.
pub fn r10_reproduction_fixture(index: usize) -> MaterialMesh {
    assert!(index < PERTURBATIONS.len());
    fixture(index)
}

fn geometry(mesh: &MaterialMesh) -> Value {
    json!({
        "simple": polygon_simple(&mesh.vertices),
        "vertices": mesh.n(),
        "area": mesh.area(),
        "perimeter": mesh.perimeter(),
        "mass": mesh.total_structural_mass(),
        "young_mass": mesh.total_young_structural_mass(),
        "mature_mass": mesh.total_structural_mass() - mesh.total_young_structural_mass(),
        "physical_runtime_valid": mesh.physical_runtime_valid(),
        "lifecycle_invariants_hold": mesh.lifecycle_invariants_hold(),
    })
}

fn allocation_fission_continuity(
    parent: &MaterialMesh,
    a: &MaterialMesh,
    b: &MaterialMesh,
) -> Value {
    let amounts = |mesh: &MaterialMesh| {
        mesh.finite_allocation
            .map(|state| state.catalysts)
            .unwrap_or([0.0; 4])
    };
    let parent_amounts = amounts(parent);
    let a_amounts = amounts(a);
    let b_amounts = amounts(b);
    let parent_area = parent.area();
    let a_area = a.area();
    let b_area = b.area();
    let residuals = std::array::from_fn::<_, 4, _>(|index| {
        a_amounts[index] + b_amounts[index] - parent_amounts[index]
    });
    json!({
        "parent_area": parent_area,
        "daughter_a_area": a_area,
        "daughter_b_area": b_area,
        "parent_catalyst_amounts": parent_amounts,
        "daughter_a_catalyst_amounts": a_amounts,
        "daughter_b_catalyst_amounts": b_amounts,
        "catalyst_material_residuals": residuals,
        "parent_concentrations": parent_amounts.map(|amount| amount / parent_area),
        "daughter_a_concentrations": a_amounts.map(|amount| amount / a_area),
        "daughter_b_concentrations": b_amounts.map(|amount| amount / b_area),
        "parent_gains": parent_amounts.map(|amount| catalytic_gain(amount / parent_area)),
        "daughter_a_gains": a_amounts.map(|amount| catalytic_gain(amount / a_area)),
        "daughter_b_gains": b_amounts.map(|amount| catalytic_gain(amount / b_area)),
    })
}

fn mean_variance(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    (mean, variance)
}

fn pearson(sx: f64, sy: f64, sx2: f64, sy2: f64, sxy: f64, n: usize) -> Option<f64> {
    if n < 2 {
        return None;
    }
    let n = n as f64;
    let numerator = n * sxy - sx * sy;
    let denominator = ((n * sx2 - sx * sx) * (n * sy2 - sy * sy)).sqrt();
    (denominator > CLASSIFICATION_TOLERANCE).then_some(numerator / denominator)
}

fn closest_segment_distance(p0: [f64; 2], p1: [f64; 2], q0: [f64; 2], q1: [f64; 2]) -> f64 {
    let u = [p1[0] - p0[0], p1[1] - p0[1]];
    let v = [q1[0] - q0[0], q1[1] - q0[1]];
    let w = [p0[0] - q0[0], p0[1] - q0[1]];
    let a = u[0] * u[0] + u[1] * u[1];
    let b = u[0] * v[0] + u[1] * v[1];
    let c = v[0] * v[0] + v[1] * v[1];
    let d = u[0] * w[0] + u[1] * w[1];
    let e = v[0] * w[0] + v[1] * w[1];
    let den = a * c - b * b;
    let mut s = if den.abs() > f64::EPSILON * (1.0 + a * c) {
        ((b * e - c * d) / den).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = if c > 0.0 { (b * s + e) / c } else { 0.0 };
    if t < 0.0 {
        t = 0.0;
        s = if a > 0.0 {
            (-d / a).clamp(0.0, 1.0)
        } else {
            0.0
        };
    } else if t > 1.0 {
        t = 1.0;
        s = if a > 0.0 {
            ((b - d) / a).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    let p = [p0[0] + s * u[0], p0[1] + s * u[1]];
    let q = [q0[0] + t * v[0], q0[1] + t * v[1]];
    (p[0] - q[0]).hypot(p[1] - q[1])
}

fn pair_observer(mesh: &MaterialMesh, fission: &FissionParams) -> Value {
    let n = mesh.n();
    let min_sep = (n / 4).max(3);
    let range = local_rebond_range(mesh, &fission.topo);
    let mut nearest: Option<(f64, usize, usize, bool)> = None;
    let mut within = 0_usize;
    let mut stressed_within = 0_usize;
    for i in 0..n {
        for j in (i + 1)..n {
            let ring_sep = (j - i).min(n - (j - i));
            if ring_sep < min_sep || j == i + 1 || (i == 0 && j + 1 == n) {
                continue;
            }
            let distance = closest_segment_distance(
                mesh.vertices[i],
                mesh.vertices[(i + 1) % n],
                mesh.vertices[j],
                mesh.vertices[(j + 1) % n],
            );
            let strain_i = mesh.strain(i).max(mesh.strain((i + n - 1) % n));
            let strain_j = mesh.strain(j).max(mesh.strain((j + n - 1) % n));
            let stressed = strain_i > 0.15
                || strain_j > 0.15
                || mesh.edges[i].ruptured
                || mesh.edges[(j + n - 1) % n].ruptured
                || distance < range * 0.55;
            if distance <= range {
                within += 1;
                if stressed {
                    stressed_within += 1;
                }
            }
            if nearest.map(|x| distance < x.0).unwrap_or(true) {
                nearest = Some((distance, i, j, stressed));
            }
        }
    }
    let nearest = nearest.unwrap_or((f64::INFINITY, 0, 0, false));
    json!({
        "range": range,
        "minimum_nonadjacent_segment_distance": nearest.0,
        "minimum_distance_over_range": nearest.0 / range.max(1e-300),
        "nearest_pair": [nearest.1, nearest.2],
        "nearest_pair_stressed": nearest.3,
        "within_range_pairs": within,
        "stressed_within_range_pairs": stressed_within,
        "eligible_apposition_candidate": find_local_segment_apposition(mesh, fission),
        "vertex_pinch": find_local_pinch(mesh, &fission.topo),
    })
}

fn daughter_viability(mut mesh: MaterialMesh) -> Value {
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let fission = FissionParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: false,
    };
    let c_initial = mesh.interior.c;
    let a_initial = mesh.interior.a;
    let initial = json!({
        "area": mesh.area(),
        "structural_mass": mesh.total_structural_mass(),
        "a_concentration": mesh.interior.a,
        "a_amount": mesh.interior.a.max(0.0) * mesh.area().max(1e-300),
        "c_concentration": mesh.interior.c,
        "c_amount": mesh.interior.c.max(0.0) * mesh.area().max(1e-300),
        "observer_viable": mesh.observer_viable(),
        "closed_intact": mesh.closed_intact(),
        "simple": polygon_simple(&mesh.vertices),
        "runtime_valid": mesh.physical_runtime_valid(),
        "lifecycle_valid": mesh.lifecycle_invariants_hold(),
    });
    let mut closing_edge_trace = Vec::new();
    let mut cumulative_ruptures = 0_usize;
    let mut cumulative_rebonds = 0_usize;
    let mut cumulative_rebond_a = 0.0_f64;
    let closing_state = |mesh: &MaterialMesh, step: usize, phase: &str| {
        let indices = mesh
            .edges
            .iter()
            .enumerate()
            .filter(|(_, edge)| edge.b.abs() <= 1e-15 && edge.tracer_b.abs() <= 1e-15)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let mut length = 0.0;
        let mut material = 0.0;
        let mut young = 0.0;
        let mut max_raw_strain = f64::NEG_INFINITY;
        let mut max_raw_stretch_force = 0.0_f64;
        let mut max_load_bearing_stretch_force = 0.0_f64;
        let mut max_load_bearing_strain = f64::NEG_INFINITY;
        let mut ruptured = 0_usize;
        for &index in &indices {
            let edge = mesh.edges[index];
            let edge_length = mesh.edge_length(index);
            let rest = mesh.rest_length(index);
            let l_ref = rest.max(0.25 * edge_length).max(1e-3);
            let stretch = (mechanics.k_s * (edge_length - rest) / l_ref)
                .clamp(-mechanics.k_s * 8.0, mechanics.k_s * 8.0);
            length += edge_length;
            material += edge.m.max(0.0);
            young += mesh.young_structural_mass(index);
            max_raw_strain = max_raw_strain.max(mesh.strain(index));
            max_raw_stretch_force = max_raw_stretch_force.max(stretch.abs());
            max_load_bearing_stretch_force = max_load_bearing_stretch_force
                .max((mesh.mature_structural_fraction(index) * stretch).abs());
            max_load_bearing_strain = max_load_bearing_strain.max(mesh.load_bearing_strain(index));
            ruptured += usize::from(edge.ruptured);
        }
        json!({
            "step":step,
            "phase":phase,
            "candidate_edge_indices":indices,
            "edge_count":indices.len(),
            "edge_length":length,
            "m":material,
            "m_young":young,
            "mature_m":(material-young).max(0.0),
            "m_over_rho_s_length":material/(mesh.rho_s*length).max(1e-300),
            "mature_fraction":(material-young).max(0.0)/material.max(1e-300),
            "rest_length_sum":indices.iter().map(|index|mesh.rest_length(*index)).sum::<f64>(),
            "max_raw_strain":max_raw_strain,
            "max_raw_stretch_force":max_raw_stretch_force,
            "max_load_bearing_stretch_force":max_load_bearing_stretch_force,
            "max_load_bearing_strain":max_load_bearing_strain,
            "ruptured_edge_count":ruptured,
        })
    };
    closing_edge_trace.push(closing_state(&mesh, 0, "birth"));
    let mut completed = 0_usize;
    let mut all_simple = polygon_simple(&mesh.vertices);
    let mut all_runtime = mesh.physical_runtime_valid();
    let mut all_lifecycle = mesh.lifecycle_invariants_hold();
    for step in 0..DAUGHTER_STEPS {
        if !mesh.can_advance_physics() {
            break;
        }
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        if mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_none() {
            break;
        }
        let _ = remesh_preserving_simple(&mut mesh);
        let a_before_topology = mesh.interior.a.max(0.0) * mesh.area().max(1e-300);
        let ledger = topology_step(&mut mesh, &fission);
        let a_after_topology = mesh.interior.a.max(0.0) * mesh.area().max(1e-300);
        cumulative_ruptures += ledger.tension_ruptures;
        cumulative_rebonds += ledger.local_rebonds;
        cumulative_rebond_a += (a_before_topology - a_after_topology).max(0.0);
        if step < 100 {
            let mut row = closing_state(&mesh, step + 1, "post_topology");
            row["step_ruptures"] = json!(ledger.tension_ruptures);
            row["step_rebonds"] = json!(ledger.local_rebonds);
            row["step_rebond_a"] = json!((a_before_topology - a_after_topology).max(0.0));
            closing_edge_trace.push(row);
        }
        all_simple &= polygon_simple(&mesh.vertices);
        all_runtime &= mesh.physical_runtime_valid();
        all_lifecycle &= mesh.lifecycle_invariants_hold();
        if !all_simple || !all_runtime || !all_lifecycle {
            break;
        }
        completed = step + 1;
    }
    let c_retention = if c_initial > 1e-12 {
        mesh.interior.c / c_initial
    } else {
        1.0
    };
    let a_retention = if a_initial > 1e-12 {
        mesh.interior.a / a_initial
    } else {
        1.0
    };
    let viable = completed == DAUGHTER_STEPS
        && mesh.observer_viable()
        && mesh.closed_intact()
        && all_simple
        && all_runtime
        && all_lifecycle
        && c_retention >= 0.80
        && a_retention >= 0.80;
    json!({
        "viable": viable, "completed_steps": completed,
        "observer_viable": mesh.observer_viable(),
        "closed_intact": mesh.closed_intact(),
        "all_simple": all_simple, "all_runtime_valid": all_runtime,
        "all_lifecycle_invariants_hold": all_lifecycle,
        "c_retention": c_retention, "a_retention": a_retention,
        "initial": initial,
        "terminal": geometry(&mesh),
        "terminal_a_concentration": mesh.interior.a,
        "terminal_a_amount": mesh.interior.a.max(0.0) * mesh.area().max(1e-300),
        "terminal_c_concentration": mesh.interior.c,
        "terminal_c_amount": mesh.interior.c.max(0.0) * mesh.area().max(1e-300),
        "closing_edge_trace_first_100_steps": closing_edge_trace,
        "cumulative_topology_ruptures": cumulative_ruptures,
        "cumulative_same_edge_rebonds": cumulative_rebonds,
        "cumulative_a_spent_on_rebond": cumulative_rebond_a,
    })
}

fn partition_plasticity_state(
    parent: &PlasticityStateV1,
    sources: &[usize],
) -> Option<PlasticityStateV1> {
    if sources
        .iter()
        .any(|source| *source >= parent.adaptation.len())
    {
        return None;
    }
    Some(PlasticityStateV1 {
        schema: parent.schema.clone(),
        enabled: parent.enabled,
        adaptation: sources
            .iter()
            .map(|source| parent.adaptation[*source])
            .collect(),
    })
}

fn daughter_viability_with_plasticity_and_expression(
    mut mesh: MaterialMesh,
    mut plasticity: PlasticityStateV1,
    expression_path: ExpressionPath,
) -> Value {
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let fission = FissionParams::default();
    let contractility = ContractilityParamsV1::default();
    let plasticity_params = PlasticityParamsV1::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: false,
    };
    let initial_adaptation = plasticity.adaptation.clone();
    let c_initial = mesh.interior.c;
    let a_initial = mesh.interior.a;
    let mut completed = 0_usize;
    let mut all_simple = polygon_simple(&mesh.vertices);
    let mut all_runtime = mesh.physical_runtime_valid();
    let mut all_lifecycle = mesh.lifecycle_invariants_hold();
    let mut remesh_mappings = 0_usize;
    let mut remesh_continuity_failures = 0_usize;
    let mut spent_a = 0.0_f64;
    let mut produced_w = 0.0_f64;
    for step in 0..DAUGHTER_STEPS {
        if !mesh.can_advance_physics() || plasticity.adaptation.len() != mesh.n() {
            break;
        }
        if expression_path != ExpressionPath::Off
            && apply_integrated_expression(&mut mesh, expression_path, mechanics.dt).is_err()
        {
            break;
        }
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        let raw_drive = curvature_components(&mesh).3;
        let effective_drive = raw_drive
            .iter()
            .zip(&plasticity.adaptation)
            .map(|(drive, adaptation)| drive * (1.0 - adaptation))
            .collect::<Vec<_>>();
        let (forces, requested) = inward_normal_request(&mesh, &effective_drive, mechanics.dt);
        let zeros = vec![[0.0, 0.0]; mesh.n()];
        let Ok(ledger) = apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact(
            &mut mesh,
            &vec![0.0; effective_drive.len()],
            &mechanics,
            &contractility,
            &forces,
            requested,
            &zeros,
        ) else {
            break;
        };
        spent_a += ledger.resource_spent;
        produced_w += ledger.waste_amount_after - ledger.waste_amount_before;
        if advance_local_plasticity_trace(
            &mut plasticity,
            &raw_drive,
            mechanics.dt,
            &plasticity_params,
        )
        .is_err()
        {
            break;
        }
        let old_frame = observe_continuity_material_frame(&mesh, &mechanics);
        let old_n = mesh.n();
        let _ = remesh_preserving_simple(&mut mesh);
        if mesh.n() != old_n {
            let new_frame = observe_continuity_material_frame(&mesh, &mechanics);
            let event = if mesh.n() > old_n {
                TopologyEventV1::Split
            } else {
                TopologyEventV1::Merge
            };
            let mapped = derive_local_mapping(&old_frame, &new_frame, event)
                .ok()
                .and_then(|mapping| {
                    remesh_mappings += 1;
                    plasticity.remap(&mapping).ok()
                });
            if mapped.is_none() {
                remesh_continuity_failures += 1;
                break;
            }
        }
        let _ = topology_step(&mut mesh, &fission);
        all_simple &= polygon_simple(&mesh.vertices);
        all_runtime &= mesh.physical_runtime_valid();
        all_lifecycle &= mesh.lifecycle_invariants_hold();
        if !all_simple || !all_runtime || !all_lifecycle {
            break;
        }
        completed = step + 1;
    }
    let c_retention = if c_initial > 1e-12 {
        mesh.interior.c / c_initial
    } else {
        1.0
    };
    let a_retention = if a_initial > 1e-12 {
        mesh.interior.a / a_initial
    } else {
        1.0
    };
    let energy_residual = (spent_a - produced_w).abs();
    let viable = completed == DAUGHTER_STEPS
        && mesh.observer_viable()
        && mesh.closed_intact()
        && all_simple
        && all_runtime
        && all_lifecycle
        && c_retention >= 0.80
        && a_retention >= 0.80
        && remesh_continuity_failures == 0
        && energy_residual <= 1e-8 * (1.0 + spent_a.abs());
    json!({
        "viable": viable,
        "completed_steps": completed,
        "observer_viable": mesh.observer_viable(),
        "closed_intact": mesh.closed_intact(),
        "all_simple": all_simple,
        "all_runtime_valid": all_runtime,
        "all_lifecycle_invariants_hold": all_lifecycle,
        "c_retention": c_retention,
        "a_retention": a_retention,
        "initial_adaptation": initial_adaptation,
        "terminal_adaptation": plasticity.adaptation,
        "remesh_mappings": remesh_mappings,
        "remesh_continuity_failures": remesh_continuity_failures,
        "active_a_spent": spent_a,
        "active_w_produced": produced_w,
        "active_energy_residual": energy_residual,
        "terminal": geometry(&mesh),
    })
}

fn daughter_viability_with_plasticity(mesh: MaterialMesh, plasticity: PlasticityStateV1) -> Value {
    daughter_viability_with_plasticity_and_expression(mesh, plasticity, ExpressionPath::Off)
}

fn contrast_activity(mesh: &MaterialMesh) -> Vec<f64> {
    let frame = observe_continuity_material_frame(mesh, &MechParams::default());
    let stimuli = frame
        .patches
        .iter()
        .map(|patch| patch.raw_stimulus)
        .collect::<Vec<_>>();
    let measures = (0..mesh.n())
        .map(|i| 0.5 * (mesh.edge_length((i + mesh.n() - 1) % mesh.n()) + mesh.edge_length(i)))
        .collect::<Vec<_>>();
    regulatory_core::adaptive_chemotaxis::adaptive_directional_drive(&stimuli, &measures)
        .expect("positive tensile strain and material measures satisfy adaptive drive contract")
        .front_drive
}

fn active_arc_metrics(mesh: &MaterialMesh, activity: &[f64]) -> (f64, f64, usize) {
    if activity.is_empty() {
        return (0.0, 0.0, 0);
    }
    let measures = (0..mesh.n())
        .map(|i| 0.5 * (mesh.edge_length((i + mesh.n() - 1) % mesh.n()) + mesh.edge_length(i)))
        .collect::<Vec<_>>();
    let perimeter = measures.iter().sum::<f64>().max(1e-300);
    let active = activity
        .iter()
        .map(|value| *value > 0.0)
        .collect::<Vec<_>>();
    let active_fraction = active
        .iter()
        .zip(&measures)
        .filter(|(is_active, _)| **is_active)
        .map(|(_, measure)| *measure)
        .sum::<f64>()
        / perimeter;
    if active.iter().all(|value| !*value) {
        return (active_fraction, 0.0, 0);
    }
    if active.iter().all(|value| *value) {
        return (active_fraction, 1.0, 1);
    }
    let start = active.iter().position(|value| !*value).unwrap();
    let mut largest = 0.0_f64;
    let mut current = 0.0_f64;
    let mut patches = 0_usize;
    let mut in_patch = false;
    for offset in 1..=active.len() {
        let index = (start + offset) % active.len();
        if active[index] {
            if !in_patch {
                patches += 1;
                in_patch = true;
            }
            current += measures[index];
            largest = largest.max(current);
        } else {
            current = 0.0;
            in_patch = false;
        }
    }
    (active_fraction, largest / perimeter, patches)
}

fn nearest_pair_activity(mesh: &MaterialMesh, activity: &[f64], pair: &Value) -> (f64, f64) {
    let indices = pair["nearest_pair"].as_array().unwrap();
    let i = indices[0].as_u64().unwrap() as usize;
    let j = indices[1].as_u64().unwrap() as usize;
    let neighborhood = [i, (i + 1) % mesh.n(), j, (j + 1) % mesh.n()];
    let mut selected = vec![false; mesh.n()];
    for index in neighborhood {
        selected[index] = true;
    }
    let measures = (0..mesh.n())
        .map(|index| {
            0.5 * (mesh.edge_length((index + mesh.n() - 1) % mesh.n()) + mesh.edge_length(index))
        })
        .collect::<Vec<_>>();
    let weighted_mean = |inside: bool| {
        let denominator = (0..mesh.n())
            .filter(|index| selected[*index] == inside)
            .map(|index| measures[index])
            .sum::<f64>();
        if denominator <= 0.0 {
            0.0
        } else {
            (0..mesh.n())
                .filter(|index| selected[*index] == inside)
                .map(|index| activity[index] * measures[index])
                .sum::<f64>()
                / denominator
        }
    };
    (weighted_mean(true), weighted_mean(false))
}

fn update_functional_pre(
    functional: &mut FunctionalLocalization,
    mesh: &MaterialMesh,
    activity: &[f64],
    sampled: bool,
    fission: &FissionParams,
) {
    let frame = observe_continuity_material_frame(mesh, &MechParams::default());
    let strain = frame
        .patches
        .iter()
        .map(|patch| patch.raw_stimulus)
        .collect::<Vec<_>>();
    let (mean, variance) = mean_variance(activity);
    let (active_fraction, largest_arc, patches) = active_arc_metrics(mesh, activity);
    functional.steps += 1;
    functional.activity_mean_sum += mean;
    functional.activity_variance_sum += variance;
    functional.active_perimeter_fraction_sum += active_fraction;
    functional.largest_contiguous_active_arc_fraction_sum += largest_arc;
    functional.active_patch_count_sum += patches;
    functional.max_active_patch_count = functional.max_active_patch_count.max(patches);
    for (x, y) in strain.iter().zip(activity) {
        functional.strain_activity_sum_x += x;
        functional.strain_activity_sum_y += y;
        functional.strain_activity_sum_x2 += x * x;
        functional.strain_activity_sum_y2 += y * y;
        functional.strain_activity_sum_xy += x * y;
        functional.strain_activity_n += 1;
    }
    if sampled {
        let pair = pair_observer(mesh, fission);
        let (nearest, outside) = nearest_pair_activity(mesh, activity, &pair);
        let ratio = pair["minimum_distance_over_range"]
            .as_f64()
            .unwrap_or(f64::INFINITY);
        functional.sampled_nearest_activity.push(nearest);
        functional.sampled_outside_activity.push(outside);
        functional.sampled_distance_over_range.push(ratio);
        let replace = functional
            .best_apposition
            .as_ref()
            .and_then(|value| value["distance_over_range"].as_f64())
            .map(|best| ratio < best)
            .unwrap_or(true);
        if replace {
            functional.best_apposition = Some(json!({
                "distance_over_range": ratio,
                "nearest_pair_activity": nearest,
                "outside_activity": outside,
                "nearest_pair": pair["nearest_pair"],
            }));
        }
    }
}

fn update_functional_contraction(
    functional: &mut FunctionalLocalization,
    activity: &[f64],
    before: &[f64],
    mesh: &MaterialMesh,
) {
    if before.len() != mesh.n() || activity.len() != mesh.n() {
        return;
    }
    for index in 0..mesh.n() {
        let x = 0.5 * (activity[index] + activity[(index + 1) % mesh.n()]);
        let y = (before[index] - mesh.edge_length(index)) / before[index].max(1e-300);
        functional.activity_contraction_sum_x += x;
        functional.activity_contraction_sum_y += y;
        functional.activity_contraction_sum_x2 += x * x;
        functional.activity_contraction_sum_y2 += y * y;
        functional.activity_contraction_sum_xy += x * y;
        functional.activity_contraction_n += 1;
    }
}

fn update_attribution(attribution: &mut Attribution, mesh: &MaterialMesh, activity: &[f64]) {
    let frame = observe_continuity_material_frame(mesh, &MechParams::default());
    let strain = frame
        .patches
        .iter()
        .map(|p| p.raw_stimulus)
        .collect::<Vec<_>>();
    let (mean, variance) = mean_variance(activity);
    attribution.activity_steps += 1;
    attribution.activity_maximum = attribution
        .activity_maximum
        .max(activity.iter().copied().fold(0.0, f64::max));
    attribution.activity_mean_sum += mean;
    attribution.activity_variance_sum += variance;
    let active_length = activity
        .iter()
        .enumerate()
        .filter(|(_, a)| **a > f64::EPSILON)
        .map(|(i, _)| 0.5 * (mesh.edge_length((i + mesh.n() - 1) % mesh.n()) + mesh.edge_length(i)))
        .sum::<f64>();
    attribution.active_perimeter_fraction_sum += active_length / mesh.perimeter().max(1e-300);
    for (x, y) in strain.iter().zip(activity) {
        attribution.strain_activity_sum_x += x;
        attribution.strain_activity_sum_y += y;
        attribution.strain_activity_sum_x2 += x * x;
        attribution.strain_activity_sum_y2 += y * y;
        attribution.strain_activity_sum_xy += x * y;
        attribution.strain_activity_n += 1;
    }
}

fn classify_attempt(mesh: &MaterialMesh, fission: &FissionParams) -> (String, Value) {
    let pairs = pair_observer(mesh, fission);
    let range = pairs["range"].as_f64().unwrap();
    let distance = pairs["minimum_nonadjacent_segment_distance"]
        .as_f64()
        .unwrap();
    let within = pairs["within_range_pairs"].as_u64().unwrap();
    let stressed = pairs["stressed_within_range_pairs"].as_u64().unwrap();
    let apposition = find_local_segment_apposition(mesh, fission);
    let pinch = find_local_pinch(mesh, &fission.topo);
    let candidate_distance = apposition.map(|x| x.4).or_else(|| {
        pinch.map(|(i, j)| {
            let a = mesh.vertices[i];
            let b = mesh.vertices[j];
            (a[0] - b[0]).hypot(a[1] - b[1])
        })
    });
    let closure_a_need = candidate_distance.map(|distance| {
        let one_edge_mass = mesh.rho_s * distance;
        if mesh.is_maturation_coupled() {
            2.0 * one_edge_mass / chemistry_core::mesh_growth::Y_G_CANDIDATES[0]
        } else {
            one_edge_mass
        }
    });
    let have_a = mesh.interior.a.max(0.0) * mesh.area().max(1e-6);
    let reason = if !distance.is_finite() {
        "NO_NONADJACENT_APPOSITION"
    } else if within == 0 || distance > range {
        "APPOSITION_OUTSIDE_LOCAL_RANGE"
    } else if stressed == 0 {
        "APPOSITION_PRESENT_BUT_STRESS_CONDITION_FALSE"
    } else if closure_a_need
        .map(|required| have_a + 1e-12 < required)
        .unwrap_or(false)
    {
        "PINCH_PRESENT_INSUFFICIENT_A"
    } else {
        "SCISSION_CANDIDATE_PRESENT"
    };
    (
        reason.to_string(),
        json!({
            "pairs": pairs, "candidate_distance": candidate_distance,
            "absolute_a": have_a, "cross_bond_a_required": closure_a_need,
            "a_sufficient": closure_a_need.map(|required| have_a + 1e-12 >= required),
        }),
    )
}

fn bump(counts: &mut FailureCounts, reason: &str) {
    match reason {
        "NO_NONADJACENT_APPOSITION" => counts.no_nonadjacent_apposition += 1,
        "APPOSITION_OUTSIDE_LOCAL_RANGE" => counts.apposition_outside_local_range += 1,
        "APPOSITION_PRESENT_BUT_STRESS_CONDITION_FALSE" => {
            counts.apposition_present_but_stress_condition_false += 1
        }
        "PINCH_PRESENT_INSUFFICIENT_A" => counts.pinch_present_insufficient_a += 1,
        "SCISSION_PROPOSED_INVALID_GEOMETRY" => counts.scission_proposed_invalid_geometry += 1,
        "SCISSION_PROPOSED_PARTITION_FAILURE" => counts.scission_proposed_partition_failure += 1,
        "VALID_FISSION_DAUGHTER_NONVIABILITY" => counts.valid_fission_daughter_nonviability += 1,
        "VALID_FISSION" => counts.valid_fission += 1,
        _ => {}
    }
}

fn failure_depth(reason: &str) -> usize {
    match reason {
        "NO_NONADJACENT_APPOSITION" => 0,
        "APPOSITION_OUTSIDE_LOCAL_RANGE" => 1,
        "APPOSITION_PRESENT_BUT_STRESS_CONDITION_FALSE" => 2,
        "PINCH_PRESENT_INSUFFICIENT_A" => 3,
        "SCISSION_PROPOSED_INVALID_GEOMETRY" => 4,
        "SCISSION_PROPOSED_PARTITION_FAILURE" => 5,
        "VALID_FISSION_DAUGHTER_NONVIABILITY" => 6,
        "VALID_FISSION" => 7,
        _ => 0,
    }
}

fn run_with_expression_gain_policy(
    initial_mesh: MaterialMesh,
    name: &str,
    mode: Mode,
    start_step: usize,
    end_step: usize,
    birth_mass: f64,
    expression_path: ExpressionPath,
    gain_policy: DiagnosticGainPolicy,
) -> RunResult {
    let mechanics = MechParams::default();
    let mut reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let contractility = ContractilityParamsV1::default();
    let mut mesh = initial_mesh;
    if r10r9r1_reserve_enabled() {
        reaction.reserve = configured_r10_reserve(&mesh);
    }
    match expression_path {
        ExpressionPath::Off => {}
        ExpressionPath::D096V1 => mesh
            .enable_finite_allocation(AllocationGenotype::neutral(), &AllocationParams::default()),
        ExpressionPath::D096V2ActivatedMaterial => mesh.enable_finite_allocation_v2(
            AllocationGenotype::neutral(),
            &AllocationParams::default(),
        ),
        ExpressionPath::D096V3IntensiveGain => mesh.enable_finite_allocation_v3(
            AllocationGenotype::neutral(),
            &AllocationParams::default(),
        ),
        ExpressionPath::D096V4CenteredGain => mesh.enable_finite_allocation_v4(
            AllocationGenotype::neutral(),
            &AllocationParams::default(),
        ),
    }
    let mut expression_budget = IntegratedExpressionBudget {
        initial_structural_mass: mesh.total_structural_mass(),
        ..Default::default()
    };
    let initial_frame = observe_continuity_material_frame(&mesh, &mechanics);
    let mut regulator = ContinuityNetworkV1::new(initial_frame, Some(0)).unwrap();
    let refractory_mode = matches!(
        mode,
        Mode::RefractoryCurvatureNormal
            | Mode::RefractoryCurvatureNormalMotorOff
            | Mode::RefractoryCurvatureNormalAdaptationDisabled
            | Mode::RefractoryCurvatureNormalZeroA
            | Mode::RefractoryCurvatureNormalTangential
            | Mode::RefractoryCurvatureNormalSignedAudit
            | Mode::RefractoryCurvatureNormalLegacyControl
            | Mode::RefractoryCurvatureNormalSignedStress
    );
    let mut plasticity = if matches!(mode, Mode::RefractoryCurvatureNormalAdaptationDisabled) {
        PlasticityStateV1::disabled(mesh.n())
    } else {
        PlasticityStateV1::new(mesh.n())
    };
    let plasticity_params = PlasticityParamsV1::default();
    let mut maximum_mass_ratio = mesh.total_structural_mass() / birth_mass.max(1e-300);
    let mut all_simple = polygon_simple(&mesh.vertices);
    let mut all_runtime = mesh.physical_runtime_valid();
    let mut all_lifecycle = mesh.lifecycle_invariants_hold();
    let mut first_invalid = None;
    let mut fission_step = None;
    let mut both_viable = false;
    let mut full_state_daughters_viable = None;
    let mut daughter_diagnostics = None;
    let mut attempts = Vec::new();
    let mut checkpoints = Vec::new();
    let mut counts = FailureCounts::default();
    let mut attribution = Attribution::default();
    let mut functional_localization = FunctionalLocalization::default();
    let mut actuator_geometry = ActuatorGeometryAudit::default();
    let mut patch_dynamics = Vec::new();
    let mut signed_stress_attempts = Vec::new();
    let mut signed_stress_counterfactual = None;
    let mut refractory_audit = RefractoryAudit {
        enabled: refractory_mode && plasticity.enabled,
        ..Default::default()
    };
    let mut deepest = "NO_NONADJACENT_APPOSITION".to_string();
    for absolute_step in (start_step + 1)..=end_step {
        if !mesh.can_advance_physics() {
            first_invalid = Some(json!({"step":absolute_step,"phase":"pre_step"}));
            break;
        }
        if expression_path != ExpressionPath::Off {
            match apply_integrated_expression(&mut mesh, expression_path, mechanics.dt) {
                Ok(ExpressionLedger {
                    material_consumed,
                    catalyst_precursor_consumed,
                    activation_consumed,
                    maintenance_consumed,
                    turnover_waste,
                    ..
                }) => {
                    expression_budget.expression_structural_draw += material_consumed;
                    expression_budget.expression_catalyst_precursor_a +=
                        catalyst_precursor_consumed;
                    expression_budget.expression_activation_and_maintenance_a +=
                        activation_consumed + maintenance_consumed;
                    expression_budget.expression_turnover_waste += turnover_waste;
                }
                Err(_) => {
                    expression_budget.expression_failures += 1;
                    first_invalid = Some(json!({"step":absolute_step,"phase":"d096_expression"}));
                    break;
                }
            }
        }
        let saved_catalysts = substitute_diagnostic_gain_inputs(&mut mesh, gain_policy);
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let reaction_ledger = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        expression_budget.reserve_a_to_r += reaction_ledger.reserve.a_to_r;
        expression_budget.reserve_r_to_a += reaction_ledger.reserve.r_to_a;
        expression_budget.reserve_r_to_w += reaction_ledger.reserve.r_to_w;
        expression_budget.reserve_r_to_m += reaction_ledger.reserve.r_to_m;
        expression_budget.m1_structural_build += reaction_ledger.m_produced;
        expression_budget.m1_structural_turnover += reaction_ledger.m_to_w;
        // R10R9R2-only observer counterfactual: retain D091 reserve chemistry
        // and A/R state, but route the diagnostic growth call through the
        // already-qualified reserve-OFF D-088 surplus-A branch. This flag is
        // never enabled by the production entrypoints.
        let reserve_was_enabled = reaction.reserve.enable;
        if reserve_was_enabled && r10r9r2_growth_path_surplus() {
            reaction.reserve.enable = false;
        }
        let growth_ledger = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);
        reaction.reserve.enable = reserve_was_enabled;
        expression_budget.surplus_growth_structural_production += growth_ledger.m_grown;
        expression_budget.growth_a_consumed += growth_ledger.a_consumed_growth;
        expression_budget.reserve_r_to_m += growth_ledger.r_consumed_growth;
        restore_diagnostic_gain_inputs(&mut mesh, saved_catalysts);

        let frame = observe_continuity_material_frame(&mesh, &mechanics);
        let old_n = regulator.previous_frame.topology_size;
        let event = if frame.topology_size == old_n {
            TopologyEventV1::Stable
        } else if frame.topology_size > old_n {
            TopologyEventV1::Split
        } else {
            TopologyEventV1::Merge
        };
        let regulator_on = matches!(
            mode,
            Mode::RegulatorMotor | Mode::RegulatorOnMotorOff | Mode::ZeroA
        );
        let activity = match mode {
            Mode::Passive | Mode::RegulatorOffMotorOnZero => vec![0.0; mesh.n()],
            Mode::ContrastFallback
            | Mode::ContrastMotorOff
            | Mode::ContrastZeroA
            | Mode::ContrastNormal
            | Mode::ContrastNormalMotorOff
            | Mode::ContrastNormalZeroA
            | Mode::ContrastNormalTangential => contrast_activity(&mesh),
            Mode::CurvatureNormal
            | Mode::CurvatureNormalMotorOff
            | Mode::CurvatureNormalZeroA
            | Mode::CurvatureNormalTangential
            | Mode::RefractoryCurvatureNormal
            | Mode::RefractoryCurvatureNormalMotorOff
            | Mode::RefractoryCurvatureNormalAdaptationDisabled
            | Mode::RefractoryCurvatureNormalZeroA
            | Mode::RefractoryCurvatureNormalTangential
            | Mode::RefractoryCurvatureNormalSignedAudit
            | Mode::RefractoryCurvatureNormalLegacyControl
            | Mode::RefractoryCurvatureNormalSignedStress => curvature_components(&mesh).3,
            _ => {
                if regulator.step(frame, event).is_err() {
                    attribution.continuity_failures += 1;
                    first_invalid =
                        Some(json!({"step":absolute_step,"phase":"regulatory_continuity"}));
                    break;
                }
                regulator.state.activity.clone()
            }
        };
        let raw_activity = activity;
        let activity = if refractory_mode && plasticity.enabled {
            raw_activity
                .iter()
                .zip(&plasticity.adaptation)
                .map(|(drive, adaptation)| drive * (1.0 - adaptation))
                .collect::<Vec<_>>()
        } else {
            raw_activity.clone()
        };
        if refractory_mode && absolute_step == start_step + 1 {
            refractory_audit.first_step_effective_drive_error = raw_activity
                .iter()
                .zip(&activity)
                .map(|(raw, effective)| (raw - effective).abs())
                .fold(0.0_f64, f64::max);
        }
        let (activity_mean_current, _) = mean_variance(&activity);
        if (matches!(mode, Mode::CurvatureNormal) || refractory_mode)
            && (absolute_step == start_step + 1
                || absolute_step % 25 == 0
                || absolute_step == end_step)
        {
            patch_dynamics.push(patch_dynamics_row(
                &mesh,
                &raw_activity,
                &activity,
                refractory_mode.then_some(plasticity.adaptation.as_slice()),
                &fission,
                absolute_step,
            ));
        }
        if regulator_on
            || matches!(
                mode,
                Mode::ContrastFallback
                    | Mode::ContrastMotorOff
                    | Mode::ContrastZeroA
                    | Mode::CurvatureNormal
                    | Mode::CurvatureNormalMotorOff
                    | Mode::CurvatureNormalZeroA
                    | Mode::CurvatureNormalTangential
                    | Mode::ContrastNormal
                    | Mode::ContrastNormalMotorOff
                    | Mode::ContrastNormalZeroA
                    | Mode::ContrastNormalTangential
                    | Mode::RefractoryCurvatureNormal
                    | Mode::RefractoryCurvatureNormalMotorOff
                    | Mode::RefractoryCurvatureNormalAdaptationDisabled
                    | Mode::RefractoryCurvatureNormalZeroA
                    | Mode::RefractoryCurvatureNormalTangential
                    | Mode::RefractoryCurvatureNormalSignedAudit
                    | Mode::RefractoryCurvatureNormalLegacyControl
                    | Mode::RefractoryCurvatureNormalSignedStress
            )
        {
            update_attribution(&mut attribution, &mesh, &activity);
        }
        let sampled_localization = absolute_step == start_step + 1
            || absolute_step % 250 == 0
            || absolute_step == end_step;
        update_functional_pre(
            &mut functional_localization,
            &mesh,
            &activity,
            sampled_localization,
            &fission,
        );
        let edge_lengths_before = (0..mesh.n())
            .map(|index| mesh.edge_length(index))
            .collect::<Vec<_>>();
        let actuator_geometry_row = (sampled_localization
            && matches!(mode, Mode::RegulatorMotor | Mode::ContrastFallback))
        .then(|| actuator_geometry_before(&mesh, &activity, &mechanics, &contractility, &fission));

        let structural_before_mechanics = mesh.total_structural_mass();
        let mechanics_ok = match mode {
            Mode::Passive
            | Mode::RegulatorOnMotorOff
            | Mode::ContrastMotorOff
            | Mode::CurvatureNormalMotorOff
            | Mode::ContrastNormalMotorOff
            | Mode::RefractoryCurvatureNormalMotorOff => {
                mechanics_step_with_local_self_contact(&mut mesh, &mechanics).is_some()
            }
            Mode::RegulatorOffMotorOnZero | Mode::RegulatorMotor | Mode::ContrastFallback => {
                let zeros = vec![[0.0, 0.0]; mesh.n()];
                match apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact(
                    &mut mesh, &activity, &mechanics, &contractility, &zeros, 0.0, &zeros,
                ) {
                    Ok(ledger) => {
                        attribution.requested_a += ledger.requested_resource;
                        attribution.spent_a += ledger.resource_spent;
                        attribution.produced_w += ledger.waste_amount_after - ledger.waste_amount_before;
                        true
                    }
                    Err(_) => false,
                }
            }
            Mode::CurvatureNormal
            | Mode::CurvatureNormalTangential
            | Mode::ContrastNormal
            | Mode::ContrastNormalTangential
            | Mode::RefractoryCurvatureNormal
            | Mode::RefractoryCurvatureNormalAdaptationDisabled
            | Mode::RefractoryCurvatureNormalTangential
            | Mode::RefractoryCurvatureNormalSignedAudit
            | Mode::RefractoryCurvatureNormalLegacyControl
            | Mode::RefractoryCurvatureNormalSignedStress => {
                let (forces, requested) = inward_normal_request(&mesh, &activity, mechanics.dt);
                let tangential_activity = if matches!(
                    mode,
                    Mode::CurvatureNormalTangential
                        | Mode::ContrastNormalTangential
                        | Mode::RefractoryCurvatureNormalTangential
                ) {
                    activity.clone()
                } else {
                    vec![0.0; mesh.n()]
                };
                let passive = vec![[0.0, 0.0]; mesh.n()];
                match apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact(
                    &mut mesh,
                    &tangential_activity,
                    &mechanics,
                    &contractility,
                    &forces,
                    requested,
                    &passive,
                ) {
                    Ok(ledger) => {
                        attribution.requested_a += ledger.requested_resource;
                        attribution.spent_a += ledger.resource_spent;
                        attribution.produced_w += ledger.waste_amount_after - ledger.waste_amount_before;
                        true
                    }
                    Err(_) => false,
                }
            }
            Mode::ZeroA | Mode::ContrastZeroA => {
                let area_before = mesh.area().max(1e-300);
                let saved_a = mesh.interior.a.max(0.0) * area_before;
                mesh.interior.a = 0.0;
                let zeros = vec![[0.0, 0.0]; mesh.n()];
                let ledger = apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact(
                    &mut mesh, &activity, &mechanics, &contractility, &zeros, 0.0, &zeros,
                );
                match ledger {
                    Ok(ledger) => {
                        attribution.zero_a_spent += ledger.resource_spent;
                        mesh.interior.a = saved_a / mesh.area().max(1e-300);
                        true
                    }
                    Err(_) => false,
                }
            }
            Mode::CurvatureNormalZeroA
            | Mode::ContrastNormalZeroA
            | Mode::RefractoryCurvatureNormalZeroA => {
                let area_before = mesh.area().max(1e-300);
                let saved_a = mesh.interior.a.max(0.0) * area_before;
                mesh.interior.a = 0.0;
                let (forces, requested) = inward_normal_request(&mesh, &activity, mechanics.dt);
                let tangential_activity = vec![0.0; mesh.n()];
                let passive = vec![[0.0, 0.0]; mesh.n()];
                let ledger = apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact(
                    &mut mesh,
                    &tangential_activity,
                    &mechanics,
                    &contractility,
                    &forces,
                    requested,
                    &passive,
                );
                match ledger {
                    Ok(ledger) => {
                        attribution.zero_a_spent += ledger.resource_spent;
                        mesh.interior.a = saved_a / mesh.area().max(1e-300);
                        true
                    }
                    Err(_) => false,
                }
            }
        };
        if !mechanics_ok {
            first_invalid = Some(json!({"step":absolute_step,"phase":"mechanics"}));
            break;
        }
        if refractory_mode {
            if advance_local_plasticity_trace(
                &mut plasticity,
                &raw_activity,
                mechanics.dt,
                &plasticity_params,
            )
            .is_err()
            {
                refractory_audit.remesh_continuity_failures += 1;
                first_invalid = Some(json!({"step":absolute_step,"phase":"plasticity_commit"}));
                break;
            }
            refractory_audit.accepted_updates += 1;
            refractory_audit.maximum_adaptation = refractory_audit.maximum_adaptation.max(
                plasticity
                    .adaptation
                    .iter()
                    .copied()
                    .fold(0.0_f64, f64::max),
            );
            refractory_audit.adaptation_variance_sum += mean_variance(&plasticity.adaptation).1;
            refractory_audit.effective_drive_maximum = refractory_audit
                .effective_drive_maximum
                .max(activity.iter().copied().fold(0.0_f64, f64::max));
        }
        update_functional_contraction(
            &mut functional_localization,
            &activity,
            &edge_lengths_before,
            &mesh,
        );
        if let Some(row) = actuator_geometry_row {
            update_actuator_geometry_after(&mut actuator_geometry, &mesh, row);
        }
        let pre_remesh_frame =
            refractory_mode.then(|| observe_continuity_material_frame(&mesh, &mechanics));
        let old_n = mesh.n();
        let _ = remesh_preserving_simple(&mut mesh);
        if refractory_mode && mesh.n() != old_n {
            let new_frame = observe_continuity_material_frame(&mesh, &mechanics);
            let event = if mesh.n() > old_n {
                TopologyEventV1::Split
            } else {
                TopologyEventV1::Merge
            };
            let remap_result =
                derive_local_mapping(pre_remesh_frame.as_ref().unwrap(), &new_frame, event)
                    .and_then(|mapping| {
                        refractory_audit.remesh_mapping_count += 1;
                        refractory_audit.maximum_mapping_distance = refractory_audit
                            .maximum_mapping_distance
                            .max(mapping.maximum_mapping_distance);
                        plasticity.remap(&mapping).map_err(|error| {
                            regulatory_core::continuity::ContinuityError::MappingUnavailable(
                                error.to_string(),
                            )
                        })
                    });
            if remap_result.is_err() {
                refractory_audit.remesh_continuity_failures += 1;
                first_invalid = Some(json!({"step":absolute_step,"phase":"plasticity_remap"}));
                break;
            }
        }
        let planar = PlanarRingTopology::from_mesh(&mesh);
        if absolute_step.saturating_sub(1) % 10 == 0 {
            let _ = topology_step(&mut mesh, &fission);
        }
        expression_budget.remesh_topology_structural_net +=
            mesh.total_structural_mass() - structural_before_mechanics;
        all_simple &= polygon_simple(&mesh.vertices);
        all_runtime &= mesh.physical_runtime_valid();
        all_lifecycle &= mesh.lifecycle_invariants_hold();
        if !all_simple || !all_runtime || !all_lifecycle {
            first_invalid = Some(json!({"step":absolute_step,"phase":"post_topology"}));
            break;
        }
        let mass_ratio = mesh.total_structural_mass() / birth_mass.max(1e-300);
        maximum_mass_ratio = maximum_mass_ratio.max(mass_ratio);
        let max_strain = (0..mesh.n())
            .map(|i| mesh.strain(i))
            .fold(f64::NEG_INFINITY, f64::max);
        let strains = (0..mesh.n()).map(|i| mesh.strain(i)).collect::<Vec<_>>();
        let (_, strain_variance) = mean_variance(&strains);
        let checkpoint = absolute_step == start_step + 1
            || absolute_step % 250 == 0
            || absolute_step == end_step;
        let attempt_tick = mass_ratio >= 1.35 && absolute_step.saturating_sub(1) % 25 == 0;
        let pair = (checkpoint || attempt_tick).then(|| pair_observer(&mesh, &fission));
        if checkpoint && attribution.activity_steps > 0 {
            attribution.activity_neck_x.push(activity_mean_current);
            attribution.activity_neck_y.push(
                pair.as_ref().unwrap()["minimum_distance_over_range"]
                    .as_f64()
                    .unwrap_or(f64::INFINITY),
            );
        }
        if checkpoint {
            let compressed_edges = (0..mesh.n())
                .filter(|index| mesh.strain(*index) < 0.0)
                .count();
            let compressed_mature_mass = (0..mesh.n())
                .filter(|index| mesh.strain(*index) < 0.0)
                .map(|index| mesh.mature_structural_mass(index))
                .sum::<f64>();
            let total_mature_mass = (0..mesh.n())
                .map(|index| mesh.mature_structural_mass(index))
                .sum::<f64>();
            let compressive_stretch_work_proxy = (0..mesh.n())
                .filter(|index| mesh.strain(*index) < 0.0)
                .map(|index| {
                    let length = mesh.edge_length(index);
                    let rest = mesh.rest_length(index);
                    let l_ref = rest.max(0.25 * length).max(1e-3);
                    let fs_raw = (mechanics.k_s * (length - rest) / l_ref)
                        .clamp(-mechanics.k_s * 8.0, mechanics.k_s * 8.0);
                    (fs_raw * (length - rest)).abs()
                })
                .sum::<f64>();
            let (signed_curvature, concavity, _, _) = curvature_components(&mesh);
            let (_, curvature_variance) = mean_variance(&signed_curvature);
            checkpoints.push(json!({
                "step": absolute_step, "mass_over_birth": mass_ratio,
                "total_mass": mesh.total_structural_mass(),
                "young_mass": mesh.total_young_structural_mass(),
                "mature_mass": mesh.total_structural_mass()-mesh.total_young_structural_mass(),
                "actual_perimeter": mesh.perimeter(),
                "mature_rest_perimeter": (0..mesh.n()).map(|i| mesh.rest_length(i)).sum::<f64>(),
                "area": mesh.area(), "vertices": mesh.n(),
                "max_tensile_strain": max_strain, "strain_variance": strain_variance,
                "max_compression": strains.iter().copied().fold(f64::INFINITY,f64::min),
                "compressed_edges":compressed_edges,
                "compressed_mature_mass_fraction":compressed_mature_mass/total_mature_mass.max(1e-300),
                "compressive_stretch_work_proxy":compressive_stretch_work_proxy,
                "curvature_variance":curvature_variance,
                "concavity_count":concavity.iter().filter(|value|**value>0.0).count(),
                "rupture_count": mesh.edges.iter().filter(|e|e.ruptured).count(),
                "pair_observer": pair.as_ref().unwrap(), "absolute_a": mesh.interior.a.max(0.0)*mesh.area(),
                "simple": true, "runtime_valid": true,
            }));
        }
        if attempt_tick {
            if mode == Mode::RefractoryCurvatureNormalSignedAudit {
                let audits = segment_apposition_stress_audit(&mesh, &fission);
                if !audits.is_empty() {
                    signed_stress_attempts.push(json!({
                        "step": absolute_step,
                        "candidates": audits,
                    }));
                }
                if signed_stress_counterfactual.is_none() {
                    for candidate in segment_apposition_stress_audit(&mesh, &fission)
                        .into_iter()
                        .filter(|candidate| {
                            candidate.signed_magnitude_predicate
                                && !candidate.legacy_tensile_only_predicate
                        })
                    {
                        let Some((a, b, event)) = try_local_segment_fission_at_observed_apposition(
                            &mesh,
                            &fission,
                            candidate.edge_i,
                            candidate.edge_j,
                            candidate.segment_i_fraction,
                            candidate.segment_j_fraction,
                        ) else {
                            continue;
                        };
                        let parent_simple = polygon_simple(&mesh.vertices);
                        let daughter_a_simple = polygon_simple(&a.vertices);
                        let daughter_b_simple = polygon_simple(&b.vertices);
                        let daughter_a_runtime = a.physical_runtime_valid();
                        let daughter_b_runtime = b.physical_runtime_valid();
                        let daughter_a_lifecycle = a.lifecycle_invariants_hold();
                        let daughter_b_lifecycle = b.lifecycle_invariants_hold();
                        let daughter_a = daughter_viability(a);
                        let daughter_b = daughter_viability(b);
                        signed_stress_counterfactual = Some(json!({
                            "step": absolute_step,
                            "candidate": candidate,
                            "parent_simple": parent_simple,
                            "daughter_a_simple": daughter_a_simple,
                            "daughter_b_simple": daughter_b_simple,
                            "daughter_a_runtime_valid": daughter_a_runtime,
                            "daughter_b_runtime_valid": daughter_b_runtime,
                            "daughter_a_lifecycle_valid": daughter_a_lifecycle,
                            "daughter_b_lifecycle_valid": daughter_b_lifecycle,
                            "partition": event.partition,
                            "daughter_a_continuation": daughter_a,
                            "daughter_b_continuation": daughter_b,
                            "observer_only": true,
                        }));
                        break;
                    }
                }
            }
            let (mut reason, detail) = classify_attempt(&mesh, &fission);
            let proposed = try_local_fission(&mesh, &fission).or_else(|| {
                if matches!(
                    mode,
                    Mode::RefractoryCurvatureNormalSignedAudit
                        | Mode::RefractoryCurvatureNormalLegacyControl
                ) {
                    try_local_segment_fission_legacy_tensile_only(&mesh, &fission)
                } else {
                    planar
                        .as_ref()
                        .and_then(|p| p.try_local_scission(&mesh, &fission))
                        .or_else(|| try_local_segment_fission(&mesh, &fission))
                }
            });
            if let Some((a, b, event)) = proposed {
                if !event.partition.ok {
                    reason = "SCISSION_PROPOSED_PARTITION_FAILURE".into();
                } else if !polygon_simple(&a.vertices)
                    || !polygon_simple(&b.vertices)
                    || !a.physical_runtime_valid()
                    || !b.physical_runtime_valid()
                    || !a.lifecycle_invariants_hold()
                    || !b.lifecycle_invariants_hold()
                {
                    reason = "SCISSION_PROPOSED_INVALID_GEOMETRY".into();
                } else {
                    let allocation_fission = allocation_fission_continuity(&mesh, &a, &b);
                    let full_state = if mode == Mode::RefractoryCurvatureNormalSignedStress {
                        let state_a = partition_plasticity_state(
                            &plasticity,
                            &event.daughter_a_parent_vertex_sources,
                        );
                        let state_b = partition_plasticity_state(
                            &plasticity,
                            &event.daughter_b_parent_vertex_sources,
                        );
                        match (state_a, state_b) {
                            (Some(state_a), Some(state_b))
                                if state_a.adaptation.len() == a.n()
                                    && state_b.adaptation.len() == b.n() =>
                            {
                                let result_a = daughter_viability_with_plasticity_and_expression(
                                    a.clone(),
                                    state_a,
                                    expression_path,
                                );
                                let result_b = daughter_viability_with_plasticity_and_expression(
                                    b.clone(),
                                    state_b,
                                    expression_path,
                                );
                                let pass = result_a["viable"] == true && result_b["viable"] == true;
                                full_state_daughters_viable = Some(pass);
                                Some(json!({
                                    "source_correspondence": {
                                        "daughter_a": event.daughter_a_parent_vertex_sources,
                                        "daughter_b": event.daughter_b_parent_vertex_sources,
                                    },
                                    "daughter_a": result_a,
                                    "daughter_b": result_b,
                                    "both_viable": pass,
                                }))
                            }
                            _ => {
                                full_state_daughters_viable = Some(false);
                                Some(json!({
                                    "mapping_failure": true,
                                    "daughter_a_sources": event.daughter_a_parent_vertex_sources,
                                    "daughter_b_sources": event.daughter_b_parent_vertex_sources,
                                }))
                            }
                        }
                    } else {
                        None
                    };
                    let va = daughter_viability(a);
                    let vb = daughter_viability(b);
                    both_viable = va["viable"] == true && vb["viable"] == true;
                    let mut diagnostics = json!({
                        "step": absolute_step,
                        "daughter_a": va,
                        "daughter_b": vb,
                        "both_viable": both_viable,
                        "full_state": full_state,
                    });
                    if matches!(
                        expression_path,
                        ExpressionPath::D096V3IntensiveGain | ExpressionPath::D096V4CenteredGain
                    ) {
                        diagnostics
                            .as_object_mut()
                            .expect("daughter diagnostics object")
                            .insert("allocation_fission_continuity".into(), allocation_fission);
                    }
                    daughter_diagnostics = Some(diagnostics);
                    reason = if both_viable {
                        "VALID_FISSION"
                    } else {
                        "VALID_FISSION_DAUGHTER_NONVIABILITY"
                    }
                    .into();
                    fission_step = Some(absolute_step);
                }
            }
            bump(&mut counts, &reason);
            if failure_depth(&reason) >= failure_depth(&deepest) {
                deepest = reason.clone();
            }
            attempts.push(json!({"step":absolute_step,"reason":reason,"detail":detail}));
            if fission_step.is_some() {
                break;
            }
        }
    }
    expression_budget.active_motor_a = attribution.spent_a;
    expression_budget.active_motor_w = attribution.produced_w;
    expression_budget.terminal_structural_mass = mesh.total_structural_mass();
    expression_budget.terminal_catalyst_mass = mesh
        .finite_allocation
        .map(|state| state.catalysts.iter().sum())
        .unwrap_or(0.0);
    RunResult {
        name: name.into(),
        mode,
        start_step,
        terminal_step: fission_step.unwrap_or(end_step),
        birth_mass,
        max_mass_over_birth: maximum_mass_ratio,
        physical_fission: fission_step.is_some(),
        both_daughters_viable: both_viable,
        full_state_daughters_viable,
        daughter_diagnostics,
        fission_step,
        first_invalid,
        all_simple,
        all_runtime_valid: all_runtime,
        all_lifecycle_valid: all_lifecycle,
        failure_counts: counts,
        deepest_failure: deepest,
        attempts,
        checkpoints,
        attribution,
        final_geometry: geometry(&mesh),
        functional_localization,
        actuator_geometry,
        patch_dynamics,
        refractory_audit,
        signed_stress_attempts,
        signed_stress_counterfactual,
        expression_budget,
        final_mesh: mesh,
        final_plasticity: plasticity,
    }
}

fn run_with_expression(
    initial_mesh: MaterialMesh,
    name: &str,
    mode: Mode,
    start_step: usize,
    end_step: usize,
    birth_mass: f64,
    expression_path: ExpressionPath,
) -> RunResult {
    run_with_expression_gain_policy(
        initial_mesh,
        name,
        mode,
        start_step,
        end_step,
        birth_mass,
        expression_path,
        DiagnosticGainPolicy::ProductionAmount,
    )
}

fn run(
    initial_mesh: MaterialMesh,
    name: &str,
    mode: Mode,
    start_step: usize,
    end_step: usize,
    birth_mass: f64,
) -> RunResult {
    run_with_expression(
        initial_mesh,
        name,
        mode,
        start_step,
        end_step,
        birth_mass,
        ExpressionPath::Off,
    )
}

/// Exact production-V4 parent state from the first R10 qualifying campaign arm.
/// This is an assay fixture: it contains the authoritative mesh and organism-owned
/// refractory state immediately before the accepted signed-stress fission.
pub fn r10_seed3_fission_state() -> (MaterialMesh, PlasticityStateV1, f64, usize) {
    let mesh = fixture(2);
    let birth_mass = mesh.total_structural_mass();
    let result = run(
        mesh,
        "seed_3_c_0.08",
        Mode::RefractoryCurvatureNormalSignedStress,
        0,
        14_778,
        birth_mass,
    );
    assert!(result.physical_fission);
    (
        result.final_mesh,
        result.final_plasticity,
        birth_mass,
        result.fission_step.expect("R10 seed-3 fission step"),
    )
}

/// Partition organism-owned refractory state using the exact local parent
/// correspondence emitted by the physical fission operation.
pub fn r10_partition_plasticity_state(
    parent: &PlasticityStateV1,
    sources: &[usize],
) -> Option<PlasticityStateV1> {
    partition_plasticity_state(parent, sources)
}

/// Apply exactly one accepted R9/R10 refractory curvature-normal mechanics
/// operation, including ordinary local remesh continuity and optional frozen
/// topology cadence. Transport, reactions, growth, expression, and fission are
/// intentionally owned by the calling ecology harness.
pub fn r10_refractory_mechanics_step(
    mesh: &mut MaterialMesh,
    plasticity: &mut PlasticityStateV1,
    topology_tick: bool,
) -> Option<(f64, f64, usize)> {
    if plasticity.adaptation.len() != mesh.n() || !mesh.can_advance_physics() {
        return None;
    }
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let plasticity_params = PlasticityParamsV1::default();
    let raw_drive = curvature_components(mesh).3;
    let effective_drive = raw_drive
        .iter()
        .zip(&plasticity.adaptation)
        .map(|(drive, adaptation)| drive * (1.0 - adaptation))
        .collect::<Vec<_>>();
    let (forces, requested) = inward_normal_request(mesh, &effective_drive, mechanics.dt);
    let zeros = vec![[0.0, 0.0]; mesh.n()];
    let ledger = apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact(
        mesh,
        &vec![0.0; mesh.n()],
        &mechanics,
        &contractility,
        &forces,
        requested,
        &zeros,
    )
    .ok()?;
    advance_local_plasticity_trace(plasticity, &raw_drive, mechanics.dt, &plasticity_params)
        .ok()?;
    let old_frame = observe_continuity_material_frame(mesh, &mechanics);
    let old_n = mesh.n();
    let _ = remesh_preserving_simple(mesh);
    let mut remesh_mappings = 0;
    if mesh.n() != old_n {
        let new_frame = observe_continuity_material_frame(mesh, &mechanics);
        let event = if mesh.n() > old_n {
            TopologyEventV1::Split
        } else {
            TopologyEventV1::Merge
        };
        let mapping = derive_local_mapping(&old_frame, &new_frame, event).ok()?;
        plasticity.remap(&mapping).ok()?;
        remesh_mappings = 1;
    }
    if topology_tick {
        let _ = topology_step(mesh, &FissionParams::default());
    }
    (polygon_simple(&mesh.vertices)
        && mesh.physical_runtime_valid()
        && mesh.lifecycle_invariants_hold()
        && plasticity.adaptation.len() == mesh.n())
    .then_some((
        ledger.resource_spent,
        ledger.waste_amount_after - ledger.waste_amount_before,
        remesh_mappings,
    ))
}

/// Observer-only R9 Gate 1 replay of the sealed R8R1 curvature-normal campaign.
/// It adds per-patch diagnostics but does not add or advance adaptation state.
pub fn run_r9_gate1() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r9_gate1.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let runs = campaign(Mode::CurvatureNormal, 14_778);
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive": "DC-FINAL-001-R9-REFRACTORY-CURVATURE-NORMAL-CORTEX-REPRODUCTION-AND-END-GOAL-CLOSURE-001",
            "gate": "GATE_1_STATIC_ATTRACTOR_OBSERVER_ONLY",
            "scientific_runtime_changed": false,
            "adaptation_state_present": false,
            "campaign": campaign_summary(&runs),
        }))
        .unwrap(),
    )
    .unwrap();
}

fn refractory_patch_summary(runs: &[RunResult]) -> Value {
    let rows = runs
        .iter()
        .map(|run| {
            let raw_changes = run
                .patch_dynamics
                .windows(2)
                .filter(|window| {
                    window[0]["dominant_patch_index"] != window[1]["dominant_patch_index"]
                })
                .count();
            let effective_changes = run
                .patch_dynamics
                .windows(2)
                .filter(|window| {
                    window[0]["effective_dominant_patch_index"]
                        != window[1]["effective_dominant_patch_index"]
                })
                .count();
            let relocation_rows = run
                .patch_dynamics
                .iter()
                .filter(|row| row["dominant_patch_index"] != row["effective_dominant_patch_index"])
                .count();
            let final_row = run.patch_dynamics.last().cloned();
            json!({
                "name": run.name,
                "physical_fission": run.physical_fission,
                "both_daughters_viable": run.both_daughters_viable,
                "raw_dominant_patch_changes": raw_changes,
                "effective_dominant_patch_changes": effective_changes,
                "samples_with_raw_effective_relocation": relocation_rows,
                "sample_count": run.patch_dynamics.len(),
                "refractory_audit": run.refractory_audit,
                "final_sample": final_row,
            })
        })
        .collect::<Vec<_>>();
    let total_relocations = rows
        .iter()
        .filter_map(|row| row["samples_with_raw_effective_relocation"].as_u64())
        .sum::<u64>();
    json!({
        "arms": rows,
        "total_samples_with_raw_effective_relocation": total_relocations,
        "patch_relocation_observed": total_relocations > 0,
    })
}

/// R9 composes the frozen curvature-normal actuator with the existing local
/// plasticity trace. No new coefficient or fission information enters the
/// physical path.
pub fn run_r9() {
    const R9_DIRECTIVE: &str =
        "DC-FINAL-001-R9-REFRACTORY-CURVATURE-NORMAL-CORTEX-REPRODUCTION-AND-END-GOAL-CLOSURE-001";
    let mut output = PathBuf::from("/tmp/dcfinal001_r9.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }

    let horizon = 14_778;
    let passive = campaign(Mode::Passive, horizon);
    let static_normal = campaign(Mode::CurvatureNormal, horizon);
    let refractory_normal = campaign(Mode::RefractoryCurvatureNormal, horizon);
    let adaptation_disabled = campaign(Mode::RefractoryCurvatureNormalAdaptationDisabled, horizon);
    let motor_off = campaign(Mode::RefractoryCurvatureNormalMotorOff, horizon);
    let zero_a = campaign(Mode::RefractoryCurvatureNormalZeroA, horizon);

    let (_, static_fissions, static_viable) = result_counts(&static_normal);
    let (refractory_growth, refractory_fissions, refractory_viable) =
        result_counts(&refractory_normal);
    let refractory_pass =
        refractory_growth >= 8 && refractory_fissions >= 7 && refractory_viable >= 6;
    let improved = refractory_fissions > static_fissions || refractory_viable > static_viable;
    let refractory_tangential = (improved && !refractory_pass)
        .then(|| campaign(Mode::RefractoryCurvatureNormalTangential, horizon));
    let selected = refractory_tangential.as_ref().unwrap_or(&refractory_normal);
    let (selected_growth, selected_fissions, selected_viable) = result_counts(selected);
    let reproduction_pass = selected_growth >= 8 && selected_fissions >= 7 && selected_viable >= 6;

    let energy = active_energy_summary(&refractory_normal);
    let disabled_parity = campaign_state_parity(&static_normal, &adaptation_disabled);
    let motor_off_parity = campaign_state_parity(&passive, &motor_off);
    let remesh_continuity_pass = refractory_normal.iter().all(|run| {
        run.refractory_audit.remesh_continuity_failures == 0
            && run.refractory_audit.accepted_updates > 0
    });
    let first_step_identity = refractory_normal
        .iter()
        .all(|run| run.refractory_audit.first_step_effective_drive_error <= 1e-15);
    let zero_a_spent = zero_a
        .iter()
        .map(|run| run.attribution.zero_a_spent)
        .sum::<f64>();
    let patch_dynamics = refractory_patch_summary(&refractory_normal);

    let result = json!({
        "directive": R9_DIRECTIVE,
        "starting_head": "cdeec37274403286bd43efe847d30ec9b4d1593d",
        "owner_override": "ACTIVE",
        "qualification_horizon": horizon,
        "refractory_contract": {
            "schema": regulatory_core::PLASTICITY_SCHEMA_V1,
            "load_rate_per_time": FROZEN_ADAPTATION_LOAD_RATE_PER_TIME,
            "recovery_rate_per_time": FROZEN_ADAPTATION_RECOVERY_RATE_PER_TIME,
            "parameters": PlasticityParamsV1::default(),
            "new_free_parameters": 0,
            "raw_curvature_drives_adaptation": true,
            "adaptation_before_drives_current_force": true,
            "commit_after_accepted_mechanics": true,
            "first_step_identity": first_step_identity,
        },
        "controls": {
            "passive": campaign_summary(&passive),
            "static_curvature_normal": campaign_summary(&static_normal),
            "adaptation_disabled": campaign_summary(&adaptation_disabled),
            "adaptation_disabled_parity": disabled_parity,
            "motor_off": campaign_summary(&motor_off),
            "motor_off_passive_parity": motor_off_parity,
            "zero_a": campaign_summary(&zero_a),
            "zero_a_active_spend": zero_a_spent,
            "zero_a_pass": zero_a_spent <= CLASSIFICATION_TOLERANCE,
        },
        "refractory_normal": campaign_summary(&refractory_normal),
        "refractory_patch_dynamics": patch_dynamics,
        "remesh_continuity_pass": remesh_continuity_pass,
        "normal_energy_closure": energy,
        "conditional_normal_plus_tangential": {
            "triggered": refractory_tangential.is_some(),
            "trigger_reason": if improved && !refractory_pass {
                "REFRACTORY_NORMAL_IMPROVED_BUT_BELOW_QUALIFICATION"
            } else if refractory_pass {
                "NOT_REQUIRED_REFRACTORY_NORMAL_QUALIFIED"
            } else {
                "NOT_AUTHORIZED_NO_MATERIAL_IMPROVEMENT"
            },
            "campaign": refractory_tangential.as_ref().map(|runs| campaign_summary(runs)),
        },
        "selected_reproduction": {
            "mode": if refractory_tangential.is_some() {
                Mode::RefractoryCurvatureNormalTangential.label()
            } else {
                Mode::RefractoryCurvatureNormal.label()
            },
            "growth_qualified": selected_growth,
            "geometry_valid_fissions": selected_fissions,
            "simple_viable_daughter_pairs": selected_viable,
            "pass": reproduction_pass,
        },
        "evolution_execution": if reproduction_pass {"PENDING_GATE12_16"} else {"NOT_REACHED_GATE7_8_STOP"},
        "classification": if reproduction_pass {
            "V4_REFRACTORY_CURVATURE_NORMAL_ROBUST_REPRODUCTION_QUALIFIED_PENDING_EVOLUTION"
        } else {
            "V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED"
        },
        "digital_cell_end_goal": if reproduction_pass {"PENDING"} else {"NOT_ESTABLISHED"},
        "scientific_default_runtime_changed": false,
        "next_execution_started": false,
    });
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}

/// R10 Gates 1-2: exact R9 mechanics and legacy fission behavior with an
/// observer-only signed-load replay and clone-only downstream fission checks.
pub fn run_r10_signed_stress_audit() {
    const R10_DIRECTIVE: &str =
        "DC-FINAL-001-R10-SIGNED-LOAD-BEARING-NECK-STRESS-REPRODUCTION-AND-END-GOAL-CLOSURE-001";
    let mut output = PathBuf::from("/tmp/dcfinal001_r10_signed_stress_audit.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let runs = campaign(Mode::RefractoryCurvatureNormalSignedAudit, 14_778);
    let (growth, fissions, viable) = result_counts(&runs);
    let seed_summary = runs
        .iter()
        .enumerate()
        .map(|(index, run)| {
            let newly_admitted = run
                .signed_stress_attempts
                .iter()
                .flat_map(|attempt| attempt["candidates"].as_array().into_iter().flatten())
                .filter(|candidate| {
                    candidate["signed_magnitude_predicate"] == true
                        && candidate["legacy_tensile_only_predicate"] == false
                })
                .count();
            json!({
                "seed": index + 1,
                "legacy_physical_fission": run.physical_fission,
                "legacy_both_daughters_viable": run.both_daughters_viable,
                "in_range_attempts": run.signed_stress_attempts.len(),
                "newly_admitted_candidate_count": newly_admitted,
                "counterfactual": run.signed_stress_counterfactual,
            })
        })
        .collect::<Vec<_>>();
    let seed_pass = |seed: usize| {
        runs[seed - 1]
            .signed_stress_counterfactual
            .as_ref()
            .map(|counterfactual| {
                counterfactual["partition"]["ok"] == true
                    && counterfactual["daughter_a_continuation"]["viable"] == true
                    && counterfactual["daughter_b_continuation"]["viable"] == true
            })
            .unwrap_or(false)
    };
    let seed_7_pass = seed_pass(7);
    let seed_10_pass = seed_pass(10);
    let mismatch_confirmed = seed_7_pass && seed_10_pass;
    let result = json!({
        "directive": R10_DIRECTIVE,
        "starting_head": "42ec99f1eac5f302aa501a8168429c62d6045680",
        "gate": "GATES_1_2_OBSERVER_ONLY",
        "scientific_runtime_changed": false,
        "production_stress_predicate_changed": false,
        "qualification_horizon": 14_778,
        "legacy_r9_replay": {
            "growth_qualified": growth,
            "geometry_valid_fissions": fissions,
            "simple_viable_daughter_pairs": viable,
            "pass": fissions == 5 && viable == 5,
        },
        "effective_signed_strain_contract": {
            "v4_compression": "raw_strain",
            "v4_tension": "mature_fraction_times_raw_strain",
            "non_v4": "raw_strain",
            "stress_magnitude": 0.15,
            "new_free_parameters": 0,
        },
        "seed_7_compression_candidate": if seed_7_pass {"PASS"} else {"FAIL"},
        "seed_10_compression_candidate": if seed_10_pass {"PASS"} else {"FAIL"},
        "counterfactual_daughters": if mismatch_confirmed {"PASS"} else {"FAIL"},
        "classification": if mismatch_confirmed {
            "V4_SCISSION_TENSILE_ONLY_STRESS_MISMATCH_CONFIRMED"
        } else {
            "V4_SCISSION_TENSILE_ONLY_STRESS_MISMATCH_NOT_CONFIRMED"
        },
        "seed_summary": seed_summary,
        "campaign": campaign_summary(&runs),
    });
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}

/// R10 Gates 7-8: exact legacy control and the V4-only signed-load production
/// predicate on matched R9 refractory-normal campaigns.
pub fn run_r10_reproduction() {
    const R10_DIRECTIVE: &str =
        "DC-FINAL-001-R10-SIGNED-LOAD-BEARING-NECK-STRESS-REPRODUCTION-AND-END-GOAL-CLOSURE-001";
    let mut output = PathBuf::from("/tmp/dcfinal001_r10_reproduction.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let legacy = campaign(Mode::RefractoryCurvatureNormalLegacyControl, 14_778);
    let signed = campaign(Mode::RefractoryCurvatureNormalSignedStress, 14_778);
    let (legacy_growth, legacy_fissions, legacy_viable) = result_counts(&legacy);
    let (signed_growth, signed_fissions, signed_viable) = result_counts(&signed);
    let full_state_viable_pairs = signed
        .iter()
        .filter(|run| run.full_state_daughters_viable == Some(true))
        .count();
    let legacy_pass = legacy_growth == 10 && legacy_fissions == 5 && legacy_viable == 5;
    let reproduction_pass = signed_growth >= 8 && signed_fissions >= 7 && signed_viable >= 6;
    let full_state_pass = full_state_viable_pairs >= 6;
    let energy = active_energy_summary(&signed);
    let result = json!({
        "directive": R10_DIRECTIVE,
        "starting_head": "42ec99f1eac5f302aa501a8168429c62d6045680",
        "qualification_horizon": 14_778,
        "legacy_r9_control": campaign_summary(&legacy),
        "legacy_r9_control_pass": legacy_pass,
        "signed_stress_r9": campaign_summary(&signed),
        "signed_stress_counts": {
            "growth_qualified": signed_growth,
            "geometry_valid_fissions": signed_fissions,
            "simple_viable_daughter_pairs": signed_viable,
            "full_state_viable_daughter_pairs": full_state_viable_pairs,
        },
        "normal_energy_closure": energy,
        "stress_contract": {
            "v4_only": true,
            "stress_threshold": 0.15,
            "threshold_unchanged": true,
            "apposition_range_unchanged": true,
            "extreme_proximity_fraction": 0.55,
            "extreme_proximity_fraction_unchanged": true,
            "rupture_semantics": "TENSILE_ONLY_UNCHANGED",
            "new_free_parameters": 0,
        },
        "robust_v4_reproduction": reproduction_pass,
        "full_state_daughter_inheritance": {
            "executed": true,
            "viable_pairs": full_state_viable_pairs,
            "required_pairs": 6,
            "pass": full_state_pass,
        },
        "downstream_execution": if reproduction_pass && full_state_pass {
            "PENDING_GATES_9_15"
        } else {
            "NOT_REACHED_REPRODUCTION_OR_FULL_STATE_GATE"
        },
        "classification": if reproduction_pass && full_state_pass {
            "V4_SIGNED_LOAD_BEARING_NECK_STRESS_ROBUST_REPRODUCTION_QUALIFIED_PENDING_DOWNSTREAM"
        } else {
            "V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED"
        },
    });
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}

/// R10R3 Gate 3: run the exact R10 ten-arm production campaign with neutral
/// historical D096-v1 expression continuously enabled. Mutation remains off.
fn run_r10r3_integrated_reproduction(
    expression_path: ExpressionPath,
    expression_contract: &str,
    default_output: &str,
) {
    let mut output = PathBuf::from(default_output);
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let runs = PERTURBATIONS
        .iter()
        .copied()
        .enumerate()
        .map(|(index, (kind, magnitude))| {
            std::thread::spawn(move || {
                let mesh = fixture(index);
                let birth_mass = mesh.total_structural_mass();
                run_with_expression(
                    mesh,
                    &format!("seed_{}_{}_{}", index + 1, kind, magnitude),
                    Mode::RefractoryCurvatureNormalSignedStress,
                    0,
                    14_778,
                    birth_mass,
                    expression_path,
                )
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|handle| handle.join().expect("R10R3 D096-v1 campaign arm"))
        .collect::<Vec<_>>();
    let growth = runs
        .iter()
        .filter(|run| run.max_mass_over_birth >= 1.35)
        .count();
    let fissions = runs.iter().filter(|run| run.physical_fission).count();
    let full_state_viable_pairs = runs
        .iter()
        .filter(|run| run.full_state_daughters_viable == Some(true))
        .count();
    let rows = runs
        .iter()
        .map(|run| {
            json!({
                "name": run.name,
                "birth_mass": run.birth_mass,
                "maximum_mass_over_birth": run.max_mass_over_birth,
                "physical_fission": run.physical_fission,
                "fission_step": run.fission_step,
                "full_state_daughters_viable": run.full_state_daughters_viable,
                "deepest_failure": run.deepest_failure,
                "failure_counts": run.failure_counts,
                "attempts": run.attempts,
                "checkpoints": run.checkpoints,
                "all_simple": run.all_simple,
                "all_runtime_valid": run.all_runtime_valid,
                "all_lifecycle_valid": run.all_lifecycle_valid,
                "attribution": run.attribution,
                "expression_budget": run.expression_budget,
                "final_geometry": run.final_geometry,
                "actuator_geometry": run.actuator_geometry,
                "patch_dynamics": run.patch_dynamics,
                "refractory_audit": run.refractory_audit,
                "signed_stress_attempts": run.signed_stress_attempts,
                "signed_stress_counterfactual": run.signed_stress_counterfactual,
                "daughter_diagnostics": run.daughter_diagnostics,
            })
        })
        .collect::<Vec<_>>();
    let directive = if r10r9r3_buffered_reserve_enabled() {
        "DC-FINAL-001-R10R9R3-D091V2-BUFFERED-RESERVE-CANONICAL-GROWTH-REPRODUCTION-SPECIALIZATION-AND-M4-CLOSURE-001"
    } else if expression_path == ExpressionPath::D096V3IntensiveGain {
        "DC-FINAL-001-R10R4-D096-INTENSIVE-CATALYST-GAIN-INTEGRATED-REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001"
    } else if expression_path == ExpressionPath::D096V4CenteredGain {
        "DC-FINAL-001-R10R5-D096-FINITE-BUDGET-CENTERED-GAIN-INTEGRATED-REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001"
    } else {
        "DC-FINAL-001-R10R3-D096-ACTIVATED-MATERIAL-EXPRESSION-INTEGRATED-REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001"
    };
    let gate = if expression_path == ExpressionPath::D096V3IntensiveGain {
        "GATE_10_D096_V3_INTEGRATED_REPRODUCTION"
    } else if expression_path == ExpressionPath::D096V4CenteredGain {
        "GATE_8_D096_V4_INTEGRATED_REPRODUCTION"
    } else {
        "GATE_3_DIRECT_D096_V1_INTEGRATED_REPRODUCTION_BASELINE"
    };
    let value = json!({
        "directive": directive,
        "gate": gate,
        "expression_contract": expression_contract,
        "mutation_enabled": false,
        "horizon": 14_778,
        "d091_reserve_enabled": r10r9r1_reserve_enabled(),
        "growth_path": if r10r9r1_reserve_enabled() && r10r9r3_buffered_reserve_enabled() {
            "D091V2_BUFFERED_RESERVE_CANONICAL_D088_GROWTH"
        } else if r10r9r1_reserve_enabled() && r10r9r2_growth_path_surplus() {
            "D088_SURPLUS_A_OBSERVER_COUNTERFACTUAL"
        } else if r10r9r1_reserve_enabled() {
            "D091_RESERVE_FUNDED"
        } else {
            "D088_SURPLUS_A_PRODUCTION"
        },
        "d091_configuration": if r10r9r1_reserve_enabled() {
            serde_json::to_value(
                configured_r10_reserve(&fixture(0)),
            )
            .unwrap()
        } else {
            serde_json::Value::Null
        },
        "counts": {
            "growth_qualified": growth,
            "geometry_valid_fissions": fissions,
            "full_state_viable_daughter_pairs": full_state_viable_pairs,
        },
        "robust_reproduction": fissions >= 7 && full_state_viable_pairs >= 6,
        "runs": rows,
    });
    fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

pub fn run_r10r3_d096v1_integrated_reproduction() {
    run_r10r3_integrated_reproduction(
        ExpressionPath::D096V1,
        "D096_V1_STRUCTURAL_CONVERSION",
        "/tmp/dcfinal001_r10r3_d096v1_reproduction.json",
    );
}

pub fn run_r10r3_d096v2_integrated_reproduction() {
    run_r10r3_integrated_reproduction(
        ExpressionPath::D096V2ActivatedMaterial,
        "D096_V2_ACTIVATED_MATERIAL",
        "/tmp/dcfinal001_r10r3_d096v2_reproduction.json",
    );
}

pub fn run_r10r4_d096v3_integrated_reproduction() {
    run_r10r3_integrated_reproduction(
        ExpressionPath::D096V3IntensiveGain,
        "D096_V3_ACTIVATED_MATERIAL_INTENSIVE_GAIN",
        "/tmp/dcfinal001_r10r4_d096v3_reproduction.json",
    );
}

pub fn run_r10r5_d096v4_integrated_reproduction() {
    run_r10r3_integrated_reproduction(
        ExpressionPath::D096V4CenteredGain,
        "D096_V4_FINITE_BUDGET_CENTERED_INTENSIVE_GAIN",
        "/tmp/dcfinal001_r10r5_d096v4_reproduction.json",
    );
}

pub fn run_r10r9r3_d091v2_integrated_reproduction() {
    run_r10r3_integrated_reproduction(
        ExpressionPath::D096V4CenteredGain,
        "D091_V2_BUFFERED_RESERVE_CANONICAL_D088_GROWTH",
        "/tmp/dcfinal001_r10r9r3_d091v2_reproduction.json",
    );
}

pub fn run_r10r9r2_reproduction() {
    let mode = match env::var("DCFINAL001_R10R9R2_D096").ok().as_deref() {
        Some("off") | Some("OFF") | None => (ExpressionPath::Off, "D096_OFF"),
        Some("v4") | Some("V4") | Some("on") | Some("ON") => (
            ExpressionPath::D096V4CenteredGain,
            "D096_V4_FINITE_BUDGET_CENTERED_INTENSIVE_GAIN",
        ),
        Some(other) => panic!("unsupported DCFINAL001_R10R9R2_D096={other}"),
    };
    run_r10r3_integrated_reproduction(mode.0, mode.1, "/tmp/dcfinal001_r10r9r2_reproduction.json");
}

fn r10r4_diagnostic_campaign(policy: DiagnosticGainPolicy) -> Vec<RunResult> {
    PERTURBATIONS
        .iter()
        .copied()
        .enumerate()
        .map(|(index, (kind, magnitude))| {
            std::thread::spawn(move || {
                let mesh = fixture(index);
                let birth_mass = mesh.total_structural_mass();
                run_with_expression_gain_policy(
                    mesh,
                    &format!("seed_{}_{}_{}", index + 1, kind, magnitude),
                    Mode::RefractoryCurvatureNormalSignedStress,
                    0,
                    14_778,
                    birth_mass,
                    ExpressionPath::D096V2ActivatedMaterial,
                    policy,
                )
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|handle| handle.join().expect("R10R4 diagnostic campaign arm"))
        .collect()
}

fn r10r4_diagnostic_summary(policy: DiagnosticGainPolicy, runs: &[RunResult]) -> Value {
    let (growth, fissions, viable_pairs) = result_counts(runs);
    let fission = FissionParams::default();
    let rows = runs
        .iter()
        .map(|run| {
            let area = run.final_mesh.area();
            let catalysts = run
                .final_mesh
                .finite_allocation
                .map(|state| state.catalysts)
                .unwrap_or([0.0; 4]);
            let absolute_gains = catalysts.map(catalytic_gain);
            let concentration_gains = catalysts.map(|amount| catalytic_gain(amount / area));
            json!({
                "name": run.name,
                "physical_fission": run.physical_fission,
                "full_state_daughters_viable": run.full_state_daughters_viable,
                "fission_step": run.fission_step,
                "deepest_failure": run.deepest_failure,
                "maximum_mass_over_birth": run.max_mass_over_birth,
                "terminal_area": area,
                "terminal_catalyst_amounts": catalysts,
                "terminal_catalyst_total": catalysts.iter().sum::<f64>(),
                "terminal_catalyst_concentrations": catalysts.map(|amount| amount / area),
                "absolute_amount_gains": absolute_gains,
                "intensive_concentration_gains": concentration_gains,
                "terminal_apposition": pair_observer(&run.final_mesh, &fission),
                "all_simple": run.all_simple,
                "all_runtime_valid": run.all_runtime_valid,
                "all_lifecycle_valid": run.all_lifecycle_valid,
                "expression_budget": run.expression_budget,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "gain_policy": policy.label(),
        "counts": {
            "growth_qualified": growth,
            "geometry_valid_fissions": fissions,
            "full_state_viable_daughter_pairs": viable_pairs,
        },
        "runs": rows,
    })
}

/// R10R4 Gates 1-4. All five arms preserve exact D096-v2 catalyst production,
/// physical material costs, maintenance, turnover, and stored catalyst state.
/// The four counterfactual arms substitute only what the existing gain readers
/// observe during reactions/growth, then restore the physical catalyst amounts.
pub fn run_r10r4_gain_audit() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r10r4_gain_audit.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let policies = [
        DiagnosticGainPolicy::ProductionAmount,
        DiagnosticGainPolicy::CostOnly,
        DiagnosticGainPolicy::ProcessingActivationOnly,
        DiagnosticGainPolicy::StructuralBuildOnly,
        DiagnosticGainPolicy::IntensiveConcentration,
    ];
    let campaigns = policies
        .into_iter()
        .map(|policy| {
            let runs = r10r4_diagnostic_campaign(policy);
            r10r4_diagnostic_summary(policy, &runs)
        })
        .collect::<Vec<_>>();
    let parent_area = 100.0;
    let parent_amount = 5.0;
    let daughter_fraction = 0.37;
    let scaled_area = parent_area * 4.0;
    let scaled_amount = parent_amount * 4.0;
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive": "DC-FINAL-001-R10R4-D096-INTENSIVE-CATALYST-GAIN-INTEGRATED-REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001",
            "gate": "GATES_1_4_OBSERVER_ONLY",
            "production_biology_changed": false,
            "physical_catalyst_contract": {
                "state_unit": "EXTENSIVE_PHYSICAL_MATERIAL_AMOUNT",
                "synthesis_and_turnover_conserved": true,
                "fission_partition": "AREA_FRACTION",
                "classification": "D096_CATALYST_IS_EXTENSIVE_PHYSICAL_MATERIAL",
            },
            "scale_counterfactual": {
                "parent": {
                    "area": parent_area,
                    "catalyst_amount": parent_amount,
                    "absolute_gain": catalytic_gain(parent_amount),
                    "concentration_gain": catalytic_gain(parent_amount / parent_area),
                },
                "uniform_geometric_scale_factor": 2.0,
                "scaled": {
                    "area": scaled_area,
                    "catalyst_amount": scaled_amount,
                    "absolute_gain": catalytic_gain(scaled_amount),
                    "concentration_gain": catalytic_gain(scaled_amount / scaled_area),
                },
            },
            "fission_counterfactual": {
                "daughter_area_fraction": daughter_fraction,
                "daughter_area": parent_area * daughter_fraction,
                "daughter_catalyst_amount": parent_amount * daughter_fraction,
                "parent_absolute_gain": catalytic_gain(parent_amount),
                "daughter_absolute_gain": catalytic_gain(parent_amount * daughter_fraction),
                "parent_concentration_gain": catalytic_gain(parent_amount / parent_area),
                "daughter_concentration_gain": catalytic_gain(
                    parent_amount * daughter_fraction / (parent_area * daughter_fraction)
                ),
            },
            "gain_saturation_constant": 0.1,
            "new_parameters": 0,
            "campaigns": campaigns,
        }))
        .unwrap(),
    )
    .unwrap();
}

/// R10R5 Gates 1-4: observer-only finite-budget centered-gain algebra.
pub fn run_r10r5_gain_audit() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r10r5_gain_audit.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let cases = [
        ("zero", [0.0; 4]),
        ("neutral", [0.25; 4]),
        ("skewed", [0.1, 0.0, 0.0, 0.0]),
        ("mixed", [0.05, 0.02, 0.01, 0.0]),
    ];
    let rows = cases
        .into_iter()
        .map(|(name, concentrations)| {
            let total = concentrations.iter().sum::<f64>();
            let gains = concentrations
                .map(|concentration| 1.0 + ((4.0 * concentration) - total) / (0.1 + total));
            json!({
                "name": name,
                "concentrations": concentrations,
                "total_concentration": total,
                "gains": gains,
                "gain_sum": gains.iter().sum::<f64>(),
                "all_positive": gains.iter().all(|gain| *gain > 0.0),
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive": "DC-FINAL-001-R10R5-D096-FINITE-BUDGET-CENTERED-GAIN-INTEGRATED-REPRODUCTION-EVOLUTION-AND-END-GOAL-CLOSURE-001",
            "gate": "GATES_1_4_OBSERVER_ONLY",
            "equation": "gain_i = 1 + (4*c_i - C)/(0.1 + C)",
            "neutral_sum": 4.0,
            "neutral_gains": [1.0, 1.0, 1.0, 1.0],
            "scale_invariant": true,
            "new_parameters": 0,
            "cases": rows,
        }))
        .unwrap(),
    )
    .unwrap();
}

fn campaign(mode: Mode, horizon: usize) -> Vec<RunResult> {
    PERTURBATIONS
        .iter()
        .copied()
        .enumerate()
        .map(|(i, (kind, magnitude))| {
            std::thread::spawn(move || {
                let mesh = fixture(i);
                let birth = mesh.total_structural_mass();
                run(
                    mesh,
                    &format!("seed_{}_{}_{}", i + 1, kind, magnitude),
                    mode,
                    0,
                    horizon,
                    birth,
                )
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|handle| handle.join().expect("campaign arm panicked"))
        .collect()
}

fn extend_failed_passive(runs: &[RunResult], tau_steps: usize) -> Vec<RunResult> {
    runs.iter()
        .filter(|result| !result.physical_fission)
        .map(|result| {
            let mesh = result.final_mesh.clone();
            let name = result.name.clone();
            let birth_mass = result.birth_mass;
            std::thread::spawn(move || {
                run(
                    mesh,
                    &name,
                    Mode::Passive,
                    LEGACY_HORIZON,
                    LEGACY_HORIZON + tau_steps,
                    birth_mass,
                )
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|handle| handle.join().expect("passive extension arm panicked"))
        .collect()
}

fn campaign_summary(runs: &[RunResult]) -> Value {
    json!({
        "arms": runs.len(),
        "growth_qualified": runs.iter().filter(|r|r.max_mass_over_birth>=1.35).count(),
        "geometry_valid_fissions": runs.iter().filter(|r|r.physical_fission).count(),
        "simple_viable_daughter_pairs": runs.iter().filter(|r|r.both_daughters_viable).count(),
        "all_parent_states_simple": runs.iter().all(|r|r.all_simple),
        "all_runtime_valid": runs.iter().all(|r|r.all_runtime_valid),
        "all_lifecycle_valid": runs.iter().all(|r|r.all_lifecycle_valid),
        "runs": runs,
    })
}

fn reference_length_counterfactual(runs: &[RunResult]) -> Value {
    let mechanics = MechParams::default();
    let fission = FissionParams::default();
    let rows=runs.iter().map(|run| {
        let mesh=&run.final_mesh;
        let normal=(0..mesh.n()).map(|i|mesh.rest_length(i)).collect::<Vec<_>>();
        let upper=(0..mesh.n()).map(|i|(mesh.edges[i].m.max(0.0)/mesh.rho_s.max(1e-15)).max(1e-15)).collect::<Vec<_>>();
        let mut a=mesh.clone(); let mut b=mesh.clone();
        let normal_ok=mechanics_step_with_reference_lengths(&mut a,&mechanics,&normal);
        let upper_ok=mechanics_step_with_reference_lengths(&mut b,&mechanics,&upper);
        json!({
            "name":run.name,
            "normal": {"accepted":normal_ok,"simple":polygon_simple(&a.vertices),"pair":pair_observer(&a,&fission)},
            "immediate_maturation_upper_bound": {"accepted":upper_ok,"simple":polygon_simple(&b.vertices),"pair":pair_observer(&b,&fission)},
            "observer_only":true,
        })
    }).collect::<Vec<_>>();
    json!({"rows":rows,"production_state_mutated":false})
}

fn correlations(a: &Attribution) -> Value {
    let n = a.activity_neck_x.len();
    let sx = a.activity_neck_x.iter().sum::<f64>();
    let sy = a.activity_neck_y.iter().sum::<f64>();
    let sx2 = a.activity_neck_x.iter().map(|x| x * x).sum::<f64>();
    let sy2 = a.activity_neck_y.iter().map(|x| x * x).sum::<f64>();
    let sxy = a
        .activity_neck_x
        .iter()
        .zip(&a.activity_neck_y)
        .map(|(x, y)| x * y)
        .sum::<f64>();
    let lagged_n = n.saturating_sub(1);
    let lagged_x = &a.activity_neck_x[..lagged_n];
    let later_narrowing = a
        .activity_neck_y
        .windows(2)
        .map(|window| window[0] - window[1])
        .collect::<Vec<_>>();
    let lagged_sx = lagged_x.iter().sum::<f64>();
    let lagged_sy = later_narrowing.iter().sum::<f64>();
    let lagged_sx2 = lagged_x.iter().map(|x| x * x).sum::<f64>();
    let lagged_sy2 = later_narrowing.iter().map(|x| x * x).sum::<f64>();
    let lagged_sxy = lagged_x
        .iter()
        .zip(&later_narrowing)
        .map(|(x, y)| x * y)
        .sum::<f64>();
    json!({
        "strain_activity": pearson(a.strain_activity_sum_x,a.strain_activity_sum_y,a.strain_activity_sum_x2,a.strain_activity_sum_y2,a.strain_activity_sum_xy,a.strain_activity_n),
        "activity_vs_same_step_distance_over_range":pearson(sx,sy,sx2,sy2,sxy,n),
        "activity_vs_later_neck_narrowing": pearson(lagged_sx,lagged_sy,lagged_sx2,lagged_sy2,lagged_sxy,lagged_n),
        "later_narrowing_definition": "distance_over_range[t] - distance_over_range[t+1]",
    })
}

fn main() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r5_v4_neck.json");
    let args = env::args().collect::<Vec<_>>();
    for i in 1..args.len() {
        if args[i] == "--output" && i + 1 < args.len() {
            output = PathBuf::from(&args[i + 1]);
        }
    }
    let reaction = ReactionParams::default();
    let mechanics = MechParams::default();
    let tau_exact = 1.0 / (reaction.k_turn * mechanics.dt);
    let tau_steps = tau_exact.ceil() as usize;
    let passive = campaign(Mode::Passive, LEGACY_HORIZON);
    let failed_extensions = extend_failed_passive(&passive, tau_steps);
    let extension_fissions = failed_extensions
        .iter()
        .filter(|r| r.physical_fission)
        .count();
    let horizon_material = extension_fissions > 0;
    let qualification_horizon = if horizon_material {
        LEGACY_HORIZON + tau_steps
    } else {
        LEGACY_HORIZON
    };
    let mechanochemical = campaign(Mode::RegulatorMotor, qualification_horizon);
    let zero_activity = campaign(Mode::RegulatorOffMotorOnZero, qualification_horizon);
    let regulator_motor_off = campaign(Mode::RegulatorOnMotorOff, qualification_horizon);
    let zero_a = campaign(Mode::ZeroA, qualification_horizon);
    let total_activity_steps = mechanochemical
        .iter()
        .map(|r| r.attribution.activity_steps)
        .sum::<usize>();
    let mean_variance = mechanochemical
        .iter()
        .map(|r| r.attribution.activity_variance_sum)
        .sum::<f64>()
        / total_activity_steps.max(1) as f64;
    let contrast_required = mean_variance <= CLASSIFICATION_TOLERANCE;
    let contrast =
        contrast_required.then(|| campaign(Mode::ContrastFallback, qualification_horizon));
    let selected = contrast.as_ref().unwrap_or(&mechanochemical);
    let fissions = selected.iter().filter(|r| r.physical_fission).count();
    let viable = selected.iter().filter(|r| r.both_daughters_viable).count();
    let reproduction_pass = fissions >= 7
        && viable >= 6
        && selected
            .iter()
            .filter(|r| r.max_mass_over_birth >= 1.35)
            .count()
            >= 8;
    let requested = selected
        .iter()
        .map(|r| r.attribution.requested_a)
        .sum::<f64>();
    let spent = selected.iter().map(|r| r.attribution.spent_a).sum::<f64>();
    let produced = selected
        .iter()
        .map(|r| r.attribution.produced_w)
        .sum::<f64>();
    let result = json!({
        "directive":DIRECTIVE,
        "source_timescale":{"k_turn":reaction.k_turn,"dt":mechanics.dt,"tau_steps_exact":tau_exact,"tau_steps_ceil":tau_steps},
        "legacy_horizon":LEGACY_HORIZON,
        "maturation_horizon_audit":{"failed_arm_extensions":failed_extensions,"extension_fissions":extension_fissions,"classification":if horizon_material{"LEGACY_HORIZON_MATERIALLY_TRUNCATES_V4_FISSION"}else{"LEGACY_HORIZON_NOT_PRIMARY_CAUSE"}},
        "qualification_horizon":qualification_horizon,
        "passive":campaign_summary(&passive),
        "reference_length_counterfactual":reference_length_counterfactual(&passive),
        "mechanochemical":campaign_summary(&mechanochemical),
        "controls":{
            "regulator_off_motor_on_zero_activity":campaign_summary(&zero_activity),
            "regulator_on_motor_off":campaign_summary(&regulator_motor_off),
            "zero_a":campaign_summary(&zero_a),
        },
        "spatial_attribution":mechanochemical.iter().map(|r|json!({"name":r.name,"correlations":correlations(&r.attribution),"attribution":r.attribution})).collect::<Vec<_>>(),
        "contrast_fallback":{"required":contrast_required,"classification_tolerance":CLASSIFICATION_TOLERANCE,"mean_activity_variance":mean_variance,"campaign":contrast.as_ref().map(|r|campaign_summary(r))},
        "active_energy_closure":{"requested_a":requested,"spent_a":spent,"produced_w":produced,"residual":(spent-produced).abs(),"pass":(spent-produced).abs()<=1e-8*(1.0+spent)},
        "reproduction":{"selected_mode":if contrast_required{Mode::ContrastFallback.label()}else{Mode::RegulatorMotor.label()},"growth_qualified":selected.iter().filter(|r|r.max_mass_over_birth>=1.35).count(),"geometry_valid_fissions":fissions,"simple_viable_daughter_pairs":viable,"pass":reproduction_pass},
        "evolution_execution":if reproduction_pass{"PENDING_GATE10_14"}else{"NOT_REACHED_GATE9_STOP"},
        "classification":if reproduction_pass{"V4_ROBUST_PHYSICAL_REPRODUCTION_QUALIFIED_PENDING_EVOLUTION"}else{"V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED"},
        "digital_cell_end_goal":if reproduction_pass{"PENDING"}else{"NOT_ESTABLISHED"},
        "shutdown_recommended":!reproduction_pass,
        "new_free_parameters":0,
    });
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}

fn functional_correlations(functional: &FunctionalLocalization) -> Value {
    let count = functional.sampled_nearest_activity.len();
    let lagged_count = count.saturating_sub(1);
    let nearest = &functional.sampled_nearest_activity[..lagged_count];
    let outside = &functional.sampled_outside_activity[..lagged_count];
    let narrowing = functional
        .sampled_distance_over_range
        .windows(2)
        .map(|window| window[0] - window[1])
        .collect::<Vec<_>>();
    let correlate = |x: &[f64], y: &[f64]| {
        let sx = x.iter().sum::<f64>();
        let sy = y.iter().sum::<f64>();
        let sx2 = x.iter().map(|value| value * value).sum::<f64>();
        let sy2 = y.iter().map(|value| value * value).sum::<f64>();
        let sxy = x
            .iter()
            .zip(y)
            .map(|(left, right)| left * right)
            .sum::<f64>();
        pearson(sx, sy, sx2, sy2, sxy, x.len())
    };
    json!({
        "strain_to_activity": pearson(
            functional.strain_activity_sum_x,
            functional.strain_activity_sum_y,
            functional.strain_activity_sum_x2,
            functional.strain_activity_sum_y2,
            functional.strain_activity_sum_xy,
            functional.strain_activity_n,
        ),
        "activity_to_same_step_local_edge_contraction": pearson(
            functional.activity_contraction_sum_x,
            functional.activity_contraction_sum_y,
            functional.activity_contraction_sum_x2,
            functional.activity_contraction_sum_y2,
            functional.activity_contraction_sum_xy,
            functional.activity_contraction_n,
        ),
        "nearest_pair_activity_to_later_neck_narrowing": correlate(nearest, &narrowing),
        "outside_activity_to_later_neck_narrowing": correlate(outside, &narrowing),
        "later_neck_narrowing_definition": "minimum_distance_over_range[t] - minimum_distance_over_range[t+1]",
    })
}

fn functional_summary(runs: &[RunResult]) -> Value {
    let rows = runs
        .iter()
        .map(|run| {
            let functional = &run.functional_localization;
            let denominator = functional.steps.max(1) as f64;
            json!({
                "name": run.name,
                "physical_fission": run.physical_fission,
                "both_daughters_viable": run.both_daughters_viable,
                "fission_step": run.fission_step,
                "mean_activity": functional.activity_mean_sum / denominator,
                "mean_activity_variance": functional.activity_variance_sum / denominator,
                "mean_active_perimeter_fraction": functional.active_perimeter_fraction_sum / denominator,
                "mean_largest_contiguous_active_arc_fraction": functional.largest_contiguous_active_arc_fraction_sum / denominator,
                "mean_active_patch_count": functional.active_patch_count_sum as f64 / denominator,
                "max_active_patch_count": functional.max_active_patch_count,
                "best_apposition": functional.best_apposition,
                "correlations": functional_correlations(functional),
            })
        })
        .collect::<Vec<_>>();
    let mean = |field: &str| {
        rows.iter()
            .filter_map(|row| row[field].as_f64())
            .sum::<f64>()
            / rows.len().max(1) as f64
    };
    json!({
        "arms": rows,
        "campaign_means": {
            "activity": mean("mean_activity"),
            "activity_variance": mean("mean_activity_variance"),
            "active_perimeter_fraction": mean("mean_active_perimeter_fraction"),
            "largest_contiguous_active_arc_fraction": mean("mean_largest_contiguous_active_arc_fraction"),
            "active_patch_count": mean("mean_active_patch_count"),
        },
    })
}

fn result_counts(runs: &[RunResult]) -> (usize, usize, usize) {
    (
        runs.iter()
            .filter(|result| result.max_mass_over_birth >= 1.35)
            .count(),
        runs.iter().filter(|result| result.physical_fission).count(),
        runs.iter()
            .filter(|result| result.both_daughters_viable)
            .count(),
    )
}

/// R5R1 executes the sole preauthorized zero-parameter contrast fallback.
/// It is kept in this assay module so the exact R5 fixture, mechanics order,
/// topology, fission, and viability contracts are reused without divergence.
pub fn run_r5r1() {
    const R5R1_DIRECTIVE: &str = "DC-FINAL-001-R5R1-ZERO-PARAMETER-STRAIN-CONTRAST-NECK-LOCALIZATION-AND-TERMINAL-CLOSURE-001";
    let mut output = PathBuf::from("/tmp/dcfinal001_r5r1_strain_contrast.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let reaction = ReactionParams::default();
    let mechanics = MechParams::default();
    let tau_steps = (1.0 / (reaction.k_turn * mechanics.dt)).ceil() as usize;
    let qualification_horizon = LEGACY_HORIZON + tau_steps;
    assert_eq!(qualification_horizon, 14_778);

    let passive = campaign(Mode::Passive, qualification_horizon);
    let original = campaign(Mode::RegulatorMotor, qualification_horizon);
    let contrast = campaign(Mode::ContrastFallback, qualification_horizon);
    let contrast_motor_off = campaign(Mode::ContrastMotorOff, qualification_horizon);
    let contrast_zero_a = campaign(Mode::ContrastZeroA, qualification_horizon);

    let original_functional = functional_summary(&original);
    let contrast_functional = functional_summary(&contrast);
    let original_nonuniform = original_functional["campaign_means"]["activity_variance"]
        .as_f64()
        .unwrap_or(0.0)
        > CLASSIFICATION_TOLERANCE;
    let original_lagged = original_functional["arms"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|arm| {
            arm["correlations"]["nearest_pair_activity_to_later_neck_narrowing"].as_f64()
        })
        .collect::<Vec<_>>();
    let no_positive_neck_coupling = !original_lagged.is_empty()
        && original_lagged
            .iter()
            .all(|correlation| *correlation <= 0.0);
    let localization_classification = if !original_nonuniform {
        "UNIFORM"
    } else if no_positive_neck_coupling {
        "SPATIALLY_NONUNIFORM_BUT_FUNCTIONALLY_DIFFUSE"
    } else {
        "NECK_EFFECTIVELY_LOCALIZED"
    };

    let (passive_growth, passive_fissions, passive_viable) = result_counts(&passive);
    let (original_growth, original_fissions, original_viable) = result_counts(&original);
    let (contrast_growth, contrast_fissions, contrast_viable) = result_counts(&contrast);
    let (_, motor_off_fissions, motor_off_viable) = result_counts(&contrast_motor_off);
    let (_, zero_a_fissions, zero_a_viable) = result_counts(&contrast_zero_a);
    let requested = contrast
        .iter()
        .map(|result| result.attribution.requested_a)
        .sum::<f64>();
    let spent = contrast
        .iter()
        .map(|result| result.attribution.spent_a)
        .sum::<f64>();
    let produced = contrast
        .iter()
        .map(|result| result.attribution.produced_w)
        .sum::<f64>();
    let energy_residual = (spent - produced).abs();
    let energy_pass = energy_residual <= 1e-8 * (1.0 + spent);
    let zero_a_spent = contrast_zero_a
        .iter()
        .map(|result| result.attribution.zero_a_spent)
        .sum::<f64>();
    let reproduction_pass = contrast_growth >= 8 && contrast_fissions >= 7 && contrast_viable >= 6;

    let result = json!({
        "directive": R5R1_DIRECTIVE,
        "starting_head": "404dd374b8a75adc6b77a97c064b0197ab334628",
        "qualification_horizon": qualification_horizon,
        "localization_reclassification": {
            "classification": localization_classification,
            "variance_only_gate_rejected": true,
            "original_activity_nonuniform": original_nonuniform,
            "original_nearest_activity_lagged_correlations": original_lagged,
            "all_observed_correlations_nonpositive": no_positive_neck_coupling,
            "metrics": original_functional,
        },
        "contrast_contract": {
            "implementation": "adaptive_directional_drive.front_drive",
            "formula": "max(local_positive_tensile_strain - perimeter_weighted_mean_strain, 0)",
            "new_free_parameters": 0,
            "new_thresholds": 0,
            "new_state_variables": 0,
            "world_coordinates_read": false,
            "body_size_read": false,
            "reproduction_state_read": false,
            "observer_feedback": false,
        },
        "campaigns": {
            "passive": campaign_summary(&passive),
            "original_regulator": campaign_summary(&original),
            "contrast": campaign_summary(&contrast),
            "contrast_motor_off": campaign_summary(&contrast_motor_off),
            "contrast_zero_a": campaign_summary(&contrast_zero_a),
        },
        "spatialization": {
            "original": original_functional,
            "contrast": contrast_functional,
        },
        "controls": {
            "passive": {"growth":passive_growth,"fissions":passive_fissions,"viable_pairs":passive_viable},
            "original_regulator": {"growth":original_growth,"fissions":original_fissions,"viable_pairs":original_viable},
            "contrast_motor_off": {"fissions":motor_off_fissions,"viable_pairs":motor_off_viable},
            "contrast_zero_a": {"fissions":zero_a_fissions,"viable_pairs":zero_a_viable,"active_a_spent":zero_a_spent},
        },
        "active_energy_closure": {
            "requested_a": requested,
            "spent_a": spent,
            "produced_w": produced,
            "residual": energy_residual,
            "pass": energy_pass,
        },
        "reproduction": {
            "growth_qualified": contrast_growth,
            "geometry_valid_fissions": contrast_fissions,
            "simple_viable_daughter_pairs": contrast_viable,
            "pass": reproduction_pass,
        },
        "evolution_execution": if reproduction_pass {"REQUIRED_CONTINUE"} else {"NOT_REACHED_GATE5_STOP"},
        "classification": if reproduction_pass {"V4_ZERO_PARAMETER_STRAIN_CONTRAST_ROBUST_REPRODUCTION_QUALIFIED_PENDING_EVOLUTION"} else {"V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED"},
        "digital_cell_end_goal": if reproduction_pass {"PENDING_EVOLUTION_AND_FINAL_INTEGRATION"} else {"NOT_ESTABLISHED"},
        "shutdown_recommended": !reproduction_pass,
        "new_free_parameters": 0,
    });
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}

/// R6 Gate 1 replays the sealed R5R1 tangential contrast arm and observes the
/// active edge-tension force geometry. It does not add or apply a normal-force
/// mechanism.
pub fn run_r6_gate1() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r6_gate1.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let reaction = ReactionParams::default();
    let mechanics = MechParams::default();
    let horizon = LEGACY_HORIZON + (1.0 / (reaction.k_turn * mechanics.dt)).ceil() as usize;
    assert_eq!(horizon, 14_778);
    let contrast = campaign(Mode::ContrastFallback, horizon);
    let inward = contrast
        .iter()
        .map(|run| run.actuator_geometry.inward_normal_component_sum)
        .sum::<f64>();
    let outward = contrast
        .iter()
        .map(|run| run.actuator_geometry.outward_normal_component_sum)
        .sum::<f64>();
    let normal = contrast
        .iter()
        .map(|run| run.actuator_geometry.absolute_normal_component_sum)
        .sum::<f64>();
    let tangential = contrast
        .iter()
        .map(|run| run.actuator_geometry.absolute_tangential_component_sum)
        .sum::<f64>();
    let inward_fraction = inward / (inward + outward).max(1e-300);
    let normal_fraction = normal / (normal + tangential).max(1e-300);
    let equivalent = inward_fraction > 0.5 && normal_fraction >= 0.5;
    let result = json!({
        "directive": "DC-FINAL-001-R6-CURVATURE-GATED-NORMAL-CONSTRICTION-EMERGENCY-CLOSURE-001",
        "gate": 1,
        "starting_head": "456cc98837842af933d3972b8588707ea801bf28",
        "sealed_r5r1_replay": campaign_summary(&contrast),
        "force_geometry": {
            "inward_normal_component_sum": inward,
            "outward_normal_component_sum": outward,
            "absolute_normal_component_sum": normal,
            "absolute_tangential_component_sum": tangential,
            "inward_fraction_of_signed_normal_magnitude": inward_fraction,
            "normal_fraction_of_normal_plus_tangential_magnitude": normal_fraction,
            "per_arm_best_apposition": contrast.iter().map(|run| json!({
                "name":run.name,
                "geometry":run.actuator_geometry,
            })).collect::<Vec<_>>(),
        },
        "equivalence_rule": "inward_fraction > 0.5 AND normal_fraction >= 0.5",
        "classification": if equivalent {
            "TANGENTIAL_TENSION_ALREADY_EQUIVALENT_TO_NORMAL_CONSTRICTION"
        } else {
            "TANGENTIAL_TENSION_GEOMETRY_MISMATCH_SUPPORTED"
        },
        "scientific_state_changed": false,
    });
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}

fn curvature_signal_qualification() -> Value {
    let base = fixture(0);
    let base_drive = curvature_components(&base).3;
    let mut rotated = base.clone();
    let angle = 0.731_f64;
    let (sine, cosine) = angle.sin_cos();
    for point in &mut rotated.vertices {
        let x = point[0];
        let y = point[1];
        point[0] = cosine * x - sine * y + 3.25;
        point[1] = sine * x + cosine * y - 1.75;
    }
    let rotated_drive = curvature_components(&rotated).3;
    let rotation_translation_error = base_drive
        .iter()
        .zip(&rotated_drive)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f64::max);

    let mut reflected = base.clone();
    for point in &mut reflected.vertices {
        point[0] = -point[0];
    }
    let reflected_drive = curvature_components(&reflected).3;
    let reflection_error = base_drive
        .iter()
        .zip(&reflected_drive)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f64::max);

    let mut regular = chemistry_core::mesh_population::MeshPopulation::seed_one(5.0, 1, 2.2)
        .individuals
        .remove(0)
        .mesh;
    let regular_drive = curvature_components(&regular).3;
    let regular_max = regular_drive.iter().copied().fold(0.0, f64::max);
    let center = regular.centroid();
    let concave_vertex = 0;
    regular.vertices[concave_vertex][0] =
        0.25 * regular.vertices[concave_vertex][0] + 0.75 * center[0];
    regular.vertices[concave_vertex][1] =
        0.25 * regular.vertices[concave_vertex][1] + 0.75 * center[1];
    let (_, concavity, deficit, concave_drive) = curvature_components(&regular);
    let concave_response = concave_drive[concave_vertex];

    let (forces, requested) = inward_normal_request(&base, &base_drive, MechParams::default().dt);
    let maximum_force = forces
        .iter()
        .map(|force| force[0].hypot(force[1]))
        .fold(0.0, f64::max);
    let maximum_direction_error = forces
        .iter()
        .enumerate()
        .filter(|(vertex, _)| base_drive[*vertex] > 0.0)
        .map(|(vertex, force)| {
            let (inward, _) = local_inward_normal_and_tangent(&base, vertex);
            let magnitude = force[0].hypot(force[1]);
            if magnitude <= 0.0 {
                0.0
            } else {
                (1.0 - (force[0] * inward[0] + force[1] * inward[1]) / magnitude).abs()
            }
        })
        .fold(0.0, f64::max);
    let pass = rotation_translation_error <= 1e-12
        && reflection_error <= 1e-12
        && regular_max <= 1e-12
        && concave_response > 0.0
        && maximum_force <= MAX_EXTERNAL_FORCE_PER_VERTEX + 1e-12
        && maximum_direction_error <= 1e-12
        && requested >= 0.0;
    json!({
        "pass":pass,
        "formula":{
            "signed_turn":"orientation_sign * atan2(cross(tangent_prev,tangent_next), dot(tangent_prev,tangent_next))",
            "concavity":"max(-signed_turn/pi,0)",
            "curvature_deficit":"max((0.5*(previous_turn+next_turn)-signed_turn)/pi,0)",
            "normal_drive":"max(concavity,curvature_deficit) clamped [0,1]",
        },
        "rotation_translation_error":rotation_translation_error,
        "reflection_error":reflection_error,
        "regular_polygon_max_drive":regular_max,
        "concave_vertex":concave_vertex,
        "concavity_at_concave_vertex":concavity[concave_vertex],
        "deficit_at_concave_vertex":deficit[concave_vertex],
        "drive_at_concave_vertex":concave_response,
        "maximum_force":maximum_force,
        "force_bound":MAX_EXTERNAL_FORCE_PER_VERTEX,
        "maximum_inward_direction_error":maximum_direction_error,
        "requested_a":requested,
    })
}

fn apposition_score(runs: &[RunResult]) -> Value {
    let within = runs
        .iter()
        .flat_map(|run| &run.attempts)
        .map(|attempt| {
            attempt["detail"]["pairs"]["within_range_pairs"]
                .as_u64()
                .unwrap_or(0)
        })
        .sum::<u64>();
    let stressed = runs
        .iter()
        .flat_map(|run| &run.attempts)
        .map(|attempt| {
            attempt["detail"]["pairs"]["stressed_within_range_pairs"]
                .as_u64()
                .unwrap_or(0)
        })
        .sum::<u64>();
    let minimum_ratio = runs
        .iter()
        .flat_map(|run| &run.checkpoints)
        .filter_map(|checkpoint| {
            checkpoint["pair_observer"]["minimum_distance_over_range"].as_f64()
        })
        .fold(f64::INFINITY, f64::min);
    json!({
        "within_range_candidate_count_at_attempts":within,
        "stress_qualified_candidate_count_at_attempts":stressed,
        "minimum_sampled_distance_over_range":minimum_ratio,
    })
}

fn active_energy_summary(runs: &[RunResult]) -> Value {
    let requested = runs
        .iter()
        .map(|run| run.attribution.requested_a)
        .sum::<f64>();
    let spent = runs.iter().map(|run| run.attribution.spent_a).sum::<f64>();
    let produced = runs
        .iter()
        .map(|run| run.attribution.produced_w)
        .sum::<f64>();
    let residual = (spent - produced).abs();
    json!({
        "requested_a":requested,
        "spent_a":spent,
        "produced_w":produced,
        "residual":residual,
        "pass":residual <= 1e-8*(1.0+spent),
    })
}

fn campaign_state_parity(left: &[RunResult], right: &[RunResult]) -> Value {
    let mut maximum_vertex_error = 0.0_f64;
    let mut maximum_mass_error = 0.0_f64;
    let mut topology_mismatches = 0_usize;
    let mut event_mismatches = 0_usize;
    for (a, b) in left.iter().zip(right) {
        if a.final_mesh.n() != b.final_mesh.n() {
            topology_mismatches += 1;
        } else {
            for (pa, pb) in a.final_mesh.vertices.iter().zip(&b.final_mesh.vertices) {
                maximum_vertex_error = maximum_vertex_error
                    .max((pa[0] - pb[0]).abs())
                    .max((pa[1] - pb[1]).abs());
            }
        }
        maximum_mass_error = maximum_mass_error.max(
            (a.final_mesh.total_structural_mass() - b.final_mesh.total_structural_mass()).abs(),
        );
        if a.physical_fission != b.physical_fission
            || a.both_daughters_viable != b.both_daughters_viable
            || a.fission_step != b.fission_step
        {
            event_mismatches += 1;
        }
    }
    let pass = left.len() == right.len()
        && topology_mismatches == 0
        && event_mismatches == 0
        && maximum_vertex_error <= CLASSIFICATION_TOLERANCE
        && maximum_mass_error <= CLASSIFICATION_TOLERANCE;
    json!({
        "pass": pass,
        "arms_compared": left.len().min(right.len()),
        "maximum_vertex_error": maximum_vertex_error,
        "maximum_structural_mass_error": maximum_mass_error,
        "topology_mismatches": topology_mismatches,
        "event_mismatches": event_mismatches,
    })
}

fn cross_composition_qualification() -> Value {
    let base = fixture(0);
    let drive = contrast_activity(&base);
    let repeated_drive = contrast_activity(&base);
    let drive_identity_error = drive
        .iter()
        .zip(&repeated_drive)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    let base_normals = (0..base.n())
        .map(|i| local_inward_normal_and_tangent(&base, i).0)
        .collect::<Vec<_>>();
    let repeated_normals = (0..base.n())
        .map(|i| local_inward_normal_and_tangent(&base, i).0)
        .collect::<Vec<_>>();
    let normal_identity_error = base_normals
        .iter()
        .zip(&repeated_normals)
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0, f64::max);

    let angle = 0.731_f64;
    let (sine, cosine) = angle.sin_cos();
    let mut rotated = base.clone();
    for point in &mut rotated.vertices {
        let x = point[0];
        let y = point[1];
        point[0] = cosine * x - sine * y + 3.25;
        point[1] = sine * x + cosine * y - 1.75;
    }
    let rotated_drive = contrast_activity(&rotated);
    let rotation_drive_error = drive
        .iter()
        .zip(&rotated_drive)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    let rotation_normal_error = base_normals
        .iter()
        .enumerate()
        .map(|(i, normal)| {
            let expected = [
                cosine * normal[0] - sine * normal[1],
                sine * normal[0] + cosine * normal[1],
            ];
            let observed = local_inward_normal_and_tangent(&rotated, i).0;
            (expected[0] - observed[0])
                .abs()
                .max((expected[1] - observed[1]).abs())
        })
        .fold(0.0, f64::max);

    let mut reflected = base.clone();
    for point in &mut reflected.vertices {
        point[0] = -point[0];
    }
    let reflected_drive = contrast_activity(&reflected);
    let reflection_drive_error = drive
        .iter()
        .zip(&reflected_drive)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    let reflection_normal_error = base_normals
        .iter()
        .enumerate()
        .map(|(i, normal)| {
            let expected = [-normal[0], normal[1]];
            let observed = local_inward_normal_and_tangent(&reflected, i).0;
            (expected[0] - observed[0])
                .abs()
                .max((expected[1] - observed[1]).abs())
        })
        .fold(0.0, f64::max);

    let mut regular = chemistry_core::mesh_population::MeshPopulation::seed_one(5.0, 1, 2.2)
        .individuals
        .remove(0)
        .mesh;
    regular.stamp_maturation_coupled_schema();
    let regular_drive = contrast_activity(&regular);
    let regular_max_drive = regular_drive.iter().copied().fold(0.0, f64::max);
    let (forces, requested_a) = inward_normal_request(&base, &drive, MechParams::default().dt);
    let maximum_force = forces
        .iter()
        .map(|force| force[0].hypot(force[1]))
        .fold(0.0, f64::max);
    let pass = drive_identity_error <= CLASSIFICATION_TOLERANCE
        && normal_identity_error <= CLASSIFICATION_TOLERANCE
        && rotation_drive_error <= CLASSIFICATION_TOLERANCE
        && rotation_normal_error <= CLASSIFICATION_TOLERANCE
        && reflection_drive_error <= CLASSIFICATION_TOLERANCE
        && reflection_normal_error <= CLASSIFICATION_TOLERANCE
        && regular_max_drive <= CLASSIFICATION_TOLERANCE
        && maximum_force <= MAX_EXTERNAL_FORCE_PER_VERTEX + CLASSIFICATION_TOLERANCE
        && requested_a >= 0.0;
    json!({
        "pass": pass,
        "classification": if pass {"STRAIN_CONTRAST_NORMAL_COMPOSITION_EXACT"} else {"STRAIN_CONTRAST_NORMAL_COMPOSITION_INVALID"},
        "drive_authority": "R5R1 adaptive_directional_drive.front_drive",
        "drive_formula": "max(local_positive_tensile_strain - perimeter_weighted_mean_strain, 0)",
        "normal_authority": "R6 orientation-aware current_local_inward_normal",
        "force_authority": "R6 funded extra-force path and frozen force budget",
        "curvature_term_in_primary_drive": false,
        "new_free_parameters": 0,
        "fission_or_topology_information_read": false,
        "drive_identity_error": drive_identity_error,
        "normal_identity_error": normal_identity_error,
        "rotation_drive_error": rotation_drive_error,
        "rotation_normal_error": rotation_normal_error,
        "reflection_drive_error": reflection_drive_error,
        "reflection_normal_error": reflection_normal_error,
        "uniform_no_strain_max_drive": regular_max_drive,
        "maximum_requested_force": maximum_force,
        "force_bound": MAX_EXTERNAL_FORCE_PER_VERTEX,
        "requested_a": requested_a,
    })
}

fn daughter_diagnostic_rows(runs: &[RunResult]) -> Value {
    json!({
        "physical_fissions": runs.iter().filter(|run| run.physical_fission).count(),
        "rows": runs.iter().filter_map(|run| run.daughter_diagnostics.as_ref().map(|diagnostics| json!({
            "name": run.name,
            "mode": run.mode.label(),
            "fission_step": run.fission_step,
            "diagnostics": diagnostics,
        }))).collect::<Vec<_>>(),
    })
}

fn r8_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/fixtures/dcfinal001r8")
        .join(name)
}

fn replay_r8_daughter_fixture(name: &str) -> Value {
    let path = r8_fixture_path(name);
    let raw = fs::read_to_string(&path).expect("R8 daughter fixture must be readable");
    let mesh: MaterialMesh =
        serde_json::from_str(&raw).expect("R8 daughter fixture must be a MaterialMesh");
    json!({
        "fixture":name,
        "source_sha256_recorded_in_protocol":true,
        "corrected_continuation":daughter_viability(mesh),
    })
}

fn r8r1_edge_force_counterfactual(mesh: &MaterialMesh, index: usize) -> Value {
    let mechanics = MechParams::default();
    let length = mesh.edge_length(index);
    let rest = mesh.rest_length(index);
    let l_ref = rest.max(0.25 * length).max(1e-3);
    let fs_raw =
        (mechanics.k_s * (length - rest) / l_ref).clamp(-mechanics.k_s * 8.0, mechanics.k_s * 8.0);
    let raw_strain = mesh.strain(index);
    let mature_fraction = mesh.mature_structural_fraction(index);
    let r8_fs = mature_fraction * fs_raw;
    let sign_aware_fs = if raw_strain < 0.0 {
        fs_raw
    } else {
        mature_fraction * fs_raw
    };
    let edge = mesh.edges[index];
    json!({
        "edge":index,
        "length":length,
        "total_m":edge.m,
        "m_young":mesh.young_structural_mass(index),
        "mature_m":mesh.mature_structural_mass(index),
        "mature_fraction":mature_fraction,
        "mature_rest_length":rest,
        "raw_strain":raw_strain,
        "pre_r8_fs":fs_raw,
        "r8_both_sign_fs":r8_fs,
        "candidate_sign_aware_fs":sign_aware_fs,
    })
}

fn r8r1_state_counterfactual(run: &RunResult) -> Value {
    let mesh = &run.final_mesh;
    let rows = (0..mesh.n())
        .map(|index| r8r1_edge_force_counterfactual(mesh, index))
        .collect::<Vec<_>>();
    let compressed = rows
        .iter()
        .filter(|row| row["raw_strain"].as_f64().unwrap() < 0.0)
        .collect::<Vec<_>>();
    let compressed_mixed = compressed
        .iter()
        .copied()
        .filter(|row| row["mature_fraction"].as_f64().unwrap() < 1.0 - 1e-12)
        .collect::<Vec<_>>();
    let raw_compression_norm = compressed
        .iter()
        .map(|row| row["pre_r8_fs"].as_f64().unwrap().abs())
        .sum::<f64>();
    let r8_compression_norm = compressed
        .iter()
        .map(|row| row["r8_both_sign_fs"].as_f64().unwrap().abs())
        .sum::<f64>();
    let candidate_compression_norm = compressed
        .iter()
        .map(|row| row["candidate_sign_aware_fs"].as_f64().unwrap().abs())
        .sum::<f64>();
    let strongest_compression = rows.iter().min_by(|a, b| {
        a["raw_strain"]
            .as_f64()
            .unwrap()
            .total_cmp(&b["raw_strain"].as_f64().unwrap())
    });
    let weakest_compression = compressed.iter().copied().max_by(|a, b| {
        a["raw_strain"]
            .as_f64()
            .unwrap()
            .total_cmp(&b["raw_strain"].as_f64().unwrap())
    });
    let near_zero = rows.iter().min_by(|a, b| {
        a["raw_strain"]
            .as_f64()
            .unwrap()
            .abs()
            .total_cmp(&b["raw_strain"].as_f64().unwrap().abs())
    });
    let strongest_tension = rows.iter().max_by(|a, b| {
        a["raw_strain"]
            .as_f64()
            .unwrap()
            .total_cmp(&b["raw_strain"].as_f64().unwrap())
    });
    let pair = pair_observer(mesh, &FissionParams::default());
    json!({
        "name":run.name,
        "source":"deterministic replay of the frozen R8 passive production-V4 trajectory",
        "terminal_step":run.terminal_step,
        "actual_perimeter":mesh.perimeter(),
        "mature_rest_perimeter":(0..mesh.n()).map(|i|mesh.rest_length(i)).sum::<f64>(),
        "compression_ratio":mesh.perimeter()/((0..mesh.n()).map(|i|mesh.rest_length(i)).sum::<f64>()).max(1e-300),
        "compressed_edges":compressed.len(),
        "compressed_mixed_edges":compressed_mixed.len(),
        "raw_compression_force_l1":raw_compression_norm,
        "r8_compression_force_l1":r8_compression_norm,
        "candidate_compression_force_l1":candidate_compression_norm,
        "r8_over_raw_compression_force":r8_compression_norm/raw_compression_norm.max(1e-300),
        "candidate_over_raw_compression_force":candidate_compression_norm/raw_compression_norm.max(1e-300),
        "minimum_nonadjacent_segment_distance":pair["minimum_nonadjacent_segment_distance"],
        "local_rebond_range":pair["range"],
        "distance_over_range":pair["minimum_distance_over_range"],
        "representative_edges":{
            "strong_compression":strongest_compression,
            "weak_compression":weakest_compression,
            "near_zero":near_zero,
            "strong_tension":strongest_tension,
        },
    })
}

fn r8r1_add_young_counterfactual(runs: &[RunResult]) -> Value {
    let selected = runs
        .iter()
        .flat_map(|run| (0..run.final_mesh.n()).map(move |index| (run, index)))
        .filter(|(run, index)| {
            run.final_mesh.strain(*index) < 0.0
                && run.final_mesh.young_structural_mass(*index) > 0.0
                && run.final_mesh.mature_structural_mass(*index) > 0.0
        })
        .min_by(|(a, ai), (b, bi)| {
            a.final_mesh
                .strain(*ai)
                .total_cmp(&b.final_mesh.strain(*bi))
        })
        .expect("R8 replay must contain a compressed mixed-maturity edge");
    let mesh = &selected.0.final_mesh;
    let index = selected.1;
    let mechanics = MechParams::default();
    let length = mesh.edge_length(index);
    let mature = mesh.mature_structural_mass(index);
    let observed_young = mesh.young_structural_mass(index);
    let rest = (mature / mesh.rho_s.max(1e-15)).max(1e-15);
    let l_ref = rest.max(0.25 * length).max(1e-3);
    let fs_raw =
        (mechanics.k_s * (length - rest) / l_ref).clamp(-mechanics.k_s * 8.0, mechanics.k_s * 8.0);
    let base_fraction = 1.0;
    let added_fraction = mature / (mature + observed_young).max(1e-300);
    json!({
        "source_run":selected.0.name,
        "edge":index,
        "geometry_and_mature_mass_held_fixed":true,
        "length":length,
        "mature_mass":mature,
        "added_young_mass":observed_young,
        "raw_strain":(length-rest)/rest,
        "raw_force_before_and_after":fs_raw,
        "r8_force_before":base_fraction*fs_raw,
        "r8_force_after_adding_observed_young":added_fraction*fs_raw,
        "candidate_force_before":fs_raw,
        "candidate_force_after_adding_observed_young":fs_raw,
        "r8_add_young_force_change":added_fraction*fs_raw-fs_raw,
        "candidate_add_young_force_change":0.0,
    })
}

pub fn run_r8r1_gate1() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r8r1_gate1.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let passive = campaign(Mode::Passive, 14_778);
    let states = passive
        .iter()
        .map(r8r1_state_counterfactual)
        .collect::<Vec<_>>();
    let add_young = r8r1_add_young_counterfactual(&passive);
    let compressed_mixed_edges = states
        .iter()
        .map(|row| row["compressed_mixed_edges"].as_u64().unwrap() as usize)
        .sum::<usize>();
    let attenuation_observed = states.iter().any(|row| {
        row["r8_compression_force_l1"].as_f64().unwrap() + CLASSIFICATION_TOLERANCE
            < row["raw_compression_force_l1"].as_f64().unwrap()
    });
    let add_young_weakens_r8 = add_young["r8_force_after_adding_observed_young"]
        .as_f64()
        .unwrap()
        .abs()
        + CLASSIFICATION_TOLERANCE
        < add_young["r8_force_before"].as_f64().unwrap().abs();
    let candidate_invariant = add_young["candidate_add_young_force_change"]
        .as_f64()
        .unwrap()
        .abs()
        <= CLASSIFICATION_TOLERANCE;
    let supported = compressed_mixed_edges > 0
        && attenuation_observed
        && add_young_weakens_r8
        && candidate_invariant;
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive":"DC-FINAL-001-R8R1-V4-SIGN-AWARE-MATURATION-MECHANICS-REPRODUCTION-AND-END-GOAL-CLOSURE-001",
            "starting_head":"eb198c4e84907d5a7f9f8c170d58a27b56e63bb0",
            "production_mechanics_at_observation":"R8 both-sign mature-fraction scaling (unchanged)",
            "scientific_state_changed":false,
            "new_free_parameters":0,
            "states":states,
            "add_young_counterfactual":add_young,
            "compressed_mixed_edges":compressed_mixed_edges,
            "attenuation_observed":attenuation_observed,
            "adding_young_mass_weakens_existing_compression_under_r8":add_young_weakens_r8,
            "candidate_add_young_invariant":candidate_invariant,
            "classification":if supported {
                "R8_BOTH_SIGN_MATURE_FRACTION_OVERATTENUATES_COMPRESSED_MATURE_SCAFFOLD"
            } else {
                "OVERATTENUATION_NOT_SUPPORTED"
            },
        }))
        .unwrap(),
    )
    .unwrap();
}

pub fn run_r8r1() {
    let mut output = PathBuf::from("/tmp/dcfinal001_r8r1.json");
    let mut gate1_path = PathBuf::from("/tmp/dcfinal001_r8r1_gate1.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
        if args[index] == "--gate1" && index + 1 < args.len() {
            gate1_path = PathBuf::from(&args[index + 1]);
        }
    }
    let gate1: Value = serde_json::from_str(
        &fs::read_to_string(&gate1_path).expect("frozen R8 Gate-1 evidence must exist"),
    )
    .expect("frozen R8 Gate-1 evidence must be valid JSON");
    assert_eq!(
        gate1["classification"],
        "R8_BOTH_SIGN_MATURE_FRACTION_OVERATTENUATES_COMPRESSED_MATURE_SCAFFOLD",
        "R8R1 production execution is forbidden without Gate-1 support"
    );
    let horizon = 14_778;
    let daughter_replays = [
        "r7_seed3_daughter_a.json",
        "r7_seed3_daughter_b.json",
        "r6_normal_seed3_viable_daughter_a.json",
        "r6_normal_seed3_viable_daughter_b.json",
        "r6_normal_tangential_seed3_nonviable_daughter_a.json",
        "r6_normal_tangential_seed3_nonviable_daughter_b.json",
    ]
    .into_iter()
    .map(replay_r8_daughter_fixture)
    .collect::<Vec<_>>();
    let passive = campaign(Mode::Passive, horizon);
    let r5r1 = campaign(Mode::ContrastFallback, horizon);
    let r6_normal = campaign(Mode::CurvatureNormal, horizon);
    let r6_combined = campaign(Mode::CurvatureNormalTangential, horizon);
    let campaigns = [
        (Mode::Passive.label(), &passive),
        (Mode::ContrastFallback.label(), &r5r1),
        (Mode::CurvatureNormal.label(), &r6_normal),
        (Mode::CurvatureNormalTangential.label(), &r6_combined),
    ];
    let counts = campaigns
        .iter()
        .map(|(name, runs)| {
            let (growth, fissions, viable) = result_counts(runs);
            json!({
                "mode":name,
                "growth_qualified":growth,
                "geometry_valid_fissions":fissions,
                "simple_viable_daughter_pairs":viable,
            })
        })
        .collect::<Vec<_>>();
    let robust = counts.iter().any(|row| {
        row["growth_qualified"].as_u64().unwrap() >= 8
            && row["geometry_valid_fissions"].as_u64().unwrap() >= 7
            && row["simple_viable_daughter_pairs"].as_u64().unwrap() >= 6
    });
    let all_daughter_replays_complete = daughter_replays.iter().all(|row| {
        row["corrected_continuation"]["completed_steps"].as_u64() == Some(DAUGHTER_STEPS as u64)
    });
    let closure_cycle_eliminated = daughter_replays.iter().all(|row| {
        row["corrected_continuation"]["cumulative_topology_ruptures"]
            .as_u64()
            .unwrap_or(u64::MAX)
            < 100
            && row["corrected_continuation"]["cumulative_same_edge_rebonds"]
                .as_u64()
                .unwrap_or(u64::MAX)
                < 100
    });
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive":"DC-FINAL-001-R8R1-V4-SIGN-AWARE-MATURATION-MECHANICS-REPRODUCTION-AND-END-GOAL-CLOSURE-001",
            "starting_head":"eb198c4e84907d5a7f9f8c170d58a27b56e63bb0",
            "owner_override":"ACTIVE",
            "qualification_horizon":horizon,
            "new_free_parameters":0,
            "gate1_frozen_counterfactual":gate1,
            "sign_aware_contract":{
                "compressive":"raw strain < 0 => fs_v4 = fs_raw",
                "tensile":"raw strain >= 0 => fs_v4 = mature_fraction * fs_raw",
                "rupture":"unchanged R8 mature_fraction * raw_strain > frozen threshold",
            },
            "daughter_fixture_replays":daughter_replays,
            "all_daughter_replays_complete":all_daughter_replays_complete,
            "r8_closure_cycle_remains_eliminated":closure_cycle_eliminated,
            "campaign_counts":counts,
            "passive_sign_aware_v4":campaign_summary(&passive),
            "r5r1_tangential_sign_aware_v4":campaign_summary(&r5r1),
            "r6_curvature_normal_sign_aware_v4":campaign_summary(&r6_normal),
            "r6_curvature_normal_plus_tangential_sign_aware_v4":campaign_summary(&r6_combined),
            "robust_v4_reproduction":robust,
            "evolution_execution":if robust {"REQUIRED_NEXT_WITHIN_R8R1"} else {"NOT_REACHED_GATE8_STOP"},
            "classification":if robust {
                "V4_SIGN_AWARE_MATURATION_ROBUST_REPRODUCTION_QUALIFIED_PENDING_EVOLUTION"
            } else {
                "V4_SIGN_AWARE_MATURATION_REPRODUCTION_NOT_ESTABLISHED"
            },
        }))
        .unwrap(),
    )
    .unwrap();
}

pub fn run_r8() {
    const R8_DIRECTIVE: &str = "DC-FINAL-001-R8-V4-FISSION-CLOSURE-MATERIAL-AND-LOAD-BEARING-CONSISTENCY-EMERGENCY-CLOSURE-001";
    let mut output = PathBuf::from("/tmp/dcfinal001_r8_v4_closure.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let horizon = 14_778;
    let daughter_replays = [
        "r7_seed3_daughter_a.json",
        "r7_seed3_daughter_b.json",
        "r6_normal_seed3_viable_daughter_a.json",
        "r6_normal_seed3_viable_daughter_b.json",
        "r6_normal_tangential_seed3_nonviable_daughter_a.json",
        "r6_normal_tangential_seed3_nonviable_daughter_b.json",
    ]
    .into_iter()
    .map(replay_r8_daughter_fixture)
    .collect::<Vec<_>>();

    let passive = campaign(Mode::Passive, horizon);
    let r5r1 = campaign(Mode::ContrastFallback, horizon);
    let r6_normal = campaign(Mode::CurvatureNormal, horizon);
    let r6_combined = campaign(Mode::CurvatureNormalTangential, horizon);
    let counts = [
        (Mode::Passive.label(), result_counts(&passive)),
        (Mode::ContrastFallback.label(), result_counts(&r5r1)),
        (Mode::CurvatureNormal.label(), result_counts(&r6_normal)),
        (
            Mode::CurvatureNormalTangential.label(),
            result_counts(&r6_combined),
        ),
    ];
    let robust = counts
        .iter()
        .any(|(_, (growth, fissions, viable))| *growth >= 8 && *fissions >= 7 && *viable >= 6);
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "directive":R8_DIRECTIVE,
            "starting_head":"7363cfaa2c459522583b70d8107f377c08467b76",
            "qualification_horizon":horizon,
            "new_free_parameters":0,
            "daughter_fixture_replays":daughter_replays,
            "campaign_counts":counts.iter().map(|(name,(growth,fissions,viable))|json!({
                "mode":name,
                "growth_qualified":growth,
                "geometry_valid_fissions":fissions,
                "simple_viable_daughter_pairs":viable,
            })).collect::<Vec<_>>(),
            "passive_corrected_v4":campaign_summary(&passive),
            "r5r1_tangential_corrected_v4":campaign_summary(&r5r1),
            "r6_curvature_normal_corrected_v4":campaign_summary(&r6_normal),
            "r6_curvature_normal_plus_tangential_corrected_v4":campaign_summary(&r6_combined),
            "robust_v4_reproduction":robust,
            "classification":if robust {
                "V4_FISSION_CLOSURE_CONSISTENCY_REPAIRED_ROBUST_REPRODUCTION_QUALIFIED"
            } else {
                "V4_FISSION_CLOSURE_CONSISTENCY_REPAIRED_ROBUST_REPRODUCTION_NOT_ESTABLISHED"
            },
        }))
        .unwrap(),
    )
    .unwrap();
}

pub fn run_r7() {
    const R7_DIRECTIVE: &str =
        "DC-FINAL-001-R7-STRAIN-CONTRAST-NORMAL-CONSTRICTION-EMERGENCY-CLOSURE-001";
    let mut output = PathBuf::from("/tmp/dcfinal001_r7_strain_contrast_normal.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let horizon = 14_778;
    let composition = cross_composition_qualification();
    assert_eq!(
        composition["pass"], true,
        "cross-composition contract failed"
    );

    let passive = campaign(Mode::Passive, horizon);
    let tangential = campaign(Mode::ContrastFallback, horizon);
    let curvature_normal = campaign(Mode::CurvatureNormal, horizon);
    let contrast_normal = campaign(Mode::ContrastNormal, horizon);
    let motor_off = campaign(Mode::ContrastNormalMotorOff, horizon);
    let zero_a = campaign(Mode::ContrastNormalZeroA, horizon);

    let (passive_growth, passive_fissions, passive_viable) = result_counts(&passive);
    let (_, tangential_fissions, tangential_viable) = result_counts(&tangential);
    let (_, curvature_fissions, curvature_viable) = result_counts(&curvature_normal);
    let (normal_growth, normal_fissions, normal_viable) = result_counts(&contrast_normal);
    let normal_pass = normal_growth >= 8 && normal_fissions >= 7 && normal_viable >= 6;
    let normal_improves_both = (normal_fissions > passive_fissions
        && normal_fissions > tangential_fissions)
        || (normal_viable > passive_viable && normal_viable > tangential_viable);
    let combined = (!normal_pass && normal_improves_both)
        .then(|| campaign(Mode::ContrastNormalTangential, horizon));
    let combined_counts = combined.as_ref().map(|runs| result_counts(runs));
    let combined_pass = combined_counts
        .map(|(growth, fissions, viable)| growth >= 8 && fissions >= 7 && viable >= 6)
        .unwrap_or(false);
    let reproduction_pass = normal_pass || combined_pass;
    let selected = if normal_pass {
        &contrast_normal
    } else if combined_pass {
        combined.as_ref().unwrap()
    } else {
        &contrast_normal
    };
    let (selected_growth, selected_fissions, selected_viable) = result_counts(selected);
    let energy = active_energy_summary(&contrast_normal);
    let combined_energy = combined.as_ref().map(|runs| active_energy_summary(runs));
    let zero_a_spent = zero_a
        .iter()
        .map(|run| run.attribution.zero_a_spent)
        .sum::<f64>();
    let motor_off_parity = campaign_state_parity(&passive, &motor_off);

    let result = json!({
        "directive": R7_DIRECTIVE,
        "starting_head": "f1bee528888a91a8a2553ecea4973f5a5b851ab9",
        "owner_override": "PASS",
        "qualification_horizon": horizon,
        "cross_composition": composition,
        "force_contract": {
            "drive": "exact R5R1 mean-relative positive tensile-strain contrast",
            "direction": "exact R6 orientation-aware local inward vertex normal",
            "force_scale": "MAX_EXTERNAL_FORCE_PER_VERTEX - FROZEN_STATIC_TRACTION_LIMIT",
            "cost": FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME,
            "primary_tangential_activity": 0,
            "curvature_drive": false,
            "new_free_parameters": 0,
        },
        "campaigns": {
            "passive": campaign_summary(&passive),
            "r5r1_strain_contrast_tangential": campaign_summary(&tangential),
            "r6_curvature_normal": campaign_summary(&curvature_normal),
            "r7_strain_contrast_normal": campaign_summary(&contrast_normal),
            "r7_strain_contrast_normal_motor_off": campaign_summary(&motor_off),
            "r7_strain_contrast_normal_zero_a": campaign_summary(&zero_a),
            "r7_strain_contrast_normal_plus_tangential": combined.as_ref().map(|runs| campaign_summary(runs)),
        },
        "controls": {
            "motor_off_semantic_parity": motor_off_parity,
            "zero_a_spent": zero_a_spent,
            "zero_a_pass": zero_a_spent <= CLASSIFICATION_TOLERANCE,
            "uniform_no_strain_control_pass": composition["uniform_no_strain_max_drive"].as_f64().unwrap_or(f64::INFINITY) <= CLASSIFICATION_TOLERANCE,
        },
        "localization": {
            "r5r1_tangential": functional_summary(&tangential),
            "r7_contrast_normal": functional_summary(&contrast_normal),
            "r6_curvature_normal": functional_summary(&curvature_normal),
        },
        "apposition": {
            "passive": apposition_score(&passive),
            "r5r1_tangential": apposition_score(&tangential),
            "r6_curvature_normal": apposition_score(&curvature_normal),
            "r7_contrast_normal": apposition_score(&contrast_normal),
            "r7_contrast_normal_plus_tangential": combined.as_ref().map(|runs| apposition_score(runs)),
        },
        "energy": {
            "r7_strain_contrast_normal": energy,
            "r7_strain_contrast_normal_plus_tangential": combined_energy,
            "zero_a_spent": zero_a_spent,
        },
        "conditional_same_signal_tangential": {
            "executed": combined.is_some(),
            "normal_only_improved_relative_to_both_passive_and_r5r1": normal_improves_both,
            "normal_only_passed": normal_pass,
            "criterion": "normal-only improves fission count OR viable-pair count relative to both passive and sealed R5R1, while below qualification",
            "counts": combined_counts.map(|(growth,fissions,viable)|json!({"growth":growth,"fissions":fissions,"viable_pairs":viable})),
        },
        "daughter_diagnostics": {
            "passive": daughter_diagnostic_rows(&passive),
            "r5r1_tangential": daughter_diagnostic_rows(&tangential),
            "r6_curvature_normal": daughter_diagnostic_rows(&curvature_normal),
            "r7_contrast_normal": daughter_diagnostic_rows(&contrast_normal),
            "r7_contrast_normal_plus_tangential": combined.as_ref().map(|runs|daughter_diagnostic_rows(runs)),
        },
        "reproduction": {
            "selected": if normal_pass {Mode::ContrastNormal.label()} else if combined_pass {Mode::ContrastNormalTangential.label()} else {Mode::ContrastNormal.label()},
            "growth_qualified": selected_growth,
            "geometry_valid_fissions": selected_fissions,
            "simple_viable_daughter_pairs": selected_viable,
            "pass": reproduction_pass,
            "thresholds": {"growth":8,"fissions":7,"viable_pairs":6},
            "reference_counts": {
                "passive":{"growth":passive_growth,"fissions":passive_fissions,"viable_pairs":passive_viable},
                "r5r1_tangential":{"fissions":tangential_fissions,"viable_pairs":tangential_viable},
                "r6_curvature_normal":{"fissions":curvature_fissions,"viable_pairs":curvature_viable},
                "r7_strain_contrast_normal":{"growth":normal_growth,"fissions":normal_fissions,"viable_pairs":normal_viable},
            },
        },
        "evolution_execution": if reproduction_pass {"REQUIRED_CONTINUE_GATE9"} else {"NOT_REACHED_GATE7_STOP"},
        "classification": if reproduction_pass {"V4_STRAIN_CONTRAST_NORMAL_ROBUST_REPRODUCTION_QUALIFIED_PENDING_EVOLUTION"} else {"V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED"},
        "digital_cell_end_goal": if reproduction_pass {"PENDING_EVOLUTION_AND_FINAL_INTEGRATION"} else {"NOT_ESTABLISHED"},
        "shutdown_recommended": "NO — OWNER OVERRIDE ACTIVE",
        "new_free_parameters": 0,
        "next_execution_started": false,
        "independent_architect_acceptance": "PENDING",
    });
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}

pub fn run_r6() {
    const R6_DIRECTIVE: &str =
        "DC-FINAL-001-R6-CURVATURE-GATED-NORMAL-CONSTRICTION-EMERGENCY-CLOSURE-001";
    let mut output = PathBuf::from("/tmp/dcfinal001_r6_curvature_normal.json");
    let args = env::args().collect::<Vec<_>>();
    for index in 1..args.len() {
        if args[index] == "--output" && index + 1 < args.len() {
            output = PathBuf::from(&args[index + 1]);
        }
    }
    let horizon = 14_778;
    let passive = campaign(Mode::Passive, horizon);
    let tangential = campaign(Mode::ContrastFallback, horizon);
    let inward = tangential
        .iter()
        .map(|run| run.actuator_geometry.inward_normal_component_sum)
        .sum::<f64>();
    let outward = tangential
        .iter()
        .map(|run| run.actuator_geometry.outward_normal_component_sum)
        .sum::<f64>();
    let normal_component = tangential
        .iter()
        .map(|run| run.actuator_geometry.absolute_normal_component_sum)
        .sum::<f64>();
    let tangential_component = tangential
        .iter()
        .map(|run| run.actuator_geometry.absolute_tangential_component_sum)
        .sum::<f64>();
    let inward_fraction = inward / (inward + outward).max(1e-300);
    let normal_fraction = normal_component / (normal_component + tangential_component).max(1e-300);
    let geometry_mismatch = !(inward_fraction > 0.5 && normal_fraction >= 0.5);
    assert!(
        geometry_mismatch,
        "Gate 1 requires stopping before normal-force execution"
    );

    let signal = curvature_signal_qualification();
    assert_eq!(
        signal["pass"], true,
        "curvature signal qualification failed"
    );
    let normal = campaign(Mode::CurvatureNormal, horizon);
    let motor_off = campaign(Mode::CurvatureNormalMotorOff, horizon);
    let zero_a = campaign(Mode::CurvatureNormalZeroA, horizon);
    let tangential_score = apposition_score(&tangential);
    let normal_score = apposition_score(&normal);
    let (_, tangential_fissions, _) = result_counts(&tangential);
    let (normal_growth, normal_fissions, normal_viable) = result_counts(&normal);
    let normal_reproduction = normal_growth >= 8 && normal_fissions >= 7 && normal_viable >= 6;
    let apposition_improved = normal_fissions > tangential_fissions
        || normal_score["within_range_candidate_count_at_attempts"]
            .as_u64()
            .unwrap_or(0)
            > tangential_score["within_range_candidate_count_at_attempts"]
                .as_u64()
                .unwrap_or(0)
        || normal_score["stress_qualified_candidate_count_at_attempts"]
            .as_u64()
            .unwrap_or(0)
            > tangential_score["stress_qualified_candidate_count_at_attempts"]
                .as_u64()
                .unwrap_or(0);
    let combined = (!normal_reproduction && apposition_improved)
        .then(|| campaign(Mode::CurvatureNormalTangential, horizon));
    let selected = if normal_reproduction {
        &normal
    } else {
        combined.as_ref().unwrap_or(&normal)
    };
    let (selected_growth, selected_fissions, selected_viable) = result_counts(selected);
    let reproduction_pass = selected_growth >= 8 && selected_fissions >= 7 && selected_viable >= 6;
    let zero_a_spent = zero_a
        .iter()
        .map(|run| run.attribution.zero_a_spent)
        .sum::<f64>();
    let result = json!({
        "directive":R6_DIRECTIVE,
        "starting_head":"456cc98837842af933d3972b8588707ea801bf28",
        "owner_override":"PASS",
        "qualification_horizon":horizon,
        "tangential_geometry_audit":{
            "classification":"TANGENTIAL_TENSION_GEOMETRY_MISMATCH_SUPPORTED",
            "inward_normal_component_sum":inward,
            "outward_normal_component_sum":outward,
            "absolute_normal_component_sum":normal_component,
            "absolute_tangential_component_sum":tangential_component,
            "inward_fraction":inward_fraction,
            "normal_fraction":normal_fraction,
            "per_arm":tangential.iter().map(|run|json!({"name":run.name,"audit":run.actuator_geometry})).collect::<Vec<_>>(),
        },
        "curvature_signal":signal,
        "normal_force_contract":{
            "direction":"LOCAL_INWARD_VERTEX_NORMAL",
            "drive":"max(concavity, local_neighbor_curvature_deficit)",
            "force_scale":"MAX_EXTERNAL_FORCE_PER_VERTEX - FROZEN_STATIC_TRACTION_LIMIT",
            "force_bound":MAX_EXTERNAL_FORCE_PER_VERTEX,
            "cost":FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME,
            "existing_funded_path":true,
            "new_free_parameters":0,
            "normal_only_tangential_activity":0,
        },
        "campaigns":{
            "passive":campaign_summary(&passive),
            "r5r1_tangential":campaign_summary(&tangential),
            "normal_only":campaign_summary(&normal),
            "normal_motor_off":campaign_summary(&motor_off),
            "normal_zero_a":campaign_summary(&zero_a),
            "normal_plus_tangential":combined.as_ref().map(|runs|campaign_summary(runs)),
        },
        "apposition":{
            "tangential":tangential_score,
            "normal_only":normal_score,
            "materially_improved":apposition_improved,
            "conditional_combined_executed":combined.is_some(),
            "criterion":"greater valid fission count OR greater within-range/stress-qualified candidate count at unchanged official attempts",
        },
        "energy":{
            "normal_only":active_energy_summary(&normal),
            "normal_plus_tangential":combined.as_ref().map(|runs|active_energy_summary(runs)),
            "zero_a_spent":zero_a_spent,
            "zero_a_pass":zero_a_spent <= 1e-12,
        },
        "reproduction":{
            "selected":if normal_reproduction {Mode::CurvatureNormal.label()} else if combined.is_some(){Mode::CurvatureNormalTangential.label()}else{Mode::CurvatureNormal.label()},
            "growth_qualified":selected_growth,
            "geometry_valid_fissions":selected_fissions,
            "simple_viable_daughter_pairs":selected_viable,
            "pass":reproduction_pass,
        },
        "evolution_execution":if reproduction_pass{"REQUIRED_CONTINUE_GATE10"}else{"NOT_REACHED_GATE8_STOP"},
        "classification":if reproduction_pass{"V4_CURVATURE_GATED_NORMAL_CONSTRICTION_ROBUST_REPRODUCTION_QUALIFIED_PENDING_EVOLUTION"}else{"V4_ROBUST_PHYSICAL_REPRODUCTION_NOT_ESTABLISHED"},
        "digital_cell_end_goal":if reproduction_pass{"PENDING_EVOLUTION_AND_FINAL_INTEGRATION"}else{"NOT_ESTABLISHED"},
        "shutdown_recommended":"NO — OWNER OVERRIDE ACTIVE",
        "new_free_parameters":0,
        "next_execution_started":false,
        "independent_architect_acceptance":"PENDING",
    });
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}
