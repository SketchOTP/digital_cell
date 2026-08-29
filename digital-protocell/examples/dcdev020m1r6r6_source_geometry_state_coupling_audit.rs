//! DC-DEV-020-M1-R6-R6 observer-only source/geometry state-coupling audit.
//!
//! This package replays the accepted R6-R5 source schedules and records the
//! stock integrals and structural factors needed to separate source-history
//! loading from moving-geometry structural cycling.  No observer value is
//! written back into the production trajectory.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r5_entry;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{
    g_strain, q_catalyst, reactions_step, structural_build_flux, try_local_rebond, ReactionLedger,
    ReactionParams,
};
use chemistry_core::mesh_transport::{permeability, TransportParams};
use phase1_certifier::frozen::FROZEN_CENTER;
use regulatory_core::{stable_json_hash, FiniteSpatialBackingReservoirV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R6-R6-SOURCE-GEOMETRY-STATE-COUPLING-AUDIT-001";
const STARTING_HEAD: &str = "73067f702a8f5386c440629c454e40ab1e434e91";
const DT: f64 = 0.02;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const RESOURCE_CONCENTRATION: f64 = 2.063914918930895;
const HORIZON: usize = 8_000;
const DEPRIVATION: usize = 480;
const TOLERANCE: f64 = 1e-8;
const TARGET_EQUAL_TOTAL: f64 = 162.4646405383817;
const CHECKPOINTS: [usize; 7] = [0, 480, 1_000, 2_000, 4_000, 6_000, 8_000];
const ATLAS_DENSE_ROOT: &str =
    r"\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r6";

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    area: f64,
    perimeter: f64,
    vertex_count: usize,
    n: f64,
    f: f64,
    a: f64,
    c: f64,
    structural_m: f64,
    organized_material: f64,
    strict_material: f64,
    resource_n: f64,
    resource_f: f64,
    delivery_n: f64,
    delivery_f: f64,
    closed_intact: bool,
    observer_viable: bool,
}

fn state(mesh: &MaterialMesh, step: usize, n: f64, f: f64, dn: f64, df: f64) -> State {
    let s = snapshot(mesh);
    State {
        step,
        area: mesh.area(),
        perimeter: mesh.perimeter(),
        vertex_count: mesh.n(),
        n: s.n,
        f: s.f,
        a: s.a,
        c: s.c,
        structural_m: s.structural_m,
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        resource_n: n,
        resource_f: f,
        delivery_n: dn,
        delivery_f: df,
        closed_intact: mesh.closed_intact(),
        observer_viable: mesh.observer_viable(),
    }
}

#[derive(Debug, Clone, Serialize, Default)]
struct SinkTotals {
    activation: f64,
    a_decay: f64,
    catalyst_production: f64,
    catalyst_turnover: f64,
    structural_production: f64,
    structural_turnover: f64,
    membrane_production: f64,
    membrane_turnover: f64,
}

impl SinkTotals {
    fn absorb(&mut self, ledger: &ReactionLedger) {
        self.activation += ledger.a_produced;
        self.a_decay += ledger.a_decayed;
        self.catalyst_production += ledger.c_produced;
        self.catalyst_turnover += ledger.c_turned;
        self.structural_production += ledger.m_produced;
        self.structural_turnover += ledger.m_to_w;
        self.membrane_production += ledger.l_produced;
        self.membrane_turnover += ledger.bind_extent + ledger.unbind_extent;
    }

    fn organized_loss(&self) -> f64 {
        self.a_decay + self.catalyst_turnover + self.structural_turnover
    }
}

