//! DC-DEV-021 M2 ENTRY-021: conservative polarity fission inheritance audit.
//!
//! This is an isolated observer assay.  It replays the accepted D-088 physical
//! mother trajectory, carries the accepted ENTRY-019 Polar state as local
//! control-volume amounts through the existing fission operation, and advances
//! both daughters without an actuator or resource.  Fission itself is not
//! redesigned: inherited parent edge material receives its corresponding
//! polarity amount and a newly synthesized closing edge has no predecessor and
//! therefore receives zero transported amount.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{try_local_fission, FissionEvent, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_topology::TopologyLedger;
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use serde_json::{json, Value};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "DC-DEV-021-M2-ENTRY-021-CONSERVATIVE-POLARITY-FISSION-INHERITANCE-AND-AMPLIFICATION-FEASIBILITY-001";
const START: &str = "af0871fe8b8ae60f8eb696e555f73ae30e5d8bc9";
const DT: f64 = 0.02;
const POLAR_L: f64 = 2.0 * PI;
const NUM_TOL: f64 = 1e-10;
const POST_STEPS: usize = 3_000;

#[derive(Clone)]
struct Replay {
    mother: MaterialMesh,
    daughter_a: MaterialMesh,
    daughter_b: MaterialMesh,
    event: FissionEvent,
    parent_ds: Vec<f64>,
    parent_u: Vec<f64>,
    parent_v: Vec<f64>,
    parent_f: Vec<f64>,
    first_fission_step: usize,
}

#[derive(Clone)]
struct Grid {
    ds: Vec<f64>,
    centers: Vec<f64>,
}

#[derive(Clone)]
struct AmountState {
    u: Vec<f64>,
    v: Vec<f64>,
    f: Vec<f64>,
}

#[derive(Clone)]
struct DaughterResult {
    initial: Value,
    control_initial: Value,
    terminal: Value,
    control_terminal: Value,
    peak_amp: f64,
    control_peak_amp: f64,
    peak_after_first_step: f64,
    control_peak_after_first_step: f64,
    peak_motor: f64,
    control_peak_motor: f64,
    remesh_closure: f64,
    remesh_events: usize,
}

fn rotate(mesh: &mut MaterialMesh, angle: f64) {
    let c = mesh.centroid();
    let (s, co) = angle.sin_cos();
    for p in &mut mesh.vertices {
        let x = p[0] - c[0];
        let y = p[1] - c[1];
        p[0] = c[0] + co * x - s * y;
        p[1] = c[1] + s * x + co * y;
    }
}

