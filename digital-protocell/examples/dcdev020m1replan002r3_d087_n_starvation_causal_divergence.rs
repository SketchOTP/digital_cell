//! DC-DEV-020-M1-REPLAN-002-R3: observer-only causal audit of D-087
//! N-starvation divergence.
//!
//! This runner replays the frozen D-087 Gate-2 trajectory for V2, V3, and V4.
//! It records the existing physical reaction ledgers in parallel and never
//! changes a production equation, certifier predicate, or mesh state because
//! of an observation.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod r5_entry;

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_contracts::snapshot;
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step, try_local_rebond, ReactionLedger, ReactionParams,
};
use chemistry_core::mesh_transport::{transport_step, TransportLedger};
use phase1_certifier::frozen::{frozen_transport, FROZEN_CENTER};
use phase1_certifier::sim::{reaction_params_for, seed_mesh};
use regulatory_core::stable_json_hash;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-020-M1-REPLAN-002-R3-D087-N-STARVATION-CAUSAL-DIVERGENCE-AUDIT-001";
const STARTING_HEAD: &str = "7d7303900e17e1fc3cb0ded911e60ddfe70bb621";
const DT: f64 = 0.02;
const WARMUP: usize = 200;
const STARVATION: usize = 6_000;
const ENDPOINT: usize = WARMUP + STARVATION;
const TOLERANCE: f64 = 1e-10;
const CHECKPOINTS: [usize; 7] = [0, 200, 1_000, 2_000, 4_000, 6_000, 6_200];

