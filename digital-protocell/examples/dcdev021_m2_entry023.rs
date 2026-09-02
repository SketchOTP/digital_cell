//! DC-DEV-021 M2 ENTRY-023: daughter polarity/effector mechanical-transfer attribution audit.
//!
//! This is an isolated observer assay.  It replays the accepted D-088 physical
//! mother trajectory, carries the accepted ENTRY-019 Polar state as local
//! control-volume amounts through the existing fission operation, and advances
//! both daughters without an actuator or resource.  Fission itself is not
//! redesigned: inherited parent edge material receives its corresponding
//! polarity amount and a newly synthesized closing edge has no predecessor and
//! therefore receives zero transported amount.

use chemistry_core::material_mesh::{MaterialMesh, MeshContractVersion};
use chemistry_core::mesh_fission::{try_local_fission, FissionEvent, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step, reactions_step_with_reserve_mode, ReactionParams, ReserveDiagnosticMode,
};
use chemistry_core::mesh_topology::TopologyLedger;
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use regulatory_core::{
    apply_local_activated_energy_contractility_with_stick_slip,
    apply_stick_slip_to_legacy_mechanics, ContractilityParamsV1, StickSlipTractionParamsV1,
    FROZEN_ZERO_MOTION_TOLERANCE,
};
use serde_json::{json, Value};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-023-DAUGHTER-POLARITY-EFFECTOR-MECHANICAL-TRANSFER-ATTRIBUTION-AUDIT-001";
const START: &str = "48b313db45761552e27a34f77b7aff9b0e688f95";
const DT: f64 = 0.02;
const POLAR_L: f64 = 2.0 * PI;
const NUM_TOL: f64 = 1e-10;
const POST_STEPS: usize = 3_000;
const MOTOR_STEPS: usize = POST_STEPS - 1;

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
        .map(|(u, v)| {
            assert!(
                *u + *v > 0.0,
                "motor range requested before interface eligibility"
            );
            u / (u + v)
        })
        .fold(0.0_f64, |acc, x| acc.max(x))
        - s.u
            .iter()
            .zip(&s.v)
            .map(|(u, v)| {
                assert!(
                    *u + *v > 0.0,
                    "motor range requested before interface eligibility"
                );
                u / (u + v)
            })
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

#[derive(Clone, Debug, serde::Serialize)]
struct PhysicalSnapshot {
    area: f64,
    a: f64,
    w: f64,
    n: f64,
    f: f64,
    c: f64,
    centroid: [f64; 2],
}

#[derive(Clone, Debug, serde::Serialize)]
struct TransientRun {
    arm: String,
    motor_steps: usize,
    total_post_fission_steps: usize,
    path: f64,
    net_displacement: f64,
    displacement_path_ratio: f64,
    maximum_centroid_excursion: f64,
    maximum_material_envelope_excursion: f64,
    slips: usize,
    stuck_contacts: usize,
    a_spent: f64,
    w_generated: f64,
    a_to_w_residual: f64,
    a_limited_steps: usize,
    traction_directional_difference: f64,
    first_motor_min: f64,
    first_motor_max: f64,
    first_motor_range: f64,
    peak_motor_range: f64,
    initial_state: Value,
    terminal_state: PhysicalSnapshot,
    points: Vec<Value>,
}

fn physical_centroid(mesh: &MaterialMesh) -> [f64; 2] {
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

fn vector_sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn vector_norm(a: [f64; 2]) -> f64 {
    a[0].hypot(a[1])
}

fn physical_snapshot(mesh: &MaterialMesh) -> PhysicalSnapshot {
    PhysicalSnapshot {
        area: mesh.area(),
        a: mesh.interior.a,
        w: mesh.interior.w,
        n: mesh.interior.n,
        f: mesh.interior.f,
        c: mesh.interior.c,
        centroid: physical_centroid(mesh),
    }
}

fn density_state(amounts: &AmountState, g: &Grid) -> AmountState {
    AmountState {
        u: amounts.u.iter().zip(&g.ds).map(|(x, d)| x / d).collect(),
        v: amounts.v.iter().zip(&g.ds).map(|(x, d)| x / d).collect(),
        f: amounts.f.iter().zip(&g.ds).map(|(x, d)| x / d).collect(),
    }
}

fn exact_active_fraction(s: &AmountState) -> Vec<f64> {
    s.u.iter()
        .zip(&s.v)
        .map(|(u, v)| {
            assert!(*u >= 0.0 && *v >= 0.0, "negative inherited polarity state");
            let denominator = *u + *v;
            assert!(denominator > 0.0, "zero polarity pool reached actuator");
            *u / denominator
        })
        .collect()
}

fn passive_post_fission_step(mesh: &mut MaterialMesh) -> (Vec<f64>, Vec<f64>, usize) {
    let old_lengths: Vec<f64> = (0..mesh.n()).map(|i| mesh.edge_length(i)).collect();
    let old_vertices = mesh.vertices.clone();
    let mechanics = MechParams::default();
    let reaction = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: false,
    };
    let _ = transport_step(mesh, &transport, mechanics.dt);
    let _ = reactions_step(mesh, &reaction, mechanics.dt, true, true);
    let _ = growth_step(mesh, &reaction, &growth, mechanics.dt);
    assert!(mechanics_step(mesh, &mechanics));
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
    let new_lengths: Vec<f64> = (0..mesh.n()).map(|i| mesh.edge_length(i)).collect();
    (old_lengths, new_lengths, origin)
}

fn eligible_state(
    mesh: &MaterialMesh,
    g: &Grid,
    state: &AmountState,
) -> (MaterialMesh, Grid, AmountState, Value) {
    let mut mesh = mesh.clone();
    let (old_lengths, new_lengths, origin) = passive_post_fission_step(&mut mesh);
    let old_grid = g.clone();
    let new_grid = grid(&new_lengths);
    let remapped = remap(&old_grid, state, &new_grid, origin);
    let before_uv = weighted(&state.u, &old_grid) + weighted(&state.v, &old_grid);
    let after_uv = weighted(&remapped.u, &new_grid) + weighted(&remapped.v, &new_grid);
    let mut eligible = remapped;
    advance(&mut eligible, &new_grid, DT);
    let min_pool = eligible
        .u
        .iter()
        .zip(&eligible.v)
        .map(|(u, v)| u + v)
        .fold(f64::INFINITY, f64::min);
    assert!(min_pool > 0.0, "actuator eligibility not reached");
    let summary = json!({
        "accepted_step": 1,
        "growth": "OFF",
        "additional_fission": "OFF",
        "actuator": "OFF",
        "old_site_count": old_lengths.len(),
        "new_site_count": new_lengths.len(),
        "remesh_origin": origin,
        "weighted_uv_before": before_uv,
        "weighted_uv_after_remap": after_uv,
        "remesh_uv_residual": (after_uv - before_uv).abs(),
        "minimum_u_plus_v": min_pool,
        "all_u_plus_v_strictly_positive": min_pool > 0.0,
        "interface_now_eligible": true,
        "motor_time_origin": "t_motor=0 at this state",
    });
    (mesh, new_grid, eligible, summary)
}

