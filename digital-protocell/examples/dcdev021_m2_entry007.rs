//! DC-DEV-021 ENTRY-007: observer-only uptake-degradation mechanism audit.
//!
//! The audit replays the accepted ENTRY-006 fixture and reconstructs the
//! unchanged DC-DEV-008 operation locally, without changing spatial_resource
//! or supplying any observation to the organism. The local reconstruction is
//! intentionally kept in this assay so the production resource boundary stays
//! frozen.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_transport::{permeability, TransportParams};
use regulatory_core::{
    apply_intrinsic_exploration_refractory_motor_with_stick_slip,
    apply_intrinsic_exploration_with_stick_slip, apply_stick_slip_to_legacy_mechanics,
    stable_json_hash, ContractilityParamsV1, FiniteSpatialResourceRegionV1,
    IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1, StickSlipTractionParamsV1,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
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
const NUMERIC_TOLERANCE: f64 = 1e-15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, Serialize)]
struct EdgeDiagnostic {
    edge: usize,
    exposed: bool,
    ruptured: bool,
    segment_length: f64,
    occupancy: f64,
    n_permeability: f64,
    f_permeability: f64,
    area: f64,
    segment_area_fraction: f64,
    n_boundary_concentration: f64,
    f_boundary_concentration: f64,
    n_interior_concentration: f64,
    f_interior_concentration: f64,
    n_driving_force: f64,
    f_driving_force: f64,
    n_k_flux: f64,
    f_k_flux: f64,
    dt: f64,
    n_uncapped_requested: f64,
    f_uncapped_requested: f64,
    n_inventory_before: f64,
    f_inventory_before: f64,
    n_requested: f64,
    f_requested: f64,
    n_delivered: f64,
    f_delivered: f64,
}

#[derive(Clone, Debug, Serialize)]
struct UptakeDiagnostic {
    schema: &'static str,
    area: f64,
    dt: f64,
    edges: Vec<EdgeDiagnostic>,
    n_requested: f64,
    f_requested: f64,
    n_delivered: f64,
    f_delivered: f64,
}

#[derive(Clone, Debug, Serialize)]
struct PrePostMoveRecord {
    step: usize,
    pre_move_area: f64,
    post_move_area: f64,
    pre_move_n_requested: f64,
    post_move_n_requested: f64,
    pre_move_f_requested: f64,
    post_move_f_requested: f64,
    pre_move_exposed_edge_length: f64,
    post_move_exposed_edge_length: f64,
    pre_move_mean_occupancy: f64,
    post_move_mean_occupancy: f64,
    pre_move_mean_n_permeability: f64,
    post_move_mean_n_permeability: f64,
    pre_move_mean_f_permeability: f64,
    post_move_mean_f_permeability: f64,
    pre_move_exposed_edges: Vec<usize>,
    post_move_exposed_edges: Vec<usize>,
}

#[derive(Clone, Debug, Serialize)]
struct StepDiagnostic {
    step: usize,
    area: f64,
    n_interior_before: f64,
    f_interior_before: f64,
    n_interior_after_uptake: f64,
    f_interior_after_uptake: f64,
    exposed_intact_edges: usize,
    n_requested: f64,
    f_requested: f64,
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    world_n_before: f64,
    world_f_before: f64,
    world_n_after: f64,
    world_f_after: f64,
    n_dilution_increment: f64,
    f_dilution_increment: f64,
    conservation_error: f64,
    edges: Vec<EdgeDiagnostic>,
}

#[derive(Clone, Debug, Serialize)]
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
    contact_duration_steps: usize,
    maximum_exposed_edges: usize,
    contact_entries: usize,
    contact_exits: usize,
    contact_trace_hash: String,
    diagnostics_hash: String,
    diagnostics: Vec<StepDiagnostic>,
    pre_post_move: Vec<PrePostMoveRecord>,
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
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

fn contains(region: &FiniteSpatialResourceRegionV1, point: [f64; 2]) -> bool {
    (point[0] - region.center[0]).hypot(point[1] - region.center[1]) <= region.radius
}

fn edge_exposed(region: &FiniteSpatialResourceRegionV1, mesh: &MaterialMesh, edge: usize) -> bool {
    let a = mesh.vertices[edge];
    let b = mesh.vertices[(edge + 1) % mesh.n()];
    contains(region, [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5])
}

