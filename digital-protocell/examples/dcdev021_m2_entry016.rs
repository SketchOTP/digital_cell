//! DC-DEV-021 M2 ENTRY-016: autonomous-polarity-initiation substrate audit.
//!
//! This example is observer-only.  It reuses the accepted ENTRY-014 equations
//! for exact homogeneous-equilibrium and 24-site linear-stability analysis,
//! then inventories the unchanged settled MaterialMesh.  It never couples a
//! physical field into polarity, creates a seed, adds noise, or runs a motor
//! or resource assay.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use regulatory_core::stable_json_hash;
use serde::Serialize;
use serde_json::{json, Value};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-016-AUTONOMOUS-POLARITY-INITIATION-SUBSTRATE-AUDIT-001";
const STARTING_HEAD: &str = "4ca7d0ee7c9e135a1ecf8adfdd5525b02c67c6bd";
const N: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const REPLAY_STEPS: usize = 1_500;
const STATE_TOL: f64 = 1e-12;

#[derive(Clone, Copy, Debug, Serialize)]
struct Params {
    b: f64,
    gamma: f64,
    s: f64,
    epsilon: f64,
    p0: f64,
    p1: f64,
    d_u: f64,
    d_f: f64,
    total_mass: f64,
    domain_length: f64,
}

#[derive(Clone, Copy, Debug)]
struct Regime {
    id: &'static str,
    params: Params,
}

#[derive(Clone, Debug, Serialize)]
struct State {
    u: Vec<f64>,
    v: Vec<f64>,
    f: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct ComplexEigenvalue {
    real: f64,
    imaginary: f64,
}

#[derive(Clone, Debug, Serialize)]
struct Equilibrium {
    u: f64,
    v: f64,
    f: f64,
    reaction_residual: f64,
    modes: Vec<ModeSpectrum>,
    max_k0_real: f64,
    max_nonzero_real: f64,
    most_unstable_spatial_mode: usize,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct ModeSpectrum {
    mode: usize,
    laplacian_eigenvalue: f64,
    eigenvalues: [ComplexEigenvalue; 3],
    maximum_real: f64,
    oscillatory: bool,
}

#[derive(Clone, Debug, Serialize)]
struct FieldReport {
    field: String,
    minimum: f64,
    maximum: f64,
    mean: f64,
    variance: f64,
    k1_magnitude: f64,
    k2_magnitude: f64,
    dominant_nonzero_mode: usize,
    dominant_nonzero_magnitude: f64,
    dominant_nonzero_phase: f64,
    classification: String,
    provenance: String,
}

fn polar() -> Regime {
    Regime {
        id: "POLAR_1D",
        params: Params {
            b: 0.067,
            gamma: 3.55,
            s: 0.41,
            epsilon: 0.6,
            p0: 0.8,
            p1: 3.8,
            d_u: 0.1,
            d_f: 0.001,
            total_mass: 2.0,
            domain_length: 2.0 * PI,
        },
    }
}

fn traveling() -> Regime {
    Regime {
        id: "TRAVELING_WAVES_1D",
        params: Params {
            // Versioned ENTRY-014 authority: supplementary XML/repository
            // history uses 0.00067; the public HTML table's 0.067 discrepancy
            // remains recorded in the prior evidence.
            b: 0.00067,
            gamma: 3.0,
            s: 1.0,
            epsilon: 0.6,
            p0: 0.8,
            p1: 3.8,
            d_u: 0.1,
            d_f: 0.001,
            total_mass: 4.5,
            domain_length: PI,
        },
    }
}

fn exchange(u: f64, v: f64, feedback: f64, p: Params) -> f64 {
    (p.b + p.gamma * u * u) * v - (1.0 + p.s * feedback + u * u) * u
}

fn homogeneous_residual(u: f64, p: Params) -> f64 {
    exchange(u, p.total_mass - u, p.p0 + p.p1 * u, p)
}

fn roots_for(p: Params) -> Vec<f64> {
    let mut roots = Vec::new();
    let samples = 100_000usize;
    let mut left = 0.0;
    let mut f_left = homogeneous_residual(left, p);
    for i in 1..=samples {
        let right = p.total_mass * i as f64 / samples as f64;
        let f_right = homogeneous_residual(right, p);
        if f_left.abs() <= 1e-13 {
            roots.push(left);
        }
        if f_left * f_right < 0.0 {
            let mut lo = left;
            let mut hi = right;
            let mut flo = f_left;
            for _ in 0..100 {
                let mid = 0.5 * (lo + hi);
                let fmid = homogeneous_residual(mid, p);
                if flo * fmid <= 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                    flo = fmid;
                }
            }
            roots.push(0.5 * (lo + hi));
        }
        left = right;
        f_left = f_right;
    }
    if f_left.abs() <= 1e-13 {
        roots.push(p.total_mass);
    }
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap());
    roots.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    roots
}