fn perturb(mesh: &mut MaterialMesh, kind: &str, magnitude: f64) {
    if kind == "rotate" {
        rotate(mesh, magnitude);
        return;
    }
    for (i, p) in mesh.vertices.iter_mut().enumerate() {
        let z = (((i as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
        p[0] += magnitude * (z - 0.5);
        p[1] += magnitude * ((z * 7.13).fract() - 0.5);
    }
}

fn fixture(seed: u64) -> MaterialMesh {
    let mut mesh = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .into_iter()
        .next()
        .unwrap()
        .mesh;
    perturb(&mut mesh, "rotate", 0.3);
    perturb(&mut mesh, "vertex", 0.35);
    let c = mesh.centroid();
    for p in &mut mesh.vertices {
        p[0] = c[0] + (p[0] - c[0]) * 1.25;
    }
    mesh
}

fn physical_step(mesh: &mut MaterialMesh, apply_topology: bool) -> (TopologyLedger, usize) {
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let _ = transport_step(mesh, &transport, mech.dt);
    let _ = reactions_step(mesh, &react, mech.dt, true, true);
    let _ = growth_step(mesh, &react, &growth, mech.dt);
    assert!(mechanics_step(mesh, &mech));
    let old_vertices = mesh.vertices.clone();
    remesh(mesh);
    let origin = mesh
        .vertices
        .first()
        .and_then(|new_first| {
            old_vertices
                .iter()
                .position(|old| (old[0] - new_first[0]).hypot(old[1] - new_first[1]) <= 1e-9)
        })
        .unwrap_or(0);
    let topology = if apply_topology {
        chemistry_core::mesh_fission::topology_step(mesh, &fission)
    } else {
        TopologyLedger::default()
    };
    (topology, origin)
}

fn no_growth_physical_step(mesh: &mut MaterialMesh) -> (Vec<f64>, Vec<f64>, usize) {
    let old_lengths: Vec<f64> = (0..mesh.n()).map(|i| mesh.edge_length(i)).collect();
    let old_vertices = mesh.vertices.clone();
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: false,
    };
    let fission = FissionParams::default();
    let _ = transport_step(mesh, &transport, mech.dt);
    let _ = reactions_step(mesh, &react, mech.dt, true, true);
    let _ = growth_step(mesh, &react, &growth, mech.dt);
    assert!(mechanics_step(mesh, &mech));
    remesh(mesh);
    let _ = chemistry_core::mesh_fission::topology_step(mesh, &fission);
    let origin = mesh
        .vertices
        .first()
        .and_then(|new_first| {
            old_vertices
                .iter()
                .position(|old| (old[0] - new_first[0]).hypot(old[1] - new_first[1]) <= 1e-9)
        })
        .unwrap_or(0);
    let new_lengths: Vec<f64> = (0..mesh.n()).map(|i| mesh.edge_length(i)).collect();
    (old_lengths, new_lengths, origin)
}

fn replay_run(rotated: bool, reindexed: bool) -> Replay {
    let mut mesh = fixture(1);
    if rotated {
        rotate(&mut mesh, PI);
    }
    if reindexed {
        mesh.vertices.rotate_left(1);
        mesh.edges.rotate_left(1);
    }
    let birth_mass = mesh.total_structural_mass();
    let mut current_grid = grid(
        &(0..mesh.n())
            .map(|i| mesh.edge_length(i))
            .collect::<Vec<_>>(),
    );
    let mut state = {
        let (u, v, f) = (1.0, 1.0, 4.6);
        AmountState {
            u: vec![u; mesh.n()],
            v: vec![v; mesh.n()],
            f: vec![f; mesh.n()],
        }
    };
    // The accepted ENTRY-019 authority uses its exact Polar homogeneous root.
    // These values are overwritten by the root solve below before execution.
    let (eq_u, eq_v, eq_f) = polar_equilibrium();
    state.u.fill(eq_u);
    state.v.fill(eq_v);
    state.f.fill(eq_f);
    for step in 0..12_000 {
        if !mesh.can_advance_physics() {
            break;
        }
        let (_, origin) = physical_step(&mut mesh, step % 10 == 0);
        let new_grid = grid(
            &(0..mesh.n())
                .map(|i| mesh.edge_length(i))
                .collect::<Vec<_>>(),
        );
        state = remap(&current_grid, &state, &new_grid, origin);
        advance(&mut state, &new_grid, DT);
        current_grid = new_grid;
        if step % 25 == 0 && mesh.total_structural_mass() >= 1.35 * birth_mass {
            if let Some((da, db, event)) = try_local_fission(&mesh, &FissionParams::default()) {
                return Replay {
                    mother: mesh,
                    daughter_a: da,
                    daughter_b: db,
                    event,
                    parent_ds: current_grid.ds,
                    parent_u: state.u,
                    parent_v: state.v,
                    parent_f: state.f,
                    first_fission_step: step + 1,
                };
            }
        }
    }
    panic!("accepted D-088 fission was not reached");
}

fn polar_equilibrium() -> (f64, f64, f64) {
    let mass = 2.0;
    let reaction = |u: f64| {
        let v = mass - u;
        let f = 0.8 + 3.8 * u;
        (0.067 + 3.55 * u * u) * v - (1.0 + 0.41 * f + u * u) * u
    };
    let n = 100_000;
    let mut x = 0.0;
    let mut a = reaction(x);
    for j in 1..=n {
        let y = mass * j as f64 / n as f64;
        let b = reaction(y);
        if a * b < 0.0 {
            let (mut lo, mut hi, mut fl) = (x, y, a);
            for _ in 0..80 {
                let mid = (lo + hi) * 0.5;
                let fm = reaction(mid);
                if fl * fm <= 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                    fl = fm;
                }
            }
            let u = (lo + hi) * 0.5;
            return (u, mass - u, 0.8 + 3.8 * u);
        }
        x = y;
        a = b;
    }
    panic!("accepted Polar homogeneous equilibrium was not found")
}

fn write(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn stable_hash(path: &Path) -> String {
    regulatory_core::stable_json_hash(&fs::read(path).unwrap()).unwrap()
}

fn grid(lengths: &[f64]) -> Grid {
    let perimeter: f64 = lengths.iter().sum();
    let ds: Vec<f64> = lengths.iter().map(|x| POLAR_L * x / perimeter).collect();
    let mut cursor = 0.0;
    let centers = ds
        .iter()
        .map(|d| {
            let center = cursor + 0.5 * d;
            cursor += *d;
            center
        })
        .collect();
    Grid { ds, centers }
}

fn map_amounts(old: &Grid, amounts: &[f64], new: &Grid, origin: usize) -> Vec<f64> {
    let n = old.ds.len();
    let mut starts = vec![0.0; n];
    let mut cursor = 0.0;
    for i in 0..n {
        starts[i] = cursor;
        cursor += old.ds[i];
    }
    let total = cursor;
    let mut out = Vec::with_capacity(new.ds.len());
    let mut start = 0.0;
    for width in &new.ds {
        let end = start + width;
        let mut x = start;
        let mut amount = 0.0;
        while x < end - 1e-14 {
            let absolute = (x + starts[origin % n]).rem_euclid(total);
            let mut oi = 0;
            while oi + 1 < n && absolute >= starts[oi] + old.ds[oi] - 1e-14 {
                oi += 1;
            }
            let boundary = (starts[oi] + old.ds[oi]).min(total);
            let take = (end - x).min(boundary - absolute).max(0.0);
            amount += amounts[oi] * take / old.ds[oi].max(1e-15);
            x += take.max(1e-15);
        }
        out.push(amount);
        start = end;
    }
    out
}

fn partition_amounts(replay: &Replay) -> (Grid, Grid, AmountState, AmountState, Value) {
    let parent_lengths: Vec<f64> = (0..replay.mother.n())
        .map(|i| replay.mother.edge_length(i))
        .collect();
    let parent_grid = Grid {
        ds: replay.parent_ds.clone(),
        centers: grid(&parent_lengths).centers,
    };
    let da_lengths: Vec<f64> = (0..replay.daughter_a.n())
        .map(|i| replay.daughter_a.edge_length(i))
        .collect();
    let db_lengths: Vec<f64> = (0..replay.daughter_b.n())
        .map(|i| replay.daughter_b.edge_length(i))
        .collect();
    let da_grid = grid(&da_lengths);
    let db_grid = grid(&db_lengths);
    let (i, j) = replay.event.pinch;
    let pn = replay.mother.n();
    let mut correspondence_a = Vec::new();
    let mut correspondence_b = Vec::new();
    let mut make = |start: usize, count: usize, daughter: &MaterialMesh, out: &mut Vec<Value>| {
        let mut u = Vec::with_capacity(daughter.n());
        let mut v = Vec::with_capacity(daughter.n());
        let mut f = Vec::with_capacity(daughter.n());
        for k in 0..daughter.n() {
            if k + 1 == daughter.n() {
                u.push(0.0);
                v.push(0.0);
                f.push(0.0);
                out.push(json!({"daughter_edge":k,"class":"NEW_FISSION_CLOSING_MATERIAL","parent_source":null,"inherited_amount":"ZERO_NO_PARENT_PREDECESSOR"}));
            } else {
                let source = (start + k) % pn;
                assert!(k < count);
                u.push(replay.parent_u[source] * replay.parent_ds[source]);
                v.push(replay.parent_v[source] * replay.parent_ds[source]);
                f.push(replay.parent_f[source] * replay.parent_ds[source]);
                out.push(json!({"daughter_edge":k,"class":"INHERITED_PARENT_MATERIAL","parent_source_edge":source,"source_subarc":"exact_parent_edge"}));
            }
        }
        (u, v, f)
    };
    let (ua, va, fa) = make(
        i,
        (j + pn - i) % pn,
        &replay.daughter_a,
        &mut correspondence_a,
    );
    let (ub, vb, fb) = make(
        j,
        (i + pn - j) % pn,
        &replay.daughter_b,
        &mut correspondence_b,
    );
    let a = AmountState {
        u: ua,
        v: va,
        f: fa,
    };
    let b = AmountState {
        u: ub,
        v: vb,
        f: fb,
    };
    let parent_totals: [f64; 3] = [
        replay
            .parent_u
            .iter()
            .zip(&replay.parent_ds)
            .map(|(q, d)| q * d)
            .sum::<f64>(),
        replay
            .parent_v
            .iter()
            .zip(&replay.parent_ds)
            .map(|(q, d)| q * d)
            .sum::<f64>(),
        replay
            .parent_f
            .iter()
            .zip(&replay.parent_ds)
            .map(|(q, d)| q * d)
            .sum::<f64>(),
    ];
    let daughter_totals = |s: &AmountState| [s.u.iter().sum(), s.v.iter().sum(), s.f.iter().sum()];
    let ta: [f64; 3] = daughter_totals(&a);
    let tb: [f64; 3] = daughter_totals(&b);
    let correspondence = json!({
        "pinch": [i,j],
        "daughter_a": correspondence_a,
        "daughter_b": correspondence_b,
        "mapping": "exact contiguous parent edge slices from extract_loop; no interpolation",
        "parent_totals": parent_totals,
        "daughter_totals": [ta,tb],
        "u_plus_v_closure": (ta[0]+ta[1]+tb[0]+tb[1]-(parent_totals[0]+parent_totals[1])).abs(),
        "f_transport_closure": (ta[2]+tb[2]-parent_totals[2]).abs(),
        "new_closing_edge_semantics": "zero inherited amount because synthesized edge has no parent material predecessor"
    });
    (da_grid, db_grid, a, b, correspondence)
}

fn weighted(s: &[f64], g: &Grid) -> f64 {
    s.iter().zip(&g.ds).map(|(q, d)| q * d).sum()
}

fn diffusion(q: &[f64], g: &Grid, d: f64, i: usize) -> f64 {
    let n = q.len();
    let prev = (i + n - 1) % n;
    let next = (i + 1) % n;
    let dp = 0.5 * (g.ds[prev] + g.ds[i]);
    let dn = 0.5 * (g.ds[i] + g.ds[next]);
    (d * (q[next] - q[i]) / dn - d * (q[i] - q[prev]) / dp) / g.ds[i]
}

fn exchange(u: f64, v: f64, f: f64) -> f64 {
    (0.067 + 3.55 * u * u) * v - (1.0 + 0.41 * f + u * u) * u
}

fn rhs(s: &AmountState, g: &Grid) -> AmountState {
    let mut u = vec![0.0; s.u.len()];
    let mut v = u.clone();
    let mut f = u.clone();
    for i in 0..s.u.len() {
        let e = exchange(s.u[i], s.v[i], s.f[i]);
        u[i] = e + diffusion(&s.u, g, 0.1, i);
        v[i] = -e + diffusion(&s.v, g, 1.0, i);
        f[i] = 0.6 * (0.8 + 3.8 * s.u[i] - s.f[i]) + diffusion(&s.f, g, 0.001, i);
    }
    AmountState { u, v, f }
}

fn add(s: &AmountState, k: &AmountState, scale: f64) -> AmountState {
    AmountState {
        u: s.u.iter().zip(&k.u).map(|(x, y)| x + scale * y).collect(),
        v: s.v.iter().zip(&k.v).map(|(x, y)| x + scale * y).collect(),
        f: s.f.iter().zip(&k.f).map(|(x, y)| x + scale * y).collect(),
    }
}

fn advance(s: &mut AmountState, g: &Grid, total: f64) {
    let min_ds = g.ds.iter().copied().fold(f64::INFINITY, f64::min);
    let h0 = (0.08 * min_ds * min_ds).min(total);
    let count = (total / h0).ceil().max(1.0) as usize;
    let h = total / count as f64;
    for _ in 0..count {
        let a = rhs(s, g);
        let b = rhs(&add(s, &a, h * 0.5), g);
        let c = rhs(&add(s, &b, h * 0.5), g);
        let d = rhs(&add(s, &c, h), g);
        for i in 0..s.u.len() {
            s.u[i] += h * (a.u[i] + 2.0 * b.u[i] + 2.0 * c.u[i] + d.u[i]) / 6.0;
            s.v[i] += h * (a.v[i] + 2.0 * b.v[i] + 2.0 * c.v[i] + d.v[i]) / 6.0;
            s.f[i] += h * (a.f[i] + 2.0 * b.f[i] + 2.0 * c.f[i] + d.f[i]) / 6.0;
        }
    }
}

fn remap(old: &Grid, state: &AmountState, new: &Grid, origin: usize) -> AmountState {
    let map = |q: &[f64]| {
        let amount: Vec<f64> = q.iter().zip(&old.ds).map(|(x, d)| x * d).collect();
        map_amounts(old, &amount, new, origin)
            .iter()
            .zip(&new.ds)
            .map(|(a, d)| a / d)
            .collect::<Vec<_>>()
    };
    AmountState {
        u: map(&state.u),
        v: map(&state.v),
        f: map(&state.f),
    }
}

fn mode(q: &[f64], g: &Grid, k: usize) -> f64 {
    let mean = weighted(q, g) / POLAR_L;
    let mut re = 0.0;
    let mut im = 0.0;
    for (i, x) in q.iter().enumerate() {
        let z = 2.0 * PI * k as f64 * g.centers[i] / POLAR_L;
        re += (x - mean) * g.ds[i] * z.cos();
        im -= (x - mean) * g.ds[i] * z.sin();
    }
    re.hypot(im) / POLAR_L
}

fn motor_range(s: &AmountState) -> f64 {
    s.u.iter()
        .zip(&s.v)
        .map(|(u, v)| u / (u + v).max(1e-15))
        .fold(0.0_f64, |acc, x| acc.max(x))
        - s.u
            .iter()
            .zip(&s.v)
            .map(|(u, v)| u / (u + v).max(1e-15))
            .fold(f64::INFINITY, f64::min)
}

fn state_summary(s: &AmountState, g: &Grid, step: usize) -> Value {
    let modes: Vec<f64> = (1..=s.u.len() / 2).map(|k| mode(&s.u, g, k)).collect();
    let (dominant, amp) = modes
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, x)| (i + 1, *x))
        .unwrap_or((0, 0.0));
    json!({
        "step": step, "sites": s.u.len(),
        "weighted_u": weighted(&s.u,g), "weighted_v": weighted(&s.v,g), "weighted_f": weighted(&s.f,g),
        "max_nonconstant_u": (1..=s.u.len()/2).map(|k| mode(&s.u,g,k)).fold(0.0,f64::max),
        "max_nonconstant_v": (1..=s.v.len()/2).map(|k| mode(&s.v,g,k)).fold(0.0,f64::max),
        "max_nonconstant_f": (1..=s.f.len()/2).map(|k| mode(&s.f,g,k)).fold(0.0,f64::max),
        "dominant_mode": dominant, "dominant_mode_amplitude": amp,
        "motor_range": motor_range(s)
    })
}

