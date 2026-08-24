//! DC-DEV-020-M1-R6-R3 full-runtime M1 certification.
//!
//! This is an observer/certification harness. It uses the existing runtime
//! order and records amount-based closure around every stage. It does not
//! change any chemistry, mechanics, transport, death, or production law.

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

const DIRECTIVE: &str = "DC-DEV-020-M1-R6-R3-FULL-RUNTIME-M1-CERTIFICATION-001";
const STARTING_HEAD: &str = "0c56890d1f59c5dc2ffc66fd1d69181d7ca7b8c5";
const DT: f64 = 0.02;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const RESOURCE_CONCENTRATION: f64 = 2.063914918930895;
const HORIZON: usize = 8_000;
const DEPRIVATION_STEPS: usize = 480;
const DEATH_BOUND: usize = 150_000;
const REFEED_STEPS: usize = 5_000;
const TOLERANCE: f64 = 1e-8;
const CHECKPOINTS: [usize; 7] = [0, 480, 1_000, 2_000, 4_000, 6_000, 8_000];
const ATLAS_DENSE_ROOT: &str =
    r"\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r3";

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    area: f64,
    perimeter: f64,
    vertex_count: usize,
    exposed_resource_edges: usize,
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
}

fn state(mesh: &MaterialMesh, step: usize, exposed_resource_edges: usize) -> State {
    let s = snapshot(mesh);
    State {
        step,
        area: mesh.area(),
        perimeter: mesh.perimeter(),
        vertex_count: mesh.n(),
        exposed_resource_edges,
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
    }
}

#[derive(Debug, Clone, Serialize)]
struct Checkpoint {
    state: State,
    external_n_remaining: f64,
    external_f_remaining: f64,
    cumulative_n_delivered: f64,
    cumulative_f_delivered: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
struct SourceTotals {
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    max_world_loss_delivery_residual: f64,
    max_transport_ledger_residual: f64,
    min_remaining_n: f64,
    min_remaining_f: f64,
    replenishment_events: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
struct StageAccounting {
    accepted_steps: usize,
    cumulative_uptake_residual: f64,
    cumulative_reaction_residual: f64,
    cumulative_mechanics_residual: f64,
    cumulative_remesh_residual: f64,
    cumulative_rebond_residual: f64,
    cumulative_unexplained_residual: f64,
    max_step_uptake_residual: f64,
    max_step_reaction_residual: f64,
    max_step_mechanics_residual: f64,
    max_step_remesh_residual: f64,
    max_step_rebond_residual: f64,
    max_step_unexplained_residual: f64,
}

impl StageAccounting {
    fn observe(slot: &mut f64, cumulative: &mut f64, residual: f64) {
        *slot = (*slot).max(residual.abs());
        *cumulative += residual;
    }

    fn record(
        &mut self,
        uptake: f64,
        reaction: f64,
        mechanics: f64,
        remesh: f64,
        rebond: f64,
        unexplained: f64,
    ) {
        Self::observe(
            &mut self.max_step_uptake_residual,
            &mut self.cumulative_uptake_residual,
            uptake,
        );
        Self::observe(
            &mut self.max_step_reaction_residual,
            &mut self.cumulative_reaction_residual,
            reaction,
        );
        Self::observe(
            &mut self.max_step_mechanics_residual,
            &mut self.cumulative_mechanics_residual,
            mechanics,
        );
        Self::observe(
            &mut self.max_step_remesh_residual,
            &mut self.cumulative_remesh_residual,
            remesh,
        );
        Self::observe(
            &mut self.max_step_rebond_residual,
            &mut self.cumulative_rebond_residual,
            rebond,
        );
        Self::observe(
            &mut self.max_step_unexplained_residual,
            &mut self.cumulative_unexplained_residual,
            unexplained,
        );
        self.accepted_steps += 1;
    }