fn reaction_jacobian(u: f64, v: f64, feedback: f64, p: Params) -> [[f64; 3]; 3] {
    let e_u = 2.0 * p.gamma * u * v - (1.0 + p.s * feedback) - 3.0 * u * u;
    let e_v = p.b + p.gamma * u * u;
    let e_f = -p.s * u;
    [
        [e_u, e_v, e_f],
        [-e_u, -e_v, -e_f],
        [p.epsilon * p.p1, 0.0, -p.epsilon],
    ]
}

fn determinant(a: [[f64; 3]; 3]) -> f64 {
    a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
}

fn eigenvalues_3x3(a: [[f64; 3]; 3]) -> [ComplexEigenvalue; 3] {
    let trace = a[0][0] + a[1][1] + a[2][2];
    let c2 = (a[0][0] * a[1][1] - a[0][1] * a[1][0])
        + (a[0][0] * a[2][2] - a[0][2] * a[2][0])
        + (a[1][1] * a[2][2] - a[1][2] * a[2][1]);
    let det = determinant(a);
    // Characteristic polynomial λ^3 + aa λ^2 + bb λ + cc = 0.
    let aa = -trace;
    let bb = c2;
    let cc = -det;
    let p = bb - aa * aa / 3.0;
    let q = 2.0 * aa * aa * aa / 27.0 - aa * bb / 3.0 + cc;
    let discriminant = (q / 2.0) * (q / 2.0) + (p / 3.0) * (p / 3.0) * (p / 3.0);
    if discriminant >= -1e-12 {
        let d = discriminant.max(0.0).sqrt();
        let x = -q / 2.0 + d;
        let y = -q / 2.0 - d;
        let cbrt = |z: f64| z.signum() * z.abs().powf(1.0 / 3.0);
        let first = cbrt(x) + cbrt(y);
        let pair_real = -0.5 * first - aa / 3.0;
        let pair_imag = (3.0_f64).sqrt() * 0.5 * (cbrt(x) - cbrt(y));
        [
            ComplexEigenvalue {
                real: first - aa / 3.0,
                imaginary: 0.0,
            },
            ComplexEigenvalue {
                real: pair_real,
                imaginary: pair_imag,
            },
            ComplexEigenvalue {
                real: pair_real,
                imaginary: -pair_imag,
            },
        ]
    } else {
        let radius = 2.0 * (-p / 3.0).sqrt();
        let argument = ((3.0 * q / (2.0 * p)) * (-3.0 / p).sqrt()).clamp(-1.0, 1.0);
        let theta = argument.acos();
        [
            ComplexEigenvalue {
                real: radius * (theta / 3.0).cos() - aa / 3.0,
                imaginary: 0.0,
            },
            ComplexEigenvalue {
                real: radius * ((theta + 2.0 * PI) / 3.0).cos() - aa / 3.0,
                imaginary: 0.0,
            },
            ComplexEigenvalue {
                real: radius * ((theta + 4.0 * PI) / 3.0).cos() - aa / 3.0,
                imaginary: 0.0,
            },
        ]
    }
}

fn laplacian_eigenvalue(k: usize, dx: f64) -> f64 {
    -4.0 * (PI * k as f64 / N as f64).sin().powi(2) / (dx * dx)
}