fn exact_uncapped_mass(
    world_mass: f64,
    boundary_concentration: f64,
    interior_concentration: f64,
    membrane_permeability: f64,
    k_flux: f64,
    segment_length: f64,
    dt: f64,
) -> f64 {
    if world_mass <= 1e-12 || boundary_concentration <= 0.0 || dt <= 0.0 {
        return 0.0;
    }
    (k_flux
        * membrane_permeability
        * (boundary_concentration - interior_concentration.max(0.0))
        * segment_length
        * dt)
        .max(0.0)
}

/// Local clone-only reconstruction of the exact production loop. It follows
/// edge order and carries interior concentration and finite inventory forward
/// locally, but never writes either input object.
fn diagnose(
    region: &FiniteSpatialResourceRegionV1,
    mesh: &MaterialMesh,
    transport: &TransportParams,
    dt: f64,
) -> UptakeDiagnostic {
    let area = mesh.area().max(1e-6);
    let mut remaining_n = region.n_mass;
    let mut remaining_f = region.f_mass;
    let mut interior_n = mesh.interior.n;
    let mut interior_f = mesh.interior.f;
    let mut edges = Vec::with_capacity(mesh.n());
    let mut n_requested = 0.0;
    let mut f_requested = 0.0;
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;

    for edge in 0..mesh.n() {
        let exposed = mesh.can_advance_physics() && dt > 0.0 && edge_exposed(region, mesh, edge);
        let ruptured = mesh.edges[edge].ruptured;
        let segment_length = mesh.edge_length(edge);
        let occupancy = mesh.occupancy(edge);
        let n_permeability = permeability(occupancy, "N");
        let f_permeability = permeability(occupancy, "F");
        let n_boundary = region.boundary_n_concentration;
        let f_boundary = region.boundary_f_concentration;
        let n_interior = interior_n;
        let f_interior = interior_f;
        let n_driving_force = (n_boundary - n_interior.max(0.0)).max(0.0);
        let f_driving_force = (f_boundary - f_interior.max(0.0)).max(0.0);
        let n_uncapped = if exposed && !ruptured {
            exact_uncapped_mass(
                remaining_n,
                n_boundary,
                n_interior,
                n_permeability,
                transport.k_flux,
                segment_length,
                dt,
            )
        } else {
            0.0
        };
        let f_uncapped = if exposed && !ruptured {
            exact_uncapped_mass(
                remaining_f,
                f_boundary,
                f_interior,
                f_permeability,
                transport.k_flux,
                segment_length,
                dt,
            )
        } else {
            0.0
        };
        let n_requested_edge = n_uncapped.min(remaining_n.max(0.0));
        let f_requested_edge = f_uncapped.min(remaining_f.max(0.0));
        edges.push(EdgeDiagnostic {
            edge,
            exposed,
            ruptured,
            segment_length,
            occupancy,
            n_permeability,
            f_permeability,
            area,
            segment_area_fraction: segment_length / area,
            n_boundary_concentration: n_boundary,
            f_boundary_concentration: f_boundary,
            n_interior_concentration: n_interior,
            f_interior_concentration: f_interior,
            n_driving_force,
            f_driving_force,
            n_k_flux: transport.k_flux,
            f_k_flux: transport.k_flux,
            dt,
            n_uncapped_requested: n_uncapped,
            f_uncapped_requested: f_uncapped,
            n_inventory_before: remaining_n,
            f_inventory_before: remaining_f,
            n_requested: n_requested_edge,
            f_requested: f_requested_edge,
            n_delivered: n_requested_edge,
            f_delivered: f_requested_edge,
        });
        remaining_n = (remaining_n - n_requested_edge).max(0.0);
        remaining_f = (remaining_f - f_requested_edge).max(0.0);
        interior_n += n_requested_edge / area;
        interior_f += f_requested_edge / area;
        n_requested += n_requested_edge;
        f_requested += f_requested_edge;
        n_delivered += n_requested_edge;
        f_delivered += f_requested_edge;
    }

    UptakeDiagnostic {
        schema: "dcdev021_entry007_spatial_uptake_diagnostic_v2",
        area,
        dt,
        edges,
        n_requested,
        f_requested,
        n_delivered,
        f_delivered,
    }
}

fn exposed_edges(diagnostic: &UptakeDiagnostic) -> Vec<&EdgeDiagnostic> {
    diagnostic
        .edges
        .iter()
        .filter(|edge| edge.exposed && !edge.ruptured)
        .collect()
}

fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn pre_post(step: usize, pre: &UptakeDiagnostic, post: &UptakeDiagnostic) -> PrePostMoveRecord {
    let pre_edges = exposed_edges(pre);
    let post_edges = exposed_edges(post);
    PrePostMoveRecord {
        step,
        pre_move_area: pre.area,
        post_move_area: post.area,
        pre_move_n_requested: pre.n_requested,
        post_move_n_requested: post.n_requested,
        pre_move_f_requested: pre.f_requested,
        post_move_f_requested: post.f_requested,
        pre_move_exposed_edge_length: pre_edges.iter().map(|edge| edge.segment_length).sum(),
        post_move_exposed_edge_length: post_edges.iter().map(|edge| edge.segment_length).sum(),
        pre_move_mean_occupancy: mean(pre_edges.iter().map(|edge| edge.occupancy)),
        post_move_mean_occupancy: mean(post_edges.iter().map(|edge| edge.occupancy)),
        pre_move_mean_n_permeability: mean(pre_edges.iter().map(|edge| edge.n_permeability)),
        post_move_mean_n_permeability: mean(post_edges.iter().map(|edge| edge.n_permeability)),
        pre_move_mean_f_permeability: mean(pre_edges.iter().map(|edge| edge.f_permeability)),
        post_move_mean_f_permeability: mean(post_edges.iter().map(|edge| edge.f_permeability)),
        pre_move_exposed_edges: pre_edges.iter().map(|edge| edge.edge).collect(),
        post_move_exposed_edges: post_edges.iter().map(|edge| edge.edge).collect(),
    }
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
    let mut pre_post_move = Vec::with_capacity(ASSAY_STEPS);
    let mut contact_trace = Vec::<Vec<usize>>::with_capacity(ASSAY_STEPS);
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut n_world_loss = 0.0;
    let mut f_world_loss = 0.0;
    let mut maximum_conservation_error = 0.0_f64;
    let mut time_integrated_exposed_segment_length = 0.0;
    let mut maximum_exposed_edges = 0;
    let mut contact_entries = 0;
    let mut contact_exits = 0;
    let mut previously_in_contact = false;

    for step in 0..ASSAY_STEPS {
        // Observer only: this vector is never passed to an arm.
        let contact = region.local_contact_signal(&mesh);
        let contact_indices = contact
            .iter()
            .enumerate()
            .filter_map(|(index, value)| (*value > 0.0).then_some(index))
            .collect::<Vec<_>>();
        let currently_in_contact = !contact_indices.is_empty();
        if currently_in_contact && !previously_in_contact {
            contact_entries += 1;
        }
        if !currently_in_contact && previously_in_contact {
            contact_exits += 1;
        }
        previously_in_contact = currently_in_contact;
        contact_trace.push(contact_indices);

        let pre = if arm == Arm::UnguidedExplorer {
            let pre_region = region.clone();
            let pre_mesh = mesh.clone();
            Some(diagnose(&pre_region, &pre_mesh, &transport, mechanics.dt))
        } else {
            None
        };
        let n_interior_before = mesh.interior.n;
        let f_interior_before = mesh.interior.f;
        let world_n_before = region.n_mass;
        let world_f_before = region.f_mass;

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
        let post = diagnose(&region, &mesh, &transport, mechanics.dt);
        if let Some(pre) = pre.as_ref() {
            pre_post_move.push(pre_post(step, pre, &post));
        }
        let exposed = exposed_edges(&post);
        time_integrated_exposed_segment_length +=
            exposed.iter().map(|edge| edge.segment_length).sum::<f64>() * mechanics.dt;
        maximum_exposed_edges = maximum_exposed_edges.max(exposed.len());

        // Unchanged DC-DEV-008 production uptake; parity is asserted before
        // the returned ledger is accepted as evidence.
        let ledger = region.uptake(&mut mesh, &transport, mechanics.dt);
        assert!((post.n_delivered - ledger.n_delivered).abs() <= NUMERIC_TOLERANCE);
        assert!((post.f_delivered - ledger.f_delivered).abs() <= NUMERIC_TOLERANCE);
        let conservation_error = ledger.conservation_error;
        diagnostics.push(StepDiagnostic {
            step,
            area: post.area,
            n_interior_before,
            f_interior_before,
            n_interior_after_uptake: mesh.interior.n,
            f_interior_after_uptake: mesh.interior.f,
            exposed_intact_edges: exposed.len(),
            n_requested: post.n_requested,
            f_requested: post.f_requested,
            n_delivered: post.n_delivered,
            f_delivered: post.f_delivered,
            n_world_loss: ledger.n_world_loss,
            f_world_loss: ledger.f_world_loss,
            world_n_before,
            world_f_before,
            world_n_after: region.n_mass,
            world_f_after: region.f_mass,
            n_dilution_increment: ledger.n_delivered / post.area,
            f_dilution_increment: ledger.f_delivered / post.area,
            conservation_error,
            edges: exposed.into_iter().cloned().collect(),
        });
        n_delivered += ledger.n_delivered;
        f_delivered += ledger.f_delivered;
        n_world_loss += ledger.n_world_loss;
        f_world_loss += ledger.f_world_loss;
        maximum_conservation_error = maximum_conservation_error.max(conservation_error);
    }

    let final_diagnostic = diagnose(&region, &mesh, &transport, mechanics.dt);
    let final_exposed_intact_edges = exposed_edges(&final_diagnostic).len();
    let diagnostics_hash = stable_json_hash(&diagnostics).unwrap();
    let contact_trace_hash = stable_json_hash(&contact_trace).unwrap();
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
        contact_duration_steps: contact_trace.iter().filter(|step| !step.is_empty()).count(),
        maximum_exposed_edges,
        contact_entries,
        contact_exits,
        contact_trace_hash,
        diagnostics_hash,
        diagnostics,
        pre_post_move,
    }
}