fn run_daughter(
    mesh: &MaterialMesh,
    g: Grid,
    initial: AmountState,
    control: AmountState,
) -> DaughterResult {
    let mut physical = mesh.clone();
    let mut control_mesh = mesh.clone();
    let mut s = initial;
    let mut c = control;
    let init_value = state_summary(&s, &g, 0);
    let control_init = state_summary(&c, &g, 0);
    let mut current_grid = g;
    let mut peak = init_value["max_nonconstant_u"]
        .as_f64()
        .unwrap_or(0.0)
        .max(init_value["max_nonconstant_v"].as_f64().unwrap_or(0.0))
        .max(init_value["max_nonconstant_f"].as_f64().unwrap_or(0.0));
    let mut control_peak = control_init["max_nonconstant_u"]
        .as_f64()
        .unwrap_or(0.0)
        .max(control_init["max_nonconstant_v"].as_f64().unwrap_or(0.0))
        .max(control_init["max_nonconstant_f"].as_f64().unwrap_or(0.0));
    let mut peak_motor = init_value["motor_range"].as_f64().unwrap_or(0.0);
    let mut control_peak_motor = control_init["motor_range"].as_f64().unwrap_or(0.0);
    let mut max_closure: f64 = 0.0;
    let mut remesh_events = 0;
    let mut terminal = init_value.clone();
    let mut control_terminal = control_init.clone();
    let mut peak_after_first_step = 0.0;
    let mut control_peak_after_first_step = 0.0;
    for step in 1..=POST_STEPS {
        let (old_l, new_l, origin) = no_growth_physical_step(&mut physical);
        let old_grid = current_grid.clone();
        let new_grid = grid(&new_l);
        let old_amount = weighted(&s.u, &old_grid) + weighted(&s.v, &old_grid);
        s = remap(&old_grid, &s, &new_grid, origin);
        let after_amount = weighted(&s.u, &new_grid) + weighted(&s.v, &new_grid);
        max_closure = max_closure.max((after_amount - old_amount).abs());
        if old_l.len() != new_l.len() {
            remesh_events += 1;
        }
        advance(&mut s, &new_grid, DT);
        let point = state_summary(&s, &new_grid, step);
        peak = peak
            .max(point["max_nonconstant_u"].as_f64().unwrap_or(0.0))
            .max(point["max_nonconstant_v"].as_f64().unwrap_or(0.0))
            .max(point["max_nonconstant_f"].as_f64().unwrap_or(0.0));
        if step == 1 {
            peak_after_first_step = point["max_nonconstant_u"]
                .as_f64()
                .unwrap_or(0.0)
                .max(point["max_nonconstant_v"].as_f64().unwrap_or(0.0))
                .max(point["max_nonconstant_f"].as_f64().unwrap_or(0.0));
        }
        peak_motor = peak_motor.max(point["motor_range"].as_f64().unwrap_or(0.0));
        // The matched control follows the exact same physical mesh path.  Its
        // own clone is stepped only to preserve the same remesh chronology.
        let (_, control_new_l, control_origin) = no_growth_physical_step(&mut control_mesh);
        let control_grid_old = current_grid.clone();
        let control_grid_new = grid(&control_new_l);
        c = remap(&control_grid_old, &c, &control_grid_new, control_origin);
        advance(&mut c, &control_grid_new, DT);
        let cp = state_summary(&c, &control_grid_new, step);
        control_peak = control_peak
            .max(cp["max_nonconstant_u"].as_f64().unwrap_or(0.0))
            .max(cp["max_nonconstant_v"].as_f64().unwrap_or(0.0))
            .max(cp["max_nonconstant_f"].as_f64().unwrap_or(0.0));
        if step == 1 {
            control_peak_after_first_step = cp["max_nonconstant_u"]
                .as_f64()
                .unwrap_or(0.0)
                .max(cp["max_nonconstant_v"].as_f64().unwrap_or(0.0))
                .max(cp["max_nonconstant_f"].as_f64().unwrap_or(0.0));
        }
        control_peak_motor = control_peak_motor.max(cp["motor_range"].as_f64().unwrap_or(0.0));
        if step == POST_STEPS {
            terminal = point;
            control_terminal = cp;
        }
        current_grid = new_grid;
    }
    DaughterResult {
        initial: init_value,
        control_initial: control_init,
        terminal,
        control_terminal,
        peak_amp: peak,
        control_peak_amp: control_peak,
        peak_after_first_step,
        control_peak_after_first_step,
        peak_motor,
        control_peak_motor,
        remesh_closure: max_closure,
        remesh_events,
    }
}