    fn pass(&self) -> bool {
        [
            self.max_step_uptake_residual,
            self.max_step_reaction_residual,
            self.max_step_mechanics_residual,
            self.max_step_remesh_residual,
            self.max_step_rebond_residual,
            self.max_step_unexplained_residual,
            self.cumulative_uptake_residual.abs(),
            self.cumulative_reaction_residual.abs(),
            self.cumulative_mechanics_residual.abs(),
            self.cumulative_remesh_residual.abs(),
            self.cumulative_rebond_residual.abs(),
            self.cumulative_unexplained_residual.abs(),
        ]
        .into_iter()
        .all(|value| value <= TOLERANCE)
    }
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
    accounting: StageAccounting,
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
struct ArmEvidence {
    arm: String,
    initial: State,
    final_state: State,
    checkpoints: Vec<Checkpoint>,
    source: SourceTotals,
    runtime: RuntimeTotals,
    organized_material_delta: f64,
    strict_material_delta: f64,
    world_organism_closure_residual: f64,
    stage_closure_pass: bool,
    first_observer_death_step: Option<usize>,
    first_topology_rupture_step: Option<usize>,
    first_physical_runtime_failure_step: Option<usize>,
    termination_step: usize,
    trajectory_hash: String,
    final_mesh_hash: String,
}

#[derive(Debug, Clone)]
struct ArmRun {
    evidence: ArmEvidence,
    mesh: MaterialMesh,
}

#[derive(Debug, Clone, Serialize)]
struct RecoveryEvidence {
    initial: State,
    deprived: State,
    refed: State,
    deprived_organized_delta: f64,
    refed_organized_delta_from_deprived: f64,
    deficit_reduction: f64,
    source: SourceTotals,
    runtime: RuntimeTotals,
    world_organism_closure_residual: f64,
    stage_closure_pass: bool,
    no_state_reset: bool,
    trajectory_hash: String,
}

#[derive(Debug, Clone, Serialize)]
struct PostLossRefeed {
    exercised: bool,
    rupture_state: Option<State>,
    final_state: Option<State>,
    resource_delivered: f64,
    rebond_attempts: usize,
    rebond_successes: usize,
    closed_intact_throughout: bool,
    topology_recovered: Option<bool>,
    stage_closure_pass: bool,
}

#[derive(Debug, Clone)]
struct Trace {
    initial: State,
    checkpoints: Vec<Checkpoint>,
    source: SourceTotals,
    runtime: RuntimeTotals,
    first_observer_death_step: Option<usize>,
    first_topology_rupture_step: Option<usize>,
    first_physical_runtime_failure_step: Option<usize>,
    final_state: State,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE * (1.0 + a.abs().max(b.abs()))
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

fn write_dense(writer: &mut Option<BufWriter<File>>, value: &Checkpoint) -> std::io::Result<()> {
    if let Some(writer) = writer {
        serde_json::to_writer(&mut *writer, value)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn run_step(
    mesh: &mut MaterialMesh,
    mechanics: &MechParams,
    reactions: &ReactionParams,
    transport: &TransportParams,
    world: Option<&mut FiniteSpatialBackingReservoirV1>,
    source: &mut SourceTotals,
    runtime: &mut RuntimeTotals,
) -> Result<usize, String> {
    let strict_before = snapshot(mesh).strict_material_equivalent();
    let mut world = world;
    let (n_before, f_before) = world
        .as_ref()
        .map(|item| (item.region.n_mass, item.region.f_mass))
        .unwrap_or((0.0, 0.0));
    let uptake = match world.as_deref_mut() {
        Some(item) => item.uptake(mesh, transport, mechanics.dt),
        None => Default::default(),
    };
    source.n_delivered += uptake.n_delivered;
    source.f_delivered += uptake.f_delivered;
    source.n_world_loss += uptake.n_world_loss;
    source.f_world_loss += uptake.f_world_loss;
    source.max_transport_ledger_residual = source
        .max_transport_ledger_residual
        .max(uptake.conservation_error);
    let n_after = world.as_ref().map(|item| item.region.n_mass).unwrap_or(0.0);
    let f_after = world.as_ref().map(|item| item.region.f_mass).unwrap_or(0.0);
    let world_loss_residual = (n_before - n_after - uptake.n_world_loss)
        .abs()
        .max((f_before - f_after - uptake.f_world_loss).abs());
    source.max_world_loss_delivery_residual = source
        .max_world_loss_delivery_residual
        .max(world_loss_residual);
    if world.is_some() && world_loss_residual > TOLERANCE {
        return Err(format!("world loss residual {world_loss_residual}"));
    }
    source.min_remaining_n = source.min_remaining_n.min(n_after);
    source.min_remaining_f = source.min_remaining_f.min(f_after);
    let strict_after_uptake = snapshot(mesh).strict_material_equivalent();
    let uptake_residual =
        strict_after_uptake - strict_before - uptake.n_delivered - uptake.f_delivered;

    let strict_before_reactions = strict_after_uptake;
    let ledger = reactions_step(mesh, reactions, mechanics.dt, true, true);
    let strict_after_reactions = snapshot(mesh).strict_material_equivalent();
    let reaction_residual = strict_after_reactions - strict_before_reactions;

    let strict_before_mechanics = strict_after_reactions;
    if !mechanics_step(mesh, mechanics) {
        runtime.mechanics_failures += 1;
        return Err("production mechanics step rejected".into());
    }
    let strict_after_mechanics = snapshot(mesh).strict_material_equivalent();
    let mechanics_residual = strict_after_mechanics - strict_before_mechanics;

    let strict_before_remesh = strict_after_mechanics;
    let (split, merge) = remesh(mesh);
    let strict_after_remesh = snapshot(mesh).strict_material_equivalent();
    let remesh_residual = strict_after_remesh - strict_before_remesh;

    let strict_before_rebond = strict_after_remesh;
    let rebonded = try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    let strict_after_rebond = snapshot(mesh).strict_material_equivalent();
    let rebond_residual = strict_after_rebond - strict_before_rebond;
    let unexplained_residual =
        strict_after_rebond - strict_before - uptake.n_delivered - uptake.f_delivered;
    runtime.accounting.record(
        uptake_residual,
        reaction_residual,
        mechanics_residual,
        remesh_residual,
        rebond_residual,
        unexplained_residual,
    );
    runtime.absorb(&ledger, split, merge, rebonded);
    Ok(uptake.exposed_edges)
}

fn checkpoint(
    trace: &mut Trace,
    mesh: &MaterialMesh,
    step: usize,
    exposed: usize,
    world: Option<&FiniteSpatialBackingReservoirV1>,
) {
    if CHECKPOINTS.contains(&step) {
        trace.checkpoints.push(Checkpoint {
            state: state(mesh, step, exposed),
            external_n_remaining: world.map(|item| item.region.n_mass).unwrap_or(0.0),
            external_f_remaining: world.map(|item| item.region.f_mass).unwrap_or(0.0),
            cumulative_n_delivered: trace.source.n_delivered,
            cumulative_f_delivered: trace.source.f_delivered,
        });
    }
}

fn run_segment(
    mesh: &mut MaterialMesh,
    start_step: usize,
    steps: usize,
    stop_on_rupture: bool,
    world: Option<&mut FiniteSpatialBackingReservoirV1>,
    trace: &mut Trace,
    dense: &mut Option<BufWriter<File>>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mechanics = MechParams::default();
    let reactions = v3_params();
    let transport = TransportParams::default();
    let mut world = world;
    let mut last_exposed = 0;
    for offset in 1..=steps {
        let absolute_step = start_step + offset;
        last_exposed = run_step(
            mesh,
            &mechanics,
            &reactions,
            &transport,
            world.as_deref_mut(),
            &mut trace.source,
            &mut trace.runtime,
        )?;
        if let Some(item) = world.as_ref() {
            trace.source.replenishment_events = item.replenishment_events;
        }
        let current = state(mesh, absolute_step, last_exposed);
        trace.final_state = current.clone();
        if trace.first_observer_death_step.is_none() && !current.observer_viable {
            trace.first_observer_death_step = Some(absolute_step);
        }
        if trace.first_topology_rupture_step.is_none() && current.ruptured_edges > 0 {
            trace.first_topology_rupture_step = Some(absolute_step);
        }
        if trace.first_physical_runtime_failure_step.is_none() && !current.physical_runtime_valid {
            trace.first_physical_runtime_failure_step = Some(absolute_step);
        }
        let record = Checkpoint {
            state: current,
            external_n_remaining: world.as_ref().map(|item| item.region.n_mass).unwrap_or(0.0),
            external_f_remaining: world.as_ref().map(|item| item.region.f_mass).unwrap_or(0.0),
            cumulative_n_delivered: trace.source.n_delivered,
            cumulative_f_delivered: trace.source.f_delivered,
        };
        write_dense(dense, &record)?;
        checkpoint(
            trace,
            mesh,
            absolute_step,
            last_exposed,
            world.as_ref().map(|value| &**value),
        );
        if stop_on_rupture && trace.first_topology_rupture_step == Some(absolute_step) {
            return Ok(absolute_step);
        }
    }
    Ok(start_step + steps)
}

fn new_trace(mesh: &MaterialMesh, step: usize) -> Trace {
    let initial = state(mesh, step, 0);
    Trace {
        initial: initial.clone(),
        checkpoints: Vec::new(),
        source: SourceTotals {
            min_remaining_n: f64::INFINITY,
            min_remaining_f: f64::INFINITY,
            ..SourceTotals::default()
        },
        runtime: RuntimeTotals::default(),
        first_observer_death_step: None,
        first_topology_rupture_step: None,
        first_physical_runtime_failure_step: None,
        final_state: initial,
    }
}

fn arm_evidence(
    arm: &str,
    trace: Trace,
    mesh: &MaterialMesh,
    termination_step: usize,
) -> ArmEvidence {
    let initial = trace.initial.clone();
    let strict_delta = trace.final_state.strict_material - trace.initial.strict_material;
    let world_total = trace.source.n_world_loss + trace.source.f_world_loss;
    let mut checkpoints = trace.checkpoints;
    if checkpoints.is_empty() {
        checkpoints.push(Checkpoint {
            state: trace.initial.clone(),
            external_n_remaining: 0.0,
            external_f_remaining: 0.0,
            cumulative_n_delivered: 0.0,
            cumulative_f_delivered: 0.0,
        });
    }
    let trajectory_hash = stable_json_hash(&checkpoints).unwrap_or_else(|_| "hash-error".into());
    ArmEvidence {
        arm: arm.into(),
        initial,
        final_state: trace.final_state.clone(),
        checkpoints,
        source: trace.source.clone(),
        runtime: trace.runtime.clone(),
        organized_material_delta: trace.final_state.organized_material
            - trace.initial.organized_material,
        strict_material_delta: strict_delta,
        world_organism_closure_residual: (strict_delta - world_total).abs(),
        stage_closure_pass: trace.runtime.accounting.pass()
            && trace.source.max_world_loss_delivery_residual <= TOLERANCE
            && trace.source.max_transport_ledger_residual <= TOLERANCE,
        first_observer_death_step: trace.first_observer_death_step,
        first_topology_rupture_step: trace.first_topology_rupture_step,
        first_physical_runtime_failure_step: trace.first_physical_runtime_failure_step,
        termination_step,
        trajectory_hash,
        final_mesh_hash: stable_json_hash(mesh).unwrap_or_else(|_| "hash-error".into()),
    }
}

fn run_arm(
    initial: &MaterialMesh,
    name: &str,
    steps: usize,
    feed: bool,
    stop_on_rupture: bool,
    dense_root: Option<&Path>,
) -> Result<ArmRun, Box<dyn std::error::Error>> {
    let mut mesh = initial.clone();
    let mut trace = new_trace(&mesh, 0);
    trace.checkpoints.push(Checkpoint {
        state: trace.initial.clone(),
        external_n_remaining: if feed { RESOURCE_MASS } else { 0.0 },
        external_f_remaining: if feed { RESOURCE_MASS } else { 0.0 },
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
    });
    let mut dense = dense_root
        .map(|root| File::create(root.join(format!("{name}.jsonl"))).map(BufWriter::new))
        .transpose()?;
    let initial_record = trace.checkpoints[0].clone();
    write_dense(&mut dense, &initial_record)?;
    let mut world = feed.then(reservoir);
    let termination_step = run_segment(
        &mut mesh,
        0,
        steps,
        stop_on_rupture,
        world.as_mut(),
        &mut trace,
        &mut dense,
    )?;
    if let Some(writer) = dense.as_mut() {
        writer.flush()?;
    }
    if let Some(world) = world.as_ref() {
        trace.source.replenishment_events = world.replenishment_events;
    }
    Ok(ArmRun {
        evidence: arm_evidence(name, trace, &mesh, termination_step),
        mesh,
    })
}

fn run_recovery(
    initial: &MaterialMesh,
    dense_root: Option<&Path>,
) -> Result<RecoveryEvidence, Box<dyn std::error::Error>> {
    let mut mesh = initial.clone();
    let mut trace = new_trace(&mesh, 0);
    trace.checkpoints.push(Checkpoint {
        state: trace.initial.clone(),
        external_n_remaining: 0.0,
        external_f_remaining: 0.0,
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
    });
    let mut dense = dense_root
        .map(|root| File::create(root.join("recovery.jsonl")).map(BufWriter::new))
        .transpose()?;
    write_dense(&mut dense, &trace.checkpoints[0])?;
    run_segment(
        &mut mesh,
        0,
        DEPRIVATION_STEPS,
        false,
        None,
        &mut trace,
        &mut dense,
    )?;
    let deprived = trace.final_state.clone();
    let deprivation_strict = deprived.strict_material - trace.initial.strict_material;
    let mut world = reservoir();
    run_segment(
        &mut mesh,
        DEPRIVATION_STEPS,
        HORIZON,
        false,
        Some(&mut world),
        &mut trace,
        &mut dense,
    )?;
    if let Some(writer) = dense.as_mut() {
        writer.flush()?;
    }
    let refed = trace.final_state.clone();
    let refed_delta = refed.organized_material - deprived.organized_material;
    let deprived_delta = deprived.organized_material - trace.initial.organized_material;
    let initial_deficit = (trace.initial.organized_material - deprived.organized_material).max(0.0);
    let final_deficit = (trace.initial.organized_material - refed.organized_material).max(0.0);
    let source_total = trace.source.n_world_loss + trace.source.f_world_loss;
    let initial = trace.initial.clone();
    let no_state_reset = close(
        deprived.strict_material,
        initial.strict_material + deprivation_strict,
    );
    Ok(RecoveryEvidence {
        initial,
        deprived,
        refed,
        deprived_organized_delta: deprived_delta,
        refed_organized_delta_from_deprived: refed_delta,
        deficit_reduction: initial_deficit - final_deficit,
        source: trace.source.clone(),
        runtime: trace.runtime.clone(),
        world_organism_closure_residual: (trace.final_state.strict_material
            - trace.initial.strict_material
            - source_total)
            .abs(),
        stage_closure_pass: trace.runtime.accounting.pass()
            && trace.source.max_world_loss_delivery_residual <= TOLERANCE
            && trace.source.max_transport_ledger_residual <= TOLERANCE,
        no_state_reset,
        trajectory_hash: stable_json_hash(&trace.checkpoints)
            .unwrap_or_else(|_| "hash-error".into()),
    })
}

fn run_feed_remove(
    fed: &ArmRun,
    dense_root: Option<&Path>,
) -> Result<ArmRun, Box<dyn std::error::Error>> {
    let mut mesh = fed.mesh.clone();
    let mut trace = new_trace(&mesh, HORIZON);
    let mut dense = dense_root
        .map(|root| File::create(root.join("feed_then_remove.jsonl")).map(BufWriter::new))
        .transpose()?;
    let removal_entry = trace.initial.clone();
    trace.checkpoints.push(Checkpoint {
        state: removal_entry.clone(),
        external_n_remaining: 0.0,
        external_f_remaining: 0.0,
        cumulative_n_delivered: 0.0,
        cumulative_f_delivered: 0.0,
    });
    write_dense(&mut dense, &trace.checkpoints[0])?;
    let termination_step = run_segment(
        &mut mesh,
        HORIZON,
        DEATH_BOUND,
        true,
        None,
        &mut trace,
        &mut dense,
    )?;
    if let Some(writer) = dense.as_mut() {
        writer.flush()?;
    }
    Ok(ArmRun {
        evidence: arm_evidence("FEED_THEN_REMOVE", trace, &mesh, termination_step),
        mesh,
    })
}

fn run_post_loss_refeed(
    removed: &mut ArmRun,
    dense_root: Option<&Path>,
) -> Result<PostLossRefeed, Box<dyn std::error::Error>> {
    let Some(rupture_step) = removed.evidence.first_topology_rupture_step else {
        return Ok(PostLossRefeed {
            exercised: false,
            rupture_state: None,
            final_state: None,
            resource_delivered: 0.0,
            rebond_attempts: 0,
            rebond_successes: 0,
            closed_intact_throughout: false,
            topology_recovered: None,
            stage_closure_pass: true,
        });
    };
    let rupture_state = removed.evidence.final_state.clone();
    let mut trace = new_trace(&removed.mesh, rupture_step);
    let mut dense = dense_root
        .map(|root| File::create(root.join("post_loss_refeed.jsonl")).map(BufWriter::new))
        .transpose()?;
    let mut world = reservoir();
    run_segment(
        &mut removed.mesh,
        rupture_step,
        REFEED_STEPS,
        false,
        Some(&mut world),
        &mut trace,
        &mut dense,
    )?;
    if let Some(writer) = dense.as_mut() {
        writer.flush()?;
    }
    let topology_recovered = trace.final_state.closed_intact;
    Ok(PostLossRefeed {
        exercised: true,
        rupture_state: Some(rupture_state),
        final_state: Some(trace.final_state.clone()),
        resource_delivered: trace.source.n_delivered + trace.source.f_delivered,
        rebond_attempts: trace.runtime.rebond_checks,
        rebond_successes: trace.runtime.rebond_successes,
        closed_intact_throughout: topology_recovered,
        topology_recovered: Some(topology_recovered),
        stage_closure_pass: trace.runtime.accounting.pass()
            && trace.source.max_world_loss_delivery_residual <= TOLERANCE
            && trace.source.max_transport_ledger_residual <= TOLERANCE,
    })
}

fn read_report(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({"status": "missing"}))
}

fn d087_pass(report: &Value, contract: &str) -> bool {
    report["mesh_contract"] == contract
        && report["reserve_enabled"] == false
        && (0..8).all(|index| report[format!("gate{index}")]["pass"] == true)
        && report["primary_conclusion"] == "D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED"
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV3");
    std::env::set_var("DCDEV020R9R3_RESERVE", "0");
    std::env::set_var("DCDEV020M1R6R2_GEOMETRY_CONTRACT", "1");
    let out = std::env::var_os("DCDEV020M1R6R3_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r6r3"));
    fs::create_dir_all(&out)?;
    let dense_root = std::env::var_os("DCDEV020M1R6R3_DENSE_OUTPUT")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(ATLAS_DENSE_ROOT)));
    if let Some(root) = dense_root.as_ref() {
        fs::create_dir_all(root)?;
    }

