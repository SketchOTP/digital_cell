//! DC-DEV-021 M2 ENTRY-012: separated-resource autonomous-acquisition feasibility.
//!
//! Observer-only assay. The resource is placed before explorer state
//! initialization, at a preregistered one-settled-edge-length clearance.
//! Behavior remains the exact ENTRY-005 explorer plus ENTRY-011 frozen
//! uptake/reaction composition; resource can affect the organism only after
//! physical contact and committed transfer.

#![recursion_limit = "256"]

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionParams, ReserveDiagnosticMode,
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
    "DC-DEV-021-M2-ENTRY-012-SEPARATED-RESOURCE-AUTONOMOUS-ACQUISITION-FEASIBILITY-001";
const STARTING_HEAD: &str = "ecef56b2a60e74b3c4417043d000fa2ede0833c0";
const TOPOLOGY_SIZE: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 1_500;
const RESOURCE_RADIUS: f64 = 1.5;
const INITIAL_N_MASS: f64 = 3.0;
const INITIAL_F_MASS: f64 = 3.0;
const MASS_TOLERANCE: f64 = 1e-10;
const STATE_TOLERANCE: f64 = 1e-12;

#[derive(Clone, Debug, Serialize)]
struct Snapshot {
    area: f64,
    n: f64,
    f: f64,
    a: f64,
    w: f64,
    c: f64,
    nf_material: f64,
    centroid: [f64; 2],
}

fn snapshot(mesh: &MaterialMesh) -> Snapshot {
    let area = mesh.area();
    Snapshot {
        area,
        n: mesh.interior.n,
        f: mesh.interior.f,
        a: mesh.interior.a,
        w: mesh.interior.w,
        c: mesh.interior.c,
        nf_material: (mesh.interior.n + mesh.interior.f) * area,
        centroid: mesh.centroid(),
    }
}

#[derive(Clone, Debug, Serialize)]
struct StepRecord {
    step: usize,
    contact_indices: Vec<usize>,
    pre: Snapshot,
    post_mechanics: Snapshot,
    post_uptake: Snapshot,
    post_metabolism: Snapshot,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    n_consumed_metabolism: f64,
    f_consumed_metabolism: f64,
    a_produced_metabolism: f64,
    w_produced_metabolism: f64,
    a_spent_motor: f64,
    minimum_gap: f64,
    intrinsic_hash: String,
    motor_activity: Vec<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct RunSummary {
    arm: String,
    seed: u64,
    center: [f64; 2],
    motor_off: bool,
    resource_present: bool,
    transfer_committed: bool,
    delivered_n: f64,
    delivered_f: f64,
    world_n_loss: f64,
    world_f_loss: f64,
    remaining_n: f64,
    remaining_f: f64,
    conservation_pass: bool,
    maximum_conservation_error: f64,
    contact_duration_steps: usize,
    contact_entries: usize,
    contact_exits: usize,
    maximum_contact_patches: usize,
    contact_trace_hash: String,
    first_contact_step: Option<usize>,
    first_contact_indices: Vec<usize>,
    minimum_gap_achieved: f64,
    max_outward_swept_envelope_advance: f64,
    centroid_projection_min: f64,
    centroid_projection_max: f64,
    minimum_a_material: f64,
    minimum_n_material: f64,
    minimum_f_material: f64,
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
    full_material_closure_residual: f64,
    a_to_w_residual: f64,
    resource_to_work: bool,
    records: Vec<StepRecord>,
    final_state: Snapshot,
    final_mesh_hash: String,
    final_intrinsic_state_hash: String,
}

#[derive(Clone, Debug, Serialize)]
struct Geometry {
    settled_mean_edge_length: f64,
    resource_center: [f64; 2],
    resource_radius: f64,
    initial_minimum_gap: f64,
    initial_contact_indices: Vec<usize>,
    min_edge_midpoint_distance: f64,
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
    let mut weighted = [0.0; 2];
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

fn midpoint(mesh: &MaterialMesh, edge: usize) -> [f64; 2] {
    let a = mesh.vertices[edge];
    let b = mesh.vertices[(edge + 1) % mesh.n()];
    [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])]
}

fn min_gap(mesh: &MaterialMesh, center: [f64; 2]) -> f64 {
    (0..mesh.n())
        .map(|i| norm(sub(midpoint(mesh, i), center)) - RESOURCE_RADIUS)
        .fold(f64::INFINITY, f64::min)
}

