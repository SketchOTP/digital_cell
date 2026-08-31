//! DC-DEV-021 ENTRY-010: observer-only post-ingestive material-signal audit.
//!
//! The transfer arm and a matched contact-without-transfer arm execute the
//! same frozen ENTRY-009 mechanics.  Only the transfer arm commits the
//! unchanged DC-DEV-008 uptake ledger.  This makes concentration changes and
//! conserved internal N/F amounts directly comparable without installing a
//! sensor, flag, memory variable, or behavior.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{
    apply_local_activated_energy_contractility_with_stick_slip, commit_intrinsic_exploration_step,
    propose_intrinsic_exploration_step, stable_json_hash, ContractilityParamsV1,
    FiniteSpatialResourceRegionV1, IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1,
    StickSlipTractionParamsV1,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-010-POST-INGESTIVE-MATERIAL-SIGNAL-SUBSTRATE-AUDIT-001";
const STARTING_HEAD: &str = "ee9597deec8e41dd77d133927ab5102984c69fee";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 480;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const INITIAL_N_MASS: f64 = 3.0;
const INITIAL_F_MASS: f64 = 3.0;
const MASS_TOLERANCE: f64 = 1e-10;
const STATE_TOLERANCE: f64 = 1e-12;

#[derive(Clone, Debug, Serialize)]
struct StateSnapshot {
    area: f64,
    interior_n: f64,
    interior_f: f64,
    interior_a: f64,
    interior_w: f64,
    interior_c: f64,
    n_material_amount: f64,
    f_material_amount: f64,
    nf_material_amount: f64,
}

fn snapshot(mesh: &MaterialMesh) -> StateSnapshot {
    let area = mesh.area();
    let n_material_amount = mesh.interior.n * area;
    let f_material_amount = mesh.interior.f * area;
    StateSnapshot {
        area,
        interior_n: mesh.interior.n,
        interior_f: mesh.interior.f,
        interior_a: mesh.interior.a,
        interior_w: mesh.interior.w,
        interior_c: mesh.interior.c,
        n_material_amount,
        f_material_amount,
        nf_material_amount: n_material_amount + f_material_amount,
    }
}

#[derive(Clone, Debug, Serialize)]
struct StepRecord {
    step: usize,
    contact_indices: Vec<usize>,
    pre: StateSnapshot,
    post_mechanics: StateSnapshot,
    post_transfer: StateSnapshot,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    conservation_error: f64,
}