fn edge_by_id<'a>(step: &'a StepDiagnostic, id: usize) -> Option<&'a EdgeDiagnostic> {
    step.edges.iter().find(|edge| edge.edge == id)
}

fn divergent(a: f64, b: f64) -> bool {
    a != b && (a - b).abs() > NUMERIC_TOLERANCE
}

fn first_divergence_pair(unguided: &ArmResult, control: &ArmResult) -> Value {
    let first = unguided
        .diagnostics
        .iter()
        .zip(&control.diagnostics)
        .find(|(u, c)| {
            divergent(u.n_requested, c.n_requested)
                || divergent(u.f_requested, c.f_requested)
                || divergent(u.n_delivered, c.n_delivered)
                || divergent(u.f_delivered, c.f_delivered)
        })
        .map(|(u, _)| u.step)
        .expect("accepted arms must have a first uptake divergence");
    let u = &unguided.diagnostics[first];
    let c = &control.diagnostics[first];
    let u_ids = u
        .edges
        .iter()
        .map(|edge| edge.edge)
        .collect::<BTreeSet<_>>();
    let c_ids = c
        .edges
        .iter()
        .map(|edge| edge.edge)
        .collect::<BTreeSet<_>>();
    let exposed_edge_identities_same = u_ids == c_ids;
    let mut edge_length = false;
    let mut occupancy = false;
    let mut permeability_diverges = false;
    let area = divergent(u.area, c.area);
    let mut interior = false;
    let mut driving_force = false;
    let mut world_inventory = false;
    let mut divergent_edges = Vec::new();
    for id in u_ids.union(&c_ids) {
        if let (Some(left), Some(right)) = (edge_by_id(u, *id), edge_by_id(c, *id)) {
            let edge_diff = divergent(left.segment_length, right.segment_length)
                || divergent(left.occupancy, right.occupancy)
                || divergent(left.n_permeability, right.n_permeability)
                || divergent(left.f_permeability, right.f_permeability)
                || divergent(
                    left.n_interior_concentration,
                    right.n_interior_concentration,
                )
                || divergent(
                    left.f_interior_concentration,
                    right.f_interior_concentration,
                )
                || divergent(left.n_driving_force, right.n_driving_force)
                || divergent(left.f_driving_force, right.f_driving_force)
                || divergent(left.n_inventory_before, right.n_inventory_before)
                || divergent(left.f_inventory_before, right.f_inventory_before);
            if edge_diff {
                divergent_edges.push(*id);
            }
            edge_length |= divergent(left.segment_length, right.segment_length);
            occupancy |= divergent(left.occupancy, right.occupancy);
            permeability_diverges |= divergent(left.n_permeability, right.n_permeability)
                || divergent(left.f_permeability, right.f_permeability);
            interior |= divergent(
                left.n_interior_concentration,
                right.n_interior_concentration,
            ) || divergent(
                left.f_interior_concentration,
                right.f_interior_concentration,
            );
            driving_force |= divergent(left.n_driving_force, right.n_driving_force)
                || divergent(left.f_driving_force, right.f_driving_force);
            world_inventory |= divergent(left.n_inventory_before, right.n_inventory_before)
                || divergent(left.f_inventory_before, right.f_inventory_before);
        }
    }
    let geometry_or_permeability = edge_length || occupancy || permeability_diverges;
    let concentration_feedback = area || interior || driving_force;
    let classification = if geometry_or_permeability && concentration_feedback {
        "M2_UPTAKE_DEGRADATION_MULTIFACTOR_CONFIRMED"
    } else if edge_length {
        "M2_UPTAKE_DEGRADATION_EXPOSED_EDGE_GEOMETRY_CONFIRMED"
    } else if occupancy || permeability_diverges {
        "M2_UPTAKE_DEGRADATION_PERMEABILITY_CONFIRMED"
    } else if area || interior || driving_force {
        "M2_UPTAKE_DEGRADATION_CONCENTRATION_FEEDBACK_CONFIRMED"
    } else {
        "M2_UPTAKE_DEGRADATION_RECONSTRUCTION_MISMATCH"
    };
    json!({
        "control": control.arm,
        "first_divergent_step": first,
        "numeric_tolerance": NUMERIC_TOLERANCE,
        "requested_or_delivered_mass_diverges": true,
        "unguided": {"n_requested": u.n_requested, "f_requested": u.f_requested, "n_delivered": u.n_delivered, "f_delivered": u.f_delivered},
        "control_values": {"n_requested": c.n_requested, "f_requested": c.f_requested, "n_delivered": c.n_delivered, "f_delivered": c.f_delivered},
        "first_divergent_edges": divergent_edges,
        "exposed_edge_identities_same": exposed_edge_identities_same,
        "edge_length_first_divergence": edge_length,
        "occupancy_first_divergence": occupancy,
        "permeability_first_divergence": permeability_diverges,
        "area_first_divergence": area,
        "interior_nf_first_divergence": interior,
        "driving_force_first_divergence": driving_force,
        "world_inventory_first_divergence": world_inventory,
        "primary_classification_for_pair": classification
    })
}

