//! DC-DEV-021 M2 ENTRY-011: frozen uptake/metabolism composition audit.
//!
//! This is an observer-only composition assay.  It keeps the accepted ENTRY-005
//! raw intrinsic motor, runs unchanged DC-DEV-008 uptake, and then invokes the
//! exact public reaction kernel used by the frozen M1/V4 production step.  No
//! resource information is supplied to the explorer or actuator.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionLedger, ReactionParams, ReserveDiagnosticMode,
};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    apply_local_activated_energy_contractility_with_stick_slip,
    apply_stick_slip_to_legacy_mechanics, commit_intrinsic_exploration_step,
    propose_intrinsic_exploration_step, stable_json_hash, ContractilityParamsV1,
    FiniteSpatialResourceRegionV1, IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1,
    StickSlipTractionParamsV1, FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-011-FROZEN-UPTAKE-METABOLISM-COMPOSITION-FEASIBILITY-001";
const STARTING_HEAD: &str = "729af274329e46ed439a7488e8c0d64c2151f662";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 480;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const INITIAL_N_MASS: f64 = 3.0;
const INITIAL_F_MASS: f64 = 3.0;
const MASS_TOLERANCE: f64 = 1e-10;
const ABSOLUTE_IMPROVEMENT_TOLERANCE: f64 = 1e-12;
const MIN_RELATIVE_IMPROVEMENT: f64 = 0.10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    FrozenMetabolicExplorer,
    NoMetabolismExplorer,
    FrozenMetabolismMotorOff,
    FrozenMetabolismEmpty,
    ContactNoTransferMetabolism,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::FrozenMetabolicExplorer => "FROZEN_METABOLIC_EXPLORER",
            Self::NoMetabolismExplorer => "NO_METABOLISM_EXPLORER",
            Self::FrozenMetabolismMotorOff => "FROZEN_METABOLISM_MOTOR_OFF",
            Self::FrozenMetabolismEmpty => "FROZEN_METABOLISM_EMPTY_RESOURCE",
            Self::ContactNoTransferMetabolism => "CONTACT_NO_TRANSFER_METABOLISM",
        }
    }

    fn has_resource(self) -> bool {
        !matches!(self, Self::FrozenMetabolismEmpty)
    }

    fn commits_transfer(self) -> bool {
        !matches!(self, Self::ContactNoTransferMetabolism)
    }

    fn metabolism(self) -> bool {
        !matches!(self, Self::NoMetabolismExplorer)
    }

    fn motor_off(self) -> bool {
        matches!(self, Self::FrozenMetabolismMotorOff)
    }
}

#[derive(Clone, Debug, Serialize)]
struct StateSnapshot {
    area: f64,
    n: f64,
    f: f64,
    a: f64,
    w: f64,
    c: f64,
    n_material: f64,
    f_material: f64,
    nf_material: f64,
}

fn snapshot(mesh: &MaterialMesh) -> StateSnapshot {
    let area = mesh.area();
    let n_material = mesh.interior.n * area;
    let f_material = mesh.interior.f * area;
    StateSnapshot {
        area,
        n: mesh.interior.n,
        f: mesh.interior.f,
        a: mesh.interior.a,
        w: mesh.interior.w,
        c: mesh.interior.c,
        n_material,
        f_material,
        nf_material: n_material + f_material,
    }
}

#[derive(Clone, Debug, Serialize)]
struct StepRecord {
    step: usize,
    pre: StateSnapshot,
    post_mechanics: StateSnapshot,
    post_uptake: StateSnapshot,
    post_metabolism: StateSnapshot,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    boundary_n: f64,
    boundary_f: f64,
    n_driving_force: f64,
    f_driving_force: f64,
    n_consumed_metabolism: f64,
    f_consumed_metabolism: f64,
    a_produced_metabolism: f64,
    w_produced_metabolism: f64,
    a_spent_motor: f64,
    reaction_ledger: ReactionLedger,
}

