//! DC-DEV-021 ENTRY-002: observer-only temporal-navigation substrate audit.
//!
//! This program neither installs a navigation mechanism nor changes any
//! production component.  It replays the frozen DC-DEV-013 causal order with
//! the already-qualified opt-in A-funded actuator, then observes whether an
//! unforced reserve-OFF V4 body produces exploratory motion on its own.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use regulatory_core::{
    apply_local_activated_energy_contractility_with_stick_slip,
    apply_stick_slip_to_legacy_mechanics, stable_json_hash, ContinuityMaterialFrameV1,
    ContinuityNetworkV1, ContractilityParamsV1, FiniteSpatialResourceRegionV1, PlasticityParamsV1,
    PlasticityStateV1, StickSlipTractionParamsV1, TopologyEventV1, FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-021-M2-ENTRY-002-TEMPORAL-NAVIGATION-SUBSTRATE-AUDIT-001";
const ENTRY_HEAD: &str = "54f3af09804a9accd845dfcae2dfce13d1918b7c";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 480;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const INITIAL_N_MASS: f64 = 3.0;
const INITIAL_F_MASS: f64 = 3.0;
const MASS_TOLERANCE: f64 = 1e-10;
const MIN_RELATIVE_IMPROVEMENT: f64 = 0.10;

#[derive(Clone, Copy)]
enum DirectArm {
    ActiveContact,
    SensorOffAFunded,
    MotorOff,
    ZeroA,
    EmptySham,
}

impl DirectArm {
    fn label(self) -> &'static str {
        match self {
            Self::ActiveContact => "A_ACTIVE_CONTACT",
            Self::SensorOffAFunded => "SENSOR_OFF_A_FUNDED",
            Self::MotorOff => "MOTOR_OFF",
            Self::ZeroA => "ZERO_A",
            Self::EmptySham => "EMPTY_SHAM",
        }
    }

    fn sensor_enabled(self) -> bool {
        !matches!(self, Self::SensorOffAFunded)
    }

    fn motor_enabled(self) -> bool {
        !matches!(self, Self::MotorOff)
    }

    fn zero_a(self) -> bool {
        matches!(self, Self::ZeroA)
    }

    fn resource_present(self) -> bool {
        !matches!(self, Self::EmptySham)
    }
}

#[derive(Debug, Clone, Serialize)]
struct DirectArmResult {
    arm: String,
    cumulative_acquisition: f64,
    n_delivered: f64,
    f_delivered: f64,
    contact_duration_steps: usize,
    time_integrated_exposed_patches: f64,
    maximum_funded_tension: f64,
    activated_spent: f64,
    reserve_before: f64,
    reserve_after: f64,
    substrate_work: f64,
    stuck_contacts: usize,
    slipping_contacts: usize,
    material_centroid_displacement: f64,
    vertex_centroid_displacement: f64,
    material_vertex_displacement_agreement: f64,
    maximum_conservation_error: f64,
    final_mesh_hash: String,
    contact_signal_range: TraceRange,
    uptake_range: TraceRange,
    maximum_regulatory_activity: f64,
    #[serde(skip_serializing)]
    signal_total_trace: Vec<f64>,
    #[serde(skip_serializing)]
    uptake_trace: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
struct TraceRange {
    min: f64,
    max: f64,
    max_step_delta: f64,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn norm(value: [f64; 2]) -> f64 {
    value[0].hypot(value[1])
}

fn subtract(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn material_centroid(mesh: &MaterialMesh) -> [f64; 2] {
    let mut weighted = [0.0, 0.0];
    let mut total = 0.0;
    for i in 0..mesh.n() {
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % mesh.n()];
        let weight = (mesh.edges[i].m + mesh.edges[i].b).max(0.0);
        let midpoint = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
        weighted[0] += weight * midpoint[0];
        weighted[1] += weight * midpoint[1];
        total += weight;
    }
    if total <= f64::EPSILON {
        mesh.centroid()
    } else {
        [weighted[0] / total, weighted[1] / total]
    }
}

fn seed_v4_mesh() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        TOPOLOGY_SIZE,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            n: 0.4,
            f: 0.4,
            r: 0.0,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    mesh
}

fn settle_body(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed_v4_mesh();
    for _ in 0..SETTLEMENT_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    assert!(mesh.lifecycle_invariants_hold());
    assert!(mesh.interior.r == 0.0);
    mesh
}

fn transport() -> chemistry_core::mesh_transport::TransportParams {
    chemistry_core::mesh_transport::TransportParams::default()
}

fn run_direct_arm(
    settled: &MaterialMesh,
    arm: DirectArm,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> DirectArmResult {
    let mut mesh = settled.clone();
    if arm.zero_a() {
        mesh.interior.a = 0.0;
    }
    let reserve_before = mesh.interior.r;
    let initial_material_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        if arm.resource_present() {
            INITIAL_N_MASS
        } else {
            0.0
        },
        if arm.resource_present() {
            INITIAL_F_MASS
        } else {
            0.0
        },
    );
    let initial_signal = if arm.sensor_enabled() {
        region.local_contact_signal(&mesh)
    } else {
        vec![0.0; mesh.n()]
    };
    let initial_frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
        &mesh.vertices,
        &initial_signal,
        mechanics.dt,
    );
    let mut network = ContinuityNetworkV1::new(initial_frame, Some(21002)).unwrap();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut activated_spent = 0.0;
    let mut max_tension = 0.0_f64;
    let mut contact_duration_steps = 0;
    let mut integrated_exposed = 0.0;
    let mut substrate_work = 0.0;
    let mut stuck_contacts = 0;
    let mut slipping_contacts = 0;
    let mut max_conservation_error = 0.0_f64;
    let mut signal_total_trace = Vec::with_capacity(ASSAY_STEPS);
    let mut uptake_trace = Vec::with_capacity(ASSAY_STEPS);
    let mut activity_max = 0.0_f64;