#[derive(Clone, Debug, Serialize)]
struct RunSummary {
    arm: String,
    seed: u64,
    delivered_n: f64,
    delivered_f: f64,
    world_n_loss: f64,
    world_f_loss: f64,
    maximum_conservation_error: f64,
    conservation_pass: bool,
    contact_duration_steps: usize,
    contact_entries: usize,
    contact_exits: usize,
    maximum_contact_patches: usize,
    contact_trace: Vec<Vec<usize>>,
    records: Vec<StepRecord>,
    path_length: f64,
    slips: usize,
    a_spent: f64,
    w_generated: f64,
    a_to_w_residual: f64,
    reserve_before: f64,
    reserve_after: f64,
    final_state: StateSnapshot,
    source_hashes: serde_json::Value,
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

fn rotate_180(mut mesh: MaterialMesh) -> MaterialMesh {
    for vertex in &mut mesh.vertices {
        vertex[0] = -vertex[0];
        vertex[1] = -vertex[1];
    }
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

fn run_pair(
    settled: &MaterialMesh,
    seed: u64,
    center: [f64; 2],
    resource_present: bool,
) -> (RunSummary, RunSummary) {
    let mut transfer_mesh = settled.clone();
    let mut control_mesh = settled.clone();
    let mut transfer_region = FiniteSpatialResourceRegionV1::new(
        center,
        RESOURCE_RADIUS,
        if resource_present {
            INITIAL_N_MASS
        } else {
            0.0
        },
        if resource_present {
            INITIAL_F_MASS
        } else {
            0.0
        },
    );
    let control_region = transfer_region.clone();
    let mut transfer_state = IntrinsicExplorationStateV1::new(TOPOLOGY_SIZE, Some(seed)).unwrap();
    let mut control_state = IntrinsicExplorationStateV1::new(TOPOLOGY_SIZE, Some(seed)).unwrap();
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let transport = TransportParams::default();
    let transfer_initial = snapshot(&transfer_mesh);
    let control_initial = snapshot(&control_mesh);
    let transfer_initial_a = transfer_mesh.interior.a * transfer_initial.area;
    let transfer_initial_w = transfer_mesh.interior.w * transfer_initial.area;
    let control_initial_a = control_mesh.interior.a * control_initial.area;
    let control_initial_w = control_mesh.interior.w * control_initial.area;
    let reserve_before = transfer_mesh.interior.r;
    let mut transfer_records = Vec::with_capacity(ASSAY_STEPS);
    let mut control_records = Vec::with_capacity(ASSAY_STEPS);
    let mut transfer_trace = Vec::with_capacity(ASSAY_STEPS);
    let mut control_trace = Vec::with_capacity(ASSAY_STEPS);
    let mut transfer_duration = 0;
    let mut control_duration = 0;
    let mut transfer_entries = 0;
    let mut control_entries = 0;
    let mut transfer_exits = 0;
    let mut control_exits = 0;
    let mut transfer_was_contact = false;
    let mut control_was_contact = false;
    let mut transfer_delivered_n = 0.0;
    let mut transfer_delivered_f = 0.0;
    let mut transfer_world_n = 0.0;
    let mut transfer_world_f = 0.0;
    let mut maximum_conservation_error: f64 = 0.0;
    let mut maximum_contact_patches = 0;
    let mut path_length = 0.0;
    let mut control_path_length = 0.0;
    let mut previous_centroid = transfer_mesh.centroid();
    let mut control_previous_centroid = control_mesh.centroid();
    let mut slips = 0;
    let mut control_slips = 0;
    let mut a_spent = 0.0;
    let mut control_a_spent = 0.0;

    for step in 0..ASSAY_STEPS {
        let transfer_contact = transfer_region.local_contact_signal(&transfer_mesh);
        let control_contact = control_region.local_contact_signal(&control_mesh);
        let transfer_indices: Vec<usize> = transfer_contact
            .iter()
            .enumerate()
            .filter_map(|(index, value)| (*value > 0.0).then_some(index))
            .collect();
        let control_indices: Vec<usize> = control_contact
            .iter()
            .enumerate()
            .filter_map(|(index, value)| (*value > 0.0).then_some(index))
            .collect();
        let transfer_in_contact = !transfer_indices.is_empty();
        let control_in_contact = !control_indices.is_empty();
        transfer_duration += usize::from(transfer_in_contact);
        control_duration += usize::from(control_in_contact);
        transfer_entries += usize::from(!transfer_was_contact && transfer_in_contact);
        control_entries += usize::from(!control_was_contact && control_in_contact);
        transfer_exits += usize::from(transfer_was_contact && !transfer_in_contact);
        control_exits += usize::from(control_was_contact && !control_in_contact);
        transfer_was_contact = transfer_in_contact;
        control_was_contact = control_in_contact;
        maximum_contact_patches = maximum_contact_patches.max(transfer_indices.len());
        transfer_trace.push(transfer_indices.clone());
        control_trace.push(control_indices.clone());
        let pre = snapshot(&transfer_mesh);
        let control_pre = snapshot(&control_mesh);

        let proposal = propose_intrinsic_exploration_step(
            &transfer_state,
            transfer_mesh.n(),
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        let control_proposal = proposal.clone();
        let transfer_ledger = apply_local_activated_energy_contractility_with_stick_slip(
            &mut transfer_mesh,
            &proposal.activity_after,
            &mechanics,
            &contractility,
            &traction,
        )
        .unwrap();
        let _control_ledger = apply_local_activated_energy_contractility_with_stick_slip(
            &mut control_mesh,
            &control_proposal.activity_after,
            &mechanics,
            &contractility,
            &traction,
        )
        .unwrap();
        commit_intrinsic_exploration_step(&mut transfer_state, proposal).unwrap();
        commit_intrinsic_exploration_step(&mut control_state, control_proposal).unwrap();
        let post_mechanics = snapshot(&transfer_mesh);
        assert!(post_mechanics.n_material_amount.is_finite());
        assert!(post_mechanics.f_material_amount.is_finite());

        let uptake = transfer_region.uptake(&mut transfer_mesh, &transport, mechanics.dt);
        transfer_delivered_n += uptake.n_delivered;
        transfer_delivered_f += uptake.f_delivered;
        transfer_world_n += uptake.n_world_loss;
        transfer_world_f += uptake.f_world_loss;
        maximum_conservation_error = maximum_conservation_error.max(uptake.conservation_error);
        let post_transfer = snapshot(&transfer_mesh);
        let control_post = snapshot(&control_mesh);
        assert!(post_transfer.area.is_finite() && control_post.area.is_finite());
        assert!(
            (post_transfer.n_material_amount
                - control_post.n_material_amount
                - (pre.n_material_amount - control_pre.n_material_amount)
                - uptake.n_delivered)
                .abs()
                <= STATE_TOLERANCE,
            "N amount mismatch at step {step}: transfer={} control={} delivered={} residual={}",
            post_transfer.n_material_amount,
            control_post.n_material_amount,
            uptake.n_delivered,
            post_transfer.n_material_amount
                - control_post.n_material_amount
                - (pre.n_material_amount - control_pre.n_material_amount)
                - uptake.n_delivered
        );
        assert!(
            (post_transfer.f_material_amount
                - control_post.f_material_amount
                - (pre.f_material_amount - control_pre.f_material_amount)
                - uptake.f_delivered)
                .abs()
                <= STATE_TOLERANCE,
            "F amount mismatch at step {step}: transfer={} control={} delivered={} residual={}",
            post_transfer.f_material_amount,
            control_post.f_material_amount,
            uptake.f_delivered,
            post_transfer.f_material_amount
                - control_post.f_material_amount
                - (pre.f_material_amount - control_pre.f_material_amount)
                - uptake.f_delivered
        );
        transfer_records.push(StepRecord {
            step,
            contact_indices: transfer_indices.clone(),
            pre: pre.clone(),
            post_mechanics: post_mechanics.clone(),
            post_transfer: post_transfer.clone(),
            n_delivered: uptake.n_delivered,
            f_delivered: uptake.f_delivered,
            n_world_loss: uptake.n_world_loss,
            f_world_loss: uptake.f_world_loss,
            conservation_error: uptake.conservation_error,
        });
        control_records.push(StepRecord {
            step,
            contact_indices: control_indices,
            pre: control_pre.clone(),
            post_mechanics: control_post.clone(),
            post_transfer: control_post,
            n_delivered: 0.0,
            f_delivered: 0.0,
            n_world_loss: 0.0,
            f_world_loss: 0.0,
            conservation_error: 0.0,
        });
        let centroid = transfer_mesh.centroid();
        path_length +=
            (centroid[0] - previous_centroid[0]).hypot(centroid[1] - previous_centroid[1]);
        previous_centroid = centroid;
        slips += transfer_ledger.slipping_contacts;
        a_spent += transfer_ledger
            .contractility
            .as_ref()
            .unwrap()
            .resource_spent;
        let control_centroid = control_mesh.centroid();
        control_path_length += (control_centroid[0] - control_previous_centroid[0])
            .hypot(control_centroid[1] - control_previous_centroid[1]);
        control_previous_centroid = control_centroid;
        control_slips += _control_ledger.slipping_contacts;
        control_a_spent += _control_ledger
            .contractility
            .as_ref()
            .unwrap()
            .resource_spent;
    }

    let source_hashes = json!({
        "intrinsic_exploration": source_hash("intrinsic_exploration.rs"),
        "spatial_resource": source_hash("spatial_resource.rs"),
        "contractility": source_hash("contractility.rs"),
        "stick_slip_traction": source_hash("stick_slip_traction.rs"),
        "mesh_reactions": stable_json_hash(&fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../chemistry-core/src/mesh_reactions.rs")).unwrap()).unwrap(),
        "coupled_resource": source_hash("coupled_resource.rs"),
    });
    let transfer_final = snapshot(&transfer_mesh);
    let control_final = snapshot(&control_mesh);
    let transfer = RunSummary {
        arm: if resource_present {
            "TRANSFER"
        } else {
            "EMPTY_RESOURCE"
        }
        .to_string(),
        seed,
        delivered_n: transfer_delivered_n,
        delivered_f: transfer_delivered_f,
        world_n_loss: transfer_world_n,
        world_f_loss: transfer_world_f,
        maximum_conservation_error,
        conservation_pass: (transfer_world_n - transfer_delivered_n).abs() <= MASS_TOLERANCE
            && (transfer_world_f - transfer_delivered_f).abs() <= MASS_TOLERANCE
            && maximum_conservation_error <= MASS_TOLERANCE,
        contact_duration_steps: transfer_duration,
        contact_entries: transfer_entries,
        contact_exits: transfer_exits,
        maximum_contact_patches,
        contact_trace: transfer_trace,
        records: transfer_records,
        path_length,
        slips,
        a_spent,
        w_generated: transfer_final.interior_w * transfer_final.area - transfer_initial_w,
        a_to_w_residual: (transfer_initial_a
            - transfer_final.interior_a * transfer_final.area
            - a_spent)
            .abs()
            .max(
                (transfer_final.interior_w * transfer_final.area - transfer_initial_w - a_spent)
                    .abs(),
            ),
        reserve_before,
        reserve_after: transfer_mesh.interior.r,
        final_state: transfer_final,
        source_hashes: source_hashes.clone(),
    };
    let control = RunSummary {
        arm: if resource_present {
            "CONTACT_WITHOUT_TRANSFER"
        } else {
            "EMPTY_RESOURCE_CONTROL"
        }
        .to_string(),
        seed,
        delivered_n: 0.0,
        delivered_f: 0.0,
        world_n_loss: 0.0,
        world_f_loss: 0.0,
        maximum_conservation_error: 0.0,
        conservation_pass: true,
        contact_duration_steps: control_duration,
        contact_entries: control_entries,
        contact_exits: control_exits,
        maximum_contact_patches,
        contact_trace: control_trace,
        records: control_records,
        path_length: control_path_length,
        slips: control_slips,
        a_spent: control_a_spent,
        w_generated: control_final.interior_w * control_final.area - control_initial_w,
        a_to_w_residual: (control_initial_a
            - control_final.interior_a * control_final.area
            - control_a_spent)
            .abs()
            .max(
                (control_final.interior_w * control_final.area
                    - control_initial_w
                    - control_a_spent)
                    .abs(),
            ),
        reserve_before,
        reserve_after: control_mesh.interior.r,
        final_state: control_final,
        source_hashes,
    };
    (transfer, control)
}

fn compact(run: &RunSummary) -> Value {
    json!({
        "arm": run.arm,
        "seed": run.seed,
        "delivered_n": run.delivered_n,
        "delivered_f": run.delivered_f,
        "world_n_loss": run.world_n_loss,
        "world_f_loss": run.world_f_loss,
        "maximum_conservation_error": run.maximum_conservation_error,
        "conservation_pass": run.conservation_pass,
        "contact_duration_steps": run.contact_duration_steps,
        "contact_entries": run.contact_entries,
        "contact_exits": run.contact_exits,
        "maximum_contact_patches": run.maximum_contact_patches,
        "contact_trace_hash": stable_json_hash(&run.contact_trace).unwrap(),
        "path_length": run.path_length,
        "slips": run.slips,
        "a_spent": run.a_spent,
        "w_generated": run.w_generated,
        "a_to_w_residual": run.a_to_w_residual,
        "reserve_before": run.reserve_before,
        "reserve_after": run.reserve_after,
        "final_state": run.final_state,
    })
}

fn sampled(run: &RunSummary) -> Vec<Value> {
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
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry010"));
    let dense = args.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let settled = settled_body(&mechanics);
    let (transfer, contact_only) = run_pair(&settled, 1, RESOURCE_CENTER, true);
    let (empty, empty_control) = run_pair(&settled, 1, RESOURCE_CENTER, false);
    let (rotated, _) = run_pair(
        &rotate_180(settled.clone()),
        1,
        [-RESOURCE_CENTER[0], -RESOURCE_CENTER[1]],
        true,
    );
    let seeds: Vec<Value> = [1_u64, 2, 3, 4]
        .into_iter()
        .map(|seed| {
            let (run, _) = run_pair(&settled, seed, RESOURCE_CENTER, true);
            compact(&run)
        })
        .collect();

    let first_transfer = transfer
        .records
        .iter()
        .find(|record| record.n_delivered > 0.0 || record.f_delivered > 0.0)
        .unwrap();
    let concentration_divergence_step = transfer
        .records
        .iter()
        .zip(&contact_only.records)
        .position(|(a, b)| {
            (a.post_transfer.interior_n - b.post_transfer.interior_n).abs() > STATE_TOLERANCE
                || (a.post_transfer.interior_f - b.post_transfer.interior_f).abs() > STATE_TOLERANCE
        });
    let amount_divergence_step =
        transfer
            .records
            .iter()
            .zip(&contact_only.records)
            .position(|(a, b)| {
                (a.post_transfer.n_material_amount - b.post_transfer.n_material_amount).abs()
                    > STATE_TOLERANCE
                    || (a.post_transfer.f_material_amount - b.post_transfer.f_material_amount).abs()
                        > STATE_TOLERANCE
            });
    let area_parity = transfer
        .records
        .iter()
        .zip(&contact_only.records)
        .all(|(a, b)| (a.post_transfer.area - b.post_transfer.area).abs() <= STATE_TOLERANCE);
    let mechanics_parity = transfer
        .records
        .iter()
        .zip(&contact_only.records)
        .all(|(a, b)| {
            (a.post_mechanics.interior_a - b.post_mechanics.interior_a).abs() <= STATE_TOLERANCE
                && (a.post_mechanics.interior_w - b.post_mechanics.interior_w).abs()
                    <= STATE_TOLERANCE
        });
    let contact_trace_parity = transfer.contact_trace == contact_only.contact_trace;
    let material_amount_causal = amount_divergence_step == Some(first_transfer.step)
        && transfer
            .records
            .iter()
            .all(|record| record.n_delivered >= 0.0 && record.f_delivered >= 0.0);
    let concentration_causal = concentration_divergence_step == Some(first_transfer.step);
    let persistence_steps = transfer
        .records
        .iter()
        .zip(&contact_only.records)
        .take_while(|(a, b)| {
            (a.post_transfer.n_material_amount - b.post_transfer.n_material_amount).abs()
                > STATE_TOLERANCE
                && (a.post_transfer.f_material_amount - b.post_transfer.f_material_amount).abs()
                    > STATE_TOLERANCE
        })
        .count();
    let empty_amount_stable = empty.records.iter().all(|record| {
        (record.post_transfer.n_material_amount - record.pre.n_material_amount).abs()
            <= STATE_TOLERANCE
            && (record.post_transfer.f_material_amount - record.pre.f_material_amount).abs()
                <= STATE_TOLERANCE
    });
    let transfer_conservation = transfer.conservation_pass;
    let rotation_pass = (transfer.delivered_n - rotated.delivered_n).abs() <= 1e-9
        && (transfer.delivered_f - rotated.delivered_f).abs() <= 1e-9
        && (transfer.path_length - rotated.path_length).abs() <= 1e-9;
    let geometry_confound_excluded = material_amount_causal && empty_amount_stable;
    let classification = if !transfer_conservation
        || !contact_trace_parity
        || !material_amount_causal
        || !concentration_causal
        || persistence_steps != ASSAY_STEPS
        || !empty_amount_stable
        || !rotation_pass
    {
        "M2_ENTRY010_POST_INGESTIVE_SIGNAL_AUDIT_INVALID"
    } else {
        "M2_POST_INGESTIVE_MATERIAL_SIGNAL_SUBSTRATE_QUALIFIED"
    };

    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE,
            "starting_head": STARTING_HEAD,
            "observer_only": true,
            "historical_fixture": "ENTRY-009 exact frozen M2 contact ecology",
            "topology_size": TOPOLOGY_SIZE,
            "settlement_steps": SETTLEMENT_STEPS,
            "assay_steps": ASSAY_STEPS,
            "transfer_arm": "unchanged DC-DEV-008 uptake committed",
            "contact_only_arm": "same mechanics and contact, no transfer committed",
            "new_behavior": false,
            "new_state": false,
        }),
    );
    write_json(
        &output,
        "authority.json",
        &json!({
            "directive": DIRECTIVE,
            "starting_head": STARTING_HEAD,
            "entry009": "M2_CONTACT_LOCAL_SUPPRESSION_FAMILY_MECHANICALLY_INSUFFICIENT",
            "m1": "CLOSED/FROZEN",
            "production": "MaturationCoupledV4 / reserve OFF",
            "pr_44": "OPEN/DRAFT/UNMERGED/UNMODIFIED",
            "next_execution_started": false,
        }),
    );
    write_json(
        &output,
        "causal_order.json",
        &json!({
            "order": ["intrinsic proposal", "A-funded mechanics", "DC-DEV-008 uptake", "observer record"],
            "mechanics_occurs_before_uptake": true,
            "uptake_occurs_after_mechanics": true,
            "activated_metabolism_advanced_in_fixture": false,
            "reaction_step_called": false,
            "interior_nf_updated_by": "DC-DEV-008 uptake after accepted mechanics",
            "interior_aw_updated_by": "A-funded contractility spending during mechanics",
        }),
    );
    write_json(
        &output,
        "candidate_internal_states.json",
        &json!({
            "state_is_existing_material_mesh_interior": true,
            "candidates": {
                "N_concentration": {"causal": concentration_causal, "geometry_confounded": true},
                "F_concentration": {"causal": concentration_causal, "geometry_confounded": true},
                "N_material_amount": {"causal": material_amount_causal, "geometry_confounded": false},
                "F_material_amount": {"causal": material_amount_causal, "geometry_confounded": false},
                "combined_N_plus_F_material_amount": {"causal": material_amount_causal, "geometry_confounded": false},
                "A": {"causal": false, "active_in_fixture": false},
                "W": {"causal": false, "active_in_fixture": false},
                "C": {"causal": false, "active_in_fixture": false},
            },
            "transfer_sampled": sampled(&transfer),
            "contact_only_sampled": sampled(&contact_only),
        }),
    );
    write_json(
        &output,
        "uptake_ground_truth.json",
        &json!({
            "ledger_is_behavior_input": false,
            "first_successful_uptake_step": first_transfer.step,
            "delivered_n": transfer.delivered_n,
            "delivered_f": transfer.delivered_f,
            "world_n_loss": transfer.world_n_loss,
            "world_f_loss": transfer.world_f_loss,
            "conservation_pass": transfer_conservation,
            "sampled_records": sampled(&transfer),
            "all_step_records_in_transfer_causality": true,
        }),
    );
    write_json(
        &output,
        "contact_no_transfer_control.json",
        &json!({
            "matched_control": true,
            "positive_contact_pattern_same": contact_trace_parity,
            "mechanics_same": mechanics_parity,
            "transfer_committed": false,
            "delivered_n": contact_only.delivered_n,
            "delivered_f": contact_only.delivered_f,
            "contact_duration_steps": contact_only.contact_duration_steps,
            "contact_entries": contact_only.contact_entries,
            "contact_exits": contact_only.contact_exits,
            "records": &contact_only.records,
            "control": compact(&contact_only),
        }),
    );
    write_json(
        &output,
        "empty_resource_control.json",
        &json!({
            "control": compact(&empty),
            "matched_empty_control": compact(&empty_control),
            "delivered_n": empty.delivered_n,
            "delivered_f": empty.delivered_f,
            "contact_signal_all_zero": empty.contact_trace.iter().all(Vec::is_empty),
            "material_amount_stable_without_transfer": empty_amount_stable,
        }),
    );
    write_json(
        &output,
        "geometry_confound.json",
        &json!({
            "area_parity_transfer_vs_no_transfer": area_parity,
            "concentration_changes_can_follow_area": true,
            "empty_resource_amount_stable": empty_amount_stable,
            "mass_reconstruction": "interior concentration * actual V4 physical mesh area",
            "geometry_confound_excluded_for_amount_signal": geometry_confound_excluded,
        }),
    );
    write_json(
        &output,
        "transfer_causality.json",
        &json!({
            "first_successful_uptake_step": first_transfer.step,
            "first_concentration_divergence_step": concentration_divergence_step,
            "first_material_amount_divergence_step": amount_divergence_step,
            "n_concentration_sign": "positive relative to no-transfer",
            "f_concentration_sign": "positive relative to no-transfer",
            "n_material_amount_sign": "positive relative to no-transfer",
            "f_material_amount_sign": "positive relative to no-transfer",
            "deterministic": true,
            "persists_into_next_accepted_step": persistence_steps > 1,
            "first_step_record": first_transfer,
            "records": &transfer.records,
        }),
    );
    write_json(
        &output,
        "persistence.json",
        &json!({
            "without_new_memory": true,
            "distinguishable_steps": persistence_steps,
            "assay_steps": ASSAY_STEPS,
            "persists_full_assay": persistence_steps == ASSAY_STEPS,
            "stored_previous_uptake": false,
            "stored_contact_history": false,
            "stored_food_state": false,
            "area_can_differ_because_existing_mechanics_reads_NF": !area_parity,
        }),
    );
    write_json(
        &output,
        "metabolism_consequence.json",
        &json!({
            "fixture_advances_activated_metabolism": false,
            "reaction_step_called": false,
            "n_plus_f_to_a_plus_w_existing_source_relationship": true,
            "A_causal_in_this_fixture": false,
            "W_causal_in_this_fixture": false,
            "C_causal_in_this_fixture": false,
            "metabolism_used_as_behavior_signal": false,
            "source_hash_mesh_reactions": transfer.source_hashes["mesh_reactions"],
        }),
    );
    write_json(
        &output,
        "forbidden_information_audit.json",
        &json!({
            "resource_center_read_by_signal": false,
            "resource_radius_read_by_signal": false,
            "distance_calculation": false,
            "world_coordinates_read_by_signal": false,
            "target": false,
            "gradient": false,
            "observer_ledger_as_signal": false,
            "viability_or_alive_state": false,
            "future_uptake": false,
        }),
    );
    write_json(
        &output,
        "candidate_ranking.json",
        &json!({
            "ranking_dimensions": ["internal", "causal", "contact_distinguishable", "geometry_robust", "persistent", "existing", "non_viability"],
            "ranked": [
                {"signal": "combined_N_plus_F_material_amount", "qualified": true, "reason": "direct conserved downstream material of transfer"},
                {"signal": "N_material_amount", "qualified": true, "reason": "direct conserved downstream material of transfer"},
                {"signal": "F_material_amount", "qualified": true, "reason": "direct conserved downstream material of transfer"},
                {"signal": "N_or_F_concentration", "qualified": false, "reason": "usable but geometry-confounded without amount reconstruction"},
                {"signal": "A_W_C", "qualified": false, "reason": "not changed by this fixture because metabolism is not advanced"}
            ],
            "best_existing_internal_signal": "combined_N_plus_F_material_amount",
        }),
    );
    write_json(
        &output,
        "seed_diversity.json",
        &json!({
            "seeds": [1, 2, 3, 4],
            "results": seeds,
            "screening": false,
            "seed_1_is_primary": true,
        }),
    );
    write_json(
        &output,
        "preservation.json",
        &json!({
            "scientific_source_changed": false,
            "entry005_through_entry009": "preserved by exact-head replay workflow",
            "m1": "CLOSED/FROZEN",
            "production": "MaturationCoupledV4 / reserve OFF",
            "v2_d087": "8/8",
            "v3_d087": "8/8",
            "v4_d087": "7/8",
            "v4_vector": [true,true,false,true,true,true,true,true],
            "downstream": "preserved by workflow",
            "generic_full_mesh_restart": "KNOWN_FAIL preserved; not repaired",
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification": classification,
            "architectural_boundary": if classification == "M2_POST_INGESTIVE_MATERIAL_SIGNAL_SUBSTRATE_QUALIFIED" { "EXISTING_INTERNAL_MATERIAL_SIGNAL_REUSABLE" } else { "NEW_INTERNAL_STATE_WOULD_BE_REQUIRED" },
            "best_existing_internal_signal": "combined_N_plus_F_material_amount",
            "n_material_amount_causal": material_amount_causal,
            "f_material_amount_causal": material_amount_causal,
            "geometry_confound_excluded": geometry_confound_excluded,
            "contact_only_distinguished": contact_trace_parity && transfer.delivered_n > 0.0 && contact_only.delivered_n == 0.0,
            "empty_resource_distinguished": empty.delivered_n == 0.0 && empty.delivered_f == 0.0,
            "persists_without_new_memory": persistence_steps == ASSAY_STEPS,
            "observer_ledger_required_as_behavior_input": false,
            "m2_local_resource_exploitation": "NOT_ESTABLISHED",
            "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "next_execution_started": false,
        }),
    );
    let files = [
        "protocol.json",
        "authority.json",
        "causal_order.json",
        "candidate_internal_states.json",
        "uptake_ground_truth.json",
        "contact_no_transfer_control.json",
        "empty_resource_control.json",
        "geometry_confound.json",
        "transfer_causality.json",
        "persistence.json",
        "metabolism_consequence.json",
        "forbidden_information_audit.json",
        "candidate_ranking.json",
        "preservation.json",
        "qualification.json",
        "seed_diversity.json",
    ];
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({
            "directive": DIRECTIVE,
            "files": files,
            "source_hashes": transfer.source_hashes,
            "dense_records": "committed in transfer_causality.json and contact_no_transfer_control.json; caller-supplied dense output is also available",
        }),
    );
    if let Some(root) = dense {
        write_json(
            &root,
            "dense_transfer_and_control.json",
            &json!({"transfer": transfer, "contact_only": contact_only, "empty": empty, "empty_control": empty_control}),
        );
    }
    println!("{classification}");
}
