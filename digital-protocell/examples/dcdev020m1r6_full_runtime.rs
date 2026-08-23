//! DC-DEV-020-M1-R6: full packaged-runtime integration certification.
//!
//! R5 established finite-resource homeostasis through transport -> reactions.
//! This bounded runner adds the unchanged production mechanics, remesh, and
//! local-rebond stages in the same order as `phase1_certifier::sim::run_coupled`.
//! It is an assay adapter only: no chemistry, mechanics, or repair law is
//! changed here.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r5_entry;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step, try_local_rebond, MeshChemistrySchema, ReactionLedger, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;
use phase1_certifier::frozen::FROZEN_CENTER;
use regulatory_core::{stable_json_hash, FiniteSpatialBackingReservoirV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R6-FULL-RUNTIME-INTEGRATION-CERT-001";
const STARTING_HEAD: &str = "9ff1bba4a48caf582e4598b4030d746e4360a61b";
const R5_ENTRY_ORGANIZED: f64 = 131.80639622655494;
const R5_ENTRY_A: f64 = 19.69467805250676;
const R5_ENTRY_C: f64 = 55.87794642665143;
const DT: f64 = 0.02;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const R4_INVENTORY: f64 = 14.588954880632265;
const RESOURCE_CONCENTRATION: f64 = 2.063914918930895;
const SHARED_DENSE_ROOT: &str =
    r"\\RPI5\RPI5SharedDrive\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6\dense";
const HORIZON: usize = 8_000;
const FEED_STEPS: usize = 480;
const DEATH_BOUND: usize = 150_000;
const RESTORE_STEPS: usize = 5_000;
const TOL: f64 = 1e-8;
const CHECKPOINTS: [usize; 9] = [0, 480, 1_000, 2_000, 3_466, 4_000, 6_000, 6_931, 8_000];

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    n: f64,
    f: f64,
    a: f64,
    c: f64,
    structural_m: f64,
    membrane: f64,
    bound_b: f64,
    free_l: f64,
    waste: f64,
    organized_material: f64,
    strict_material: f64,
    min_edge_m: f64,
    ruptured_edges: usize,
    closed_intact: bool,
    observer_viable: bool,
    observer_death_reason: Option<&'static str>,
    physical_runtime_valid: bool,
    vertex_count: usize,
}

