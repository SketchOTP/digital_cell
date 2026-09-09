//! DC-FINAL-001-R5: production-V4 neck-generation diagnosis and bounded
//! existing-mechanism mechanochemical reproduction qualification.
//!
//! This is an assay harness. It does not add a division command, target neck,
//! cleavage axis, body-size controller, or a new physical parameter.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{
    find_local_segment_apposition, topology_step, try_local_fission, try_local_segment_fission,
    FissionParams,
};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step_with_reference_lengths, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_self_contact::{mechanics_step_with_local_self_contact, polygon_simple};
use chemistry_core::mesh_topology::{find_local_pinch, local_rebond_range};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use chemistry_core::planar_ring_topology::{remesh_preserving_simple, PlanarRingTopology};
use regulatory_core::continuity::{ContinuityNetworkV1, TopologyEventV1};
use regulatory_core::contractility::{
    apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact,
    ContractilityParamsV1,
};
use regulatory_core::material_adapter::observe_continuity_material_frame;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum Mode {
    Passive,
    RegulatorMotor,
    RegulatorOffMotorOnZero,
    RegulatorOnMotorOff,
    ZeroA,
    ContrastFallback,
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
        }
    }
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
    final_mesh: MaterialMesh,
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
        "all_simple": all_simple, "all_runtime_valid": all_runtime,
        "all_lifecycle_invariants_hold": all_lifecycle,
        "c_retention": c_retention, "a_retention": a_retention,
        "terminal": geometry(&mesh),
    })
}

fn contrast_activity(mesh: &MaterialMesh) -> Vec<f64> {
    let frame = observe_continuity_material_frame(mesh, &MechParams::default());
    let perimeter = mesh.perimeter().max(1e-300);
    let mean = frame
        .patches
        .iter()
        .enumerate()
        .map(|(i, p)| {
            p.raw_stimulus
                * 0.5
                * (mesh.edge_length((i + mesh.n() - 1) % mesh.n()) + mesh.edge_length(i))
        })
        .sum::<f64>()
        / perimeter;
    frame
        .patches
        .iter()
        .map(|p| (p.raw_stimulus - mean).max(0.0))
        .collect()
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
    let need = candidate_distance.map(|d| mesh.rho_s * d);
    let have_a = mesh.interior.a.max(0.0) * mesh.area().max(1e-6);
    let reason = if !distance.is_finite() {
        "NO_NONADJACENT_APPOSITION"
    } else if within == 0 || distance > range {
        "APPOSITION_OUTSIDE_LOCAL_RANGE"
    } else if stressed == 0 {
        "APPOSITION_PRESENT_BUT_STRESS_CONDITION_FALSE"
    } else if need.map(|x| have_a + 1e-12 < x).unwrap_or(false) {
        "PINCH_PRESENT_INSUFFICIENT_A"
    } else {
        "SCISSION_CANDIDATE_PRESENT"
    };
    (
        reason.to_string(),
        json!({
            "pairs": pairs, "candidate_distance": candidate_distance,
            "absolute_a": have_a, "cross_bond_a_required": need,
            "a_sufficient": need.map(|x| have_a + 1e-12 >= x),
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

fn run(
    initial_mesh: MaterialMesh,
    name: &str,
    mode: Mode,
    start_step: usize,
    end_step: usize,
    birth_mass: f64,
) -> RunResult {
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let contractility = ContractilityParamsV1::default();
    let mut mesh = initial_mesh;
    let initial_frame = observe_continuity_material_frame(&mesh, &mechanics);
    let mut regulator = ContinuityNetworkV1::new(initial_frame, Some(0)).unwrap();
    let mut maximum_mass_ratio = mesh.total_structural_mass() / birth_mass.max(1e-300);
    let mut all_simple = polygon_simple(&mesh.vertices);
    let mut all_runtime = mesh.physical_runtime_valid();
    let mut all_lifecycle = mesh.lifecycle_invariants_hold();
    let mut first_invalid = None;
    let mut fission_step = None;
    let mut both_viable = false;
    let mut attempts = Vec::new();
    let mut checkpoints = Vec::new();
    let mut counts = FailureCounts::default();
    let mut attribution = Attribution::default();
    let mut deepest = "NO_NONADJACENT_APPOSITION".to_string();
    for absolute_step in (start_step + 1)..=end_step {
        if !mesh.can_advance_physics() {
            first_invalid = Some(json!({"step":absolute_step,"phase":"pre_step"}));
            break;
        }
        let _ = transport_step(&mut mesh, &transport, mechanics.dt);
        let _ = reactions_step(&mut mesh, &reaction, mechanics.dt, true, true);
        let _ = growth_step(&mut mesh, &reaction, &growth, mechanics.dt);

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
            Mode::ContrastFallback => contrast_activity(&mesh),
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
        let (activity_mean_current, _) = mean_variance(&activity);
        if regulator_on || mode == Mode::ContrastFallback {
            update_attribution(&mut attribution, &mesh, &activity);
        }

        let mechanics_ok = match mode {
            Mode::Passive | Mode::RegulatorOnMotorOff => {
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
            Mode::ZeroA => {
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
        };
        if !mechanics_ok {
            first_invalid = Some(json!({"step":absolute_step,"phase":"mechanics"}));
            break;
        }
        let _ = remesh_preserving_simple(&mut mesh);
        let planar = PlanarRingTopology::from_mesh(&mesh);
        if absolute_step.saturating_sub(1) % 10 == 0 {
            let _ = topology_step(&mut mesh, &fission);
        }
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
                "rupture_count": mesh.edges.iter().filter(|e|e.ruptured).count(),
                "pair_observer": pair.as_ref().unwrap(), "absolute_a": mesh.interior.a.max(0.0)*mesh.area(),
                "simple": true, "runtime_valid": true,
            }));
        }
        if attempt_tick {
            let (mut reason, detail) = classify_attempt(&mesh, &fission);
            let proposed = try_local_fission(&mesh, &fission).or_else(|| {
                planar
                    .as_ref()
                    .and_then(|p| p.try_local_scission(&mesh, &fission))
                    .or_else(|| try_local_segment_fission(&mesh, &fission))
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
                    let va = daughter_viability(a);
                    let vb = daughter_viability(b);
                    both_viable = va["viable"] == true && vb["viable"] == true;
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
    RunResult {
        name: name.into(),
        mode,
        start_step,
        terminal_step: fission_step.unwrap_or(end_step),
        birth_mass,
        max_mass_over_birth: maximum_mass_ratio,
        physical_fission: fission_step.is_some(),
        both_daughters_viable: both_viable,
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
        final_mesh: mesh,
    }
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