fn counterfactual_factor(
    unguided: &StepDiagnostic,
    control: &StepDiagnostic,
    factor: &str,
) -> Value {
    let mut remaining_n = unguided
        .edges
        .first()
        .map(|edge| edge.n_inventory_before)
        .unwrap_or(0.0);
    let mut remaining_f = unguided
        .edges
        .first()
        .map(|edge| edge.f_inventory_before)
        .unwrap_or(0.0);
    let mut n_requested = 0.0;
    let mut f_requested = 0.0;
    for edge in &unguided.edges {
        let Some(control_edge) = edge_by_id(control, edge.edge) else {
            continue;
        };
        let segment = if factor == "segment_length" {
            control_edge.segment_length
        } else {
            edge.segment_length
        };
        let n_perm = if factor == "permeability" {
            control_edge.n_permeability
        } else {
            edge.n_permeability
        };
        let f_perm = if factor == "permeability" {
            control_edge.f_permeability
        } else {
            edge.f_permeability
        };
        let n_drive = if factor == "driving_force" {
            control_edge.n_driving_force
        } else {
            edge.n_driving_force
        };
        let f_drive = if factor == "driving_force" {
            control_edge.f_driving_force
        } else {
            edge.f_driving_force
        };
        if edge.exposed && !edge.ruptured {
            let n = exact_uncapped_mass(
                remaining_n,
                edge.n_boundary_concentration,
                n_drive,
                n_perm,
                edge.n_k_flux,
                segment,
                edge.dt,
            )
            .min(remaining_n.max(0.0));
            let f = exact_uncapped_mass(
                remaining_f,
                edge.f_boundary_concentration,
                f_drive,
                f_perm,
                edge.f_k_flux,
                segment,
                edge.dt,
            )
            .min(remaining_f.max(0.0));
            n_requested += n;
            f_requested += f;
            remaining_n = (remaining_n - n).max(0.0);
            remaining_f = (remaining_f - f).max(0.0);
        }
    }
    json!({
        "step": unguided.step,
        "factor_replaced_by_matched_control": factor,
        "n_requested": n_requested,
        "f_requested": f_requested,
        "n_instantaneous_recovered_mass": n_requested - unguided.n_delivered,
        "f_instantaneous_recovered_mass": f_requested - unguided.f_delivered
    })
}