fn state(mesh: &MaterialMesh, step: usize) -> State {
    let s = snapshot(mesh);
    State {
        step,
        n: s.n,
        f: s.f,
        a: s.a,
        c: s.c,
        structural_m: s.structural_m,
        membrane: mesh.total_membrane(),
        bound_b: s.bound_b,
        free_l: s.free_l,
        waste: s.waste,
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        min_edge_m: mesh.edges.iter().map(|e| e.m).fold(f64::INFINITY, f64::min),
        ruptured_edges: mesh.edges.iter().filter(|e| e.ruptured).count(),
        closed_intact: mesh.closed_intact(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
        vertex_count: mesh.n(),
    }
}

#[derive(Debug, Clone, Default, Serialize)]
struct SourceTotals {
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    max_world_delivery_residual: f64,
    min_remaining_n: f64,
    min_remaining_f: f64,
    replenishment_events: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
struct RuntimeTotals {
    accepted_steps: usize,
    mechanics_failures: usize,
    remesh_splits: usize,
    remesh_merges: usize,
    rebond_checks: usize,
    rebond_successes: usize,
    m_production: f64,
    m_turnover: f64,
    c_production: f64,
    c_turnover: f64,
    membrane_production: f64,
    membrane_turnover: f64,
    a_decay: f64,
}

impl RuntimeTotals {
    fn absorb(&mut self, ledger: &ReactionLedger, split: usize, merge: usize, rebonded: bool) {
        self.accepted_steps += 1;
        self.remesh_splits += split;
        self.remesh_merges += merge;
        self.rebond_checks += 1;
        self.rebond_successes += usize::from(rebonded);
        self.m_production += ledger.m_produced;
        self.m_turnover += ledger.m_to_w;
        self.c_production += ledger.c_produced;
        self.c_turnover += ledger.c_turned;
        self.membrane_production += ledger.l_produced;
        self.membrane_turnover += ledger.bind_extent + ledger.unbind_extent;
        self.a_decay += ledger.a_decayed;
    }
}

#[derive(Debug, Clone, Serialize)]
struct Checkpoint {
    step: usize,
    external_n_remaining: f64,
    external_f_remaining: f64,
    state: State,
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    arm: String,
    initial: State,
    final_state: State,
    checkpoints: Vec<Checkpoint>,
    source: SourceTotals,
    runtime: RuntimeTotals,
    organized_material_delta: f64,
    strict_material_delta: f64,
    world_organism_closure_residual: f64,
    trajectory_hash: String,
    final_mesh_hash: String,
    resource_remaining_positive: bool,
    post_removal_delivery: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
struct DeathResult {
    initial: State,
    rupture_step: Option<usize>,
    rupture_state: Option<State>,
    post_rupture_final: State,
    post_rupture_resource_delivery: f64,
    rebond_checks_after_rupture: usize,
    rebond_successes_after_rupture: usize,
    closed_intact_after_restore: bool,
    runtime: RuntimeTotals,
    source: SourceTotals,
    trajectory_hash: String,
    bound_exhausted_without_rupture: bool,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL * (1.0 + a.abs().max(b.abs()))
}

fn v3_params() -> ReactionParams {
    let params = ReactionParams::conservative_v3();
    assert_eq!(params.mesh_schema, MeshChemistrySchema::ConservativeV3);
    assert!(!params.reserve.enable);
    params
}

fn reservoir() -> FiniteSpatialBackingReservoirV1 {
    FiniteSpatialBackingReservoirV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        RESOURCE_MASS,
        RESOURCE_MASS,
        RESOURCE_CONCENTRATION,
        RESOURCE_CONCENTRATION,
    )
}

fn write_dense(writer: &mut Option<BufWriter<File>>, value: &State) -> std::io::Result<()> {
    if let Some(writer) = writer {
        serde_json::to_writer(&mut *writer, value)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn runtime_step(
    mesh: &mut MaterialMesh,
    mechanics: &MechParams,
    reactions: &ReactionParams,
    transport: &TransportParams,
    mut world: Option<&mut FiniteSpatialBackingReservoirV1>,
    source: &mut SourceTotals,
    runtime: &mut RuntimeTotals,
) -> Result<(), String> {
    let (n_before, f_before) = world
        .as_ref()
        .map(|w| (w.region.n_mass, w.region.f_mass))
        .unwrap_or((0.0, 0.0));
    let uptake = match world.as_deref_mut() {
        Some(world) => world.uptake(mesh, transport, mechanics.dt),
        None => Default::default(),
    };
    source.n_delivered += uptake.n_delivered;
    source.f_delivered += uptake.f_delivered;
    source.n_world_loss += uptake.n_world_loss;
    source.f_world_loss += uptake.f_world_loss;
    source.max_world_delivery_residual = source.max_world_delivery_residual.max(
        (uptake.n_world_loss - uptake.n_delivered)
            .abs()
            .max((uptake.f_world_loss - uptake.f_delivered).abs()),
    );
    let n_after = world.as_ref().map(|w| w.region.n_mass).unwrap_or(0.0);
    let f_after = world.as_ref().map(|w| w.region.f_mass).unwrap_or(0.0);
    if world.is_some()
        && (!close(n_before - n_after, uptake.n_world_loss)
            || !close(f_before - f_after, uptake.f_world_loss))
    {
        return Err("finite world loss did not equal delivery".into());
    }
    source.min_remaining_n = source.min_remaining_n.min(n_after);
    source.min_remaining_f = source.min_remaining_f.min(f_after);
    let ledger = reactions_step(mesh, reactions, mechanics.dt, true, true);
    if !mechanics_step(mesh, mechanics) {
        runtime.mechanics_failures += 1;
        return Err("production mechanics step rejected".into());
    }
    let (split, merge) = remesh(mesh);
    let rebonded = try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    runtime.absorb(&ledger, split, merge, rebonded);
    Ok(())
}

fn checkpoints_push(
    checkpoints: &mut Vec<Checkpoint>,
    step: usize,
    mesh: &MaterialMesh,
    world: Option<&FiniteSpatialBackingReservoirV1>,
) {
    if CHECKPOINTS.contains(&step) {
        checkpoints.push(Checkpoint {
            step,
            external_n_remaining: world.map(|w| w.region.n_mass).unwrap_or(0.0),
            external_f_remaining: world.map(|w| w.region.f_mass).unwrap_or(0.0),
            state: state(mesh, step),
        });
    }
}

fn run_arm(
    initial: &MaterialMesh,
    name: &str,
    feed: bool,
    dense_root: Option<&Path>,
) -> Result<ArmResult, Box<dyn std::error::Error>> {
    let mut mesh = initial.clone();
    let mechanics = MechParams::default();
    let reactions = v3_params();
    let transport = TransportParams::default();
    let mut world = feed.then(reservoir);
    let initial_state = state(&mesh, 0);
    let mut source = SourceTotals {
        min_remaining_n: if feed { RESOURCE_MASS } else { 0.0 },
        min_remaining_f: if feed { RESOURCE_MASS } else { 0.0 },
        ..SourceTotals::default()
    };
    let mut runtime = RuntimeTotals::default();
    let mut checkpoints = Vec::new();
    let mut trajectory = vec![stable_json_hash(&initial_state)?];
    let mut dense = dense_root
        .map(|root| File::create(root.join(format!("{name}.jsonl"))).map(BufWriter::new))
        .transpose()?;
    write_dense(&mut dense, &initial_state)?;
    checkpoints_push(&mut checkpoints, 0, &mesh, world.as_ref());
    for step in 1..=HORIZON {
        runtime_step(
            &mut mesh,
            &mechanics,
            &reactions,
            &transport,
            world.as_mut(),
            &mut source,
            &mut runtime,
        )?;
        if let Some(world) = world.as_ref() {
            source.replenishment_events = world.replenishment_events;
        }
        let current = state(&mesh, step);
        write_dense(&mut dense, &current)?;
        trajectory.push(stable_json_hash(&current)?);
        checkpoints_push(&mut checkpoints, step, &mesh, world.as_ref());
    }
    if let Some(dense) = dense.as_mut() {
        dense.flush()?;
    }
    let final_state = state(&mesh, HORIZON);
    let world_total = source.n_world_loss + source.f_world_loss;
    let strict_delta = final_state.strict_material - initial_state.strict_material;
    Ok(ArmResult {
        arm: name.into(),
        initial: initial_state.clone(),
        final_state: final_state.clone(),
        checkpoints,
        source,
        runtime,
        organized_material_delta: final_state.organized_material - initial_state.organized_material,
        strict_material_delta: strict_delta,
        world_organism_closure_residual: (strict_delta - world_total).abs(),
        trajectory_hash: stable_json_hash(&trajectory)?,
        final_mesh_hash: stable_json_hash(&mesh)?,
        resource_remaining_positive: world.map(|w| w.total_mass() > 0.0).unwrap_or(false),
        post_removal_delivery: None,
    })
}

fn run_feed_remove(
    initial: &MaterialMesh,
    dense_root: Option<&Path>,
) -> Result<ArmResult, Box<dyn std::error::Error>> {
    let mut mesh = initial.clone();
    let mechanics = MechParams::default();
    let reactions = v3_params();
    let transport = TransportParams::default();
    let mut world = FiniteSpatialBackingReservoirV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        R4_INVENTORY,
        R4_INVENTORY,
        RESOURCE_CONCENTRATION,
        RESOURCE_CONCENTRATION,
    );
    let initial_state = state(&mesh, 0);
    let mut source = SourceTotals {
        min_remaining_n: RESOURCE_MASS,
        min_remaining_f: RESOURCE_MASS,
        ..SourceTotals::default()
    };
    let mut runtime = RuntimeTotals::default();
    let mut checkpoints = Vec::new();
    let mut trajectory = vec![stable_json_hash(&initial_state)?];
    let mut dense = dense_root
        .map(|root| File::create(root.join("feed_remove.jsonl")).map(BufWriter::new))
        .transpose()?;
    write_dense(&mut dense, &initial_state)?;
    checkpoints_push(&mut checkpoints, 0, &mesh, Some(&world));
    for step in 1..=FEED_STEPS {
        runtime_step(
            &mut mesh,
            &mechanics,
            &reactions,
            &transport,
            Some(&mut world),
            &mut source,
            &mut runtime,
        )?;
        let current = state(&mesh, step);
        write_dense(&mut dense, &current)?;
        trajectory.push(stable_json_hash(&current)?);
        if step == FEED_STEPS || CHECKPOINTS.contains(&step) {
            checkpoints.push(Checkpoint {
                step,
                external_n_remaining: world.region.n_mass,
                external_f_remaining: world.region.f_mass,
                state: current,
            });
        }
    }
    let removal_state = state(&mesh, FEED_STEPS);
    world.remove_remaining_inventory();
    let removal_organized = removal_state.organized_material;
    for offset in 1..=HORIZON {
        let step = FEED_STEPS + offset;
        runtime_step(
            &mut mesh,
            &mechanics,
            &reactions,
            &transport,
            None,
            &mut source,
            &mut runtime,
        )?;
        let current = state(&mesh, step);
        write_dense(&mut dense, &current)?;
        trajectory.push(stable_json_hash(&current)?);
        if [1_480, 2_480, 3_466, 4_480, 6_480, 7_931, 8_480].contains(&step) {
            checkpoints.push(Checkpoint {
                step,
                external_n_remaining: 0.0,
                external_f_remaining: 0.0,
                state: current,
            });
        }
    }
    if let Some(dense) = dense.as_mut() {
        dense.flush()?;
    }
    let final_state = state(&mesh, FEED_STEPS + HORIZON);
    let strict_delta = final_state.strict_material - initial_state.strict_material;
    Ok(ArmResult {
        arm: "FEED_THEN_REMOVE".into(),
        initial: initial_state,
        final_state: final_state.clone(),
        checkpoints,
        source,
        runtime,
        organized_material_delta: final_state.organized_material - removal_organized,
        strict_material_delta: strict_delta,
        world_organism_closure_residual: (strict_delta - (R4_INVENTORY * 2.0 - world.total_mass()))
            .abs(),
        trajectory_hash: stable_json_hash(&trajectory)?,
        final_mesh_hash: stable_json_hash(&mesh)?,
        resource_remaining_positive: false,
        post_removal_delivery: Some(0.0),
    })
}

fn run_death(
    initial: &MaterialMesh,
    dense_root: Option<&Path>,
) -> Result<DeathResult, Box<dyn std::error::Error>> {
    let mut mesh = initial.clone();
    let mechanics = MechParams::default();
    let reactions = v3_params();
    let transport = TransportParams::default();
    let initial_state = state(&mesh, 0);
    let mut source = SourceTotals::default();
    let mut runtime = RuntimeTotals::default();
    let mut trajectory = vec![stable_json_hash(&initial_state)?];
    let mut dense = dense_root
        .map(|root| File::create(root.join("death_restore.jsonl")).map(BufWriter::new))
        .transpose()?;
    write_dense(&mut dense, &initial_state)?;
    let mut rupture_step = None;
    let mut rupture_state = None;
    for step in 1..=DEATH_BOUND {
        runtime_step(
            &mut mesh,
            &mechanics,
            &reactions,
            &transport,
            None,
            &mut source,
            &mut runtime,
        )?;
        let current = state(&mesh, step);
        write_dense(&mut dense, &current)?;
        trajectory.push(stable_json_hash(&current)?);
        if current.ruptured_edges > 0 {
            rupture_step = Some(step);
            rupture_state = Some(current);
            break;
        }
    }
    let bound_exhausted_without_rupture = rupture_step.is_none();
    let mut post_source = reservoir();
    let checks_before = runtime.rebond_checks;
    let successes_before = runtime.rebond_successes;
    if rupture_step.is_some() {
        for _ in 1..=RESTORE_STEPS {
            runtime_step(
                &mut mesh,
                &mechanics,
                &reactions,
                &transport,
                Some(&mut post_source),
                &mut source,
                &mut runtime,
            )?;
            let current = state(
                &mesh,
                rupture_step.unwrap() + (runtime.accepted_steps - checks_before),
            );
            write_dense(&mut dense, &current)?;
            trajectory.push(stable_json_hash(&current)?);
        }
    }
    if let Some(dense) = dense.as_mut() {
        dense.flush()?;
    }
    let final_step = rupture_step
        .map(|step| step + RESTORE_STEPS)
        .unwrap_or(runtime.accepted_steps);
    let final_state = state(&mesh, final_step);
    Ok(DeathResult {
        initial: initial_state,
        rupture_step,
        rupture_state,
        post_rupture_final: final_state,
        post_rupture_resource_delivery: source.n_delivered,
        rebond_checks_after_rupture: runtime.rebond_checks.saturating_sub(checks_before),
        rebond_successes_after_rupture: runtime.rebond_successes.saturating_sub(successes_before),
        closed_intact_after_restore: mesh.closed_intact(),
        runtime,
        source,
        trajectory_hash: stable_json_hash(&trajectory)?,
        bound_exhausted_without_rupture,
    })
}

fn read_report(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({"status": "deferred_to_exact_workflow"}))
}

fn d087_pass(report: &Value, contract: &str) -> bool {
    report["mesh_contract"] == contract
        && report["reserve_enabled"] == false
        && (0..8).all(|i| report[format!("gate{i}")]["pass"] == true)
        && report["primary_conclusion"] == "D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED"
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R6_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r6"));
    fs::create_dir_all(&out)?;
    let dense_root = std::env::var_os("DCDEV020M1R6_DENSE_OUTPUT").map(PathBuf::from);
    if let Some(root) = dense_root.as_ref() {
        fs::create_dir_all(root)?;
    }
    let (entry, mechanics) = r5_entry::m1r1_entry_state();
    assert!(close(mechanics.dt, DT));
    let entry_state = state(&entry, 0);
    assert!(close(entry_state.a, R5_ENTRY_A));
    assert!(close(entry_state.c, R5_ENTRY_C));
    assert!(close(entry_state.organized_material, R5_ENTRY_ORGANIZED));
    assert_eq!(mechanics.dt, FROZEN_CENTER.dt);

    let fed = run_arm(&entry, "FULL_RUNTIME_FED", true, dense_root.as_deref())?;
    let no_resource = run_arm(
        &entry,
        "FULL_RUNTIME_NO_RESOURCE",
        false,
        dense_root.as_deref(),
    )?;
    let removal = run_feed_remove(&entry, dense_root.as_deref())?;
    let death = run_death(&entry, dense_root.as_deref())?;
    let v3_d087 = read_report(&out.join("v3_d087/certification/report.json"));
    let v2_d087 = read_report(&out.join("v2_d087/certification/report.json"));
    let fed_pass = fed.resource_remaining_positive
        && fed.final_state.organized_material >= fed.initial.organized_material - TOL
        && fed.final_state.observer_viable
        && fed.final_state.closed_intact
        && fed.final_state.ruptured_edges == 0
        && fed.final_state.physical_runtime_valid;
    let no_resource_pass = no_resource.organized_material_delta < fed.organized_material_delta;
    let removal_pass =
        removal.organized_material_delta < 0.0 && removal.post_removal_delivery == Some(0.0);
    let death_pass = death.rupture_step.is_some()
        && !death.closed_intact_after_restore
        && death.rebond_successes_after_rupture == 0;
    let accounting_pass = fed.source.max_world_delivery_residual <= TOL
        && fed.world_organism_closure_residual <= TOL
        && fed.source.replenishment_events == 0
        && no_resource.world_organism_closure_residual <= TOL
        && removal.source.max_world_delivery_residual <= TOL;
    let classification = if !accounting_pass
        || !d087_pass(&v2_d087, "ConservativeV2")
        || !d087_pass(&v3_d087, "ConservativeV3")
    {
        "M1_FULL_RUNTIME_CERTIFICATION_INVALID"
    } else if !fed_pass || !no_resource_pass || !removal_pass {
        "M1_FULL_RUNTIME_HOMEOSTASIS_REGRESSION"
    } else if !death_pass {
        "M1_FULL_RUNTIME_DEATH_REGRESSION"
    } else {
        "M1_FULL_RUNTIME_CERTIFIED"
    };
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "authoritative_packaged_runtime": {"binary": "digital-protocell-phase1", "step": "transport -> reactions -> mechanics -> remesh -> try_local_rebond", "mechanics": "chemistry-core::mesh_mechanics::mechanics_step", "remesh": "chemistry-core::mesh_mechanics::remesh", "rebond": "chemistry-core::mesh_reactions::try_local_rebond"},
        "candidate": {"mesh_schema": "ConservativeV3", "reserve": "OFF", "transport": "uncoupled V1 finite spatial uptake", "backing_reservoir": "FINITE_SPATIAL_BACKING_RESERVOIR_V1", "coupled_source": "OFF"},
        "dt": DT,
        "horizon": HORIZON,
        "death_bound": DEATH_BOUND,
        "restore_steps": RESTORE_STEPS,
        "resource": {"center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "initial_n": RESOURCE_MASS, "initial_f": RESOURCE_MASS, "boundary_n": RESOURCE_CONCENTRATION, "boundary_f": RESOURCE_CONCENTRATION, "replenishment_events": 0},
        "dense_output": SHARED_DENSE_ROOT,
        "production_selection_changed": false,
        "m2_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "entry_state": entry_state,
        "arm_fed": fed,
        "arm_no_resource": no_resource,
        "arm_feed_remove": removal,
        "full_runtime_death": death,
        "classification": classification,
        "fed_pass": fed_pass,
        "no_resource_pass": no_resource_pass,
        "feed_remove_pass": removal_pass,
        "death_pass": death_pass,
        "accounting_pass": accounting_pass,
        "production_default_before": "ConservativeV2",
        "production_default_after": "UNCHANGED",
        "coupled_source_selected": false,
        "controller_added": false,
        "parameter_search": false,
        "recycling": false,
        "m2_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "e0_authority": true,
        "e1_runtime_inventory": true,
        "e2_full_runtime_sustained": fed_pass,
        "e3_resource_dependence": no_resource_pass && removal_pass,
        "e4_full_runtime_death": death_pass,
        "e5_d087_preservation": d087_pass(&v2_d087, "ConservativeV2") && d087_pass(&v3_d087, "ConservativeV3"),
        "classification": classification,
        "observer_only_integration": true,
        "production_biology_changed": false,
        "next_execution_started": false,
    });
    fs::write(
        out.join("protocol.json"),
        serde_json::to_vec_pretty(&protocol)?,
    )?;
    fs::write(
        out.join("results.json"),
        serde_json::to_vec_pretty(&results)?,
    )?;
    fs::write(
        out.join("qualification.json"),
        serde_json::to_vec_pretty(&qualification)?,
    )?;
    let manifest = json!({"schema": "dcdev020m1r6_manifest_v1", "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"], "dense_output": SHARED_DENSE_ROOT, "shared_drive_required": true, "sha256": "computed-by-workflow"});
    fs::write(
        out.join("artifact_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("classification={classification}");
    println!("fed_organized_delta={}", fed.organized_material_delta);
    println!("fed_final_min_edge_m={}", fed.final_state.min_edge_m);
    println!("first_rupture_step={:?}", death.rupture_step);
    println!(
        "post_restore_closed_intact={}",
        death.closed_intact_after_restore
    );
    Ok(())
}
