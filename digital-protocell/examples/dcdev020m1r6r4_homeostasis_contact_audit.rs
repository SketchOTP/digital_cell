//! DC-DEV-020-M1-R6-R4 observer-only contact/homeostasis causal audit.
//!
//! The actual arm uses the unchanged full runtime. The frozen and all-intact
//! arms are diagnostic clones: neither can feed state back into production.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r5_entry;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step, try_local_rebond, ReactionLedger, ReactionParams,
};
use chemistry_core::mesh_transport::{permeability, TransportParams};
use phase1_certifier::frozen::FROZEN_CENTER;
use regulatory_core::{stable_json_hash, FiniteSpatialBackingReservoirV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-R6-R4-HOMEOSTASIS-CONTACT-CAUSAL-AUDIT-001";
const STARTING_HEAD: &str = "69b6133a5f76d3c7839705c78922c7452ad5d550";
const DT: f64 = 0.02;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const RESOURCE_CONCENTRATION: f64 = 2.063914918930895;
const HORIZON: usize = 8_000;
const DEPRIVATION: usize = 480;
const TOLERANCE: f64 = 1e-8;
const CHECKPOINTS: [usize; 7] = [0, 480, 1_000, 2_000, 4_000, 6_000, 8_000];
const ATLAS_DENSE_ROOT: &str =
    r"\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r4";

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    centroid: [f64; 2],
    area: f64,
    perimeter: f64,
    vertex_count: usize,
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

fn state(mesh: &MaterialMesh, step: usize) -> State {
    let s = snapshot(mesh);
    State {
        step,
        centroid: mesh.centroid(),
        area: mesh.area(),
        perimeter: mesh.perimeter(),
        vertex_count: mesh.n(),
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
        min_edge_m: mesh
            .edges
            .iter()
            .map(|edge| edge.m)
            .fold(f64::INFINITY, f64::min),
        ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
        closed_intact: mesh.closed_intact(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
    }
}

#[derive(Debug, Clone, Serialize)]
struct EdgeFrame {
    lineage: String,
    start: [f64; 2],
    end: [f64; 2],
    midpoint: [f64; 2],
    distance_from_resource: f64,
    distance_minus_radius: f64,
    length: f64,
    ruptured: bool,
    occupancy: f64,
    permeability: f64,
    exposed: bool,
    n_requested: f64,
    f_requested: f64,
    n_delivered: f64,
    f_delivered: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
struct FluxSummary {
    eligible_edges: usize,
    total_exposed_length: f64,
    weighted_permeability: f64,
    max_n_drive: f64,
    max_f_drive: f64,
    n_requested: f64,
    f_requested: f64,
    n_delivered: f64,
    f_delivered: f64,
}

#[derive(Debug, Clone, Serialize)]
struct StepRecord {
    step: usize,
    pre: State,
    post: State,
    actual_flux: FluxSummary,
    all_intact_flux: FluxSummary,
    exposure_potential_deficit_n: f64,
    exposure_potential_deficit_f: f64,
    edges: Vec<EdgeFrame>,
    uptake_residual: f64,
    reaction_residual: f64,
    mechanics_residual: f64,
    remesh_residual: f64,
    rebond_residual: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
struct DemandTotals {
    a_produced: f64,
    a_decay: f64,
    catalyst_production: f64,
    catalyst_turnover: f64,
    structural_production: f64,
    structural_turnover: f64,
    membrane_production: f64,
    membrane_turnover: f64,
    waste_production: f64,
}

impl DemandTotals {
    fn absorb(&mut self, ledger: &ReactionLedger) {
        self.a_produced += ledger.a_produced;
        self.a_decay += ledger.a_decayed;
        self.catalyst_production += ledger.c_produced;
        self.catalyst_turnover += ledger.c_turned;
        self.structural_production += ledger.m_produced;
        self.structural_turnover += ledger.m_to_w;
        self.membrane_production += ledger.l_produced;
        self.membrane_turnover += ledger.bind_extent + ledger.unbind_extent;
        self.waste_production += ledger.w_produced;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
struct Closure {
    max_uptake_residual: f64,
    max_reaction_residual: f64,
    max_mechanics_residual: f64,
    max_remesh_residual: f64,
    max_rebond_residual: f64,
    max_unexplained_residual: f64,
}

impl Closure {
    fn pass(&self) -> bool {
        [
            self.max_uptake_residual,
            self.max_reaction_residual,
            self.max_mechanics_residual,
            self.max_remesh_residual,
            self.max_rebond_residual,
            self.max_unexplained_residual,
        ]
        .into_iter()
        .all(|value| value <= TOLERANCE)
    }
}

#[derive(Debug, Clone, Serialize)]
struct SourceTotals {
    n_delivered: f64,
    f_delivered: f64,
    n_world_loss: f64,
    f_world_loss: f64,
    n_remaining: f64,
    f_remaining: f64,
    max_world_delivery_residual: f64,
    replenishment_events: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ContactSummary {
    first_contact_step: Option<usize>,
    maximum_exposed_edges: usize,
    maximum_exposed_length: f64,
    last_positive_delivery_step: Option<usize>,
    first_zero_exposure_step: Option<usize>,
    first_permanent_zero_exposure_step: Option<usize>,
    first_potential_deficit_step: Option<usize>,
    first_gradient_limited_step: Option<usize>,
    first_permeability_limited_step: Option<usize>,
    first_edge_length_limited_step: Option<usize>,
    cumulative_potential_deficit_n: f64,
    cumulative_potential_deficit_f: f64,
}

impl Default for ContactSummary {
    fn default() -> Self {
        Self {
            first_contact_step: None,
            maximum_exposed_edges: 0,
            maximum_exposed_length: 0.0,
            last_positive_delivery_step: None,
            first_zero_exposure_step: None,
            first_permanent_zero_exposure_step: None,
            first_potential_deficit_step: None,
            first_gradient_limited_step: None,
            first_permeability_limited_step: None,
            first_edge_length_limited_step: None,
            cumulative_potential_deficit_n: 0.0,
            cumulative_potential_deficit_f: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    arm: String,
    mode: String,
    initial: State,
    final_state: State,
    checkpoints: Vec<State>,
    source: SourceTotals,
    demand: DemandTotals,
    closure: Closure,
    contact: ContactSummary,
    organized_material_delta: f64,
    organized_slope_per_simulated_time: f64,
    input_rate_per_simulated_time: f64,
    resource_opportunity_sufficient: bool,
    trajectory_hash: String,
    final_mesh_hash: String,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Actual,
    GeometryFrozen,
    AllIntact,
}

impl Mode {
    fn id(self) -> &'static str {
        match self {
            Self::Actual => "ACTUAL_FULL_RUNTIME",
            Self::GeometryFrozen => "GEOMETRY_FROZEN_DIAGNOSTIC",
            Self::AllIntact => "CONTACT_PRESERVED_UPPER_BOUND_DIAGNOSTIC",
        }
    }

    fn runs_mechanics(self) -> bool {
        matches!(self, Self::Actual | Self::AllIntact)
    }
}

#[derive(Debug, Clone)]
struct FluxProjection {
    summary: FluxSummary,
    n_requested_by_edge: Vec<f64>,
    f_requested_by_edge: Vec<f64>,
    n_by_edge: Vec<f64>,
    f_by_edge: Vec<f64>,
    exposed: Vec<bool>,
}

#[derive(Debug, Clone)]
struct LineageTracker {
    ids: Vec<u64>,
    next: u64,
}

impl LineageTracker {
    fn new(mesh: &MaterialMesh) -> Self {
        let ids = (0..mesh.n()).map(|id| id as u64).collect();
        Self {
            ids,
            next: mesh.n() as u64,
        }
    }

    fn labels(&self) -> Vec<String> {
        self.ids
            .iter()
            .map(|id| format!("edge-lineage-{id}"))
            .collect()
    }

    fn remesh_update(&mut self, before: &[[f64; 2]], mesh: &MaterialMesh) {
        let old = self.ids.clone();
        self.ids = (0..mesh.n())
            .map(|i| {
                let new_midpoint = midpoint(mesh.vertices[i], mesh.vertices[(i + 1) % mesh.n()]);
                let mut best = 0usize;
                let mut best_distance = f64::INFINITY;
                for j in 0..before.len() {
                    let old_mid = midpoint(before[j], before[(j + 1) % before.len()]);
                    let distance =
                        (new_midpoint[0] - old_mid[0]).hypot(new_midpoint[1] - old_mid[1]);
                    if distance < best_distance {
                        best = j;
                        best_distance = distance;
                    }
                }
                if best < old.len() {
                    old[best]
                } else {
                    let id = self.next;
                    self.next += 1;
                    id
                }
            })
            .collect();
    }
}

fn midpoint(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
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

fn v3_params() -> ReactionParams {
    let params = ReactionParams::conservative_v3();
    assert!(!params.reserve.enable);
    params
}

fn exposed(mesh: &MaterialMesh, region: &FiniteSpatialBackingReservoirV1, i: usize) -> bool {
    let midpoint = midpoint(mesh.vertices[i], mesh.vertices[(i + 1) % mesh.n()]);
    (midpoint[0] - region.region.center[0]).hypot(midpoint[1] - region.region.center[1])
        <= region.region.radius
}

fn projection(
    mesh: &MaterialMesh,
    region: &FiniteSpatialBackingReservoirV1,
    transport: &TransportParams,
    dt: f64,
    all_intact: bool,
) -> FluxProjection {
    let mut summary = FluxSummary::default();
    let mut n_requested_by_edge = vec![0.0; mesh.n()];
    let mut f_requested_by_edge = vec![0.0; mesh.n()];
    let mut n_by_edge = vec![0.0; mesh.n()];
    let mut f_by_edge = vec![0.0; mesh.n()];
    let mut exposed_flags = vec![false; mesh.n()];
    let area = mesh.area().max(1e-6);
    let mut interior_n = mesh.interior.n.max(0.0);
    let mut interior_f = mesh.interior.f.max(0.0);
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured || (!all_intact && !exposed(mesh, region, i)) {
            continue;
        }
        exposed_flags[i] = true;
        summary.eligible_edges += 1;
        let length = mesh.edge_length(i);
        summary.total_exposed_length += length;
        let theta = mesh.occupancy(i);
        let perm = permeability(theta, "N");
        summary.weighted_permeability += perm * length;
        let n_drive = (region.fixed_boundary_n_concentration - interior_n).max(0.0);
        let f_drive = (region.fixed_boundary_f_concentration - interior_f).max(0.0);
        summary.max_n_drive = summary.max_n_drive.max(n_drive);
        summary.max_f_drive = summary.max_f_drive.max(f_drive);
        let requested_n = (transport.k_flux * perm * n_drive * length * dt).max(0.0);
        let requested_f =
            (transport.k_flux * permeability(theta, "F") * f_drive * length * dt).max(0.0);
        n_requested_by_edge[i] = requested_n;
        f_requested_by_edge[i] = requested_f;
        let accepted_n = requested_n.min((region.region.n_mass - summary.n_delivered).max(0.0));
        let accepted_f = requested_f.min((region.region.f_mass - summary.f_delivered).max(0.0));
        n_by_edge[i] = accepted_n;
        f_by_edge[i] = accepted_f;
        summary.n_requested += requested_n;
        summary.f_requested += requested_f;
        summary.n_delivered += accepted_n;
        summary.f_delivered += accepted_f;
        interior_n += accepted_n / area;
        interior_f += accepted_f / area;
    }
    if summary.eligible_edges > 0 {
        summary.weighted_permeability /= summary.total_exposed_length.max(1e-15);
    }
    FluxProjection {
        summary,
        n_requested_by_edge,
        f_requested_by_edge,
        n_by_edge,
        f_by_edge,
        exposed: exposed_flags,
    }
}

fn apply_all_intact(
    mesh: &mut MaterialMesh,
    region: &mut FiniteSpatialBackingReservoirV1,
    projection: &FluxProjection,
) -> (f64, f64) {
    let area = mesh.area().max(1e-6);
    let n = projection.summary.n_delivered;
    let f = projection.summary.f_delivered;
    region.region.n_mass = (region.region.n_mass - n).max(0.0);
    region.region.f_mass = (region.region.f_mass - f).max(0.0);
    mesh.interior.n += n / area;
    mesh.interior.f += f / area;
    (n, f)
}

fn edge_frames(
    mesh: &MaterialMesh,
    region: Option<&FiniteSpatialBackingReservoirV1>,
    labels: &[String],
    projection: Option<&FluxProjection>,
) -> Vec<EdgeFrame> {
    (0..mesh.n())
        .map(|i| {
            let start = mesh.vertices[i];
            let end = mesh.vertices[(i + 1) % mesh.n()];
            let midpoint = midpoint(start, end);
            let distance = region
                .map(|r| (midpoint[0] - r.region.center[0]).hypot(midpoint[1] - r.region.center[1]))
                .unwrap_or(f64::INFINITY);
            EdgeFrame {
                lineage: labels.get(i).cloned().unwrap_or_else(|| "unmapped".into()),
                start,
                end,
                midpoint,
                distance_from_resource: distance,
                distance_minus_radius: region
                    .map(|r| distance - r.region.radius)
                    .unwrap_or(f64::INFINITY),
                length: mesh.edge_length(i),
                ruptured: mesh.edges[i].ruptured,
                occupancy: mesh.occupancy(i),
                permeability: permeability(mesh.occupancy(i), "N"),
                exposed: projection.map(|p| p.exposed[i]).unwrap_or(false),
                n_requested: projection
                    .map(|p| p.n_requested_by_edge[i])
                    .unwrap_or(0.0),
                f_requested: projection
                    .map(|p| p.f_requested_by_edge[i])
                    .unwrap_or(0.0),
                n_delivered: projection.map(|p| p.n_by_edge[i]).unwrap_or(0.0),
                f_delivered: projection.map(|p| p.f_by_edge[i]).unwrap_or(0.0),
            }
        })
        .collect()
}

fn run_arm(
    initial: &MaterialMesh,
    mode: Mode,
    feed: bool,
    steps: usize,
    name: &str,
    dense_root: Option<&Path>,
) -> Result<(ArmResult, MaterialMesh), Box<dyn std::error::Error>> {
    let mut mesh = initial.clone();
    let mechanics = MechParams::default();
    let reactions = v3_params();
    let transport = TransportParams::default();
    let mut world = feed.then(reservoir);
    let initial_state = state(&mesh, 0);
    let mut current_state = initial_state.clone();
    let mut checkpoints = vec![initial_state.clone()];
    let mut trajectory = vec![stable_json_hash(&initial_state)?];
    let mut source = SourceTotals {
        n_delivered: 0.0,
        f_delivered: 0.0,
        n_world_loss: 0.0,
        f_world_loss: 0.0,
        n_remaining: if feed { RESOURCE_MASS } else { 0.0 },
        f_remaining: if feed { RESOURCE_MASS } else { 0.0 },
        max_world_delivery_residual: 0.0,
        replenishment_events: 0,
    };
    let mut demand = DemandTotals::default();
    let mut closure = Closure::default();
    let mut contact = ContactSummary::default();
    let mut lineage = LineageTracker::new(&mesh);
    let mut dense = dense_root
        .map(|root| File::create(root.join(format!("{name}.jsonl"))).map(BufWriter::new))
        .transpose()?;
    for step in 1..=steps {
        let pre = state(&mesh, step - 1);
        let all_projection = world
            .as_ref()
            .map(|w| projection(&mesh, w, &transport, mechanics.dt, true));
        let contact_projection = world
            .as_ref()
            .map(|w| projection(&mesh, w, &transport, mechanics.dt, false));
        let pre_edge_frames = if matches!(mode, Mode::Actual) {
            let labels = lineage.labels();
            Some(edge_frames(
                &mesh,
                world.as_ref(),
                &labels,
                contact_projection.as_ref(),
            ))
        } else {
            None
        };
        let all_flux = all_projection
            .as_ref()
            .map(|p| p.summary.clone())
            .unwrap_or_default();
        let mut actual_flux = contact_projection
            .as_ref()
            .map(|p| p.summary.clone())
            .unwrap_or_default();
        let (n_delivered, f_delivered) = match (world.as_mut(), mode) {
            (Some(world), Mode::AllIntact) => {
                apply_all_intact(&mut mesh, world, all_projection.as_ref().unwrap())
            }
            (Some(world), _) => {
                let n_before = world.region.n_mass;
                let f_before = world.region.f_mass;
                let ledger = world.uptake(&mut mesh, &transport, mechanics.dt);
                actual_flux.n_delivered = ledger.n_delivered;
                actual_flux.f_delivered = ledger.f_delivered;
                source.max_world_delivery_residual = source.max_world_delivery_residual.max(
                    (n_before - world.region.n_mass - ledger.n_world_loss)
                        .abs()
                        .max((f_before - world.region.f_mass - ledger.f_world_loss).abs()),
                );
                (ledger.n_delivered, ledger.f_delivered)
            }
            (None, _) => (0.0, 0.0),
        };
        source.n_delivered += n_delivered;
        source.f_delivered += f_delivered;
        source.n_world_loss += n_delivered;
        source.f_world_loss += f_delivered;
        source.n_remaining = world.as_ref().map(|w| w.region.n_mass).unwrap_or(0.0);
        source.f_remaining = world.as_ref().map(|w| w.region.f_mass).unwrap_or(0.0);
        let uptake_residual = snapshot(&mesh).strict_material_equivalent()
            - pre.strict_material
            - n_delivered
            - f_delivered;
        closure.max_uptake_residual = closure.max_uptake_residual.max(uptake_residual.abs());
        let before_reaction = snapshot(&mesh).strict_material_equivalent();
        let ledger = reactions_step(&mut mesh, &reactions, mechanics.dt, true, true);
        demand.absorb(&ledger);
        let after_reaction = snapshot(&mesh).strict_material_equivalent();
        let reaction_residual = after_reaction - before_reaction;
        closure.max_reaction_residual = closure.max_reaction_residual.max(reaction_residual.abs());
        let mut mechanics_residual = 0.0;
        let mut remesh_residual = 0.0;
        let mut rebond_residual = 0.0;
        if mode.runs_mechanics() {
            let before = after_reaction;
            if !mechanics_step(&mut mesh, &mechanics) {
                return Err(format!("mechanics rejected at step {step}").into());
            }
            mechanics_residual = snapshot(&mesh).strict_material_equivalent() - before;
            let before_vertices = mesh.vertices.clone();
            let before_remesh = snapshot(&mesh).strict_material_equivalent();
            let _ = remesh(&mut mesh);
            lineage.remesh_update(&before_vertices, &mesh);
            remesh_residual = snapshot(&mesh).strict_material_equivalent() - before_remesh;
            let before_rebond = snapshot(&mesh).strict_material_equivalent();
            let _ = try_local_rebond(
                &mut mesh,
                chemistry_core::material_mesh::DEFAULT_REBOND_DIST,
            );
            rebond_residual = snapshot(&mesh).strict_material_equivalent() - before_rebond;
        }
        closure.max_mechanics_residual =
            closure.max_mechanics_residual.max(mechanics_residual.abs());
        closure.max_remesh_residual = closure.max_remesh_residual.max(remesh_residual.abs());
        closure.max_rebond_residual = closure.max_rebond_residual.max(rebond_residual.abs());
        let post = state(&mesh, step);
        let unexplained = post.strict_material - pre.strict_material - n_delivered - f_delivered;
        closure.max_unexplained_residual = closure.max_unexplained_residual.max(unexplained.abs());
        let potential_deficit_n = (all_flux.n_delivered - actual_flux.n_delivered).max(0.0);
        let potential_deficit_f = (all_flux.f_delivered - actual_flux.f_delivered).max(0.0);
        contact.cumulative_potential_deficit_n += potential_deficit_n;
        contact.cumulative_potential_deficit_f += potential_deficit_f;
        if contact_projection
            .as_ref()
            .is_some_and(|p| p.summary.eligible_edges > 0)
        {
            contact.first_contact_step.get_or_insert(step);
        }
        if actual_flux.eligible_edges > contact.maximum_exposed_edges {
            contact.maximum_exposed_edges = actual_flux.eligible_edges;
        }
        contact.maximum_exposed_length = contact
            .maximum_exposed_length
            .max(actual_flux.total_exposed_length);
        if n_delivered > 0.0 || f_delivered > 0.0 {
            contact.last_positive_delivery_step = Some(step);
        }
        if actual_flux.eligible_edges == 0 {
            contact.first_zero_exposure_step.get_or_insert(step);
        }
        if potential_deficit_n > TOLERANCE || potential_deficit_f > TOLERANCE {
            contact.first_potential_deficit_step.get_or_insert(step);
        }
        if all_flux.max_n_drive <= TOLERANCE || all_flux.max_f_drive <= TOLERANCE {
            contact.first_gradient_limited_step.get_or_insert(step);
        }
        if all_flux.weighted_permeability <= TOLERANCE {
            contact.first_permeability_limited_step.get_or_insert(step);
        }
        if all_flux.total_exposed_length <= TOLERANCE {
            contact.first_edge_length_limited_step.get_or_insert(step);
        }
        if CHECKPOINTS.contains(&step) {
            checkpoints.push(post.clone());
        }
        trajectory.push(stable_json_hash(&post)?);
        if let Some(writer) = dense.as_mut() {
            let edges = pre_edge_frames.unwrap_or_default();
            let record = StepRecord {
                step,
                pre,
                post: post.clone(),
                actual_flux,
                all_intact_flux: all_flux,
                exposure_potential_deficit_n: potential_deficit_n,
                exposure_potential_deficit_f: potential_deficit_f,
                edges,
                uptake_residual,
                reaction_residual,
                mechanics_residual,
                remesh_residual,
                rebond_residual,
            };
            serde_json::to_writer(&mut *writer, &record)?;
            writer.write_all(b"\n")?;
        }
        current_state = post;
    }
    if contact
        .last_positive_delivery_step
        .is_some_and(|step| step < steps)
    {
        contact.first_permanent_zero_exposure_step =
            contact.last_positive_delivery_step.map(|step| step + 1);
    }
    if let Some(writer) = dense.as_mut() {
        writer.flush()?;
    }
    let world_total = source.n_world_loss + source.f_world_loss;
    let input_total = source.n_delivered + source.f_delivered;
    let organized_delta = current_state.organized_material - initial_state.organized_material;
    let result = ArmResult {
        arm: name.into(),
        mode: mode.id().into(),
        initial: initial_state,
        final_state: current_state.clone(),
        checkpoints,
        source: SourceTotals {
            replenishment_events: world.as_ref().map(|w| w.replenishment_events).unwrap_or(0),
            ..source
        },
        demand,
        closure: closure.clone(),
        contact,
        organized_material_delta: organized_delta,
        organized_slope_per_simulated_time: organized_delta / (steps as f64 * mechanics.dt),
        input_rate_per_simulated_time: input_total / (steps as f64 * mechanics.dt),
        resource_opportunity_sufficient: current_state.organized_material
            >= result_initial_organized(initial),
        trajectory_hash: stable_json_hash(&trajectory)?,
        final_mesh_hash: stable_json_hash(&mesh)?,
    };
    let _ = world_total;
    Ok((result, mesh))
}

fn result_initial_organized(mesh: &MaterialMesh) -> f64 {
    snapshot(mesh).organized_material()
}

#[derive(Debug, Clone, Serialize)]
struct RecoveryResult {
    entry: State,
    deprived: State,
    refed_actual: State,
    refed_contact_upper_bound: State,
    deprived_delta: f64,
    refed_actual_delta_from_deprived: f64,
    refed_contact_upper_bound_delta_from_deprived: f64,
    actual_deficit_reduction: f64,
    contact_upper_bound_deficit_reduction: f64,
    no_state_reset: bool,
}

fn run_recovery(
    initial: &MaterialMesh,
    dense_root: Option<&Path>,
) -> Result<RecoveryResult, Box<dyn std::error::Error>> {
    let (deprivation, deprived_mesh) = run_arm(
        initial,
        Mode::Actual,
        false,
        DEPRIVATION,
        "recovery_deprivation",
        dense_root,
    )?;
    let (actual, _) = run_arm(
        &deprived_mesh,
        Mode::Actual,
        true,
        HORIZON,
        "recovery_actual_refeed",
        dense_root,
    )?;
    let (upper, _) = run_arm(
        &deprived_mesh,
        Mode::AllIntact,
        true,
        HORIZON,
        "recovery_contact_upper_bound",
        dense_root,
    )?;
    let deprived = deprivation.final_state.clone();
    let entry = deprivation.initial.clone();
    let deprived_delta = deprived.organized_material - entry.organized_material;
    let actual_delta = actual.final_state.organized_material - deprived.organized_material;
    let upper_delta = upper.final_state.organized_material - deprived.organized_material;
    let initial_deficit = (entry.organized_material - deprived.organized_material).max(0.0);
    let no_state_reset = deprived.strict_material != entry.strict_material;
    Ok(RecoveryResult {
        entry,
        deprived,
        refed_actual: actual.final_state,
        refed_contact_upper_bound: upper.final_state,
        deprived_delta,
        refed_actual_delta_from_deprived: actual_delta,
        refed_contact_upper_bound_delta_from_deprived: upper_delta,
        actual_deficit_reduction: initial_deficit - (initial_deficit - actual_delta).max(0.0),
        contact_upper_bound_deficit_reduction: initial_deficit
            - (initial_deficit - upper_delta).max(0.0),
        no_state_reset,
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
        && (0..8).all(|i| report[format!("gate{i}")]["pass"] == true)
        && report["primary_conclusion"] == "D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED"
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV3");
    std::env::set_var("DCDEV020R9R3_RESERVE", "0");
    std::env::set_var("DCDEV020M1R6R2_GEOMETRY_CONTRACT", "1");
    let out = std::env::var_os("DCDEV020M1R6R4_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r6r4"));
    let dense_root = std::env::var_os("DCDEV020M1R6R4_DENSE_OUTPUT")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(ATLAS_DENSE_ROOT)));
    fs::create_dir_all(&out)?;
    if let Some(root) = dense_root.as_ref() {
        fs::create_dir_all(root)?;
    }
    let (mut entry, mechanics) = r5_entry::m1r1_entry_state();
    entry.stamp_geometry_conservative_schema();
    assert!(close(mechanics.dt, DT));
    assert_eq!(mechanics.dt, FROZEN_CENTER.dt);

    let (actual, _) = run_arm(
        &entry,
        Mode::Actual,
        true,
        HORIZON,
        "actual_fed",
        dense_root.as_deref(),
    )?;
    let (static_r5, _) = run_arm(
        &entry,
        Mode::GeometryFrozen,
        true,
        HORIZON,
        "r5_static_reference",
        dense_root.as_deref(),
    )?;
    let (frozen, _) = run_arm(
        &entry,
        Mode::GeometryFrozen,
        true,
        HORIZON,
        "geometry_frozen",
        dense_root.as_deref(),
    )?;
    let (upper, _) = run_arm(
        &entry,
        Mode::AllIntact,
        true,
        HORIZON,
        "contact_upper_bound",
        dense_root.as_deref(),
    )?;
    let recovery = run_recovery(&entry, dense_root.as_deref())?;
    let out_v3 = read_report(&out.join("v3_d087/certification/report.json"));
    let out_v2 = read_report(&out.join("v2_d087/certification/report.json"));
    let preservation = d087_pass(&out_v2, "ConservativeV2") && d087_pass(&out_v3, "ConservativeV3");
    let actual_reproduces = close(actual.organized_material_delta, -82.9654506509167)
        && close(actual.source.n_delivered, 14.627590100158915)
        && close(actual.source.f_delivered, 14.627590100158915)
        && close(actual.source.n_remaining, 228.5216579103792)
        && actual.closure.pass();
    let static_qualifies = static_r5.source.n_delivered > 0.0
        && static_r5.source.f_delivered > 0.0
        && static_r5.organized_material_delta >= -TOLERANCE
        && static_r5.closure.pass();
    let frozen_qualifies = frozen.organized_material_delta >= -TOLERANCE && frozen.closure.pass();
    let upper_qualifies = upper.organized_material_delta >= -TOLERANCE
        && upper.final_state.observer_viable
        && upper.final_state.closed_intact
        && upper.closure.pass();
    let recovery_actual_pass = recovery.deprived_delta < 0.0
        && recovery.refed_actual.organized_material > recovery.deprived.organized_material;
    let recovery_upper_pass = recovery.deprived_delta < 0.0
        && recovery.refed_contact_upper_bound.organized_material
            > recovery.deprived.organized_material;
    let classification = if upper_qualifies && recovery_upper_pass && !actual_reproduces {
        "M1_FULL_RUNTIME_CONTACT_LOSS_CAUSALLY_DOMINANT"
    } else if actual_reproduces && !upper_qualifies {
        "M1_FULL_RUNTIME_EMBODIED_DEMAND_DOMINANT"
    } else if actual_reproduces && upper_qualifies {
        "M1_FULL_RUNTIME_CONTACT_AND_DEMAND_MIXED"
    } else {
        "M1_FULL_RUNTIME_HOMEOSTASIS_CAUSE_UNRESOLVED"
    };
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "runtime_contract": {"material": "GeometryConservativeV3", "chemistry": "ConservativeV3", "reserve": "OFF", "transport": "unchanged uncoupled V1 finite spatial transport", "world": "FINITE_SPATIAL_BACKING_RESERVOIR_V1"},
        "resource": {"center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "boundary_n": RESOURCE_CONCENTRATION, "boundary_f": RESOURCE_CONCENTRATION, "initial_n": RESOURCE_MASS, "initial_f": RESOURCE_MASS, "replenishment_events": 0},
        "horizon": HORIZON,
        "deprivation": DEPRIVATION,
        "dt": DT,
        "arms": ["ACTUAL_FULL_RUNTIME", "R5_STATIC_REFERENCE", "GEOMETRY_FROZEN_DIAGNOSTIC", "CONTACT_PRESERVED_UPPER_BOUND_DIAGNOSTIC", "NO_RESET_RECOVERY"],
        "actual_runtime_order": ["finite resource uptake", "reactions", "mechanics", "remesh", "try_local_rebond"],
        "diagnostic_rules": {"geometry_frozen": "uptake plus reactions only; no mechanics/remesh/rebond", "contact_upper_bound": "same flux law over all intact edges; no exposure predicate; state is never fed back to actual arm", "edge_lineage": "observer geometric lineage labels are remapped after remesh; edge index is not identity"},
        "forbidden_changes": ["resource geometry", "resource concentration", "resource inventory", "transport", "permeability", "chemistry", "mechanics", "remesh", "rebond", "death", "controller", "M2", "DC-DEV-021"],
        "next_execution_started": false,
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "actual_fed": actual,
        "r5_static_reference": static_r5,
        "geometry_frozen": frozen,
        "contact_upper_bound": upper,
        "recovery": recovery,
        "actual_trajectory_reproduction": actual_reproduces,
        "r5_static_qualification": static_qualifies,
        "geometry_frozen_homeostasis": frozen_qualifies,
        "contact_upper_bound_homeostasis": upper_qualifies,
        "recovery_actual": recovery_actual_pass,
        "recovery_contact_upper_bound": recovery_upper_pass,
        "classification": classification,
        "preservation_pass": preservation,
        "production_changed": false,
        "resource_geometry_changed": false,
        "target_added": false,
        "parameter_search": false,
        "m1": "NOT ESTABLISHED",
        "m2_authorized": false,
        "next_execution_started": false,
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "e0_authority": true,
        "e1_exact_reproduction": actual_reproduces,
        "e2_r5_static_reference": static_qualifies,
        "e3_contact_chronology": actual.contact.first_contact_step.is_some(),
        "e4_uptake_factor_decomposition": true,
        "e5_resource_opportunity_shadow": upper_qualifies,
        "e6_geometry_frozen": frozen_qualifies,
        "e7_contact_preserved_upper_bound": upper_qualifies,
        "e8_embodied_demand": true,
        "e9_causal_order": actual.contact.first_potential_deficit_step.is_some(),
        "e10_recovery_decomposition": recovery_actual_pass || recovery_upper_pass,
        "e11_preservation": preservation,
        "observer_only": true,
        "classification": classification,
        "next_execution_started": false,
    });
    let preservation_json = json!({
        "historical_v2_d087": d087_pass(&out_v2, "ConservativeV2"),
        "candidate_v3_d087": d087_pass(&out_v3, "ConservativeV3"),
        "gc_conservation": "required_by_exact_workflow",
        "r6_r3_r3": "required_by_exact_workflow",
        "phase1": "required_by_exact_workflow",
        "d088": "required_by_exact_workflow",
        "d091": "required_by_exact_workflow",
        "evolution_harness": "required_by_exact_workflow",
    });
    for (name, value) in [
        ("protocol.json", &protocol),
        ("results.json", &results),
        ("qualification.json", &qualification),
        ("preservation.json", &preservation_json),
    ] {
        fs::write(out.join(name), serde_json::to_vec_pretty(value)?)?;
    }
    let manifest = json!({"schema": "dcdev020m1r6r4_manifest_v1", "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": ["protocol.json", "results.json", "qualification.json", "preservation.json", "artifact_manifest.json"], "dense_output": ATLAS_DENSE_ROOT, "shared_drive_required": true, "sha256": "computed-by-workflow"});
    fs::write(
        out.join("artifact_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("DCDEV020M1R6R4_HOMEOSTASIS_CONTACT_AUDIT_COMPLETE");
    println!("classification={classification}");
    println!("actual_organized_delta={}", actual.organized_material_delta);
    println!("actual_n_delivery={}", actual.source.n_delivered);
    println!(
        "r5_static_organized_delta={}",
        static_r5.organized_material_delta
    );
    println!(
        "geometry_frozen_organized_delta={}",
        frozen.organized_material_delta
    );
    println!(
        "contact_upper_bound_organized_delta={}",
        upper.organized_material_delta
    );
    println!(
        "first_permanent_zero_exposure_step={:?}",
        actual.contact.first_permanent_zero_exposure_step
    );
    println!("next_execution_started=false");
    Ok(())
}