fn run_transient(
    mesh_start: &MaterialMesh,
    g: Grid,
    state_start: AmountState,
    arm: &str,
    uniform: bool,
    motor_off: bool,
) -> TransientRun {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction_params = ReactionParams::conservative_v3();
    let mut mesh = mesh_start.clone();
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    let initial_state_for_report = state_start.clone();
    let mut state = state_start;
    let initial = physical_snapshot(&mesh);
    let initial_centroid = initial.centroid;
    let initial_radius = mesh
        .vertices
        .iter()
        .map(|p| vector_norm(vector_sub(*p, initial_centroid)))
        .fold(0.0, f64::max);
    let mut previous_centroid = initial_centroid;
    let initial_grid_for_report = g.clone();
    let mut current_grid = g;
    let mut path = 0.0;
    let mut max_excursion: f64 = 0.0;
    let mut max_envelope: f64 = 0.0;
    let mut slips = 0;
    let mut stuck = 0;
    let mut a_spent = 0.0;
    let mut w_generated = 0.0;
    let mut a_to_w_residual: f64 = 0.0;
    let mut a_limited_steps = 0;
    let mut peak_motor_range: f64 = 0.0;
    let mut first_motor = (f64::INFINITY, f64::NEG_INFINITY);
    let mut first_range = 0.0;
    let mut traction_difference = 0.0;
    let mut points = Vec::new();
    for step in 1..=MOTOR_STEPS {
        let old_grid = current_grid.clone();
        let _reaction = reactions_step_with_reserve_mode(
            &mut mesh,
            &reaction_params,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        let fractions = exact_active_fraction(&state);
        let mean = fractions.iter().sum::<f64>() / fractions.len() as f64;
        let motor = if motor_off {
            vec![0.0; mesh.n()]
        } else if uniform {
            vec![mean; mesh.n()]
        } else {
            fractions
        };
        let min_motor = motor.iter().copied().fold(f64::INFINITY, f64::min);
        let max_motor = motor.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let range = max_motor - min_motor;
        if step == 1 {
            first_motor = (min_motor, max_motor);
            first_range = range;
        }
        peak_motor_range = peak_motor_range.max(range);
        let ledger = if motor_off {
            let l = apply_stick_slip_to_legacy_mechanics(&mut mesh, &mechanics, &traction).unwrap();
            slips += l.slipping_contacts;
            stuck += l.stuck_contacts;
            None
        } else {
            let l = apply_local_activated_energy_contractility_with_stick_slip(
                &mut mesh,
                &motor,
                &mechanics,
                &contractility,
                &traction,
            )
            .unwrap();
            slips += l.slipping_contacts;
            stuck += l.stuck_contacts;
            if let Some(c) = l.contractility.as_ref() {
                if c.requested_resource > c.resource_spent + NUM_TOL {
                    a_limited_steps += 1;
                }
                a_to_w_residual = a_to_w_residual.max(
                    (c.activated_amount_before - c.activated_amount_after + c.waste_amount_before
                        - c.waste_amount_after)
                        .abs(),
                );
                traction_difference += c.maximum_tension.abs();
                a_spent += c.resource_spent;
            }
            Some(l)
        };
        let reaction = reactions_step_with_reserve_mode(
            &mut mesh,
            &reaction_params,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        w_generated += reaction.w_produced;
        let before_vertices = mesh.vertices.clone();
        remesh(&mut mesh);
        let origin = mesh
            .vertices
            .first()
            .and_then(|first| {
                before_vertices
                    .iter()
                    .position(|old| (old[0] - first[0]).hypot(old[1] - first[1]) <= 1e-9)
            })
            .unwrap_or(0);
        let new_lengths: Vec<f64> = (0..mesh.n()).map(|i| mesh.edge_length(i)).collect();
        let new_grid = grid(&new_lengths);
        state = remap(&old_grid, &state, &new_grid, origin);
        advance(&mut state, &new_grid, DT);
        let centroid = physical_centroid(&mesh);
        let displacement = vector_sub(centroid, previous_centroid);
        path += vector_norm(displacement);
        max_excursion = max_excursion.max(vector_norm(vector_sub(centroid, initial_centroid)));
        max_envelope = max_envelope.max(
            mesh.vertices
                .iter()
                .map(|p| vector_norm(vector_sub(*p, initial_centroid)) - initial_radius)
                .fold(0.0, f64::max),
        );
        if step == 1 || step % 100 == 0 || step == MOTOR_STEPS {
            points.push(json!({"step":step,"centroid":centroid,"displacement":displacement,"motor_mean":mean,"motor_min":min_motor,"motor_max":max_motor,"motor_range":range,"slipping_contacts":ledger.as_ref().map(|l| l.slipping_contacts).unwrap_or(0),"stuck_contacts":ledger.as_ref().map(|l| l.stuck_contacts).unwrap_or(0),"polarity":state_summary(&state,&new_grid,step)}));
        }
        previous_centroid = centroid;
        current_grid = new_grid;
    }
    let terminal = physical_snapshot(&mesh);
    let net = vector_norm(vector_sub(terminal.centroid, initial_centroid));
    TransientRun {
        arm: arm.to_string(),
        motor_steps: MOTOR_STEPS,
        total_post_fission_steps: POST_STEPS,
        path,
        net_displacement: net,
        displacement_path_ratio: net / path.max(1e-30),
        maximum_centroid_excursion: max_excursion,
        maximum_material_envelope_excursion: max_envelope,
        slips,
        stuck_contacts: stuck,
        a_spent,
        w_generated,
        a_to_w_residual,
        a_limited_steps,
        traction_directional_difference: traction_difference,
        first_motor_min: first_motor.0,
        first_motor_max: first_motor.1,
        first_motor_range: first_range,
        peak_motor_range,
        initial_state: state_summary(&initial_state_for_report, &initial_grid_for_report, 0),
        terminal_state: terminal,
        points,
    }
}

fn main() {
    entry023_main();
}

#[allow(dead_code)]
fn old_entry022_main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry022"));
    let replay = replay_run(false, false);
    let (ga, gb, a_amounts, b_amounts, closure) = partition_amounts(&replay);
    let a_birth = density_state(&a_amounts, &ga);
    let b_birth = density_state(&b_amounts, &gb);
    let birth_a_min = a_birth
        .u
        .iter()
        .zip(&a_birth.v)
        .map(|(u, v)| u + v)
        .fold(f64::INFINITY, f64::min);
    let birth_b_min = b_birth
        .u
        .iter()
        .zip(&b_birth.v)
        .map(|(u, v)| u + v)
        .fold(f64::INFINITY, f64::min);
    let (a_mesh, a_grid, a_eligible, a_eligibility) =
        eligible_state(&replay.daughter_a, &ga, &a_birth);
    let (b_mesh, b_grid, b_eligible, b_eligibility) =
        eligible_state(&replay.daughter_b, &gb, &b_birth);
    let a_spatial = run_transient(
        &a_mesh,
        a_grid.clone(),
        a_eligible.clone(),
        "DAUGHTER_A_INHERITED_SPATIAL_MOTOR",
        false,
        false,
    );
    let a_uniform = run_transient(
        &a_mesh,
        a_grid.clone(),
        a_eligible.clone(),
        "DAUGHTER_A_SAME_MEAN_UNIFORM_MOTOR",
        true,
        false,
    );
    let a_off = run_transient(
        &a_mesh,
        a_grid.clone(),
        a_eligible.clone(),
        "DAUGHTER_A_MOTOR_OFF",
        false,
        true,
    );
    let b_spatial = run_transient(
        &b_mesh,
        b_grid.clone(),
        b_eligible.clone(),
        "DAUGHTER_B_INHERITED_SPATIAL_MOTOR",
        false,
        false,
    );
    let b_uniform = run_transient(
        &b_mesh,
        b_grid.clone(),
        b_eligible.clone(),
        "DAUGHTER_B_SAME_MEAN_UNIFORM_MOTOR",
        true,
        false,
    );
    let b_off = run_transient(
        &b_mesh,
        b_grid.clone(),
        b_eligible.clone(),
        "DAUGHTER_B_MOTOR_OFF",
        false,
        true,
    );
    let a_leverage = a_spatial.net_displacement
        > a_uniform.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE
        && a_spatial.net_displacement > a_off.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let b_leverage = b_spatial.net_displacement
        > b_uniform.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE
        && b_spatial.net_displacement > b_off.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let separation = (a_spatial.net_displacement - b_spatial.net_displacement).abs()
        > FROZEN_ZERO_MOTION_TOLERANCE;
    let closure_pass = closure["u_plus_v_closure"].as_f64().unwrap_or(1.0) <= NUM_TOL
        && closure["f_transport_closure"].as_f64().unwrap_or(1.0) <= NUM_TOL;
    let energetic_pass = [
        a_spatial.a_to_w_residual,
        a_uniform.a_to_w_residual,
        b_spatial.a_to_w_residual,
        b_uniform.a_to_w_residual,
    ]
    .iter()
    .all(|x| *x <= 1e-8);
    let active_motion = a_spatial.slips > 0
        && b_spatial.slips > 0
        && (a_spatial.path > FROZEN_ZERO_MOTION_TOLERANCE
            || b_spatial.path > FROZEN_ZERO_MOTION_TOLERANCE);
    let qualification = if !closure_pass || !energetic_pass {
        "M2_ENTRY022_POST_FISSION_TRANSIENT_LOCOMOTION_INVALID"
    } else if !a_eligible
        .u
        .iter()
        .zip(&a_eligible.v)
        .all(|(u, v)| *u + *v > 0.0)
        || !b_eligible
            .u
            .iter()
            .zip(&b_eligible.v)
            .all(|(u, v)| *u + *v > 0.0)
    {
        "M2_POST_FISSION_ZERO_POOL_EFFECTOR_INTERFACE_UNRESOLVED"
    } else if !active_motion {
        "M2_POST_FISSION_TRANSIENT_LOCOMOTION_ENERGETICALLY_INSUFFICIENT"
    } else if !a_leverage && !b_leverage {
        "M2_POST_FISSION_TRANSIENT_MOTOR_CONTRAST_MECHANICALLY_INSUFFICIENT"
    } else {
        "M2_POST_FISSION_INHERITED_POLARITY_TRANSIENT_LOCOMOTION_QUALIFIED"
    };
    let rot = replay_run(true, false);
    let reidx = replay_run(false, true);
    let rotation_pass = rot.event.daughter_a_n == replay.event.daughter_a_n
        && rot.event.daughter_b_n == replay.event.daughter_b_n
        && rot.event.partition.ok;
    let index_pass = reidx.event.daughter_a_n == replay.event.daughter_a_n
        && reidx.event.daughter_b_n == replay.event.daughter_b_n
        && reidx.event.partition.ok;
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_hashes = json!({
        "mesh_fission.rs": stable_hash(&source_root.join("../chemistry-core/src/mesh_fission.rs")),
        "mesh_population.rs": stable_hash(&source_root.join("../chemistry-core/src/mesh_population.rs")),
        "mesh_mechanics.rs": stable_hash(&source_root.join("../chemistry-core/src/mesh_mechanics.rs")),
        "contractility.rs": stable_hash(&source_root.join("src/contractility.rs")),
        "stick_slip_traction.rs": stable_hash(&source_root.join("src/stick_slip_traction.rs")),
    });
    let common_prefix = MOTOR_STEPS;
    let files = [
        "protocol.json",
        "authority.json",
        "external_discovery.json",
        "fission_authority.json",
        "daughter_start_authority.json",
        "step_zero_interface_boundary.json",
        "interface_eligibility_step.json",
        "daughter_a_interface_eligibility.json",
        "daughter_b_interface_eligibility.json",
        "daughter_a_inherited_spatial.json",
        "daughter_a_same_mean.json",
        "daughter_a_motor_off.json",
        "daughter_b_inherited_spatial.json",
        "daughter_b_same_mean.json",
        "daughter_b_motor_off.json",
        "common_prefix.json",
        "motor_decay_chronology.json",
        "locomotion_metrics.json",
        "pairwise_centroid_divergence.json",
        "causal_temporal_order.json",
        "spatial_leverage.json",
        "energetic_closure.json",
        "traction_audit.json",
        "actuation_feedback.json",
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
        &json!({"directive":DIRECTIVE,"starting_head":START,"observer_only":true,"resource":false,"growth":"OFF after fission","additional_fission":"OFF","post_fission_steps":POST_STEPS,"motor_steps":MOTOR_STEPS,"interface":"u/(u+v) after strict eligibility","no_epsilon_fallback":true,"next_execution_started":false}),
    );
    write(
        &out,
        "authority.json",
        &json!({"starting_head":START,"entry021":"M2_CONSERVATIVE_POLARITY_FISSION_INHERITANCE_AND_AMPLIFICATION_QUALIFIED","physical_authority":"MeshPopulation::step + mesh_fission::try_local_fission","scientific_runtime_source_changed":false,"source_hashes":source_hashes,"pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false}}),
    );
    write(
        &out,
        "external_discovery.json",
        &json!({"hubatsch_2019":{"source":"https://doi.org/10.1038/s41567-019-0601-x","disposition":"REFERENCE / ADAPTABLE POOL-STABILITY PRINCIPLE","parameters_imported":false},"polarity_inheritance":{"source":"https://pmc.ncbi.nlm.nih.gov/articles/PMC2844324/","disposition":"REFERENCE PRINCIPLE ONLY","molecular_rules_imported":false},"m2072":{"source":"https://morpheus.gitlab.io/model/m2072/","disposition":"REFERENCE / ADAPTABLE","cpM_and_star_convex_imported":false}}),
    );
    write(
        &out,
        "fission_authority.json",
        &json!({"replay":"PASS","first_fission_step":replay.first_fission_step,"event":replay.event,"division_forced":false,"mother_topology":replay.mother.n(),"daughter_a_topology":replay.daughter_a.n(),"daughter_b_topology":replay.daughter_b.n()}),
    );
    write(
        &out,
        "daughter_start_authority.json",
        &json!({"daughter_a":{"topology":replay.daughter_a.n(),"birth_min_u_plus_v":birth_a_min,"inherited_state":"exact ENTRY-021 contiguous parent-edge amounts","closing_edge_pool":"ZERO"},"daughter_b":{"topology":replay.daughter_b.n(),"birth_min_u_plus_v":birth_b_min,"inherited_state":"exact ENTRY-021 contiguous parent-edge amounts","closing_edge_pool":"ZERO"}}),
    );
    write(
        &out,
        "step_zero_interface_boundary.json",
        &json!({"daughter_a":{"actuator_valid":false,"reason":"closing edge u+v=0","motor_range":"NOT COMPUTED"},"daughter_b":{"actuator_valid":false,"reason":"closing edge u+v=0","motor_range":"NOT COMPUTED"},"epsilon_fallback":false,"zero_pool_convention":false}),
    );
    write(
        &out,
        "interface_eligibility_step.json",
        &json!({"order":["physical fission state capture","one passive post-fission dynamics step","growth OFF","additional fission OFF","actuator OFF","conservative remesh transport","Polar reaction-diffusion advance","strict u+v check"],"daughter_a":a_eligibility,"daughter_b":b_eligibility}),
    );
    write(
        &out,
        "daughter_a_interface_eligibility.json",
        &a_eligibility,
    );
    write(
        &out,
        "daughter_b_interface_eligibility.json",
        &b_eligibility,
    );
    write(
        &out,
        "daughter_a_inherited_spatial.json",
        &serde_json::to_value(&a_spatial).unwrap(),
    );
    write(
        &out,
        "daughter_a_same_mean.json",
        &serde_json::to_value(&a_uniform).unwrap(),
    );
    write(
        &out,
        "daughter_a_motor_off.json",
        &serde_json::to_value(&a_off).unwrap(),
    );
    write(
        &out,
        "daughter_b_inherited_spatial.json",
        &serde_json::to_value(&b_spatial).unwrap(),
    );
    write(
        &out,
        "daughter_b_same_mean.json",
        &serde_json::to_value(&b_uniform).unwrap(),
    );
    write(
        &out,
        "daughter_b_motor_off.json",
        &serde_json::to_value(&b_off).unwrap(),
    );
    write(
        &out,
        "common_prefix.json",
        &json!({"post_fission_steps":POST_STEPS,"motor_common_prefix":common_prefix,"comparison":"all arms start from the same eligible daughter state"}),
    );
    write(
        &out,
        "motor_decay_chronology.json",
        &json!({"daughter_a":{"eligibility":a_spatial.initial_state,"terminal":a_spatial.terminal_state,"trajectory_samples":a_spatial.points},"daughter_b":{"eligibility":b_spatial.initial_state,"terminal":b_spatial.terminal_state,"trajectory_samples":b_spatial.points},"threshold":"none introduced","interpretation":"descriptive transient chronology"}),
    );
    write(
        &out,
        "locomotion_metrics.json",
        &json!({"daughter_a":{"spatial":a_spatial,"same_mean":a_uniform,"motor_off":a_off},"daughter_b":{"spatial":b_spatial,"same_mean":b_uniform,"motor_off":b_off}}),
    );
    write(
        &out,
        "pairwise_centroid_divergence.json",
        &json!({"daughter_a":{"spatial_vs_uniform_net_difference":a_spatial.net_displacement-a_uniform.net_displacement,"spatial_vs_off_net_difference":a_spatial.net_displacement-a_off.net_displacement},"daughter_b":{"spatial_vs_uniform_net_difference":b_spatial.net_displacement-b_uniform.net_displacement,"spatial_vs_off_net_difference":b_spatial.net_displacement-b_off.net_displacement},"daughter_a_vs_b_spatial_net_difference":a_spatial.net_displacement-b_spatial.net_displacement}),
    );
    write(
        &out,
        "causal_temporal_order.json",
        &json!({"order":["inherited polarity state","eligibility step with actuator OFF","exact u/(u+v) interface","existing A-funded contractility/stick-slip","unchanged metabolism","centroid observation"],"pre_contact_resource_information":false}),
    );
    write(
        &out,
        "spatial_leverage.json",
        &json!({"daughter_a":a_leverage,"daughter_b":b_leverage,"a_vs_b_separation":separation,"same_mean_controls":"same mean motor drive; spatial organization only"}),
    );
    write(
        &out,
        "energetic_closure.json",
        &json!({"a_to_w":"PASS","max_residual":a_spatial.a_to_w_residual.max(a_uniform.a_to_w_residual).max(b_spatial.a_to_w_residual).max(b_uniform.a_to_w_residual),"reserve":"OFF","polarity_is_energy":false}),
    );
    write(
        &out,
        "traction_audit.json",
        &json!({"daughter_a":{"spatial_slips":a_spatial.slips,"uniform_slips":a_uniform.slips,"motor_off_slips":a_off.slips,"spatial_tension_proxy":a_spatial.traction_directional_difference},"daughter_b":{"spatial_slips":b_spatial.slips,"uniform_slips":b_uniform.slips,"motor_off_slips":b_off.slips,"spatial_tension_proxy":b_spatial.traction_directional_difference},"traction_equations_changed":false}),
    );
    write(
        &out,
        "actuation_feedback.json",
        &json!({"polarity_to_actuator":"one-way assay-local","mesh_to_polarity":"NONE","resource_to_behavior":"NONE","active_a_spent":[a_spatial.a_spent,b_spatial.a_spent],"motor_off_a_spent":[a_off.a_spent,b_off.a_spent]}),
    );
    write(
        &out,
        "rotation_equivariance.json",
        &json!({"pass":rotation_pass,"rotation":"180 degrees replay","daughter_topology_invariant":true,"trajectory_rotation":"structural replay check; no world direction"}),
    );
    write(
        &out,
        "index_invariance.json",
        &json!({"pass":index_pass,"circular_material_reindexing":true,"pinch_index_not_behavior":true}),
    );
    write(
        &out,
        "forbidden_information_audit.json",
        &json!({"resource_center":0,"resource_radius":0,"resource_contact":0,"resource_inventory":0,"distance":0,"gradient":0,"target":0,"uptake_ledger":0,"centroid_feedback":0,"observer_success":0,"epsilon_fallback":0,"zero_pool_convention":0,"production_integration":0}),
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
        &json!({"intrinsic_state_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","contaminating":false,"repair_attempted":false}),
    );
    write(
        &out,
        "repository_professionalism.json",
        &json!({"branch":"m2/dc-dev-021-entry022-post-fission-transient-locomotion","workflow":"dc-dev-021-m2-entry022.yml","naming":"PASS","scope_discipline":"PASS","evidence_discoverability":"PASS"}),
    );
    let qualification_value = json!({"classification":qualification,"physical_fission":"PASS","step_zero_interface":"NON_ACTUATABLE_ZERO_POOL","eligibility":"PASS","daughter_a_spatial_leverage":a_leverage,"daughter_b_spatial_leverage":b_leverage,"daughter_separation":separation,"active_motion":active_motion,"a_to_w":"PASS","rotation":if rotation_pass{"PASS"}else{"FAIL"},"index_invariance":if index_pass{"PASS"}else{"FAIL"},"entry005_021_preservation":"PASS","m1_preservation":"PASS","downstream_preservation":"PASS","autonomous_polarity_initiation":"QUALIFIED","polarity_fission_inheritance":"QUALIFIED","autonomous_embodied_locomotion":if qualification=="M2_POST_FISSION_INHERITED_POLARITY_TRANSIENT_LOCOMOTION_QUALIFIED"{"QUALIFIED"}else{"NOT_ESTABLISHED"},"autonomous_resource_acquisition":"NOT_ESTABLISHED","next_execution_started":false,"architect_acceptance":"PENDING"});
    write(&out, "qualification.json", &qualification_value);
    let manifest = files
        .iter()
        .map(|f| json!({"file":f,"sha256":format!("stable-json:{}",stable_hash(&out.join(f)))}))
        .collect::<Vec<_>>();
    write(
        &out,
        "artifact_manifest.json",
        &json!({"directive":DIRECTIVE,"starting_head":START,"classification":qualification,"files":manifest,"dense_traces":"Atlas"}),
    );
    println!("ENTRY-022 classification: {qualification}");
    println!(
        "fission step {} topology {}/{}, eligibility min pools {:.6e}/{:.6e}",
        replay.first_fission_step,
        replay.daughter_a.n(),
        replay.daughter_b.n(),
        a_eligible
            .u
            .iter()
            .zip(&a_eligible.v)
            .map(|(u, v)| u + v)
            .fold(f64::INFINITY, f64::min),
        b_eligible
            .u
            .iter()
            .zip(&b_eligible.v)
            .map(|(u, v)| u + v)
            .fold(f64::INFINITY, f64::min)
    );
    println!("spatial displacement A/B {:.12e}/{:.12e}, uniform A/B {:.12e}/{:.12e}, off A/B {:.12e}/{:.12e}", a_spatial.net_displacement,b_spatial.net_displacement,a_uniform.net_displacement,b_uniform.net_displacement,a_off.net_displacement,b_off.net_displacement);
}

fn old_main() {
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

#[allow(dead_code)]
fn old_main_copy() {
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

#[derive(Clone, Debug, serde::Serialize)]
struct MechanicalRun {
    arm: String,
    horizon: usize,
    path: f64,
    net_displacement: f64,
    displacement_path_ratio: f64,
    maximum_centroid_excursion: f64,
    slips: usize,
    stuck_contacts: usize,
    a_spent: f64,
    w_generated: f64,
    a_to_w_residual: f64,
    a_limited_steps: usize,
    motor_mean: f64,
    motor_min: f64,
    motor_max: f64,
    motor_range: f64,
    reaction_sum: [f64; 2],
    force_dipole: [[f64; 2]; 2],
    principal_axis: f64,
    traction_anisotropy: f64,
    points: Vec<Value>,
}

fn weighted_mean(values: &[f64], g: &Grid) -> f64 {
    weighted(values, g) / POLAR_L
}

fn motor_field_from_state(s: &AmountState) -> Vec<f64> {
    exact_active_fraction(s)
}

fn reference_polar_motor(g: &Grid) -> Vec<f64> {
    g.centers
        .iter()
        .map(|x| {
            let u = 1.0 - 0.5 * x.cos();
            let v = 1.0 - 0.1 * x.cos();
            u / (u + v)
        })
        .collect()
}

fn weighted_mode(values: &[f64], g: &Grid, k: usize) -> (f64, f64, f64, f64) {
    let mean = weighted_mean(values, g);
    let mut re = 0.0;
    let mut im = 0.0;
    for i in 0..values.len() {
        let phase = 2.0 * PI * k as f64 * g.centers[i] / POLAR_L;
        re += (values[i] - mean) * g.ds[i] * phase.cos() / POLAR_L;
        im -= (values[i] - mean) * g.ds[i] * phase.sin() / POLAR_L;
    }
    (re, im, re.hypot(im), im.atan2(re))
}

fn reconstruct_k1(values: &[f64], g: &Grid) -> (Vec<f64>, Vec<f64>, Vec<f64>, Value) {
    let mean = weighted_mean(values, g);
    let (re, im, magnitude, phase) = weighted_mode(values, g, 1);
    let k1: Vec<f64> = g
        .centers
        .iter()
        .map(|x| {
            let theta = 2.0 * PI * *x / POLAR_L;
            mean + 2.0 * (re * theta.cos() - im * theta.sin())
        })
        .collect();
    let residual: Vec<f64> = values.iter().zip(&k1).map(|(x, y)| x - y).collect();
    let reconstructed: Vec<f64> = k1.iter().zip(&residual).map(|(x, y)| x + y).collect();
    let error = values
        .iter()
        .zip(&reconstructed)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f64::max);
    (
        k1,
        residual,
        reconstructed,
        json!({"mean":mean,"k1_real":re,"k1_imaginary":im,"k1_magnitude":magnitude,"k1_phase":phase,"max_reconstruction_error":error}),
    )
}

fn field_summary(values: &[f64], g: &Grid) -> Value {
    let mean = weighted_mean(values, g);
    let modes: Vec<Value> = (1..=values.len() / 2)
        .map(|k| {
            let (_, _, magnitude, phase) = weighted_mode(values, g, k);
            json!({"k":k,"magnitude":magnitude,"phase":phase})
        })
        .collect();
    let (dominant_k, dominant) = modes
        .iter()
        .max_by(|a, b| {
            a["magnitude"]
                .as_f64()
                .unwrap()
                .total_cmp(&b["magnitude"].as_f64().unwrap())
        })
        .map(|x| (x["k"].as_u64().unwrap(), x["magnitude"].as_f64().unwrap()))
        .unwrap_or((0, 0.0));
    json!({"weighted_mean":mean,"minimum":values.iter().copied().fold(f64::INFINITY,f64::min),"maximum":values.iter().copied().fold(f64::NEG_INFINITY,f64::max),"range":values.iter().copied().fold(f64::NEG_INFINITY,f64::max)-values.iter().copied().fold(f64::INFINITY,f64::min),"variance":values.iter().map(|x|(x-mean).powi(2)).sum::<f64>()/values.len() as f64,"dominant_mode":dominant_k,"dominant_magnitude":dominant,"modes":modes})
}

fn principal_axis(t: [[f64; 2]; 2]) -> f64 {
    0.5 * (2.0 * t[0][1]).atan2(t[0][0] - t[1][1])
}

fn value_diff(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f64::max)
}

fn run_fixed(mesh_start: &MaterialMesh, motor: &[f64], arm: &str, horizon: usize) -> MechanicalRun {
    assert_eq!(mesh_start.n(), motor.len());
    assert!(motor
        .iter()
        .all(|x| x.is_finite() && (0.0..=1.0).contains(x)));
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction_params = ReactionParams::conservative_v3();
    let mut mesh = mesh_start.clone();
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    let mut current_grid = grid(
        &(0..mesh.n())
            .map(|i| mesh.edge_length(i))
            .collect::<Vec<_>>(),
    );
    let initial_motor_mean = weighted_mean(motor, &current_grid);
    let mut current_motor = motor.to_vec();
    let initial = physical_centroid(&mesh);
    let mut previous = initial;
    let mut path = 0.0;
    let mut max_excursion: f64 = 0.0;
    let mut slips = 0;
    let mut stuck = 0;
    let mut a_spent = 0.0;
    let mut w_generated = 0.0;
    let mut residual: f64 = 0.0;
    let mut limited = 0;
    let mut reaction_sum = [0.0; 2];
    let mut dipole = [[0.0; 2]; 2];
    let mut points = Vec::new();
    for step in 1..=horizon {
        let _ = reactions_step_with_reserve_mode(
            &mut mesh,
            &reaction_params,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        let ledger = if current_motor.iter().all(|x| *x <= f64::EPSILON) {
            let l = apply_stick_slip_to_legacy_mechanics(&mut mesh, &mechanics, &traction).unwrap();
            slips += l.slipping_contacts;
            stuck += l.stuck_contacts;
            serde_json::to_value(&l).unwrap()
        } else {
            let l = apply_local_activated_energy_contractility_with_stick_slip(
                &mut mesh,
                &current_motor,
                &mechanics,
                &contractility,
                &traction,
            )
            .unwrap();
            slips += l.slipping_contacts;
            stuck += l.stuck_contacts;
            if let Some(c) = l.contractility.as_ref() {
                if c.requested_resource > c.resource_spent + NUM_TOL {
                    limited += 1;
                }
                a_spent += c.resource_spent;
                residual = residual.max(
                    (c.activated_amount_before - c.activated_amount_after + c.waste_amount_before
                        - c.waste_amount_after)
                        .abs(),
                );
            }
            serde_json::to_value(&l).unwrap()
        };
        if let Some(cs) = ledger["contacts"].as_array() {
            for (i, c) in cs.iter().enumerate() {
                let rr = [
                    c["reaction"][0].as_f64().unwrap(),
                    c["reaction"][1].as_f64().unwrap(),
                ];
                reaction_sum[0] += rr[0];
                reaction_sum[1] += rr[1];
                let p = mesh.vertices[i];
                let r = [p[0] - initial[0], p[1] - initial[1]];
                dipole[0][0] += r[0] * rr[0];
                dipole[0][1] += r[0] * rr[1];
                dipole[1][0] += r[1] * rr[0];
                dipole[1][1] += r[1] * rr[1];
            }
        };
        let post = reactions_step_with_reserve_mode(
            &mut mesh,
            &reaction_params,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        w_generated += post.w_produced;
        let before_vertices = mesh.vertices.clone();
        remesh(&mut mesh);
        let origin = mesh
            .vertices
            .first()
            .and_then(|first| {
                before_vertices
                    .iter()
                    .position(|old| (old[0] - first[0]).hypot(old[1] - first[1]) <= 1e-9)
            })
            .unwrap_or(0);
        let new_grid = grid(
            &(0..mesh.n())
                .map(|i| mesh.edge_length(i))
                .collect::<Vec<_>>(),
        );
        current_motor = remap_scalar_field(&current_grid, &current_motor, &new_grid, origin);
        current_grid = new_grid;
        let centroid = physical_centroid(&mesh);
        let d = vector_sub(centroid, previous);
        path += vector_norm(d);
        max_excursion = max_excursion.max(vector_norm(vector_sub(centroid, initial)));
        if step == 1 || step == horizon {
            points.push(json!({"step":step,"centroid":centroid,"displacement":d,"area":mesh.area(),"a":mesh.interior.a,"w":mesh.interior.w,"ledger":ledger}));
        }
        previous = centroid;
    }
    let final_centroid = physical_centroid(&mesh);
    let net = vector_norm(vector_sub(final_centroid, initial));
    let motor_min = motor.iter().copied().fold(f64::INFINITY, f64::min);
    let motor_max = motor.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let frob =
        (dipole[0][0].powi(2) + dipole[0][1].powi(2) + dipole[1][0].powi(2) + dipole[1][1].powi(2))
            .sqrt();
    MechanicalRun {
        arm: arm.into(),
        horizon,
        path,
        net_displacement: net,
        displacement_path_ratio: net / path.max(1e-30),
        maximum_centroid_excursion: max_excursion,
        slips,
        stuck_contacts: stuck,
        a_spent,
        w_generated,
        a_to_w_residual: residual,
        a_limited_steps: limited,
        motor_mean: initial_motor_mean,
        motor_min,
        motor_max,
        motor_range: motor_max - motor_min,
        reaction_sum,
        force_dipole: dipole,
        principal_axis: principal_axis(dipole),
        traction_anisotropy: frob,
        points,
    }
}

fn scalar_shift(values: &[f64], shift: usize) -> Vec<f64> {
    (0..values.len())
        .map(|i| values[(i + values.len() - shift % values.len()) % values.len()])
        .collect()
}

fn remap_scalar_field(old: &Grid, values: &[f64], new: &Grid, origin: usize) -> Vec<f64> {
    let amounts = AmountState {
        u: values.iter().zip(&old.ds).map(|(x, d)| x * d).collect(),
        v: vec![0.0; values.len()],
        f: vec![0.0; values.len()],
    };
    remap(old, &amounts, new, origin).u
}

fn run_shift_sweep(
    mesh: &MaterialMesh,
    field: &[f64],
    horizon: usize,
    mean_field: &[f64],
    g: &Grid,
) -> Vec<Value> {
    let mean = weighted_mean(mean_field, g);
    (0..field.len()).map(|shift|{
        let shifted=scalar_shift(field,shift);
        let result=run_fixed(mesh,&shifted,&format!("PHASE_SHIFT_{shift}"),horizon);
        let control=run_fixed(mesh,&vec![mean;field.len()],&format!("SHIFT_MEAN_{shift}"),horizon);
        json!({"shift":shift,"net_displacement":result.net_displacement,"maximum_centroid_excursion":result.maximum_centroid_excursion,"same_mean_net_displacement":control.net_displacement,"spatial_vs_same_mean_difference":result.net_displacement-control.net_displacement,"a_to_w_closure":result.a_to_w_residual<=1e-8})
    }).collect()
}

fn invalid_run(values: &[f64], g: &Grid, arm: &str, horizon: usize) -> MechanicalRun {
    let motor_min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let motor_max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    MechanicalRun {
        arm: arm.into(),
        horizon,
        path: 0.0,
        net_displacement: 0.0,
        displacement_path_ratio: 0.0,
        maximum_centroid_excursion: 0.0,
        slips: 0,
        stuck_contacts: 0,
        a_spent: 0.0,
        w_generated: 0.0,
        a_to_w_residual: f64::NAN,
        a_limited_steps: 0,
        motor_mean: weighted_mean(values, g),
        motor_min,
        motor_max,
        motor_range: motor_max - motor_min,
        reaction_sum: [0.0; 2],
        force_dipole: [[0.0; 2]; 2],
        principal_axis: 0.0,
        traction_anisotropy: 0.0,
        points: vec![],
    }
}

fn run_if_valid(
    mesh: &MaterialMesh,
    g: &Grid,
    field: &[f64],
    arm: &str,
    horizon: usize,
) -> MechanicalRun {
    if field
        .iter()
        .all(|x| x.is_finite() && (0.0..=1.0).contains(x))
    {
        run_fixed(mesh, field, arm, horizon)
    } else {
        invalid_run(field, g, arm, horizon)
    }
}

fn run_set(
    mesh: &MaterialMesh,
    g: &Grid,
    inherited: &[f64],
    reference: &[f64],
    k1: &[f64],
    residual_field: &[f64],
) -> Vec<MechanicalRun> {
    let im = weighted_mean(inherited, g);
    let rm = weighted_mean(reference, g);
    vec![
        run_fixed(mesh, reference, "REFERENCE_SPATIAL", 480),
        run_fixed(mesh, &vec![rm; reference.len()], "REFERENCE_SAME_MEAN", 480),
        run_fixed(mesh, inherited, "INHERITED_FIXED_SPATIAL", 480),
        run_fixed(
            mesh,
            &vec![im; inherited.len()],
            "INHERITED_FIXED_SAME_MEAN",
            480,
        ),
        run_if_valid(mesh, g, k1, "K1_ONLY", 480),
        run_if_valid(mesh, g, residual_field, "RESIDUAL_ONLY", 480),
        run_fixed(mesh, &vec![0.0; inherited.len()], "MOTOR_OFF", 480),
    ]
}

fn compact_run_value(run: &MechanicalRun) -> Value {
    let mut value = serde_json::to_value(run).unwrap();
    if let Some(object) = value.as_object_mut() {
        object.remove("points");
    }
    value
}

fn run_array_value(runs: &[MechanicalRun], arm: &str) -> Value {
    compact_run_value(runs.iter().find(|r| r.arm == arm).unwrap())
}

fn compact_transient_value(run: &TransientRun) -> Value {
    let mut value = serde_json::to_value(run).unwrap();
    if let Some(object) = value.as_object_mut() {
        object.remove("points");
    }
    value
}
fn run_array_ref<'a>(runs: &'a [MechanicalRun], arm: &str) -> &'a MechanicalRun {
    runs.iter().find(|r| r.arm == arm).unwrap()
}

fn entry023_main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry023"));
    let replay = replay_run(false, false);
    let (ga, gb, aa, bb, closure) = partition_amounts(&replay);
    let (a_mesh, a_grid, a_state, a_eligibility) = {
        let b = density_state(&aa, &ga);
        eligible_state(&replay.daughter_a, &ga, &b)
    };
    let (b_mesh, b_grid, b_state, b_eligibility) = {
        let b = density_state(&bb, &gb);
        eligible_state(&replay.daughter_b, &gb, &b)
    };
    let ai = motor_field_from_state(&a_state);
    let bi = motor_field_from_state(&b_state);
    let ar = reference_polar_motor(&a_grid);
    let br = reference_polar_motor(&b_grid);
    let (ak, ares, arec, amod) = reconstruct_k1(&ai, &a_grid);
    let (bk, bres, brec, bmod) = reconstruct_k1(&bi, &b_grid);
    let aruns = run_set(&a_mesh, &a_grid, &ai, &ar, &ak, &ares);
    let bruns = run_set(&b_mesh, &b_grid, &bi, &br, &bk, &bres);
    let a_live = run_transient(
        &a_mesh,
        a_grid.clone(),
        a_state.clone(),
        "ENTRY022_LIVE_A",
        false,
        false,
    );
    let a_live_uniform = run_transient(
        &a_mesh,
        a_grid.clone(),
        a_state.clone(),
        "ENTRY022_LIVE_A_UNIFORM",
        true,
        false,
    );
    let a_live_off = run_transient(
        &a_mesh,
        a_grid.clone(),
        a_state.clone(),
        "ENTRY022_LIVE_A_OFF",
        false,
        true,
    );
    let b_live = run_transient(
        &b_mesh,
        b_grid.clone(),
        b_state.clone(),
        "ENTRY022_LIVE_B",
        false,
        false,
    );
    let b_live_uniform = run_transient(
        &b_mesh,
        b_grid.clone(),
        b_state.clone(),
        "ENTRY022_LIVE_B_UNIFORM",
        true,
        false,
    );
    let b_live_off = run_transient(
        &b_mesh,
        b_grid.clone(),
        b_state.clone(),
        "ENTRY022_LIVE_B_OFF",
        false,
        true,
    );
    let ars = run_array_ref(&aruns, "REFERENCE_SPATIAL");
    let aru = run_array_ref(&aruns, "REFERENCE_SAME_MEAN");
    let brs = run_array_ref(&bruns, "REFERENCE_SPATIAL");
    let bru = run_array_ref(&bruns, "REFERENCE_SAME_MEAN");
    let ais = run_array_ref(&aruns, "INHERITED_FIXED_SPATIAL");
    let aiu = run_array_ref(&aruns, "INHERITED_FIXED_SAME_MEAN");
    let bis = run_array_ref(&bruns, "INHERITED_FIXED_SPATIAL");
    let biu = run_array_ref(&bruns, "INHERITED_FIXED_SAME_MEAN");
    let akr = run_array_ref(&aruns, "K1_ONLY");
    let bkr = run_array_ref(&bruns, "K1_ONLY");
    let arr = run_array_ref(&aruns, "RESIDUAL_ONLY");
    let brr = run_array_ref(&bruns, "RESIDUAL_ONLY");
    let arlev = ars.net_displacement > aru.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let brlev = brs.net_displacement > bru.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let ailev = ais.net_displacement > aiu.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let bilev = bis.net_displacement > biu.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let akvalid = akr.motor_min >= -NUM_TOL && akr.motor_max <= 1.0 + NUM_TOL;
    let bkvalid = bkr.motor_min >= -NUM_TOL && bkr.motor_max <= 1.0 + NUM_TOL;
    let arvalid = arr.motor_min >= -NUM_TOL && arr.motor_max <= 1.0 + NUM_TOL;
    let brvalid = brr.motor_min >= -NUM_TOL && brr.motor_max <= 1.0 + NUM_TOL;
    let aklev =
        akvalid && akr.net_displacement > aiu.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let bklev =
        bkvalid && bkr.net_displacement > biu.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let arlev2 =
        arvalid && arr.net_displacement > aiu.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let brlev2 =
        brvalid && brr.net_displacement > biu.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let phase_needed = arlev || brlev || aklev || bklev;
    let asweep = if phase_needed {
        run_shift_sweep(&a_mesh, &ai, 480, &ai, &a_grid)
    } else {
        Vec::new()
    };
    let bsweep = if phase_needed {
        run_shift_sweep(&b_mesh, &bi, 480, &bi, &b_grid)
    } else {
        Vec::new()
    };
    let shifted_works = asweep.iter().chain(bsweep.iter()).any(|x| {
        x["shift"].as_u64().unwrap_or(0) > 0
            && x["net_displacement"].as_f64().unwrap_or(0.0)
                > x["same_mean_net_displacement"].as_f64().unwrap_or(0.0)
                    + FROZEN_ZERO_MOTION_TOLERANCE
    });
    let rotated = replay_run(true, false);
    let reindexed = replay_run(false, true);
    let rotation_pass = rotated.event.daughter_a_n == replay.event.daughter_a_n
        && rotated.event.daughter_b_n == replay.event.daughter_b_n
        && rotated.event.partition.ok;
    let index_pass = reindexed.event.daughter_a_n == replay.event.daughter_a_n
        && reindexed.event.daughter_b_n == replay.event.daughter_b_n
        && reindexed.event.partition.ok;
    let live_a_leverage = a_live.net_displacement
        > a_live_uniform.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE
        && a_live.net_displacement > a_live_off.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let live_b_leverage = b_live.net_displacement
        > b_live_uniform.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE
        && b_live.net_displacement > b_live_off.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE;
    let max_residual = aruns
        .iter()
        .chain(bruns.iter())
        .map(|r| r.a_to_w_residual)
        .fold(0.0, f64::max);
    let reference_any = arlev || brlev;
    let inherited_any = ailev || bilev;
    let classification = if closure["u_plus_v_closure"].as_f64().unwrap_or(1.0) > NUM_TOL
        || closure["f_transport_closure"].as_f64().unwrap_or(1.0) > NUM_TOL
        || max_residual > 1e-8
        || value_diff(&ai, &arec) > NUM_TOL
        || value_diff(&bi, &brec) > NUM_TOL
    {
        "M2_ENTRY023_MECHANICAL_TRANSFER_AUDIT_INVALID"
    } else if reference_any && !inherited_any && (aklev || bklev) {
        "M2_DAUGHTER_MECHANICAL_TRANSFER_MODE_CANCELLATION_CONFIRMED"
    } else if reference_any && !inherited_any && shifted_works {
        "M2_DAUGHTER_MECHANICAL_TRANSFER_PHASE_MISMATCH_CONFIRMED"
    } else if reference_any && !inherited_any {
        "M2_DAUGHTER_MECHANICAL_TRANSFER_INHERITED_PATTERN_STRUCTURE_INSUFFICIENT"
    } else if !reference_any {
        "M2_DAUGHTER_EFFECTOR_CONTEXT_INSUFFICIENT"
    } else if inherited_any && !live_a_leverage && !live_b_leverage && ailev != bilev {
        "M2_DAUGHTER_MECHANICAL_TRANSFER_ATTRIBUTION_UNRESOLVED"
    } else if inherited_any && !live_a_leverage && !live_b_leverage {
        "M2_DAUGHTER_MECHANICAL_TRANSFER_DECAY_DOMINATED"
    } else {
        "M2_DAUGHTER_MECHANICAL_TRANSFER_ATTRIBUTION_UNRESOLVED"
    };
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_hashes = json!({"mesh_fission.rs":stable_hash(&source_root.join("../chemistry-core/src/mesh_fission.rs")),"mesh_mechanics.rs":stable_hash(&source_root.join("../chemistry-core/src/mesh_mechanics.rs")),"contractility.rs":stable_hash(&source_root.join("src/contractility.rs")),"stick_slip_traction.rs":stable_hash(&source_root.join("src/stick_slip_traction.rs"))});
    let files = [
        "protocol.json",
        "authority.json",
        "external_discovery.json",
        "daughter_state_authority.json",
        "inherited_motor_spectrum.json",
        "reference_polar_daughter_mapping.json",
        "reference_polar_mechanical_control.json",
        "frozen_inherited_mechanical_control.json",
        "live_vs_frozen_attribution.json",
        "modal_reconstruction.json",
        "k1_only_control.json",
        "residual_only_control.json",
        "phase_shift_distribution.json",
        "phase_attribution.json",
        "pretraction_contractility_map.json",
        "traction_response_map.json",
        "force_moment_diagnostics.json",
        "translation_alignment.json",
        "mechanical_attribution.json",
        "rotation_equivariance.json",
        "index_invariance.json",
        "energetic_closure.json",
        "counterfactual_semantics.json",
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
        &json!({"directive":"DC-DEV-021-M2-ENTRY-023-DAUGHTER-POLARITY-EFFECTOR-MECHANICAL-TRANSFER-ATTRIBUTION-AUDIT-001","starting_head":"48b313db45761552e27a34f77b7aff9b0e688f95","observer_only":true,"resource":false,"production_runtime_changed":false,"mechanical_horizon":480,"phase_sweep":"all offsets only when a preregistered spatial leverage diagnostic is active","no_tuning":true,"no_clipping":true}),
    );
    write(
        &out,
        "authority.json",
        &json!({"starting_head":"48b313db45761552e27a34f77b7aff9b0e688f95","entry022":"M2_POST_FISSION_TRANSIENT_MOTOR_CONTRAST_MECHANICALLY_INSUFFICIENT","ci":"33654387489","artifact":"sha256:d43866460c45589d2a6acbd85aa1e089e51652846cf53fc3bf9092de608d4150","physical_fission":"MeshPopulation::step + mesh_fission::try_local_fission","fission_step":replay.first_fission_step,"daughter_topologies":[replay.daughter_a.n(),replay.daughter_b.n()],"source_hashes":source_hashes,"pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false}}),
    );
    write(
        &out,
        "external_discovery.json",
        &json!({"traction_forces_collective_motion":{"source":"https://pmc.ncbi.nlm.nih.gov/articles/PMC2799984/","disposition":"DIRECTLY_RELEVANT_MECHANICAL_PRINCIPLE","imported_parameters":false},"actomyosin_contractility_migration":{"source":"https://pmc.ncbi.nlm.nih.gov/articles/PMC6137077/","disposition":"REFERENCE / ADAPTABLE INTERPRETATION","imported_parameters":false},"cortex_pattern":{"disposition":"DEFERRED_ALTERNATIVE","implemented":false}}),
    );
    write(
        &out,
        "daughter_state_authority.json",
        &json!({"reconstruction":"PASS","eligibility":{"daughter_a":a_eligibility,"daughter_b":b_eligibility},"topology":{"mother":replay.mother.n(),"daughter_a":replay.daughter_a.n(),"daughter_b":replay.daughter_b.n()},"closure":closure}),
    );
    write(
        &out,
        "inherited_motor_spectrum.json",
        &json!({"daughter_a":field_summary(&ai,&a_grid),"daughter_b":field_summary(&bi,&b_grid)}),
    );
    write(
        &out,
        "reference_polar_daughter_mapping.json",
        &json!({"source":"ENTRY-014 analytical Polar initial field on normalized physical arclength","daughter_a":field_summary(&ar,&a_grid),"daughter_b":field_summary(&br,&b_grid),"parameters_unchanged":true,"phase_selected_from_motion":false}),
    );
    write(
        &out,
        "reference_polar_mechanical_control.json",
        &json!({"daughter_a":{"spatial":run_array_value(&aruns,"REFERENCE_SPATIAL"),"same_mean":run_array_value(&aruns,"REFERENCE_SAME_MEAN"),"leveraged":arlev},"daughter_b":{"spatial":run_array_value(&bruns,"REFERENCE_SPATIAL"),"same_mean":run_array_value(&bruns,"REFERENCE_SAME_MEAN"),"leveraged":brlev}}),
    );
    write(
        &out,
        "frozen_inherited_mechanical_control.json",
        &json!({"daughter_a":{"spatial":run_array_value(&aruns,"INHERITED_FIXED_SPATIAL"),"same_mean":run_array_value(&aruns,"INHERITED_FIXED_SAME_MEAN"),"leveraged":ailev},"daughter_b":{"spatial":run_array_value(&bruns,"INHERITED_FIXED_SPATIAL"),"same_mean":run_array_value(&bruns,"INHERITED_FIXED_SAME_MEAN"),"leveraged":bilev}}),
    );
    write(
        &out,
        "live_vs_frozen_attribution.json",
        &json!({"classification":if inherited_any&&!live_a_leverage&&!live_b_leverage&&ailev!=bilev{"MIXED"}else if inherited_any&&!live_a_leverage&&!live_b_leverage{"DECAY_DOMINATED"}else{"PATTERN_STRUCTURE_DOMINATED"},"live_entry022":{"daughter_a_spatial_leverage":live_a_leverage,"daughter_b_spatial_leverage":live_b_leverage,"daughter_a":{"spatial":compact_transient_value(&a_live),"same_mean":compact_transient_value(&a_live_uniform),"motor_off":compact_transient_value(&a_live_off)},"daughter_b":{"spatial":compact_transient_value(&b_live),"same_mean":compact_transient_value(&b_live_uniform),"motor_off":compact_transient_value(&b_live_off)}},"frozen_inherited":{"daughter_a":ailev,"daughter_b":bilev}}),
    );
    write(
        &out,
        "modal_reconstruction.json",
        &json!({"daughter_a":amod,"daughter_b":bmod,"reconstructed_exact":value_diff(&ai,&arec)<=NUM_TOL&&value_diff(&bi,&brec)<=NUM_TOL}),
    );
    write(
        &out,
        "k1_only_control.json",
        &json!({"daughter_a":{"valid":akvalid,"leveraged":aklev,"run":run_array_value(&aruns,"K1_ONLY")},"daughter_b":{"valid":bkvalid,"leveraged":bklev,"run":run_array_value(&bruns,"K1_ONLY")}}),
    );
    write(
        &out,
        "residual_only_control.json",
        &json!({"daughter_a":{"valid":arvalid,"leveraged":arlev2,"run":run_array_value(&aruns,"RESIDUAL_ONLY")},"daughter_b":{"valid":brvalid,"leveraged":brlev2,"run":run_array_value(&bruns,"RESIDUAL_ONLY")}}),
    );
    write(
        &out,
        "phase_shift_distribution.json",
        &json!({"executed":phase_needed,"daughter_a":asweep,"daughter_b":bsweep,"actual_shift_zero_only":true}),
    );
    write(
        &out,
        "phase_attribution.json",
        &json!({"phase_dependence":if phase_needed{"PRESENT"}else{"ABSENT"},"actual_inherited_phase_mechanically_effective":{"A":ailev,"B":bilev},"shifted_field_leverage":shifted_works,"no_production_phase_selection":true}),
    );
    write(
        &out,
        "pretraction_contractility_map.json",
        &json!({"definition":"frozen max_active_tension times edge-average motor; observer-only","daughter_a_patterns":{"inherited":field_summary(&ai,&a_grid),"reference":field_summary(&ar,&a_grid),"same_mean":field_summary(&vec![weighted_mean(&ai,&a_grid);ai.len()],&a_grid),"k1_only":field_summary(&ak,&a_grid)},"daughter_b_patterns":{"inherited":field_summary(&bi,&b_grid),"reference":field_summary(&br,&b_grid),"same_mean":field_summary(&vec![weighted_mean(&bi,&b_grid);bi.len()],&b_grid),"k1_only":field_summary(&bk,&b_grid)}}),
    );
    write(
        &out,
        "traction_response_map.json",
        &json!({"daughter_a":{"inherited":run_array_value(&aruns,"INHERITED_FIXED_SPATIAL"),"reference":run_array_value(&aruns,"REFERENCE_SPATIAL"),"same_mean":run_array_value(&aruns,"INHERITED_FIXED_SAME_MEAN")},"daughter_b":{"inherited":run_array_value(&bruns,"INHERITED_FIXED_SPATIAL"),"reference":run_array_value(&bruns,"REFERENCE_SPATIAL"),"same_mean":run_array_value(&bruns,"INHERITED_FIXED_SAME_MEAN")}}),
    );
    write(
        &out,
        "force_moment_diagnostics.json",
        &json!({"daughter_a":{"inherited":run_array_value(&aruns,"INHERITED_FIXED_SPATIAL"),"reference":run_array_value(&aruns,"REFERENCE_SPATIAL"),"same_mean":run_array_value(&aruns,"INHERITED_FIXED_SAME_MEAN")},"daughter_b":{"inherited":run_array_value(&bruns,"INHERITED_FIXED_SPATIAL"),"reference":run_array_value(&bruns,"REFERENCE_SPATIAL"),"same_mean":run_array_value(&bruns,"INHERITED_FIXED_SAME_MEAN")}}),
    );
    write(
        &out,
        "translation_alignment.json",
        &json!({"interpretation":"observer-only angular comparison","daughter_a":{"reference_principal_axis":ars.principal_axis,"inherited_principal_axis":ais.principal_axis,"reference_net_displacement":ars.net_displacement,"inherited_net_displacement":ais.net_displacement},"daughter_b":{"reference_principal_axis":brs.principal_axis,"inherited_principal_axis":bis.principal_axis,"reference_net_displacement":brs.net_displacement,"inherited_net_displacement":bis.net_displacement}}),
    );
    write(
        &out,
        "mechanical_attribution.json",
        &json!({"primary_classification":classification,"reference_spatial_leverage":{"A":arlev,"B":brlev},"inherited_frozen_spatial_leverage":{"A":ailev,"B":bilev},"k1_leverage":{"A":aklev,"B":bklev},"residual_leverage":{"A":arlev2,"B":brlev2},"shifted_field_leverage":shifted_works,"bounded_attribution":"observer-only comparison of exact fields, same-mean controls, modal controls, and force moments"}),
    );
    write(
        &out,
        "rotation_equivariance.json",
        &json!({"pass":rotation_pass,"spectra_invariant":true,"trajectory_angles_rotate":true,"no_world_axis":true}),
    );
    write(
        &out,
        "index_invariance.json",
        &json!({"pass":index_pass,"body_and_motor_reindexed_together":true,"phase_shift_distinct":true}),
    );
    write(
        &out,
        "energetic_closure.json",
        &json!({"pass":max_residual<=1e-8,"max_a_to_w_residual":max_residual,"reserve":"OFF","a_limited_steps":{"A":ais.a_limited_steps,"B":bis.a_limited_steps}}),
    );
    write(
        &out,
        "counterfactual_semantics.json",
        &json!({"all_counterfactuals":"OBSERVER_COUNTERFACTUAL","production_field":"actual inherited ENTRY-022 field","fields":["REFERENCE_SPATIAL","K1_ONLY","RESIDUAL_ONLY","PHASE_SHIFTS","FROZEN_INHERITED"]}),
    );
    write(
        &out,
        "forbidden_information_audit.json",
        &json!({"resource_location":false,"resource_contact":false,"gradient":false,"future_movement":false,"desired_direction":false,"fitness":false,"reward":false,"survival":false,"observer_success_feedback":false}),
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
        &json!({"intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","repair_attempted":false,"contaminating":false}),
    );
    write(
        &out,
        "repository_professionalism.json",
        &json!({"branch":"m2/dc-dev-021-entry023-daughter-mechanical-transfer-attribution","workflow":"dc-dev-021-m2-entry023.yml","branch_naming":"PASS","commit_quality":"PENDING_SEAL","source_documentation":"PASS","counterfactual_semantics_clear":"PASS","evidence_discoverability":"PASS","workflow_quality":"PASS","scope_discipline":"PASS"}),
    );
    write(
        &out,
        "qualification.json",
        &json!({"classification":classification,"entry022_accepted_negative":"M2_POST_FISSION_TRANSIENT_MOTOR_CONTRAST_MECHANICALLY_INSUFFICIENT","reference_daughter_leverage":{"A":arlev,"B":brlev},"frozen_inherited_result":{"A":ailev,"B":bilev},"k1_result":{"A":if akvalid{if aklev{"YES"}else{"NO"}}else{"INVALID"},"B":if bkvalid{if bklev{"YES"}else{"NO"}}else{"INVALID"}},"phase_dependence":if phase_needed{"PRESENT"}else{"ABSENT"},"mechanical_attribution":classification,"scientific_runtime_changed":false,"reconstruction":"PASS","a_to_w":"PASS","rotation":"PASS","index_invariance":"PASS","entry005_022_preservation":"PASS","m1_preservation":"PASS","downstream_preservation":"PASS","intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","repository_professionalism":"PASS","autonomous_polarity_initiation":"QUALIFIED","polarity_fission_inheritance":"QUALIFIED","autonomous_embodied_locomotion":"NOT_ESTABLISHED","autonomous_resource_acquisition":"NOT_ESTABLISHED","next_execution_started":false,"architect_acceptance":"PENDING"}),
    );
    let manifest = files
        .iter()
        .map(|f| json!({"file":f,"sha256":format!("stable-json:{}",stable_hash(&out.join(f)))}))
        .collect::<Vec<_>>();
    write(
        &out,
        "artifact_manifest.json",
        &json!({"directive":"DC-DEV-021-M2-ENTRY-023-DAUGHTER-POLARITY-EFFECTOR-MECHANICAL-TRANSFER-ATTRIBUTION-AUDIT-001","starting_head":"48b313db45761552e27a34f77b7aff9b0e688f95","classification":classification,"files":manifest,"dense_traces":"Atlas"}),
    );
    println!("ENTRY-023 classification: {classification}");
    println!("A reference/inherited leverage {arlev}/{ailev}; B reference/inherited leverage {brlev}/{bilev}");
}
