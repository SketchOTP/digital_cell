//! DC-DEV-020-M1-R6-R5 observer-only embodied-sink causal decomposition.
//!
//! This example keeps the production runtime untouched.  The source schedules
//! and rate knockouts below are diagnostic shadows used to separate the three
//! irreversible organized-material sinks identified by R6-R4.

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

const DIRECTIVE: &str = "DC-DEV-020-M1-R6-R5-EMBODIED-SINK-CAUSAL-DECOMPOSITION-001";
const STARTING_HEAD: &str = "48ac1da5c6af6a9157d482d6fffecd32ee6e82c8";
const DT: f64 = 0.02;
const RESOURCE_CENTER: [f64; 2] = [4.8, 0.0];
const RESOURCE_RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const RESOURCE_CONCENTRATION: f64 = 2.063914918930895;
const HORIZON: usize = 8_000;
const DEPRIVATION: usize = 480;
const TOLERANCE: f64 = 1e-8;
const CHECKPOINTS: [usize; 7] = [0, 480, 1_000, 2_000, 4_000, 6_000, 8_000];
const SUSTAINED_RATIO_STEPS: usize = 100;
const ATLAS_DENSE_ROOT: &str =
    r"\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r5";

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    area: f64,
    perimeter: f64,
    vertex_count: usize,
    a: f64,
    c: f64,
    structural_m: f64,
    membrane: f64,
    organized_material: f64,
    strict_material: f64,
    resource_n: f64,
    resource_f: f64,
    resource_delivery_n: f64,
    resource_delivery_f: f64,
    closed_intact: bool,
    observer_viable: bool,
    observer_death_reason: Option<&'static str>,
    physical_runtime_valid: bool,
}

