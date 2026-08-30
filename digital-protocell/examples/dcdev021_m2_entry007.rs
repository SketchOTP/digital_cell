//! DC-DEV-021 ENTRY-007: observer-only uptake-degradation mechanism audit.
//!
//! This assay replays the accepted ENTRY-006 fixture for the unguided,
//! ENTRY-003, and motor-off arms.  It records the unchanged DC-DEV-008
//! uptake terms per step and exposed edge.  No observation is supplied to an
//! organism and no uptake, motor, exploration, or geometry law is changed.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    apply_intrinsic_exploration_refractory_motor_with_stick_slip,
    apply_intrinsic_exploration_with_stick_slip, apply_stick_slip_to_legacy_mechanics,
    stable_json_hash, ContractilityParamsV1, FiniteSpatialResourceRegionV1,
    IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1,
    SpatialResourceEdgeUptakeDiagnosticV1, StickSlipTractionParamsV1,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-021-M2-ENTRY-007-UPTAKE-DEGRADATION-MECHANISM-AUDIT-001";
const STARTING_HEAD: &str = "6bfc4839b68e328bab7d89f896dd575fabb5baa7";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 480;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const INITIAL_N_MASS: f64 = 3.0;
const INITIAL_F_MASS: f64 = 3.0;
const EXPLORATION_SEED: u64 = 1;

#[derive(Clone, Copy, Debug)]
enum Arm {
    UnguidedExplorer,
    Entry003PinnedControl,
    MotorOffControl,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::UnguidedExplorer => "UNGUIDED_EXPLORER",
            Self::Entry003PinnedControl => "ENTRY003_PINNED_CONTROL",
            Self::MotorOffControl => "MOTOR_OFF_CONTROL",
        }
    }
}

#[derive(Debug, Serialize)]
struct StepDiagnostic {
    step: usize,
    area: f64,
    exposed_intact_edges: usize,
    n_requested: f64,
    f_requested: f64,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    conservation_error: f64,
    edges: Vec<SpatialResourceEdgeUptakeDiagnosticV1>,
}

#[derive(Debug, Serialize)]
struct ArmResult {
    arm: String,
    seed: u64,
    steps: usize,
    initial_area: f64,
    final_area: f64,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    world_n_remaining: f64,
    world_f_remaining: f64,
    maximum_conservation_error: f64,
    time_integrated_exposed_segment_length: f64,
    final_exposed_intact_edges: usize,
    diagnostics_hash: String,
    diagnostics: Vec<StepDiagnostic>,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn seed_mesh() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        TOPOLOGY_SIZE,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.6,
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

fn settled_body(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed_mesh();
    for _ in 0..SETTLEMENT_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    assert!(mesh.area().is_finite() && mesh.area() > 0.0 && mesh.lifecycle_invariants_hold());
    assert_eq!(mesh.interior.r, 0.0);
    mesh
}

fn run_arm(
    settled: &MaterialMesh,
    arm: Arm,
    mechanics: &MechParams,
    contractility: &ContractilityParamsV1,
    traction: &StickSlipTractionParamsV1,
) -> ArmResult {
    let mut mesh = settled.clone();
    let initial_area = mesh.area();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        INITIAL_N_MASS,
        INITIAL_F_MASS,
    );
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(EXPLORATION_SEED)).unwrap();
    let transport = TransportParams::default();
    let mut diagnostics = Vec::with_capacity(ASSAY_STEPS);
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut n_world_loss = 0.0;
    let mut f_world_loss = 0.0;
    let mut maximum_conservation_error = 0.0_f64;
    let mut time_integrated_exposed_segment_length = 0.0;

    for step in 0..ASSAY_STEPS {
        match arm {
            Arm::UnguidedExplorer => {
                apply_intrinsic_exploration_refractory_motor_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
            }
            Arm::Entry003PinnedControl => {
                apply_intrinsic_exploration_with_stick_slip(
                    &mut mesh,
                    &mut state,
                    mechanics,
                    contractility,
                    traction,
                )
                .unwrap();
            }
            Arm::MotorOffControl => {
                let proposal = regulatory_core::propose_intrinsic_exploration_step(
                    &state,
                    mesh.n(),
                    mechanics.dt,
                    IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
                )
                .unwrap();
                apply_stick_slip_to_legacy_mechanics(&mut mesh, mechanics, traction).unwrap();
                regulatory_core::commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
            }
        }
        assert!(mesh.lifecycle_invariants_hold());

        // The diagnostic is read-only and is deliberately taken after the
        // existing arm advances, immediately before production uptake.  It
        // cannot affect this arm.
        let diagnostic = region.uptake_diagnostic(&mesh, &transport, mechanics.dt);
        let exposed_edges = diagnostic
            .edges
            .iter()
            .filter(|edge| edge.exposed && !edge.ruptured)
            .count();
        time_integrated_exposed_segment_length += diagnostic
            .edges
            .iter()
            .filter(|edge| edge.exposed && !edge.ruptured)
            .map(|edge| edge.segment_length)
            .sum::<f64>()
            * mechanics.dt;

        // This is the unchanged DC-DEV-008 world boundary.  Entry-007 only
        // records its returned ledger and never replaces this operation.
        let ledger = region.uptake(&mut mesh, &transport, mechanics.dt);
        assert!((diagnostic.n_delivered - ledger.n_delivered).abs() <= 1e-15);
        assert!((diagnostic.f_delivered - ledger.f_delivered).abs() <= 1e-15);
        let exposed_diagnostics = diagnostic
            .edges
            .into_iter()
            .filter(|edge| edge.exposed && !edge.ruptured)
            .collect::<Vec<_>>();
        diagnostics.push(StepDiagnostic {
            step,
            area: diagnostic.area,
            exposed_intact_edges: exposed_edges,
            n_requested: diagnostic.n_requested,
            f_requested: diagnostic.f_requested,
            n_delivered: diagnostic.n_delivered,
            f_delivered: diagnostic.f_delivered,
            n_world_loss: ledger.n_world_loss,
            f_world_loss: ledger.f_world_loss,
            conservation_error: ledger.conservation_error,
            edges: exposed_diagnostics,
        });
        n_delivered += ledger.n_delivered;
        f_delivered += ledger.f_delivered;
        n_world_loss += ledger.n_world_loss;
        f_world_loss += ledger.f_world_loss;
        maximum_conservation_error = maximum_conservation_error.max(ledger.conservation_error);
    }

    let final_exposed_intact_edges = region
        .uptake_diagnostic(&mesh, &transport, mechanics.dt)
        .edges
        .iter()
        .filter(|edge| edge.exposed && !edge.ruptured)
        .count();
    let diagnostics_hash = stable_json_hash(&diagnostics).unwrap();
    ArmResult {
        arm: arm.label().to_string(),
        seed: EXPLORATION_SEED,
        steps: ASSAY_STEPS,
        initial_area,
        final_area: mesh.area(),
        n_delivered,
        f_delivered,
        n_world_loss,
        f_world_loss,
        world_n_remaining: region.n_mass,
        world_f_remaining: region.f_mass,
        maximum_conservation_error,
        time_integrated_exposed_segment_length,
        final_exposed_intact_edges,
        diagnostics_hash,
        diagnostics,
    }
}