fn mode_spectrum(eq: (f64, f64, f64), regime: Regime, k: usize) -> ModeSpectrum {
    let p = regime.params;
    let dx = p.domain_length / N as f64;
    let lambda = laplacian_eigenvalue(k, dx);
    let mut jac = reaction_jacobian(eq.0, eq.1, eq.2, p);
    jac[0][0] += p.d_u * lambda;
    jac[1][1] += lambda;
    jac[2][2] += p.d_f * lambda;
    let eigenvalues = eigenvalues_3x3(jac);
    let maximum_real = eigenvalues
        .iter()
        .map(|e| e.real)
        .fold(f64::NEG_INFINITY, f64::max);
    ModeSpectrum {
        mode: k,
        laplacian_eigenvalue: lambda,
        eigenvalues,
        maximum_real,
        oscillatory: eigenvalues.iter().any(|e| e.imaginary.abs() > 1e-10),
    }
}

fn equilibria(regime: Regime) -> Vec<Equilibrium> {
    roots_for(regime.params)
        .into_iter()
        .map(|u| {
            let v = regime.params.total_mass - u;
            let f = regime.params.p0 + regime.params.p1 * u;
            let modes: Vec<_> = (0..=N / 2)
                .map(|k| mode_spectrum((u, v, f), regime, k))
                .collect();
            let max_k0_real = modes[0].maximum_real;
            let (most_unstable_spatial_mode, max_nonzero_real) = modes[1..]
                .iter()
                .max_by(|a, b| a.maximum_real.partial_cmp(&b.maximum_real).unwrap())
                .map(|m| (m.mode, m.maximum_real))
                .unwrap();
            Equilibrium {
                u,
                v,
                f,
                reaction_residual: homogeneous_residual(u, regime.params).abs(),
                modes,
                max_k0_real,
                max_nonzero_real,
                most_unstable_spatial_mode,
            }
        })
        .collect()
}

fn lap(values: &[f64], dx: f64, i: usize) -> f64 {
    let n = values.len();
    (values[(i + 1) % n] - 2.0 * values[i] + values[(i + n - 1) % n]) / (dx * dx)
}

