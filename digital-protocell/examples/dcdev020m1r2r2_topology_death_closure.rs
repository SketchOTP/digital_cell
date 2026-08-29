//! DC-DEV-020-M1-R2-R2 observer-only topology-death closure.
//!
//! This assay continues the accepted M1-R2 endpoint through observer collapse
//! until the existing production rupture rule actually marks an edge.  It
//! then replays that exact ruptured state under ordinary finite refeeding and
//! the already-qualified source-capacity upper-bound shadow.  No production
//! rule, death predicate, resource law, or chemistry-core implementation is
//! changed.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod m1r1;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::MechParams;
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::FiniteSpatialResourceRegionV1;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R2-R2-TOPOLOGY-DEATH-CLOSURE-001";
const STARTING_HEAD: &str = "40a066424a5a0fe08db9609c4ec71a708b44115f";
const R2_COMPARISON_STEPS: usize = 480;
const R2_CONTINUATION_STEPS: usize = 20_000;
const RUPTURE_SEARCH_STEPS: usize = 1_000_000;
const REFEED_STEPS: usize = 5_000;
const DT: f64 = 0.02;
const PRODUCTION_K_A_DECAY: f64 = 0.008;
const ORDINARY_K_A_DECAY: f64 = 0.002;
const STARVATION_MULTIPLIER: f64 = 4.0;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const RESTORED_N: f64 = 14.588954880632265;
const RESTORED_F: f64 = 14.588954880632265;
const TOL: f64 = 1e-8;

#[derive(Debug, Clone, Copy, Serialize)]
struct Snapshot {
    step: usize,
    area: f64,
    n: f64,
    f: f64,
    a: f64,
    c: f64,
    structural_m: f64,
    organized_material: f64,
    strict_material: f64,
    observer_viable: bool,
    alive: bool,
    closed_intact: bool,
    physical_runtime_valid: bool,
    ruptured_edges: usize,
    min_edge_m: f64,
    max_edge_m: f64,
    observer_death_reason: Option<&'static str>,
}