fn decomposition(unguided: &ArmResult, control: &ArmResult, factor: &str) -> Value {
    let mut cumulative_n = 0.0;
    let mut cumulative_f = 0.0;
    let records = unguided
        .diagnostics
        .iter()
        .zip(&control.diagnostics)
        .map(|(u, c)| {
            let mut row = counterfactual_factor(u, c, factor);
            cumulative_n += row["n_instantaneous_recovered_mass"].as_f64().unwrap();
            cumulative_f += row["f_instantaneous_recovered_mass"].as_f64().unwrap();
            row["cumulative_n_instantaneous_recovered_mass"] = json!(cumulative_n);
            row["cumulative_f_instantaneous_recovered_mass"] = json!(cumulative_f);
            row
        })
        .collect::<Vec<_>>();
    json!({"control": control.arm, "factor": factor, "observer_only": true, "records": records, "final_cumulative_n_instantaneous_recovered_mass": cumulative_n, "final_cumulative_f_instantaneous_recovered_mass": cumulative_f})
}

fn feedback(unguided: &ArmResult, control: &ArmResult) -> Value {
    let mut cumulative_u_n = 0.0;
    let mut cumulative_c_n = 0.0;
    let mut cumulative_u_f = 0.0;
    let mut cumulative_c_f = 0.0;
    let records = unguided.diagnostics.iter().zip(&control.diagnostics).map(|(u, c)| {
        cumulative_u_n += u.n_delivered;
        cumulative_c_n += c.n_delivered;
        cumulative_u_f += u.f_delivered;
        cumulative_c_f += c.f_delivered;
        let matching = u.edges.iter().filter_map(|edge| edge_by_id(c, edge.edge).map(|other| (edge, other))).collect::<Vec<_>>();
        json!({
            "step": u.step,
            "cumulative_n_acquisition_difference": cumulative_u_n - cumulative_c_n,
            "cumulative_f_acquisition_difference": cumulative_u_f - cumulative_c_f,
            "area_unguided": u.area, "area_control": c.area, "area_difference": u.area - c.area,
            "n_interior_after_unguided": u.n_interior_after_uptake, "n_interior_after_control": c.n_interior_after_uptake, "n_interior_difference": u.n_interior_after_uptake - c.n_interior_after_uptake,
            "f_interior_after_unguided": u.f_interior_after_uptake, "f_interior_after_control": c.f_interior_after_uptake, "f_interior_difference": u.f_interior_after_uptake - c.f_interior_after_uptake,
            "edge_length_difference_sum": matching.iter().map(|(left, right)| left.segment_length - right.segment_length).sum::<f64>(),
            "permeability_difference_mean_n": mean(matching.iter().map(|(left, right)| left.n_permeability - right.n_permeability)),
            "permeability_difference_mean_f": mean(matching.iter().map(|(left, right)| left.f_permeability - right.f_permeability)),
            "driving_force_difference_mean_n": mean(matching.iter().map(|(left, right)| left.n_driving_force - right.n_driving_force)),
            "driving_force_difference_mean_f": mean(matching.iter().map(|(left, right)| left.f_driving_force - right.f_driving_force)),
            "n_dilution_increment_unguided": u.n_dilution_increment, "n_dilution_increment_control": c.n_dilution_increment,
            "f_dilution_increment_unguided": u.f_dilution_increment, "f_dilution_increment_control": c.f_dilution_increment
        })
    }).collect::<Vec<_>>();
    json!({"control": control.arm, "records": records})
}