#[derive(Debug, Clone, Serialize)]
struct SourceStep {
    step: usize,
    n: f64,
    f: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
struct SourceObservation {
    exposed_edges: usize,
    intact_edges: usize,
    exposed_length: f64,
    intact_length: f64,
    exposed_permeability_length: f64,
    intact_permeability_length: f64,
    n_drive: f64,
    f_drive: f64,
    exposed_potential_n: f64,
    exposed_potential_f: f64,
    all_intact_potential_n: f64,
    all_intact_potential_f: f64,
    actual_n: f64,
    actual_f: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
struct StructuralFactors {
    a_concentration: f64,
    q_c: f64,
    strain_gain_length_weighted: f64,
    edge_geometry: f64,
    build_flux: f64,
    build_factor_product: f64,
    mean_strain: f64,
    m_amount: f64,
    m_turnover: f64,
    zero_strain_turnover: f64,
}

#[derive(Debug, Clone, Serialize)]
struct StepRecord {
    step: usize,
    pre: State,
    reaction_input: State,
    post: State,
    source: SourceObservation,
    sinks: SinkTotals,
    factors: StructuralFactors,
    a_decay_material_time: f64,
    a_material_time: f64,
    c_material_time: f64,
    m_material_time: f64,
    organized_identity_residual: f64,
    strict_residual: f64,
    mechanics_residual: f64,
    remesh_residual: f64,
    rebond_residual: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
struct Closure {
    max_organized_identity_residual: f64,
    max_strict_residual: f64,
    max_mechanics_residual: f64,
    max_remesh_residual: f64,
    max_rebond_residual: f64,
}

impl Closure {
    fn pass(&self) -> bool {
        [
            self.max_organized_identity_residual,
            self.max_strict_residual,
            self.max_mechanics_residual,
            self.max_remesh_residual,
            self.max_rebond_residual,
        ]
        .into_iter()
        .all(|v| v <= TOLERANCE)
    }
}

#[derive(Debug, Clone, Serialize)]
struct SourceProfile {
    total_n: f64,
    total_f: f64,
    step_10_percent: Option<usize>,
    step_25_percent: Option<usize>,
    step_50_percent: Option<usize>,
    step_75_percent: Option<usize>,
    step_90_percent: Option<usize>,
    step_100_percent: Option<usize>,
    maximum_per_step: f64,
    median_positive_delivery: f64,
    last_positive_delivery_step: Option<usize>,
    resource_exhaustion_step: Option<usize>,
    zero_input_tail_duration: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    arm: String,
    mode: String,
    initial: State,
    final_state: State,
    checkpoints: Vec<State>,
    source_steps: usize,
    source_schedule_hash: String,
    n_delivered: f64,
    f_delivered: f64,
    n_remaining: f64,
    f_remaining: f64,
    sinks: SinkTotals,
    closure: Closure,
    organized_material_delta: f64,
    a_material_time: f64,
    a_decay_material_time: f64,
    c_material_time: f64,
    m_material_time: f64,
    max_exposed_edges: usize,
    last_positive_delivery_step: Option<usize>,
    trajectory_hash: String,
    final_mesh_hash: String,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Moving,
    Frozen,
}

impl Mode {
    fn id(self) -> &'static str {
        match self {
            Self::Moving => "MOVING_FULL_RUNTIME_DIAGNOSTIC",
            Self::Frozen => "GEOMETRY_FROZEN_STATIC_DIAGNOSTIC",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SourceMode<'a> {
    None,
    Spatial,
    AllIntact,
    Schedule(&'a [SourceStep]),
}

impl SourceMode<'_> {
    fn id(self) -> &'static str {
        match self {
            Self::None => "NO_RESOURCE",
            Self::Spatial => "FINITE_SPATIAL_RESOURCE",
            Self::AllIntact => "ALL_INTACT_EDGE_UPPER_BOUND",
            Self::Schedule(_) => "SEALED_PER_STEP_SCHEDULE",
        }
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE * (1.0 + a.abs().max(b.abs()))
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

fn edge_midpoint(mesh: &MaterialMesh, i: usize) -> [f64; 2] {
    [
        (mesh.vertices[i][0] + mesh.vertices[(i + 1) % mesh.n()][0]) * 0.5,
        (mesh.vertices[i][1] + mesh.vertices[(i + 1) % mesh.n()][1]) * 0.5,
    ]
}

fn edge_exposed(mesh: &MaterialMesh, i: usize) -> bool {
    let mid = edge_midpoint(mesh, i);
    (mid[0] - RESOURCE_CENTER[0]).hypot(mid[1] - RESOURCE_CENTER[1]) <= RESOURCE_RADIUS
}

fn source_observation(
    mesh: &MaterialMesh,
    world: Option<&FiniteSpatialBackingReservoirV1>,
    transport: &TransportParams,
    all_intact: bool,
) -> SourceObservation {
    let mut observation = SourceObservation::default();
    let Some(world) = world else {
        return observation;
    };
    observation.n_drive =
        (world.fixed_boundary_n_concentration - mesh.interior.n.max(0.0)).max(0.0);
    observation.f_drive =
        (world.fixed_boundary_f_concentration - mesh.interior.f.max(0.0)).max(0.0);
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        let length = mesh.edge_length(i);
        let exposed = edge_exposed(mesh, i);
        let theta = mesh.occupancy(i);
        let perm_n = permeability(theta, "N");
        let perm_f = permeability(theta, "F");
        let n_request = transport.k_flux * perm_n * observation.n_drive * length * DT;
        let f_request = transport.k_flux * perm_f * observation.f_drive * length * DT;
        observation.intact_edges += 1;
        observation.intact_length += length;
        observation.intact_permeability_length += perm_n.min(perm_f) * length;
        observation.all_intact_potential_n += n_request.max(0.0);
        observation.all_intact_potential_f += f_request.max(0.0);
        if exposed {
            observation.exposed_edges += 1;
            observation.exposed_length += length;
            observation.exposed_permeability_length += perm_n.min(perm_f) * length;
            observation.exposed_potential_n += n_request.max(0.0);
            observation.exposed_potential_f += f_request.max(0.0);
        }
    }
    if all_intact {
        observation.exposed_edges = observation.intact_edges;
        observation.exposed_length = observation.intact_length;
        observation.exposed_permeability_length = observation.intact_permeability_length;
        observation.exposed_potential_n = observation.all_intact_potential_n;
        observation.exposed_potential_f = observation.all_intact_potential_f;
    }
    observation
}

fn all_intact_projection(
    mesh: &mut MaterialMesh,
    world: &FiniteSpatialBackingReservoirV1,
    transport: &TransportParams,
    dt: f64,
) -> (f64, f64) {
    let area = mesh.area().max(1e-6);
    let mut n = 0.0;
    let mut f = 0.0;
    let mut interior_n = mesh.interior.n.max(0.0);
    let mut interior_f = mesh.interior.f.max(0.0);
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        let length = mesh.edge_length(i);
        let theta = mesh.occupancy(i);
        let n_request = transport.k_flux
            * permeability(theta, "N")
            * (world.fixed_boundary_n_concentration - interior_n).max(0.0)
            * length
            * dt;
        let f_request = transport.k_flux
            * permeability(theta, "F")
            * (world.fixed_boundary_f_concentration - interior_f).max(0.0)
            * length
            * dt;
        let dn = n_request.max(0.0).min((world.region.n_mass - n).max(0.0));
        let df = f_request.max(0.0).min((world.region.f_mass - f).max(0.0));
        n += dn;
        f += df;
        interior_n += dn / area;
        interior_f += df / area;
    }
    (n, f)
}

fn apply_schedule(mesh: &mut MaterialMesh, n: f64, f: f64) {
    let area = mesh.area().max(1e-6);
    mesh.interior.n += n / area;
    mesh.interior.f += f / area;
}

fn structural_factors(mesh: &MaterialMesh, p: &ReactionParams, dt: f64) -> StructuralFactors {
    let mut factors = StructuralFactors {
        a_concentration: mesh.interior.a.max(0.0),
        q_c: q_catalyst(mesh.interior.c, p.q_c),
        ..StructuralFactors::default()
    };
    let mut length = 0.0;
    let mut strain_length = 0.0;
    let mut strain_sum = 0.0;
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        let ell = mesh.edge_length(i);
        let strain = mesh.strain(i).max(0.0);
        let gain = g_strain(strain, p.g0, p.k_eps);
        length += ell;
        strain_length += gain * ell;
        strain_sum += strain;
        factors.build_flux += structural_build_flux(mesh, i, p) * dt;
        factors.m_amount += mesh.edges[i].m.max(0.0);
        let scale = 1.0 / (1.0 + 2.0 * strain);
        factors.m_turnover += p.k_turn * scale * mesh.edges[i].m.max(0.0) * dt;
        factors.zero_strain_turnover += p.k_turn * mesh.edges[i].m.max(0.0) * dt;
    }
    factors.edge_geometry = length;
    factors.strain_gain_length_weighted = if length > 0.0 {
        strain_length / length
    } else {
        0.0
    };
    factors.mean_strain = if mesh.n() > 0 {
        strain_sum / mesh.n() as f64
    } else {
        0.0
    };
    factors.build_factor_product = factors.a_concentration
        * factors.q_c
        * factors.strain_gain_length_weighted
        * factors.edge_geometry;
    factors
}

fn run_arm(
    initial: &MaterialMesh,
    mode: Mode,
    source_mode: SourceMode<'_>,
    steps: usize,
    name: &str,
    dense_root: Option<&Path>,
) -> Result<(ArmResult, MaterialMesh, Vec<SourceStep>, Vec<StepRecord>), Box<dyn std::error::Error>>
{
    let mut mesh = initial.clone();
    let mechanics = MechParams::default();
    assert_eq!(mechanics.dt, DT);
    let params = ReactionParams::conservative_v3();
    assert!(!params.reserve.enable);
    let transport = TransportParams::default();
    let mut world =
        matches!(source_mode, SourceMode::Spatial | SourceMode::AllIntact).then(reservoir);
    let initial_state = state(&mesh, 0, 0.0, 0.0, 0.0, 0.0);
    let mut checkpoints = vec![initial_state.clone()];
    let mut trajectory = vec![stable_json_hash(&initial_state)?];
    let mut source_schedule = Vec::with_capacity(steps);
    let mut records = Vec::with_capacity(steps);
    let mut schedule_remaining_n = if matches!(source_mode, SourceMode::Schedule(_)) {
        RESOURCE_MASS
    } else {
        0.0
    };
    let mut schedule_remaining_f = schedule_remaining_n;
    let mut sinks = SinkTotals::default();
    let mut closure = Closure::default();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut a_material_time = 0.0;
    let mut a_decay_material_time = 0.0;
    let mut c_material_time = 0.0;
    let mut m_material_time = 0.0;
    let mut max_exposed_edges = 0;
    let mut last_positive_delivery_step = None;
    let mut dense = dense_root
        .map(|root| File::create(root.join(format!("{name}.jsonl"))).map(BufWriter::new))
        .transpose()?;

    for step in 1..=steps {
        let pre = state(
            &mesh,
            step - 1,
            world
                .as_ref()
                .map(|w| w.region.n_mass)
                .unwrap_or(schedule_remaining_n),
            world
                .as_ref()
                .map(|w| w.region.f_mass)
                .unwrap_or(schedule_remaining_f),
            0.0,
            0.0,
        );
        let mut source = source_observation(
            &mesh,
            world.as_ref(),
            &transport,
            matches!(source_mode, SourceMode::AllIntact),
        );
        let (dn, df) = match source_mode {
            SourceMode::None => (0.0, 0.0),
            SourceMode::Spatial => {
                let w = world.as_mut().expect("spatial resource");
                let before_n = w.region.n_mass;
                let before_f = w.region.f_mass;
                let ledger = w.uptake(&mut mesh, &transport, mechanics.dt);
                assert!(ledger.conservation_error <= TOLERANCE);
                assert!(close(before_n - w.region.n_mass, ledger.n_world_loss));
                assert!(close(before_f - w.region.f_mass, ledger.f_world_loss));
                (ledger.n_delivered, ledger.f_delivered)
            }
            SourceMode::AllIntact => {
                let w = world.as_mut().expect("all-intact resource");
                let (n, f) = all_intact_projection(&mut mesh, w, &transport, mechanics.dt);
                w.region.n_mass -= n;
                w.region.f_mass -= f;
                apply_schedule(&mut mesh, n, f);
                (n, f)
            }
            SourceMode::Schedule(schedule) => {
                let item = schedule.get(step - 1).ok_or("sealed schedule too short")?;
                schedule_remaining_n -= item.n;
                schedule_remaining_f -= item.f;
                if schedule_remaining_n < -TOLERANCE || schedule_remaining_f < -TOLERANCE {
                    return Err("sealed schedule exceeded finite inventory".into());
                }
                apply_schedule(&mut mesh, item.n, item.f);
                (item.n, item.f)
            }
        };
        source.actual_n = dn;
        source.actual_f = df;
        source_schedule.push(SourceStep { step, n: dn, f: df });
        n_delivered += dn;
        f_delivered += df;
        if dn > 0.0 || df > 0.0 {
            last_positive_delivery_step = Some(step);
        }
        max_exposed_edges = max_exposed_edges.max(source.exposed_edges);
        let reaction_input = state(
            &mesh,
            step - 1,
            world
                .as_ref()
                .map(|w| w.region.n_mass)
                .unwrap_or(schedule_remaining_n),
            world
                .as_ref()
                .map(|w| w.region.f_mass)
                .unwrap_or(schedule_remaining_f),
            dn,
            df,
        );
        let before_reaction_organized = reaction_input.organized_material;
        let before_strict = reaction_input.strict_material;
        a_material_time += reaction_input.a * mechanics.dt;
        c_material_time += reaction_input.c * mechanics.dt;
        m_material_time += reaction_input.structural_m * mechanics.dt;
        let factors = structural_factors(&mesh, &params, mechanics.dt);
        let ledger = reactions_step(&mut mesh, &params, mechanics.dt, true, true);
        a_decay_material_time += (ledger.a_before_reserve + ledger.a_decayed) * mechanics.dt;
        let mut step_sinks = SinkTotals::default();
        step_sinks.absorb(&ledger);
        sinks.activation += step_sinks.activation;
        sinks.a_decay += step_sinks.a_decay;
        sinks.catalyst_production += step_sinks.catalyst_production;
        sinks.catalyst_turnover += step_sinks.catalyst_turnover;
        sinks.structural_production += step_sinks.structural_production;
        sinks.structural_turnover += step_sinks.structural_turnover;
        sinks.membrane_production += step_sinks.membrane_production;
        sinks.membrane_turnover += step_sinks.membrane_turnover;
        let after_reaction = snapshot(&mesh);
        let identity_expected = step_sinks.activation - step_sinks.organized_loss();
        let organized_identity_residual =
            after_reaction.organized_material() - before_reaction_organized - identity_expected;
        let strict_residual = after_reaction.strict_material_equivalent() - before_strict;
        let mut mechanics_residual = 0.0;
        let mut remesh_residual = 0.0;
        let mut rebond_residual = 0.0;
        if matches!(mode, Mode::Moving) {
            let before = snapshot(&mesh).strict_material_equivalent();
            if !mechanics_step(&mut mesh, &mechanics) {
                return Err(format!("mechanics rejected at step {step}").into());
            }
            mechanics_residual = snapshot(&mesh).strict_material_equivalent() - before;
            let before_remesh = snapshot(&mesh).strict_material_equivalent();
            let _ = remesh(&mut mesh);
            remesh_residual = snapshot(&mesh).strict_material_equivalent() - before_remesh;
            let before_rebond = snapshot(&mesh).strict_material_equivalent();
            let _ = try_local_rebond(
                &mut mesh,
                chemistry_core::material_mesh::DEFAULT_REBOND_DIST,
            );
            rebond_residual = snapshot(&mesh).strict_material_equivalent() - before_rebond;
        }
        closure.max_organized_identity_residual = closure
            .max_organized_identity_residual
            .max(organized_identity_residual.abs());
        closure.max_strict_residual = closure.max_strict_residual.max(strict_residual.abs());
        closure.max_mechanics_residual =
            closure.max_mechanics_residual.max(mechanics_residual.abs());
        closure.max_remesh_residual = closure.max_remesh_residual.max(remesh_residual.abs());
        closure.max_rebond_residual = closure.max_rebond_residual.max(rebond_residual.abs());
        let post = state(
            &mesh,
            step,
            world
                .as_ref()
                .map(|w| w.region.n_mass)
                .unwrap_or(schedule_remaining_n),
            world
                .as_ref()
                .map(|w| w.region.f_mass)
                .unwrap_or(schedule_remaining_f),
            dn,
            df,
        );
        if CHECKPOINTS.contains(&step) {
            checkpoints.push(post.clone());
        }
        let record = StepRecord {
            step,
            pre,
            reaction_input,
            post: post.clone(),
            source,
            sinks: step_sinks,
            factors,
            a_decay_material_time,
            a_material_time,
            c_material_time,
            m_material_time,
            organized_identity_residual,
            strict_residual,
            mechanics_residual,
            remesh_residual,
            rebond_residual,
        };
        trajectory.push(stable_json_hash(&post)?);
        if let Some(writer) = dense.as_mut() {
            serde_json::to_writer(&mut *writer, &record)?;
            writer.write_all(b"\n")?;
        }
        records.push(record);
    }
    if let Some(writer) = dense.as_mut() {
        writer.flush()?;
    }
    let final_state = records.last().map(|r| r.post.clone()).ok_or("empty arm")?;
    let schedule_hash = stable_json_hash(&source_schedule)?;
    let result = ArmResult {
        arm: name.into(),
        mode: format!("{}+{}", mode.id(), source_mode.id()),
        initial: initial_state.clone(),
        final_state: final_state.clone(),
        checkpoints,
        source_steps: source_schedule.len(),
        source_schedule_hash: schedule_hash,
        n_delivered,
        f_delivered,
        n_remaining: world
            .as_ref()
            .map(|w| w.region.n_mass)
            .unwrap_or(schedule_remaining_n),
        f_remaining: world
            .as_ref()
            .map(|w| w.region.f_mass)
            .unwrap_or(schedule_remaining_f),
        sinks,
        closure,
        organized_material_delta: final_state.organized_material - initial_state.organized_material,
        a_material_time,
        a_decay_material_time,
        c_material_time,
        m_material_time,
        max_exposed_edges,
        last_positive_delivery_step,
        trajectory_hash: stable_json_hash(&trajectory)?,
        final_mesh_hash: stable_json_hash(&mesh)?,
    };
    Ok((result, mesh, source_schedule, records))
}

fn profile(schedule: &[SourceStep]) -> SourceProfile {
    let total_n = schedule.iter().map(|x| x.n).sum::<f64>();
    let total_f = schedule.iter().map(|x| x.f).sum::<f64>();
    let mut cumulative = 0.0;
    let mut threshold = [None; 6];
    let mut positive = schedule
        .iter()
        .filter_map(|x| (x.n > 0.0).then_some(x.n))
        .collect::<Vec<_>>();
    positive.sort_by(f64::total_cmp);
    for item in schedule {
        cumulative += item.n;
        for (i, fraction) in [0.1, 0.25, 0.5, 0.75, 0.9, 1.0].into_iter().enumerate() {
            if threshold[i].is_none() && cumulative >= total_n * fraction - TOLERANCE {
                threshold[i] = Some(item.step);
            }
        }
    }
    let median_positive_delivery = if positive.is_empty() {
        0.0
    } else {
        positive[positive.len() / 2]
    };
    let last_positive_delivery_step = schedule
        .iter()
        .rev()
        .find(|x| x.n > 0.0 || x.f > 0.0)
        .map(|x| x.step);
    let zero_input_tail_duration = last_positive_delivery_step
        .map_or(schedule.len(), |step| schedule.len().saturating_sub(step));
    SourceProfile {
        total_n,
        total_f,
        step_10_percent: threshold[0],
        step_25_percent: threshold[1],
        step_50_percent: threshold[2],
        step_75_percent: threshold[3],
        step_90_percent: threshold[4],
        step_100_percent: threshold[5],
        maximum_per_step: schedule.iter().map(|x| x.n.max(x.f)).fold(0.0, f64::max),
        median_positive_delivery,
        last_positive_delivery_step,
        resource_exhaustion_step: threshold[5],
        zero_input_tail_duration,
    }
}

fn equal_total_frontloaded(upper: &[SourceStep]) -> Vec<SourceStep> {
    let mut remaining = TARGET_EQUAL_TOTAL;
    let mut result = Vec::with_capacity(upper.len());
    for item in upper {
        let take = item.n.min(remaining).max(0.0);
        result.push(SourceStep {
            step: item.step,
            n: take,
            f: take,
        });
        remaining -= take;
    }
    if remaining.abs() > 1e-10 {
        return Vec::new();
    }
    result
}

fn matched_decomposition(
    static_arm: &ArmResult,
    moving_arm: &ArmResult,
    static_records: &[StepRecord],
    moving_records: &[StepRecord],
) -> Value {
    let mut activation_delta = 0.0;
    let mut a_decay_delta = 0.0;
    let mut c_turnover_delta = 0.0;
    let mut m_turnover_delta = 0.0;
    let mut static_m_shadow = 0.0;
    let mut moving_m_shadow = 0.0;
    let mut static_strain_suppression = 0.0;
    let mut moving_strain_suppression = 0.0;
    let mut swap = [0.0_f64; 4];
    for (s, m) in static_records.iter().zip(moving_records) {
        activation_delta += m.sinks.activation - s.sinks.activation;
        a_decay_delta += m.sinks.a_decay - s.sinks.a_decay;
        c_turnover_delta += m.sinks.catalyst_turnover - s.sinks.catalyst_turnover;
        m_turnover_delta += m.sinks.structural_turnover - s.sinks.structural_turnover;
        static_m_shadow += s.factors.zero_strain_turnover;
        moving_m_shadow += m.factors.zero_strain_turnover;
        static_strain_suppression += s.factors.zero_strain_turnover - s.factors.m_turnover;
        moving_strain_suppression += m.factors.zero_strain_turnover - m.factors.m_turnover;
        let values = [
            s.factors.a_concentration,
            s.factors.q_c,
            s.factors.strain_gain_length_weighted,
            s.factors.edge_geometry,
        ];
        let moving_values = [
            m.factors.a_concentration,
            m.factors.q_c,
            m.factors.strain_gain_length_weighted,
            m.factors.edge_geometry,
        ];
        let moving_product = moving_values.iter().product::<f64>();
        for i in 0..4 {
            let mut shadow = moving_values;
            shadow[i] = values[i];
            swap[i] += moving_product - shadow.iter().product::<f64>();
        }
    }
    json!({
        "static_organized_delta": static_arm.organized_material_delta,
        "moving_organized_delta": moving_arm.organized_material_delta,
        "moving_minus_static": {
            "activation": activation_delta,
            "a_decay": a_decay_delta,
            "catalyst_turnover": c_turnover_delta,
            "structural_turnover": m_turnover_delta,
        },
        "static_m_material_time": static_arm.m_material_time,
        "moving_m_material_time": moving_arm.m_material_time,
        "static_zero_strain_turnover_shadow": static_m_shadow,
        "moving_zero_strain_turnover_shadow": moving_m_shadow,
        "static_strain_suppression": static_strain_suppression,
        "moving_strain_suppression": moving_strain_suppression,
        "structural_build_factor_swap_reduction": {
            "a_concentration": swap[0],
            "q_c": swap[1],
            "strain": swap[2],
            "edge_geometry": swap[3],
        },
        "static_a_material_time": static_arm.a_material_time,
        "static_a_decay_material_time": static_arm.a_decay_material_time,
        "moving_a_material_time": moving_arm.a_material_time,
        "moving_a_decay_material_time": moving_arm.a_decay_material_time,
        "static_c_material_time": static_arm.c_material_time,
        "moving_c_material_time": moving_arm.c_material_time,
    })
}

fn read_report(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|x| serde_json::from_str(&x).ok())
        .unwrap_or_else(|| json!({"status":"missing"}))
}

fn d087_pass(report: &Value, contract: &str) -> bool {
    report["mesh_contract"] == contract
        && report["reserve_enabled"] == false
        && (0..8).all(|i| report[format!("gate{i}")]["pass"] == true)
        && report["primary_conclusion"] == "D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED"
}

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R6R6_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r6r6"));
    let dense_root = std::env::var_os("DCDEV020M1R6R6_DENSE_OUTPUT")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(ATLAS_DENSE_ROOT)));
    fs::create_dir_all(&out)?;
    if let Some(root) = dense_root.as_ref() {
        fs::create_dir_all(root)?;
    }
    let (mut entry, mechanics) = r5_entry::m1r1_entry_state();
    entry.stamp_geometry_conservative_schema();
    assert_eq!(mechanics.dt, FROZEN_CENTER.dt);

    let (actual, _, _, _) = run_arm(
        &entry,
        Mode::Moving,
        SourceMode::Spatial,
        HORIZON,
        "actual_moving",
        dense_root.as_deref(),
    )?;
    let (frozen, _, frozen_schedule, frozen_records) = run_arm(
        &entry,
        Mode::Frozen,
        SourceMode::Spatial,
        HORIZON,
        "geometry_frozen",
        dense_root.as_deref(),
    )?;
    let (contact_upper, _, upper_schedule, _) = run_arm(
        &entry,
        Mode::Moving,
        SourceMode::AllIntact,
        HORIZON,
        "contact_upper_bound",
        dense_root.as_deref(),
    )?;
    let (matched_static, _, _, _) = run_arm(
        &entry,
        Mode::Frozen,
        SourceMode::Schedule(&upper_schedule),
        HORIZON,
        "matched_source_static",
        dense_root.as_deref(),
    )?;
    let (matched_moving, _, _, matched_moving_records) = run_arm(
        &entry,
        Mode::Moving,
        SourceMode::Schedule(&frozen_schedule),
        HORIZON,
        "matched_source_moving",
        dense_root.as_deref(),
    )?;
    let (matched_static_frozen, _, _, _) = run_arm(
        &entry,
        Mode::Frozen,
        SourceMode::Schedule(&frozen_schedule),
        HORIZON,
        "matched_source_static_frozen_schedule",
        dense_root.as_deref(),
    )?;
    let frontloaded = equal_total_frontloaded(&upper_schedule);
    if frontloaded.is_empty() {
        return Err("could not construct equal-total frontloaded schedule".into());
    }
    let (frontloaded_static, _, _, frontloaded_records) = run_arm(
        &entry,
        Mode::Frozen,
        SourceMode::Schedule(&frontloaded),
        HORIZON,
        "frontloaded_equal_total_static",
        dense_root.as_deref(),
    )?;

    let (deprived, deprived_mesh, _, _) = run_arm(
        &entry,
        Mode::Moving,
        SourceMode::None,
        DEPRIVATION,
        "recovery_deprivation",
        dense_root.as_deref(),
    )?;
    let (recovery, _, _, recovery_records) = run_arm(
        &deprived_mesh,
        Mode::Moving,
        SourceMode::Schedule(&upper_schedule),
        HORIZON,
        "recovery_refeed_upper",
        dense_root.as_deref(),
    )?;

    let d087_v3 = read_report(&out.join("v3_d087/certification/report.json"));
    let d087_v2 = read_report(&out.join("v2_d087/certification/report.json"));
    let preservation =
        d087_pass(&d087_v2, "ConservativeV2") && d087_pass(&d087_v3, "ConservativeV3");
    let decomposition = matched_decomposition(
        &matched_static_frozen,
        &matched_moving,
        &frozen_records,
        &matched_moving_records,
    );
    let frozen_profile = profile(&frozen_schedule);
    let upper_profile = profile(&upper_schedule);
    let frontloaded_profile = profile(&frontloaded);
    let source_history_causal = matched_static_frozen.organized_material_delta >= -TOLERANCE
        && frontloaded_static.organized_material_delta < -TOLERANCE;
    let frontloaded_a_decay_excess =
        frontloaded_static.sinks.a_decay - matched_static_frozen.sinks.a_decay;
    let frontloaded_c_turnover_excess =
        frontloaded_static.sinks.catalyst_turnover - matched_static_frozen.sinks.catalyst_turnover;
    let structural_turnover_excess =
        matched_moving.sinks.structural_turnover - matched_static_frozen.sinks.structural_turnover;
    let geometry_path = matched_moving.organized_material_delta < -TOLERANCE
        && matched_static_frozen.organized_material_delta >= -TOLERANCE
        && structural_turnover_excess > TOLERANCE
        && decomposition["structural_build_factor_swap_reduction"]
            .as_object()
            .is_some();
    let classification = if source_history_causal && geometry_path {
        "M1_SOURCE_FRONTLOAD_AND_GEOMETRY_STRUCTURAL_CYCLE_CONFIRMED"
    } else if source_history_causal {
        "M1_SOURCE_FRONTLOAD_CAUSAL_GEOMETRY_MECHANISM_UNRESOLVED"
    } else if geometry_path {
        "M1_GEOMETRY_STRUCTURAL_CYCLE_CAUSAL_SOURCE_PROFILE_NOT_CONFIRMED"
    } else {
        "M1_SOURCE_GEOMETRY_STATE_COUPLING_UNRESOLVED"
    };

    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "runtime": {"material":"GeometryConservativeV3", "chemistry":"ConservativeV3", "reserve":"OFF", "dt":DT},
        "resource": {"center":RESOURCE_CENTER, "radius":RESOURCE_RADIUS, "boundary_n":RESOURCE_CONCENTRATION, "boundary_f":RESOURCE_CONCENTRATION, "initial_n":RESOURCE_MASS, "initial_f":RESOURCE_MASS, "replenishment_events":0},
        "horizon": HORIZON,
        "deprivation": DEPRIVATION,
        "equal_total_target": TARGET_EQUAL_TOTAL,
        "arms": ["actual_moving", "geometry_frozen", "contact_upper_bound", "matched_source_static", "matched_source_static_frozen_schedule", "matched_source_moving", "frontloaded_equal_total_static", "recovery_deprivation", "recovery_refeed_upper"],
        "observer_only": true,
        "forbidden_changes": ["production coefficients", "chemistry", "mechanics", "transport", "resource geometry", "resource inventory", "recycling", "salvage", "reserve", "controller", "M2"],
        "next_execution_started": false,
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "r6_r5_reproduction": {"actual":actual, "geometry_frozen":frozen, "contact_upper_bound":contact_upper, "matched_source_static":matched_static, "matched_source_moving":matched_moving, "matched_source_static_frozen_schedule":matched_static_frozen},
        "source_profiles": {"frozen":frozen_profile, "contact_upper":upper_profile, "frontloaded_equal_total":frontloaded_profile},
        "frontloaded_equal_total_static": frontloaded_static,
        "source_history": {"causal":source_history_causal, "a_decay_excess":frontloaded_a_decay_excess, "c_turnover_excess":frontloaded_c_turnover_excess, "frozen_a_decay":matched_static_frozen.sinks.a_decay, "frontloaded_a_decay":frontloaded_static.sinks.a_decay, "frozen_c_turnover":matched_static_frozen.sinks.catalyst_turnover, "frontloaded_c_turnover":frontloaded_static.sinks.catalyst_turnover, "frontloaded_records":frontloaded_records.len()},
        "matched_source_decomposition": decomposition,
        "geometry_structural_turnover_excess": structural_turnover_excess,
        "recovery": {"deprived":deprived, "refed":recovery, "refeed_records":recovery_records.len()},
        "preservation_pass": preservation,
        "classification": classification,
        "production_changed": false,
        "parameter_search": false,
        "recycling_salvage_reserve_added": false,
        "target_size_or_homeostat_added": false,
        "m1": "NOT ESTABLISHED",
        "m2_authorized": false,
        "next_execution_started": false,
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "e0_authority": true,
        "e1_r6_r5_reproduction": actual.closure.pass() && frozen.closure.pass() && contact_upper.closure.pass() && matched_static_frozen.closure.pass() && matched_moving.closure.pass(),
        "e2_stock_sink_identity": close(matched_static_frozen.sinks.a_decay, 0.008 * matched_static_frozen.a_decay_material_time) && close(matched_static_frozen.sinks.catalyst_turnover, 0.01 * matched_static_frozen.c_material_time),
        "e3_source_profiles": true,
        "e4_equal_total_frontloaded_test": true,
        "e5_geometry_decomposition": geometry_path,
        "e6_recovery_correspondence": !recovery_records.is_empty(),
        "e7_preservation": preservation,
        "e8_remote_ci": "required",
        "observer_only": true,
        "classification": classification,
        "next_execution_started": false,
    });
    let preservation_json = json!({
        "historical_v2_d087": d087_pass(&d087_v2, "ConservativeV2"),
        "candidate_v3_d087": d087_pass(&d087_v3, "ConservativeV3"),
        "gc_conservation": "required_by_exact_workflow",
        "r6_r3_r3": "required_by_exact_workflow",
        "r6_r4": "required_by_exact_workflow",
        "r6_r5": "required_by_exact_workflow",
        "phase1": "required_by_exact_workflow",
        "d088": "required_by_exact_workflow",
        "d091": "required_by_exact_workflow",
        "evolution_harness": "required_by_exact_workflow",
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("preservation.json"), &preservation_json)?;
    write_json(
        &out.join("artifact_manifest.json"),
        &json!({"schema":"dcdev020m1r6r6_manifest_v1", "directive":DIRECTIVE, "starting_head":STARTING_HEAD, "files":["protocol.json","results.json","qualification.json","preservation.json","artifact_manifest.json"], "dense_output":ATLAS_DENSE_ROOT, "shared_drive_required":true, "sha256":"computed-by-workflow"}),
    )?;
    println!("DCDEV020M1R6R6_SOURCE_GEOMETRY_STATE_COUPLING_AUDIT_COMPLETE");
    println!("classification={classification}");
    println!(
        "static_frozen_organized_delta={}",
        matched_static_frozen.organized_material_delta
    );
    println!(
        "frontloaded_equal_total_static_organized_delta={}",
        frontloaded_static.organized_material_delta
    );
    println!(
        "matched_moving_organized_delta={}",
        matched_moving.organized_material_delta
    );
    println!("source_history_causal={source_history_causal}");
    println!("geometry_path={geometry_path}");
    println!("next_execution_started=false");
    Ok(())
}