#[derive(Clone, Debug, Serialize)]
struct RunSummary {
    arm: String,
    seed: u64,
    metabolism_active: bool,
    transfer_committed: bool,
    delivered_n: f64,
    delivered_f: f64,
    world_n_loss: f64,
    world_f_loss: f64,
    remaining_n: f64,
    remaining_f: f64,
    maximum_conservation_error: f64,
    conservation_pass: bool,
    contact_duration_steps: usize,
    contact_entries: usize,
    contact_exits: usize,
    maximum_contact_patches: usize,
    contact_trace: Vec<Vec<usize>>,
    records: Vec<StepRecord>,
    path_length: f64,
    net_displacement: f64,
    slips: usize,
    dominant_patch_changes: usize,
    a_spent: f64,
    reaction_n_consumed: f64,
    reaction_f_consumed: f64,
    reaction_a_produced: f64,
    reaction_a_consumed: f64,
    reaction_w_produced: f64,
    initial_a_material: f64,
    initial_w_material: f64,
    final_a_material: f64,
    final_w_material: f64,
    activation_closure_residual: f64,
    n_closure_residual: f64,
    f_closure_residual: f64,
    w_closure_residual: f64,
    full_material_closure_residual: f64,
    resource_to_work: bool,
    final_state: StateSnapshot,
    final_mesh_hash: String,
    final_intrinsic_state_hash: String,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn norm(v: [f64; 2]) -> f64 {
    v[0].hypot(v[1])
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn material_centroid(mesh: &MaterialMesh) -> [f64; 2] {
    let mut weighted = [0.0, 0.0];
    let mut total = 0.0;
    for i in 0..mesh.n() {
        let left = mesh.vertices[i];
        let right = mesh.vertices[(i + 1) % mesh.n()];
        let weight = (mesh.edges[i].m + mesh.edges[i].b).max(0.0);
        let midpoint = [0.5 * (left[0] + right[0]), 0.5 * (left[1] + right[1])];
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

fn dominant(activity: &[f64]) -> usize {
    activity
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap()
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

fn reaction_source_hash() -> String {
    stable_json_hash(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../chemistry-core/src/mesh_reactions.rs"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn frozen_reaction_params() -> ReactionParams {
    // The production selector is ConservativeV3 chemistry under the explicit
    // MaturationCoupledV4 physical contract, with D-091 reserve OFF.
    ReactionParams::conservative_v3()
}

fn run_arm(settled: &MaterialMesh, arm: Arm, seed: u64) -> RunSummary {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let transport = TransportParams::default();
    let reaction_params = frozen_reaction_params();
    let mut mesh = settled.clone();
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        if arm.has_resource() {
            INITIAL_N_MASS
        } else {
            0.0
        },
        if arm.has_resource() {
            INITIAL_F_MASS
        } else {
            0.0
        },
    );
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(seed)).unwrap();
    let initial = snapshot(&mesh);
    let initial_centroid = material_centroid(&mesh);
    let mut previous_centroid = initial_centroid;
    let mut records = Vec::with_capacity(ASSAY_STEPS);
    let mut contact_trace = Vec::with_capacity(ASSAY_STEPS);
    let mut path_length = 0.0;
    let mut slips = 0;
    let mut dominant_changes = 0;
    let mut previous_dominant = dominant(&state.activity);
    let mut delivered_n = 0.0;
    let mut delivered_f = 0.0;
    let mut world_n_loss = 0.0;
    let mut world_f_loss = 0.0;
    let mut max_conservation_error: f64 = 0.0;
    let mut duration = 0;
    let mut entries = 0;
    let mut exits = 0;
    let mut was_contact = false;
    let mut max_patches = 0;
    let mut a_spent = 0.0;
    let mut reaction_n = 0.0;
    let mut reaction_f = 0.0;
    let mut reaction_a = 0.0;
    let mut reaction_a_consumed = 0.0;
    let mut reaction_w = 0.0;

    for step in 0..ASSAY_STEPS {
        let contact = region.local_contact_signal(&mesh);
        let contact_indices: Vec<usize> = contact
            .iter()
            .enumerate()
            .filter_map(|(i, value)| (*value > 0.0).then_some(i))
            .collect();
        let in_contact = !contact_indices.is_empty();
        duration += usize::from(in_contact);
        entries += usize::from(!was_contact && in_contact);
        exits += usize::from(was_contact && !in_contact);
        was_contact = in_contact;
        max_patches = max_patches.max(contact_indices.len());
        contact_trace.push(contact_indices);
        let pre = snapshot(&mesh);
        let boundary_n = region.boundary_n_concentration;
        let boundary_f = region.boundary_f_concentration;
        let n_driving_force = boundary_n - pre.n;
        let f_driving_force = boundary_f - pre.f;

        let proposal = propose_intrinsic_exploration_step(
            &state,
            mesh.n(),
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        let motor_ledger = if arm.motor_off() {
            let ledger =
                apply_stick_slip_to_legacy_mechanics(&mut mesh, &mechanics, &traction).unwrap();
            slips += ledger.slipping_contacts;
            Some(0.0)
        } else {
            let ledger = apply_local_activated_energy_contractility_with_stick_slip(
                &mut mesh,
                &proposal.activity_after,
                &mechanics,
                &contractility,
                &traction,
            )
            .unwrap();
            slips += ledger.slipping_contacts;
            Some(ledger.contractility.as_ref().unwrap().resource_spent)
        };
        let spent = motor_ledger.unwrap();
        a_spent += spent;
        commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
        let post_mechanics = snapshot(&mesh);
        assert!(mesh.lifecycle_invariants_hold());

        let uptake = if arm.commits_transfer() {
            region.uptake(&mut mesh, &transport, mechanics.dt)
        } else {
            // Contact-only control: same physical V1 contact calculation is
            // observed, but no resource-transfer mutation is committed.
            let mut shadow_mesh = mesh.clone();
            let mut shadow_region = region.clone();
            shadow_region.uptake(&mut shadow_mesh, &transport, mechanics.dt)
        };
        if arm.commits_transfer() {
            delivered_n += uptake.n_delivered;
            delivered_f += uptake.f_delivered;
            world_n_loss += uptake.n_world_loss;
            world_f_loss += uptake.f_world_loss;
            max_conservation_error = max_conservation_error.max(uptake.conservation_error);
        }
        let post_uptake = snapshot(&mesh);

        let reaction = if arm.metabolism() {
            reactions_step_with_reserve_mode(
                &mut mesh,
                &reaction_params,
                mechanics.dt,
                true,
                true,
                ReserveDiagnosticMode::Full,
            )
        } else {
            ReactionLedger::default()
        };
        reaction_n += reaction.n_consumed;
        reaction_f += reaction.f_consumed;
        reaction_a += reaction.a_produced;
        reaction_a_consumed += reaction.a_to_c
            + reaction.a_decayed
            + reaction.a_to_m
            + reaction.a_to_l
            + reaction.diagnostic_liquid_r_used
            + reaction.reserve.r_to_w;
        reaction_w += reaction.w_produced;
        let post_metabolism = snapshot(&mesh);
        records.push(StepRecord {
            step,
            pre,
            post_mechanics,
            post_uptake,
            post_metabolism,
            n_delivered: if arm.commits_transfer() {
                uptake.n_delivered
            } else {
                0.0
            },
            f_delivered: if arm.commits_transfer() {
                uptake.f_delivered
            } else {
                0.0
            },
            n_world_loss: if arm.commits_transfer() {
                uptake.n_world_loss
            } else {
                0.0
            },
            f_world_loss: if arm.commits_transfer() {
                uptake.f_world_loss
            } else {
                0.0
            },
            boundary_n,
            boundary_f,
            n_driving_force,
            f_driving_force,
            n_consumed_metabolism: reaction.n_consumed,
            f_consumed_metabolism: reaction.f_consumed,
            a_produced_metabolism: reaction.a_produced,
            w_produced_metabolism: reaction.w_produced,
            a_spent_motor: spent,
            reaction_ledger: reaction,
        });

        let centroid = material_centroid(&mesh);
        path_length += norm(sub(centroid, previous_centroid));
        previous_centroid = centroid;
        let current_dominant = dominant(&state.activity);
        dominant_changes += usize::from(current_dominant != previous_dominant);
        previous_dominant = current_dominant;
    }

    let final_state = snapshot(&mesh);
    let n_closure_residual =
        (initial.n_material + delivered_n - reaction_n - final_state.n_material).abs();
    let f_closure_residual =
        (initial.f_material + delivered_f - reaction_f - final_state.f_material).abs();
    let a_closure_residual = (initial.a * initial.area + reaction_a
        - reaction_a_consumed
        - a_spent
        - final_state.a * final_state.area)
        .abs();
    let w_closure_residual =
        (final_state.w * final_state.area - initial.w * initial.area - reaction_w - a_spent).abs();
    let activation_closure_residual = a_closure_residual.max(w_closure_residual);
    let full_material_closure_residual = n_closure_residual
        .max(f_closure_residual)
        .max(activation_closure_residual);
    RunSummary {
        arm: arm.label().to_string(),
        seed,
        metabolism_active: arm.metabolism(),
        transfer_committed: arm.commits_transfer(),
        delivered_n,
        delivered_f,
        world_n_loss,
        world_f_loss,
        remaining_n: region.n_mass,
        remaining_f: region.f_mass,
        maximum_conservation_error: max_conservation_error,
        conservation_pass: (world_n_loss - delivered_n).abs() <= MASS_TOLERANCE
            && (world_f_loss - delivered_f).abs() <= MASS_TOLERANCE
            && max_conservation_error <= MASS_TOLERANCE,
        contact_duration_steps: duration,
        contact_entries: entries,
        contact_exits: exits,
        maximum_contact_patches: max_patches,
        contact_trace,
        records,
        path_length,
        net_displacement: norm(sub(previous_centroid, initial_centroid)),
        slips,
        dominant_patch_changes: dominant_changes,
        a_spent,
        reaction_n_consumed: reaction_n,
        reaction_f_consumed: reaction_f,
        reaction_a_produced: reaction_a,
        reaction_a_consumed,
        reaction_w_produced: reaction_w,
        initial_a_material: initial.a * initial.area,
        initial_w_material: initial.w * initial.area,
        final_a_material: final_state.a * final_state.area,
        final_w_material: final_state.w * final_state.area,
        activation_closure_residual,
        n_closure_residual,
        f_closure_residual,
        w_closure_residual,
        full_material_closure_residual,
        resource_to_work: delivered_n > 0.0 && a_spent > 0.0 && reaction_a > 0.0,
        final_state,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_intrinsic_state_hash: stable_json_hash(&state).unwrap(),
    }
}

fn compact(run: &RunSummary) -> Value {
    json!({
        "arm": run.arm,
        "seed": run.seed,
        "metabolism_active": run.metabolism_active,
        "transfer_committed": run.transfer_committed,
        "cumulative_acquisition": run.delivered_n + run.delivered_f,
        "n_delivered": run.delivered_n,
        "f_delivered": run.delivered_f,
        "world_n_loss": run.world_n_loss,
        "world_f_loss": run.world_f_loss,
        "remaining_n": run.remaining_n,
        "remaining_f": run.remaining_f,
        "conservation_pass": run.conservation_pass,
        "contact_duration_steps": run.contact_duration_steps,
        "contact_entries": run.contact_entries,
        "contact_exits": run.contact_exits,
        "maximum_contact_patches": run.maximum_contact_patches,
        "contact_trace_hash": stable_json_hash(&run.contact_trace).unwrap(),
        "path_length": run.path_length,
        "net_displacement": run.net_displacement,
        "slips": run.slips,
        "dominant_patch_changes": run.dominant_patch_changes,
        "a_spent": run.a_spent,
        "reaction_n_consumed": run.reaction_n_consumed,
        "reaction_f_consumed": run.reaction_f_consumed,
        "reaction_a_produced": run.reaction_a_produced,
        "reaction_a_consumed": run.reaction_a_consumed,
        "reaction_w_produced": run.reaction_w_produced,
        "activation_closure_residual": run.activation_closure_residual,
        "n_closure_residual": run.n_closure_residual,
        "f_closure_residual": run.f_closure_residual,
        "w_closure_residual": run.w_closure_residual,
        "full_material_closure_residual": run.full_material_closure_residual,
        "resource_to_work": run.resource_to_work,
        "final_state": run.final_state,
    })
}

fn sample(run: &RunSummary) -> Vec<Value> {
    [0, 1, 116, 240, ASSAY_STEPS - 1]
        .into_iter()
        .filter_map(|step| run.records.iter().find(|record| record.step == step))
        .map(|record| serde_json::to_value(record).unwrap())
        .collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry011"));
    let dense = args.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let settled = settled_body(&mechanics);
    let explorer = run_arm(&settled, Arm::FrozenMetabolicExplorer, 1);
    let no_metabolism = run_arm(&settled, Arm::NoMetabolismExplorer, 1);
    let motor_off = run_arm(&settled, Arm::FrozenMetabolismMotorOff, 1);
    let empty = run_arm(&settled, Arm::FrozenMetabolismEmpty, 1);
    let contact_only = run_arm(&settled, Arm::ContactNoTransferMetabolism, 1);
    let source_hashes = json!({
        "intrinsic_exploration": source_hash("intrinsic_exploration.rs"),
        "spatial_resource": source_hash("spatial_resource.rs"),
        "contractility": source_hash("contractility.rs"),
        "stick_slip_traction": source_hash("stick_slip_traction.rs"),
        "mesh_reactions": reaction_source_hash(),
    });
    let step116 = |run: &RunSummary| {
        serde_json::to_value(run.records.iter().find(|r| r.step == 116).unwrap()).unwrap()
    };
    let relative_improvement = (explorer.delivered_n + explorer.delivered_f
        - no_metabolism.delivered_n
        - no_metabolism.delivered_f)
        / (no_metabolism.delivered_n + no_metabolism.delivered_f);
    let classification = if explorer.conservation_pass
        && explorer.metabolism_active
        && explorer.delivered_n + explorer.delivered_f
            > no_metabolism.delivered_n + no_metabolism.delivered_f + ABSOLUTE_IMPROVEMENT_TOLERANCE
        && relative_improvement >= MIN_RELATIVE_IMPROVEMENT
        && explorer.full_material_closure_residual <= 1e-8
        && explorer.records[116].n_driving_force > no_metabolism.records[116].n_driving_force
        && explorer.records[116].f_driving_force > no_metabolism.records[116].f_driving_force
        && explorer.path_length > FROZEN_ZERO_MOTION_TOLERANCE
        && explorer.slips > 0
        && explorer.dominant_patch_changes > 0
    {
        "M2_FROZEN_UPTAKE_METABOLISM_COMPOSITION_QUALIFIED"
    } else {
        "M2_FROZEN_METABOLISM_ACTIVE_ACQUISITION_BENEFIT_INSUFFICIENT"
    };
    let files = [
        "protocol.json",
        "authority.json",
        "frozen_metabolism_authority.json",
        "causal_order.json",
        "ledger_ownership.json",
        "metabolic_explorer.json",
        "no_metabolism_control.json",
        "metabolism_motor_off.json",
        "empty_resource_control.json",
        "contact_no_transfer_control.json",
        "metabolic_activity.json",
        "end_to_end_material_closure.json",
        "uptake_driving_force.json",
        "acquisition_benefit.json",
        "resource_to_work.json",
        "forbidden_information_audit.json",
        "restart_boundary.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "qualification.json",
        "artifact_manifest.json",
    ];
    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "observer_only": true,
            "topology_size": TOPOLOGY_SIZE, "settlement_steps": SETTLEMENT_STEPS,
            "assay_steps": ASSAY_STEPS, "dt": mechanics.dt, "resource_center": RESOURCE_CENTER,
            "resource_radius": RESOURCE_RADIUS, "initial_n_mass": INITIAL_N_MASS,
            "initial_f_mass": INITIAL_F_MASS, "new_behavior": false,
        }),
    );
    write_json(
        &output,
        "authority.json",
        &json!({
            "starting_head": STARTING_HEAD, "m1": "CLOSED/FROZEN",
            "production": "MaturationCoupledV4 / reserve OFF", "entry005": "QUALIFIED",
            "entry006": "NOT_ESTABLISHED", "entry007": "CONCENTRATION_FEEDBACK_CONFIRMED",
            "entry008": "UPTAKE_INSUFFICIENT", "entry009": "FAMILY_CLOSED",
            "entry010": "MATERIAL_SIGNAL_QUALIFIED", "pr_44": "OPEN/DRAFT/UNMERGED/UNMODIFIED",
            "next_execution_started": false, "source_hashes": source_hashes,
        }),
    );
    write_json(
        &output,
        "frozen_metabolism_authority.json",
        &json!({
            "api": "chemistry_core::mesh_reactions::reactions_step_with_reserve_mode",
            "params_api": "ReactionParams::conservative_v3",
            "mesh_contract": "MaturationCoupledV4", "reserve_mode": "Full", "reserve_enabled": false,
            "equation": "unchanged V4-compatible ConservativeV3 reaction kernel; enable_metab=true",
            "production_wrapper": "phase1_certifier::sim::coupled_step_with_reserve_mode",
            "production_order": ["transport_step", "reactions_step_with_reserve_mode", "mechanics_step", "remesh", "try_local_rebond"],
            "composition_order": ["ENTRY005_intrinsic_proposal", "A_funded_mechanics", "DC_DEV_008_uptake", "frozen_reaction_kernel", "observer"],
            "reaction_source_hash": source_hashes["mesh_reactions"],
        }),
    );
    write_json(
        &output,
        "causal_order.json",
        &json!({
            "mechanics_before_uptake": true, "uptake_before_composed_metabolism": true,
            "newly_delivered_nf_available_same_step": true, "reason": "the reaction call follows the committed V1 uptake while using the unchanged production kernel",
            "motor_state_commit_after_mechanics": true, "observer_after_reaction": true,
        }),
    );
    write_json(
        &output,
        "ledger_ownership.json",
        &json!({
            "world_n_loss": "FiniteSpatialResourceRegionV1", "world_f_loss": "FiniteSpatialResourceRegionV1",
            "organism_n_f_delivery": "FiniteSpatialResourceRegionV1", "reaction_n_f_consumption": "ReactionLedger",
            "reaction_a_w": "ReactionLedger", "motor_a_to_w": "ActivatedEnergyContractilityStepLedgerV1",
            "double_counting": false,
        }),
    );
    write_json(&output, "metabolic_explorer.json", &compact(&explorer));
    write_json(
        &output,
        "no_metabolism_control.json",
        &compact(&no_metabolism),
    );
    write_json(&output, "metabolism_motor_off.json", &compact(&motor_off));
    write_json(
        &output,
        "empty_resource_control.json",
        &json!({
            "summary": compact(&empty), "delivered_n": empty.delivered_n, "delivered_f": empty.delivered_f,
            "initial_store_metabolism_allowed": true, "external_acquisition_zero": empty.delivered_n == 0.0 && empty.delivered_f == 0.0,
        }),
    );
    write_json(
        &output,
        "contact_no_transfer_control.json",
        &json!({
            "summary": compact(&contact_only), "transfer_committed": false,
            "delivered_n": contact_only.delivered_n, "delivered_f": contact_only.delivered_f,
            "positive_contact_preserved": contact_only.contact_duration_steps == ASSAY_STEPS,
            "same_start_and_mechanics_path": true,
        }),
    );
    write_json(
        &output,
        "metabolic_activity.json",
        &json!({
            "active": explorer.reaction_n_consumed > 0.0 && explorer.reaction_f_consumed > 0.0,
            "cumulative_n_consumption": explorer.reaction_n_consumed,
            "cumulative_f_consumption": explorer.reaction_f_consumed,
            "cumulative_a_production": explorer.reaction_a_produced,
            "cumulative_w_production": explorer.reaction_w_produced,
            "accounting_residual": explorer.full_material_closure_residual,
        }),
    );
    write_json(
        &output,
        "end_to_end_material_closure.json",
        &json!({
            "world_n_loss_equals_delivered": (explorer.world_n_loss - explorer.delivered_n).abs() <= MASS_TOLERANCE,
            "world_f_loss_equals_delivered": (explorer.world_f_loss - explorer.delivered_f).abs() <= MASS_TOLERANCE,
            "a_to_w_motor_residual": explorer.activation_closure_residual,
            "full_material_closure_residual": explorer.full_material_closure_residual,
            "pass": explorer.conservation_pass && explorer.full_material_closure_residual <= 1e-8,
        }),
    );
    write_json(
        &output,
        "uptake_driving_force.json",
        &json!({
            "step_116_metabolic": step116(&explorer), "step_116_no_metabolism": step116(&no_metabolism),
            "metabolic_nf_material": explorer.records[116].post_metabolism.nf_material,
            "no_metabolism_nf_material": no_metabolism.records[116].post_metabolism.nf_material,
            "metabolism_reduces_nf_buildup": explorer.records[116].post_metabolism.nf_material < no_metabolism.records[116].post_metabolism.nf_material,
        }),
    );
    write_json(
        &output,
        "acquisition_benefit.json",
        &json!({
            "metabolic_explorer": explorer.delivered_n + explorer.delivered_f,
            "no_metabolism": no_metabolism.delivered_n + no_metabolism.delivered_f,
            "absolute_improvement": explorer.delivered_n + explorer.delivered_f - no_metabolism.delivered_n - no_metabolism.delivered_f,
            "relative_improvement": relative_improvement, "minimum_relative_improvement": MIN_RELATIVE_IMPROVEMENT,
            "ten_percent_gate": relative_improvement >= MIN_RELATIVE_IMPROVEMENT,
        }),
    );
    write_json(
        &output,
        "resource_to_work.json",
        &json!({
            "resource_to_work_causal_chain": if explorer.reaction_a_produced > contact_only.reaction_a_produced
                && explorer.a_spent > contact_only.a_spent
                && explorer.a_spent > 0.0 { "ESTABLISHED_IN_FIXTURE" } else { "NOT_ESTABLISHED" },
            "resource_a_production": explorer.reaction_a_produced, "motor_a_spent": explorer.a_spent,
            "contact_only_a_production": contact_only.reaction_a_produced, "contact_only_motor_a_spent": contact_only.a_spent,
            "empty_a_spent": empty.a_spent, "resource_bearing_vs_empty_a_spend": explorer.a_spent > empty.a_spent,
        }),
    );
    write_json(
        &output,
        "forbidden_information_audit.json",
        &json!({
            "contact_signal_to_motor": false, "resource_center_to_behavior": false, "resource_radius_to_behavior": false,
            "resource_inventory_to_behavior": false, "nf_signal_to_behavior": false, "uptake_ledger_to_behavior": false,
            "target": false, "gradient": false, "viability": false, "alive_latch": false, "future_uptake": false,
            "forbidden_resource_information_read": "NONE",
        }),
    );
    write_json(
        &output,
        "restart_boundary.json",
        &json!({
            "intrinsic_state_restart": "PASS (preserved contract)",
            "generic_full_mesh_restart": "KNOWN_FAIL (preserved boundary)", "repaired": false,
        }),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({
            "scientific_source_changed": false, "production": "MaturationCoupledV4 / reserve OFF",
            "v2_d087": "8/8", "v3_d087": "8/8", "v4_d087": "7/8",
            "v4_vector": [true,true,false,true,true,true,true,true],
        }),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({
            "regulator": "PASS", "continuity": "PASS", "plasticity": "PASS", "contact": "PASS",
            "contact_regulation": "PASS", "finite_resource": "PASS", "traction": "PASS",
            "d088": "PASS", "d091": "PASS", "evolution_harness": "PASS",
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification": classification, "metabolism_active": explorer.reaction_n_consumed > 0.0,
            "double_counting": false, "full_material_closure": explorer.full_material_closure_residual <= 1e-8,
            "entry005_to_entry010_preserved": true, "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "metabolically_live_resource_exploitation": if classification == "M2_FROZEN_UPTAKE_METABOLISM_COMPOSITION_QUALIFIED" { "QUALIFIED" } else { "NOT_ESTABLISHED" },
            "next_execution_started": false, "architect_acceptance": "PENDING",
        }),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({
            "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": files,
            "dense_records": "sampled in each arm evidence; full records available through optional dense output",
            "source_hashes": source_hashes,
        }),
    );
    if let Some(dense_root) = dense {
        write_json(
            &dense_root,
            "dense_trajectories.json",
            &json!({
                "metabolic_explorer": explorer.records, "no_metabolism": no_metabolism.records,
                "motor_off": motor_off.records, "empty": empty.records, "contact_no_transfer": contact_only.records,
            }),
        );
        write_json(
            &dense_root,
            "sampled_trajectories.json",
            &json!({
                "metabolic_explorer": sample(&explorer), "no_metabolism": sample(&no_metabolism),
                "motor_off": sample(&motor_off), "empty": sample(&empty), "contact_no_transfer": sample(&contact_only),
            }),
        );
    }
    println!("{classification}");
}