    for _ in 0..ASSAY_STEPS {
        // The frozen DC-DEV-013 timing: observe, regulate, act/advance, uptake.
        let signal = if arm.sensor_enabled() {
            region.local_contact_signal(&mesh)
        } else {
            vec![0.0; mesh.n()]
        };
        let exposed = signal.iter().filter(|value| **value > 0.0).count();
        if exposed > 0 {
            contact_duration_steps += 1;
        }
        integrated_exposed += exposed as f64 * mechanics.dt;
        signal_total_trace.push(signal.iter().sum());
        let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &mesh.vertices,
            &signal,
            mechanics.dt,
        );
        network.step(frame, TopologyEventV1::Stable).unwrap();
        activity_max = activity_max.max(network.state.activity.iter().copied().fold(0.0, f64::max));

        let motion = if arm.motor_enabled() {
            apply_local_activated_energy_contractility_with_stick_slip(
                &mut mesh,
                &network.state.activity,
                mechanics,
                contractility,
                traction,
            )
            .unwrap()
        } else {
            let passive =
                apply_stick_slip_to_legacy_mechanics(&mut mesh, mechanics, traction).unwrap();
            regulatory_core::ActivatedEnergyStickSlipStepLedgerV1 {
                schema: passive.schema,
                contacts: passive.contacts,
                maximum_stick_reaction: passive.maximum_stick_reaction,
                maximum_slip_reaction: passive.maximum_slip_reaction,
                stuck_contacts: passive.stuck_contacts,
                slipping_contacts: passive.slipping_contacts,
                substrate_work: passive.substrate_work,
                maximum_attempted_velocity: passive.maximum_attempted_velocity,
                maximum_accepted_velocity: passive.maximum_accepted_velocity,
                contractility: None,
            }
        };
        stuck_contacts += motion.stuck_contacts;
        slipping_contacts += motion.slipping_contacts;
        substrate_work += motion.substrate_work;
        if let Some(ledger) = motion.contractility {
            activated_spent += ledger.resource_spent;
            max_tension = max_tension.max(ledger.maximum_tension);
            assert_eq!(ledger.reserve_before, ledger.reserve_after);
        }
        let resource = region.uptake(&mut mesh, &transport(), mechanics.dt);
        n_delivered += resource.n_delivered;
        f_delivered += resource.f_delivered;
        uptake_trace.push(resource.n_delivered + resource.f_delivered);
        max_conservation_error = max_conservation_error.max(resource.conservation_error);
        assert!(resource.conservation_error <= MASS_TOLERANCE);
        assert!(mesh.lifecycle_invariants_hold());
        assert_eq!(mesh.interior.r, reserve_before);
    }

    let final_material_centroid = material_centroid(&mesh);
    let final_vertex_centroid = mesh.centroid();
    let material_displacement = norm(subtract(final_material_centroid, initial_material_centroid));
    let vertex_displacement = norm(subtract(final_vertex_centroid, initial_vertex_centroid));
    DirectArmResult {
        arm: arm.label().to_string(),
        cumulative_acquisition: n_delivered + f_delivered,
        n_delivered,
        f_delivered,
        contact_duration_steps,
        time_integrated_exposed_patches: integrated_exposed,
        maximum_funded_tension: max_tension,
        activated_spent,
        reserve_before,
        reserve_after: mesh.interior.r,
        substrate_work,
        stuck_contacts,
        slipping_contacts,
        material_centroid_displacement: material_displacement,
        vertex_centroid_displacement: vertex_displacement,
        material_vertex_displacement_agreement: (material_displacement - vertex_displacement).abs(),
        maximum_conservation_error: max_conservation_error,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        contact_signal_range: trace_range(&signal_total_trace),
        uptake_range: trace_range(&uptake_trace),
        maximum_regulatory_activity: activity_max,
        signal_total_trace,
        uptake_trace,
    }
}