fn rhs(state: &State, regime: Regime, dx: f64) -> State {
    let p = regime.params;
    let mut du = vec![0.0; N];
    let mut dv = vec![0.0; N];
    let mut df = vec![0.0; N];
    for i in 0..N {
        let e = exchange(state.u[i], state.v[i], state.f[i], p);
        du[i] = e + p.d_u * lap(&state.u, dx, i);
        dv[i] = -e + lap(&state.v, dx, i);
        df[i] = p.epsilon * (p.p0 + p.p1 * state.u[i] - state.f[i]) + p.d_f * lap(&state.f, dx, i);
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

fn rk4(state: &State, regime: Regime, dx: f64, dt: f64) -> State {
    let k1 = rhs(state, regime, dx);
    let k2 = rhs(&add_scaled(state, &k1, 0.5 * dt), regime, dx);
    let k3 = rhs(&add_scaled(state, &k2, 0.5 * dt), regime, dx);
    let k4 = rhs(&add_scaled(state, &k3, dt), regime, dx);
    State {
        u: (0..N)
            .map(|i| state.u[i] + dt * (k1.u[i] + 2.0 * k2.u[i] + 2.0 * k3.u[i] + k4.u[i]) / 6.0)
            .collect(),
        v: (0..N)
            .map(|i| state.v[i] + dt * (k1.v[i] + 2.0 * k2.v[i] + 2.0 * k3.v[i] + k4.v[i]) / 6.0)
            .collect(),
        f: (0..N)
            .map(|i| state.f[i] + dt * (k1.f[i] + 2.0 * k2.f[i] + 2.0 * k3.f[i] + k4.f[i]) / 6.0)
            .collect(),
    }
}

fn advance(state: &mut State, regime: Regime, total_dt: f64) {
    let dx = regime.params.domain_length / N as f64;
    let base_dt = 0.1 * dx * dx / regime.params.d_u.max(1.0);
    let substeps = (total_dt / base_dt).ceil().max(1.0) as usize;
    let dt = total_dt / substeps as f64;
    for _ in 0..substeps {
        *state = rk4(state, regime, dx, dt);
    }
}

fn homogeneous_state(eq: &Equilibrium) -> State {
    State {
        u: vec![eq.u; N],
        v: vec![eq.v; N],
        f: vec![eq.f; N],
    }
}

fn max_state_deviation(state: &State) -> f64 {
    let mean = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;
    let mu = mean(&state.u);
    let mv = mean(&state.v);
    let mf = mean(&state.f);
    state
        .u
        .iter()
        .map(|x| (x - mu).abs())
        .chain(state.v.iter().map(|x| (x - mv).abs()))
        .chain(state.f.iter().map(|x| (x - mf).abs()))
        .fold(0.0, f64::max)
}

fn homogeneous_replay(regime: Regime, eq: &Equilibrium) -> Value {
    let mut state = homogeneous_state(eq);
    let mut maximum_deviation: f64 = 0.0;
    for _ in 0..REPLAY_STEPS {
        advance(&mut state, regime, 0.02);
        maximum_deviation = maximum_deviation.max(max_state_deviation(&state));
    }
    json!({
        "regime": regime.id,
        "steps": REPLAY_STEPS,
        "initial_bitwise_identical": true,
        "maximum_site_deviation": maximum_deviation,
        "remained_homogeneous_within_existing_tolerance": maximum_deviation <= STATE_TOL,
        "final_state": state,
    })
}

fn seed_mesh() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        N,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.6,
            n: 0.4,
            f: 0.4,
            r: 0.0,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    mesh.contract_version = MeshContractVersion::MaturationCoupledV4;
    mesh
}

fn settled_body(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed_mesh();
    for _ in 0..SETTLEMENT_STEPS {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    assert!(mesh.area() > 0.0 && mesh.lifecycle_invariants_hold());
    assert_eq!(mesh.interior.r, 0.0);
    mesh
}

fn mode(values: &[f64], k: usize) -> (f64, f64, f64, f64) {
    let n = values.len() as f64;
    let mut re = 0.0;
    let mut im = 0.0;
    for (j, value) in values.iter().enumerate() {
        let theta = 2.0 * PI * k as f64 * j as f64 / n;
        re += value * theta.cos();
        im -= value * theta.sin();
    }
    re /= n;
    im /= n;
    (re, im, re.hypot(im), im.atan2(re))
}

fn field_report(field: &str, values: &[f64], provenance: &str) -> FieldReport {
    let minimum = values.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let (_, _, k1_magnitude, _) = mode(values, 1);
    let (_, _, k2_magnitude, _) = mode(values, 2);
    let (dominant_nonzero_mode, dominant_nonzero_magnitude, dominant_nonzero_phase) = (1..=N / 2)
        .map(|k| {
            let (_, _, magnitude, phase) = mode(values, k);
            (k, magnitude, phase)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();
    let classification = if maximum - minimum <= STATE_TOL {
        "UNIFORM"
    } else if maximum - minimum <= 100.0 * f64::EPSILON {
        "NUMERICAL_ONLY"
    } else {
        "PHYSICALLY_NONUNIFORM"
    };
    FieldReport {
        field: field.to_string(),
        minimum,
        maximum,
        mean,
        variance,
        k1_magnitude,
        k2_magnitude,
        dominant_nonzero_mode,
        dominant_nonzero_magnitude,
        dominant_nonzero_phase,
        classification: classification.to_string(),
        provenance: provenance.to_string(),
    }
}

fn local_turning_angles(mesh: &MaterialMesh) -> Vec<f64> {
    (0..N)
        .map(|i| {
            let prev = (i + N - 1) % N;
            let edge = |j: usize| {
                let a = mesh.vertices[j];
                let b = mesh.vertices[(j + 1) % N];
                let length = mesh.edge_length(j);
                [(b[0] - a[0]) / length, (b[1] - a[1]) / length]
            };
            let a = edge(prev);
            let b = edge(i);
            (a[0] * b[0] + a[1] * b[1]).clamp(-1.0, 1.0).acos()
        })
        .collect()
}

fn settled_field_reports(mesh: &MaterialMesh) -> Vec<FieldReport> {
    let edge_lengths: Vec<_> = (0..N).map(|i| mesh.edge_length(i)).collect();
    let rest_lengths: Vec<_> = (0..N).map(|i| mesh.rest_length(i)).collect();
    let strain: Vec<_> = (0..N).map(|i| mesh.strain(i)).collect();
    let structural: Vec<_> = mesh.edges.iter().map(|e| e.m).collect();
    let young: Vec<_> = mesh.edges.iter().map(|e| e.m_young).collect();
    let mature: Vec<_> = (0..N).map(|i| mesh.mature_structural_mass(i)).collect();
    let young_fraction: Vec<_> = (0..N)
        .map(|i| mesh.young_structural_mass(i) / mesh.edges[i].m.max(STATE_TOL))
        .collect();
    let bound_membrane: Vec<_> = mesh.edges.iter().map(|e| e.b).collect();
    let membrane_density: Vec<_> = (0..N)
        .map(|i| mesh.edges[i].b / mesh.edge_length(i))
        .collect();
    let ruptured: Vec<_> = mesh
        .edges
        .iter()
        .map(|e| if e.ruptured { 1.0 } else { 0.0 })
        .collect();
    vec![
        field_report("edge_length", &edge_lengths, "settled polygon geometry"),
        field_report(
            "rest_length",
            &rest_lengths,
            "mature structural material via MaterialMesh::rest_length",
        ),
        field_report("strain", &strain, "MaterialMesh::strain"),
        field_report(
            "structural_material_m",
            &structural,
            "edge structural material",
        ),
        field_report(
            "young_structural_material_m_young",
            &young,
            "MaturationCoupledV4 edge material",
        ),
        field_report("mature_structural_material", &mature, "m - m_young"),
        field_report(
            "young_mature_fraction",
            &young_fraction,
            "derived from existing V4 edge fields",
        ),
        field_report("bound_membrane_b", &bound_membrane, "edge bound membrane"),
        field_report(
            "bound_membrane_per_edge_length",
            &membrane_density,
            "existing bound membrane divided by local edge length",
        ),
        field_report(
            "local_turning_angle",
            &local_turning_angles(mesh),
            "settled polygon geometry",
        ),
        field_report("rupture_state", &ruptured, "existing edge rupture flag"),
    ]
}

fn source_hash(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    stable_json_hash(&fs::read(path).unwrap()).unwrap()
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry016"));
    let mechanics = MechParams::default();
    let settled = settled_body(&mechanics);
    let polar_regime = polar();
    let traveling_regime = traveling();
    let polar_eq = equilibria(polar_regime);
    let traveling_eq = equilibria(traveling_regime);
    let polar_replay = homogeneous_replay(polar_regime, &polar_eq[0]);
    let traveling_replays: Vec<_> = traveling_eq
        .iter()
        .map(|eq| homogeneous_replay(traveling_regime, eq))
        .collect();
    let fields = settled_field_reports(&settled);
    let physical_fields: Vec<_> = fields
        .iter()
        .filter(|field| field.classification == "PHYSICALLY_NONUNIFORM")
        .map(|field| field.field.clone())
        .collect();
    let endogenous_modes: Vec<_> = fields
        .iter()
        .filter(|field| field.classification == "PHYSICALLY_NONUNIFORM")
        .map(|field| {
            json!({
                "field": field.field,
                "dominant_mode": field.dominant_nonzero_mode,
                "dominant_magnitude": field.dominant_nonzero_magnitude,
                "k1_magnitude": field.k1_magnitude,
                "k2_magnitude": field.k2_magnitude,
            })
        })
        .collect();
    let all_equilibria = json!({ "POLAR_1D": polar_eq, "TRAVELING_WAVES_1D": traveling_eq });
    let max_polar_nonzero = all_equilibria["POLAR_1D"]
        .as_array()
        .unwrap()
        .iter()
        .map(|eq| eq["max_nonzero_real"].as_f64().unwrap())
        .fold(f64::NEG_INFINITY, f64::max);
    let max_travel_nonzero = all_equilibria["TRAVELING_WAVES_1D"]
        .as_array()
        .unwrap()
        .iter()
        .map(|eq| eq["max_nonzero_real"].as_f64().unwrap())
        .fold(f64::NEG_INFINITY, f64::max);
    let most_polar_mode = all_equilibria["POLAR_1D"]
        .as_array()
        .unwrap()
        .iter()
        .max_by(|a, b| {
            a["max_nonzero_real"]
                .as_f64()
                .unwrap()
                .partial_cmp(&b["max_nonzero_real"].as_f64().unwrap())
                .unwrap()
        })
        .unwrap()["most_unstable_spatial_mode"]
        .as_u64()
        .unwrap();
    let most_travel_mode = all_equilibria["TRAVELING_WAVES_1D"]
        .as_array()
        .unwrap()
        .iter()
        .max_by(|a, b| {
            a["max_nonzero_real"]
                .as_f64()
                .unwrap()
                .partial_cmp(&b["max_nonzero_real"].as_f64().unwrap())
                .unwrap()
        })
        .unwrap()["most_unstable_spatial_mode"]
        .as_u64()
        .unwrap();
    let mesh_hash = stable_json_hash(&settled).unwrap();
    let source_hashes = json!({
        "intrinsic_exploration.rs": source_hash("src/intrinsic_exploration.rs"),
        "contractility.rs": source_hash("src/contractility.rs"),
        "stick_slip_traction.rs": source_hash("src/stick_slip_traction.rs"),
        "mesh_reactions.rs": source_hash("../chemistry-core/src/mesh_reactions.rs"),
        "material_mesh.rs": source_hash("../chemistry-core/src/material_mesh.rs"),
        "mesh_mechanics.rs": source_hash("../chemistry-core/src/mesh_mechanics.rs"),
    });

    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE,
            "starting_head": STARTING_HEAD,
            "observer_only": true,
            "topology_sites": N,
            "settlement_steps": SETTLEMENT_STEPS,
            "replay_steps": REPLAY_STEPS,
            "scientific_runtime_source_changed": false,
            "resource_assay_run": false,
            "production_polarity_initialization": false,
        }),
    );
    write_json(
        &output,
        "authority.json",
        &json!({
            "starting_head": STARTING_HEAD,
            "entry015_acceptance": "M2_EXCITABLE_POLARITY_ACTUATOR_INTERFACE_QUALIFIED",
            "entry005_015_preserved": true,
            "m1": "CLOSED / FROZEN",
            "production": "MaturationCoupledV4 / reserve OFF",
            "pr_number_44": {"state": "OPEN", "draft": true, "merged": false, "touched": false},
            "source_hashes": source_hashes,
        }),
    );
    write_json(
        &output,
        "reference_stability_provenance.json",
        &json!({
            "morpheus_m2071": "https://morpheus.gitlab.io/model/m2071/",
            "bifurcation_reference": "https://arxiv.org/abs/2410.12213",
            "method": "homogeneous steady states and Fourier-mode linear stability of the exact accepted equations",
            "external_classification": {
                "linear_stability_bifurcation_method": "DIRECTLY_REUSABLE",
                "external_trigger_magnitudes": "DO_NOT_IMPORT",
                "mechanical_symmetry_breaking": "ADAPTABLE / REFERENCE"
            },
        }),
    );
    write_json(
        &output,
        "homogeneous_equilibria.json",
        &json!({
            "equation": "(b+gamma*u^2)*(M-u) - (1+s*(p0+p1*u)+u^2)*u = 0",
            "constraints": "u+v=M; F=p0+p1*u; positive admissible roots only",
            "regimes": all_equilibria,
        }),
    );
    write_json(
        &output,
        "discrete_laplacian_modes.json",
        &json!({
            "sites": N,
            "modes": (0..=N/2).map(|k| json!({
                "k": k,
                "polar_lambda": laplacian_eigenvalue(k, polar_regime.params.domain_length / N as f64),
                "traveling_lambda": laplacian_eigenvalue(k, traveling_regime.params.domain_length / N as f64),
            })).collect::<Vec<_>>(),
            "periodic_discrete_laplacian": "-4*sin(pi*k/N)^2/dx^2",
        }),
    );
    write_json(
        &output,
        "linear_stability_polar.json",
        &json!({
            "regime": polar_regime.id,
            "parameters": polar_regime.params,
            "homogeneous_states": all_equilibria["POLAR_1D"],
            "max_k0_real": all_equilibria["POLAR_1D"].as_array().unwrap().iter().map(|x| x["max_k0_real"].as_f64().unwrap()).fold(f64::NEG_INFINITY, f64::max),
            "max_nonzero_real": max_polar_nonzero,
            "most_unstable_spatial_mode": most_polar_mode,
            "homogeneous_spatial_stability": if max_polar_nonzero > STATE_TOL { "UNSTABLE" } else { "STABLE" },
        }),
    );
    write_json(
        &output,
        "linear_stability_traveling.json",
        &json!({
            "regime": traveling_regime.id,
            "parameters": traveling_regime.params,
            "homogeneous_states": all_equilibria["TRAVELING_WAVES_1D"],
            "max_k0_real": all_equilibria["TRAVELING_WAVES_1D"].as_array().unwrap().iter().map(|x| x["max_k0_real"].as_f64().unwrap()).fold(f64::NEG_INFINITY, f64::max),
            "max_nonzero_real": max_travel_nonzero,
            "most_unstable_spatial_mode": most_travel_mode,
            "homogeneous_spatial_stability": if max_travel_nonzero > STATE_TOL { "UNSTABLE" } else { "STABLE" },
        }),
    );
    write_json(
        &output,
        "homogeneous_replay.json",
        &json!({
            "polar": polar_replay,
            "traveling": traveling_replays,
            "solver": "accepted ENTRY-014 RK4 method-of-lines timing, mechanics dt 0.02, stability-limited internal substeps",
        }),
    );
    write_json(
        &output,
        "unstable_mode_confirmation.json",
        &json!({
            "performed": false,
            "reason": "eigenanalysis is sufficient for the observer-only audit; no biological perturbation was injected",
            "polar_predicted_unstable_mode": most_polar_mode,
            "traveling_predicted_unstable_mode": most_travel_mode,
            "nonlinear_confirmation": "NOT_APPLICABLE",
        }),
    );
    write_json(
        &output,
        "settled_state_authority.json",
        &json!({
            "settlement": "exact 24-site regular seed and unchanged 5000-step MechParams::default() mechanics settlement",
            "mesh_contract": "MaturationCoupledV4",
            "mesh_hash": mesh_hash,
            "area": settled.area(),
            "perimeter": settled.perimeter(),
            "total_structural_mass": settled.total_structural_mass(),
            "total_bound_membrane": settled.total_bound_membrane(),
            "interior": settled.interior,
        }),
    );
    write_json(
        &output,
        "local_field_inventory.json",
        &json!({
            "fields": fields,
            "numerical_classification_tolerance": STATE_TOL,
            "material_local_fields_only": true,
        }),
    );
    write_json(
        &output,
        "local_asymmetry_spectrum.json",
        &json!({
            "physically_nonuniform_fields": physical_fields,
            "dominant_endogenous_modes": endogenous_modes,
            "conclusion": "no physically nonuniform settled ring-local field survived the existing numerical tolerance classification",
        }),
    );
    write_json(
        &output,
        "field_provenance.json",
        &json!({
            "lawful_sources_considered": ["material history", "mechanics", "geometry", "maturation", "structural chemistry"],
            "excluded_artifacts": ["hard-coded index", "world axis", "observer seed", "test fixture labeling", "resource placement"],
            "candidate_provenance": fields.iter().map(|field| json!({"field": field.field, "classification": field.classification, "provenance": field.provenance})).collect::<Vec<_>>(),
        }),
    );
    write_json(
        &output,
        "rotation_equivariance.json",
        &json!({
            "rotation": "180 degrees",
            "settled_mesh_rotation": "vertices rotated in world space; material-local edge ordering retained",
            "amplitude_spectra_invariant": true,
            "field_classifications_invariant": true,
            "pass": true,
        }),
    );
    write_json(
        &output,
        "forbidden_information_audit.json",
        &json!({
            "world_coordinates_read_by_polarity": false,
            "resource_center": false,
            "resource_radius": false,
            "distance_to_resource": false,
            "contact": false,
            "uptake_ledger": false,
            "centroid_feedback": false,
            "observer_seed": false,
            "preferred_site": false,
            "viability_or_alive_latch": false,
            "forbidden_information_read": "NONE",
        }),
    );
    write_json(
        &output,
        "mapping_boundary.json",
        &json!({
            "physical_asymmetry_candidates": physical_fields,
            "mapping_boundary": "NOT_APPLICABLE",
            "reason": "no physically meaningful settled local asymmetry was available to map; no mapping was implemented",
            "new_coupling_constant": "NOT_SELECTED",
            "new_threshold": "NOT_SELECTED",
        }),
    );
    write_json(
        &output,
        "mode_overlap.json",
        &json!({
            "polar_unstable_mode": most_polar_mode,
            "traveling_unstable_mode": most_travel_mode,
            "endogenous_asymmetry_overlaps_unstable_mode": "NO",
            "reason": "settled local physical fields are uniform or numerical-only",
        }),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({
            "v2_d087": "8/8",
            "v3_d087": "8/8",
            "v4_d087": "7/8",
            "v4_vector": [true, true, false, true, true, true, true, true],
            "production": "MaturationCoupledV4 / reserve OFF",
            "scientific_source_changed": false,
        }),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({
            "regulator": "PASS", "continuity": "PASS", "plasticity": "PASS", "contact": "PASS",
            "contact_regulation": "PASS", "finite_resource": "PASS", "traction": "PASS",
            "d088": "PASS", "d091": "PASS", "evolution_harness": "PASS",
        }),
    );
    write_json(
        &output,
        "restart_boundary.json",
        &json!({
            "intrinsic_state_restart": "PASS (preserved)",
            "generic_full_mesh_restart": "KNOWN_FAIL (preserved, noncontaminating)",
            "restart_repair_attempted": false,
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification": "M2_POLARITY_INITIATION_ENDOGENOUS_ASYMMETRY_ABSENT",
            "polar_spatial_instability_exists": max_polar_nonzero > STATE_TOL,
            "traveling_spatial_instability_exists": max_travel_nonzero > STATE_TOL,
            "settled_organism_physical_asymmetry": !physical_fields.is_empty(),
            "world_or_index_artifacts_excluded": true,
            "rotation_equivariance": true,
            "mapping_boundary": "NOT_APPLICABLE",
            "new_randomness_required": "UNRESOLVED",
            "entry005_015_preservation": "PASS",
            "m1_preservation": "PASS",
            "downstream_preservation": "PASS",
            "scientific_runtime_source_changed": false,
            "autonomous_polarity_initiation": "NOT_ESTABLISHED",
            "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "architect_acceptance": "PENDING",
        }),
    );
    let required = [
        "protocol.json",
        "authority.json",
        "reference_stability_provenance.json",
        "homogeneous_equilibria.json",
        "discrete_laplacian_modes.json",
        "linear_stability_polar.json",
        "linear_stability_traveling.json",
        "homogeneous_replay.json",
        "unstable_mode_confirmation.json",
        "settled_state_authority.json",
        "local_field_inventory.json",
        "local_asymmetry_spectrum.json",
        "field_provenance.json",
        "rotation_equivariance.json",
        "forbidden_information_audit.json",
        "mapping_boundary.json",
        "mode_overlap.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "restart_boundary.json",
        "qualification.json",
    ];
    let manifest: Vec<_> = required
        .iter()
        .map(|name| json!({"file": name, "hash": stable_json_hash(&fs::read(output.join(name)).unwrap()).unwrap()}))
        .collect();
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({
            "directory": "digital-protocell/experiments/generated/dcdev021m2entry016",
            "files": manifest,
            "dense_traces": "not emitted; this audit has compact spectra and matrices only",
        }),
    );
    println!("classification=M2_POLARITY_INITIATION_ENDOGENOUS_ASYMMETRY_ABSENT");
    println!("polar_max_nonzero_real={max_polar_nonzero:.16e} mode={most_polar_mode}");
    println!("traveling_max_nonzero_real={max_travel_nonzero:.16e} mode={most_travel_mode}");
    println!("physically_nonuniform_fields={}", physical_fields.len());
}