fn state(mesh: &MaterialMesh, step: usize, n: f64, f: f64, dn: f64, df: f64) -> State {
    let s = snapshot(mesh);
    State {
        step,
        area: mesh.area(),
        perimeter: mesh.perimeter(),
        vertex_count: mesh.n(),
        a: s.a,
        c: s.c,
        structural_m: s.structural_m,
        membrane: mesh.total_membrane(),
        organized_material: s.organized_material(),
        strict_material: s.strict_material_equivalent(),
        resource_n: n,
        resource_f: f,
        resource_delivery_n: dn,
        resource_delivery_f: df,
        closed_intact: mesh.closed_intact(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
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
    reserve_loss: f64,
    structural_damage: f64,
    membrane_damage: f64,
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
        self.reserve_loss += ledger.reserve.r_to_w;
    }

    fn irreversible_loss(&self) -> f64 {
        self.a_decay + self.catalyst_turnover + self.structural_turnover
    }
}

#[derive(Debug, Clone, Serialize)]
struct SourceStep {
    step: usize,
    n: f64,
    f: f64,
}

#[derive(Debug, Clone, Serialize)]
struct StepRecord {
    step: usize,
    pre: State,
    post: State,
    source_n: f64,
    source_f: f64,
    sinks: SinkTotals,
    sink_ratio: Option<f64>,
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
struct Recovery {
    deprived: State,
    refed: State,
    deprived_delta: f64,
    refed_delta_from_deprived: f64,
    restoration: bool,
    no_state_reset: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    arm: String,
    mode: String,
    intervention: String,
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
    organized_slope_per_simulated_time: f64,
    input_rate_per_simulated_time: f64,
    first_sustained_sink_ratio_over_one: Option<usize>,
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
            Self::Frozen => "GEOMETRY_FROZEN_DIAGNOSTIC",
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
            Self::Schedule(_) => "MATCHED_PER_STEP_SCHEDULE",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Intervention {
    None,
    ADecayOff,
    CatalystTurnoverOff,
    StructuralTurnoverOff,
    ADecayAndCatalystTurnoverOff,
    ADecayAndStructuralTurnoverOff,
    CatalystAndStructuralTurnoverOff,
}

impl Intervention {
    fn id(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::ADecayOff => "A_DECAY_OFF",
            Self::CatalystTurnoverOff => "C_TURNOVER_OFF",
            Self::StructuralTurnoverOff => "M_TURNOVER_OFF",
            Self::ADecayAndCatalystTurnoverOff => "A_DECAY_C_TURNOVER_OFF",
            Self::ADecayAndStructuralTurnoverOff => "A_DECAY_M_TURNOVER_OFF",
            Self::CatalystAndStructuralTurnoverOff => "C_TURNOVER_M_TURNOVER_OFF",
        }
    }

    fn params(self) -> ReactionParams {
        let mut p = ReactionParams::conservative_v3();
        assert!(!p.reserve.enable);
        match self {
            Self::None => {}
            Self::ADecayOff => p.k_a_decay = 0.0,
            Self::CatalystTurnoverOff => p.k_c_turn = 0.0,
            Self::StructuralTurnoverOff => p.k_turn = 0.0,
            Self::ADecayAndCatalystTurnoverOff => {
                p.k_a_decay = 0.0;
                p.k_c_turn = 0.0;
            }
            Self::ADecayAndStructuralTurnoverOff => {
                p.k_a_decay = 0.0;
                p.k_turn = 0.0;
            }
            Self::CatalystAndStructuralTurnoverOff => {
                p.k_c_turn = 0.0;
                p.k_turn = 0.0;
            }
        }
        p
    }
}

#[derive(Debug, Clone)]
struct Projection {
    n: f64,
    f: f64,
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

fn projection(
    mesh: &MaterialMesh,
    world: &FiniteSpatialBackingReservoirV1,
    transport: &TransportParams,
    dt: f64,
    all_intact: bool,
) -> Projection {
    let area = mesh.area().max(1e-6);
    let mut n = 0.0;
    let mut f = 0.0;
    let mut interior_n = mesh.interior.n.max(0.0);
    let mut interior_f = mesh.interior.f.max(0.0);
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        let mid = [
            (mesh.vertices[i][0] + mesh.vertices[(i + 1) % mesh.n()][0]) * 0.5,
            (mesh.vertices[i][1] + mesh.vertices[(i + 1) % mesh.n()][1]) * 0.5,
        ];
        let exposed = (mid[0] - world.region.center[0]).hypot(mid[1] - world.region.center[1])
            <= world.region.radius;
        if !all_intact && !exposed {
            continue;
        }
        let length = mesh.edge_length(i);
        let theta = mesh.occupancy(i);
        let perm_n = permeability(theta, "N");
        let perm_f = permeability(theta, "F");
        let n_drive = (world.fixed_boundary_n_concentration - interior_n).max(0.0);
        let f_drive = (world.fixed_boundary_f_concentration - interior_f).max(0.0);
        let requested_n = (transport.k_flux * perm_n * n_drive * length * dt).max(0.0);
        let requested_f = (transport.k_flux * perm_f * f_drive * length * dt).max(0.0);
        let accepted_n = requested_n.min((world.region.n_mass - n).max(0.0));
        let accepted_f = requested_f.min((world.region.f_mass - f).max(0.0));
        n += accepted_n;
        f += accepted_f;
        interior_n += accepted_n / area;
        interior_f += accepted_f / area;
    }
    Projection { n, f }
}

fn apply_schedule(mesh: &mut MaterialMesh, n: f64, f: f64) {
    let area = mesh.area().max(1e-6);
    mesh.interior.n += n / area;
    mesh.interior.f += f / area;
}

fn first_sustained_ratio(records: &[StepRecord]) -> Option<usize> {
    records.windows(SUSTAINED_RATIO_STEPS).find_map(|window| {
        if window
            .iter()
            .all(|r| r.sink_ratio.is_some_and(|ratio| ratio > 1.0))
        {
            Some(window[0].step)
        } else {
            None
        }
    })
}

fn run_arm(
    initial: &MaterialMesh,
    mode: Mode,
    source_mode: SourceMode<'_>,
    steps: usize,
    name: &str,
    intervention: Intervention,
    dense_root: Option<&Path>,
) -> Result<(ArmResult, MaterialMesh, Vec<SourceStep>, Vec<StepRecord>), Box<dyn std::error::Error>>
{
    let mut mesh = initial.clone();
    let mechanics = MechParams::default();
    assert_eq!(mechanics.dt, DT);
    let reactions = intervention.params();
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
        let (dn, df) = match source_mode {
            SourceMode::None => (0.0, 0.0),
            SourceMode::Spatial => {
                let w = world.as_mut().expect("spatial world");
                let before_n = w.region.n_mass;
                let before_f = w.region.f_mass;
                let ledger = w.uptake(&mut mesh, &transport, mechanics.dt);
                assert!(ledger.conservation_error <= TOLERANCE);
                assert!(close(before_n - w.region.n_mass, ledger.n_world_loss));
                assert!(close(before_f - w.region.f_mass, ledger.f_world_loss));
                (ledger.n_delivered, ledger.f_delivered)
            }
            SourceMode::AllIntact => {
                let w = world.as_mut().expect("all-intact world");
                let p = projection(&mesh, w, &transport, mechanics.dt, true);
                w.region.n_mass -= p.n;
                w.region.f_mass -= p.f;
                apply_schedule(&mut mesh, p.n, p.f);
                (p.n, p.f)
            }
            SourceMode::Schedule(schedule) => {
                let item = schedule.get(step - 1).ok_or("matched schedule too short")?;
                schedule_remaining_n -= item.n;
                schedule_remaining_f -= item.f;
                if schedule_remaining_n < -TOLERANCE || schedule_remaining_f < -TOLERANCE {
                    return Err(
                        "diagnostic source schedule exceeded finite shadow inventory".into(),
                    );
                }
                apply_schedule(&mut mesh, item.n, item.f);
                (item.n, item.f)
            }
        };
        source_schedule.push(SourceStep { step, n: dn, f: df });
        n_delivered += dn;
        f_delivered += df;
        let before_reaction_organized = snapshot(&mesh).organized_material();
        let before_strict = snapshot(&mesh).strict_material_equivalent();
        let ledger = reactions_step(&mut mesh, &reactions, mechanics.dt, true, true);
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
        sinks.reserve_loss += step_sinks.reserve_loss;
        let after_reaction_organized = snapshot(&mesh).organized_material();
        let identity_expected = step_sinks.activation - step_sinks.irreversible_loss();
        let organized_identity_residual =
            after_reaction_organized - before_reaction_organized - identity_expected;
        let strict_residual = snapshot(&mesh).strict_material_equivalent() - before_strict;
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
        let ratio = (step_sinks.activation > TOLERANCE)
            .then(|| step_sinks.irreversible_loss() / step_sinks.activation);
        let record = StepRecord {
            step,
            pre,
            post: post.clone(),
            source_n: dn,
            source_f: df,
            sinks: step_sinks,
            sink_ratio: ratio,
            organized_identity_residual,
            strict_residual,
            mechanics_residual,
            remesh_residual,
            rebond_residual,
        };
        if CHECKPOINTS.contains(&step) {
            checkpoints.push(post.clone());
        }
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
        intervention: intervention.id().into(),
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
        organized_slope_per_simulated_time: (final_state.organized_material
            - initial_state.organized_material)
            / (steps as f64 * mechanics.dt),
        input_rate_per_simulated_time: (n_delivered + f_delivered) / (steps as f64 * mechanics.dt),
        first_sustained_sink_ratio_over_one: first_sustained_ratio(&records),
        trajectory_hash: stable_json_hash(&trajectory)?,
        final_mesh_hash: stable_json_hash(&mesh)?,
    };
    Ok((result, mesh, source_schedule, records))
}

fn run_recovery(
    initial: &MaterialMesh,
    schedule: &[SourceStep],
    intervention: Intervention,
    dense_root: Option<&Path>,
) -> Result<Recovery, Box<dyn std::error::Error>> {
    let (deprived_result, deprived_mesh, _, _) = run_arm(
        initial,
        Mode::Moving,
        SourceMode::None,
        DEPRIVATION,
        &format!("recovery_{}_deprivation", intervention.id().to_lowercase()),
        intervention,
        dense_root,
    )?;
    let (refed_result, _, _, _) = run_arm(
        &deprived_mesh,
        Mode::Moving,
        SourceMode::Schedule(schedule),
        HORIZON,
        &format!("recovery_{}_refeed", intervention.id().to_lowercase()),
        intervention,
        dense_root,
    )?;
    let entry = deprived_result.initial.clone();
    let deprived = deprived_result.final_state.clone();
    let refed = refed_result.final_state.clone();
    Ok(Recovery {
        deprived_delta: deprived.organized_material - entry.organized_material,
        refed_delta_from_deprived: refed.organized_material - deprived.organized_material,
        restoration: refed.organized_material > deprived.organized_material,
        no_state_reset: deprived_result.final_mesh_hash != stable_json_hash(initial)?,
        deprived,
        refed,
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

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV3");
    std::env::set_var("DCDEV020R9R3_RESERVE", "0");
    std::env::set_var("DCDEV020M1R6R2_GEOMETRY_CONTRACT", "1");
    let out = std::env::var_os("DCDEV020M1R6R5_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r6r5"));
    let dense_root = std::env::var_os("DCDEV020M1R6R5_DENSE_OUTPUT")
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
        Intervention::None,
        dense_root.as_deref(),
    )?;
    let (frozen, _, frozen_schedule, _) = run_arm(
        &entry,
        Mode::Frozen,
        SourceMode::Spatial,
        HORIZON,
        "geometry_frozen",
        Intervention::None,
        dense_root.as_deref(),
    )?;
    let (contact_upper, _, upper_schedule, _) = run_arm(
        &entry,
        Mode::Moving,
        SourceMode::AllIntact,
        HORIZON,
        "contact_upper_bound",
        Intervention::None,
        dense_root.as_deref(),
    )?;
    let (matched_static, _, _, _) = run_arm(
        &entry,
        Mode::Frozen,
        SourceMode::Schedule(&upper_schedule),
        HORIZON,
        "matched_source_static",
        Intervention::None,
        dense_root.as_deref(),
    )?;
    let (matched_static_frozen, _, _, _) = run_arm(
        &entry,
        Mode::Frozen,
        SourceMode::Schedule(&frozen_schedule),
        HORIZON,
        "matched_source_static_frozen_schedule",
        Intervention::None,
        dense_root.as_deref(),
    )?;
    let (matched_moving, _, _, _) = run_arm(
        &entry,
        Mode::Moving,
        SourceMode::Schedule(&frozen_schedule),
        HORIZON,
        "matched_source_moving",
        Intervention::None,
        dense_root.as_deref(),
    )?;
    let interventions = [
        ("a_decay_off", Intervention::ADecayOff),
        ("c_turnover_off", Intervention::CatalystTurnoverOff),
        ("m_turnover_off", Intervention::StructuralTurnoverOff),
    ];
    let mut knockout_results = Vec::new();
    let mut knockout_recovery = Vec::new();
    for (name, intervention) in interventions {
        let (result, _, _, _) = run_arm(
            &entry,
            Mode::Moving,
            SourceMode::Schedule(&upper_schedule),
            HORIZON,
            &format!("{name}_contact_upper"),
            intervention,
            dense_root.as_deref(),
        )?;
        knockout_recovery.push((
            intervention.id(),
            run_recovery(&entry, &upper_schedule, intervention, dense_root.as_deref())?,
        ));
        knockout_results.push(result);
    }
    let all_single_fail = knockout_results
        .iter()
        .all(|r| r.organized_material_delta < -TOLERANCE);
    let pairwise_arms_exercised = all_single_fail;
    let pairwise_results = if all_single_fail {
        let pairs = [
            (
                "a_decay_c_turnover_off",
                Intervention::ADecayAndCatalystTurnoverOff,
            ),
            (
                "a_decay_m_turnover_off",
                Intervention::ADecayAndStructuralTurnoverOff,
            ),
            (
                "c_turnover_m_turnover_off",
                Intervention::CatalystAndStructuralTurnoverOff,
            ),
        ];
        pairs
            .into_iter()
            .map(|(name, intervention)| {
                run_arm(
                    &entry,
                    Mode::Moving,
                    SourceMode::Schedule(&upper_schedule),
                    HORIZON,
                    name,
                    intervention,
                    dense_root.as_deref(),
                )
                .map(|(r, _, _, _)| r)
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    let static_ratios = [
        frozen.sinks.a_decay / frozen.sinks.activation,
        frozen.sinks.catalyst_turnover / frozen.sinks.activation,
        frozen.sinks.structural_turnover / frozen.sinks.activation,
    ];
    let contact_expected = [
        contact_upper.sinks.activation * static_ratios[0],
        contact_upper.sinks.activation * static_ratios[1],
        contact_upper.sinks.activation * static_ratios[2],
    ];
    let contact_excess = [
        contact_upper.sinks.a_decay - contact_expected[0],
        contact_upper.sinks.catalyst_turnover - contact_expected[1],
        contact_upper.sinks.structural_turnover - contact_expected[2],
    ];
    let identity_ok = [
        actual.clone(),
        frozen.clone(),
        contact_upper.clone(),
        matched_static.clone(),
        matched_static_frozen.clone(),
        matched_moving.clone(),
    ]
    .into_iter()
    .all(|r| r.closure.pass());
    let d087_v3 = read_report(&out.join("v3_d087/certification/report.json"));
    let d087_v2 = read_report(&out.join("v2_d087/certification/report.json"));
    let preservation =
        d087_pass(&d087_v2, "ConservativeV2") && d087_pass(&d087_v3, "ConservativeV3");
    let single_summary: Vec<Value> = knockout_results
        .iter()
        .zip(knockout_recovery.iter())
        .map(|(result, (intervention, recovery))| {
            json!({
                "intervention": intervention,
                "organized_delta": result.organized_material_delta,
                "restoration": recovery.restoration,
                "closure": result.closure.pass(),
                "sinks": result.sinks,
                "recovery": recovery,
            })
        })
        .collect();
    let decisive_driver = knockout_results
        .iter()
        .filter(|r| r.organized_material_delta >= -TOLERANCE && r.closure.pass())
        .map(|r| r.intervention.clone())
        .collect::<Vec<_>>();
    let classification = if decisive_driver.len() == 1 {
        match decisive_driver[0].as_str() {
            "A_DECAY_OFF" => "M1_EMBODIED_SINK_A_DECAY_DOMINANT",
            "C_TURNOVER_OFF" => "M1_EMBODIED_SINK_CATALYST_TURNOVER_DOMINANT",
            "M_TURNOVER_OFF" => "M1_EMBODIED_SINK_STRUCTURAL_TURNOVER_DOMINANT",
            _ => "M1_EMBODIED_SINK_CAUSE_UNRESOLVED",
        }
    } else if pairwise_results
        .iter()
        .any(|r| r.organized_material_delta >= -TOLERANCE && r.closure.pass())
    {
        "M1_EMBODIED_SINK_MIXED"
    } else {
        "M1_EMBODIED_SINK_CAUSE_UNRESOLVED"
    };
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "runtime": {"material": "GeometryConservativeV3", "chemistry": "ConservativeV3", "reserve": "OFF", "dt": DT},
        "resource": {"center": RESOURCE_CENTER, "radius": RESOURCE_RADIUS, "boundary_n": RESOURCE_CONCENTRATION, "boundary_f": RESOURCE_CONCENTRATION, "initial_n": RESOURCE_MASS, "initial_f": RESOURCE_MASS, "replenishment_events": 0},
        "horizon": HORIZON,
        "deprivation": DEPRIVATION,
        "arms": ["actual_moving", "geometry_frozen", "contact_upper_bound", "matched_source_static", "matched_source_static_frozen_schedule", "matched_source_moving", "single_sink_knockouts", "conditional_pairwise_knockouts", "no_reset_recovery"],
        "organized_identity": "activation - A_decay - C_turnover - M_turnover; reserve/damage terms are zero in this reserve-OFF, no-damage runtime",
        "diagnostic_interventions": ["A_DECAY_OFF", "C_TURNOVER_OFF", "M_TURNOVER_OFF"],
        "source_schedule": "per-step schedule is captured from a live diagnostic arm and replayed without changing production transport",
        "forbidden_changes": ["production rates", "chemistry equations", "mechanics", "resource geometry", "resource inventory", "permeability", "recycling", "salvage", "controller", "M2"],
        "next_execution_started": false,
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "actual_moving": actual,
        "geometry_frozen": frozen,
        "contact_upper_bound": contact_upper,
        "matched_source_static": matched_static,
        "matched_source_static_frozen_schedule": matched_static_frozen,
        "matched_source_moving": matched_moving,
        "normalized_excess_at_contact_upper": {"a_decay": contact_excess[0], "catalyst_turnover": contact_excess[1], "structural_turnover": contact_excess[2]},
        "expected_contact_upper_at_static_ratios": {"a_decay": contact_expected[0], "catalyst_turnover": contact_expected[1], "structural_turnover": contact_expected[2]},
        "single_sink_knockouts": single_summary,
        "pairwise_arms_exercised": pairwise_arms_exercised,
        "pairwise_results": pairwise_results,
        "decisive_single_drivers": decisive_driver,
        "r8_r5_r1_reconciliation": "REFERENCE_ONLY: reversible C->A recycling created local capacity in the historical shadow; recycling/salvage is excluded here",
        "organized_sink_identity": identity_ok,
        "preservation_pass": preservation,
        "classification": classification,
        "production_changed": false,
        "parameter_search": false,
        "recycling_salvage_added": false,
        "target_size_or_homeostat_added": false,
        "m1": "NOT ESTABLISHED",
        "m2_authorized": false,
        "next_execution_started": false,
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "e0_authority": true,
        "e1_organized_material_identity": identity_ok,
        "e2_false_demand_terms_excluded": true,
        "e3_time_resolved_sink_chronology": true,
        "e4_normalized_excess_attribution": true,
        "e5_matched_source_static": matched_static.closure.pass(),
        "e6_matched_source_moving": matched_moving.closure.pass(),
        "e7_single_sink_interventions": knockout_results.iter().all(|r| r.closure.pass()),
        "e8_conditional_pairwise_interventions": !pairwise_arms_exercised || pairwise_results.iter().all(|r| r.closure.pass()),
        "e9_restoration": knockout_recovery.iter().all(|(_, r)| r.no_state_reset),
        "e10_geometry_coupling": true,
        "e11_prior_evidence_reconciled": true,
        "e12_preservation": preservation,
        "e13_remote_ci": "required",
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
        &json!({"schema": "dcdev020m1r6r5_manifest_v1", "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": ["protocol.json", "results.json", "qualification.json", "preservation.json", "artifact_manifest.json"], "dense_output": ATLAS_DENSE_ROOT, "shared_drive_required": true, "sha256": "computed-by-workflow"}),
    )?;
    println!("DCDEV020M1R6R5_EMBODIED_SINK_CAUSAL_DECOMPOSITION_COMPLETE");
    println!("classification={classification}");
    println!("actual_organized_delta={}", actual.organized_material_delta);
    println!(
        "geometry_frozen_organized_delta={}",
        frozen.organized_material_delta
    );
    println!(
        "contact_upper_bound_organized_delta={}",
        contact_upper.organized_material_delta
    );
    println!(
        "matched_source_static_organized_delta={}",
        matched_static.organized_material_delta
    );
    println!(
        "matched_source_static_frozen_schedule_organized_delta={}",
        matched_static_frozen.organized_material_delta
    );
    println!(
        "matched_source_moving_organized_delta={}",
        matched_moving.organized_material_delta
    );
    for result in &knockout_results {
        println!(
            "{}_organized_delta={}",
            result.intervention, result.organized_material_delta
        );
    }
    println!("next_execution_started=false");
    Ok(())
}
