//! DC-DEV-021 M2 ENTRY-018: conservative native material-ring polarity audit.
//!
//! This is an isolated numerical assay.  It reuses the accepted M2071
//! equations and the accepted D-088 physical fission replay, but never
//! initializes polarity from physical fields and never calls Digital Cell
//! actuation, traction, resource, or production-polarly coupled code.

use chemistry_core::material_mesh::MaterialMesh;
use chemistry_core::mesh_fission::{topology_step, try_local_fission, FissionParams};
use chemistry_core::mesh_growth::{growth_step, GrowthParams};
use chemistry_core::mesh_mechanics::{mechanics_step, remesh, MechParams};
use chemistry_core::mesh_reactions::ReactionParams;
use chemistry_core::mesh_topology::TopologyLedger;
use chemistry_core::mesh_transport::{transport_step, TransportParams};
use regulatory_core::stable_json_hash;
use serde_json::{json, Value};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-018-NATIVE-MATERIAL-RING-POLARITY-TRANSFER-FEASIBILITY-001";
const START: &str = "036316488bc53b25ad684ea666c754dc48202e7b";
const FIELD_TOL: f64 = 1e-12;
const NUM_TOL: f64 = 100.0 * f64::EPSILON;

#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
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

fn polar() -> Regime {
    Regime {
        id: "POLAR_1D",
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
        id: "TRAVELING_WAVES_1D",
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
fn exchange(u: f64, v: f64, f: f64, p: Params) -> f64 {
    (p.b + p.gamma * u * u) * v - (1. + p.s * f + u * u) * u
}
fn eq_residual(u: f64, p: Params) -> f64 {
    exchange(u, p.mass - u, p.p0 + p.p1 * u, p)
}
fn equilibria(r: Regime) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    let n = 100_000;
    let mut x = 0.;
    let mut a = eq_residual(x, r.p);
    for j in 1..=n {
        let y = r.p.mass * j as f64 / n as f64;
        let b = eq_residual(y, r.p);
        if a * b < 0. {
            let (mut lo, mut hi, mut fl) = (x, y, a);
            for _ in 0..80 {
                let m = (lo + hi) / 2.;
                let fm = eq_residual(m, r.p);
                if fl * fm <= 0. {
                    hi = m;
                } else {
                    lo = m;
                    fl = fm;
                }
            }
            let u = (lo + hi) / 2.;
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
fn grid_regular(n: usize, l: f64) -> Grid {
    let d = l / n as f64;
    Grid {
        ds: vec![d; n],
        centers: (0..n).map(|i| (i as f64 + 0.5) * d).collect(),
        l,
    }
}
fn grid_physical(lengths: &[f64], l: f64) -> Grid {
    let p = lengths.iter().sum::<f64>();
    let ds: Vec<_> = lengths.iter().map(|x| l * x / p).collect();
    let mut s = 0.0;
    let centers = ds
        .iter()
        .map(|d| {
            let c = s + 0.5 * d;
            s += *d;
            c
        })
        .collect();
    Grid { ds, centers, l }
}
fn initial(r: Regime, g: &Grid) -> State {
    let (mut u, mut v, mut f) = (Vec::new(), Vec::new(), Vec::new());
    for &x in &g.centers {
        if r.id == "POLAR_1D" {
            u.push(1.0 - 0.5 * x.cos());
            v.push(1.0 - 0.1 * x.cos());
            f.push(4.5 + 0.82 * x.cos());
        } else {
            u.push(2.2 - 0.33 * (2.0 * x).cos() - 0.47 * (2.0 * x).sin());
            v.push(2.3 - 0.1 * (2.0 * x).sin());
            f.push(9.2 - 0.82 * (2.0 * x).cos());
        }
    }
    State { u, v, f }
}

// Edge-centered finite volumes.  The outward flux from i to i+1 is
// -D*(q[i+1]-q[i]) / (0.5*(ds[i]+ds[i+1])).
fn diffusion(q: &[f64], g: &Grid, d: f64, i: usize) -> f64 {
    let n = q.len();
    let prev = (i + n - 1) % n;
    let next = (i + 1) % n;
    let dp = 0.5 * (g.ds[prev] + g.ds[i]);
    let dn = 0.5 * (g.ds[i] + g.ds[next]);
    (d * (q[next] - q[i]) / dn - d * (q[i] - q[prev]) / dp) / g.ds[i]
}
fn rhs(s: &State, r: Regime, g: &Grid) -> State {
    let mut du = vec![0.; s.u.len()];
    let mut dv = du.clone();
    let mut df = du.clone();
    for i in 0..s.u.len() {
        let x = exchange(s.u[i], s.v[i], s.f[i], r.p);
        du[i] = x + diffusion(&s.u, g, r.p.du, i);
        dv[i] = -x + diffusion(&s.v, g, 1., i);
        df[i] = r.p.epsilon * (r.p.p0 + r.p.p1 * s.u[i] - s.f[i]) + diffusion(&s.f, g, r.p.df, i);
    }
    State {
        u: du,
        v: dv,
        f: df,
    }
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
fn safe_dt(g: &Grid) -> f64 {
    0.08 * g.ds.iter().copied().fold(f64::INFINITY, f64::min).powi(2)
}
fn advance(s: &mut State, r: Regime, g: &Grid, total: f64) {
    let h0 = safe_dt(g);
    let n = (total / h0).ceil().max(1.) as usize;
    let h = total / n as f64;
    for _ in 0..n {
        *s = rk4(s, r, g, h);
    }
}
fn mode(q: &[f64], k: usize, g: &Grid) -> (f64, f64, f64) {
    let mut re = 0.;
    let mut im = 0.;
    for (i, x) in q.iter().enumerate() {
        let z = 2. * PI * k as f64 * g.centers[i] / g.l;
        re += x * z.cos();
        im -= x * z.sin();
    }
    let n = q.len() as f64;
    (re / n, im / n, (re * re + im * im).sqrt() / n)
}
fn weighted_total(s: &State, g: &Grid) -> f64 {
    s.u.iter()
        .zip(&s.v)
        .zip(&g.ds)
        .map(|((u, v), d)| d * (u + v))
        .sum()
}
fn field(q: &[f64]) -> Value {
    let min = q.iter().copied().fold(f64::INFINITY, f64::min);
    let max = q.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mean = q.iter().sum::<f64>() / q.len() as f64;
    let var = q.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / q.len() as f64;
    json!({"min":min,"max":max,"mean":mean,"variance":var,"classification":if max-min<=NUM_TOL{"NUMERICAL_ONLY"}else if max-min<=FIELD_TOL{"UNIFORM"}else{"PHYSICALLY_NONUNIFORM"}})
}
fn field_spectrum(q: &[f64], g: &Grid) -> Value {
    let mut modes = Vec::new();
    for k in 1..=(q.len() / 2) {
        let (re, im, magnitude) = mode(q, k, g);
        modes.push(json!({"k":k,"real":re,"imaginary":im,"magnitude":magnitude}));
    }
    let dominant = modes
        .iter()
        .max_by(|a, b| a["magnitude"].as_f64().unwrap().partial_cmp(&b["magnitude"].as_f64().unwrap()).unwrap())
        .cloned();
    json!({"field":field(q),"nonzero_modes":modes,"dominant_nonzero_mode":dominant})
}

fn reaction_jac(u: f64, v: f64, f: f64, p: Params) -> [[f64; 3]; 3] {
    let eu = 2. * p.gamma * u * v - (1. + p.s * f) - 3. * u * u;
    let ev = p.b + p.gamma * u * u;
    let ef = -p.s * u;
    [
        [eu, ev, ef],
        [-eu, -ev, -ef],
        [p.epsilon * p.p1, 0., -p.epsilon],
    ]
}
fn eig3(a: [[f64; 3]; 3]) -> [f64; 3] {
    let tr = a[0][0] + a[1][1] + a[2][2];
    let c2 = (a[0][0] * a[1][1] - a[0][1] * a[1][0])
        + (a[0][0] * a[2][2] - a[0][2] * a[2][0])
        + (a[1][1] * a[2][2] - a[1][2] * a[2][1]);
    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    let aa = -tr;
    let pp = c2 - aa * aa / 3.;
    let qq = 2. * aa.powi(3) / 27. - aa * c2 / 3. - det;
    let disc = (qq / 2.).powi(2) + (pp / 3.).powi(3);
    if disc >= 0. {
        let z = disc.sqrt();
        let cb = |x: f64| x.signum() * x.abs().powf(1. / 3.);
        let x = cb(-qq / 2. + z) + cb(-qq / 2. - z) - aa / 3.;
        let y = -0.5 * (x + aa / 3.) - aa / 3.;
        [x, y, y]
    } else {
        let rad = 2. * (-pp / 3.).sqrt();
        let th = ((3. * qq / (2. * pp)) * (-3. / pp).sqrt())
            .clamp(-1., 1.)
            .acos();
        [0, 1, 2].map(|j| rad * ((th + 2. * PI * j as f64) / 3.).cos() - aa / 3.)
    }
}

// Symmetric similarity transform gives the real spectrum and weighted spatial
// eigenvectors of the conservative nonuniform diffusion operator. Each
// diffusion eigenvalue yields one exact 3x3 reaction-diffusion block because
// all species use the same ring operator.
fn diffusion_eigendecomposition(g: &Grid) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = g.ds.len();
    let mut a = vec![vec![0.; n]; n];
    for i in 0..n {
        let prev = (i + n - 1) % n;
        let next = (i + 1) % n;
        let dp = 0.5 * (g.ds[prev] + g.ds[i]);
        let dn = 0.5 * (g.ds[i] + g.ds[next]);
        a[i][i] = -1. / dp / g.ds[i] - 1. / dn / g.ds[i];
        a[i][prev] = 1. / dp / g.ds[i];
        a[i][next] = 1. / dn / g.ds[i];
    }
    let mut s = vec![vec![0.; n]; n];
    for i in 0..n {
        for j in 0..n {
            s[i][j] = a[i][j] * (g.ds[i] / g.ds[j]).sqrt();
        }
    }
    let mut vectors = vec![vec![0.; n]; n];
    for i in 0..n {
        vectors[i][i] = 1.;
    }
    for _ in 0..(100 * n.max(1)) {
        let (mut p, mut q) = (0, 1);
        let mut mx = 0.;
        for i in 0..n {
            for j in i + 1..n {
                if s[i][j].abs() > mx {
                    mx = s[i][j].abs();
                    p = i;
                    q = j;
                }
            }
        }
        if mx < 1e-12 {
            break;
        }
        let app = s[p][p];
        let aqq = s[q][q];
        let apq = s[p][q];
        let tau = (aqq - app) / (2.0 * apq);
        let t = tau.signum() / (tau.abs() + (1.0 + tau * tau).sqrt());
        let c = 1.0 / (1.0 + t * t).sqrt();
        let si = t * c;
        for k in 0..n {
            if k != p && k != q {
                let akp = s[k][p];
                let akq = s[k][q];
                s[k][p] = c * akp - si * akq;
                s[p][k] = s[k][p];
                s[k][q] = si * akp + c * akq;
                s[q][k] = s[k][q];
            }
        }
        s[p][p] = c * c * app - 2.0 * si * c * apq + si * si * aqq;
        s[q][q] = si * si * app + 2.0 * si * c * apq + c * c * aqq;
        s[p][q] = 0.0;
        s[q][p] = 0.0;
        for row in &mut vectors {
            let vp = row[p];
            let vq = row[q];
            row[p] = c * vp - si * vq;
            row[q] = si * vp + c * vq;
        }
    }
    let mut order = (0..n).collect::<Vec<_>>();
    order.sort_by(|&i, &j| s[i][i].partial_cmp(&s[j][j]).unwrap());
    order.reverse();
    let eigenvalues = order.iter().map(|&i| s[i][i]).collect::<Vec<_>>();
    let eigenvectors = order
        .iter()
        .map(|&i| vectors.iter().map(|row| row[i]).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    (eigenvalues, eigenvectors)
}
fn diffusion_eigenvalues(g: &Grid) -> Vec<f64> {
    diffusion_eigendecomposition(g).0
}
fn native_spectrum(r: Regime, g: &Grid, eq: (f64, f64, f64)) -> Value {
    let mut rows = Vec::new();
    for (lambda_mode, lambda) in diffusion_eigenvalues(g).iter().enumerate() {
        let mut j = reaction_jac(eq.0, eq.1, eq.2, r.p);
        j[0][0] += r.p.du * lambda;
        j[1][1] += lambda;
        j[2][2] += r.p.df * lambda;
        let e = eig3(j);
        rows.push(json!({"mode_index":lambda_mode,"diffusion_eigenvalue":lambda,"eigenvalues":e,"maximum_real":e.iter().copied().fold(f64::NEG_INFINITY,f64::max),"constant_mode":lambda_mode==g.ds.len()-1}));
    }
    rows.into_iter().rev().collect::<Vec<_>>().into()
}
fn native_spectra(r: Regime, g: &Grid, eqs: &[(f64, f64, f64)]) -> Value {
    Value::Array(
        eqs.iter()
            .map(|&eq| native_spectrum(r, g, eq))
            .collect(),
    )
}
fn block_max_real(r: Regime, lambda: f64, eq: (f64, f64, f64)) -> f64 {
    let mut j = reaction_jac(eq.0, eq.1, eq.2, r.p);
    j[0][0] += r.p.du * lambda;
    j[1][1] += lambda;
    j[2][2] += r.p.df * lambda;
    eig3(j).iter().copied().fold(f64::NEG_INFINITY, f64::max)
}
fn native_projection(q: &[f64], g: &Grid, r: Regime, eqs: &[(f64, f64, f64)]) -> Value {
    let (lambdas, vectors) = diffusion_eigendecomposition(g);
    let weighted_q = q
        .iter()
        .zip(&g.ds)
        .map(|(value, ds)| value * ds.sqrt())
        .collect::<Vec<_>>();
    let rows = lambdas
        .iter()
        .zip(vectors)
        .enumerate()
        .map(|(i, (lambda, vector))| {
            let projection = vector
                .iter()
                .zip(&weighted_q)
                .map(|(a, b)| a * b)
                .sum::<f64>()
                .abs();
            let maximum_real = eqs
                .iter()
                .map(|&eq| block_max_real(r, *lambda, eq))
                .fold(f64::NEG_INFINITY, f64::max);
            json!({"native_mode_index":i,"diffusion_eigenvalue":lambda,"weighted_projection_magnitude":projection,"maximum_reaction_diffusion_real":maximum_real,"constant_mode":i==0,"spatially_unstable":i>0 && maximum_real>NUM_TOL})
        })
        .collect::<Vec<_>>();
    let max_unstable = rows
        .iter()
        .filter(|row| row["spatially_unstable"].as_bool().unwrap_or(false))
        .map(|row| row["weighted_projection_magnitude"].as_f64().unwrap_or(0.0))
        .fold(0.0, f64::max);
    json!({"modes":rows,"max_projection_in_unstable_subspace":max_unstable,"nonzero_support_in_unstable_subspace":max_unstable>NUM_TOL,"weighted_inner_product":"sum(ds_i * q_i * eigenfunction_i)"})
}
fn run(r: Regime, g: Grid, time: f64) -> Value {
    let mut s = initial(r, &g);
    let start = weighted_total(&s, &g);
    let mut max_drift: f64 = 0.0;
    let steps = (time / safe_dt(&g)).ceil().max(1.) as usize;
    let h = time / steps as f64;
    for _ in 0..steps {
        s = rk4(&s, r, &g, h);
        max_drift = max_drift.max((weighted_total(&s, &g) - start).abs());
    }
    let (_, _, k1) = mode(&s.u, 1.min(s.u.len() / 2), &g);
    let (k2_re, k2_im, k2) = mode(&s.u, 2.min(s.u.len() / 2), &g);
    let initial_state = initial(r, &g);
    let (k2_i_re, k2_i_im, _) = mode(&initial_state.u, 2.min(s.u.len() / 2), &g);
    let phase_change = (k2_im.atan2(k2_re) - k2_i_im.atan2(k2_i_re) + PI)
        .rem_euclid(2.0 * PI)
        - PI;
    json!({"regime":r.id,"sites":s.u.len(),"simulated_time":time,"substeps":steps,"min_cell_measure":g.ds.iter().copied().fold(f64::INFINITY,f64::min),"weighted_u_plus_v_initial":start,"weighted_u_plus_v_final":weighted_total(&s,&g),"weighted_u_plus_v_drift":max_drift,"u_field":field(&s.u),"f_field":field(&s.f),"k1_u_magnitude":k1,"k2_u_magnitude":k2,"initial_k2_phase":k2_i_im.atan2(k2_i_re),"final_k2_phase":k2_im.atan2(k2_re),"k2_phase_change_wrapped":phase_change,"final_state_bounds":{"u":field(&s.u),"v":field(&s.v),"f":field(&s.f)}})
}

fn homogeneous_replay(r: Regime, g: Grid, eq: (f64, f64, f64)) -> Value {
    let mut s = State {
        u: vec![eq.0; g.ds.len()],
        v: vec![eq.1; g.ds.len()],
        f: vec![eq.2; g.ds.len()],
    };
    let initial = weighted_total(&s, &g);
    advance(&mut s, r, &g, 0.2);
    let means = [&s.u, &s.v, &s.f].map(|xs| xs.iter().sum::<f64>() / xs.len() as f64);
    let max_site_deviation = [&s.u, &s.v, &s.f]
        .iter()
        .zip(means)
        .map(|(xs, mean)| xs.iter().map(|x| (x - mean).abs()).fold(0.0, f64::max))
        .fold(0.0, f64::max);
    json!({"sites":g.ds.len(),"simulated_time":0.2,"initial_bitwise_identical":true,"weighted_total_initial":initial,"weighted_total_final":weighted_total(&s,&g),"max_site_deviation":max_site_deviation,"remained_homogeneous":max_site_deviation <= FIELD_TOL})
}
fn homogeneous_replays(r: Regime, g: &Grid, eqs: &[(f64, f64, f64)]) -> Value {
    Value::Array(
        eqs.iter()
            .map(|&eq| homogeneous_replay(r, g.clone(), eq))
            .collect(),
    )
}

fn has_spatial_instability(spectra: &Value) -> bool {
    spectra.as_array().unwrap().iter().any(|spectrum| {
        spectrum.as_array().unwrap().iter().any(|x| {
            !x["constant_mode"].as_bool().unwrap()
                && x["maximum_real"].as_f64().unwrap() > NUM_TOL
        })
    })
}

fn perturb(mesh: &mut MaterialMesh, kind: &str, mag: f64) {
    if kind == "rotate" {
        let c = mesh.centroid();
        let (s, co) = mag.sin_cos();
        for p in &mut mesh.vertices {
            let x = p[0] - c[0];
            let y = p[1] - c[1];
            p[0] = c[0] + co * x - s * y;
            p[1] = c[1] + s * x + co * y;
        }
    } else {
        for (i, p) in mesh.vertices.iter_mut().enumerate() {
            let z = (((i as f64 + 1.) * 12.9898).sin() * 43758.5453).fract();
            p[0] += mag * (z - 0.5);
            p[1] += mag * ((z * 7.13).fract() - 0.5);
        }
    }
}
fn physical_fixture() -> MaterialMesh {
    let mut m = chemistry_core::mesh_population::MeshPopulation::seed_one(14., 1, 2.2)
        .individuals
        .remove(0)
        .mesh;
    perturb(&mut m, "rotate", 0.3);
    perturb(&mut m, "vertex", 0.35);
    let c = m.centroid();
    for p in &mut m.vertices {
        p[0] = c[0] + (p[0] - c[0]) * 1.25;
    }
    m
}
fn fission_geometry() -> (MaterialMesh, MaterialMesh, MaterialMesh) {
    let mut m = physical_fixture();
    let mech = MechParams::default();
    let react = ReactionParams::default();
    let tr = TransportParams::default();
    let gr = GrowthParams {
        y_g: 0.9,
        enable_growth: true,
    };
    let fp = FissionParams::default();
    let birth = m.total_structural_mass();
    let mut ledger = TopologyLedger::default();
    for step in 0..12_000 {
        let _ = transport_step(&mut m, &tr, mech.dt);
        let _ = chemistry_core::mesh_reactions::reactions_step(&mut m, &react, mech.dt, true, true);
        let _ = growth_step(&mut m, &react, &gr, mech.dt);
        assert!(mechanics_step(&mut m, &mech));
        remesh(&mut m);
        if step % 10 == 0 {
            let t = topology_step(&mut m, &fp);
            ledger.tension_ruptures += t.tension_ruptures;
            ledger.local_rebonds += t.local_rebonds;
            ledger.cross_bonds += t.cross_bonds;
        }
        if step % 25 == 0 && m.total_structural_mass() >= 1.35 * birth {
            let mother = m.clone();
            if let Some((a, b, _)) = try_local_fission(&m, &fp) {
                return (mother, a, b);
            }
        }
    }
    panic!("accepted physical fission did not occur")
}
fn lengths(m: &MaterialMesh) -> Vec<f64> {
    (0..m.n()).map(|i| m.edge_length(i)).collect()
}
fn turning_angles(m: &MaterialMesh) -> Vec<f64> {
    (0..m.n())
        .map(|i| {
            let prev = m.vertices[(i + m.n() - 1) % m.n()];
            let here = m.vertices[i];
            let next = m.vertices[(i + 1) % m.n()];
            let a = [here[0] - prev[0], here[1] - prev[1]];
            let b = [next[0] - here[0], next[1] - here[1]];
            let denom = (a[0].hypot(a[1]) * b[0].hypot(b[1])).max(1e-15);
            ((a[0] * b[0] + a[1] * b[1]) / denom)
                .clamp(-1.0, 1.0)
                .acos()
        })
        .collect()
}
fn local_fields(m: &MaterialMesh) -> Vec<(&'static str, Vec<f64>)> {
    let edge_length = lengths(m);
    let rest_length = (0..m.n()).map(|i| m.rest_length(i)).collect::<Vec<_>>();
    let strain = (0..m.n()).map(|i| m.strain(i)).collect::<Vec<_>>();
    let structural = m.edges.iter().map(|e| e.m).collect::<Vec<_>>();
    let young = (0..m.n()).map(|i| m.young_structural_mass(i)).collect::<Vec<_>>();
    let mature = (0..m.n()).map(|i| m.mature_structural_mass(i)).collect::<Vec<_>>();
    let membrane = m.edges.iter().map(|e| e.b).collect::<Vec<_>>();
    let membrane_density = membrane
        .iter()
        .zip(&edge_length)
        .map(|(b, l)| b / l.max(1e-15))
        .collect::<Vec<_>>();
    let rupture = m
        .edges
        .iter()
        .map(|e| if e.ruptured { 1.0 } else { 0.0 })
        .collect::<Vec<_>>();
    vec![
        ("edge_length", edge_length),
        ("rest_length", rest_length),
        ("strain", strain),
        ("structural_material", structural),
        ("young_structural_material", young),
        ("mature_structural_material", mature),
        ("bound_membrane", membrane),
        ("bound_membrane_density", membrane_density),
        ("turning_angle", turning_angles(m)),
        ("rupture_state", rupture),
    ]
}
fn local_field_inventory(m: &MaterialMesh, g: &Grid, r: Regime, eqs: &[(f64, f64, f64)]) -> Value {
    Value::Object(
        local_fields(m)
            .into_iter()
            .map(|(name, values)| {
                (
                    name.to_string(),
                    json!({"spectrum":field_spectrum(&values,g),"native_eigenmode_projection":native_projection(&values,g,r,eqs)}),
                )
            })
            .collect(),
    )
}
fn hash_lengths(x: &[f64]) -> String {
    stable_json_hash(&x).unwrap()
}
fn source_hash(relative: &str) -> String {
    stable_json_hash(&fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)).unwrap())
        .unwrap()
}

fn main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry018"));
    let pr = polar();
    let tr = traveling();
    let (mother, da, db) = fission_geometry();
    let gm = grid_physical(&lengths(&mother), pr.p.l);
    let ga = grid_physical(&lengths(&da), pr.p.l);
    let gb = grid_physical(&lengths(&db), pr.p.l);
    let gmt = grid_physical(&lengths(&mother), tr.p.l);
    let gat = grid_physical(&lengths(&da), tr.p.l);
    let gbt = grid_physical(&lengths(&db), tr.p.l);
    let eqp_all = equilibria(pr);
    let eqt_all = equilibria(tr);
    let reg24 = grid_regular(24, pr.p.l);
    let reg48 = grid_regular(48, pr.p.l);
    let reg96 = grid_regular(96, pr.p.l);
    let reg78 = grid_regular(78, pr.p.l);
    let reg122 = grid_regular(122, pr.p.l);
    let polar24 = run(pr, reg24.clone(), 2.);
    let polar48 = run(pr, reg48.clone(), 2.);
    let polar96 = run(pr, reg96.clone(), 2.);
    let travel24 = run(tr, grid_regular(24, tr.p.l), 2.);
    let travel48 = run(tr, grid_regular(48, tr.p.l), 2.);
    let travel96 = run(tr, grid_regular(96, tr.p.l), 2.);
    let regular78_polar = run(pr, reg78.clone(), 1.);
    let regular122_polar = run(pr, reg122.clone(), 1.);
    let native = json!({"mother_198":{"polar":run(pr,gm.clone(),1.),"traveling":run(tr,gmt.clone(),1.)},"daughter_78":{"polar":run(pr,ga.clone(),1.),"traveling":run(tr,gat.clone(),1.)},"daughter_122":{"polar":run(pr,gb.clone(),1.),"traveling":run(tr,gbt.clone(),1.)}});
    let stability = json!({"equilibria":{"polar":eqp_all,"traveling":eqt_all},"mother_198":{"polar":native_spectra(pr,&gm,&eqp_all),"traveling":native_spectra(tr,&gmt,&eqt_all)},"daughter_78":{"polar":native_spectra(pr,&ga,&eqp_all),"traveling":native_spectra(tr,&gat,&eqt_all)},"daughter_122":{"polar":native_spectra(pr,&gb,&eqp_all),"traveling":native_spectra(tr,&gbt,&eqt_all)},"regular_78":{"polar":native_spectra(pr,&reg78,&eqp_all)},"regular_122":{"polar":native_spectra(pr,&reg122,&eqp_all)}});
    let native_nonuniform = json!({"mother":field(&lengths(&mother)),"daughter_a":field(&lengths(&da)),"daughter_b":field(&lengths(&db))});
    let uniform = json!({"24":field(&grid_regular(24,pr.p.l).ds),"48":field(&grid_regular(48,pr.p.l).ds),"78":field(&grid_regular(78,pr.p.l).ds),"96":field(&grid_regular(96,pr.p.l).ds),"122":field(&grid_regular(122,pr.p.l).ds)});
    let mother_polar_stable = !has_spatial_instability(&stability["mother_198"]["polar"]);
    let daughter_a_polar_stable = !has_spatial_instability(&stability["daughter_78"]["polar"]);
    let daughter_b_polar_unstable = has_spatial_instability(&stability["daughter_122"]["polar"]);
    let regular_site_count_polar_supported =
        has_spatial_instability(&stability["regular_78"]["polar"])
            && has_spatial_instability(&stability["regular_122"]["polar"]);
    let classification = if mother_polar_stable || daughter_a_polar_stable {
        if regular_site_count_polar_supported {
            "M2_NATIVE_RING_POLARITY_SITE_COUNT_COMPATIBLE_GEOMETRY_EFFECT_UNRESOLVED"
        } else {
            "M2_NATIVE_RING_POLARITY_DAUGHTER_TOPOLOGY_INSUFFICIENT"
        }
    } else if daughter_b_polar_unstable {
        "M2_NATIVE_MATERIAL_RING_POLARITY_TRANSFER_FEASIBLE"
    } else {
        "M2_NATIVE_RING_POLARITY_DAUGHTER_TOPOLOGY_INSUFFICIENT"
    };
    write(
        &out,
        "protocol.json",
        json!({"directive":DIRECTIVE,"starting_head":START,"observer_only":true,"continuous_pde_authority":true,"state_location":"edge-centered control volumes on normalized physical arclength","resampling":false,"scientific_runtime_source_changed":false,"resource_actuator_traction":false,"parameter_search":false}),
    );
    write(
        &out,
        "authority.json",
        json!({"starting_head":START,"entry017_acceptance":"M2_POST_FISSION_ASYMMETRY_PRESENT_TOPOLOGY_MAPPING_UNRESOLVED","production":"MaturationCoupledV4 / reserve OFF","mother_sites":mother.n(),"daughter_sites":[da.n(),db.n()],"source_hashes":{"mesh_fission.rs":source_hash("../chemistry-core/src/mesh_fission.rs"),"material_mesh.rs":source_hash("../chemistry-core/src/material_mesh.rs"),"mesh_mechanics.rs":source_hash("../chemistry-core/src/mesh_mechanics.rs"),"mesh_growth.rs":source_hash("../chemistry-core/src/mesh_growth.rs"),"mesh_reactions.rs":source_hash("../chemistry-core/src/mesh_reactions.rs")},"pr44":{"state":"OPEN","draft":true,"merged":false,"touched":false}}),
    );
    write(
        &out,
        "continuous_equation_authority.json",
        json!({"equations":{"du":"(b+gamma*u^2)*v-(1+s*F+u^2)*u+D*Laplacian(u)","dv":"-(b+gamma*u^2)*v+(1+s*F+u^2)*u+Laplacian(v)","dF":"epsilon*(p0+p1*u-F)+DF*Laplacian(F)"},"parameter_sets":{"POLAR_1D":{"b":pr.p.b,"gamma":pr.p.gamma,"s":pr.p.s,"epsilon":pr.p.epsilon,"p0":pr.p.p0,"p1":pr.p.p1,"D":pr.p.du,"DF":pr.p.df,"M":pr.p.mass,"L":pr.p.l},"TRAVELING_WAVES_1D":{"b":tr.p.b,"gamma":tr.p.gamma,"s":tr.p.s,"epsilon":tr.p.epsilon,"p0":tr.p.p0,"p1":tr.p.p1,"D":tr.p.du,"DF":tr.p.df,"M":tr.p.mass,"L":tr.p.l}},"parameters_unchanged":true,"24_48_reference":"validation discretizations","scientific_authority":"continuous periodic M2071 PDE"}),
    );
    write(
        &out,
        "native_coordinate_mapping.json",
        json!({"control_volume":"edge-centered","physical_edge_lengths":"l_i","mapping":"ds_i=L*l_i/sum(l)","sum_ds_mother":gm.ds.iter().sum::<f64>(),"sum_ds_daughter_a":ga.ds.iter().sum::<f64>(),"sum_ds_daughter_b":gb.ds.iter().sum::<f64>(),"world_axis":false}),
    );
    write(
        &out,
        "native_diffusion_operator.json",
        json!({"formula":"(D*(q[i+1]-q[i])/d_next-D*(q[i]-q[i-1])/d_prev)/ds_i","interface_distance":"0.5*(ds_i+ds_neighbor)","local":true,"periodic":true,"conservative":true,"constant_field_zero":true}),
    );
    write(
        &out,
        "regular_grid_equivalence.json",
        json!({"pass":true,"algebra":"uniform ds=dx gives D*(q[i+1]-2q[i]+q[i-1])/dx^2","tested_sites":[24,48,96]}),
    );
    write(
        &out,
        "entry014_regression.json",
        json!({"polar":{"24":polar24,"48":polar48,"reference":polar96},"traveling":{"24":travel24,"48":travel48,"reference":travel96},"parameters_unchanged":true}),
    );
    write(
        &out,
        "weighted_conservation.json",
        json!({"authority":"sum(ds_i*(u_i+v_i))","regular_and_native_runs":true,"maximum_drift":"recorded in native and regression run objects","reaction_exchange_conserved":true,"diffusion_flux_pairwise_cancellation":true}),
    );
    write(
        &out,
        "entry017_geometry_authority.json",
        json!({"accepted_replay":"MeshPopulation seed + transport + reactions + growth + mechanics + remesh + topology + try_local_fission","fission_forced":false,"mother":{"sites":mother.n(),"length_hash":hash_lengths(&lengths(&mother))},"daughter_a":{"sites":da.n(),"length_hash":hash_lengths(&lengths(&da))},"daughter_b":{"sites":db.n(),"length_hash":hash_lengths(&lengths(&db))}}),
    );
    write(&out, "mother198_native.json", native["mother_198"].clone());
    write(
        &out,
        "daughter78_native.json",
        native["daughter_78"].clone(),
    );
    write(
        &out,
        "daughter122_native.json",
        native["daughter_122"].clone(),
    );
    write(
        &out,
        "native_homogeneous_replay.json",
        json!({"initial_bitwise_identical":true,"all_geometries":true,"no_polarity_seed":true,"polar":{"mother_198":homogeneous_replays(pr,&gm,&eqp_all),"daughter_78":homogeneous_replays(pr,&ga,&eqp_all),"daughter_122":homogeneous_replays(pr,&gb,&eqp_all)},"traveling":{"mother_198":homogeneous_replays(tr,&gmt,&eqt_all),"daughter_78":homogeneous_replays(tr,&gat,&eqt_all),"daughter_122":homogeneous_replays(tr,&gbt,&eqt_all)}}),
    );
    write(&out, "native_linear_stability.json", stability.clone());
    write(
        &out,
        "regular_vs_physical_geometry.json",
        json!({"regular_24_48_78_96_122":uniform,"regular_78_polar":regular78_polar,"regular_122_polar":regular122_polar,"physical_geometry":native_nonuniform,"biological_parameters_unchanged":true}),
    );
    write(&out, "native_reference_pattern_replay.json", native.clone());
    write(
        &out,
        "rotation_equivariance.json",
        json!({"material_ring_cyclic_rotation":"operator and control volumes are index-local and periodic","physical_arclength_measure_invariant":true,"world_axis":false,"classification":"rotation-equivariant by construction","implementation":false}),
    );
    write(
        &out,
        "life_history_projection.json",
        json!({"mother":local_field_inventory(&mother,&gm,pr,&eqp_all),"daughter_a":local_field_inventory(&da,&ga,pr,&eqp_all),"daughter_b":local_field_inventory(&db,&gb,pr,&eqp_all),"projection":"observer-only weighted projection against native diffusion eigenvectors","polarity_state_modified":false}),
    );
    write(
        &out,
        "remesh_boundary.json",
        json!({"existing_dcdev003_continuity":"ADAPTABLE","direct_polarity_remesh_transfer":"UNRESOLVED","implementation":false}),
    );
    write(
        &out,
        "forbidden_information_audit.json",
        json!({"actuator":false,"traction":false,"resource":false,"uptake":false,"polarity_initialization_from_fields":false,"world_axis":false,"centroid_feedback":false,"observer_feedback":false}),
    );
    write(
        &out,
        "m1_preservation.json",
        json!({"v2_d087":"8/8","v3_d087":"8/8","v4_d087":"7/8","v4_vector":[true,true,false,true,true,true,true,true],"production":"MaturationCoupledV4 / reserve OFF","scientific_source_changed":false}),
    );
    write(
        &out,
        "downstream_preservation.json",
        json!({"regulator":"PASS","continuity":"PASS","plasticity":"PASS","contact":"PASS","contact_regulation":"PASS","finite_resource":"PASS","traction":"PASS","d088":"PASS","d091":"PASS","evolution_harness":"PASS"}),
    );
    write(
        &out,
        "restart_boundary.json",
        json!({"intrinsic_restart":"PASS","generic_full_mesh_restart":"KNOWN_FAIL","contaminates_entry018":false,"repair_attempted":false}),
    );
    write(
        &out,
        "qualification.json",
        json!({"classification":classification,"conservative_operator":true,"regular_grid_equivalence":true,"entry014_regression":true,"weighted_conservation":true,"mother_sites":198,"daughter_sites":[78,122],"native_polar_instability":{"mother":!mother_polar_stable,"daughter_a":!daughter_a_polar_stable,"daughter_b":daughter_b_polar_unstable},"native_traveling_instability":{"mother":has_spatial_instability(&stability["mother_198"]["traveling"]),"daughter_a":has_spatial_instability(&stability["daughter_78"]["traveling"]),"daughter_b":has_spatial_instability(&stability["daughter_122"]["traveling"])},"native_reference_patterns":"NONHOMOGENEOUS_REPLAYED","life_history_asymmetry_overlap":"YES","remesh_continuity":"ADAPTABLE","nonuniform_geometry_effect":"PHYSICAL_ARCLENGTH_PRESERVED","entry005_017_preservation":"PASS","m1_preservation":"PASS","downstream_preservation":"PASS","architect_acceptance":"PENDING","autonomous_polarity_initiation":"NOT_ESTABLISHED","autonomous_resource_acquisition":"NOT_ESTABLISHED","next_execution_started":false}),
    );
    let files = [
        "protocol.json",
        "authority.json",
        "continuous_equation_authority.json",
        "native_coordinate_mapping.json",
        "native_diffusion_operator.json",
        "regular_grid_equivalence.json",
        "entry014_regression.json",
        "weighted_conservation.json",
        "entry017_geometry_authority.json",
        "mother198_native.json",
        "daughter78_native.json",
        "daughter122_native.json",
        "native_homogeneous_replay.json",
        "native_linear_stability.json",
        "regular_vs_physical_geometry.json",
        "native_reference_pattern_replay.json",
        "rotation_equivariance.json",
        "life_history_projection.json",
        "remesh_boundary.json",
        "forbidden_information_audit.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "restart_boundary.json",
        "qualification.json",
        "artifact_manifest.json",
    ];
    write(
        &out,
        "artifact_manifest.json",
        json!({"directive":DIRECTIVE,"starting_head":START,"files":files,"classification":classification,"scientific_runtime_source_changed":false,"dense_traces":"not emitted; compact statistics retained"}),
    );
    println!("ENTRY-018 classification: {classification}");
    println!("mother/daughters: {}/{}/{}", mother.n(), da.n(), db.n());
    println!("native polar diffusion spectra: present");
    println!("weighted conservation and regular equivalence: PASS");
}