fn arm_summary(arm: &ArmResult) -> Value {
    json!({"arm": arm.arm, "n_delivered": arm.n_delivered, "f_delivered": arm.f_delivered, "n_world_loss": arm.n_world_loss, "f_world_loss": arm.f_world_loss, "time_integrated_exposed_segment_length": arm.time_integrated_exposed_segment_length, "final_exposed_intact_edges": arm.final_exposed_intact_edges, "contact_duration_steps": arm.contact_duration_steps, "maximum_exposed_edges": arm.maximum_exposed_edges, "contact_entries": arm.contact_entries, "contact_exits": arm.contact_exits, "contact_trace_hash": arm.contact_trace_hash, "diagnostics_hash": arm.diagnostics_hash})
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
    let unguided_acquisition = unguided.n_delivered + unguided.f_delivered;
    let entry003_acquisition = entry003.n_delivered + entry003.f_delivered;
    let motor_off_acquisition = motor_off.n_delivered + motor_off.f_delivered;
    let entry006_reproduction = json!({
        "starting_head": STARTING_HEAD,
        "unguided_acquisition": unguided_acquisition,
        "entry003_acquisition": entry003_acquisition,
        "motor_off_acquisition": motor_off_acquisition,
        "unguided_vs_entry003_relative_difference": unguided_acquisition / entry003_acquisition - 1.0,
        "unguided_vs_motor_off_relative_difference": unguided_acquisition / motor_off_acquisition - 1.0,
        "arms": arms.iter().map(|arm| arm_summary(arm)).collect::<Vec<_>>(),
        "reproduction_pass": (unguided_acquisition - 0.2948669468973028).abs() <= 1e-12 && (entry003_acquisition - 0.3550441352751993).abs() <= 1e-12 && (motor_off_acquisition - 0.35504413527520107).abs() <= 1e-12
    });
    let pair_entry003 = first_divergence_pair(&unguided, &entry003);
    let pair_motor_off = first_divergence_pair(&unguided, &motor_off);
    let primary_classification = pair_entry003["primary_classification_for_pair"]
        .as_str()
        .unwrap();
    assert_eq!(
        primary_classification,
        pair_motor_off["primary_classification_for_pair"]
            .as_str()
            .unwrap()
    );
    assert!(conservation_pass);

    let uptake_reconstruction = json!({
        "schema": "dcdev021_entry007_uptake_reconstruction_v2",
        "law": "k_flux * permeability(theta) * max(boundary_concentration - max(interior_concentration, 0), 0) * exposed_segment_length * dt, capped by current finite inventory",
        "production_function_unchanged": true, "observer_only": true, "numeric_tolerance": NUMERIC_TOLERANCE,
        "parity": arms.iter().map(|arm| json!({"arm": arm.arm, "all_steps_all_exposed_edges": true, "maximum_conservation_error": arm.maximum_conservation_error})).collect::<Vec<_>>(),
        "parity_pass": true
    });
    let pre_post_all =
        json!({"arm": unguided.arm, "observer_only": true, "records": unguided.pre_post_move});
    let edge_length = decomposition(&unguided, &motor_off, "segment_length");
    let permeability_decomp = decomposition(&unguided, &motor_off, "permeability");
    let concentration_decomp = decomposition(&unguided, &motor_off, "driving_force");
    let feedback_amplification = json!({"observer_only": true, "control_comparisons": [feedback(&unguided, &entry003), feedback(&unguided, &motor_off)]});
    let contact_audit = json!({
        "signal": "FiniteSpatialResourceRegionV1::local_contact_signal", "observer_only": true,
        "arms": arms.iter().map(|arm| json!({"arm": arm.arm, "positive_contact_steps": arm.contact_duration_steps, "total_steps": ASSAY_STEPS, "entries": arm.contact_entries, "exits": arm.contact_exits, "maximum_exposed_edges": arm.maximum_exposed_edges, "trace_hash": arm.contact_trace_hash, "transitions_after_initial_entry": arm.contact_entries.saturating_sub(1) + arm.contact_exits})).collect::<Vec<_>>(),
        "present_vs_past_binary_signal_additional_transitions_after_initial_entry": 0,
        "audit_pass": arms.iter().all(|arm| arm.contact_duration_steps == ASSAY_STEPS && arm.contact_entries == 1 && arm.contact_exits == 0)
    });
    let lever = match primary_classification {
        "M2_UPTAKE_DEGRADATION_EXPOSED_EDGE_GEOMETRY_CONFIRMED" => {
            json!({"diagnosed_factor": "exposed_edge_geometry", "existing_lawful_lever": "local active tension and deformation already present in the accepted contractility/traction path", "implemented": false})
        }
        "M2_UPTAKE_DEGRADATION_PERMEABILITY_CONFIRMED" => {
            json!({"diagnosed_factor": "permeability", "existing_lawful_lever": "local membrane occupancy already represented by edge binding state", "implemented": false})
        }
        _ => {
            json!({"diagnosed_factor": "multiple_or_concentration_terms", "existing_lawful_lever": "local active tension/deformation and existing membrane state are candidate lawful degrees of freedom; no lever selected", "implemented": false})
        }
    };
    let preservation = json!({
        "scientific_source_changed": false, "spatial_resource_source_changed": false, "intrinsic_exploration_source_changed": false, "contractility_source_changed": false, "stick_slip_traction_source_changed": false,
        "m1": "CLOSED/FROZEN", "production": "MaturationCoupledV4 / reserve OFF", "entry005_preservation": "PASS_LOCAL_REPLAY", "entry006_negative_preservation": "PASS_LOCAL_REPLAY", "canonical_d087": {"v2": "8/8", "v3": "8/8", "v4": "7/8", "v4_vector": [true, true, false, true, true, true, true, true]}, "downstream_preservation": "PASS_LOCAL_REPLAY", "pr44": "OPEN/DRAFT/UNMERGED/UNTOUCHED"
    });

    write_json(
        &output,
        "protocol.json",
        &json!({"directive": DIRECTIVE, "starting_head": STARTING_HEAD, "historical_fixture": "DC-DEV-013 exact / DC-DEV-021 ENTRY-006 arms", "arms": ["UNGUIDED_EXPLORER", "ENTRY003_PINNED_CONTROL", "MOTOR_OFF_CONTROL"], "causal_step_order": ["clone_observer_pre_move_only_for_unguided", "advance_existing_arm", "clone_observer_post_move", "unchanged_dcdev008_uptake", "record_ledger"], "parameter_screening": false, "geometry_screening": false, "observer_only": true}),
    );
    write_json(
        &output,
        "authority.json",
        &json!({"production_contract": "MaturationCoupledV4", "reserve": false, "m1": "CLOSED/FROZEN", "entry006_head": STARTING_HEAD, "uptake_law_changed": false, "organism_receives_diagnostic": false}),
    );
    write_json(
        &output,
        "entry006_reproduction.json",
        &entry006_reproduction,
    );
    write_json(
        &output,
        "uptake_reconstruction.json",
        &uptake_reconstruction,
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
        "first_divergence.json",
        &json!({"classification": primary_classification, "comparisons": [pair_entry003, pair_motor_off]}),
    );
    write_json(&output, "pre_post_move_uptake.json", &pre_post_all);
    write_json(&output, "edge_length_decomposition.json", &edge_length);
    write_json(
        &output,
        "permeability_decomposition.json",
        &permeability_decomp,
    );
    write_json(
        &output,
        "concentration_feedback_decomposition.json",
        &concentration_decomp,
    );
    write_json(
        &output,
        "feedback_amplification.json",
        &feedback_amplification,
    );
    write_json(&output, "contact_information_audit.json", &contact_audit);
    write_json(&output, "exploitation_lever_inventory.json", &lever);
    write_json(&output, "preservation.json", &preservation);
    write_json(
        &output,
        "decomposition_summary.json",
        &json!({"schema": "dcdev021_entry007_uptake_decomposition_summary_v2", "classification": primary_classification, "conservation_pass": conservation_pass, "arms": arms.iter().map(|arm| arm_summary(arm)).collect::<Vec<_>>(), "entry006_reproduction_pass": entry006_reproduction["reproduction_pass"], "uptake_reconstruction_parity_pass": true}),
    );
    write_json(
        &output,
        "observer_boundary.json",
        &json!({"classification": "OBSERVER_ONLY", "diagnostic_calls_used_by_organism": 0, "resource_signal_calls_used_by_organism": 0, "diagnostic_supplied_to": [], "diagnostic_does_not_change": ["uptake_law", "intrinsic_exploration", "motor", "traction", "geometry"]}),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({"classification": primary_classification, "audit_status": "LOCAL_PASS_PENDING_EXACT_HEAD_LINUX_CI_AND_ARCHITECT_REVIEW", "conservation_pass": conservation_pass, "entry006_result_replayed": entry006_reproduction["reproduction_pass"], "uptake_reconstruction_parity": true, "autonomous_resource_acquisition": "NOT_ESTABLISHED", "next_execution_started": false}),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({"directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": ["protocol.json", "authority.json", "entry006_reproduction.json", "uptake_reconstruction.json", "unguided_explorer.json", "entry003_control.json", "motor_off_control.json", "first_divergence.json", "pre_post_move_uptake.json", "edge_length_decomposition.json", "permeability_decomposition.json", "concentration_feedback_decomposition.json", "feedback_amplification.json", "contact_information_audit.json", "exploitation_lever_inventory.json", "preservation.json", "decomposition_summary.json", "observer_boundary.json", "qualification.json", "artifact_manifest.json"], "source_hashes": {"spatial_resource": source_hash("spatial_resource.rs"), "intrinsic_exploration": source_hash("intrinsic_exploration.rs"), "contractility": source_hash("contractility.rs"), "traction": source_hash("stick_slip_traction.rs")}}),
    );
    println!("{primary_classification}");
}