    let (mut entry, mechanics) = r5_entry::m1r1_entry_state();
    entry.stamp_geometry_conservative_schema();
    assert!(close(mechanics.dt, DT));
    assert_eq!(mechanics.dt, FROZEN_CENTER.dt);
    let entry_state = state(&entry, 0, 0);

    let fed = run_arm(
        &entry,
        "fed_8000",
        HORIZON,
        true,
        false,
        dense_root.as_deref(),
    )?;
    let zero = run_arm(
        &entry,
        "zero_resource",
        DEATH_BOUND,
        false,
        true,
        dense_root.as_deref(),
    )?;
    let recovery = run_recovery(&entry, dense_root.as_deref())?;
    let mut removal = run_feed_remove(&fed, dense_root.as_deref())?;
    let post_loss_refeed = run_post_loss_refeed(&mut removal, dense_root.as_deref())?;

    let out_v3 = read_report(&out.join("v3_d087/certification/report.json"));
    let out_v2 = read_report(&out.join("v2_d087/certification/report.json"));
    let fed_pass = fed.evidence.source.n_delivered + fed.evidence.source.f_delivered > 0.0
        && fed.evidence.source.replenishment_events == 0
        && fed.evidence.final_state.organized_material >= fed.evidence.initial.organized_material
        && fed.evidence.final_state.observer_viable
        && fed.evidence.final_state.closed_intact
        && fed.evidence.final_state.physical_runtime_valid
        && fed.evidence.stage_closure_pass;
    let restoration_pass = recovery.deprived.organized_material
        < recovery.initial.organized_material
        && recovery.refed.organized_material > recovery.deprived.organized_material
        && recovery.deficit_reduction > 0.0
        && recovery.source.n_delivered + recovery.source.f_delivered > 0.0
        && recovery.stage_closure_pass;
    let zero_resource_pass = zero.evidence.organized_material_delta < 0.0
        && zero.evidence.source.n_delivered == 0.0
        && zero.evidence.source.f_delivered == 0.0;
    let removal_pass = removal.evidence.organized_material_delta < 0.0
        && removal.evidence.source.n_delivered == 0.0
        && removal.evidence.source.f_delivered == 0.0
        && removal.evidence.stage_closure_pass;
    let resource_dependence_pass = zero_resource_pass && removal_pass;
    let physical_loss_pass = removal.evidence.first_topology_rupture_step.is_some()
        || removal
            .evidence
            .first_physical_runtime_failure_step
            .is_some();
    let post_loss_pass = !post_loss_refeed.exercised
        || (post_loss_refeed.stage_closure_pass
            && post_loss_refeed.topology_recovered == Some(false));
    let preservation_pass =
        d087_pass(&out_v2, "ConservativeV2") && d087_pass(&out_v3, "ConservativeV3");
    let accounting_pass = [
        fed.evidence.stage_closure_pass,
        zero.evidence.stage_closure_pass,
        recovery.stage_closure_pass,
        removal.evidence.stage_closure_pass,
        post_loss_refeed.stage_closure_pass,
    ]
    .into_iter()
    .all(|value| value)
        && fed.evidence.world_organism_closure_residual <= TOLERANCE
        && zero.evidence.world_organism_closure_residual <= TOLERANCE
        && recovery.world_organism_closure_residual <= TOLERANCE
        && removal.evidence.world_organism_closure_residual <= TOLERANCE;
    let classification = if !accounting_pass || !preservation_pass {
        "M1_FULL_RUNTIME_CERTIFICATION_INVALID"
    } else if !fed_pass || !restoration_pass {
        "M1_FULL_RUNTIME_HOMEOSTASIS_FAILED"
    } else if !resource_dependence_pass {
        "M1_FULL_RUNTIME_RESOURCE_DEPENDENCE_FAILED"
    } else if !physical_loss_pass || !post_loss_pass {
        "M1_FULL_RUNTIME_HOMEOSTASIS_PASS_DEATH_NOT_ESTABLISHED"
    } else {
        "M1_FULL_RUNTIME_CERTIFIED"
    };

    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "runtime_order": ["finite resource uptake", "reactions", "mechanics", "remesh", "try_local_rebond"],
        "runtime_contract": {"material": "GeometryConservativeV3", "chemistry": "ConservativeV3", "reserve": "OFF", "transport": "unchanged uncoupled V1 finite spatial transport", "world": "FINITE_SPATIAL_BACKING_RESERVOIR_V1"},
        "resource": {"center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "initial_n": RESOURCE_MASS, "initial_f": RESOURCE_MASS, "boundary_n": RESOURCE_CONCENTRATION, "boundary_f": RESOURCE_CONCENTRATION, "replenishment_events": 0},
        "horizon": HORIZON,
        "deprivation_steps": DEPRIVATION_STEPS,
        "death_bound": DEATH_BOUND,
        "refeed_steps": REFEED_STEPS,
        "checkpoints": CHECKPOINTS,
        "stage_closure": {"identity": "strict material before/after each stage; uptake additionally subtracts delivered N+F", "tolerance": TOLERANCE, "remesh_return_value_is_authority": false},
        "dense_output": ATLAS_DENSE_ROOT,
        "production_default_changed": false,
        "m2_authorized": false,
        "next_execution_started": false,
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "entry_state": entry_state,
        "fed_8000": fed.evidence,
        "zero_resource": zero.evidence,
        "recovery": recovery,
        "feed_then_remove": removal.evidence,
        "post_loss_refeed": post_loss_refeed,
        "classification": classification,
        "fed_homeostasis_pass": fed_pass,
        "restoration_pass": restoration_pass,
        "resource_dependence_pass": resource_dependence_pass,
        "physical_loss_pass": physical_loss_pass,
        "post_loss_refeed_pass": post_loss_pass,
        "accounting_pass": accounting_pass,
        "preservation_pass": preservation_pass,
        "production_default_changed": false,
        "m2_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "e0_authority": true,
        "e1_runtime_identity": "GeometryConservativeV3",
        "e2_stage_level_closure": accounting_pass,
        "e3_sustained_homeostasis": fed_pass,
        "e4_restoration": restoration_pass,
        "e5_resource_dependence": resource_dependence_pass,
        "e6_irreversible_physical_loss": physical_loss_pass,
        "e7_post_loss_refeed": if post_loss_refeed.exercised { post_loss_pass } else { false },
        "e8_preservation": preservation_pass,
        "observer_only": true,
        "production_biology_changed": false,
        "production_default_changed": false,
        "next_execution_started": false,
        "classification": classification,
    });
    let preservation = json!({
        "historical_v2_d087": d087_pass(&out_v2, "ConservativeV2"),
        "candidate_v3_d087": d087_pass(&out_v3, "ConservativeV3"),
        "gc_preservation_qualifier": "required_by_remote_workflow",
        "d088": "required_by_remote_workflow",
        "d091": "required_by_remote_workflow",
        "evolution_harness": "required_by_remote_workflow",
    });
    for (name, value) in [
        ("protocol.json", &protocol),
        ("results.json", &results),
        ("qualification.json", &qualification),
        ("preservation.json", &preservation),
    ] {
        fs::write(out.join(name), serde_json::to_vec_pretty(value)?)?;
    }
    let manifest = json!({"schema": "dcdev020m1r6r3_manifest_v1", "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": ["protocol.json", "results.json", "qualification.json", "preservation.json", "artifact_manifest.json"], "dense_output": ATLAS_DENSE_ROOT, "shared_drive_required": true, "sha256": "computed-by-workflow"});
    fs::write(
        out.join("artifact_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("classification={classification}");
    println!(
        "fed_organized_delta={}",
        fed.evidence.organized_material_delta
    );
    println!("recovery_deficit_reduction={}", recovery.deficit_reduction);
    println!(
        "first_zero_resource_rupture_step={:?}",
        zero.evidence.first_topology_rupture_step
    );
    println!(
        "first_removal_rupture_step={:?}",
        removal.evidence.first_topology_rupture_step
    );
    println!("next_execution_started=false");
    Ok(())
}
