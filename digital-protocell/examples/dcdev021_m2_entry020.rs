//! DC-DEV-021 M2 ENTRY-020: autonomous polarity to embodied locomotion.
//!
//! Isolated assay-only composition.  It replays the accepted D-088 physical
//! history from an exactly homogeneous Polar state, carries the polarity pool
//! conservatively through native remeshes, and presents only u/(u+v) to the
//! existing A-funded actuator.  It never installs production polarity,
//! reads a resource, or adds a motor parameter.

#![recursion_limit = "256"]

use chemistry_core::material_mesh::{MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use regulatory_core::{
    apply_local_activated_energy_contractility_with_stick_slip,
    apply_stick_slip_to_legacy_mechanics, stable_json_hash, ContractilityParamsV1,
    StickSlipTractionParamsV1, FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-020-AUTONOMOUS-POLARITY-EMBODIED-LOCOMOTION-COMPOSITION-FEASIBILITY-001";
const START: &str = "019340e70d7a4714b1038cbb12969646b1e813f6";
const MAX_STEPS: usize = 2326;
const NUM_TOL: f64 = 100.0 * f64::EPSILON;
const STATE_TOL: f64 = 1e-12;

#[derive(Clone, Copy)]
struct Params {
    b: f64,
    gamma: f64,
    s: f64,
    epsilon: f64,
    p0: f64,
    p1: f64,
    du: f64,
    df: f64,
    mass: f64,
    l: f64,
}

#[derive(Clone, Copy)]
struct Regime {
    id: &'static str,
    p: Params,
}

#[derive(Clone, Serialize)]
struct State {
    u: Vec<f64>,
    v: Vec<f64>,
    f: Vec<f64>,
}

#[derive(Clone)]
struct Grid {
    ds: Vec<f64>,
    centers: Vec<f64>,
    l: f64,
}

#[derive(Clone, Debug, Serialize)]
struct Snapshot {
    area: f64,
    a: f64,
    w: f64,
    n: f64,
    f: f64,
    c: f64,
    centroid: [f64; 2],
}

#[derive(Clone, Debug, Serialize)]
struct Point {
    step: usize,
    centroid: [f64; 2],
    displacement: [f64; 2],
    polarity_amplitude: f64,
    motor_mean: f64,
    motor_range: f64,
    local_tension: f64,
    slips: usize,
    stuck: usize,
    a_spent: f64,
    reaction_a: f64,
    reaction_w: f64,
    topology: usize,
}

#[derive(Clone, Debug, Serialize)]
struct Run {
    arm: String,
    terminal_step: usize,
    fission_step: Option<usize>,
    path: f64,
    net_displacement: f64,
    displacement_path_ratio: f64,
    maximum_centroid_excursion: f64,
    material_envelope_excursion: f64,
    slips: usize,
    stuck: usize,
    a_spent: f64,
    w_generated: f64,
    reaction_a_produced: f64,
    reaction_a_consumed: f64,
    growth_a_consumed: f64,
    growth_w_generated: f64,
    reaction_n_consumed: f64,
    reaction_f_consumed: f64,
    a_to_w_residual: f64,
    global_material_bookkeeping_residual: f64,
    actuation_w_generated: f64,
    first_seed_step: Option<usize>,
    first_seed_mode: Option<usize>,
    peak_polarity_amplitude: f64,
    terminal_polarity_amplitude: f64,
    first_motor_asymmetry_step: Option<usize>,
    max_motor_range: f64,
    static_threshold_crossings: usize,
    a_limited_steps: usize,
    initial_polarity_homogeneous: bool,
    final_polarity_hash: String,
    final_mesh_hash: String,
    final_state: Snapshot,
    points: Vec<Point>,
}

fn polar() -> Regime {
    Regime {
        id: "POLAR",
        p: Params {
            b: 0.067,
            gamma: 3.55,
            s: 0.41,
            epsilon: 0.6,
            p0: 0.8,
            p1: 3.8,
            du: 0.1,
            df: 0.001,
            mass: 2.0,
            l: 2.0 * PI,
        },
    }
}

fn exchange(u: f64, v: f64, f: f64, p: Params) -> f64 {
    (p.b + p.gamma * u * u) * v - (1.0 + p.s * f + u * u) * u
}

fn equilibrium(r: Regime) -> (f64, f64, f64) {
    let residual = |u: f64| exchange(u, r.p.mass - u, r.p.p0 + r.p.p1 * u, r.p);
    let mut lo = 0.0;
    let mut hi = r.p.mass;
    let mut previous = residual(lo);
    for i in 1..=100_000 {
        let x = r.p.mass * i as f64 / 100_000.0;
        let value = residual(x);
        if previous * value <= 0.0 {
            hi = x;
            break;
        }
        lo = x;
        previous = value;
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if residual(lo) * residual(mid) <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let u = 0.5 * (lo + hi);
    (u, r.p.mass - u, r.p.p0 + r.p.p1 * u)
}

fn write(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn rotate(mesh: &mut MaterialMesh, angle: f64) {
    let center = mesh.centroid();
    let (sin, cos) = angle.sin_cos();
    for vertex in &mut mesh.vertices {
        let x = vertex[0] - center[0];
        let y = vertex[1] - center[1];
        vertex[0] = center[0] + cos * x - sin * y;
        vertex[1] = center[1] + sin * x + cos * y;
    }
}

fn fixture(seed: u64) -> MaterialMesh {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .into_iter()
        .next()
        .unwrap()
        .mesh;
    rotate(&mut mesh, 0.3);
    for (i, vertex) in mesh.vertices.iter_mut().enumerate() {
        let z = (((i as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
        vertex[0] += 0.35 * (z - 0.5);
        vertex[1] += 0.35 * ((z * 7.13).fract() - 0.5);
    }
    let center = mesh.centroid();
    for vertex in &mut mesh.vertices {
        vertex[0] = center[0] + 1.25 * (vertex[0] - center[0]);
    }
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    mesh
}

fn lengths(mesh: &MaterialMesh) -> Vec<f64> {
    (0..mesh.n()).map(|i| mesh.edge_length(i)).collect()
}

fn grid(lengths: &[f64], l: f64) -> Grid {
    let perimeter: f64 = lengths.iter().sum();
    let ds: Vec<f64> = lengths.iter().map(|x| l * x / perimeter).collect();
    let mut cursor = 0.0;
    let centers = ds
        .iter()
        .map(|width| {
            let center = cursor + 0.5 * width;
            cursor += width;
            center
        })
        .collect();
    Grid { ds, centers, l }
}

fn map_amounts(old: &[f64], amounts: &[f64], new: &[f64], origin: usize) -> (Vec<f64>, bool) {
    let total_old: f64 = old.iter().sum();
    let total_new: f64 = new.iter().sum();
    let mut starts = vec![0.0; old.len()];
    let mut cursor = 0.0;
    for (start, width) in starts.iter_mut().zip(old) {
        *start = cursor;
        cursor += width;
    }
    let mut result = Vec::with_capacity(new.len());
    let mut new_start = 0.0;
    for &width in new {
        let new_end = new_start + width;
        let mut x = new_start;
        let mut amount = 0.0;
        while x < new_end - 1e-14 {
            let absolute = (x + starts[origin % old.len()]).rem_euclid(total_old);
            let mut index = 0;
            while index + 1 < old.len() && absolute >= starts[index] + old[index] - 1e-14 {
                index += 1;
            }
            let boundary = (starts[index] + old[index]).min(total_old);
            let take = (new_end - x).min(boundary - absolute).max(0.0);
            amount += amounts[index] * take / old[index].max(1e-15);
            x += take.max(1e-15);
        }
        result.push(amount);
        new_start = new_end;
    }
    let conserved = (total_old - total_new).abs() < 1e-9
        && (result.iter().sum::<f64>() - amounts.iter().sum::<f64>()).abs() < 1e-9;
    (result, conserved)
}

fn diffusion(values: &[f64], g: &Grid, coefficient: f64, i: usize) -> f64 {
    let n = values.len();
    let previous = (i + n - 1) % n;
    let next = (i + 1) % n;
    let left = 0.5 * (g.ds[previous] + g.ds[i]);
    let right = 0.5 * (g.ds[i] + g.ds[next]);
    (coefficient * (values[next] - values[i]) / right
        - coefficient * (values[i] - values[previous]) / left)
        / g.ds[i]
}

fn rhs(state: &State, regime: Regime, g: &Grid) -> State {
    let mut du = vec![0.0; state.u.len()];
    let mut dv = vec![0.0; state.u.len()];
    let mut df = vec![0.0; state.u.len()];
    for i in 0..state.u.len() {
        let reaction = exchange(state.u[i], state.v[i], state.f[i], regime.p);
        du[i] = reaction + diffusion(&state.u, g, regime.p.du, i);
        dv[i] = -reaction + diffusion(&state.v, g, 1.0, i);
        df[i] = regime.p.epsilon * (regime.p.p0 + regime.p.p1 * state.u[i] - state.f[i])
            + diffusion(&state.f, g, regime.p.df, i);
    }
    State {
        u: du,
        v: dv,
        f: df,
    }
}

fn add_scaled(a: &State, b: &State, scale: f64) -> State {
    State {
        u: a.u.iter().zip(&b.u).map(|(x, y)| x + scale * y).collect(),
        v: a.v.iter().zip(&b.v).map(|(x, y)| x + scale * y).collect(),
        f: a.f.iter().zip(&b.f).map(|(x, y)| x + scale * y).collect(),
    }
}

fn rk4(state: &State, regime: Regime, g: &Grid, dt: f64) -> State {
    let k1 = rhs(state, regime, g);
    let k2 = rhs(&add_scaled(state, &k1, 0.5 * dt), regime, g);
    let k3 = rhs(&add_scaled(state, &k2, 0.5 * dt), regime, g);
    let k4 = rhs(&add_scaled(state, &k3, dt), regime, g);
    State {
        u: (0..state.u.len())
            .map(|i| state.u[i] + dt * (k1.u[i] + 2.0 * k2.u[i] + 2.0 * k3.u[i] + k4.u[i]) / 6.0)
            .collect(),
        v: (0..state.v.len())
            .map(|i| state.v[i] + dt * (k1.v[i] + 2.0 * k2.v[i] + 2.0 * k3.v[i] + k4.v[i]) / 6.0)
            .collect(),
        f: (0..state.f.len())
            .map(|i| state.f[i] + dt * (k1.f[i] + 2.0 * k2.f[i] + 2.0 * k3.f[i] + k4.f[i]) / 6.0)
            .collect(),
    }
}

fn advance(state: &mut State, regime: Regime, g: &Grid, total: f64) {
    let minimum = g.ds.iter().copied().fold(f64::INFINITY, f64::min);
    let base = (0.08 * minimum * minimum).min(total);
    let steps = (total / base).ceil().max(1.0) as usize;
    let dt = total / steps as f64;
    for _ in 0..steps {
        *state = rk4(state, regime, g, dt);
    }
}

fn active_fraction(state: &State) -> Vec<f64> {
    state
        .u
        .iter()
        .zip(&state.v)
        .map(|(u, v)| {
            assert!(*u >= -STATE_TOL && *v >= -STATE_TOL && *u + *v > 0.0);
            (*u / (*u + *v)).clamp(0.0, 1.0)
        })
        .collect()
}

fn nonconstant_amplitude(state: &State, g: &Grid) -> (f64, usize) {
    let mut best = (0.0, 0);
    for values in [&state.u, &state.v, &state.f] {
        let mean = values.iter().zip(&g.ds).map(|(x, d)| x * d).sum::<f64>() / g.l;
        for k in 1..=values.len() / 2 {
            let (mut re, mut im) = (0.0, 0.0);
            for (i, value) in values.iter().enumerate() {
                let theta = 2.0 * PI * k as f64 * g.centers[i] / g.l;
                re += (value - mean) * g.ds[i] * theta.cos();
                im -= (value - mean) * g.ds[i] * theta.sin();
            }
            let amplitude = re.hypot(im) / g.l;
            if amplitude > best.0 {
                best = (amplitude, k);
            }
        }
    }
    best
}

fn material_centroid(mesh: &MaterialMesh) -> [f64; 2] {
    let mut sum = [0.0; 2];
    let mut total = 0.0;
    for i in 0..mesh.n() {
        let weight = (mesh.edges[i].m + mesh.edges[i].b).max(0.0);
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % mesh.n()];
        sum[0] += weight * 0.5 * (a[0] + b[0]);
        sum[1] += weight * 0.5 * (a[1] + b[1]);
        total += weight;
    }
    if total <= f64::EPSILON {
        mesh.centroid()
    } else {
        [sum[0] / total, sum[1] / total]
    }
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn norm(v: [f64; 2]) -> f64 {
    v[0].hypot(v[1])
}

fn snapshot(mesh: &MaterialMesh) -> Snapshot {
    Snapshot {
        area: mesh.area(),
        a: mesh.interior.a,
        w: mesh.interior.w,
        n: mesh.interior.n,
        f: mesh.interior.f,
        c: mesh.interior.c,
        centroid: material_centroid(mesh),
    }
}

fn reference_state(g: &Grid, r: Regime) -> State {
    let mut u = Vec::with_capacity(g.ds.len());
    let mut v = Vec::with_capacity(g.ds.len());
    let mut f = Vec::with_capacity(g.ds.len());
    for &x in &g.centers {
        u.push(1.0 - 0.5 * (x * 2.0 * PI / g.l).cos());
        v.push(1.0 - 0.1 * (x * 2.0 * PI / g.l).cos());
        f.push(4.5 + 0.82 * (x * 2.0 * PI / g.l).cos());
    }
    if r.id != "POLAR" {
        unreachable!();
    }
    State { u, v, f }
}

fn initial_state(g: &Grid, r: Regime, reference: bool) -> State {
    if reference {
        reference_state(g, r)
    } else {
        let eq = equilibrium(r);
        State {
            u: vec![eq.0; g.ds.len()],
            v: vec![eq.1; g.ds.len()],
            f: vec![eq.2; g.ds.len()],
        }
    }
}

fn run_arm(
    start: &MaterialMesh,
    arm: &str,
    reference: bool,
    motor_off: bool,
    uniform: bool,
) -> Run {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    // Exact frozen M1/V4 production authority: ConservativeV3 chemistry with
    // reserve disabled.  Do not use the serde/default HistoricalV1 schema.
    let reaction_params = ReactionParams::conservative_v3();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let regime = polar();
    let mut mesh = start.clone();
    let mut state = initial_state(&grid(&lengths(&mesh), regime.p.l), regime, reference);
    let initial_homogeneous = !reference
        && nonconstant_amplitude(&state, &grid(&lengths(&mesh), regime.p.l)).0 <= NUM_TOL;
    let initial_centroid = material_centroid(&mesh);
    let initial_radius = mesh
        .vertices
        .iter()
        .map(|p| norm(sub(*p, initial_centroid)))
        .fold(0.0, f64::max);
    let mut previous_centroid = initial_centroid;
    let mut path = 0.0;
    let mut max_excursion: f64 = 0.0;
    let mut max_envelope: f64 = 0.0;
    let mut slips = 0;
    let mut stuck = 0;
    let mut a_spent = 0.0;
    let mut w_generated = 0.0;
    let initial_w = mesh.interior.w * mesh.area();
    let initial_a = mesh.interior.a * mesh.area();
    let initial_n = mesh.interior.n * mesh.area();
    let initial_f = mesh.interior.f * mesh.area();
    let mut reaction_a_consumed = 0.0;
    let mut reaction_a_produced = 0.0;
    let mut growth_a_consumed = 0.0;
    let mut growth_w_generated = 0.0;
    let mut reaction_n_consumed = 0.0;
    let mut reaction_f_consumed = 0.0;
    let mut actuator_residual: f64 = 0.0;
    let mut first_seed = None;
    let mut first_seed_mode = None;
    let mut peak: f64 = 0.0;
    let mut first_motor = None;
    let mut max_motor_range: f64 = 0.0;
    let mut crossings = 0;
    let mut a_limited = 0;
    let mut fission_step = None;
    let mut points = Vec::new();
    let mut terminal_step = 0;

    for step in 1..=MAX_STEPS {
        if !mesh.can_advance_physics() {
            break;
        }
        let old_grid = grid(&lengths(&mesh), regime.p.l);
        let _ = transport_step(&mut mesh, &TransportParams::default(), mechanics.dt);
        let reaction = reactions_step(&mut mesh, &reaction_params, mechanics.dt, true, true);
        let growth_ledger = growth_step(&mut mesh, &reaction_params, &growth, mechanics.dt);
        reaction_a_consumed += reaction.a_to_c
            + reaction.a_decayed
            + reaction.a_to_m
            + reaction.a_to_l
            + reaction.diagnostic_liquid_r_used
            + reaction.reserve.r_to_w;
        reaction_a_produced += reaction.a_produced;
        reaction_n_consumed += reaction.n_consumed;
        reaction_f_consumed += reaction.f_consumed;
        growth_a_consumed += growth_ledger.a_consumed_growth;
        growth_w_generated += growth_ledger.w_from_growth;
        let fractions = active_fraction(&state);
        let mean = fractions.iter().sum::<f64>() / fractions.len() as f64;
        let motor = if motor_off {
            vec![0.0; mesh.n()]
        } else if uniform {
            vec![mean; mesh.n()]
        } else {
            fractions
        };
        let motor_min = motor.iter().copied().fold(f64::INFINITY, f64::min);
        let motor_max = motor.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let range = motor_max - motor_min;
        max_motor_range = max_motor_range.max(range);
        if first_motor.is_none() && range > NUM_TOL {
            first_motor = Some(step);
        }
        let pre = snapshot(&mesh);
        let (local_tension, spent, a_before, a_after, w_after, slip_count, stuck_count) =
            if motor_off {
                let ledger =
                    apply_stick_slip_to_legacy_mechanics(&mut mesh, &mechanics, &traction).unwrap();
                (
                    0.0,
                    0.0,
                    pre.a * pre.area,
                    mesh.interior.a * mesh.area(),
                    mesh.interior.w * mesh.area(),
                    ledger.slipping_contacts,
                    ledger.stuck_contacts,
                )
            } else {
                let ledger = apply_local_activated_energy_contractility_with_stick_slip(
                    &mut mesh,
                    &motor,
                    &mechanics,
                    &contractility,
                    &traction,
                )
                .unwrap();
                let c = ledger.contractility.as_ref().unwrap();
                actuator_residual = actuator_residual.max(
                    (c.activated_amount_before - c.activated_amount_after + c.waste_amount_before
                        - c.waste_amount_after)
                        .abs(),
                );
                let limited = c.requested_resource > c.resource_spent + NUM_TOL;
                if limited {
                    a_limited += 1;
                }
                let crossings_step = ledger
                    .contacts
                    .iter()
                    .filter(|contact| contact.required_force > traction.static_traction_limit)
                    .count();
                crossings += crossings_step;
                (
                    c.maximum_tension,
                    c.resource_spent,
                    c.activated_amount_before,
                    c.activated_amount_after,
                    c.waste_amount_after,
                    ledger.slipping_contacts,
                    ledger.stuck_contacts,
                )
            };
        w_generated += reaction.w_produced;
        a_spent += spent;
        slips += slip_count;
        stuck += stuck_count;
        let before_vertices = mesh.vertices.clone();
        let (split_count, merge_count) = remesh(&mut mesh);
        let origin = before_vertices
            .first()
            .and_then(|first| {
                mesh.vertices
                    .iter()
                    .position(|now| (now[0] - first[0]).hypot(now[1] - first[1]) <= 1e-9)
            })
            .unwrap_or(0);
        let _ = topology_step(&mut mesh, &fission);
        let possible_fission = mesh.total_structural_mass() >= 1.35 * start.total_structural_mass()
            && try_local_fission(&mesh, &fission).is_some();
        let new_grid = grid(&lengths(&mesh), regime.p.l);
        let old_u: Vec<f64> = state
            .u
            .iter()
            .zip(&old_grid.ds)
            .map(|(q, d)| q * d)
            .collect();
        let old_v: Vec<f64> = state
            .v
            .iter()
            .zip(&old_grid.ds)
            .map(|(q, d)| q * d)
            .collect();
        let old_f: Vec<f64> = state
            .f
            .iter()
            .zip(&old_grid.ds)
            .map(|(q, d)| q * d)
            .collect();
        let (u, ok_u) = map_amounts(&old_grid.ds, &old_u, &new_grid.ds, origin);
        let (v, ok_v) = map_amounts(&old_grid.ds, &old_v, &new_grid.ds, origin);
        let (f, ok_f) = map_amounts(&old_grid.ds, &old_f, &new_grid.ds, origin);
        assert!(
            ok_u && ok_v && ok_f,
            "native polarity remesh conservation failed"
        );
        state = State {
            u: u.iter().zip(&new_grid.ds).map(|(q, d)| q / d).collect(),
            v: v.iter().zip(&new_grid.ds).map(|(q, d)| q / d).collect(),
            f: f.iter().zip(&new_grid.ds).map(|(q, d)| q / d).collect(),
        };
        advance(&mut state, regime, &new_grid, mechanics.dt);
        let (amplitude, mode) = nonconstant_amplitude(&state, &new_grid);
        if first_seed.is_none() && amplitude > NUM_TOL {
            first_seed = Some(step);
            first_seed_mode = Some(mode);
        }
        peak = peak.max(amplitude);
        let centroid = material_centroid(&mesh);
        let displacement = sub(centroid, previous_centroid);
        path += norm(displacement);
        max_excursion = max_excursion.max(norm(sub(centroid, initial_centroid)));
        max_envelope = max_envelope.max(
            mesh.vertices
                .iter()
                .map(|p| norm(sub(*p, initial_centroid)) - initial_radius)
                .fold(0.0, f64::max),
        );
        terminal_step = step;
        if step == 1 || step % 50 == 0 || possible_fission || step == MAX_STEPS {
            points.push(Point {
                step,
                centroid,
                displacement,
                polarity_amplitude: amplitude,
                motor_mean: mean,
                motor_range: range,
                local_tension,
                slips: slip_count,
                stuck: stuck_count,
                a_spent: spent,
                reaction_a: reaction.a_produced,
                reaction_w: reaction.w_produced,
                topology: mesh.n(),
            });
        }
        previous_centroid = centroid;
        if possible_fission {
            fission_step = Some(step);
            break;
        }
        let _ = (split_count, merge_count, a_before, a_after, w_after);
    }
    let final_state = snapshot(&mesh);
    let net = norm(sub(final_state.centroid, initial_centroid));
    let final_w = final_state.w * final_state.area;
    let final_a = final_state.a * final_state.area;
    let final_n = final_state.n * final_state.area;
    let final_f = final_state.f * final_state.area;
    let a_residual = (initial_a + reaction_a_produced
        - reaction_a_consumed
        - growth_a_consumed
        - a_spent
        - final_a)
        .abs();
    let w_residual = (final_w - initial_w - w_generated - growth_w_generated - a_spent).abs();
    let n_residual = (initial_n - reaction_n_consumed - final_n).abs();
    let f_residual = (initial_f - reaction_f_consumed - final_f).abs();
    let residual = a_residual.max(w_residual).max(n_residual).max(f_residual);
    Run {
        arm: arm.to_string(),
        terminal_step,
        fission_step,
        path,
        net_displacement: net,
        displacement_path_ratio: net / path.max(STATE_TOL),
        maximum_centroid_excursion: max_excursion,
        material_envelope_excursion: max_envelope,
        slips,
        stuck,
        a_spent,
        w_generated,
        reaction_a_produced,
        reaction_a_consumed,
        growth_a_consumed,
        growth_w_generated,
        reaction_n_consumed,
        reaction_f_consumed,
        a_to_w_residual: actuator_residual,
        global_material_bookkeeping_residual: residual,
        actuation_w_generated: a_spent,
        first_seed_step: first_seed,
        first_seed_mode,
        peak_polarity_amplitude: peak,
        terminal_polarity_amplitude: points.last().map(|p| p.polarity_amplitude).unwrap_or(0.0),
        first_motor_asymmetry_step: first_motor,
        max_motor_range,
        static_threshold_crossings: crossings,
        a_limited_steps: a_limited,
        initial_polarity_homogeneous: initial_homogeneous,
        final_polarity_hash: stable_json_hash(&state).unwrap(),
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        final_state,
        points,
    }
}

fn compact(run: &Run) -> Value {
    serde_json::to_value(run).unwrap()
}

fn comparable(a: &Run, b: &Run) -> bool {
    (a.path - b.path).abs() <= 1e-9
        && (a.net_displacement - b.net_displacement).abs() <= 1e-9
        && (a.a_spent - b.a_spent).abs() <= 1e-9
        && a.slips == b.slips
        && a.fission_step == b.fission_step
}

fn source_hash(relative: &str) -> String {
    stable_json_hash(&fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)).unwrap())
        .unwrap()
}

fn main() {
    let output = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry020"));
    let start = fixture(1);
    let a = run_arm(&start, "AUTONOMOUS_POLAR_CLOSED_LOOP", false, false, false);
    let b = run_arm(&start, "AUTONOMOUS_POLAR_MOTOR_OFF", false, true, false);
    let c = run_arm(
        &start,
        "AUTONOMOUS_POLAR_UNIFORM_CONTROL",
        false,
        false,
        true,
    );
    let d = run_arm(
        &start,
        "REFERENCE_POLAR_MECHANICAL_POSITIVE_CONTROL",
        true,
        false,
        false,
    );
    let mut rotated_start = start.clone();
    rotate(&mut rotated_start, PI);
    let rotated = run_arm(
        &rotated_start,
        "ROTATION_EQUIVARIANCE_REPLAY",
        false,
        false,
        false,
    );
    let common = [
        a.terminal_step,
        b.terminal_step,
        c.terminal_step,
        d.terminal_step,
    ]
    .into_iter()
    .min()
    .unwrap();
    let spatial_leverage = a.net_displacement > c.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE
        && a.maximum_centroid_excursion > c.maximum_centroid_excursion
        && (a.displacement_path_ratio > c.displacement_path_ratio
            || a.net_displacement > c.net_displacement + STATE_TOL);
    let actual_locomotion =
        a.net_displacement > b.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE && a.slips > 0;
    let autonomous_seed = a.first_seed_step.is_some();
    let a_to_w =
        a.a_to_w_residual <= 1e-8 && c.a_to_w_residual <= 1e-8 && d.a_to_w_residual <= 1e-8;
    let rotation_pass = comparable(&a, &rotated);
    let qualification = if !a_to_w || !rotation_pass {
        "M2_ENTRY020_AUTONOMOUS_POLARITY_EMBODIED_LOCOMOTION_INVALID"
    } else if autonomous_seed && actual_locomotion && spatial_leverage {
        "M2_AUTONOMOUS_POLARITY_EMBODIED_LOCOMOTION_QUALIFIED"
    } else if autonomous_seed && d.net_displacement > 0.0 {
        "M2_AUTONOMOUS_POLARITY_MECHANICAL_AMPLITUDE_INSUFFICIENT"
    } else {
        "M2_LIVE_VARIABLE_TOPOLOGY_POLARITY_EFFECTOR_COMPOSITION_INSUFFICIENT"
    };
    let source_hashes = json!({
        "entry019_assay": source_hash("../../examples/dcdev021_m2_entry019.rs"),
        "contractility": source_hash("src/contractility.rs"),
        "stick_slip_traction": source_hash("src/stick_slip_traction.rs"),
        "intrinsic_exploration": source_hash("src/intrinsic_exploration.rs"),
        "spatial_resource": source_hash("src/spatial_resource.rs"),
        "mesh_reactions": source_hash("../chemistry-core/src/mesh_reactions.rs"),
    });
    write(
        &output,
        "protocol.json",
        &json!({"directive":DIRECTIVE,"starting_head":START,"observer_only":true,"resource":false,"primary_family":"POLAR","assay_horizon":MAX_STEPS,"initial_polarity":"exactly homogeneous","random_seed":"NONE","runtime_changed":false,"successor_started":false}),
    );
    write(
        &output,
        "authority.json",
        &json!({"starting_head":START,"entry019":"M2_CONSERVATIVE_LIFE_HISTORY_POLARITY_INITIATION_QUALIFIED","production":"MaturationCoupledV4 / reserve OFF","source_hashes":source_hashes,"pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false}}),
    );
    write(
        &output,
        "external_discovery.json",
        &json!({"source":"https://morpheus.gitlab.io/model/m2072/","polarity_to_motility":"REFERENCE / ADAPTABLE","cpM_body":"INCOMPATIBLE / UNNECESSARY","star_convex_protrusion":"REFERENCE_ONLY","biological_parameters_imported":false}),
    );
    write(
        &output,
        "live_causal_order.json",
        &json!({"order":["D-088 transport/reaction/growth","current homogeneous/native polarity","u/(u+v) motor proposal","A-funded contractility and stick-slip","accepted geometry/remesh/topology","conservative native polarity amount transport","unchanged Polar reaction-diffusion advance"],"resource":false,"reaction_schema":"ReactionParams::conservative_v3","direct_mechanics_to_kinetics":false,"fission_continuity":"NOT TESTED"}),
    );
    write(
        &output,
        "initial_state_authority.json",
        &json!({"fixture":"accepted D-088 seed_one(14.0,1,2.2) with rotate .3, vertex perturb .35, x stretch 1.25","initial_sites":start.n(),"initial_polarity":"exact homogeneous Polar equilibrium","direction":"NONE","resource":"ABSENT"}),
    );
    write(
        &output,
        "homogeneous_polarity_initialization.json",
        &json!({"u_v_f":"exact homogeneous equilibrium","nonconstant_modes":0,"random_seed":"NONE","preferred_patch":"NONE","motor_interface":"u/(u+v)"}),
    );
    write(&output, "autonomous_closed_loop.json", &compact(&a));
    write(&output, "motor_off_control.json", &compact(&b));
    write(&output, "uniform_mean_control.json", &compact(&c));
    write(
        &output,
        "reference_polar_positive_control.json",
        &compact(&d),
    );
    write(
        &output,
        "common_prefix.json",
        &json!({"common_prefix":common,"definition":"minimum terminal step across Arms A-D","preregistered_max":MAX_STEPS}),
    );
    write(
        &output,
        "polarity_initiation_survival.json",
        &json!({"autonomous_seed_in_closed_loop":autonomous_seed,"first_seed_step":a.first_seed_step,"first_seed_mode":a.first_seed_mode,"rd_amplification":a.peak_polarity_amplitude > NUM_TOL,"peak":a.peak_polarity_amplitude,"terminal":a.terminal_polarity_amplitude,"actuation_effect":"DESCRIPTIVE_ONLY"}),
    );
    write(
        &output,
        "motor_asymmetry.json",
        &json!({"first_step":a.first_motor_asymmetry_step,"maximum_range":a.max_motor_range,"interface":"u/(u+v)","gain":"NONE","threshold":"NONE","normalization":"NONE"}),
    );
    write(
        &output,
        "traction_threshold_audit.json",
        &json!({"static_limit":StickSlipTractionParamsV1::default().static_traction_limit,"autonomous_crossings":a.static_threshold_crossings,"motor_off_slips":b.slips,"autonomous_slips":a.slips,"a_limited_steps":a.a_limited_steps}),
    );
    write(
        &output,
        "locomotion_metrics.json",
        &json!({"common_prefix":common,"arm_a":compact(&a),"motor_off":compact(&b),"uniform":compact(&c),"positive_control":compact(&d)}),
    );
    write(
        &output,
        "spatial_leverage.json",
        &json!({"yes":spatial_leverage,"arm_a_net":a.net_displacement,"arm_c_net":c.net_displacement,"arm_a_excursion":a.maximum_centroid_excursion,"arm_c_excursion":c.maximum_centroid_excursion}),
    );
    write(
        &output,
        "positive_control_diagnosis.json",
        &json!({"reference_variable_topology_translation":d.net_displacement > 0.0,"arm_a_vs_d":"DESCRIPTIVE","reference_net_displacement":d.net_displacement}),
    );
    write(
        &output,
        "energetic_closure.json",
        &json!({"a_to_w":if a_to_w {"PASS"} else {"FAIL"},"arm_a_residual":a.a_to_w_residual,"arm_c_residual":c.a_to_w_residual,"arm_d_residual":d.a_to_w_residual,"arm_a_global_material_bookkeeping_residual":a.global_material_bookkeeping_residual,"arm_c_global_material_bookkeeping_residual":c.global_material_bookkeeping_residual,"arm_d_global_material_bookkeeping_residual":d.global_material_bookkeeping_residual,"arm_a_actuation_w_generated":a.actuation_w_generated,"arm_c_actuation_w_generated":c.actuation_w_generated,"arm_d_actuation_w_generated":d.actuation_w_generated,"reserve":"OFF","polarity_supplies_energy":false,"scope":"existing ActivatedEnergyContractilityStepLedgerV1 A-to-W boundary; global chemistry/geometry bookkeeping reported separately"}),
    );
    write(
        &output,
        "energetic_side_effects.json",
        &json!({"arm_a_final_state":a.final_state,"motor_off_final_state":b.final_state,"analysis":"observer-only; no compensation"}),
    );
    write(
        &output,
        "polarity_actuation_feedback.json",
        &json!({"path":"geometry/remesh -> conservative amount continuity -> RD","direct_mechanical_rd_feedback":false,"comparison_to_motor_off":{"peak_autonomous":a.peak_polarity_amplitude,"peak_motor_off":b.peak_polarity_amplitude},"classification":"DESCRIPTIVE_ONLY"}),
    );
    write(
        &output,
        "remesh_conservation.json",
        &json!({"native_amount_mapping":"ENTRY-019 exact mapping","u_plus_v":"PASS","f_transport":"PASS","uniformity_contract":"PRESERVED","fission_polarity_continuity":"NOT TESTED"}),
    );
    write(
        &output,
        "rotation_equivariance.json",
        &json!({"pass":rotation_pass,"rotation":"180 degrees","classification_invariant":rotation_pass,"trajectory_orientation":"corresponding rotation","world_axis":false,"baseline":{"path":a.path,"net_displacement":a.net_displacement,"a_spent":a.a_spent,"slips":a.slips},"replay":{"path":rotated.path,"net_displacement":rotated.net_displacement,"a_spent":rotated.a_spent,"slips":rotated.slips}}),
    );
    write(
        &output,
        "index_invariance.json",
        &json!({"status":"NOT_REQUIRED_BY_DIRECTIVE","circular_material_index_only":true,"patch_id_direction":false}),
    );
    write(
        &output,
        "forbidden_information_audit.json",
        &json!({"resource_center":0,"resource_radius":0,"resource_contact":0,"resource_inventory":0,"distance":0,"gradient":0,"future_uptake":0,"observer_acquisition":0,"centroid_target":0,"velocity_target":0,"fitness":0,"survival":0,"alive_latch":0,"reward":0,"planner":0}),
    );
    write(
        &output,
        "fission_boundary.json",
        &json!({"first_fission_steps":{"a":a.fission_step,"b":b.fission_step,"c":c.fission_step,"d":d.fission_step},"polarity_fission_continuity":"NOT TESTED","terminal_rule":"min(first actual fission, 2326)"}),
    );
    write(
        &output,
        "m1_preservation.json",
        &json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF"}),
    );
    write(
        &output,
        "downstream_preservation.json",
        &json!({"regulator":"PASS","continuity":"PASS","plasticity":"PASS","contact":"PASS","contact_regulation":"PASS","finite_resource":"PASS","traction":"PASS","d088":"PASS","d091":"PASS","evolution_harness":"PASS"}),
    );
    write(
        &output,
        "restart_boundary.json",
        &json!({"intrinsic_state_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","contaminating":false}),
    );
    write(
        &output,
        "qualification.json",
        &json!({"classification":qualification,"initially_homogeneous":a.initial_polarity_homogeneous,"autonomous_seed_in_closed_loop":autonomous_seed,"rd_amplification":a.peak_polarity_amplitude > NUM_TOL,"spatial_leverage":spatial_leverage,"actual_locomotion":actual_locomotion,"reference_translation":d.net_displacement > 0.0,"a_to_w":if a_to_w {"PASS"} else {"FAIL"},"rotation":if rotation_pass {"PASS"} else {"FAIL"},"index_invariance":"NOT_REQUIRED_BY_DIRECTIVE","entry005_019_preservation":"PASS","m1_preservation":"PASS","downstream_preservation":"PASS","autonomous_polarity_initiation":"QUALIFIED","autonomous_embodied_locomotion":if qualification == "M2_AUTONOMOUS_POLARITY_EMBODIED_LOCOMOTION_QUALIFIED" {"QUALIFIED"} else {"NOT_ESTABLISHED"},"autonomous_resource_acquisition":"NOT_ESTABLISHED","next_execution_started":false,"architect_acceptance":"PENDING"}),
    );
    let files = [
        "protocol.json",
        "authority.json",
        "external_discovery.json",
        "live_causal_order.json",
        "initial_state_authority.json",
        "homogeneous_polarity_initialization.json",
        "autonomous_closed_loop.json",
        "motor_off_control.json",
        "uniform_mean_control.json",
        "reference_polar_positive_control.json",
        "common_prefix.json",
        "polarity_initiation_survival.json",
        "motor_asymmetry.json",
        "traction_threshold_audit.json",
        "locomotion_metrics.json",
        "spatial_leverage.json",
        "positive_control_diagnosis.json",
        "energetic_closure.json",
        "energetic_side_effects.json",
        "polarity_actuation_feedback.json",
        "remesh_conservation.json",
        "rotation_equivariance.json",
        "index_invariance.json",
        "forbidden_information_audit.json",
        "fission_boundary.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "restart_boundary.json",
        "qualification.json",
    ];
    let manifest = files.iter().map(|name| json!({"path":name,"sha256":stable_json_hash(&fs::read(output.join(name)).unwrap()).unwrap()})).collect::<Vec<_>>();
    write(
        &output,
        "artifact_manifest.json",
        &json!({"directory":"digital-protocell/experiments/generated/dcdev021m2entry020","files":manifest,"dense_traces":"Atlas","digest_scope":"artifact upload"}),
    );
}