fn trace_range(trace: &[f64]) -> TraceRange {
    let min = trace.iter().copied().fold(f64::INFINITY, f64::min);
    let max = trace.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let max_delta = trace
        .windows(2)
        .map(|values| (values[1] - values[0]).abs())
        .fold(0.0_f64, f64::max);
    TraceRange {
        min,
        max,
        max_step_delta: max_delta,
    }
}

fn run_exploration(
    settled: &MaterialMesh,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> Value {
    let mut mesh = settled.clone();
    let initial_centroid = material_centroid(&mesh);
    let initial_vertex_centroid = mesh.centroid();
    let zero_signal = vec![0.0; mesh.n()];
    let initial_frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
        &mesh.vertices,
        &zero_signal,
        mechanics.dt,
    );
    let mut network = ContinuityNetworkV1::new(initial_frame, Some(21002)).unwrap();
    let plasticity = PlasticityStateV1::new(mesh.n());
    let plasticity_params = PlasticityParamsV1::default();
    let mut path_length = 0.0;
    let mut reorientation_events = 0;
    let mut previous_heading: Option<[f64; 2]> = None;
    let mut heading_pairs = 0;
    let mut heading_dot_sum = 0.0;
    let mut a_spent = 0.0;
    let mut max_tension = 0.0_f64;
    let mut max_activity = 0.0_f64;
    let mut accepted_steps = 0;

    for _ in 0..ASSAY_STEPS {
        let frame = ContinuityMaterialFrameV1::from_positions_and_stimuli(
            &mesh.vertices,
            &zero_signal,
            mechanics.dt,
        );
        network.step(frame, TopologyEventV1::Stable).unwrap();
        max_activity = max_activity.max(network.state.activity.iter().copied().fold(0.0, f64::max));
        let before = material_centroid(&mesh);
        let ledger = apply_local_activated_energy_contractility_with_stick_slip(
            &mut mesh,
            &network.state.activity,
            mechanics,
            contractility,
            traction,
        )
        .unwrap();
        if let Some(contractility) = ledger.contractility {
            a_spent += contractility.resource_spent;
            max_tension = max_tension.max(contractility.maximum_tension);
        }
        let after = material_centroid(&mesh);
        let delta = subtract(after, before);
        let distance = norm(delta);
        path_length += distance;
        if distance > FROZEN_ZERO_MOTION_TOLERANCE {
            let heading = [delta[0] / distance, delta[1] / distance];
            if let Some(previous) = previous_heading {
                let dot = previous[0] * heading[0] + previous[1] * heading[1];
                heading_pairs += 1;
                heading_dot_sum += dot;
                if dot < 0.0 {
                    reorientation_events += 1;
                }
            }
            previous_heading = Some(heading);
        }
        accepted_steps += 1;
        assert!(mesh.lifecycle_invariants_hold());
    }
    json!({
        "resource_patch_present": false,
        "forced_activity_pattern": false,
        "net_displacement": norm(subtract(material_centroid(&mesh), initial_centroid)),
        "vertex_net_displacement": norm(subtract(mesh.centroid(), initial_vertex_centroid)),
        "path_length": path_length,
        "heading_autocorrelation": if heading_pairs == 0 { Value::Null } else { json!(heading_dot_sum / heading_pairs as f64) },
        "reorientation_events": reorientation_events,
        "maximum_regulatory_activity": max_activity,
        "activated_spent": a_spent,
        "maximum_active_tension": max_tension,
        "accepted_mechanics_steps": accepted_steps,
        "plasticity_state_exists_but_is_not_composed_with_a_funded_actuator": plasticity.enabled && plasticity_params.schema == regulatory_core::PLASTICITY_SCHEMA_V1,
        "classification": if path_length > FROZEN_ZERO_MOTION_TOLERANCE { "ESTABLISHED" } else { "NOT_ESTABLISHED" }
    })
}

