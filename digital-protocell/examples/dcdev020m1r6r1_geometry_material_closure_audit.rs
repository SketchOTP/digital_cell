//! DC-DEV-020-M1-R6-R1: observer-only full-runtime closure attribution.
//!
//! This runner replays the accepted R6 packaged order and records independent
//! material ledgers at each boundary.  It deliberately does not rescale
//! concentrations, alter equations, or otherwise repair the runtime.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r5_entry;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step, try_local_rebond, MeshChemistrySchema, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;
use regulatory_core::{stable_json_hash, FiniteSpatialBackingReservoirV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R6-R1-GEOMETRY-MATERIAL-CLOSURE-AUDIT-001";
const STARTING_HEAD: &str = "adea13fafa1f2a85e521a44b5d77249820d107bd";
const R6_STARTING_HEAD: &str = "9ff1bba4a48caf582e4598b4030d746e4360a61b";
const DT: f64 = 0.02;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const RESOURCE_CONCENTRATION: f64 = 2.063914918930895;
const HORIZON: usize = 8_000;
const R6_REPRODUCTION_STEPS: [usize; 6] = [0, 1, 10, 100, 480, 1_000];
const R6_TRAJECTORY_HASH: &str = "be91ed02266a0078";
const R6_FINAL_MESH_HASH: &str = "e4c4dd4ff2e443d8";
const TOL: f64 = 1e-8;
const SHARED_DENSE_ROOT: &str =
    r"\\RPI5\RPI5SharedDrive\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r1\dense";

#[derive(Debug, Clone, Serialize)]
struct R6State {
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

fn r6_state(mesh: &MaterialMesh, step: usize) -> R6State {
    let s = snapshot(mesh);
    R6State {
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

#[derive(Debug, Clone, Serialize)]
struct MaterialObservation {
    step: usize,
    area: f64,
    signed_area: f64,
    perimeter: f64,
    vertex_count: usize,
    exposed_resource_edges: usize,
    centroid: [f64; 2],
    n_concentration: f64,
    f_concentration: f64,
    a_concentration: f64,
    r_concentration: f64,
    c_concentration: f64,
    w_concentration: f64,
    n_amount: f64,
    f_amount: f64,
    a_amount: f64,
    r_amount: f64,
    c_amount: f64,
    w_amount: f64,
    structural_m: f64,
    free_l: f64,
    bound_b: f64,
    hereditary: f64,
    strict_material: f64,
    organized_material: f64,
    ruptured_edges: usize,
}

fn observation(
    mesh: &MaterialMesh,
    step: usize,
    exposed_resource_edges: usize,
) -> MaterialObservation {
    let area = mesh.area().max(1e-9);
    let s = snapshot(mesh);
    MaterialObservation {
        step,
        area,
        signed_area: mesh.signed_area(),
        perimeter: mesh.perimeter(),
        vertex_count: mesh.n(),
        exposed_resource_edges,
        centroid: mesh.centroid(),
        n_concentration: mesh.interior.n,
        f_concentration: mesh.interior.f,
        a_concentration: mesh.interior.a,
        r_concentration: mesh.interior.r,
        c_concentration: mesh.interior.c,
        w_concentration: mesh.interior.w,
        n_amount: mesh.interior.n.max(0.0) * area,
        f_amount: mesh.interior.f.max(0.0) * area,
        a_amount: mesh.interior.a.max(0.0) * area,
        r_amount: mesh.interior.r.max(0.0) * area,
        c_amount: mesh.interior.c.max(0.0) * area,
        w_amount: mesh.interior.w.max(0.0) * area,
        structural_m: s.structural_m,
        free_l: s.free_l,
        bound_b: s.bound_b,
        hereditary: s.hereditary,
        strict_material: s.strict_material_equivalent(),
        organized_material: s.organized_material(),
        ruptured_edges: mesh.edges.iter().filter(|e| e.ruptured).count(),
    }
}

#[derive(Debug, Clone, Serialize)]
struct SpeciesAreaAudit {
    species: &'static str,
    concentration_before: f64,
    concentration_after: f64,
    mass_before: f64,
    mass_after: f64,
    conservative_concentration_after: f64,
    fixed_concentration_geometry_delta: f64,
    observed_mass_delta: f64,
}

fn species_audits(
    before: &MaterialObservation,
    after: &MaterialObservation,
) -> Vec<SpeciesAreaAudit> {
    let values = [
        (
            "N",
            before.n_concentration,
            after.n_concentration,
            before.n_amount,
            after.n_amount,
        ),
        (
            "F",
            before.f_concentration,
            after.f_concentration,
            before.f_amount,
            after.f_amount,
        ),
        (
            "A",
            before.a_concentration,
            after.a_concentration,
            before.a_amount,
            after.a_amount,
        ),
        (
            "R",
            before.r_concentration,
            after.r_concentration,
            before.r_amount,
            after.r_amount,
        ),
        (
            "C",
            before.c_concentration,
            after.c_concentration,
            before.c_amount,
            after.c_amount,
        ),
        (
            "W",
            before.w_concentration,
            after.w_concentration,
            before.w_amount,
            after.w_amount,
        ),
    ];
    values
        .into_iter()
        .map(
            |(species, c_before, c_after, mass_before, mass_after)| SpeciesAreaAudit {
                species,
                concentration_before: c_before,
                concentration_after: c_after,
                mass_before,
                mass_after,
                conservative_concentration_after: if after.area > 0.0 {
                    mass_before / after.area
                } else {
                    0.0
                },
                fixed_concentration_geometry_delta: c_before * (after.area - before.area),
                observed_mass_delta: mass_after - mass_before,
            },
        )
        .collect()
}

#[derive(Debug, Clone, Serialize)]
struct StageDelta {
    observed_strict_delta: f64,
    expected_strict_delta: f64,
    residual: f64,
}

fn stage_delta(
    before: &MaterialObservation,
    after: &MaterialObservation,
    expected: f64,
) -> StageDelta {
    let observed = after.strict_material - before.strict_material;
    StageDelta {
        observed_strict_delta: observed,
        expected_strict_delta: expected,
        residual: observed - expected,
    }
}

#[derive(Debug, Clone, Serialize)]
struct StepLedger {
    step: usize,
    s0_step_entry: MaterialObservation,
    s1_after_finite_uptake: MaterialObservation,
    s2_after_reactions: MaterialObservation,
    s3_after_mechanics: MaterialObservation,
    s4_after_remesh: MaterialObservation,
    s5_after_rebond: MaterialObservation,
    mechanics_area_audit: Vec<SpeciesAreaAudit>,
    remesh_area_audit: Vec<SpeciesAreaAudit>,
    uptake_exposed_edges: usize,
    n_world_loss: f64,
    f_world_loss: f64,
    n_delivered: f64,
    f_delivered: f64,
    remesh_splits: usize,
    remesh_merges: usize,
    rebonded: bool,
    delta_strict_uptake: StageDelta,
    delta_strict_reaction: StageDelta,
    delta_strict_mechanics: StageDelta,
    delta_strict_remesh: StageDelta,
    delta_strict_rebond: StageDelta,
}

#[derive(Debug, Clone, Default, Serialize)]
struct AttributionAccumulator {
    uptake_observed: f64,
    uptake_expected_world: f64,
    uptake_residual: f64,
    reaction_residual: f64,
    mechanics_residual: f64,
    remesh_residual: f64,
    rebond_residual: f64,
    mechanics_fixed_concentration_delta: f64,
    remesh_fixed_concentration_delta: f64,
}

impl AttributionAccumulator {
    fn absorb(&mut self, ledger: &StepLedger) {
        self.uptake_observed += ledger.delta_strict_uptake.observed_strict_delta;
        self.uptake_expected_world += ledger.n_world_loss + ledger.f_world_loss;
        self.uptake_residual += ledger.delta_strict_uptake.residual;
        self.reaction_residual += ledger.delta_strict_reaction.residual;
        self.mechanics_residual += ledger.delta_strict_mechanics.residual;
        self.remesh_residual += ledger.delta_strict_remesh.residual;
        self.rebond_residual += ledger.delta_strict_rebond.residual;
        self.mechanics_fixed_concentration_delta += ledger
            .mechanics_area_audit
            .iter()
            .map(|x| x.fixed_concentration_geometry_delta)
            .sum::<f64>();
        self.remesh_fixed_concentration_delta += ledger
            .remesh_area_audit
            .iter()
            .map(|x| x.fixed_concentration_geometry_delta)
            .sum::<f64>();
    }

    fn reconstructed_residual(&self) -> f64 {
        self.uptake_residual
            + self.reaction_residual
            + self.mechanics_residual
            + self.remesh_residual
            + self.rebond_residual
    }

    fn geometry_residual(&self) -> f64 {
        self.mechanics_residual + self.remesh_residual
    }

    fn geometry_fixed_concentration_delta(&self) -> f64 {
        self.mechanics_fixed_concentration_delta + self.remesh_fixed_concentration_delta
    }
}

#[derive(Debug, Clone, Serialize)]
struct IsolationResult {
    executed: bool,
    strict_delta: f64,
    area_before: f64,
    area_after: f64,
    interior_mass_change: BTreeMap<String, f64>,
    area_audit: Vec<SpeciesAreaAudit>,
}

#[derive(Debug, Clone, Serialize)]
struct ReproductionResult {
    r6_starting_head: &'static str,
    plain_trajectory_hash: String,
    trajectory_hash: String,
    final_mesh_hash: String,
    expected_trajectory_hash: &'static str,
    expected_final_mesh_hash: &'static str,
    checkpoint_steps: Vec<usize>,
    checkpoint_hashes: BTreeMap<usize, String>,
    checkpoint_agreement: BTreeMap<usize, bool>,
    observer_trajectory_parity: bool,
    committed_checkpoint_agreement: bool,
}

#[derive(Debug, Clone, Serialize)]
struct AuditRun {
    initial: MaterialObservation,
    final_state: MaterialObservation,
    trajectory: Vec<String>,
    checkpoint_states: BTreeMap<usize, MaterialObservation>,
    checkpoint_r6_states: BTreeMap<usize, R6State>,
    attribution_1000: AttributionAccumulator,
    attribution_8000: AttributionAccumulator,
    first_permanent_resource_contact_loss_step: Option<usize>,
    first_geometry_change_step: Option<usize>,
    final_mesh: MaterialMesh,
    first_remesh_fixture: Option<MaterialMesh>,
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

fn reservoir(n_mass: f64, f_mass: f64) -> FiniteSpatialBackingReservoirV1 {
    FiniteSpatialBackingReservoirV1::new(
        RESOURCE_CENTER,
        RESOURCE_RADIUS,
        n_mass,
        f_mass,
        RESOURCE_CONCENTRATION,
        RESOURCE_CONCENTRATION,
    )
}

fn exposed_edges(world: Option<&FiniteSpatialBackingReservoirV1>, mesh: &MaterialMesh) -> usize {
    world
        .map(|w| {
            w.region
                .local_contact_signal(mesh)
                .into_iter()
                .filter(|v| *v > 0.0)
                .count()
        })
        .unwrap_or(0)
}

fn run_audited_step(
    mesh: &mut MaterialMesh,
    step: usize,
    mechanics: &MechParams,
    reactions: &ReactionParams,
    transport: &TransportParams,
    mut world: Option<&mut FiniteSpatialBackingReservoirV1>,
    first_remesh_fixture: &mut Option<MaterialMesh>,
) -> Result<StepLedger, Box<dyn std::error::Error>> {
    let s0 = observation(mesh, step, exposed_edges(world.as_deref(), mesh));
    let (n_before, f_before) = world
        .as_ref()
        .map(|w| (w.region.n_mass, w.region.f_mass))
        .unwrap_or((0.0, 0.0));
    let uptake = match world.as_deref_mut() {
        Some(world) => world.uptake(mesh, transport, mechanics.dt),
        None => Default::default(),
    };
    let n_after = world.as_ref().map(|w| w.region.n_mass).unwrap_or(0.0);
    let f_after = world.as_ref().map(|w| w.region.f_mass).unwrap_or(0.0);
    if world.is_some()
        && (!close(n_before - n_after, uptake.n_world_loss)
            || !close(f_before - f_after, uptake.f_world_loss))
    {
        return Err("finite world loss did not equal delivery".into());
    }
    let s1 = observation(mesh, step, uptake.exposed_edges);
    let _reaction = reactions_step(mesh, reactions, mechanics.dt, true, true);
    let s2 = observation(mesh, step, exposed_edges(world.as_deref(), mesh));
    if !mechanics_step(mesh, mechanics) {
        return Err("production mechanics step rejected".into());
    }
    let s3 = observation(mesh, step, exposed_edges(world.as_deref(), mesh));
    let pre_remesh = mesh.clone();
    let (splits, merges) = remesh(mesh);
    if (splits > 0 || merges > 0) && first_remesh_fixture.is_none() {
        *first_remesh_fixture = Some(pre_remesh);
    }
    let s4 = observation(mesh, step, exposed_edges(world.as_deref(), mesh));
    let rebonded = try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    let s5 = observation(mesh, step, exposed_edges(world.as_deref(), mesh));
    Ok(StepLedger {
        step,
        s0_step_entry: s0.clone(),
        s1_after_finite_uptake: s1.clone(),
        s2_after_reactions: s2.clone(),
        s3_after_mechanics: s3.clone(),
        s4_after_remesh: s4.clone(),
        s5_after_rebond: s5.clone(),
        mechanics_area_audit: species_audits(&s2, &s3),
        remesh_area_audit: species_audits(&s3, &s4),
        uptake_exposed_edges: uptake.exposed_edges,
        n_world_loss: uptake.n_world_loss,
        f_world_loss: uptake.f_world_loss,
        n_delivered: uptake.n_delivered,
        f_delivered: uptake.f_delivered,
        remesh_splits: splits,
        remesh_merges: merges,
        rebonded,
        delta_strict_uptake: stage_delta(&s0, &s1, uptake.n_world_loss + uptake.f_world_loss),
        delta_strict_reaction: stage_delta(&s1, &s2, 0.0),
        delta_strict_mechanics: stage_delta(&s2, &s3, 0.0),
        delta_strict_remesh: stage_delta(&s3, &s4, 0.0),
        delta_strict_rebond: stage_delta(&s4, &s5, 0.0),
    })
}

fn run_fed_audit(
    initial: &MaterialMesh,
    dense_root: Option<&Path>,
) -> Result<AuditRun, Box<dyn std::error::Error>> {
    let mut mesh = initial.clone();
    let mechanics = MechParams::default();
    let reactions = v3_params();
    let transport = TransportParams::default();
    let mut world = reservoir(RESOURCE_MASS, RESOURCE_MASS);
    let initial_observation = observation(&mesh, 0, exposed_edges(Some(&world), &mesh));
    let mut dense = dense_root
        .map(|root| File::create(root.join("stage_ledger.jsonl")).map(BufWriter::new))
        .transpose()?;
    if let Some(writer) = dense.as_mut() {
        serde_json::to_writer(
            &mut *writer,
            &json!({"stage": "initial", "observation": initial_observation}),
        )?;
        writer.write_all(b"\n")?;
    }
    let mut trajectory = vec![stable_json_hash(&r6_state(&mesh, 0))?];
    let mut checkpoint_states = BTreeMap::new();
    let mut checkpoint_r6_states = BTreeMap::new();
    checkpoint_states.insert(0, initial_observation.clone());
    checkpoint_r6_states.insert(0, r6_state(&mesh, 0));
    let mut attribution_1000 = AttributionAccumulator::default();
    let mut attribution_8000 = AttributionAccumulator::default();
    let mut first_remesh_fixture = None;
    let mut first_permanent_resource_contact_loss_step = None;
    let mut first_geometry_change_step = None;
    let mut last_delivery_step = None;
    for step in 1..=HORIZON {
        let ledger = run_audited_step(
            &mut mesh,
            step,
            &mechanics,
            &reactions,
            &transport,
            Some(&mut world),
            &mut first_remesh_fixture,
        )?;
        if ledger.s3_after_mechanics.area != ledger.s2_after_reactions.area
            || ledger.s4_after_remesh.area != ledger.s3_after_mechanics.area
            || ledger.s4_after_remesh.vertex_count != ledger.s3_after_mechanics.vertex_count
        {
            first_geometry_change_step.get_or_insert(step);
        }
        if ledger.n_delivered > 0.0 || ledger.f_delivered > 0.0 {
            last_delivery_step = Some(step);
            first_permanent_resource_contact_loss_step = None;
        } else if last_delivery_step.is_some() {
            first_permanent_resource_contact_loss_step.get_or_insert(step);
        }
        attribution_8000.absorb(&ledger);
        if step <= 1_000 {
            attribution_1000.absorb(&ledger);
        }
        let current = ledger.s5_after_rebond.clone();
        if let Some(writer) = dense.as_mut() {
            serde_json::to_writer(&mut *writer, &ledger)?;
            writer.write_all(b"\n")?;
        }
        trajectory.push(stable_json_hash(&r6_state(&mesh, step))?);
        if [
            1, 10, 100, 480, 1_000, 2_000, 3_466, 4_000, 6_000, 6_931, 8_000,
        ]
        .contains(&step)
        {
            checkpoint_states.insert(step, current);
            checkpoint_r6_states.insert(step, r6_state(&mesh, step));
        }
    }
    if let Some(writer) = dense.as_mut() {
        writer.flush()?;
    }
    Ok(AuditRun {
        initial: initial_observation,
        final_state: observation(&mesh, HORIZON, exposed_edges(Some(&world), &mesh)),
        trajectory,
        checkpoint_states,
        checkpoint_r6_states,
        attribution_1000,
        attribution_8000,
        first_permanent_resource_contact_loss_step,
        first_geometry_change_step,
        final_mesh: mesh,
        first_remesh_fixture,
    })
}

fn mechanics_only_isolation(
    initial: &MaterialMesh,
) -> Result<IsolationResult, Box<dyn std::error::Error>> {
    let mechanics = MechParams::default();
    let mut mesh = initial.clone();
    let before = observation(&mesh, 0, 0);
    let executed = mechanics_step(&mut mesh, &mechanics);
    let after = observation(&mesh, 0, 0);
    let audits = species_audits(&before, &after);
    Ok(IsolationResult {
        executed,
        strict_delta: after.strict_material - before.strict_material,
        area_before: before.area,
        area_after: after.area,
        interior_mass_change: audits
            .iter()
            .map(|audit| (audit.species.to_string(), audit.observed_mass_delta))
            .collect(),
        area_audit: audits,
    })
}

fn remesh_only_isolation(
    fixture: Option<MaterialMesh>,
) -> Result<IsolationResult, Box<dyn std::error::Error>> {
    let Some(mut mesh) = fixture else {
        return Ok(IsolationResult {
            executed: false,
            strict_delta: 0.0,
            area_before: 0.0,
            area_after: 0.0,
            interior_mass_change: BTreeMap::new(),
            area_audit: Vec::new(),
        });
    };
    let before = observation(&mesh, 0, 0);
    let (splits, merges) = remesh(&mut mesh);
    let after = observation(&mesh, 0, 0);
    let mut result = IsolationResult {
        executed: splits > 0 || merges > 0,
        strict_delta: after.strict_material - before.strict_material,
        area_before: before.area,
        area_after: after.area,
        interior_mass_change: species_audits(&before, &after)
            .iter()
            .map(|audit| (audit.species.to_string(), audit.observed_mass_delta))
            .collect(),
        area_audit: species_audits(&before, &after),
    };
    if !result.executed {
        result.area_audit.clear();
    }
    Ok(result)
}

fn load_r6_baseline() -> Result<Value, Box<dyn std::error::Error>> {
    let path = Path::new("experiments/generated/dcdev020m1r6/results.json");
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn run_plain(initial: &MaterialMesh) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut mesh = initial.clone();
    let mechanics = MechParams::default();
    let reactions = v3_params();
    let transport = TransportParams::default();
    let mut world = reservoir(RESOURCE_MASS, RESOURCE_MASS);
    let mut trajectory = vec![stable_json_hash(&r6_state(&mesh, 0))?];
    for step in 1..=HORIZON {
        let uptake = world.uptake(&mut mesh, &transport, mechanics.dt);
        if !close(uptake.n_world_loss, uptake.n_delivered)
            || !close(uptake.f_world_loss, uptake.f_delivered)
        {
            return Err("plain R6 replay world accounting failed".into());
        }
        let _ = reactions_step(&mut mesh, &reactions, mechanics.dt, true, true);
        if !mechanics_step(&mut mesh, &mechanics) {
            return Err("plain R6 replay mechanics failed".into());
        }
        remesh(&mut mesh);
        try_local_rebond(
            &mut mesh,
            chemistry_core::material_mesh::DEFAULT_REBOND_DIST,
        );
        trajectory.push(stable_json_hash(&r6_state(&mesh, step))?);
    }
    Ok((stable_json_hash(&trajectory)?, stable_json_hash(&mesh)?))
}

fn baseline_state_matches(actual: &R6State, expected: &Value) -> bool {
    let fields = [
        (
            "a",
            actual.a,
            expected["state"]["a"].as_f64().unwrap_or(f64::NAN),
        ),
        (
            "c",
            actual.c,
            expected["state"]["c"].as_f64().unwrap_or(f64::NAN),
        ),
        (
            "structural_m",
            actual.structural_m,
            expected["state"]["structural_m"]
                .as_f64()
                .unwrap_or(f64::NAN),
        ),
        (
            "strict_material",
            actual.strict_material,
            expected["state"]["strict_material"]
                .as_f64()
                .unwrap_or(f64::NAN),
        ),
        (
            "organized_material",
            actual.organized_material,
            expected["state"]["organized_material"]
                .as_f64()
                .unwrap_or(f64::NAN),
        ),
    ];
    fields
        .iter()
        .all(|(_, actual, expected)| close(*actual, *expected))
        && actual.vertex_count
            == expected["state"]["vertex_count"]
                .as_u64()
                .unwrap_or(usize::MAX as u64) as usize
        && actual.closed_intact
            == expected["state"]["closed_intact"]
                .as_bool()
                .unwrap_or(!actual.closed_intact)
        && actual.observer_viable
            == expected["state"]["observer_viable"]
                .as_bool()
                .unwrap_or(!actual.observer_viable)
}

fn reproduction(
    run: &AuditRun,
    baseline: &Value,
) -> Result<ReproductionResult, Box<dyn std::error::Error>> {
    let mut checkpoint_hashes = BTreeMap::new();
    let mut checkpoint_agreement = BTreeMap::new();
    for step in R6_REPRODUCTION_STEPS {
        checkpoint_hashes.insert(
            step,
            stable_json_hash(
                run.checkpoint_r6_states
                    .get(&step)
                    .ok_or("missing replay checkpoint")?,
            )?,
        );
    }
    let committed_checkpoints = baseline["arm_fed"]["checkpoints"]
        .as_array()
        .ok_or("missing R6 checkpoints")?;
    let (plain_trajectory_hash, plain_final_mesh_hash) =
        run_plain(&r5_entry::m1r1_entry_state().0)?;
    let observer_trajectory_hash = stable_json_hash(&run.trajectory)?;
    let observer_final_mesh_hash = stable_json_hash(&run.final_mesh)?;
    let observer_trajectory_parity = observer_trajectory_hash == plain_trajectory_hash;
    let mut agreement = plain_trajectory_hash == R6_TRAJECTORY_HASH;
    agreement &= plain_final_mesh_hash == R6_FINAL_MESH_HASH;
    agreement &= observer_trajectory_parity;
    agreement &= observer_final_mesh_hash == plain_final_mesh_hash;
    for expected in committed_checkpoints {
        let step = expected["step"]
            .as_u64()
            .ok_or("invalid R6 checkpoint step")? as usize;
        if let Some(actual) = run.checkpoint_r6_states.get(&step) {
            let matches = baseline_state_matches(actual, expected);
            checkpoint_agreement.insert(step, matches);
            agreement &= matches;
        }
    }
    Ok(ReproductionResult {
        r6_starting_head: R6_STARTING_HEAD,
        plain_trajectory_hash,
        trajectory_hash: observer_trajectory_hash,
        final_mesh_hash: observer_final_mesh_hash,
        expected_trajectory_hash: R6_TRAJECTORY_HASH,
        expected_final_mesh_hash: R6_FINAL_MESH_HASH,
        checkpoint_steps: R6_REPRODUCTION_STEPS.to_vec(),
        checkpoint_hashes,
        checkpoint_agreement,
        observer_trajectory_parity,
        committed_checkpoint_agreement: agreement,
    })
}

fn attribution_json(acc: &AttributionAccumulator, initial: f64, final_state: f64) -> Value {
    let observed_residual = final_state - initial - acc.uptake_expected_world;
    let reconstructed = acc.reconstructed_residual();
    json!({
        "uptake_cumulative_observed": acc.uptake_observed,
        "uptake_cumulative_expected_world": acc.uptake_expected_world,
        "uptake_cumulative_residual": acc.uptake_residual,
        "reaction_cumulative_residual": acc.reaction_residual,
        "mechanics_cumulative_residual": acc.mechanics_residual,
        "remesh_cumulative_residual": acc.remesh_residual,
        "rebond_cumulative_residual": acc.rebond_residual,
        "mechanics_fixed_concentration_geometry_delta": acc.mechanics_fixed_concentration_delta,
        "remesh_fixed_concentration_geometry_delta": acc.remesh_fixed_concentration_delta,
        "geometry_cumulative_residual": acc.geometry_residual(),
        "geometry_fixed_concentration_delta": acc.geometry_fixed_concentration_delta(),
        "unexplained_residual": observed_residual - reconstructed,
        "total_reconstructed_residual": reconstructed,
        "observed_r6_residual_signed": observed_residual,
        "observed_r6_residual_abs": observed_residual.abs(),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R6R1_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r6r1"));
    fs::create_dir_all(&out)?;
    let dense_root = std::env::var_os("DCDEV020M1R6R1_DENSE_OUTPUT").map(PathBuf::from);
    if let Some(root) = dense_root.as_ref() {
        fs::create_dir_all(root)?;
    }
    let (entry, mechanics) = r5_entry::m1r1_entry_state();
    assert!(close(mechanics.dt, DT));
    let run = run_fed_audit(&entry, dense_root.as_deref())?;
    let baseline = load_r6_baseline()?;
    let reproduction = reproduction(&run, &baseline)?;
    let mechanics_isolation = mechanics_only_isolation(&entry)?;
    let remesh_isolation = remesh_only_isolation(run.first_remesh_fixture.clone())?;
    let attribution_1000 = attribution_json(
        &run.attribution_1000,
        run.initial.strict_material,
        run.checkpoint_states
            .get(&1_000)
            .ok_or("missing step 1000")?
            .strict_material,
    );
    let attribution_8000 = attribution_json(
        &run.attribution_8000,
        run.initial.strict_material,
        run.final_state.strict_material,
    );
    let geometry_residual = run.attribution_8000.geometry_residual();
    let geometry_fixed = run.attribution_8000.geometry_fixed_concentration_delta();
    let observed_residual = attribution_8000["observed_r6_residual_signed"]
        .as_f64()
        .unwrap_or(f64::NAN);
    let geometry_dominant = geometry_residual.abs() >= 0.95 * observed_residual.abs();
    let geometry_mass_coupling_matches = close(geometry_residual, geometry_fixed);
    let contact_loss_follows_geometry_change = match (
        run.first_permanent_resource_contact_loss_step,
        run.first_geometry_change_step,
    ) {
        (Some(contact), Some(geometry)) => {
            if contact >= geometry {
                "YES"
            } else {
                "NO"
            }
        }
        _ => "UNRESOLVED",
    };
    let stage_ledger_pass = close(
        run.initial.strict_material
            + run.attribution_8000.uptake_expected_world
            + run.attribution_8000.reconstructed_residual(),
        run.final_state.strict_material,
    ) && attribution_8000["unexplained_residual"]
        .as_f64()
        .unwrap_or(f64::INFINITY)
        .abs()
        <= TOL;
    let classification = if !reproduction.committed_checkpoint_agreement || !stage_ledger_pass {
        "M1_RUNTIME_CLOSURE_AUDIT_INVALID"
    } else if geometry_dominant && geometry_mass_coupling_matches {
        "M1_RUNTIME_GEOMETRY_MASS_COUPLING_CONFIRMED"
    } else if !close(
        run.attribution_8000.reconstructed_residual(),
        observed_residual,
    ) {
        "M1_RUNTIME_CLOSURE_CAUSE_UNRESOLVED"
    } else {
        "M1_RUNTIME_CLOSURE_OTHER_STAGE_CONFIRMED"
    };
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "r6_authority": {"starting_head": R6_STARTING_HEAD, "classification": "M1_FULL_RUNTIME_CERTIFICATION_INVALID", "ci": "32673647585"},
        "runtime_order": ["S0 step entry", "S1 finite uptake", "S2 reactions", "S3 mechanics", "S4 remesh", "S5 rebond"],
        "observer_only": true,
        "dt": DT,
        "horizon": HORIZON,
        "tolerance": TOL,
        "resource": {"center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "initial_n": RESOURCE_MASS, "initial_f": RESOURCE_MASS, "boundary_n": RESOURCE_CONCENTRATION, "boundary_f": RESOURCE_CONCENTRATION, "replenishment_events": 0},
        "dense_output": SHARED_DENSE_ROOT,
        "execution_dense_output": dense_root.as_ref().map(|p| p.display().to_string()),
        "canonical_shared_dense_output": SHARED_DENSE_ROOT,
        "production_biology_changed": false,
        "mechanics_changed": false,
        "remesh_changed": false,
        "chemistry_changed": false,
        "parameter_search": false,
        "controller_added": false,
        "recycling": false,
        "next_execution_started": false,
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "r6_reproduction": reproduction,
        "mechanics_only_isolation": mechanics_isolation,
        "remesh_only_isolation": remesh_isolation,
        "attribution_1000": attribution_1000,
        "attribution_8000": attribution_8000,
        "first_permanent_resource_contact_loss_step": run.first_permanent_resource_contact_loss_step,
        "first_geometry_change_step": run.first_geometry_change_step,
        "contact_loss_follows_geometry_change": contact_loss_follows_geometry_change,
        "stage_ledger_pass": stage_ledger_pass,
        "classification": classification,
        "production_biology_changed": false,
        "mechanics_changed": false,
        "remesh_changed": false,
        "chemistry_changed": false,
        "parameter_search": false,
        "controller_added": false,
        "recycling": false,
        "next_execution_started": false,
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "e0_authority": reproduction.committed_checkpoint_agreement,
        "e1_stage_ledger": stage_ledger_pass,
        "e2_isolation": {"mechanics_only": mechanics_isolation.executed, "remesh_only": remesh_isolation.executed, "rebond": "NOT_EXERCISED"},
        "e3_cumulative_attribution": attribution_8000["unexplained_residual"].as_f64().unwrap_or(f64::INFINITY).abs() <= TOL,
        "e4_preservation": "REMOTE_CI_REQUIRED",
        "e5_remote": "PENDING",
        "classification": classification,
        "shared_drive_dense_evidence": dense_root.as_ref().map(|p| p.starts_with(r"\\RPI5\")).unwrap_or(false),
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
    let manifest = json!({"schema": "dcdev020m1r6r1_manifest_v1", "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"], "dense_output": SHARED_DENSE_ROOT, "shared_drive_required": true, "sha256": "computed-by-workflow"});
    fs::write(
        out.join("artifact_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("classification={classification}");
    println!(
        "r6_reproduction={}",
        reproduction.committed_checkpoint_agreement
    );
    println!(
        "mechanics_only_strict_delta={}",
        mechanics_isolation.strict_delta
    );
    println!("remesh_only_strict_delta={}", remesh_isolation.strict_delta);
    println!("geometry_residual={geometry_residual}");
    println!("geometry_fixed_concentration_delta={geometry_fixed}");
    println!(
        "first_permanent_resource_contact_loss_step={:?}",
        run.first_permanent_resource_contact_loss_step
    );
    Ok(())
}