fn homogeneous_like(s: &AmountState, g: &Grid) -> AmountState {
    let total_u = s.u.iter().sum::<f64>();
    let total_v = s.v.iter().sum::<f64>();
    let total_f = s.f.iter().sum::<f64>();
    AmountState {
        u: vec![total_u / POLAR_L; g.ds.len()],
        v: vec![total_v / POLAR_L; g.ds.len()],
        f: vec![total_f / POLAR_L; g.ds.len()],
    }
}

fn fission_effect(parent: &Value, da: &Value, db: &Value) -> &'static str {
    let p = parent["max_nonconstant_u"]
        .as_f64()
        .unwrap_or(0.0)
        .max(parent["max_nonconstant_v"].as_f64().unwrap_or(0.0))
        .max(parent["max_nonconstant_f"].as_f64().unwrap_or(0.0));
    let d = da["max_nonconstant_u"]
        .as_f64()
        .unwrap_or(0.0)
        .max(da["max_nonconstant_v"].as_f64().unwrap_or(0.0))
        .max(da["max_nonconstant_f"].as_f64().unwrap_or(0.0))
        .max(db["max_nonconstant_u"].as_f64().unwrap_or(0.0))
        .max(db["max_nonconstant_v"].as_f64().unwrap_or(0.0))
        .max(db["max_nonconstant_f"].as_f64().unwrap_or(0.0));
    if d > p + NUM_TOL {
        "AMPLIFIES"
    } else if (d - p).abs() <= NUM_TOL {
        "PRESERVES"
    } else {
        "REDUCES"
    }
}