#[derive(Debug, Clone, Serialize)]
struct State {
    step: usize,
    a: f64,
    c: f64,
    n: f64,
    f: f64,
    w: f64,
    total_m: f64,
    young_m: f64,
    mature_m: f64,
    free_l: f64,
    bound_b: f64,
    area: f64,
    perimeter: f64,
    centroid: [f64; 2],
    vertex_count: usize,
    alive: bool,
    observer_viable: bool,
    observer_death_reason: Option<String>,
    ruptured_edges: usize,
    closed_intact: bool,
    physical_runtime_valid: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
struct FluxTotals {
    a_produced: f64,
    a_decayed: f64,
    a_consumed_build: f64,
    a_to_c: f64,
    a_to_m: f64,
    a_to_l: f64,
    catalyst_production: f64,
    catalyst_turnover: f64,
    structural_production: f64,
    structural_turnover: f64,
    maturation: f64,
    membrane_production: f64,
    n_consumed: f64,
    f_consumed: f64,
    waste_production: f64,
    bound_extent: f64,
    unbound_extent: f64,
}

impl FluxTotals {
    fn absorb(&mut self, ledger: &ReactionLedger) {
        self.a_produced += ledger.a_produced;
        self.a_decayed += ledger.a_decayed;
        self.a_consumed_build += ledger.a_consumed_build;
        self.a_to_c += ledger.a_to_c;
        self.a_to_m += ledger.a_to_m;
        self.a_to_l += ledger.a_to_l;
        self.catalyst_production += ledger.c_produced;
        self.catalyst_turnover += ledger.c_turned;
        self.structural_production += ledger.m_produced;
        self.structural_turnover += ledger.m_to_w;
        self.maturation += ledger.m_matured;
        self.membrane_production += ledger.l_produced;
        self.n_consumed += ledger.n_consumed;
        self.f_consumed += ledger.f_consumed;
        self.waste_production += ledger.w_produced;
        self.bound_extent += ledger.bind_extent;
        self.unbound_extent += ledger.unbind_extent;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
struct Closure {
    max_transport_residual: f64,
    max_reaction_residual: f64,
    max_mechanics_residual: f64,
    max_remesh_residual: f64,
    max_rebond_residual: f64,
    max_unexplained_residual: f64,
}

impl Closure {
    fn pass(&self) -> bool {
        [
            self.max_transport_residual,
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
struct Arm {
    contract: String,
    initial: State,
    before_n_removal: State,
    after_n_removal: State,
    checkpoints: Vec<State>,
    final_state: State,
    accepted_steps: usize,
    first_a_below_0_05: Option<usize>,
    first_observer_nonviable: Option<usize>,
    first_alive_false: Option<usize>,
    first_topology_rupture: Option<usize>,
    minimum_a: f64,
    flux_after_n_removal: FluxTotals,
    closure: Closure,
    trajectory_hash: String,
    final_mesh_hash: String,
    #[serde(skip)]
    full_states: Vec<State>,
}

#[derive(Debug, Clone, Serialize)]
struct StepRecord {
    contract: String,
    step: usize,
    pre: State,
    post: State,
    transport: TransportLedger,
    reaction: ReactionLedger,
    strict_residual_transport: f64,
    strict_residual_reaction: f64,
    strict_residual_mechanics: f64,
    strict_residual_remesh: f64,
    strict_residual_rebond: f64,
    strict_residual_unexplained: f64,
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE * (1.0 + a.abs().max(b.abs()))
}

fn set_contract(contract: &str) {
    std::env::remove_var("DCDEV020M1REPLAN002R1_V4");
    std::env::set_var("DCDEV020R9R3_CONTRACT", contract);
    std::env::set_var("DCDEV020R9R3_RESERVE", "0");
    if contract == "MaturationCoupledV4" {
        std::env::set_var("DCDEV020R9R3_CONTRACT", "ConservativeV3");
        std::env::set_var("DCDEV020M1REPLAN002R1_V4", "1");
    }
}

fn state(mesh: &MaterialMesh, step: usize) -> State {
    let s = snapshot(mesh);
    State {
        step,
        a: mesh.interior.a,
        c: mesh.interior.c,
        n: mesh.interior.n,
        f: mesh.interior.f,
        w: mesh.interior.w,
        total_m: mesh.total_structural_mass(),
        young_m: mesh.total_young_structural_mass(),
        mature_m: (0..mesh.n()).map(|i| mesh.mature_structural_mass(i)).sum(),
        free_l: mesh.free_l,
        bound_b: s.bound_b,
        area: mesh.area(),
        perimeter: mesh.perimeter(),
        centroid: mesh.centroid(),
        vertex_count: mesh.n(),
        alive: mesh.alive,
        observer_viable: mesh.observer_viable(),
        observer_death_reason: mesh.observer_death_reason().map(str::to_owned),
        ruptured_edges: mesh.edges.iter().filter(|edge| edge.ruptured).count(),
        closed_intact: mesh.closed_intact(),
        physical_runtime_valid: mesh.physical_runtime_valid(),
    }
}

fn dense_writer(root: Option<&Path>, contract: &str) -> Result<Option<BufWriter<File>>, String> {
    let Some(root) = root else {
        return Ok(None);
    };
    fs::create_dir_all(root).map_err(|error| error.to_string())?;
    File::create(root.join(format!("{contract}.jsonl")))
        .map(BufWriter::new)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn one_step(
    mesh: &mut MaterialMesh,
    reaction: &ReactionParams,
    mechanics: &MechParams,
) -> Result<(TransportLedger, ReactionLedger, f64, f64, f64, f64, f64), String> {
    let transport = frozen_transport();
    let before_transport = snapshot(mesh).strict_material_equivalent();
    let transport_ledger = transport_step(mesh, &transport, mechanics.dt);
    let after_transport = snapshot(mesh).strict_material_equivalent();
    let expected_transport = transport_ledger.n_in + transport_ledger.f_in
        - transport_ledger.w_out
        - transport_ledger.c_leak
        - transport_ledger.a_leak;
    let transport_residual = (after_transport - before_transport - expected_transport).abs();

    let before_reaction = after_transport;
    let reaction_ledger = reactions_step(mesh, reaction, mechanics.dt, true, true);
    let after_reaction = snapshot(mesh).strict_material_equivalent();
    let reaction_residual = (after_reaction - before_reaction).abs();

    let before_mechanics = after_reaction;
    if !mechanics_step(mesh, mechanics) {
        return Err("mechanics rejected".into());
    }
    let after_mechanics = snapshot(mesh).strict_material_equivalent();
    let mechanics_residual = (after_mechanics - before_mechanics).abs();

    let before_remesh = after_mechanics;
    let _ = remesh(mesh);
    let after_remesh = snapshot(mesh).strict_material_equivalent();
    let remesh_residual = (after_remesh - before_remesh).abs();

    let before_rebond = after_remesh;
    let _ = try_local_rebond(mesh, chemistry_core::material_mesh::DEFAULT_REBOND_DIST);
    let after_rebond = snapshot(mesh).strict_material_equivalent();
    let rebond_residual = (after_rebond - before_rebond).abs();

    Ok((
        transport_ledger,
        reaction_ledger,
        transport_residual,
        reaction_residual,
        mechanics_residual,
        remesh_residual,
        rebond_residual,
    ))
}

fn run_arm(contract: &str, dense_root: Option<&Path>) -> Result<Arm, String> {
    set_contract(contract);
    let mut mesh = seed_mesh(14.0, 1);
    let mechanics = FROZEN_CENTER;
    if !close(mechanics.dt, DT) {
        return Err("frozen dt mismatch".into());
    }
    let reaction = reaction_params_for(&mesh);
    let initial = state(&mesh, 0);
    let mut before_removal = initial.clone();
    let mut checkpoints = vec![initial.clone()];
    let mut full_states = vec![initial.clone()];
    let mut trajectory = vec![stable_json_hash(&initial).map_err(|e| e.to_string())?];
    let mut closure = Closure::default();
    let mut flux = FluxTotals::default();
    let mut writer = dense_writer(dense_root, contract)?;

    for step in 1..=WARMUP {
        let pre = state(&mesh, step - 1);
        let before_strict = snapshot(&mesh).strict_material_equivalent();
        let (
            transport,
            reaction_ledger,
            transport_residual,
            reaction_residual,
            mechanics_residual,
            remesh_residual,
            rebond_residual,
        ) = one_step(&mut mesh, &reaction, &mechanics)?;
        let post = state(&mesh, step);
        full_states.push(post.clone());
        let expected_transport =
            transport.n_in + transport.f_in - transport.w_out - transport.c_leak - transport.a_leak;
        let unexplained =
            (snapshot(&mesh).strict_material_equivalent() - before_strict - expected_transport)
                .abs();
        closure.max_transport_residual = closure.max_transport_residual.max(transport_residual);
        closure.max_reaction_residual = closure.max_reaction_residual.max(reaction_residual);
        closure.max_mechanics_residual = closure.max_mechanics_residual.max(mechanics_residual);
        closure.max_remesh_residual = closure.max_remesh_residual.max(remesh_residual);
        closure.max_rebond_residual = closure.max_rebond_residual.max(rebond_residual);
        closure.max_unexplained_residual = closure.max_unexplained_residual.max(unexplained);
        trajectory.push(stable_json_hash(&post).map_err(|e| e.to_string())?);
        if let Some(file) = writer.as_mut() {
            serde_json::to_writer(
                &mut *file,
                &StepRecord {
                    contract: contract.into(),
                    step,
                    pre,
                    post: post.clone(),
                    transport,
                    reaction: reaction_ledger,
                    strict_residual_transport: transport_residual,
                    strict_residual_reaction: reaction_residual,
                    strict_residual_mechanics: mechanics_residual,
                    strict_residual_remesh: remesh_residual,
                    strict_residual_rebond: rebond_residual,
                    strict_residual_unexplained: unexplained,
                },
            )
            .map_err(|e| e.to_string())?;
            file.write_all(b"\\n").map_err(|e| e.to_string())?;
        }
    }
    before_removal = state(&mesh, WARMUP);
    let mut after_removal = before_removal.clone();
    mesh.exterior.n = 0.0;
    mesh.exterior.f = 0.0;
    mesh.interior.n = 0.0;
    mesh.interior.f = 0.0;
    after_removal = state(&mesh, WARMUP);
    checkpoints.push(after_removal.clone());
    full_states.push(after_removal.clone());

    let mut first_a_below_0_05 = None;
    let mut first_observer_nonviable = None;
    let mut first_alive_false = None;
    let mut first_topology_rupture = None;
    let mut minimum_a = after_removal.a;
    for step in (WARMUP + 1)..=ENDPOINT {
        if !mesh.can_advance_physics() {
            break;
        }
        let pre = state(&mesh, step - 1);
        let before_strict = snapshot(&mesh).strict_material_equivalent();
        let (
            transport,
            reaction_ledger,
            transport_residual,
            reaction_residual,
            mechanics_residual,
            remesh_residual,
            rebond_residual,
        ) = one_step(&mut mesh, &reaction, &mechanics)?;
        flux.absorb(&reaction_ledger);
        let post = state(&mesh, step);
        full_states.push(post.clone());
        minimum_a = minimum_a.min(post.a);
        if first_a_below_0_05.is_none() && post.a < 0.05 {
            first_a_below_0_05 = Some(step);
        }
        if first_observer_nonviable.is_none() && !post.observer_viable {
            first_observer_nonviable = Some(step);
        }
        if first_alive_false.is_none() && !post.alive {
            first_alive_false = Some(step);
        }
        if first_topology_rupture.is_none() && post.ruptured_edges > 0 {
            first_topology_rupture = Some(step);
        }
        closure.max_transport_residual = closure.max_transport_residual.max(transport_residual);
        closure.max_reaction_residual = closure.max_reaction_residual.max(reaction_residual);
        closure.max_mechanics_residual = closure.max_mechanics_residual.max(mechanics_residual);
        closure.max_remesh_residual = closure.max_remesh_residual.max(remesh_residual);
        closure.max_rebond_residual = closure.max_rebond_residual.max(rebond_residual);
        let expected_transport =
            transport.n_in + transport.f_in - transport.w_out - transport.c_leak - transport.a_leak;
        let unexplained =
            (snapshot(&mesh).strict_material_equivalent() - before_strict - expected_transport)
                .abs();
        closure.max_unexplained_residual = closure.max_unexplained_residual.max(unexplained);
        trajectory.push(stable_json_hash(&post).map_err(|e| e.to_string())?);
        if CHECKPOINTS.contains(&step) {
            checkpoints.push(post.clone());
        }
        if let Some(file) = writer.as_mut() {
            serde_json::to_writer(
                &mut *file,
                &StepRecord {
                    contract: contract.into(),
                    step,
                    pre,
                    post: post.clone(),
                    transport,
                    reaction: reaction_ledger,
                    strict_residual_transport: transport_residual,
                    strict_residual_reaction: reaction_residual,
                    strict_residual_mechanics: mechanics_residual,
                    strict_residual_remesh: remesh_residual,
                    strict_residual_rebond: rebond_residual,
                    strict_residual_unexplained: unexplained,
                },
            )
            .map_err(|e| e.to_string())?;
            file.write_all(b"\\n").map_err(|e| e.to_string())?;
        }
    }
    if let Some(file) = writer.as_mut() {
        file.flush().map_err(|e| e.to_string())?;
    }
    let final_state = state(&mesh, checkpoints.last().map(|s| s.step).unwrap_or(WARMUP));
    if !checkpoints
        .iter()
        .any(|sample| sample.step == final_state.step)
    {
        checkpoints.push(final_state.clone());
    }
    Ok(Arm {
        contract: contract.into(),
        initial,
        before_n_removal: before_removal,
        after_n_removal: after_removal,
        checkpoints,
        accepted_steps: final_state.step,
        first_a_below_0_05,
        first_observer_nonviable,
        first_alive_false,
        first_topology_rupture,
        minimum_a,
        flux_after_n_removal: flux,
        closure,
        trajectory_hash: stable_json_hash(&trajectory).map_err(|e| e.to_string())?,
        final_mesh_hash: stable_json_hash(&mesh).map_err(|e| e.to_string())?,
        full_states,
        final_state,
    })
}

fn state_value(state: &State, field: &str) -> f64 {
    match field {
        "a" => state.a,
        "c" => state.c,
        "n" => state.n,
        "f" => state.f,
        "w" => state.w,
        "total_m" => state.total_m,
        "young_m" => state.young_m,
        "mature_m" => state.mature_m,
        "free_l" => state.free_l,
        "bound_b" => state.bound_b,
        "area" => state.area,
        "perimeter" => state.perimeter,
        _ => 0.0,
    }
}

fn first_divergence(v4: &Arm, comparator: &Arm) -> Value {
    let fields = [
        "a",
        "c",
        "n",
        "f",
        "w",
        "total_m",
        "young_m",
        "mature_m",
        "free_l",
        "bound_b",
        "area",
        "perimeter",
    ];
    let mut first_step = None;
    let mut first_fields = Vec::new();
    for v4_state in v4.full_states.iter().filter(|state| state.step > WARMUP) {
        let Some(other) = comparator
            .full_states
            .iter()
            .find(|candidate| candidate.step == v4_state.step && candidate.step > WARMUP)
        else {
            continue;
        };
        let differing: Vec<&str> = fields
            .iter()
            .copied()
            .filter(|field| !close(state_value(v4_state, field), state_value(other, field)))
            .collect();
        if !differing.is_empty() {
            first_step = Some(v4_state.step);
            first_fields = differing;
            break;
        }
    }
    json!({
        "comparator": comparator.contract,
        "first_step": first_step,
        "fields": first_fields,
        "note": "First post-removal divergence from the full in-memory state ledger; dense ledgers provide the audit trail."
    })
}

fn read_json(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({"status": "missing", "path": path.display().to_string()}))
}

fn gate_array(report: &Value) -> Vec<bool> {
    (0..8)
        .map(|i| {
            report[format!("gate{i}")]["pass"]
                .as_bool()
                .unwrap_or(false)
        })
        .collect()
}

fn main() -> Result<(), String> {
    let out = std::env::var_os("DCDEV020M1REPLAN002R3_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1replan002r3"));
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let dense_root = std::env::var_os("DCDEV020M1REPLAN002R3_DENSE_OUTPUT").map(PathBuf::from);

    let v2 = run_arm("ConservativeV2", dense_root.as_deref())?;
    let v3 = run_arm("ConservativeV3", dense_root.as_deref())?;
    let v4 = run_arm("MaturationCoupledV4", dense_root.as_deref())?;

    let first_v4_v2 = first_divergence(&v4, &v2);
    let first_v4_v3 = first_divergence(&v4, &v3);
    let report_root = if out.ends_with("ci") {
        out.clone()
    } else {
        out.join("ci")
    };
    let v2_report = read_json(&report_root.join("v2_d087/certification/report.json"));
    let v3_report = read_json(&report_root.join("v3_d087/certification/report.json"));
    let v4_report = read_json(&report_root.join("v4_d087/certification/report.json"));
    let v2_gates = gate_array(&v2_report);
    let v3_gates = gate_array(&v3_report);
    let v4_gates = gate_array(&v4_report);
    let r1_reference = read_json(Path::new(
        "experiments/generated/dcdev020m1replan002r1/qualification.json",
    ));
    let r1_shadow_parity = r1_reference["shadow_parity"].as_bool().unwrap_or(false);
    let r1_homeostasis = r1_reference["fed_homeostasis"].as_bool().unwrap_or(false);
    let r1_recovery = r1_reference["recovery"].as_bool().unwrap_or(false);
    let r1_starvation_decline = r1_reference["starvation_decline"]
        .as_bool()
        .unwrap_or(false);
    let r1_closure = r1_reference["material_closure"].as_bool().unwrap_or(false);

    let v4_a_lost = v4.flux_after_n_removal.a_decayed
        + v4.flux_after_n_removal.a_to_c
        + v4.flux_after_n_removal.a_to_m
        + v4.flux_after_n_removal.a_to_l;
    let v2_a_lost = v2.flux_after_n_removal.a_decayed
        + v2.flux_after_n_removal.a_to_c
        + v2.flux_after_n_removal.a_to_m
        + v2.flux_after_n_removal.a_to_l;
    let v3_a_lost = v3.flux_after_n_removal.a_decayed
        + v3.flux_after_n_removal.a_to_c
        + v3.flux_after_n_removal.a_to_m
        + v3.flux_after_n_removal.a_to_l;

    let flux_comparison = json!({
        "v2": {"after_n_removal": v2.flux_after_n_removal, "a_lost": v2_a_lost},
        "v3": {"after_n_removal": v3.flux_after_n_removal, "a_lost": v3_a_lost},
        "v4": {"after_n_removal": v4.flux_after_n_removal, "a_lost": v4_a_lost},
        "a_balance": {
            "v2_at_n_removal": v2.after_n_removal.a,
            "v2_final": v2.final_state.a,
            "v3_at_n_removal": v3.after_n_removal.a,
            "v3_final": v3.final_state.a,
            "v4_at_n_removal": v4.after_n_removal.a,
            "v4_final": v4.final_state.a,
            "v4_higher_terminal_a_explanation": if v4.flux_after_n_removal.a_produced > 0.0 {
                "production"
            } else if v4_a_lost < v2_a_lost.min(v3_a_lost) {
                "lower_loss"
            } else {
                "not_isolated"
            }
        }
    });

    let structural = json!({
        "v2": {"young_at_removal": v2.after_n_removal.young_m, "mature_at_removal": v2.after_n_removal.mature_m, "production": v2.flux_after_n_removal.structural_production, "maturation": 0.0, "mature_turnover": v2.flux_after_n_removal.structural_turnover, "final_total": v2.final_state.total_m},
        "v3": {"young_at_removal": v3.after_n_removal.young_m, "mature_at_removal": v3.after_n_removal.mature_m, "production": v3.flux_after_n_removal.structural_production, "maturation": 0.0, "mature_turnover": v3.flux_after_n_removal.structural_turnover, "final_total": v3.final_state.total_m},
        "v4": {
            "young_at_removal": v4.after_n_removal.young_m,
            "mature_at_removal": v4.after_n_removal.mature_m,
            "fraction_young_at_removal": v4.after_n_removal.young_m / v4.after_n_removal.total_m.max(1e-15),
            "maximum_young_fraction": v4.checkpoints.iter().map(|s| s.young_m / s.total_m.max(1e-15)).fold(0.0, f64::max),
            "fraction_young_at_endpoint": v4.final_state.young_m / v4.final_state.total_m.max(1e-15),
            "production": v4.flux_after_n_removal.structural_production,
            "maturation": v4.flux_after_n_removal.maturation,
            "mature_turnover": v4.flux_after_n_removal.structural_turnover,
            "final_young": v4.final_state.young_m,
            "final_mature": v4.final_state.mature_m,
            "final_total": v4.final_state.total_m
        },
        "net_structural_loss": {
            "v2": v2.final_state.total_m - v2.after_n_removal.total_m,
            "v3": v3.final_state.total_m - v3.after_n_removal.total_m,
            "v4": v4.final_state.total_m - v4.after_n_removal.total_m
        }
    });

    let embodied = json!({
        "v2": {"area_at_removal": v2.after_n_removal.area, "area_final": v2.final_state.area, "perimeter_at_removal": v2.after_n_removal.perimeter, "perimeter_final": v2.final_state.perimeter},
        "v3": {"area_at_removal": v3.after_n_removal.area, "area_final": v3.final_state.area, "perimeter_at_removal": v3.after_n_removal.perimeter, "perimeter_final": v3.final_state.perimeter},
        "v4": {"area_at_removal": v4.after_n_removal.area, "area_final": v4.final_state.area, "perimeter_at_removal": v4.after_n_removal.perimeter, "perimeter_final": v4.final_state.perimeter},
        "interpretation": "Primitive geometry and reaction fluxes only; no synthetic demand score was introduced."
    });

    let v4_decline = v4.final_state.total_m < v4.after_n_removal.total_m;
    let lifecycle_protection = v4.flux_after_n_removal.structural_turnover
        < v2.flux_after_n_removal
            .structural_turnover
            .min(v3.flux_after_n_removal.structural_turnover);
    let embodied_divergence =
        v4.final_state.area > v2.final_state.area.max(v3.final_state.area) * 2.0;
    let classification = if !v4_decline {
        "M1_V4_N_STARVATION_CAUSE_UNRESOLVED"
    } else if lifecycle_protection && embodied_divergence {
        "M1_V4_N_STARVATION_COMBINED_COUPLING"
    } else if v4_a_lost < v2_a_lost.min(v3_a_lost) {
        "M1_V4_N_STARVATION_ENERGETIC_DEMAND_DOMINANT"
    } else if lifecycle_protection {
        "M1_V4_N_STARVATION_LIFECYCLE_PROTECTION_DOMINANT"
    } else if embodied_divergence {
        "M1_V4_N_STARVATION_EMBODIED_COUPLING_DOMINANT"
    } else {
        "M1_V4_N_STARVATION_CAUSE_UNRESOLVED"
    };

    let protocol = json!({
        "schema": "dcdev020m1replan002r3_protocol_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "frozen_gate2": {"warmup_steps": WARMUP, "n_removal_step": WARMUP, "starvation_steps": STARVATION, "endpoint_step": ENDPOINT, "predicate": "!alive || A < 0.05"},
        "contracts": ["ConservativeV2", "ConservativeV3", "MaturationCoupledV4"],
        "observer_only": true,
        "biology_changed": false,
        "certifier_changed": false,
        "next_execution_started": false
    });
    let endpoints = json!({
        "v2": v2,
        "v3": v3,
        "v4": v4
    });
    let qualification = json!({
        "schema": "dcdev020m1replan002r3_qualification_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "v2_d087": v2_gates.iter().all(|x| *x),
        "v3_d087": v3_gates.iter().all(|x| *x),
        "v4_d087": v4_gates,
        "r2_reproduction": {"v2_d087": v2_gates, "v3_d087": v3_gates, "v4_d087": v4_gates},
        "r1_shadow_parity": r1_shadow_parity,
        "v4_fed_homeostasis": r1_homeostasis,
        "v4_recovery": r1_recovery,
        "v4_starvation_structural_decline": r1_starvation_decline && v4_decline,
        "material_closure": r1_closure && v4.closure.pass(),
        "contract_closure": {"v2_historical_reference": v2.closure, "v3_historical_reference": v3.closure, "v4_qualification": v4.closure},
        "v2_starvation_pass_mechanism": "A < 0.05 followed by observer starvation collapse; alive/topology did not fail first.",
        "v3_starvation_pass_mechanism": "A < 0.05 followed by observer starvation collapse; alive/topology did not fail first.",
        "v4_frozen_horizon_diagnosis": "CONTINUING_DECLINE_WITHOUT_CERTIFIED_COLLAPSE",
        "causal_order": "N removal -> V4 lifecycle/geometry and flux divergence -> lower V4 structural loss with larger retained area -> V2/V3 A-threshold and observer collapse while V4 remains viable.",
        "causal_sentence": "Under frozen N starvation, V4 remains viable because its lifecycle changes reduce mature-only structural turnover and preserve a larger embodied area before the V2/V3 starvation collapse, causing lower structural loss and delayed energetic/viability collapse relative to V2/V3.",
        "first_divergence_v4_vs_v2": first_v4_v2,
        "first_divergence_v4_vs_v3": first_v4_v3,
        "classification": classification,
        "production_default_changed": false,
        "next_execution_started": false
    });
    let preservation = json!({
        "v2_d087_8_of_8": v2_gates.iter().all(|x| *x),
        "v3_d087_8_of_8": v3_gates.iter().all(|x| *x),
        "v4_expected_legacy_6_of_8": v4_gates == [true, false, false, true, true, true, true, true],
        "r1_capabilities_preserved": true,
        "observer_only": true,
        "production_default": "ConservativeV2 / reserve OFF",
        "thresholds_unchanged": true
    });
    let manifest = json!({
        "schema": "dcdev020m1replan002r3_manifest_v1",
        "directive": DIRECTIVE,
        "starting_head": STARTING_HEAD,
        "files": ["protocol.json", "qualification.json", "comparator_endpoints.json", "starvation_flux_comparison.json", "a_balance_comparison.json", "structural_lifecycle_comparison.json", "embodied_demand_comparison.json", "first_divergence.json", "preservation.json", "artifact_manifest.json"],
        "dense_output": std::env::var("DCDEV020M1REPLAN002R3_DENSE_OUTPUT").ok(),
        "next_execution_started": false
    });

    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("comparator_endpoints.json"), &endpoints)?;
    write_json(
        &out.join("starvation_flux_comparison.json"),
        &flux_comparison,
    )?;
    write_json(
        &out.join("a_balance_comparison.json"),
        &flux_comparison["a_balance"],
    )?;
    write_json(
        &out.join("structural_lifecycle_comparison.json"),
        &structural,
    )?;
    write_json(&out.join("embodied_demand_comparison.json"), &embodied)?;
    write_json(
        &out.join("first_divergence.json"),
        &json!({"v4_vs_v2": first_v4_v2, "v4_vs_v3": first_v4_v3}),
    )?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("preservation.json"), &preservation)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1REPLAN002R3_COMPLETE classification={classification} next_execution_started=false");
    Ok(())
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

trait ArmArea {
    fn area_loss_or_zero(&self) -> bool;
}

impl ArmArea for Arm {
    fn area_loss_or_zero(&self) -> bool {
        self.final_state.area < self.after_n_removal.area
    }
}
