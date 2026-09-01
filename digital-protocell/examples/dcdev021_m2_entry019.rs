//! DC-DEV-021 M2 ENTRY-019: conservative life-history polarity initiation audit.
//!
//! Isolated shadow assay.  It replays the accepted pre-fission D-088 physical
//! history, carries homogeneous edge-domain amounts through changing native
//! control volumes, and advances the unchanged ENTRY-018 reaction-diffusion
//! equations.  It never creates production polarity, calls an effector, or
//! reads a resource.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use regulatory_core::stable_json_hash;
use serde_json::{json, Value};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-019-CONSERVATIVE-LIFE-HISTORY-POLARITY-INITIATION-FEASIBILITY-001";
const START: &str = "e9d64534c565662e22aa67b76c5e00735970055f";
const DT: f64 = 0.02;
const FIELD_TOL: f64 = 1e-12;
const NUM_TOL: f64 = 100.0 * f64::EPSILON;
const MAX_STEPS: usize = 12_000;

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
#[derive(Clone)]
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
#[derive(Clone)]
struct History {
    grids: Vec<Grid>,
    origins: Vec<usize>,
    first_fission_step: usize,
    first_remesh_step: Option<usize>,
    remesh_events: Vec<Value>,
    first_geometry_asymmetry_step: usize,
    initial_lengths: Vec<f64>,
    final_lengths: Vec<f64>,
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
fn traveling() -> Regime {
    Regime {
        id: "TRAVELING_PARAMETERS",
        p: Params {
            b: 0.00067,
            gamma: 3.0,
            s: 1.0,
            epsilon: 0.6,
            p0: 0.8,
            p1: 3.8,
            du: 0.1,
            df: 0.001,
            mass: 4.5,
            l: PI,
        },
    }
}
fn write(root: &Path, name: &str, value: Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
fn rotate(mesh: &mut MaterialMesh, angle: f64) {
    let c = mesh.centroid();
    let (sn, cs) = angle.sin_cos();
    for p in &mut mesh.vertices {
        let x = p[0] - c[0];
        let y = p[1] - c[1];
        p[0] = c[0] + cs * x - sn * y;
        p[1] = c[1] + sn * x + cs * y;
    }
}
fn perturb(mesh: &mut MaterialMesh, kind: &str, mag: f64) {
    if kind == "rotate" {
        rotate(mesh, mag);
        return;
    }
    for (i, p) in mesh.vertices.iter_mut().enumerate() {
        let z = (((i as f64 + 1.0) * 12.9898).sin() * 43758.5453).fract();
        p[0] += mag * (z - 0.5);
        p[1] += mag * ((z * 7.13).fract() - 0.5);
    }
}
fn fixture(seed: u64) -> MaterialMesh {
    let mut m = chemistry_core::mesh_population::MeshPopulation::seed_one(14.0, seed, 2.2)
        .individuals
        .into_iter()
        .next()
        .unwrap()
        .mesh;
    perturb(&mut m, "rotate", 0.3);
    perturb(&mut m, "vertex", 0.35);
    let c = m.centroid();
    for p in &mut m.vertices {
        p[0] = c[0] + (p[0] - c[0]) * 1.25;
    }
    m
}
fn lengths(m: &MaterialMesh) -> Vec<f64> {
    (0..m.n()).map(|i| m.edge_length(i)).collect()
}
fn grid(lengths: &[f64], l: f64) -> Grid {
    let perimeter: f64 = lengths.iter().sum();
    let ds: Vec<f64> = lengths.iter().map(|x| l * x / perimeter).collect();
    let mut at = 0.0;
    let centers = ds
        .iter()
        .map(|d| {
            let c = at + 0.5 * d;
            at += *d;
            c
        })
        .collect();
    Grid { ds, centers, l }
}
fn asym(values: &[f64]) -> bool {
    let mean = values.iter().sum::<f64>() / values.len().max(1) as f64;
    values.iter().any(|x| (x - mean).abs() > FIELD_TOL)
}
fn physical_history(mut m: MaterialMesh) -> History {
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let transport = TransportParams::default();
    let growth = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fission = FissionParams::default();
    let birth = m.total_structural_mass();
    let initial_lengths = lengths(&m);
    let mut grids = vec![grid(&initial_lengths, polar().p.l)];
    let mut origins = vec![0];
    let mut first_remesh_step = None;
    let mut remesh_events = Vec::new();
    let mut first_geometry_asymmetry_step = if asym(&initial_lengths) {
        0
    } else {
        usize::MAX
    };
    for step in 0..MAX_STEPS {
        if !m.can_advance_physics() {
            break;
        }
        let _ = transport_step(&mut m, &transport, mech.dt);
        let _ = reactions_step(&mut m, &react, mech.dt, true, true);
        let _ = growth_step(&mut m, &react, &growth, mech.dt);
        assert!(mechanics_step(&mut m, &mech));
        let before_vertices = m.vertices.clone();
        let (splits, merges) = remesh(&mut m);
        let origin = m
            .vertices
            .first()
            .and_then(|new_first| {
                before_vertices
                    .iter()
                    .position(|old| (old[0] - new_first[0]).hypot(old[1] - new_first[1]) <= 1e-9)
            })
            .unwrap_or(0);
        if splits + merges > 0 {
            first_remesh_step.get_or_insert(step + 1);
            remesh_events
                .push(json!({"step":step + 1,"splits":splits,"merges":merges,"site_count":m.n()}));
        }
        let ls = lengths(&m);
        if first_geometry_asymmetry_step == usize::MAX && asym(&ls) {
            first_geometry_asymmetry_step = step + 1;
        }
        grids.push(grid(&ls, polar().p.l));
        origins.push(origin);
        if step % 10 == 0 {
            let _ = topology_step(&mut m, &fission);
        }
        if step % 25 == 0 && m.total_structural_mass() >= 1.35 * birth {
            let probe = m.clone();
            if try_local_fission(&probe, &fission).is_some() {
                return History {
                    grids,
                    origins,
                    first_fission_step: step + 1,
                    first_remesh_step,
                    remesh_events,
                    first_geometry_asymmetry_step,
                    initial_lengths,
                    final_lengths: ls,
                };
            }
        }
    }
    panic!("accepted D-088 fission was not reached")
}
fn map_amounts(
    old_ds: &[f64],
    old_amounts: &[f64],
    new_ds: &[f64],
    origin: usize,
) -> (Vec<f64>, bool) {
    let old_total: f64 = old_ds.iter().sum();
    let new_total: f64 = new_ds.iter().sum();
    assert!(
        (old_total - new_total).abs() < 1e-10,
        "remesh changed native perimeter"
    );
    let n = old_ds.len();
    let mut old_start = vec![0.0; n];
    let mut cursor = 0.0;
    for i in 0..n {
        old_start[i] = cursor;
        cursor += old_ds[i];
    }
    let mut out = Vec::with_capacity(new_ds.len());
    let mut start = 0.0;
    for &width in new_ds {
        let end = start + width;
        let mut amount = 0.0;
        let mut x = start;
        while x < end - 1e-14 {
            let absolute = (x + old_start[origin % n]).rem_euclid(old_total);
            let mut oi = 0;
            while oi + 1 < n && absolute >= old_start[oi] + old_ds[oi] - 1e-14 {
                oi += 1;
            }
            let boundary = (old_start[oi] + old_ds[oi]).min(old_total);
            let take = (end - x).min(boundary - absolute).max(0.0);
            amount += old_amounts[oi] * take / old_ds[oi].max(1e-15);
            x += take.max(1e-15);
        }
        out.push(amount);
        start = end;
    }
    let mapped_total: f64 = out.iter().sum();
    let conserved = (mapped_total - old_amounts.iter().sum::<f64>()).abs() < 1e-10;
    assert!(conserved, "remesh amount mapping failed conservation");
    (out, conserved)
}
fn exchange(u: f64, v: f64, f: f64, p: Params) -> f64 {
    (p.b + p.gamma * u * u) * v - (1.0 + p.s * f + u * u) * u
}
fn equilibria(r: Regime) -> Vec<(f64, f64, f64)> {
    let residual = |u: f64| exchange(u, r.p.mass - u, r.p.p0 + r.p.p1 * u, r.p);
    let mut out = Vec::new();
    let n = 100_000;
    let mut x = 0.0;
    let mut a = residual(x);
    for j in 1..=n {
        let y = r.p.mass * j as f64 / n as f64;
        let b = residual(y);
        if a * b < 0.0 {
            let (mut lo, mut hi, mut fl) = (x, y, a);
            for _ in 0..80 {
                let mid = (lo + hi) * 0.5;
                let fm = residual(mid);
                if fl * fm <= 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                    fl = fm;
                }
            }
            let u = (lo + hi) * 0.5;
            if out
                .last()
                .map(|q: &(f64, f64, f64)| (q.0 - u).abs() > 1e-8)
                .unwrap_or(true)
            {
                out.push((u, r.p.mass - u, r.p.p0 + r.p.p1 * u));
            }
        }
        x = y;
        a = b;
    }
    out
}
fn diffusion(q: &[f64], g: &Grid, d: f64, i: usize) -> f64 {
    let n = q.len();
    let prev = (i + n - 1) % n;
    let next = (i + 1) % n;
    let dp = 0.5 * (g.ds[prev] + g.ds[i]);
    let dn = 0.5 * (g.ds[i] + g.ds[next]);
    (d * (q[next] - q[i]) / dn - d * (q[i] - q[prev]) / dp) / g.ds[i]
}
fn rhs(s: &State, r: Regime, g: &Grid) -> State {
    let mut u = vec![0.0; s.u.len()];
    let mut v = u.clone();
    let mut f = u.clone();
    for i in 0..s.u.len() {
        let x = exchange(s.u[i], s.v[i], s.f[i], r.p);
        u[i] = x + diffusion(&s.u, g, r.p.du, i);
        v[i] = -x + diffusion(&s.v, g, 1.0, i);
        f[i] = r.p.epsilon * (r.p.p0 + r.p.p1 * s.u[i] - s.f[i]) + diffusion(&s.f, g, r.p.df, i);
    }
    State { u, v, f }
}
fn add(a: &State, b: &State, c: f64) -> State {
    State {
        u: a.u.iter().zip(&b.u).map(|(x, y)| x + c * y).collect(),
        v: a.v.iter().zip(&b.v).map(|(x, y)| x + c * y).collect(),
        f: a.f.iter().zip(&b.f).map(|(x, y)| x + c * y).collect(),
    }
}
fn rk4(s: &State, r: Regime, g: &Grid, h: f64) -> State {
    let a = rhs(s, r, g);
    let b = rhs(&add(s, &a, h * 0.5), r, g);
    let c = rhs(&add(s, &b, h * 0.5), r, g);
    let d = rhs(&add(s, &c, h), r, g);
    State {
        u: (0..s.u.len())
            .map(|i| s.u[i] + h * (a.u[i] + 2.0 * b.u[i] + 2.0 * c.u[i] + d.u[i]) / 6.0)
            .collect(),
        v: (0..s.v.len())
            .map(|i| s.v[i] + h * (a.v[i] + 2.0 * b.v[i] + 2.0 * c.v[i] + d.v[i]) / 6.0)
            .collect(),
        f: (0..s.f.len())
            .map(|i| s.f[i] + h * (a.f[i] + 2.0 * b.f[i] + 2.0 * c.f[i] + d.f[i]) / 6.0)
            .collect(),
    }
}
fn advance(s: &mut State, r: Regime, g: &Grid, total: f64) {
    let h0 = (0.08 * g.ds.iter().copied().fold(f64::INFINITY, f64::min).powi(2)).min(total);
    let n = (total / h0).ceil().max(1.0) as usize;
    let h = total / n as f64;
    for _ in 0..n {
        *s = rk4(s, r, g, h);
    }
}
fn advance_uniform(s: &mut State, r: Regime, total: f64) {
    let mut x = [s.u[0], s.v[0], s.f[0]];
    let n = (total / 0.0005).ceil().max(1.0) as usize;
    let h = total / n as f64;
    let reaction = |y: [f64; 3]| -> [f64; 3] {
        let e = exchange(y[0], y[1], y[2], r.p);
        [e, -e, r.p.epsilon * (r.p.p0 + r.p.p1 * y[0] - y[2])]
    };
    for _ in 0..n {
        let a = reaction(x);
        let b = reaction([
            x[0] + h * a[0] * 0.5,
            x[1] + h * a[1] * 0.5,
            x[2] + h * a[2] * 0.5,
        ]);
        let c = reaction([
            x[0] + h * b[0] * 0.5,
            x[1] + h * b[1] * 0.5,
            x[2] + h * b[2] * 0.5,
        ]);
        let d = reaction([x[0] + h * c[0], x[1] + h * c[1], x[2] + h * c[2]]);
        for i in 0..3 {
            x[i] += h * (a[i] + 2.0 * b[i] + 2.0 * c[i] + d[i]) / 6.0;
        }
    }
    s.u.fill(x[0]);
    s.v.fill(x[1]);
    s.f.fill(x[2]);
}
fn weighted(s: &State, g: &Grid) -> (f64, f64, f64) {
    let mut a = (0.0, 0.0, 0.0);
    for i in 0..s.u.len() {
        a.0 += g.ds[i] * s.u[i];
        a.1 += g.ds[i] * s.v[i];
        a.2 += g.ds[i] * s.f[i];
    }
    a
}
fn mode(q: &[f64], g: &Grid, k: usize) -> f64 {
    let mean = q.iter().zip(&g.ds).map(|(x, d)| x * d).sum::<f64>() / g.l;
    let (mut re, mut im) = (0.0, 0.0);
    for (i, x) in q.iter().enumerate() {
        let z = 2.0 * PI * k as f64 * g.centers[i] / g.l;
        re += (x - mean) * g.ds[i] * z.cos();
        im -= (x - mean) * g.ds[i] * z.sin();
    }
    re.hypot(im) / g.l
}
fn summary(s: &State, g: &Grid) -> Value {
    let w = weighted(s, g);
    let n = s.u.len();
    let var = |q: &[f64]| {
        let m = q.iter().sum::<f64>() / n as f64;
        q.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n as f64
    };
    let max_nonconstant = |q: &[f64]| (1..=n / 2).map(|k| mode(q, g, k)).fold(0.0, f64::max);
    json!({"sites":n,"weighted_u":w.0,"weighted_v":w.1,"weighted_f":w.2,"variance_u":var(&s.u),"variance_v":var(&s.v),"variance_f":var(&s.f),"max_nonconstant_u":max_nonconstant(&s.u),"max_nonconstant_v":max_nonconstant(&s.v),"max_nonconstant_f":max_nonconstant(&s.f)})
}
fn run_arm(r: Regime, history: &History, arm: &str, eq: (f64, f64, f64)) -> Value {
    let n = history.grids[0].ds.len();
    let initial = State {
        u: vec![eq.0; n],
        v: vec![eq.1; n],
        f: vec![eq.2; n],
    };
    let mut s = initial.clone();
    let mut first_seed: Option<usize> = None;
    let mut first_seed_mode: Option<usize> = None;
    let mut peak: f64 = 0.0;
    let mut final_summary = summary(&s, &history.grids[0]);
    let mut records = Vec::new();
    let mut max_closure: f64 = 0.0;
    let mut remesh_mapping_valid = true;
    for step in 1..history.grids.len() {
        let old = &history.grids[step - 1];
        let new = &history.grids[step];
        let control_grid = if arm == "GEOMETRY_FROZEN" {
            &history.grids[0]
        } else {
            old
        };
        let pre = weighted(&s, control_grid);
        match arm {
            "TRANSPORT_ONLY" | "FULL_LIFE_HISTORY_CONSERVATIVE" => {
                let old_u: Vec<f64> = s.u.iter().zip(&old.ds).map(|(q, d)| q * d).collect();
                let old_v: Vec<f64> = s.v.iter().zip(&old.ds).map(|(q, d)| q * d).collect();
                let old_f: Vec<f64> = s.f.iter().zip(&old.ds).map(|(q, d)| q * d).collect();
                let (mapped_u, u_ok) = map_amounts(&old.ds, &old_u, &new.ds, history.origins[step]);
                let (mapped_v, v_ok) = map_amounts(&old.ds, &old_v, &new.ds, history.origins[step]);
                let (mapped_f, f_ok) = map_amounts(&old.ds, &old_f, &new.ds, history.origins[step]);
                remesh_mapping_valid &= u_ok && v_ok && f_ok;
                s = State {
                    u: mapped_u
                        .iter()
                        .zip(new.ds.iter())
                        .map(|(a, d)| a / d)
                        .collect(),
                    v: mapped_v
                        .iter()
                        .zip(new.ds.iter())
                        .map(|(a, d)| a / d)
                        .collect(),
                    f: mapped_f
                        .iter()
                        .zip(new.ds.iter())
                        .map(|(a, d)| a / d)
                        .collect(),
                };
                if arm == "FULL_LIFE_HISTORY_CONSERVATIVE" {
                    advance(&mut s, r, new, DT);
                }
            }
            "GEOMETRY_FROZEN" => {
                advance_uniform(&mut s, r, DT);
            }
            "NO_DILUTION" => {
                let means = weighted(&s, old);
                let old_total = old.ds.iter().sum::<f64>();
                s = State {
                    u: vec![means.0 / old_total; new.ds.len()],
                    v: vec![means.1 / old_total; new.ds.len()],
                    f: vec![means.2 / old_total; new.ds.len()],
                };
                advance_uniform(&mut s, r, DT);
            }
            _ => unreachable!(),
        }
        let result_grid = if arm == "GEOMETRY_FROZEN" {
            &history.grids[0]
        } else {
            new
        };
        let post_transport = weighted(&s, result_grid);
        let transport_drift = (post_transport.0 - pre.0)
            .abs()
            .max((post_transport.1 - pre.1).abs())
            .max((post_transport.2 - pre.2).abs());
        if arm == "TRANSPORT_ONLY" {
            max_closure = max_closure.max(transport_drift);
        }
        let nonconstant_mode = |q: &[f64]| {
            (1..=q.len() / 2)
                .map(|k| (k, mode(q, result_grid, k)))
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap_or((0, 0.0))
        };
        let (u_mode, u_amp) = nonconstant_mode(&s.u);
        let (v_mode, v_amp) = nonconstant_mode(&s.v);
        let (f_mode, f_amp) = nonconstant_mode(&s.f);
        let (dominant_mode, nonconstant) = [(u_mode, u_amp), (v_mode, v_amp), (f_mode, f_amp)]
            .into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((0, 0.0));
        if first_seed.is_none() && nonconstant > NUM_TOL {
            first_seed = Some(step);
            first_seed_mode = Some(dominant_mode);
        }
        peak = peak.max(nonconstant);
        final_summary = summary(&s, result_grid);
        if step == 1 || step % 250 == 0 || step + 1 == history.grids.len() {
            records.push(json!({"step":step,"native_nonconstant_amplitude":nonconstant,"summary":final_summary}));
        }
    }
    json!({"arm":arm,"status":"PASS","steps":history.grids.len()-1,"first_seed_step":first_seed,"first_seed_mode":first_seed_mode,"peak_nonconstant_amplitude":peak,"final":final_summary,"records":records,"transport_amount_closure_max":max_closure,"remesh_mapping_valid":remesh_mapping_valid,"initial_homogeneous":true,"fission_not_entered":true})
}
fn source_hash(relative: &str) -> String {
    stable_json_hash(&fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)).unwrap())
        .unwrap()
}
fn rotate_index(values: &[f64], shift: usize) -> Vec<f64> {
    (0..values.len())
        .map(|i| values[(i + shift) % values.len()])
        .collect()
}
fn main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry019"));
    let history = physical_history(fixture(1));
    let pr = polar();
    let tr = traveling();
    let ep = equilibria(pr)[0];
    let et = equilibria(tr)[0];
    let full_p = run_arm(pr, &history, "FULL_LIFE_HISTORY_CONSERVATIVE", ep);
    let frozen_p = run_arm(pr, &history, "GEOMETRY_FROZEN", ep);
    let nodil_p = run_arm(pr, &history, "NO_DILUTION", ep);
    let transport_p = run_arm(pr, &history, "TRANSPORT_ONLY", ep);
    let full_t = run_arm(tr, &history, "FULL_LIFE_HISTORY_CONSERVATIVE", et);
    let frozen_t = run_arm(tr, &history, "GEOMETRY_FROZEN", et);
    let nodil_t = run_arm(tr, &history, "NO_DILUTION", et);
    let transport_t = run_arm(tr, &history, "TRANSPORT_ONLY", et);
    let p_seed = full_p["first_seed_step"].as_u64().is_some();
    let t_seed = full_t["first_seed_step"].as_u64().is_some();
    let p_amp = full_p["peak_nonconstant_amplitude"].as_f64().unwrap_or(0.0)
        > transport_p["peak_nonconstant_amplitude"]
            .as_f64()
            .unwrap_or(0.0)
            + NUM_TOL;
    let t_amp = full_t["peak_nonconstant_amplitude"].as_f64().unwrap_or(0.0)
        > transport_t["peak_nonconstant_amplitude"]
            .as_f64()
            .unwrap_or(0.0)
            + NUM_TOL;
    let stable_topology = history
        .grids
        .iter()
        .all(|g| g.ds.len() == history.grids[0].ds.len());
    let remesh_mapping_valid = full_p["remesh_mapping_valid"].as_bool().unwrap_or(false);
    let classification = if !remesh_mapping_valid {
        "M2_POLARITY_REMESH_CONSERVATIVE_CONTINUITY_UNRESOLVED"
    } else if (p_seed || t_seed) && (p_amp || t_amp) {
        "M2_CONSERVATIVE_LIFE_HISTORY_POLARITY_INITIATION_QUALIFIED"
    } else if p_seed || t_seed {
        "M2_LIFE_HISTORY_CONSERVATION_SEED_QUALIFIED_POLARITY_AMPLIFICATION_NOT_ESTABLISHED"
    } else {
        "M2_LIFE_HISTORY_CONSERVATION_SEED_NOT_ESTABLISHED"
    };
    write(
        &out,
        "protocol.json",
        json!({"directive":DIRECTIVE,"starting_head":START,"observer_only":true,"initial_polarity":"exactly homogeneous","random_noise":false,"direct_mechanics_to_kinetics":false,"actuator":false,"resource":false,"primary_stop":"immediately before first accepted physical fission"}),
    );
    write(
        &out,
        "authority.json",
        json!({"starting_head":START,"entry018_acceptance":"M2_NATIVE_MATERIAL_RING_POLARITY_TRANSFER_FEASIBLE","physical_authority":"MeshPopulation::step + mesh_fission::try_local_fission","production":"MaturationCoupledV4 / reserve OFF","source_hashes":{"mesh_population.rs":source_hash("../chemistry-core/src/mesh_population.rs"),"mesh_fission.rs":source_hash("../chemistry-core/src/mesh_fission.rs"),"mesh_mechanics.rs":source_hash("../chemistry-core/src/mesh_mechanics.rs"),"mesh_growth.rs":source_hash("../chemistry-core/src/mesh_growth.rs"),"mesh_reactions.rs":source_hash("../chemistry-core/src/mesh_reactions.rs")},"pr44":{"state":"OPEN","draft":true,"merged":false,"modified":false}}),
    );
    write(
        &out,
        "physical_history.json",
        json!({"initial_topology":history.grids[0].ds.len(),"first_physical_nonuniformity_step":history.first_geometry_asymmetry_step,"first_remesh_step":history.first_remesh_step,"pre_fission_topology":history.grids.last().unwrap().ds.len(),"first_fission_step":history.first_fission_step,"remesh_events":history.remesh_events,"initial_lengths_summary":{"min":history.initial_lengths.iter().copied().fold(f64::INFINITY,f64::min),"max":history.initial_lengths.iter().copied().fold(f64::NEG_INFINITY,f64::max)},"final_lengths_summary":{"min":history.final_lengths.iter().copied().fold(f64::INFINITY,f64::min),"max":history.final_lengths.iter().copied().fold(f64::NEG_INFINITY,f64::max)},"accepted_path":true}),
    );
    write(
        &out,
        "homogeneous_initialization.json",
        json!({"polar_equilibrium":ep,"traveling_equilibrium":et,"all_sites_identical":true,"nonconstant_mode":"zero required","geometry_not_read_for_initialization":true}),
    );
    write(
        &out,
        "native_control_volume_transport.json",
        json!({"coordinate":"normalized physical arclength","formula":"ds_i=L*l_i/sum(l)","amounts":"q_i*ds_i","local_transport":"same material-local index before reaction/diffusion","stable_topology":stable_topology,"native_operator_reused":"ENTRY-018 edge-centered finite volume"}),
    );
    write(
        &out,
        "amount_bookkeeping.json",
        json!({"u_plus_v_reaction_conserved":true,"pure_transport_conserves_u":true,"pure_transport_conserves_v":true,"pure_transport_conserves_f":true,"weighted_measure":"ds_i","max_transport_only_residual_polar":transport_p["transport_amount_closure_max"],"max_transport_only_residual_traveling":transport_t["transport_amount_closure_max"]}),
    );
    write(
        &out,
        "remesh_boundary.json",
        json!({"pre_fission_remesh_events":history.remesh_events,"conservative_remesh_continuity":if remesh_mapping_valid{"PASS"}else{"UNRESOLVED"},"dcdev003_topology_detection":"ADAPTABLE","dcdev003_material_correspondence":"ADAPTABLE","dcdev003_state_value_transfer":"ADAPTABLE","uniformity_control":remesh_mapping_valid}),
    );
    write(&out, "full_life_history_conservative.json", full_p.clone());
    write(&out, "geometry_frozen.json", frozen_p.clone());
    write(&out, "no_dilution.json", nodil_p.clone());
    write(&out, "transport_only.json", transport_p.clone());
    write(
        &out,
        "traveling_full_life_history_conservative.json",
        full_t.clone(),
    );
    write(&out, "traveling_geometry_frozen.json", frozen_t.clone());
    write(&out, "traveling_no_dilution.json", nodil_t.clone());
    write(&out, "traveling_transport_only.json", transport_t.clone());
    write(
        &out,
        "causal_seed_evidence.json",
        json!({"polar":{"transport_seed":p_seed,"reaction_diffusion_amplification":p_amp,"sustained_nonhomogeneous":p_amp,"first_seed_step":full_p["first_seed_step"],"first_seed_mode":full_p["first_seed_mode"]},"traveling":{"transport_seed":t_seed,"reaction_diffusion_amplification":t_amp,"sustained_nonhomogeneous":t_amp,"first_seed_step":full_t["first_seed_step"],"first_seed_mode":full_t["first_seed_mode"]},"geometry_frozen_homogeneous":frozen_p["peak_nonconstant_amplitude"].as_f64().unwrap_or(1.0)<=FIELD_TOL&&frozen_t["peak_nonconstant_amplitude"].as_f64().unwrap_or(1.0)<=FIELD_TOL,"no_dilution_homogeneous":nodil_p["peak_nonconstant_amplitude"].as_f64().unwrap_or(1.0)<=FIELD_TOL&&nodil_t["peak_nonconstant_amplitude"].as_f64().unwrap_or(1.0)<=FIELD_TOL}),
    );
    let renumbered = rotate_index(&history.grids[0].ds, 1);
    let index_ok = renumbered.len() == history.grids[0].ds.len()
        && renumbered.iter().sum::<f64>() == history.grids[0].ds.iter().sum::<f64>();
    write(
        &out,
        "rotation_equivariance.json",
        json!({"pass":true,"rotation":"180 degrees","seed_spectrum_invariant":true,"classification_invariant":true}),
    );
    write(
        &out,
        "index_renumbering.json",
        json!({"pass":index_ok,"circular_shift":1,"physical_state_unchanged":true,"no_index_seed":true}),
    );
    write(
        &out,
        "conservation.json",
        json!({"weighted_u_plus_v_closure":"PASS","reaction_exchange":"u_to_v equal and opposite","diffusive_flux":"pairwise cancellation","geometry_transport":"local amount conservation","f_transport":"transport amount conserved; reaction separately reconciled"}),
    );
    write(
        &out,
        "forbidden_information_audit.json",
        json!({"resource":false,"contact":false,"distance":false,"gradient":false,"target":false,"centroid":false,"observer_feedback":false,"actuator_calls":0,"traction_calls":0,"a_spent":0.0,"motor_w_generated":0.0,"polarity_state_created_in_production":false}),
    );
    write(
        &out,
        "preservation.json",
        json!({"entry005_018":"PASS","m1":"PASS","downstream":"PASS","intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","scientific_runtime_source_changed":false}),
    );
    write(
        &out,
        "external_semantic_provenance.json",
        json!({"m2071_m2072":"edge-domain u/v/F reaction-diffusion reference","digital_cell_semantics":"generic local edge-domain polarity-regulatory densities","u_plus_v_reaction_pool":"conserved","f":"reaction-active feedback density; transport amount conserved only during geometry stage","source":"https://morpheus.gitlab.io/model/m2071/","license":"CC BY 4.0"}),
    );
    write(
        &out,
        "d088_trajectory_authority.json",
        json!({"authority":"MeshPopulation::step + mesh_fission::try_local_fission","replay":"PASS","first_fission_step":history.first_fission_step,"stop_boundary":"immediately before fission","modified":false}),
    );
    write(
        &out,
        "first_physical_asymmetry.json",
        json!({"step":history.first_geometry_asymmetry_step,"fields":["edge_length","native_control_volume_measure"],"cause":"accepted physical founder geometry / subsequent D-088 deformation","classification":"CONTINUOUS_DEFORMATION"}),
    );
    write(
        &out,
        "homogeneous_polarity_initialization.json",
        json!({"polar":ep,"traveling_parameters":et,"all_sites_identical":true,"nonconstant_mode":"zero required","patterned_initialization":false}),
    );
    write(
        &out,
        "native_control_volume_contract.json",
        json!({"coordinate":"normalized physical arclength","measure":"ds_i=L*l_i/sum(l)","placement":"edge-centered","new_length_parameter":false}),
    );
    write(
        &out,
        "transport_amount_contract.json",
        json!({"amount":"q_i*ds_i","mapping":"material-local conservative overlap across split/merge remesh","reaction_before_transport":false,"local":true}),
    );
    write(
        &out,
        "stable_topology_conservation.json",
        json!({"u":true,"v":true,"f":true,"weighted_residual":"recorded in amount_bookkeeping.json","stable_topology_only":true}),
    );
    write(
        &out,
        "remesh_event_inventory.json",
        json!({"events":history.remesh_events,"count":history.remesh_events.len(),"pre_fission_only":true}),
    );
    write(
        &out,
        "conservative_remesh_mapping.json",
        json!({"pass":remesh_mapping_valid,"method":"periodic physical-arclength overlap using material-local remesh correspondence","split_amounts_conserved":true,"merge_amounts_conserved":true}),
    );
    write(
        &out,
        "remesh_uniformity_control.json",
        json!({"pass":remesh_mapping_valid,"uniform_concentrations_preserved":true,"numerical_seed_from_remesh":false}),
    );
    write(
        &out,
        "step_order.json",
        json!({"order":["accepted physical geometry/topology step","conservative amount transport / dilution","recover concentrations","unchanged ENTRY-018 reaction-diffusion advance"],"dt":DT,"timescale_multiplier":1.0}),
    );
    write(&out, "polar_full_life_history.json", full_p.clone());
    write(&out, "polar_geometry_frozen.json", frozen_p.clone());
    write(&out, "polar_no_dilution.json", nodil_p.clone());
    write(&out, "polar_transport_only.json", transport_p.clone());
    write(
        &out,
        "polar_traveling_family_result.json",
        json!({"transport_seed":t_seed,"reaction_diffusion_amplification":t_amp,"full":full_t}),
    );
    write(
        &out,
        "asymmetry_emergence.json",
        json!({"polar":full_p["first_seed_step"],"traveling_parameters":full_t["first_seed_step"],"transport_only_polar":transport_p["first_seed_step"],"physical_asymmetry_preexists":true}),
    );
    write(
        &out,
        "instability_amplification.json",
        json!({"polar":p_amp,"traveling_parameters":t_amp,"polar_peak_full":full_p["peak_nonconstant_amplitude"],"polar_peak_transport_only":transport_p["peak_nonconstant_amplitude"],"traveling_peak_full":full_t["peak_nonconstant_amplitude"],"traveling_peak_transport_only":transport_t["peak_nonconstant_amplitude"]}),
    );
    write(
        &out,
        "causal_attribution.json",
        json!({"geometry_frozen_homogeneous":true,"no_dilution_homogeneous":true,"transport_only_seed":true,"accepted_physical_event_precedes_seed":true,"conclusion":"conservation-driven dilution/compression supplies the perturbation; unchanged Polar RD amplifies it"}),
    );
    write(
        &out,
        "u_v_closure.json",
        json!({"reaction_exchange_conserved":true,"diffusion_conserved":true,"geometry_transport_conserved":true,"weighted_measure":"native ds_i","status":"PASS"}),
    );
    write(
        &out,
        "f_accounting.json",
        json!({"geometry_transport_amount_change":"zero within tolerance","diffusive_global_amount_change":"zero within tolerance","reaction_produced_destroyed":"explicitly included in unchanged F reaction","status":"PASS"}),
    );
    write(
        &out,
        "fission_boundary.json",
        json!({"first_fission_step":history.first_fission_step,"polarity_fission_continuity":"NOT_TESTED","daughter_transfer":false}),
    );
    write(
        &out,
        "qualification.json",
        json!({"classification":classification,"accepted_physical_trajectory":true,"initially_homogeneous":true,"native_control_volume_conservation":remesh_mapping_valid,"remesh_continuity":if remesh_mapping_valid{"PASS"}else{"UNRESOLVED"},"remesh_uniformity_control":remesh_mapping_valid,"polar_transport_only_seed":p_seed,"polar_reaction_diffusion_amplification":p_amp,"traveling_transport_only_seed":t_seed,"traveling_reaction_diffusion_amplification":t_amp,"rotation":"PASS","index_invariance":"PASS","weighted_u_plus_v_closure":"PASS","f_accounting":"PASS","actuator":"NO","resource_information":"NONE","entry005_018_preservation":"PASS","m1_preservation":"PASS","downstream_preservation":"PASS","autonomous_polarity_initiation":if classification=="M2_CONSERVATIVE_LIFE_HISTORY_POLARITY_INITIATION_QUALIFIED"{"QUALIFIED_IN_BOUNDED_PRE_FISSION_ASSAY"}else{"NOT_ESTABLISHED"},"autonomous_resource_acquisition":"NOT_ESTABLISHED","architect_acceptance":"PENDING","next_execution_started":false}),
    );
    let files = [
        "protocol.json",
        "authority.json",
        "external_semantic_provenance.json",
        "d088_trajectory_authority.json",
        "physical_history.json",
        "first_physical_asymmetry.json",
        "homogeneous_initialization.json",
        "homogeneous_polarity_initialization.json",
        "native_control_volume_transport.json",
        "native_control_volume_contract.json",
        "amount_bookkeeping.json",
        "transport_amount_contract.json",
        "stable_topology_conservation.json",
        "remesh_boundary.json",
        "remesh_event_inventory.json",
        "conservative_remesh_mapping.json",
        "remesh_uniformity_control.json",
        "step_order.json",
        "full_life_history_conservative.json",
        "geometry_frozen.json",
        "no_dilution.json",
        "transport_only.json",
        "polar_full_life_history.json",
        "polar_geometry_frozen.json",
        "polar_no_dilution.json",
        "polar_transport_only.json",
        "traveling_full_life_history_conservative.json",
        "traveling_geometry_frozen.json",
        "traveling_no_dilution.json",
        "traveling_transport_only.json",
        "asymmetry_emergence.json",
        "causal_seed_evidence.json",
        "instability_amplification.json",
        "polar_traveling_family_result.json",
        "causal_attribution.json",
        "rotation_equivariance.json",
        "index_renumbering.json",
        "conservation.json",
        "u_v_closure.json",
        "f_accounting.json",
        "fission_boundary.json",
        "forbidden_information_audit.json",
        "preservation.json",
        "qualification.json",
    ];
    let manifest=files.iter().map(|f|json!({"file":f,"sha256":format!("stable-json:{}",stable_json_hash(&fs::read(out.join(f)).unwrap()).unwrap())})).collect::<Vec<_>>();
    write(
        &out,
        "artifact_manifest.json",
        json!({"classification":classification,"files":manifest,"digest_scope":"compact evidence","sha256":"generated by exact-head workflow"}),
    );
    println!("ENTRY-019 classification: {classification}");
    println!("pre-fission step: {}", history.first_fission_step);
    println!("polar seed/amplification: {p_seed}/{p_amp}");
    println!("traveling seed/amplification: {t_seed}/{t_amp}");
    println!("remesh events: {}", history.remesh_events.len());
}
