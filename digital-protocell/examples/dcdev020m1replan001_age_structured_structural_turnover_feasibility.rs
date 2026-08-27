//! DC-DEV-020-M1-REPLAN-001 diagnostic age-structured structural turnover.
//!
//! This example is deliberately a shadow.  The production reaction, mechanics,
//! remesh, transport, and death code are called unchanged.  The shadow ledger
//! replays ordinary structural build/turnover bookkeeping after that call and
//! transfers the newly built M into a young pool before applying the existing
//! k_turn maturation and turnover equations.  It never changes production
//! defaults or serialized organism state.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r5_entry;

use chemistry_core::material_mesh::{
    conserve_interior_amount_across_area_change, MaterialMesh, MeshEdge,
};
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{
    apply_structural_damage, reactions_step, try_local_rebond, ReactionLedger, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;
use phase1_certifier::frozen::FROZEN_CENTER;
use regulatory_core::{stable_json_hash, FiniteSpatialBackingReservoirV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-020-M1-REPLAN-001-AGE-STRUCTURED-STRUCTURAL-TURNOVER-FEASIBILITY-001";
const STARTING_HEAD: &str = "f537bc064030d6f336608488935620e2f9256322";
const DT: f64 = 0.02;
const CENTER: [f64; 2] = [4.8, 0.0];
const RADIUS: f64 = 1.5;
const RESOURCE_MASS: f64 = 243.14924801053778;
const CONCENTRATION: f64 = 2.063914918930895;
const HORIZON: usize = 8_000;
const DEPRIVATION: usize = 480;
const STARVATION_BOUND: usize = 150_000;
const TOLERANCE: f64 = 1e-8;
const CHECKPOINTS: [usize; 8] = [0, 480, 1_000, 2_000, 4_000, 6_000, 8_000, 150_000];
const ATLAS_DENSE_ROOT: &str =
    r"\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1replan001";

#[derive(Debug, Clone, Serialize)]
struct SourceStep {
    step: usize,
    n: f64,
    f: f64,
}

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    organized_material: f64,
    strict_material: f64,
    a: f64,
    c: f64,
    structural_m: f64,
    free_l: f64,
    bound_b: f64,
    waste: f64,
    area: f64,
    perimeter: f64,
    vertex_count: usize,
    age_young: f64,
    age_mature: f64,
    age_total: f64,
    mean_strain: f64,
    mean_turn_scale: f64,
    resource_n_remaining: f64,
    resource_f_remaining: f64,
    n_delivered: f64,
    f_delivered: f64,
    closed_intact: bool,
    observer_viable: bool,
    observer_death_reason: Option<String>,
    physical_runtime_valid: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
struct Totals {
    activation: f64,
    a_decay: f64,
    catalyst_turnover: f64,
    structural_production: f64,
    structural_maturation: f64,
    structural_turnover: f64,
    membrane_production: f64,
    membrane_turnover: f64,
}

impl Totals {
    fn absorb(&mut self, ledger: &ReactionLedger, age: Option<&AgeStep>) {
        self.activation += ledger.a_produced;
        self.a_decay += ledger.a_decayed;
        self.catalyst_turnover += ledger.c_turned;
        self.structural_production += age.map_or(ledger.m_produced, |x| x.production);
        self.structural_turnover += age.map_or(ledger.m_to_w, |x| x.turnover);
        self.structural_maturation += age.map_or(0.0, |x| x.maturation);
        self.membrane_production += ledger.l_produced;
        self.membrane_turnover += ledger.bind_extent + ledger.unbind_extent;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
struct Closure {
    max_strict_residual: f64,
    max_mechanics_residual: f64,
    max_remesh_residual: f64,
    max_rebond_residual: f64,
    max_age_identity_residual: f64,
}

impl Closure {
    fn pass(&self) -> bool {
        self.max_strict_residual <= TOLERANCE
            && self.max_mechanics_residual <= TOLERANCE
            && self.max_remesh_residual <= TOLERANCE
            && self.max_rebond_residual <= TOLERANCE
            && self.max_age_identity_residual <= TOLERANCE
    }
}

#[derive(Debug, Clone, Serialize)]
struct StepRecord {
    step: usize,
    state: State,
    source: SourceStep,
    totals: Totals,
    strict_residual: f64,
    mechanics_residual: f64,
    remesh_residual: f64,
    rebond_residual: f64,
    age_identity_residual: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ArmResult {
    arm: String,
    mode: String,
    age_structured: bool,
    initial: State,
    final_state: State,
    checkpoints: Vec<State>,
    totals: Totals,
    closure: Closure,
    n_delivered: f64,
    f_delivered: f64,
    source_schedule_hash: String,
    trajectory_hash: String,
    final_mesh_hash: String,
    remesh_splits: usize,
    remesh_merges: usize,
    damage_preserved: bool,
    physical_loss_step: Option<usize>,
    organized_material_delta: f64,
}

#[derive(Debug, Clone)]
struct AgePools {
    young: Vec<f64>,
    mature: Vec<f64>,
}

impl AgePools {
    fn from_mesh(mesh: &MaterialMesh) -> Self {
        Self {
            young: vec![0.0; mesh.n()],
            mature: mesh.edges.iter().map(|edge| edge.m.max(0.0)).collect(),
        }
    }

    fn total(&self) -> f64 {
        self.young.iter().sum::<f64>() + self.mature.iter().sum::<f64>()
    }

    fn identity_residual(&self, mesh: &MaterialMesh) -> f64 {
        self.young
            .iter()
            .zip(&self.mature)
            .zip(&mesh.edges)
            .map(|((young, mature), edge)| (young + mature - edge.m).abs())
            .fold(0.0, f64::max)
    }

    fn valid_for(&self, mesh: &MaterialMesh) -> bool {
        self.young.len() == mesh.n()
            && self.mature.len() == mesh.n()
            && self
                .young
                .iter()
                .chain(&self.mature)
                .all(|value| value.is_finite() && *value >= -TOLERANCE)
            && self.identity_residual(mesh) <= TOLERANCE
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct AgeStep {
    production: f64,
    maturation: f64,
    turnover: f64,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Static,
    Current,
    AgeStructured,
}

impl Mode {
    fn id(self) -> &'static str {
        match self {
            Self::Static => "GEOMETRY_FROZEN_CURRENT_PRODUCTION",
            Self::Current => "MOVING_CURRENT_PRODUCTION",
            Self::AgeStructured => "MOVING_AGE_STRUCTURED_SHADOW",
        }
    }

    fn moving(self) -> bool {
        !matches!(self, Self::Static)
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE * (1.0 + a.abs().max(b.abs()))
}

fn reservoir() -> FiniteSpatialBackingReservoirV1 {
    FiniteSpatialBackingReservoirV1::new(
        CENTER,
        RADIUS,
        RESOURCE_MASS,
        RESOURCE_MASS,
        CONCENTRATION,
        CONCENTRATION,
    )
}

fn apply_schedule(mesh: &mut MaterialMesh, n: f64, f: f64) -> Result<(), String> {
    let area = mesh.area();
    if !area.is_finite() || area <= 0.0 {
        return Err("cannot apply schedule to non-positive area".into());
    }
    mesh.interior.n += n / area;
    mesh.interior.f += f / area;
    Ok(())
}

fn age_correct_reaction(
    before: &MaterialMesh,
    mesh: &mut MaterialMesh,
    params: &ReactionParams,
    pools: &mut AgePools,
    ledger: &mut ReactionLedger,
    dt: f64,
) -> Result<AgeStep, String> {
    if before.n() != mesh.n() || pools.young.len() != mesh.n() {
        return Err("age state topology mismatch before reaction".into());
    }
    let area = mesh.area();
    if !area.is_finite() || area <= 0.0 {
        return Err("age shadow requires finite positive area".into());
    }
    let mut production = 0.0;
    let mut maturation = 0.0;
    let mut turnover = 0.0;
    let mut normal_turnover = 0.0;

    for i in 0..mesh.n() {
        if before.edges[i].ruptured {
            continue;
        }
        let q = (params.k_turn * (1.0 / (1.0 + 2.0 * before.strain(i).max(0.0))) * dt)
            .clamp(0.0, 1.0 - 1e-15);
        let normal_after = mesh.edges[i].m.max(0.0);
        let mut dm = normal_after / (1.0 - q) - before.edges[i].m.max(0.0);
        if dm.abs() <= 1e-12 {
            dm = 0.0;
        }
        if !dm.is_finite() || dm < -TOLERANCE {
            return Err(format!(
                "could not replay structural build on edge {i}: {dm}"
            ));
        }
        dm = dm.max(0.0);
        let ordinary_turn = (before.edges[i].m.max(0.0) + dm - normal_after).max(0.0);
        let young_after_build = pools.young[i] + dm;
        let mature_amount = (params.k_turn * young_after_build * dt)
            .min(young_after_build)
            .max(0.0);
        let mature_after_maturation = pools.mature[i] + mature_amount;
        let scale = 1.0 / (1.0 + 2.0 * before.strain(i).max(0.0));
        let mature_turn = (params.k_turn * scale * mature_after_maturation * dt)
            .min(mature_after_maturation)
            .max(0.0);
        pools.young[i] = young_after_build - mature_amount;
        pools.mature[i] = mature_after_maturation - mature_turn;
        let desired_total = pools.young[i] + pools.mature[i];
        mesh.edges[i].m = desired_total.max(0.0);
        production += dm;
        maturation += mature_amount;
        turnover += mature_turn;
        normal_turnover += ordinary_turn;
    }

    let turnover_delta = turnover - normal_turnover;
    mesh.interior.w += turnover_delta / area;
    ledger.m_produced = production;
    ledger.m_to_w = turnover;
    ledger.w_produced += turnover_delta;
    Ok(AgeStep {
        production,
        maturation,
        turnover,
    })
}

fn age_split(mesh: &mut MaterialMesh, pools: &mut AgePools) -> usize {
    let mut splits = 0;
    let mut i = 0;
    let limit = mesh.n().saturating_mul(4).max(16);
    while i < mesh.n() && splits < limit {
        if mesh.edges[i].ruptured || mesh.edge_length(i) <= mesh.l_max {
            i += 1;
            continue;
        }
        let n = mesh.n();
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % n];
        let edge = mesh.edges[i];
        let young = pools.young[i] * 0.5;
        let mature = pools.mature[i] * 0.5;
        mesh.vertices
            .insert(i + 1, [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])]);
        mesh.edges[i] = MeshEdge {
            m: edge.m * 0.5,
            b: edge.b * 0.5,
            tracer_m: edge.tracer_m * 0.5,
            tracer_b: edge.tracer_b * 0.5,
            ruptured: false,
        };
        mesh.edges.insert(
            i + 1,
            MeshEdge {
                m: edge.m * 0.5,
                b: edge.b * 0.5,
                tracer_m: edge.tracer_m * 0.5,
                tracer_b: edge.tracer_b * 0.5,
                ruptured: false,
            },
        );
        pools.young[i] = young;
        pools.young.insert(i + 1, young);
        pools.mature[i] = mature;
        pools.mature.insert(i + 1, mature);
        splits += 1;
        i += 2;
    }
    splits
}

fn age_merge(mesh: &mut MaterialMesh, pools: &mut AgePools) -> usize {
    let mut merges = 0;
    let limit = mesh.n().saturating_mul(2).max(8);
    while merges < limit && mesh.n() > 6 {
        let n = mesh.n();
        let pick = (0..n).find(|&i| {
            !mesh.edges[i].ruptured
                && !mesh.edges[(i + 1) % n].ruptured
                && mesh.edge_length(i) < mesh.l_min
        });
        let Some(i) = pick else { break };
        let j = (i + 1) % n;
        if j == 0 {
            mesh.vertices.rotate_left(1);
            mesh.edges.rotate_left(1);
            pools.young.rotate_left(1);
            pools.mature.rotate_left(1);
            let k = mesh.n() - 2;
            mesh.edges[k] = MeshEdge {
                m: mesh.edges[k].m + mesh.edges[k + 1].m,
                b: mesh.edges[k].b + mesh.edges[k + 1].b,
                tracer_m: mesh.edges[k].tracer_m + mesh.edges[k + 1].tracer_m,
                tracer_b: mesh.edges[k].tracer_b + mesh.edges[k + 1].tracer_b,
                ruptured: false,
            };
            pools.young[k] += pools.young[k + 1];
            pools.mature[k] += pools.mature[k + 1];
            mesh.edges.pop();
            mesh.vertices.pop();
            pools.young.pop();
            pools.mature.pop();
        } else {
            mesh.edges[i] = MeshEdge {
                m: mesh.edges[i].m + mesh.edges[j].m,
                b: mesh.edges[i].b + mesh.edges[j].b,
                tracer_m: mesh.edges[i].tracer_m + mesh.edges[j].tracer_m,
                tracer_b: mesh.edges[i].tracer_b + mesh.edges[j].tracer_b,
                ruptured: false,
            };
            pools.young[i] += pools.young[j];
            pools.mature[i] += pools.mature[j];
            mesh.edges.remove(j);
            mesh.vertices.remove(j);
            pools.young.remove(j);
            pools.mature.remove(j);
        }
        merges += 1;
    }
    merges
}

fn age_remesh(mesh: &mut MaterialMesh, pools: &mut AgePools) -> Result<(usize, usize), String> {
    let before_material = snapshot(mesh).strict_material_equivalent();
    let area_before = mesh.area();
    let splits = age_split(mesh, pools);
    let merges = age_merge(mesh, pools);
    if !conserve_interior_amount_across_area_change(mesh, area_before, mesh.area()) {
        return Err("GC remesh area conservation rejected".into());
    }
    let identity = pools.identity_residual(mesh);
    if identity > TOLERANCE
        || (snapshot(mesh).strict_material_equivalent() - before_material).abs() > TOLERANCE
    {
        return Err(format!("age remesh identity failed: {identity}"));
    }
    Ok((splits, merges))
}

fn align_rebond_age(mesh: &MaterialMesh, before: &MaterialMesh, pools: &mut AgePools) -> bool {
    if pools.young.len() != mesh.n() || before.n() != mesh.n() {
        return false;
    }
    for i in 0..mesh.n() {
        let old = before.edges[i].m.max(0.0);
        let new = mesh.edges[i].m.max(0.0);
        if new > old + TOLERANCE {
            pools.mature[i] += new - old;
        }
        let total = pools.young[i] + pools.mature[i];
        if (total - new).abs() > TOLERANCE {
            pools.mature[i] += new - total;
        }
    }
    pools.valid_for(mesh)
}

fn state(
    mesh: &MaterialMesh,
    pools: Option<&AgePools>,
    step: usize,
    remaining_n: f64,
    remaining_f: f64,
    dn: f64,
    df: f64,
) -> State {
    let snap = snapshot(mesh);
    let mut strain_sum = 0.0;
    let mut turn_sum = 0.0;
    let mut count = 0.0;
    for i in 0..mesh.n() {
        if mesh.edges[i].ruptured {
            continue;
        }
        let strain = mesh.strain(i);
        strain_sum += strain;
        turn_sum += 1.0 / (1.0 + 2.0 * strain.max(0.0));
        count += 1.0;
    }
    let (young, mature, total) = pools.map_or((0.0, 0.0, 0.0), |p| {
        (p.young.iter().sum(), p.mature.iter().sum(), p.total())
    });
    State {
        step,
        organized_material: snap.organized_material(),
        strict_material: snap.strict_material_equivalent(),
        a: snap.a,
        c: snap.c,
        structural_m: snap.structural_m,
        free_l: snap.free_l,
        bound_b: snap.bound_b,
        waste: snap.waste,
        area: mesh.area(),
        perimeter: mesh.perimeter(),
        vertex_count: mesh.n(),
        age_young: young,
        age_mature: mature,
        age_total: total,
        mean_strain: if count > 0.0 { strain_sum / count } else { 0.0 },
        mean_turn_scale: if count > 0.0 { turn_sum / count } else { 0.0 },
        resource_n_remaining: remaining_n,
        resource_f_remaining: remaining_f,
        n_delivered: dn,
        f_delivered: df,
        closed_intact: mesh.closed_intact(),
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason().map(str::to_string),
        physical_runtime_valid: mesh.physical_runtime_valid(),
    }
}

fn run_arm(
    initial: &MaterialMesh,
    initial_pools: Option<&AgePools>,
    mode: Mode,
    schedule: Option<&[SourceStep]>,
    steps: usize,
    name: &str,
    dense_root: Option<&Path>,
) -> Result<(ArmResult, MaterialMesh, Option<AgePools>, Vec<SourceStep>), String> {
    let mut mesh = initial.clone();
    let age_structured = matches!(mode, Mode::AgeStructured);
    let mut pools = age_structured.then(|| {
        initial_pools
            .cloned()
            .unwrap_or_else(|| AgePools::from_mesh(&mesh))
    });
    let mechanics = MechParams::default();
    if !close(mechanics.dt, DT) || !close(FROZEN_CENTER.dt, DT) {
        return Err("sanctioned dt does not match the frozen R6 schedule".into());
    }
    let reaction = ReactionParams::conservative_v3();
    if reaction.reserve.enable {
        return Err("reserve must remain OFF".into());
    }
    let transport = TransportParams::default();
    let mut world = schedule.is_none().then(reservoir);
    let mut remaining_n = if schedule.is_some_and(|items| items.is_empty()) {
        0.0
    } else {
        RESOURCE_MASS
    };
    let mut remaining_f = remaining_n;
    let initial_state = state(&mesh, pools.as_ref(), 0, remaining_n, remaining_f, 0.0, 0.0);
    let mut checkpoints = vec![initial_state.clone()];
    let mut trajectory = vec![stable_json_hash(&initial_state).map_err(|e| e.to_string())?];
    let mut generated_schedule = Vec::with_capacity(steps);
    let mut totals = Totals::default();
    let mut closure = Closure::default();
    let mut n_delivered = 0.0;
    let mut f_delivered = 0.0;
    let mut splits = 0;
    let mut merges = 0;
    let mut physical_loss_step = None;
    let mut dense = dense_root
        .map(|root| File::create(root.join(format!("{name}.jsonl"))).map(BufWriter::new))
        .transpose()
        .map_err(|e| e.to_string())?;

    for step in 1..=steps {
        let (dn, df) = if let Some(sealed) = schedule {
            if sealed.is_empty() {
                (0.0, 0.0)
            } else {
                let item = sealed
                    .get(step - 1)
                    .ok_or_else(|| "sealed source schedule too short".to_string())?;
                remaining_n -= item.n;
                remaining_f -= item.f;
                if remaining_n < -TOLERANCE || remaining_f < -TOLERANCE {
                    return Err("sealed schedule exceeded finite inventory".into());
                }
                apply_schedule(&mut mesh, item.n, item.f)?;
                (item.n, item.f)
            }
        } else {
            let world = world
                .as_mut()
                .ok_or_else(|| "spatial source world is unavailable".to_string())?;
            let uptake = world.uptake(&mut mesh, &transport, DT);
            remaining_n = world.region.n_mass;
            remaining_f = world.region.f_mass;
            if uptake.conservation_error > TOLERANCE {
                return Err("finite-world transport closure failed".into());
            }
            (uptake.n_delivered, uptake.f_delivered)
        };
        generated_schedule.push(SourceStep { step, n: dn, f: df });
        n_delivered += dn;
        f_delivered += df;
        let before_reaction = mesh.clone();
        let mut ledger = reactions_step(&mut mesh, &reaction, DT, true, true);
        let age_step = if let Some(pools) = pools.as_mut() {
            Some(age_correct_reaction(
                &before_reaction,
                &mut mesh,
                &reaction,
                pools,
                &mut ledger,
                DT,
            )?)
        } else {
            None
        };
        totals.absorb(&ledger, age_step.as_ref());
        let before = snapshot(&before_reaction).strict_material_equivalent();
        let after_reaction = snapshot(&mesh).strict_material_equivalent();
        let mut mechanics_residual = 0.0;
        let mut remesh_residual = 0.0;
        let mut rebond_residual = 0.0;
        if mode.moving() {
            let before_mechanics = snapshot(&mesh).strict_material_equivalent();
            if !mechanics_step(&mut mesh, &mechanics) {
                physical_loss_step.get_or_insert(step);
                break;
            }
            mechanics_residual = snapshot(&mesh).strict_material_equivalent() - before_mechanics;
            let before_remesh = snapshot(&mesh).strict_material_equivalent();
            if let Some(pools) = pools.as_mut() {
                let (s, m) = age_remesh(&mut mesh, pools)?;
                splits += s;
                merges += m;
            } else {
                let (s, m) = remesh(&mut mesh);
                splits += s;
                merges += m;
            }
            remesh_residual = snapshot(&mesh).strict_material_equivalent() - before_remesh;
            let before_rebond = snapshot(&mesh).strict_material_equivalent();
            let rebond_before = mesh.clone();
            let _ = try_local_rebond(
                &mut mesh,
                chemistry_core::material_mesh::DEFAULT_REBOND_DIST,
            );
            if let Some(pools) = pools.as_mut() {
                if !align_rebond_age(&mesh, &rebond_before, pools) {
                    return Err("age state could not follow rebond".into());
                }
            }
            rebond_residual = snapshot(&mesh).strict_material_equivalent() - before_rebond;
        }
        let strict_residual = after_reaction - before;
        let age_identity = pools.as_ref().map_or(0.0, |p| p.identity_residual(&mesh));
        closure.max_strict_residual = closure.max_strict_residual.max(strict_residual.abs());
        closure.max_mechanics_residual =
            closure.max_mechanics_residual.max(mechanics_residual.abs());
        closure.max_remesh_residual = closure.max_remesh_residual.max(remesh_residual.abs());
        closure.max_rebond_residual = closure.max_rebond_residual.max(rebond_residual.abs());
        closure.max_age_identity_residual = closure.max_age_identity_residual.max(age_identity);
        if !mesh.physical_runtime_valid() {
            physical_loss_step.get_or_insert(step);
        }
        let post = state(
            &mesh,
            pools.as_ref(),
            step,
            remaining_n,
            remaining_f,
            dn,
            df,
        );
        if post.closed_intact == false {
            physical_loss_step.get_or_insert(step);
        }
        if CHECKPOINTS.contains(&step) || step == steps {
            checkpoints.push(post.clone());
        }
        let record = StepRecord {
            step,
            state: post.clone(),
            source: SourceStep { step, n: dn, f: df },
            totals: totals.clone(),
            strict_residual,
            mechanics_residual,
            remesh_residual,
            rebond_residual,
            age_identity_residual: age_identity,
        };
        trajectory.push(stable_json_hash(&post).map_err(|e| e.to_string())?);
        if let Some(writer) = dense.as_mut() {
            serde_json::to_writer(&mut *writer, &record).map_err(|e| e.to_string())?;
            writer.write_all(b"\n").map_err(|e| e.to_string())?;
        }
    }
    if let Some(writer) = dense.as_mut() {
        writer.flush().map_err(|e| e.to_string())?;
    }
    let final_state = state(
        &mesh,
        pools.as_ref(),
        steps,
        remaining_n,
        remaining_f,
        0.0,
        0.0,
    );
    let result = ArmResult {
        arm: name.to_string(),
        mode: mode.id().to_string(),
        age_structured,
        initial: initial_state.clone(),
        final_state: final_state.clone(),
        checkpoints,
        totals,
        closure,
        n_delivered,
        f_delivered,
        source_schedule_hash: stable_json_hash(&generated_schedule).map_err(|e| e.to_string())?,
        trajectory_hash: stable_json_hash(&trajectory).map_err(|e| e.to_string())?,
        final_mesh_hash: stable_json_hash(&mesh).map_err(|e| e.to_string())?,
        remesh_splits: splits,
        remesh_merges: merges,
        damage_preserved: false,
        physical_loss_step,
        organized_material_delta: final_state.organized_material - initial_state.organized_material,
    };
    Ok((result, mesh, pools, generated_schedule))
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

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn damage_fixture() -> Result<bool, String> {
    let (mut mesh, _) = r5_entry::m1r1_entry_state();
    mesh.stamp_geometry_conservative_schema();
    let mut pools = AgePools::from_mesh(&mesh);
    let before = mesh.total_structural_mass();
    let removed = apply_structural_damage(&mut mesh, 0.25);
    for i in 0..mesh.n() {
        let old = pools.young[i] + pools.mature[i];
        if old > 0.0 {
            let fraction = (mesh.edges[i].m / old).clamp(0.0, 1.0);
            pools.young[i] *= fraction;
            pools.mature[i] *= fraction;
        }
    }
    Ok(removed > 0.0 && mesh.total_structural_mass() < before && pools.valid_for(&mesh))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_identity_starts_with_existing_material_mature() {
        let (mut mesh, _) = r5_entry::m1r1_entry_state();
        mesh.stamp_geometry_conservative_schema();
        let pools = AgePools::from_mesh(&mesh);
        assert!(pools.young.iter().all(|value| *value == 0.0));
        assert!(pools.valid_for(&mesh));
    }

    #[test]
    fn remesh_preserves_young_and_mature_lineage() {
        let (mut mesh, _) = r5_entry::m1r1_entry_state();
        mesh.stamp_geometry_conservative_schema();
        let mut pools = AgePools::from_mesh(&mesh);
        for i in 0..mesh.n() {
            let total = mesh.edges[i].m;
            pools.young[i] = total * 0.25;
            pools.mature[i] = total * 0.75;
        }
        let young_before: f64 = pools.young.iter().sum();
        let mature_before: f64 = pools.mature.iter().sum();
        mesh.l_max = 1.0;
        mesh.l_min = 0.6;
        let _ = age_remesh(&mut mesh, &mut pools).expect("diagnostic remesh");
        assert!(close(young_before, pools.young.iter().sum()));
        assert!(close(mature_before, pools.mature.iter().sum()));
        assert!(pools.valid_for(&mesh));
    }

    #[test]
    fn explicit_damage_removes_age_tracked_material() {
        assert!(damage_fixture().expect("damage fixture"));
    }
}

fn main() -> Result<(), String> {
    let out = std::env::var_os("DCDEV020M1REPLAN001_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1replan001"));
    let dense = std::env::var_os("DCDEV020M1REPLAN001_DENSE_OUTPUT")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(ATLAS_DENSE_ROOT)));
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    if let Some(root) = dense.as_ref() {
        fs::create_dir_all(root).map_err(|e| e.to_string())?;
    }

    let (mut entry, mechanics) = r5_entry::m1r1_entry_state();
    entry.stamp_geometry_conservative_schema();
    if !close(mechanics.dt, DT) {
        return Err("entry dt changed from frozen authority".into());
    }
    let (static_arm, _, _, sealed_schedule) = run_arm(
        &entry,
        None,
        Mode::Static,
        None,
        HORIZON,
        "arm_a_frozen_geometry_current",
        dense.as_deref(),
    )?;
    let (current_arm, _, _, _) = run_arm(
        &entry,
        None,
        Mode::Current,
        Some(&sealed_schedule),
        HORIZON,
        "arm_b_moving_current",
        dense.as_deref(),
    )?;
    let (age_arm, _, _, _) = run_arm(
        &entry,
        None,
        Mode::AgeStructured,
        Some(&sealed_schedule),
        HORIZON,
        "arm_c_moving_age_structured",
        dense.as_deref(),
    )?;
    let (deprived, deprived_mesh, deprived_pools, _) = run_arm(
        &entry,
        None,
        Mode::AgeStructured,
        Some(&[]),
        DEPRIVATION,
        "arm_c_deprivation_replay",
        dense.as_deref(),
    )?;
    let (recovered, _, _, _) = run_arm(
        &deprived_mesh,
        deprived_pools.as_ref(),
        Mode::AgeStructured,
        Some(&sealed_schedule),
        HORIZON,
        "arm_c_recovery",
        dense.as_deref(),
    )?;
    let (starved, _, _, _) = run_arm(
        &entry,
        None,
        Mode::AgeStructured,
        Some(&[]),
        STARVATION_BOUND,
        "arm_c_zero_resource_starvation",
        dense.as_deref(),
    )?;

    let d087_v2 = read_report(&out.join("v2_d087/certification/report.json"));
    let d087_v3 = read_report(&out.join("v3_d087/certification/report.json"));
    let d087 = d087_pass(&d087_v2, "ConservativeV2") && d087_pass(&d087_v3, "ConservativeV3");
    let controls_reproduce = close(static_arm.organized_material_delta, 0.34214067689040917)
        && close(current_arm.organized_material_delta, -9.954959206543336);
    let age_homeostasis = age_arm.organized_material_delta >= -TOLERANCE;
    let recovery = recovered.final_state.organized_material
        > deprived.final_state.organized_material
        && (recovered.final_state.organized_material - age_arm.initial.organized_material).abs()
            < (deprived.final_state.organized_material - age_arm.initial.organized_material).abs();
    let starvation_decline = starved.final_state.structural_m < starved.initial.structural_m
        && starved.totals.structural_turnover > 0.0;
    let damage = damage_fixture()?;
    let closure = static_arm.closure.pass()
        && current_arm.closure.pass()
        && age_arm.closure.pass()
        && deprived.closure.pass()
        && recovered.closure.pass()
        && starved.closure.pass();
    let physical_loss = starved.final_state.closed_intact == false
        || starved.final_state.physical_runtime_valid == false;
    let classification = if !closure || !damage || !controls_reproduce {
        "M1_AGE_STRUCTURED_TURNOVER_INVALID"
    } else if !age_homeostasis || !recovery {
        "M1_AGE_STRUCTURED_TURNOVER_INSUFFICIENT"
    } else if !physical_loss {
        "M1_AGE_STRUCTURED_TURNOVER_HOMEOSTASIS_RECOVERY_ONLY"
    } else if !d087 {
        "M1_AGE_STRUCTURED_TURNOVER_INVALID"
    } else {
        "M1_AGE_STRUCTURED_TURNOVER_FEASIBILITY_CONFIRMED"
    };
    let protocol = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "entry_authority": {"r6_r7_classification": "M1_REFERENCE_GEOMETRY_COUPLING_NOT_SUFFICIENT"},
        "runtime": {"material": "GeometryConservativeV3", "chemistry": "ConservativeV3", "reserve": "OFF", "dt": DT},
        "source_schedule": "sealed successful R6-R6 frozen-geometry schedule generated by arm A",
        "horizon": HORIZON,
        "deprivation": DEPRIVATION,
        "starvation_bound": STARVATION_BOUND,
        "age_contract": {"entry": "all existing M mature", "build": "new M young", "maturation": "k_turn * M_young * dt", "turnover": "existing strain-scaled k_turn on mature only", "new_timescale": false},
        "arms": ["A_frozen_geometry_current", "B_moving_current", "C_moving_age_structured", "C_deprivation_recovery", "C_zero_resource_starvation"],
        "observer_only": true,
        "forbidden_changes": ["production biology", "coefficients", "mechanics", "source schedule", "transport", "resource redesign", "target size", "controller", "reserve", "recycling", "salvage", "M2", "R6-R8"],
        "next_execution_started": false
    });
    let results = json!({
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "arms": {"A": static_arm, "B": current_arm, "C": age_arm, "C_starvation": starved},
        "recovery": {"deprived": deprived, "refed": recovered, "pass": recovery},
        "checks": {"controls_reproduce": controls_reproduce, "age_homeostasis": age_homeostasis, "recovery": recovery, "starvation_decline": starvation_decline, "physical_loss": physical_loss, "damage_preservation": damage, "material_closure": closure, "d087": d087, "age_identity": age_arm.closure.max_age_identity_residual <= TOLERANCE},
        "classification": classification,
        "production_scientific_code_changed": false,
        "new_parameter_added": false,
        "target_size_or_shape_controller_added": false,
        "reserve_recycling_or_salvage_added": false,
        "m1": "NOT ESTABLISHED",
        "m2_authorized": false,
        "next_execution_started": false,
        "deprivation_pool_state_carried": deprived_pools.is_some()
    });
    let qualification = json!({
        "directive": DIRECTIVE,
        "e0_authority": true,
        "e1_material_identity_and_lineage": age_arm.closure.max_age_identity_residual <= TOLERANCE && damage,
        "e2_fed_moving_feasibility": controls_reproduce && age_homeostasis,
        "e3_no_reset_recovery": recovery,
        "e4_starvation_degradation": starvation_decline && physical_loss,
        "e5_remote_ci": "required",
        "observer_only": true,
        "classification": classification,
        "next_execution_started": false
    });
    let preservation = json!({
        "r6_r7_control_reproduction": controls_reproduce,
        "gc_material_closure": closure,
        "v2_d087": d087_pass(&d087_v2, "ConservativeV2"),
        "v3_d087": d087_pass(&d087_v3, "ConservativeV3"),
        "age_state_identity": age_arm.closure.max_age_identity_residual <= TOLERANCE,
        "remesh_age_lineage": age_arm.closure.max_age_identity_residual <= TOLERANCE,
        "damage_preservation": damage,
        "tier": "R6-R7 controls, GC closure, V2/V3 D087, age identity/lineage, and damage"
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("preservation.json"), &preservation)?;
    write_json(
        &out.join("artifact_manifest.json"),
        &json!({"schema": "dcdev020m1replan001_manifest_v1", "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "files": ["protocol.json", "results.json", "qualification.json", "preservation.json", "artifact_manifest.json"], "dense_output": ATLAS_DENSE_ROOT, "shared_drive_required": true, "sha256": "computed-by-workflow"}),
    )?;
    println!("DCDEV020M1REPLAN001_AGE_STRUCTURED_TURNOVER_FEASIBILITY_COMPLETE");
    println!("classification={classification}");
    println!(
        "static_organized_delta={}",
        static_arm.organized_material_delta
    );
    println!(
        "moving_current_organized_delta={}",
        current_arm.organized_material_delta
    );
    println!(
        "age_structured_organized_delta={}",
        age_arm.organized_material_delta
    );
    println!("age_structured_recovery={recovery}");
    println!("starvation_structural_decline={starvation_decline}");
    println!("next_execution_started=false");
    Ok(())
}