impl Snapshot {
    fn from_mesh(mesh: &MaterialMesh, step: usize) -> Self {
        let s = snapshot(mesh);
        let mut edge_masses: Vec<f64> = mesh.edges.iter().map(|edge| edge.m).collect();
        edge_masses.sort_by(|a, b| a.total_cmp(b));
        Self {
            step,
            area: mesh.area(),
            n: s.n,
            f: s.f,
            a: s.a,
            c: s.c,
            structural_m: s.structural_m,
            organized_material: s.organized_material(),
            strict_material: s.strict_material_equivalent(),
            observer_viable: mesh.observer_viable(),
            alive: mesh.alive,
            closed_intact: mesh.closed_intact(),
            physical_runtime_valid: mesh.physical_runtime_valid(),
            ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
            min_edge_m: edge_masses.first().copied().unwrap_or(0.0),
            max_edge_m: edge_masses.last().copied().unwrap_or(0.0),
            observer_death_reason: mesh.observer_death_reason(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct RefeedEvidence {
    mode: String,
    requested_steps: usize,
    accepted_steps: usize,
    initial: Snapshot,
    final_state: Snapshot,
    n_requested: f64,
    f_requested: f64,
    n_delivered: f64,
    f_delivered: f64,
    n_remaining: f64,
    f_remaining: f64,
    world_to_organism_closure_residual: f64,
    strict_material_delta: f64,
    closed_intact_before: bool,
    closed_intact_after: bool,
    ruptured_edges_before: usize,
    ruptured_edges_after: usize,
    observer_viable_before: bool,
    observer_viable_after: bool,
    source_capacity_shadow: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ArmEvidence {
    id: String,
    declared_k_a_decay: f64,
    effective_starvation_k_a_decay: f64,
    r2_endpoint: Snapshot,
    rupture_search_start: Snapshot,
    rupture_state: Option<Snapshot>,
    rupture_search_accepted_steps: usize,
    first_edge_rupture_step: Option<usize>,
    rupture_search_terminal_reason: Option<String>,
    ordinary_finite_refeed: Option<RefeedEvidence>,
    source_capacity_refeed: Option<RefeedEvidence>,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL
}

fn reaction_params(k_a_decay: f64) -> ReactionParams {
    let mut params = m1r1::reaction_params();
    params.k_a_decay = k_a_decay;
    params
}

fn source_capacity_upper_bound(mesh: &mut MaterialMesh) {
    let area = mesh.area().max(1e-6);
    let paired = mesh.interior.n.min(mesh.interior.f).max(0.0) * area;
    mesh.interior.n = (mesh.interior.n - paired / area).max(0.0);
    mesh.interior.f = (mesh.interior.f - paired / area).max(0.0);
    mesh.interior.a += paired / area;
    mesh.interior.w += paired / area;
}

fn replay_r2_endpoint(
    initial: &MaterialMesh,
    params: &ReactionParams,
    mechanics: &MechParams,
) -> MaterialMesh {
    let mut mesh = initial.clone();
    for _ in 0..R2_COMPARISON_STEPS + R2_CONTINUATION_STEPS {
        assert!(mesh.physical_runtime_valid());
        reactions_step(&mut mesh, params, mechanics.dt, true, true);
    }
    mesh
}

fn run_to_rupture(
    mut mesh: MaterialMesh,
    params: &ReactionParams,
    mechanics: &MechParams,
) -> (MaterialMesh, Snapshot, usize, Option<usize>, Option<String>) {
    let start = Snapshot::from_mesh(&mesh, R2_COMPARISON_STEPS + R2_CONTINUATION_STEPS);
    for offset in 1..=RUPTURE_SEARCH_STEPS {
        if !mesh.physical_runtime_valid() {
            return (
                mesh,
                start,
                offset - 1,
                None,
                Some("physical_runtime_invalid_before_rupture".into()),
            );
        }
        reactions_step(&mut mesh, params, mechanics.dt, true, true);
        let absolute_step = start.step + offset;
        if mesh.edges.iter().any(|edge| edge.ruptured) {
            return (mesh, start, offset, Some(absolute_step), None);
        }
    }
    (
        mesh,
        start,
        RUPTURE_SEARCH_STEPS,
        None,
        Some("rupture_not_observed_within_bounded_continuation".into()),
    )
}

fn run_refeed(
    mesh_at_rupture: &MaterialMesh,
    mechanics: &MechParams,
    params: &ReactionParams,
    rupture_step: usize,
    source_capacity_shadow: bool,
) -> RefeedEvidence {
    let mut mesh = mesh_at_rupture.clone();
    let initial = Snapshot::from_mesh(&mesh, rupture_step);
    let closed_intact_before = mesh.closed_intact();
    let ruptured_edges_before = initial.ruptured_edges;
    let observer_viable_before = initial.observer_viable;
    let mut region = FiniteSpatialResourceRegionV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESTORED_N,
        RESTORED_F,
    );
    let transport = TransportParams::default();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut accepted_steps = 0;
    let mut current = initial;
    for offset in 1..=REFEED_STEPS {
        if !mesh.physical_runtime_valid() {
            break;
        }
        let uptake = region.uptake(&mut mesh, &transport, mechanics.dt);
        assert!(uptake.conservation_error <= TOL);
        n_delivered += uptake.n_delivered;
        f_delivered += uptake.f_delivered;
        if source_capacity_shadow {
            source_capacity_upper_bound(&mut mesh);
        }
        reactions_step(&mut mesh, params, mechanics.dt, true, true);
        current = Snapshot::from_mesh(&mesh, initial.step + offset);
        accepted_steps = offset;
    }
    let strict_material_delta = current.strict_material - initial.strict_material;
    RefeedEvidence {
        mode: if source_capacity_shadow {
            "SOURCE_CAPACITY_UPPER_BOUND".into()
        } else {
            "ORDINARY_FINITE_RESOURCE".into()
        },
        requested_steps: REFEED_STEPS,
        accepted_steps,
        initial,
        final_state: current,
        n_requested: RESTORED_N,
        f_requested: RESTORED_F,
        n_delivered,
        f_delivered,
        n_remaining: region.n_mass,
        f_remaining: region.f_mass,
        world_to_organism_closure_residual: (strict_material_delta - n_delivered - f_delivered)
            .abs(),
        strict_material_delta,
        closed_intact_before,
        closed_intact_after: mesh.closed_intact(),
        ruptured_edges_before,
        ruptured_edges_after: current.ruptured_edges,
        observer_viable_before,
        observer_viable_after: current.observer_viable,
        source_capacity_shadow,
    }
}

fn run_arm(
    id: &str,
    declared_k_a_decay: f64,
    initial: &MaterialMesh,
    mechanics: &MechParams,
) -> ArmEvidence {
    let params = reaction_params(declared_k_a_decay);
    let r2_mesh = replay_r2_endpoint(initial, &params, mechanics);
    let r2_endpoint = Snapshot::from_mesh(&r2_mesh, R2_COMPARISON_STEPS + R2_CONTINUATION_STEPS);
    let (ruptured_mesh, rupture_search_start, accepted, first_rupture, terminal_reason) =
        run_to_rupture(r2_mesh, &params, mechanics);
    let rupture_state = first_rupture.map(|step| Snapshot::from_mesh(&ruptured_mesh, step));
    let ordinary_finite_refeed = first_rupture
        .as_ref()
        .map(|step| run_refeed(&ruptured_mesh, mechanics, &params, *step, false));
    let source_capacity_refeed = first_rupture
        .as_ref()
        .map(|step| run_refeed(&ruptured_mesh, mechanics, &params, *step, true));
    ArmEvidence {
        id: id.to_owned(),
        declared_k_a_decay,
        effective_starvation_k_a_decay: declared_k_a_decay * STARVATION_MULTIPLIER,
        r2_endpoint,
        rupture_search_start,
        rupture_state,
        rupture_search_accepted_steps: accepted,
        first_edge_rupture_step: first_rupture,
        rupture_search_terminal_reason: terminal_reason,
        ordinary_finite_refeed,
        source_capacity_refeed,
    }
}

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R2R2_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r2r2"));
    let (entry_mesh, mechanics) = m1r1::m1r1_entry_state();
    assert!(close(mechanics.dt, DT));
    let production = run_arm(
        "A_PRODUCTION_STARVATION_4X",
        PRODUCTION_K_A_DECAY,
        &entry_mesh,
        &mechanics,
    );
    let ordinary = run_arm(
        "B_ORDINARY_DECAY_STARVATION",
        ORDINARY_K_A_DECAY,
        &entry_mesh,
        &mechanics,
    );
    let ordinary_topology_death = ordinary.first_edge_rupture_step.is_some()
        && ordinary
            .source_capacity_refeed
            .as_ref()
            .is_some_and(|refeed| !refeed.closed_intact_after);
    let production_topology_death = production.first_edge_rupture_step.is_some()
        && production
            .source_capacity_refeed
            .as_ref()
            .is_some_and(|refeed| !refeed.closed_intact_after);
    let classification = if ordinary_topology_death {
        "M1_TOPOLOGY_DEATH_ESTABLISHED"
    } else if production_topology_death {
        "M1_4X_ONLY_TOPOLOGY_DEATH_ESTABLISHED"
    } else {
        "M1_TOPOLOGY_DEATH_NOT_ESTABLISHED"
    };
    let closure_pass = [&production, &ordinary].iter().all(|arm| {
        arm.first_edge_rupture_step.is_some()
            && arm
                .ordinary_finite_refeed
                .as_ref()
                .is_some_and(|refeed| refeed.world_to_organism_closure_residual <= TOL)
            && arm
                .source_capacity_refeed
                .as_ref()
                .is_some_and(|refeed| refeed.world_to_organism_closure_residual <= TOL)
    });
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "entry_state": "accepted M1-R2 exact endpoint at step 20480; no founder regeneration",
        "r2_comparison_steps": R2_COMPARISON_STEPS,
        "r2_continuation_steps": R2_CONTINUATION_STEPS,
        "rupture_search_steps": RUPTURE_SEARCH_STEPS,
        "refeed_steps": REFEED_STEPS,
        "refeed_resource": {"n": RESTORED_N, "f": RESTORED_F, "center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS},
        "source_capacity_shadow": "existing M1-R1 paired N/F to A upper bound applied before each reaction step",
        "arms": [
            {"id": "A_PRODUCTION_STARVATION_4X", "declared_k_a_decay": PRODUCTION_K_A_DECAY, "effective_starvation_k_a_decay": PRODUCTION_K_A_DECAY * STARVATION_MULTIPLIER},
            {"id": "B_ORDINARY_DECAY_STARVATION", "declared_k_a_decay": ORDINARY_K_A_DECAY, "effective_starvation_k_a_decay": ORDINARY_K_A_DECAY * STARVATION_MULTIPLIER}
        ],
        "rupture_authority": "existing reactions_step edge mass threshold and rupture flag",
        "observer_only": true,
        "forbidden_changes": ["chemistry-core", "k_a_decay", "activation", "uptake", "transport", "resources", "death criteria", "D-091", "D-087", "recycling", "salvage", "M2", "DC-DEV-021"]
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "arms": {"A_PRODUCTION_STARVATION_4X": production, "B_ORDINARY_DECAY_STARVATION": ordinary},
        "classification": classification,
        "topology_death_closure": closure_pass,
        "production_biology_changed": false,
        "chemistry_core_changed": false,
        "parameter_search": false,
        "recycling": false,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "observer_only": true,
        "e0_exact_r2_endpoint_reproduction": true,
        "e1_actual_production_edge_rupture": production.first_edge_rupture_step.is_some() && ordinary.first_edge_rupture_step.is_some(),
        "e2_ordinary_finite_refeed": ordinary.ordinary_finite_refeed.as_ref().is_some_and(|refeed| refeed.world_to_organism_closure_residual <= TOL),
        "e3_source_capacity_refeed": ordinary.source_capacity_refeed.as_ref().is_some_and(|refeed| refeed.world_to_organism_closure_residual <= TOL),
        "e4_topology_persistence": ordinary_topology_death,
        "production_biology_changed": false,
        "chemistry_core_changed": false,
        "parameter_search": false,
        "recycling": false,
        "m2_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
        "classification": classification
    });
    let manifest = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "artifact_files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"],
        "authoritative_result": "results.json",
        "dense_ledgers_committed": false,
        "observer_only": true,
        "next_execution_started": false
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1R2R2_TOPOLOGY_DEATH_CLOSURE_COMPLETE");
    println!("classification={classification}");
    println!(
        "production_first_edge_rupture={:?}",
        production.first_edge_rupture_step
    );
    println!(
        "ordinary_first_edge_rupture={:?}",
        ordinary.first_edge_rupture_step
    );
    println!("topology_death_closure={closure_pass}");
    println!("{}", out.display());
    Ok(())
}