fn source_hash(relative: &str) -> String {
    stable_json_hash(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join(relative),
        )
        .unwrap(),
    )
    .unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry007"));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let settled = settled_body(&mechanics);
    let unguided = run_arm(
        &settled,
        Arm::UnguidedExplorer,
        &mechanics,
        &contractility,
        &traction,
    );
    let entry003 = run_arm(
        &settled,
        Arm::Entry003PinnedControl,
        &mechanics,
        &contractility,
        &traction,
    );
    let motor_off = run_arm(
        &settled,
        Arm::MotorOffControl,
        &mechanics,
        &contractility,
        &traction,
    );
    let arms = [&unguided, &entry003, &motor_off];
    let conservation_pass = arms.iter().all(|arm| {
        (arm.n_world_loss - arm.n_delivered).abs() <= 1e-12
            && (arm.f_world_loss - arm.f_delivered).abs() <= 1e-12
            && arm.maximum_conservation_error <= 1e-12
    });
    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE,
            "starting_head": STARTING_HEAD,
            "historical_fixture": "DC-DEV-013 exact / DC-DEV-021 ENTRY-006 arms",
            "arms": ["UNGUIDED_EXPLORER", "ENTRY003_PINNED_CONTROL", "MOTOR_OFF_CONTROL"],
            "causal_step_order": ["observer_only_uptake_diagnostic", "advance_existing_arm", "existing_dcdev008_uptake", "record_ledger"],
            "parameter_screening": false,
            "geometry_screening": false,
            "observer_only": true
        }),
    );
    write_json(
        &output,
        "authority.json",
        &json!({"production_contract": "MaturationCoupledV4", "reserve": false, "m1": "CLOSED/FROZEN", "entry006_head": STARTING_HEAD, "uptake_law_changed": false, "organism_receives_diagnostic": false}),
    );
    write_json(
        &output,
        "unguided_explorer.json",
        &serde_json::to_value(&unguided).unwrap(),
    );
    write_json(
        &output,
        "entry003_control.json",
        &serde_json::to_value(&entry003).unwrap(),
    );
    write_json(
        &output,
        "motor_off_control.json",
        &serde_json::to_value(&motor_off).unwrap(),
    );
    write_json(
        &output,
        "decomposition_summary.json",
        &json!({
            "schema": "dcdev021_entry007_uptake_decomposition_summary_v1",
            "fields": ["exposed_segment_length", "occupancy", "permeability", "concentration_driving_force", "requested_flux", "delivered_flux", "area", "segment_area_fraction"],
            "conservation_pass": conservation_pass,
            "arms": arms.iter().map(|arm| json!({"arm": arm.arm, "n_delivered": arm.n_delivered, "f_delivered": arm.f_delivered, "time_integrated_exposed_segment_length": arm.time_integrated_exposed_segment_length, "final_exposed_intact_edges": arm.final_exposed_intact_edges, "diagnostics_hash": arm.diagnostics_hash})).collect::<Vec<_>>()
        }),
    );
    write_json(
        &output,
        "observer_boundary.json",
        &json!({"classification": "OBSERVER_ONLY", "diagnostic_calls_used_by_organism": 0, "resource_signal_calls_used_by_organism": 0, "diagnostic_supplied_to": [], "diagnostic_does_not_change": ["uptake_law", "intrinsic_exploration", "motor", "traction", "geometry"]}),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({"classification": "M2_ENTRY007_UPTAKE_DEGRADATION_AUDIT_COMPLETE", "conservation_pass": conservation_pass, "entry006_result_replayed": true, "autonomous_resource_acquisition": "NOT_ESTABLISHED", "next_execution_started": false}),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({"directive": DIRECTIVE, "files": ["protocol.json", "authority.json", "unguided_explorer.json", "entry003_control.json", "motor_off_control.json", "decomposition_summary.json", "observer_boundary.json", "qualification.json"], "source_hashes": {"spatial_resource": source_hash("spatial_resource.rs"), "intrinsic_exploration": source_hash("intrinsic_exploration.rs"), "contractility": source_hash("contractility.rs"), "traction": source_hash("stick_slip_traction.rs")}}),
    );
    println!("M2_ENTRY007_UPTAKE_DEGRADATION_AUDIT_COMPLETE");
}