fn source_hash(path: &str) -> String {
    stable_json_hash(&fs::read(path).unwrap()).unwrap()
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry002"));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settled = settle_body(&mechanics);

    let active = run_direct_arm(
        &settled,
        DirectArm::ActiveContact,
        &mechanics,
        &contractility,
        &traction,
    );
    let sensor_off = run_direct_arm(
        &settled,
        DirectArm::SensorOffAFunded,
        &mechanics,
        &contractility,
        &traction,
    );
    let motor_off = run_direct_arm(
        &settled,
        DirectArm::MotorOff,
        &mechanics,
        &contractility,
        &traction,
    );
    let zero_a = run_direct_arm(
        &settled,
        DirectArm::ZeroA,
        &mechanics,
        &contractility,
        &traction,
    );
    let empty_sham = run_direct_arm(
        &settled,
        DirectArm::EmptySham,
        &mechanics,
        &contractility,
        &traction,
    );
    let exploration = run_exploration(&settled, &mechanics, &contractility, &traction);

    let active_sensor_improvement = (active.cumulative_acquisition
        - sensor_off.cumulative_acquisition)
        / sensor_off.cumulative_acquisition.max(f64::EPSILON);
    let active_motor_improvement = (active.cumulative_acquisition
        - motor_off.cumulative_acquisition)
        / motor_off.cumulative_acquisition.max(f64::EPSILON);
    let direct_benefit = active_sensor_improvement >= MIN_RELATIVE_IMPROVEMENT
        && active_motor_improvement >= MIN_RELATIVE_IMPROVEMENT;
    let exploration_established = exploration["classification"] == "ESTABLISHED";
    let signal_variation = active.contact_signal_range.clone();
    let uptake_variation = active.uptake_range.clone();
    let signal_changes_before_actuation = signal_variation.max_step_delta > 0.0;
    let classification = if direct_benefit {
        "M2_INSTANTANEOUS_CONTACT_ROUTE_REOPENED"
    } else if !exploration_established {
        "M2_TEMPORAL_NAVIGATION_EXPLORATION_SUBSTRATE_INSUFFICIENT"
    } else if !signal_changes_before_actuation {
        "M2_TEMPORAL_NAVIGATION_CONTACT_SIGNAL_INSUFFICIENT"
    } else {
        "M2_TEMPORAL_NAVIGATION_MEMORY_EXTENSION_REQUIRED"
    };

    let source_hashes = json!({
        "regulator": source_hash("crates/regulatory-core/src/lib.rs"),
        "plasticity": source_hash("crates/regulatory-core/src/plasticity.rs"),
        "resource_contact": source_hash("crates/regulatory-core/src/spatial_resource.rs"),
        "contractility": source_hash("crates/regulatory-core/src/contractility.rs"),
        "traction": source_hash("crates/regulatory-core/src/stick_slip_traction.rs")
    });
    if let Some(dense_root) = std::env::var_os("DCDEV021_ENTRY002_DENSE_ROOT") {
        let dense_root = PathBuf::from(dense_root);
        write_json(
            &dense_root,
            "a_funded_direct_contact_per_step.json",
            &json!({
                "arm": active.arm,
                "contact_signal_total": active.signal_total_trace,
                "uptake": active.uptake_trace,
                "steps": ASSAY_STEPS
            }),
        );
    }
    let direct_results = json!({
        "A_ACTIVE_CONTACT": active,
        "SENSOR_OFF_A_FUNDED": sensor_off,
        "MOTOR_OFF": motor_off,
        "ZERO_A": zero_a,
        "EMPTY_SHAM": empty_sham
    });
    let protocol = json!({
        "directive": DIRECTIVE,
        "entry_head": ENTRY_HEAD,
        "production_contract": "MaturationCoupledV4",
        "production_reserve": "OFF",
        "dcdev013_environment": {"center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "n_mass": INITIAL_N_MASS, "f_mass": INITIAL_F_MASS, "settlement_steps": SETTLEMENT_STEPS, "assay_steps": ASSAY_STEPS},
        "causal_order": ["observe_local_contact", "advance_existing_regulator", "apply_opt_in_a_funded_contractility_and_existing_stick_slip", "execute_existing_local_uptake"],
        "observer_only": true,
        "forbidden": ["target_coordinate", "gradient_vector", "new_sensor", "new_memory", "navigation_policy", "resource_seeking"]
    });
    let historical_forensic = json!({
        "historical_classification": "DCDEV013_RESOURCE_CONTACT_FEEDING_NOT_ESTABLISHED",
        "historical_active_acquisition": 0.35420146800801444_f64,
        "historical_sensor_off_acquisition": 0.3640975515105316_f64,
        "historical_motor_off_acquisition": 0.3640975515105316_f64,
        "historical_active_relative_difference": -0.027179758450616476_f64,
        "failure_mechanism": "instantaneous binary contact held two exposed patches for the entire frozen horizon; it raised local activity and reserve-funded tension but did not increase contact duration or exposed-patch integral, while reducing acquisition by 2.7 percent versus both matched controls",
        "causal_order": protocol["causal_order"]
    });
    let temporal_observability = json!({
        "contact_signal_before_actuation": true,
        "contact_signal_range": signal_variation,
        "uptake_occurs_after_actuation": true,
        "uptake_range": uptake_variation,
        "causally_available_before_actuation": signal_changes_before_actuation,
        "interpretation": "the physical contact signal is available before actuation, but under this frozen replay it is binary and does not vary between accepted steps; uptake varies only after the action/uptake boundary and cannot supply a same-step temporal comparison"
    });
    let history_and_control = json!({
        "existing_history_states": [
            {"name": "ContinuityNetworkV1.state.activity", "local": true, "input": "current local contact signal", "update": "neighbor diffusion plus stimulus integration minus decay", "remesh_continuity": true, "resource_affects_state": true, "influences_a_funded_actuator": true, "recent_vs_current_comparator": false},
            {"name": "PlasticityStateV1.adaptation", "local": true, "input": "local regulatory activity and accepted dt", "update": "slow load/recovery", "remesh_continuity": true, "resource_affects_state_indirectly": true, "influences_historical_r_funded_actuator": true, "influences_a_funded_actuator": false, "recent_vs_current_comparator": false}
        ],
        "existing_history_state_sufficient": false,
        "nondirectional_persistence_or_reorientation_control": "ABSENT",
        "reason": "current regulatory output changes local present-edge tension magnitude. It contains no heading, persistence, reorientation, or temporal-comparison control dimension, and plasticity is not composed with the A-funded actuator."
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "classification": classification,
        "direct_contact_route_benefit": direct_benefit,
        "resource_independent_exploration": exploration["classification"],
        "temporal_resource_variation": if signal_changes_before_actuation { "PRESENT" } else { "ABSENT" },
        "causally_available_before_actuation": signal_changes_before_actuation,
        "existing_history_state_sufficient": false,
        "nondirectional_persistence_reorientation_control": "ABSENT",
        "scientific_source_changed": false,
        "production_behavior_changed": false,
        "new_memory_implemented": false,
        "new_sensor_implemented": false,
        "new_navigation_policy_implemented": false,
        "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED"
    });

    write_json(&output, "protocol.json", &protocol);
    write_json(&output, "source_hashes.json", &source_hashes);
    write_json(&output, "dcdev013_forensic.json", &historical_forensic);
    write_json(
        &output,
        "a_funded_direct_contact_replay.json",
        &direct_results,
    );
    write_json(
        &output,
        "resource_independent_exploration.json",
        &exploration,
    );
    write_json(
        &output,
        "temporal_resource_observability.json",
        &temporal_observability,
    );
    write_json(
        &output,
        "history_and_control_inventory.json",
        &history_and_control,
    );
    write_json(&output, "qualification.json", &qualification);
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({
            "directive": DIRECTIVE,
            "entry_head": ENTRY_HEAD,
            "classification": classification,
            "compact_evidence": [
                "protocol.json",
                "source_hashes.json",
                "dcdev013_forensic.json",
                "a_funded_direct_contact_replay.json",
                "resource_independent_exploration.json",
                "temporal_resource_observability.json",
                "history_and_control_inventory.json",
                "qualification.json"
            ],
            "dense_trajectory_ledgers_committed": false,
            "dense_evidence_root": "/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev021m2entry002/"
        }),
    );

    println!("{}", qualification["classification"].as_str().unwrap());
}