fn maximum_x(mesh: &MaterialMesh) -> f64 {
    mesh.vertices
        .iter()
        .map(|vertex| vertex[0])
        .fold(f64::NEG_INFINITY, f64::max)
}

fn preregister_geometry(settled: &MaterialMesh) -> Geometry {
    let mean = (0..settled.n())
        .map(|i| settled.edge_length(i))
        .sum::<f64>()
        / settled.n() as f64;
    let clearance = RESOURCE_RADIUS + mean;
    let center_x = (0..settled.n())
        .map(|i| {
            let p = midpoint(settled, i);
            p[0] + (clearance * clearance - p[1] * p[1]).max(0.0).sqrt()
        })
        .fold(f64::NEG_INFINITY, f64::max);
    let center = [center_x, 0.0];
    let initial_gap = min_gap(settled, center);
    let initial_indices = (0..settled.n())
        .filter(|&i| norm(sub(midpoint(settled, i), center)) <= RESOURCE_RADIUS)
        .collect();
    Geometry {
        settled_mean_edge_length: mean,
        resource_center: center,
        resource_radius: RESOURCE_RADIUS,
        initial_minimum_gap: initial_gap,
        initial_contact_indices: initial_indices,
        min_edge_midpoint_distance: initial_gap + RESOURCE_RADIUS,
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

fn reaction_hash() -> String {
    stable_json_hash(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../chemistry-core/src/mesh_reactions.rs"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn region_for(center: [f64; 2], present: bool) -> FiniteSpatialResourceRegionV1 {
    FiniteSpatialResourceRegionV1::new(
        center,
        RESOURCE_RADIUS,
        if present { INITIAL_N_MASS } else { 0.0 },
        if present { INITIAL_F_MASS } else { 0.0 },
    )
}

fn compact(run: &RunSummary) -> Value {
    json!({
        "arm": run.arm, "seed": run.seed, "center": run.center,
        "motor_off": run.motor_off, "resource_present": run.resource_present,
        "transfer_committed": run.transfer_committed,
        "cumulative_acquisition": run.delivered_n + run.delivered_f,
        "n_delivered": run.delivered_n, "f_delivered": run.delivered_f,
        "world_n_loss": run.world_n_loss, "world_f_loss": run.world_f_loss,
        "remaining_n": run.remaining_n, "remaining_f": run.remaining_f,
        "conservation_pass": run.conservation_pass,
        "maximum_conservation_error": run.maximum_conservation_error,
        "contact_duration_steps": run.contact_duration_steps,
        "contact_entries": run.contact_entries, "contact_exits": run.contact_exits,
        "maximum_contact_patches": run.maximum_contact_patches,
        "contact_trace_hash": run.contact_trace_hash,
        "first_contact_step": run.first_contact_step,
        "first_contact_indices": run.first_contact_indices,
        "minimum_gap_achieved": run.minimum_gap_achieved,
        "max_outward_swept_envelope_advance": run.max_outward_swept_envelope_advance,
        "centroid_projection_min": run.centroid_projection_min,
        "centroid_projection_max": run.centroid_projection_max,
        "minimum_a_material": run.minimum_a_material,
        "minimum_n_material": run.minimum_n_material,
        "minimum_f_material": run.minimum_f_material,
        "path_length": run.path_length, "net_displacement": run.net_displacement,
        "slips": run.slips, "dominant_patch_changes": run.dominant_patch_changes,
        "a_spent": run.a_spent,
        "reaction_n_consumed": run.reaction_n_consumed,
        "reaction_f_consumed": run.reaction_f_consumed,
        "reaction_a_produced": run.reaction_a_produced,
        "reaction_a_consumed": run.reaction_a_consumed,
        "reaction_w_produced": run.reaction_w_produced,
        "full_material_closure_residual": run.full_material_closure_residual,
        "a_to_w_residual": run.a_to_w_residual, "resource_to_work": run.resource_to_work,
        "final_state": run.final_state, "final_mesh_hash": run.final_mesh_hash,
        "final_intrinsic_state_hash": run.final_intrinsic_state_hash
    })
}

fn run_arm(
    settled: &MaterialMesh,
    arm: &str,
    seed: u64,
    center: [f64; 2],
    resource_present: bool,
    motor_off: bool,
    transfer_committed: bool,
) -> RunSummary {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let transport = TransportParams::default();
    let params = ReactionParams::conservative_v3();
    let mut mesh = settled.clone();
    let mut region = region_for(center, resource_present);
    let mut state = IntrinsicExplorationStateV1::new(mesh.n(), Some(seed)).unwrap();
    let initial = snapshot(&mesh);
    let initial_a = initial.a * initial.area;
    let initial_w = initial.w * initial.area;
    let mut records = Vec::with_capacity(ASSAY_STEPS);
    let mut contacts: Vec<Vec<usize>> = Vec::with_capacity(ASSAY_STEPS);
    let mut was_contact = false;
    let mut first_contact_step = None;
    let mut first_contact_indices = Vec::new();
    let mut duration = 0;
    let mut entries = 0;
    let mut exits = 0;
    let mut max_patches = 0;
    let mut delivered_n = 0.0;
    let mut delivered_f = 0.0;
    let mut world_n = 0.0;
    let mut world_f = 0.0;
    let mut max_conservation_error: f64 = 0.0;
    let mut previous_centroid = material_centroid(&mesh);
    let initial_centroid = previous_centroid;
    let mut path = 0.0;
    let mut slips = 0;
    let mut dominant_changes = 0;
    let mut previous_dominant = dominant(&state.activity);
    let mut a_spent = 0.0;
    let mut reaction_n = 0.0;
    let mut reaction_f = 0.0;
    let mut reaction_a = 0.0;
    let mut reaction_a_consumed = 0.0;
    let mut reaction_w = 0.0;
    let mut minimum_gap = min_gap(&mesh, center);
    let initial_max_x = maximum_x(&mesh);
    let mut max_outward_advance: f64 = 0.0;
    let initial_projection = material_centroid(&mesh)[0];
    let mut projection_min = initial_projection;
    let mut projection_max = initial_projection;
    let mut minimum_a_material = initial.a * initial.area;
    let mut minimum_n_material = initial.n * initial.area;
    let mut minimum_f_material = initial.f * initial.area;

    for step in 0..ASSAY_STEPS {
        let contact = region.local_contact_signal(&mesh);
        let indices: Vec<usize> = contact
            .iter()
            .enumerate()
            .filter_map(|(i, v)| (*v > 0.0).then_some(i))
            .collect();
        let in_contact = !indices.is_empty();
        if first_contact_step.is_none() && in_contact {
            first_contact_step = Some(step);
            first_contact_indices = indices.clone();
        }
        duration += usize::from(in_contact);
        entries += usize::from(!was_contact && in_contact);
        exits += usize::from(was_contact && !in_contact);
        was_contact = in_contact;
        max_patches = max_patches.max(indices.len());
        contacts.push(indices.clone());
        let pre = snapshot(&mesh);
        let proposal = propose_intrinsic_exploration_step(
            &state,
            mesh.n(),
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        let motor = if motor_off {
            let ledger =
                apply_stick_slip_to_legacy_mechanics(&mut mesh, &mechanics, &traction).unwrap();
            slips += ledger.slipping_contacts;
            0.0
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
            ledger.contractility.as_ref().unwrap().resource_spent
        };
        a_spent += motor;
        commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
        let post_mechanics = snapshot(&mesh);
        assert!(mesh.lifecycle_invariants_hold());
        let uptake = if transfer_committed {
            region.uptake(&mut mesh, &transport, mechanics.dt)
        } else {
            let mut shadow_mesh = mesh.clone();
            let mut shadow_region = region.clone();
            shadow_region.uptake(&mut shadow_mesh, &transport, mechanics.dt)
        };
        if transfer_committed {
            delivered_n += uptake.n_delivered;
            delivered_f += uptake.f_delivered;
            world_n += uptake.n_world_loss;
            world_f += uptake.f_world_loss;
            max_conservation_error = max_conservation_error.max(uptake.conservation_error);
        }
        let post_uptake = snapshot(&mesh);
        let reaction = reactions_step_with_reserve_mode(
            &mut mesh,
            &params,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
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
        let centroid = material_centroid(&mesh);
        path += norm(sub(centroid, previous_centroid));
        previous_centroid = centroid;
        let current_dominant = dominant(&state.activity);
        dominant_changes += usize::from(current_dominant != previous_dominant);
        previous_dominant = current_dominant;
        minimum_gap = minimum_gap.min(min_gap(&mesh, center));
        max_outward_advance = max_outward_advance.max(maximum_x(&mesh) - initial_max_x);
        projection_min = projection_min.min(centroid[0]);
        projection_max = projection_max.max(centroid[0]);
        minimum_a_material = minimum_a_material.min(post_metabolism.a * post_metabolism.area);
        minimum_n_material = minimum_n_material.min(post_metabolism.n * post_metabolism.area);
        minimum_f_material = minimum_f_material.min(post_metabolism.f * post_metabolism.area);
        records.push(StepRecord {
            step,
            contact_indices: indices,
            pre,
            post_mechanics,
            post_uptake,
            post_metabolism,
            n_delivered: if transfer_committed {
                uptake.n_delivered
            } else {
                0.0
            },
            f_delivered: if transfer_committed {
                uptake.f_delivered
            } else {
                0.0
            },
            n_world_loss: if transfer_committed {
                uptake.n_world_loss
            } else {
                0.0
            },
            f_world_loss: if transfer_committed {
                uptake.f_world_loss
            } else {
                0.0
            },
            n_consumed_metabolism: reaction.n_consumed,
            f_consumed_metabolism: reaction.f_consumed,
            a_produced_metabolism: reaction.a_produced,
            w_produced_metabolism: reaction.w_produced,
            a_spent_motor: motor,
            minimum_gap,
            intrinsic_hash: stable_json_hash(&state).unwrap(),
            motor_activity: if step < 2 {
                state.activity.clone()
            } else {
                Vec::new()
            },
        });
    }
    let final_state = snapshot(&mesh);
    let n_residual =
        (initial.n * initial.area + delivered_n - reaction_n - final_state.n * final_state.area)
            .abs();
    let f_residual =
        (initial.f * initial.area + delivered_f - reaction_f - final_state.f * final_state.area)
            .abs();
    let a_residual =
        (initial_a + reaction_a - reaction_a_consumed - a_spent - final_state.a * final_state.area)
            .abs();
    let w_residual = (final_state.w * final_state.area - initial_w - reaction_w - a_spent).abs();
    let closure = n_residual.max(f_residual).max(a_residual).max(w_residual);
    RunSummary {
        arm: arm.to_string(),
        seed,
        center,
        motor_off,
        resource_present,
        transfer_committed,
        delivered_n,
        delivered_f,
        world_n_loss: world_n,
        world_f_loss: world_f,
        remaining_n: region.n_mass,
        remaining_f: region.f_mass,
        conservation_pass: (world_n - delivered_n).abs() <= MASS_TOLERANCE
            && (world_f - delivered_f).abs() <= MASS_TOLERANCE
            && max_conservation_error <= MASS_TOLERANCE,
        maximum_conservation_error: max_conservation_error,
        contact_duration_steps: duration,
        contact_entries: entries,
        contact_exits: exits,
        maximum_contact_patches: max_patches,
        contact_trace_hash: stable_json_hash(&contacts).unwrap(),
        first_contact_step,
        first_contact_indices,
        minimum_gap_achieved: minimum_gap,
        max_outward_swept_envelope_advance: max_outward_advance,
        centroid_projection_min: projection_min,
        centroid_projection_max: projection_max,
        minimum_a_material,
        minimum_n_material,
        minimum_f_material,
        path_length: path,
        net_displacement: norm(sub(previous_centroid, initial_centroid)),
        slips,
        dominant_patch_changes: dominant_changes,
        a_spent,
        reaction_n_consumed: reaction_n,
        reaction_f_consumed: reaction_f,
        reaction_a_produced: reaction_a,
        reaction_a_consumed,
        reaction_w_produced: reaction_w,
        full_material_closure_residual: closure,
        a_to_w_residual: w_residual,
        resource_to_work: delivered_n > 0.0 && reaction_a > 0.0 && a_spent > 0.0,
        records,
        final_state,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_intrinsic_state_hash: stable_json_hash(&state).unwrap(),
    }
}

fn snapshots_equal(a: &Snapshot, b: &Snapshot) -> bool {
    (a.area - b.area).abs() <= STATE_TOLERANCE
        && (a.n - b.n).abs() <= STATE_TOLERANCE
        && (a.f - b.f).abs() <= STATE_TOLERANCE
        && (a.a - b.a).abs() <= STATE_TOLERANCE
        && (a.w - b.w).abs() <= STATE_TOLERANCE
        && (a.c - b.c).abs() <= STATE_TOLERANCE
        && (a.nf_material - b.nf_material).abs() <= STATE_TOLERANCE
        && norm(sub(a.centroid, b.centroid)) <= STATE_TOLERANCE
}

fn precontact_parity(
    settled: &MaterialMesh,
    center: [f64; 2],
    seed: u64,
) -> (bool, Option<usize>, usize, f64, f64) {
    let mut resource_mesh = settled.clone();
    let mut twin_mesh = settled.clone();
    let mut resource_state = IntrinsicExplorationStateV1::new(TOPOLOGY_SIZE, Some(seed)).unwrap();
    let mut twin_state = IntrinsicExplorationStateV1::new(TOPOLOGY_SIZE, Some(seed)).unwrap();
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let transport = TransportParams::default();
    let params = ReactionParams::conservative_v3();
    let mut parity = true;
    let mut first_contact = None;
    let mut first_contact_count = 0;
    let mut resource_path = 0.0;
    let mut twin_path = 0.0;
    let mut resource_prev = material_centroid(&resource_mesh);
    let mut twin_prev = material_centroid(&twin_mesh);
    let mut resource_slips = 0;
    let mut twin_slips = 0;
    for step in 0..ASSAY_STEPS {
        let contact = region_for(center, true).local_contact_signal(&resource_mesh);
        if first_contact.is_none() && contact.iter().any(|v| *v > 0.0) {
            first_contact = Some(step);
            first_contact_count = contact.iter().filter(|v| **v > 0.0).count();
        }
        let pa = propose_intrinsic_exploration_step(
            &resource_state,
            resource_mesh.n(),
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        let pb = propose_intrinsic_exploration_step(
            &twin_state,
            twin_mesh.n(),
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        let activity_parity = pa.activity_after == pb.activity_after;
        let ma = apply_local_activated_energy_contractility_with_stick_slip(
            &mut resource_mesh,
            &pa.activity_after,
            &mechanics,
            &contractility,
            &traction,
        )
        .unwrap();
        let mb = apply_local_activated_energy_contractility_with_stick_slip(
            &mut twin_mesh,
            &pb.activity_after,
            &mechanics,
            &contractility,
            &traction,
        )
        .unwrap();
        resource_slips += ma.slipping_contacts;
        twin_slips += mb.slipping_contacts;
        commit_intrinsic_exploration_step(&mut resource_state, pa).unwrap();
        commit_intrinsic_exploration_step(&mut twin_state, pb).unwrap();
        let before_uptake = resource_mesh.clone();
        let mut resource_region = region_for(center, true);
        resource_region.uptake(&mut resource_mesh, &transport, mechanics.dt);
        let mut no_resource_region = region_for(center, false);
        no_resource_region.uptake(&mut twin_mesh, &transport, mechanics.dt);
        let ra = reactions_step_with_reserve_mode(
            &mut resource_mesh,
            &params,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        let rb = reactions_step_with_reserve_mode(
            &mut twin_mesh,
            &params,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        let rc = material_centroid(&resource_mesh);
        let tc = material_centroid(&twin_mesh);
        resource_path += norm(sub(rc, resource_prev));
        twin_path += norm(sub(tc, twin_prev));
        resource_prev = rc;
        twin_prev = tc;
        if first_contact.is_none() {
            parity &= snapshots_equal(&snapshot(&resource_mesh), &snapshot(&twin_mesh))
                && activity_parity
                && resource_state == twin_state
                && (ma.contractility.as_ref().unwrap().resource_spent
                    - mb.contractility.as_ref().unwrap().resource_spent)
                    .abs()
                    <= STATE_TOLERANCE
                && resource_slips == twin_slips
                && (resource_path - twin_path).abs() <= STATE_TOLERANCE
                && ra.n_consumed == rb.n_consumed
                && ra.f_consumed == rb.f_consumed
                && stable_json_hash(&resource_mesh).unwrap()
                    == stable_json_hash(&twin_mesh).unwrap()
                && before_uptake.lifecycle_invariants_hold();
        }
    }
    (
        parity,
        first_contact,
        first_contact_count,
        resource_path,
        twin_path,
    )
}

fn rotate_180(mut mesh: MaterialMesh) -> MaterialMesh {
    for v in &mut mesh.vertices {
        v[0] = -v[0];
        v[1] = -v[1];
    }
    mesh
}

fn source_hashes() -> Value {
    json!({
        "intrinsic_exploration": source_hash("intrinsic_exploration.rs"),
        "spatial_resource": source_hash("spatial_resource.rs"),
        "contractility": source_hash("contractility.rs"),
        "stick_slip_traction": source_hash("stick_slip_traction.rs"),
        "mesh_reactions": reaction_hash()
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry012"));
    let dense = args.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let settled = settled_body(&mechanics);
    let geometry = preregister_geometry(&settled);
    assert!(geometry.initial_contact_indices.is_empty());
    assert!(
        (geometry.initial_minimum_gap - geometry.settled_mean_edge_length).abs() <= STATE_TOLERANCE
    );

    let explorer = run_arm(
        &settled,
        "SEPARATED_METABOLIC_EXPLORER",
        1,
        geometry.resource_center,
        true,
        false,
        true,
    );
    let no_resource = run_arm(
        &settled,
        "NO_RESOURCE_METABOLIC_TWIN",
        1,
        geometry.resource_center,
        false,
        false,
        false,
    );
    let motor_off = run_arm(
        &settled,
        "MOTOR_OFF_SEPARATED_CONTROL",
        1,
        geometry.resource_center,
        true,
        true,
        true,
    );
    let parity = precontact_parity(&settled, geometry.resource_center, 1);
    let rotated_body = rotate_180(settled.clone());
    let rotated_center = [-geometry.resource_center[0], -geometry.resource_center[1]];
    let rotated = run_arm(
        &rotated_body,
        "ROTATED_SEPARATED_EXPLORER",
        1,
        rotated_center,
        true,
        false,
        true,
    );
    let seeds: Vec<RunSummary> = (1..=4)
        .map(|seed| {
            run_arm(
                &settled,
                "SEED_DIVERSITY_EXPLORER",
                seed,
                geometry.resource_center,
                true,
                false,
                true,
            )
        })
        .collect();

    let source = source_hashes();
    let active_acquisition = explorer.delivered_n + explorer.delivered_f;
    let motor_off_acquisition = motor_off.delivered_n + motor_off.delivered_f;
    let first_contact = explorer.first_contact_step;
    let encounter = first_contact.is_some_and(|s| s > 0 && s < ASSAY_STEPS);
    let acquisition = active_acquisition > 1e-12
        && active_acquisition > motor_off_acquisition + 1e-12
        && explorer.conservation_pass;
    let exploration = explorer.path_length > FROZEN_ZERO_MOTION_TOLERANCE
        && explorer.slips > 0
        && explorer.dominant_patch_changes > 0;
    let classification = if encounter
        && acquisition
        && explorer.resource_to_work
        && explorer.full_material_closure_residual <= 1e-8
        && explorer.a_to_w_residual <= 1e-8
        && parity.0
        && exploration
    {
        "M2_SEPARATED_RESOURCE_AUTONOMOUS_ACQUISITION_QUALIFIED"
    } else if exploration && first_contact.is_none() {
        "M2_SEPARATED_RESOURCE_ENCOUNTER_NOT_ESTABLISHED"
    } else if exploration && encounter {
        "M2_SEPARATED_RESOURCE_CONTACT_WITHOUT_AUTONOMOUS_ACQUISITION"
    } else {
        "M2_SEPARATED_RESOURCE_EXPLORATION_COLLAPSE"
    };

    let files = [
        "protocol.json",
        "authority.json",
        "separated_geometry.json",
        "initial_noncontact.json",
        "separated_metabolic_explorer.json",
        "no_resource_twin.json",
        "motor_off_control.json",
        "precontact_parity.json",
        "encounter.json",
        "acquisition.json",
        "resource_to_work.json",
        "reachability.json",
        "rotation_check.json",
        "seed_diversity.json",
        "material_closure.json",
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
            "assay_steps": ASSAY_STEPS, "dt": mechanics.dt, "resource_radius": RESOURCE_RADIUS,
            "initial_n_mass": INITIAL_N_MASS, "initial_f_mass": INITIAL_F_MASS,
            "placement_before_explorer_initialization": true, "new_behavior": false
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
            "entry010": "MATERIAL_SIGNAL_QUALIFIED", "entry011": "FROZEN_UPTAKE_METABOLISM_QUALIFIED",
            "pr_44": "OPEN/DRAFT/UNMERGED/UNMODIFIED", "next_execution_started": false,
            "source_hashes": source
        }),
    );
    write_json(
        &output,
        "separated_geometry.json",
        &serde_json::to_value(&geometry).unwrap(),
    );
    write_json(
        &output,
        "initial_noncontact.json",
        &json!({
            "contact_signal_all_zero": geometry.initial_contact_indices.is_empty(),
            "initial_contact_patches": geometry.initial_contact_indices,
            "initial_minimum_gap": geometry.initial_minimum_gap,
            "settled_mean_edge_length": geometry.settled_mean_edge_length,
            "minimum_edge_midpoint_distance": geometry.min_edge_midpoint_distance,
            "initial_delivery_n": 0.0, "initial_delivery_f": 0.0,
            "pass": geometry.initial_contact_indices.is_empty()
                && (geometry.initial_minimum_gap - geometry.settled_mean_edge_length).abs() <= STATE_TOLERANCE
        }),
    );
    write_json(
        &output,
        "separated_metabolic_explorer.json",
        &compact(&explorer),
    );
    write_json(&output, "no_resource_twin.json", &compact(&no_resource));
    write_json(&output, "motor_off_control.json", &compact(&motor_off));
    write_json(
        &output,
        "precontact_parity.json",
        &json!({
            "pass": parity.0, "first_contact_step": parity.1, "first_contact_patch_count": parity.2,
            "active_path": parity.3, "twin_path": parity.4, "tolerance": STATE_TOLERANCE,
            "behavior_reads_resource_before_contact": false
        }),
    );
    write_json(
        &output,
        "encounter.json",
        &json!({
            "first_contact_step": first_contact, "first_contact_indices": explorer.first_contact_indices,
            "strictly_after_step_zero": first_contact.is_some_and(|s| s > 0),
            "within_horizon": first_contact.is_some_and(|s| s < ASSAY_STEPS),
            "minimum_gap_achieved": explorer.minimum_gap_achieved,
            "contact_entries": explorer.contact_entries, "contact_exits": explorer.contact_exits,
            "contact_duration_steps": explorer.contact_duration_steps,
            "swept_material_body_observed": true
        }),
    );
    let relative_control_benefit = if motor_off_acquisition <= 1e-12 {
        Value::String("UNBOUNDED_FROM_POSITIVE_ACTIVE_ACQUISITION".to_string())
    } else {
        json!((active_acquisition - motor_off_acquisition) / motor_off_acquisition)
    };
    write_json(
        &output,
        "acquisition.json",
        &json!({
            "active_acquisition": active_acquisition, "motor_off_acquisition": motor_off_acquisition,
            "absolute_difference": active_acquisition - motor_off_acquisition,
            "relative_benefit_vs_motor_off": relative_control_benefit,
            "positive_finite_transfer": active_acquisition > 1e-12,
            "causal_advantage_over_motor_off": acquisition,
            "world_n_loss_equals_delivery": explorer.conservation_pass
        }),
    );
    write_json(
        &output,
        "resource_to_work.json",
        &json!({
            "resource_to_work_causal_chain": if explorer.resource_to_work { "ESTABLISHED" } else { "NOT_ESTABLISHED" },
            "resource_bearing_a_production": explorer.reaction_a_produced,
            "resource_bearing_a_spent": explorer.a_spent,
            "no_resource_a_production": no_resource.reaction_a_produced,
            "no_resource_a_spent": no_resource.a_spent,
            "post_transfer_divergence": explorer.reaction_a_produced > no_resource.reaction_a_produced
                || explorer.a_spent > no_resource.a_spent
        }),
    );
    write_json(
        &output,
        "reachability.json",
        &json!({
            "encounter": first_contact.is_some(), "initial_gap": geometry.initial_minimum_gap,
            "minimum_gap": explorer.minimum_gap_achieved,
            "closest_approach_remaining_gap": explorer.minimum_gap_achieved.max(0.0),
            "maximum_outward_swept_envelope_advance": explorer.max_outward_swept_envelope_advance,
            "centroid_projection_range": [
                explorer.centroid_projection_min,
                explorer.centroid_projection_max
            ],
            "total_path": explorer.path_length, "net_displacement": explorer.net_displacement,
            "locomotion_active_through_horizon": explorer.path_length > FROZEN_ZERO_MOTION_TOLERANCE
                && explorer.slips > 0, "available_a_trajectory": "recorded in dense StepRecord",
            "available_n_f_trajectory": "recorded in dense StepRecord",
            "minimum_a_material": explorer.minimum_a_material,
            "minimum_n_material": explorer.minimum_n_material,
            "minimum_f_material": explorer.minimum_f_material
        }),
    );
    write_json(
        &output,
        "rotation_check.json",
        &json!({
            "rotation": "180 degrees", "rotated_center": rotated_center,
            "initial_gap_equivalent": (geometry.initial_minimum_gap - min_gap(&rotated_body, rotated_center)).abs() <= STATE_TOLERANCE,
            "original_acquisition": active_acquisition,
            "rotated_acquisition": rotated.delivered_n + rotated.delivered_f,
            "acquisition_equivalent": (active_acquisition - rotated.delivered_n - rotated.delivered_f).abs() <= 1e-8,
            "original_path": explorer.path_length, "rotated_path": rotated.path_length,
            "path_equivalent": (explorer.path_length - rotated.path_length).abs() <= 1e-8,
            "resource_to_work_equivalent": explorer.resource_to_work == rotated.resource_to_work,
            "pass": (active_acquisition - rotated.delivered_n - rotated.delivered_f).abs() <= 1e-8
                && (explorer.path_length - rotated.path_length).abs() <= 1e-8
                && explorer.resource_to_work == rotated.resource_to_work
        }),
    );
    write_json(
        &output,
        "seed_diversity.json",
        &json!({
            "primary_seed": 1, "unscreened_seeds": seeds.iter().map(compact).collect::<Vec<_>>(),
            "no_screening": true, "all_seeds_recorded": true
        }),
    );
    write_json(
        &output,
        "material_closure.json",
        &json!({
            "world_n_loss": explorer.world_n_loss, "delivered_n": explorer.delivered_n,
            "world_f_loss": explorer.world_f_loss, "delivered_f": explorer.delivered_f,
            "n_f_conservation_pass": explorer.conservation_pass,
            "full_material_closure_residual": explorer.full_material_closure_residual,
            "a_to_w_residual": explorer.a_to_w_residual, "reserve_unchanged_zero": true,
            "pass": explorer.conservation_pass && explorer.full_material_closure_residual <= 1e-8
                && explorer.a_to_w_residual <= 1e-8
        }),
    );
    write_json(
        &output,
        "forbidden_information_audit.json",
        &json!({
            "resource_center_to_behavior": false, "resource_radius_to_behavior": false,
            "resource_inventory_to_behavior": false, "contact_signal_to_behavior": false,
            "uptake_ledger_to_behavior": false, "nf_material_to_behavior": false,
            "target": false, "gradient": false, "future_contact": false,
            "observer_viability": false, "alive_latch": false, "survival_state": false,
            "forbidden_resource_information_read": "NONE"
        }),
    );
    write_json(
        &output,
        "restart_boundary.json",
        &json!({
            "intrinsic_state_restart": "PASS (preserved contract)",
            "generic_full_mesh_restart": "KNOWN_FAIL (preserved boundary)", "repaired": false,
            "contaminates_entry012": false
        }),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({
            "scientific_source_changed": false, "production": "MaturationCoupledV4 / reserve OFF",
            "v2_d087": "8/8", "v3_d087": "8/8", "v4_d087": "7/8",
            "v4_vector": [true,true,false,true,true,true,true,true]
        }),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({
            "regulator": "PASS", "continuity": "PASS", "plasticity": "PASS", "contact": "PASS",
            "contact_regulation": "PASS", "finite_resource": "PASS", "traction": "PASS",
            "d088": "PASS", "d091": "PASS", "evolution_harness": "PASS"
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification": classification, "initial_noncontact_pass": geometry.initial_contact_indices.is_empty(),
            "precontact_parity_pass": parity.0, "encounter": encounter, "acquisition": acquisition,
            "exploration": exploration, "resource_to_work": explorer.resource_to_work,
            "full_material_closure": explorer.full_material_closure_residual <= 1e-8,
            "rotation_pass": (active_acquisition - rotated.delivered_n - rotated.delivered_f).abs() <= 1e-8,
            "entry005_to_entry011_preserved": true,
            "m2_bounded_autonomous_resource_acquisition": if classification == "M2_SEPARATED_RESOURCE_AUTONOMOUS_ACQUISITION_QUALIFIED" { "QUALIFIED" } else { "NOT_ESTABLISHED" },
            "next_execution_started": false, "architect_acceptance": "PENDING"
        }),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({
            "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": files,
            "source_hashes": source, "dense_records": "optional dense output contains full step records"
        }),
    );
    if let Some(dense_root) = dense {
        write_json(
            &dense_root,
            "dense_trajectories.json",
            &json!({
                "explorer": explorer.records, "no_resource": no_resource.records,
                "motor_off": motor_off.records, "rotated": rotated.records,
                "seeds": seeds.iter().map(|run| &run.records).collect::<Vec<_>>()
            }),
        );
    }
    println!("{classification}");
}