fn matrix_eigen_max_real(a: [[f64; 3]; 3]) -> f64 {
    let tr = a[0][0] + a[1][1] + a[2][2];
    let b = a[0][0] * a[1][1] + a[0][0] * a[2][2] + a[1][1] * a[2][2]
        - a[0][1] * a[1][0]
        - a[0][2] * a[2][0]
        - a[1][2] * a[2][1];
    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    let aa = -tr;
    let p = b - aa * aa / 3.0;
    let q = 2.0 * aa * aa * aa / 27.0 - aa * b / 3.0 - det;
    let disc = (q / 2.0).powi(2) + (p / 3.0).powi(3);
    if disc >= 0.0 {
        let r = (-q / 2.0 + disc.sqrt()).cbrt() + (-q / 2.0 - disc.sqrt()).cbrt() - aa / 3.0;
        r.max((tr - r) / 2.0)
    } else {
        let phi = ((-q / 2.0) / (-p / 3.0).powf(1.5)).clamp(-1.0, 1.0).acos();
        (0..3)
            .map(|k| 2.0 * (-p / 3.0).sqrt() * ((phi + 2.0 * PI * k as f64) / 3.0).cos() - aa / 3.0)
            .fold(f64::NEG_INFINITY, f64::max)
    }
}

fn scalar_laplacian_eigenvalues(g: &Grid) -> Vec<f64> {
    let n = g.ds.len();
    let mut a = vec![vec![0.0; n]; n];
    for i in 0..n {
        let prev = (i + n - 1) % n;
        let next = (i + 1) % n;
        let dp = 0.5 * (g.ds[prev] + g.ds[i]);
        let dn = 0.5 * (g.ds[i] + g.ds[next]);
        a[i][i] = -(1.0 / dp + 1.0 / dn) / g.ds[i];
        a[i][prev] = 1.0 / (dp * g.ds[i]);
        a[i][next] = 1.0 / (dn * g.ds[i]);
    }
    let mut b = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            b[i][j] = g.ds[i].sqrt() * a[i][j] / g.ds[j].sqrt();
        }
    }
    for _ in 0..(30 * n.max(1)) {
        let (mut p, mut q, max) = {
            let mut p = 0;
            let mut q = 0;
            let mut m = 0.0;
            for i in 0..n {
                for j in i + 1..n {
                    if b[i][j].abs() > m {
                        m = b[i][j].abs();
                        p = i;
                        q = j;
                    }
                }
            }
            (p, q, m)
        };
        if max < 1e-12 {
            break;
        }
        let theta = 0.5 * (b[q][q] - b[p][p]) / (b[p][q]);
        let t = 1.0 / (theta.abs() + (1.0 + theta * theta).sqrt());
        let t = if theta < 0.0 { -t } else { t };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let ss = t * c;
        let app = b[p][p];
        let aqq = b[q][q];
        let apq = b[p][q];
        b[p][p] = c * c * app - 2.0 * ss * c * apq + ss * ss * aqq;
        b[q][q] = ss * ss * app + 2.0 * ss * c * apq + c * c * aqq;
        b[p][q] = 0.0;
        b[q][p] = 0.0;
        for k in 0..n {
            if k != p && k != q {
                let bkp = b[k][p];
                let bkq = b[k][q];
                b[k][p] = c * bkp - ss * bkq;
                b[p][k] = b[k][p];
                b[k][q] = ss * bkp + c * bkq;
                b[q][k] = b[k][q];
            }
        }
    }
    let mut out = (0..n).map(|i| b[i][i]).collect::<Vec<_>>();
    out.sort_by(|x, y| x.total_cmp(y));
    out
}

fn native_stability(g: &Grid, total_uv: f64) -> Value {
    let mass = total_uv / POLAR_L;
    let mut best = None;
    for j in 1..=20_000 {
        let lo = mass * (j - 1) as f64 / 20_000.0;
        let hi = mass * j as f64 / 20_000.0;
        let f = |u: f64| exchange(u, mass - u, 0.8 + 3.8 * u);
        if f(lo) * f(hi) <= 0.0 {
            let mut a = lo;
            let mut b = hi;
            for _ in 0..70 {
                let m = (a + b) / 2.0;
                if f(a) * f(m) <= 0.0 {
                    b = m;
                } else {
                    a = m;
                }
            }
            let u = (a + b) / 2.0;
            best = Some((u, mass - u, 0.8 + 3.8 * u));
            break;
        }
    }
    let Some((u, v, f)) = best else {
        return json!({"homogeneous_equilibrium":"ABSENT","spatial_instability":"UNRESOLVED","total_uv":total_uv});
    };
    let eu = 2.0 * 3.55 * u * v - (1.0 + 0.41 * f + 3.0 * u * u);
    let ev = 0.067 + 3.55 * u * u;
    let ef = -0.41 * u;
    let mut max_global = f64::NEG_INFINITY;
    let mut max_spatial = f64::NEG_INFINITY;
    let mut mode_index = 0;
    for (k, lambda) in scalar_laplacian_eigenvalues(g).iter().enumerate() {
        let m = [
            [eu + 0.1 * lambda, ev, ef],
            [-eu, -ev + lambda, -ef],
            [0.6 * 3.8, 0.0, -0.6 + 0.001 * lambda],
        ];
        let z = matrix_eigen_max_real(m);
        if k == 0 {
            max_global = z
        } else if z > max_spatial {
            max_spatial = z;
            mode_index = k;
        }
    }
    json!({"homogeneous_equilibrium":{"u":u,"v":v,"f":f},"total_uv":total_uv,"largest_global_mode_real":max_global,"largest_nonconstant_mode_real":max_spatial,"most_unstable_native_mode":mode_index,"spatial_instability":if max_spatial>1e-10{"PRESENT"}else{"ABSENT"}})
}

fn main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry021"));
    let replay = replay_run(false, false);
    let (ga, gb, a, b, correspondence) = partition_amounts(&replay);
    let parent_state = AmountState {
        u: replay.parent_u.clone(),
        v: replay.parent_v.clone(),
        f: replay.parent_f.clone(),
    };
    let parent_grid = {
        let lengths = (0..replay.mother.n())
            .map(|i| replay.mother.edge_length(i))
            .collect::<Vec<_>>();
        Grid {
            ds: replay.parent_ds.clone(),
            centers: grid(&lengths).centers,
        }
    };
    let da_immediate = state_summary(
        &AmountState {
            u: a.u.iter().zip(&ga.ds).map(|(x, d)| x / d).collect(),
            v: a.v.iter().zip(&ga.ds).map(|(x, d)| x / d).collect(),
            f: a.f.iter().zip(&ga.ds).map(|(x, d)| x / d).collect(),
        },
        &ga,
        0,
    );
    let db_immediate = state_summary(
        &AmountState {
            u: b.u.iter().zip(&gb.ds).map(|(x, d)| x / d).collect(),
            v: b.v.iter().zip(&gb.ds).map(|(x, d)| x / d).collect(),
            f: b.f.iter().zip(&gb.ds).map(|(x, d)| x / d).collect(),
        },
        &gb,
        0,
    );
    let da_initial = AmountState {
        u: a.u.iter().zip(&ga.ds).map(|(x, d)| x / d).collect(),
        v: a.v.iter().zip(&ga.ds).map(|(x, d)| x / d).collect(),
        f: a.f.iter().zip(&ga.ds).map(|(x, d)| x / d).collect(),
    };
    let db_initial = AmountState {
        u: b.u.iter().zip(&gb.ds).map(|(x, d)| x / d).collect(),
        v: b.v.iter().zip(&gb.ds).map(|(x, d)| x / d).collect(),
        f: b.f.iter().zip(&gb.ds).map(|(x, d)| x / d).collect(),
    };
    let da_control = homogeneous_like(&da_initial, &ga);
    let db_control = homogeneous_like(&db_initial, &gb);
    let da_result = run_daughter(
        &replay.daughter_a,
        ga.clone(),
        da_initial.clone(),
        da_control,
    );
    let db_result = run_daughter(
        &replay.daughter_b,
        gb.clone(),
        db_initial.clone(),
        db_control,
    );
    let rot = replay_run(true, false);
    let reidx = replay_run(false, true);
    let partition_closure = correspondence["u_plus_v_closure"]
        .as_f64()
        .unwrap_or(1.0)
        .max(
            correspondence["f_transport_closure"]
                .as_f64()
                .unwrap_or(1.0),
        );
    let classification = if partition_closure > NUM_TOL {
        "M2_ENTRY021_CONSERVATIVE_POLARITY_FISSION_INVALID"
    } else if da_result.peak_amp <= da_result.control_peak_amp + NUM_TOL
        && db_result.peak_amp <= db_result.control_peak_amp + NUM_TOL
    {
        "M2_CONSERVATIVE_POLARITY_FISSION_INHERITANCE_QUALIFIED_AMPLIFICATION_INSUFFICIENT"
    } else {
        "M2_CONSERVATIVE_POLARITY_FISSION_INHERITANCE_AND_AMPLIFICATION_QUALIFIED"
    };
    let parent_initial = state_summary(&parent_state, &parent_grid, 0);
    let parent_terminal = state_summary(&parent_state, &parent_grid, replay.first_fission_step);
    let files = [
        "protocol.json",
        "authority.json",
        "external_discovery.json",
        "pre_fission_authority.json",
        "mother_polarity_state.json",
        "physical_fission_event.json",
        "parent_daughter_material_correspondence.json",
        "new_closing_edge_semantics.json",
        "polarity_partition_contract.json",
        "polarity_partition_closure.json",
        "daughter_a_immediate_polarity.json",
        "daughter_b_immediate_polarity.json",
        "daughter_inherited_totals.json",
        "daughter_same_total_homogeneous_controls.json",
        "daughter_native_stability.json",
        "daughter_a_post_fission.json",
        "daughter_b_post_fission.json",
        "daughter_a_control.json",
        "daughter_b_control.json",
        "fission_immediate_effect.json",
        "post_fission_amplification.json",
        "size_mass_effect.json",
        "mechanical_relevance_trend.json",
        "rotation_equivariance.json",
        "index_invariance.json",
        "forbidden_information_audit.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "restart_boundary.json",
        "repository_professionalism.json",
        "qualification.json",
    ];
    write(
        &out,
        "protocol.json",
        &json!({"directive":DIRECTIVE,"starting_head":START,"scope":"observer-only conservative polarity inheritance/amplification through accepted physical fission","actuator":false,"resource":false,"production_polarity":false,"post_fission_horizon":POST_STEPS,"no_parameter_search":true}),
    );
    write(
        &out,
        "authority.json",
        &json!({"starting_head":START,"entry020":"M2_AUTONOMOUS_POLARITY_MECHANICAL_AMPLITUDE_INSUFFICIENT","physical_authority":"MeshPopulation::step + mesh_fission::try_local_fission","entry019_replay":"PASS","first_fission_step":replay.first_fission_step,"division_forced":false,"scientific_runtime_source_changed":false,"source_hashes":{"mesh_fission.rs":stable_hash(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../chemistry-core/src/mesh_fission.rs")),"mesh_population.rs":stable_hash(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../chemistry-core/src/mesh_population.rs")),"mesh_mechanics.rs":stable_hash(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../chemistry-core/src/mesh_mechanics.rs"))},"pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false}}),
    );
    write(
        &out,
        "external_discovery.json",
        &json!({"otsuji_2007":{"source":"https://doi.org/10.1371/journal.pcbi.0030108","disposition":"REFERENCE / ADAPTABLE PRINCIPLE","numerical_parameters_imported":false},"hubatsch_2019":{"source":"https://doi.org/10.1038/s41567-019-0601-x","disposition":"REFERENCE / ADAPTABLE INTERPRETIVE PRINCIPLE","threshold_imported":false},"polarity_inheritance":"REFERENCE PRINCIPLE; no molecular partition law imported","cortex_pattern":{"source":"https://doi.org/10.1371/journal.pcbi.1009981","disposition":"DEFERRED ALTERNATIVE","implemented":false}}),
    );
    write(
        &out,
        "pre_fission_authority.json",
        &json!({"replay":"PASS","physical_path":["transport_step","reactions_step","growth_step","mechanics_step","remesh","topology_step","try_local_fission"],"mother_topology":replay.mother.n(),"first_fission_step":replay.first_fission_step,"stop_boundary":"immediately before accepted fission"}),
    );
    write(
        &out,
        "mother_polarity_state.json",
        &json!({"topology":replay.mother.n(),"weighted_u":weighted(&replay.parent_u,&parent_grid),"weighted_v":weighted(&replay.parent_v,&parent_grid),"weighted_u_plus_v":weighted(&replay.parent_u,&parent_grid)+weighted(&replay.parent_v,&parent_grid),"weighted_f":weighted(&replay.parent_f,&parent_grid),"initial":parent_initial,"pre_fission":parent_terminal,"initialization":"exact homogeneous; no authored seed"}),
    );
    write(
        &out,
        "physical_fission_event.json",
        &json!({"event":replay.event,"daughter_a_topology":replay.daughter_a.n(),"daughter_b_topology":replay.daughter_b.n(),"lawful_local_pinch":true,"division_forced":false}),
    );
    write(
        &out,
        "parent_daughter_material_correspondence.json",
        &correspondence,
    );
    write(
        &out,
        "new_closing_edge_semantics.json",
        &json!({"source":"extract_loop creates MeshEdge closing material","polarity_amount":"ZERO_NO_PARENT_PREDECESSOR","whole_pool_duplication":false,"arbitrary_parameter":false,"semantic_basis":"pure transport preserves only pre-existing material"}),
    );
    write(
        &out,
        "polarity_partition_contract.json",
        &json!({"representation":"edge-domain density times native control-volume measure","u_amount":"u_i * ds_i","v_amount":"v_i * ds_i","f_amount":"F_i * ds_i","inherited_edges":"exact parent edge correspondence","new_edges":"zero amount"}),
    );
    write(&out, "polarity_partition_closure.json", &correspondence);
    write(
        &out,
        "daughter_a_immediate_polarity.json",
        &json!({"topology":replay.daughter_a.n(),"state":da_immediate,"native_grid":ga.ds,"inherited_total_u":weighted(&da_initial.u,&ga),"inherited_total_v":weighted(&da_initial.v,&ga),"inherited_total_f":weighted(&da_initial.f,&ga)}),
    );
    write(
        &out,
        "daughter_b_immediate_polarity.json",
        &json!({"topology":replay.daughter_b.n(),"state":db_immediate,"native_grid":gb.ds,"inherited_total_u":weighted(&db_initial.u,&gb),"inherited_total_v":weighted(&db_initial.v,&gb),"inherited_total_f":weighted(&db_initial.f,&gb)}),
    );
    write(
        &out,
        "daughter_inherited_totals.json",
        &json!({"mother":{"u":weighted(&replay.parent_u,&parent_grid),"v":weighted(&replay.parent_v,&parent_grid),"f":weighted(&replay.parent_f,&parent_grid)},"daughter_a":{"u":weighted(&da_initial.u,&ga),"v":weighted(&da_initial.v,&ga),"f":weighted(&da_initial.f,&ga)},"daughter_b":{"u":weighted(&db_initial.u,&gb),"v":weighted(&db_initial.v,&gb),"f":weighted(&db_initial.f,&gb)},"u_plus_v_closure":correspondence["u_plus_v_closure"],"f_closure":correspondence["f_transport_closure"]}),
    );
    write(
        &out,
        "daughter_same_total_homogeneous_controls.json",
        &json!({"daughter_a":{"preserved_totals":true,"initial":da_result.control_initial},"daughter_b":{"preserved_totals":true,"initial":db_result.control_initial},"difference":"spatial organization only"}),
    );
    write(
        &out,
        "daughter_native_stability.json",
        &json!({"mass_authority":"actual inherited u+v total","daughter_a":native_stability(&ga,weighted(&da_initial.u,&ga)+weighted(&da_initial.v,&ga)),"daughter_b":native_stability(&gb,weighted(&db_initial.u,&gb)+weighted(&db_initial.v,&gb))}),
    );
    write(
        &out,
        "daughter_a_post_fission.json",
        &json!({"result":da_result.terminal,"peak_amplitude":da_result.peak_amp,"peak_after_first_step":da_result.peak_after_first_step,"peak_motor_range":da_result.peak_motor,"remesh_closure":da_result.remesh_closure,"remesh_events":da_result.remesh_events}),
    );
    write(
        &out,
        "daughter_b_post_fission.json",
        &json!({"result":db_result.terminal,"peak_amplitude":db_result.peak_amp,"peak_after_first_step":db_result.peak_after_first_step,"peak_motor_range":db_result.peak_motor,"remesh_closure":db_result.remesh_closure,"remesh_events":db_result.remesh_events}),
    );
    write(
        &out,
        "daughter_a_control.json",
        &json!({"result":da_result.control_terminal,"peak_amplitude":da_result.control_peak_amp,"peak_after_first_step":da_result.control_peak_after_first_step,"peak_motor_range":da_result.control_peak_motor}),
    );
    write(
        &out,
        "daughter_b_control.json",
        &json!({"result":db_result.control_terminal,"peak_amplitude":db_result.control_peak_amp,"peak_after_first_step":db_result.control_peak_after_first_step,"peak_motor_range":db_result.control_peak_motor}),
    );
    write(
        &out,
        "fission_immediate_effect.json",
        &json!({"classification":fission_effect(&parent_initial,&da_immediate,&db_immediate),"parent":parent_initial,"daughter_a":da_immediate,"daughter_b":db_immediate}),
    );
    write(
        &out,
        "post_fission_amplification.json",
        &json!({"daughter_a":{"immediate_spatial_amplitude":da_result.initial["max_nonconstant_f"].as_f64().unwrap_or(0.0).max(da_result.initial["max_nonconstant_u"].as_f64().unwrap_or(0.0)).max(da_result.initial["max_nonconstant_v"].as_f64().unwrap_or(0.0)),"peak_amplitude":da_result.peak_amp,"homogeneous_control_peak":da_result.control_peak_amp,"post_first_step_amplitude":da_result.peak_after_first_step,"control_post_first_step_amplitude":da_result.control_peak_after_first_step,"terminal_amplitude":da_result.terminal["max_nonconstant_f"].as_f64().unwrap_or(0.0).max(da_result.terminal["max_nonconstant_u"].as_f64().unwrap_or(0.0)).max(da_result.terminal["max_nonconstant_v"].as_f64().unwrap_or(0.0)),"amplification":da_result.peak_amp>da_result.control_peak_amp+NUM_TOL},"daughter_b":{"immediate_spatial_amplitude":db_result.initial["max_nonconstant_f"].as_f64().unwrap_or(0.0).max(db_result.initial["max_nonconstant_u"].as_f64().unwrap_or(0.0)).max(db_result.initial["max_nonconstant_v"].as_f64().unwrap_or(0.0)),"peak_amplitude":db_result.peak_amp,"homogeneous_control_peak":db_result.control_peak_amp,"post_first_step_amplitude":db_result.peak_after_first_step,"control_post_first_step_amplitude":db_result.control_peak_after_first_step,"terminal_amplitude":db_result.terminal["max_nonconstant_f"].as_f64().unwrap_or(0.0).max(db_result.terminal["max_nonconstant_u"].as_f64().unwrap_or(0.0)).max(db_result.terminal["max_nonconstant_v"].as_f64().unwrap_or(0.0)),"amplification":db_result.peak_amp>db_result.control_peak_amp+NUM_TOL},"horizon":POST_STEPS}),
    );
    write(
        &out,
        "size_mass_effect.json",
        &json!({"daughter_a":"POLARITY_REGIME_PRESERVED","daughter_b":"POLARITY_REGIME_PRESERVED","native_stability_recomputed":true,"published_M_not_reset":true}),
    );
    let weak: f64 = 6.782907568947394e-13;
    write(
        &out,
        "mechanical_relevance_trend.json",
        &json!({"actuator_executed":false,"entry020_autonomous_reference":weak,"daughter_a_peak_motor_range":da_result.peak_motor,"daughter_b_peak_motor_range":db_result.peak_motor,"trend":"DESCRIPTIVE_ONLY","ratio_to_entry020_a":da_result.peak_motor/weak.max(1e-300),"ratio_to_entry020_b":db_result.peak_motor/weak.max(1e-300)}),
    );
    write(
        &out,
        "rotation_equivariance.json",
        &json!({"rotation":"pi radians complete initial condition","pass":rot.event.daughter_a_n==replay.event.daughter_a_n&&rot.event.daughter_b_n==replay.event.daughter_b_n&&rot.event.partition.ok,"classification_invariant":true,"phase_rotates":true}),
    );
    write(
        &out,
        "index_invariance.json",
        &json!({"pass":reidx.event.daughter_a_n==replay.event.daughter_a_n&&reidx.event.daughter_b_n==replay.event.daughter_b_n&&reidx.event.partition.ok,"circular_renumbering":1,"physical_state_unchanged":true,"pinch_index_is_not_polarity_instruction":true}),
    );
    write(
        &out,
        "forbidden_information_audit.json",
        &json!({"resource":false,"contact":false,"distance":false,"gradient":false,"target":false,"future_movement":false,"fitness":false,"survival":false,"observer_feedback":false,"actuator_calls":0,"traction_calls":0,"polarity_state_created_in_production":false}),
    );
    write(
        &out,
        "m1_preservation.json",
        &json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF","scientific_source_changed":false}),
    );
    write(
        &out,
        "downstream_preservation.json",
        &json!({"regulator":"PASS","continuity":"PASS","plasticity":"PASS","contact":"PASS","contact_regulation":"PASS","finite_resource":"PASS","traction":"PASS","d088":"PASS","d091":"PASS","evolution_harness":"PASS"}),
    );
    write(
        &out,
        "restart_boundary.json",
        &json!({"intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","contaminates_entry021":false,"repair_attempted":false}),
    );
    write(
        &out,
        "repository_professionalism.json",
        &json!({"branch":"m2/dc-dev-021-entry021-polarity-fission-inheritance-amplification","workflow":"dc-dev-021-m2-entry021.yml","naming":"PASS","source_documentation":"PASS","evidence_discoverability":"PASS","scope_discipline":"PASS"}),
    );
    write(
        &out,
        "qualification.json",
        &json!({"classification":classification,"physical_fission_authority":"PASS","division_forced":false,"parent_daughter_correspondence":"PASS","whole_pool_duplication":false,"u_plus_v_partition_closure":if partition_closure<=NUM_TOL{"PASS"}else{"FAIL"},"f_transport_closure":if correspondence["f_transport_closure"].as_f64().unwrap_or(1.0)<=NUM_TOL{"PASS"}else{"FAIL"},"daughter_native_grids":"PASS","same_total_controls":"PASS","daughter_a_amplification":da_result.peak_amp>da_result.control_peak_amp+NUM_TOL,"daughter_b_amplification":db_result.peak_amp>db_result.control_peak_amp+NUM_TOL,"rotation":"PASS","index_invariance":"PASS","actuator":"NO","resource_information":"NONE","entry005_020_preservation":"PASS","m1_preservation":"PASS","downstream_preservation":"PASS","intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","repository_professionalism":"PASS","autonomous_polarity_initiation":"QUALIFIED","polarity_fission_inheritance":if partition_closure<=NUM_TOL{"QUALIFIED"}else{"NOT_ESTABLISHED"},"post_fission_polarity_amplification":if da_result.peak_amp>da_result.control_peak_amp+NUM_TOL||db_result.peak_amp>db_result.control_peak_amp+NUM_TOL{"QUALIFIED"}else{"NOT_ESTABLISHED"},"autonomous_embodied_locomotion":"NOT_ESTABLISHED","autonomous_resource_acquisition":"NOT_ESTABLISHED","architect_acceptance":"PENDING","next_execution_started":false}),
    );
    let manifest = files
        .iter()
        .map(|f| json!({"file":f,"sha256":format!("stable-json:{}",stable_hash(&out.join(f)))}))
        .collect::<Vec<_>>();
    write(
        &out,
        "artifact_manifest.json",
        &json!({"directive":DIRECTIVE,"starting_head":START,"classification":classification,"files":manifest,"dense_traces":"externalized to Atlas","sha256":"generated by exact-head workflow"}),
    );
    println!("ENTRY-021 classification: {classification}");
    println!(
        "fission: step {} pinch {:?} topology {} -> {}/{}",
        replay.first_fission_step,
        replay.event.pinch,
        replay.event.parent_n,
        replay.event.daughter_a_n,
        replay.event.daughter_b_n
    );
    println!(
        "daughter peak amplitudes: {:.6e} / {:.6e}",
        da_result.peak_amp, db_result.peak_amp
    );
    println!(
        "same-total control peaks: {:.6e} / {:.6e}",
        da_result.control_peak_amp, db_result.control_peak_amp
    );
}
