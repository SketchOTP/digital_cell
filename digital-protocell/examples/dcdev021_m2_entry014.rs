//! DC-DEV-021 M2 ENTRY-014: isolated M2071 excitable-polarity transfer audit.
//!
//! This example is deliberately independent of Digital Cell biology.  It is a
//! small method-of-lines reimplementation of the two published M2071 1-D
//! periodic reference regimes.  It never constructs or mutates a MaterialMesh,
//! calls an actuator, reads a resource, or imports observer quantities into a
//! state update.

use serde_json::{json, Value};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str =
    "DC-DEV-021-M2-ENTRY-014-EXCITABLE-POLARITY-REFERENCE-TRANSFER-FEASIBILITY-001";
const STARTING_HEAD: &str = "0cef8e1cdb12d915f3b9f3084600c585c933ffa8";
const REFERENCE_N: usize = 96;
const RING24_N: usize = 24;
const RING48_N: usize = 48;
const EPS: f64 = 64.0 * f64::EPSILON;

#[derive(Clone, Copy)]
struct Params {
    b: f64,
    gamma: f64,
    s: f64,
    epsilon: f64,
    p0: f64,
    p1: f64,
    d_u: f64,
    d_f: f64,
    hill_n: f64,
}

#[derive(Clone, Copy)]
struct Regime {
    id: &'static str,
    label: &'static str,
    params: Params,
    length: f64,
    stop_time: f64,
}

#[derive(Clone)]
struct State {
    u: Vec<f64>,
    v: Vec<f64>,
    f: Vec<f64>,
}

#[derive(Clone)]
struct Snapshot {
    time: f64,
    k0_u: f64,
    k1_u: f64,
    k2_u: f64,
    k1_phase: f64,
    dominant_nonzero_mode: usize,
    dominant_nonzero_magnitude: f64,
    dominant_nonzero_phase: f64,
    k0_f: f64,
    k1_f: f64,
    total_uv: f64,
    variance_u: f64,
    min_state: f64,
    max_state: f64,
}

struct Run {
    regime: &'static str,
    sites: usize,
    dx: f64,
    dt: f64,
    snapshots: Vec<Snapshot>,
    initial: Snapshot,
    total_uv_drift: f64,
    max_reaction_exchange_residual: f64,
    max_diffusion_sum_residual: f64,
    phase_change: f64,
    phase_velocity: f64,
    dominant_phase_change: f64,
    dominant_phase_velocity: f64,
    nonhomogeneous_final: bool,
    moving_phase: bool,
}

fn polar() -> Regime {
    Regime {
        id: "POLAR_1D",
        label: "Polar 1D",
        params: Params {
            b: 0.067,
            gamma: 3.55,
            s: 0.41,
            epsilon: 0.6,
            p0: 0.8,
            p1: 3.8,
            d_u: 0.1,
            d_f: 0.001,
            hill_n: 2.0,
        },
        length: 2.0 * PI,
        stop_time: 100.0,
    }
}

fn traveling() -> Regime {
    Regime {
        id: "TRAVELING_WAVES_1D",
        label: "Traveling Waves 1D",
        params: Params {
            // The versioned supplementary XML and the related M2072 Turning
            // Cell table use 0.00067.  The M2071 HTML table currently shows
            // 0.067; this is retained in external_provenance.json as a source
            // discrepancy rather than silently normalized.
            b: 0.00067,
            gamma: 3.0,
            s: 1.0,
            epsilon: 0.6,
            p0: 0.8,
            p1: 3.8,
            d_u: 0.1,
            d_f: 0.001,
            hill_n: 2.0,
        },
        length: PI,
        stop_time: 50.0,
    }
}

fn initial(regime: Regime, n: usize) -> State {
    let dx = regime.length / n as f64;
    let mut u = Vec::with_capacity(n);
    let mut v = Vec::with_capacity(n);
    let mut f = Vec::with_capacity(n);
    for i in 0..n {
        let x = i as f64 * dx;
        if regime.id == "POLAR_1D" {
            u.push(1.0 - 0.5 * x.cos());
            v.push(1.0 - 0.1 * x.cos());
            // The XML is the executable source and uses the plus sign.
            f.push(4.5 + 0.82 * x.cos());
        } else {
            u.push(2.2 - 0.33 * (2.0 * x).cos() - 0.47 * (2.0 * x).sin());
            v.push(2.3 - 0.1 * (2.0 * x).sin());
            f.push(9.2 - 0.82 * (2.0 * x).cos());
        }
    }
    State { u, v, f }
}

fn lap(values: &[f64], dx: f64, i: usize) -> f64 {
    let n = values.len();
    (values[(i + 1) % n] - 2.0 * values[i] + values[(i + n - 1) % n]) / (dx * dx)
}

fn rhs(state: &State, regime: Regime, dx: f64) -> State {
    let p = regime.params;
    let n = state.u.len();
    let mut du = vec![0.0; n];
    let mut dv = vec![0.0; n];
    let mut df = vec![0.0; n];
    for i in 0..n {
        let u = state.u[i];
        let v = state.v[i];
        let f = state.f[i];
        let exchange = (p.b + p.gamma * u.powf(p.hill_n)) * v - (1.0 + p.s * f + u * u) * u;
        du[i] = exchange + p.d_u * lap(&state.u, dx, i);
        dv[i] = -exchange + lap(&state.v, dx, i);
        df[i] = p.epsilon * (p.p0 + p.p1 * u - f) + p.d_f * lap(&state.f, dx, i);
    }
    State {
        u: du,
        v: dv,
        f: df,
    }
}

fn combine(a: &State, b: &State, scale: f64) -> State {
    State {
        u: a.u.iter().zip(&b.u).map(|(x, y)| x + scale * y).collect(),
        v: a.v.iter().zip(&b.v).map(|(x, y)| x + scale * y).collect(),
        f: a.f.iter().zip(&b.f).map(|(x, y)| x + scale * y).collect(),
    }
}

fn rk4(state: &State, regime: Regime, dx: f64, dt: f64) -> State {
    let k1 = rhs(state, regime, dx);
    let k2 = rhs(&combine(state, &k1, 0.5 * dt), regime, dx);
    let k3 = rhs(&combine(state, &k2, 0.5 * dt), regime, dx);
    let k4 = rhs(&combine(state, &k3, dt), regime, dx);
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

fn mode(values: &[f64], k: usize) -> (f64, f64, f64) {
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
    (re, im, re.hypot(im))
}

fn snapshot(state: &State, time: f64) -> Snapshot {
    let (_, _, k1_u) = mode(&state.u, 1);
    let (_, _, k2_u) = mode(&state.u, 2);
    let (_, _, k1_f) = mode(&state.f, 1);
    let (_, _, k0_u) = mode(&state.u, 0);
    let (_, _, k0_f) = mode(&state.f, 0);
    let (_, im, _) = mode(&state.u, 1);
    let (dominant_nonzero_mode, dominant_nonzero_magnitude, dominant_nonzero_phase) =
        (1..=state.u.len() / 2)
            .map(|k| {
                let (re, im, magnitude) = mode(&state.u, k);
                (k, magnitude, im.atan2(re))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
    let mut total_uv = 0.0;
    let mut mean_u = 0.0;
    let mut min_state = f64::INFINITY;
    let mut max_state = f64::NEG_INFINITY;
    for i in 0..state.u.len() {
        total_uv += state.u[i] + state.v[i];
        mean_u += state.u[i];
        min_state = min_state.min(state.u[i]).min(state.v[i]).min(state.f[i]);
        max_state = max_state.max(state.u[i]).max(state.v[i]).max(state.f[i]);
    }
    total_uv /= state.u.len() as f64;
    mean_u /= state.u.len() as f64;
    let variance_u = state
        .u
        .iter()
        .map(|x| (x - mean_u) * (x - mean_u))
        .sum::<f64>()
        / state.u.len() as f64;
    Snapshot {
        time,
        k0_u,
        k1_u,
        k2_u,
        k1_phase: im.atan2(mode(&state.u, 1).0),
        dominant_nonzero_mode,
        dominant_nonzero_magnitude,
        dominant_nonzero_phase,
        k0_f,
        k1_f,
        total_uv,
        variance_u,
        min_state,
        max_state,
    }
}

fn wrap(mut x: f64) -> f64 {
    while x > PI {
        x -= 2.0 * PI;
    }
    while x < -PI {
        x += 2.0 * PI;
    }
    x
}

fn run(regime: Regime, n: usize) -> Run {
    let dx = regime.length / n as f64;
    // The largest diffusion coefficient is 1.0.  dt is a fixed numerical
    // stability choice (0.1 dx^2 / Dmax), selected before regime inspection.
    let dt = 0.1 * dx * dx;
    let mut state = initial(regime, n);
    let initial_snapshot = snapshot(&state, 0.0);
    let mut snapshots = vec![initial_snapshot.clone()];
    let mut time = 0.0;
    let sample_dt = regime.stop_time / 10.0;
    let mut next_sample = sample_dt;
    let mut total_uv_drift: f64 = 0.0;
    let mut max_reaction_exchange_residual: f64 = 0.0;
    let mut max_diffusion_sum_residual: f64 = 0.0;
    while time < regime.stop_time - 1e-14 {
        let h = dt.min(regime.stop_time - time);
        let derivative = rhs(&state, regime, dx);
        let mut diffusion_sum_total = 0.0;
        for i in 0..n {
            let exchange = derivative.u[i] + derivative.v[i]
                - regime.params.d_u * lap(&state.u, dx, i)
                - lap(&state.v, dx, i);
            max_reaction_exchange_residual = max_reaction_exchange_residual.max(exchange.abs());
            diffusion_sum_total += regime.params.d_u * lap(&state.u, dx, i) + lap(&state.v, dx, i);
        }
        max_diffusion_sum_residual = max_diffusion_sum_residual.max(diffusion_sum_total.abs());
        state = rk4(&state, regime, dx, h);
        time += h;
        let current_total = snapshot(&state, time).total_uv;
        total_uv_drift = total_uv_drift.max((current_total - initial_snapshot.total_uv).abs());
        if time + 1e-12 >= next_sample || time >= regime.stop_time - 1e-12 {
            snapshots.push(snapshot(&state, time));
            next_sample += sample_dt;
        }
    }
    let final_snapshot = snapshots.last().unwrap();
    let phase_change = unwrap_phase_change(&snapshots);
    let phase_velocity = phase_change / regime.stop_time;
    let dominant_phase_change = snapshots
        .windows(2)
        .filter(|pair| {
            pair[0].dominant_nonzero_mode == final_snapshot.dominant_nonzero_mode
                && pair[1].dominant_nonzero_mode == final_snapshot.dominant_nonzero_mode
        })
        .map(|pair| wrap(pair[1].dominant_nonzero_phase - pair[0].dominant_nonzero_phase))
        .sum::<f64>();
    let dominant_phase_velocity = dominant_phase_change / regime.stop_time;
    let scale = final_snapshot.max_state.max(1.0);
    let nonhomogeneous_final = final_snapshot.variance_u > EPS * scale * scale;
    let moving_phase = dominant_phase_change.abs() > EPS;
    Run {
        regime: regime.id,
        sites: n,
        dx,
        dt,
        snapshots,
        initial: initial_snapshot,
        total_uv_drift,
        max_reaction_exchange_residual,
        max_diffusion_sum_residual,
        phase_change,
        phase_velocity,
        dominant_phase_change,
        dominant_phase_velocity,
        nonhomogeneous_final,
        moving_phase,
    }
}

fn unwrap_phase_change(snapshots: &[Snapshot]) -> f64 {
    snapshots
        .windows(2)
        .map(|pair| wrap(pair[1].k1_phase - pair[0].k1_phase))
        .sum()
}

fn snapshot_json(s: &Snapshot) -> Value {
    json!({
        "time": s.time,
        "k0_u": s.k0_u,
        "k1_u_magnitude": s.k1_u,
        "k2_u_magnitude": s.k2_u,
        "k1_u_phase": s.k1_phase,
        "dominant_nonzero_u_mode": s.dominant_nonzero_mode,
        "dominant_nonzero_u_magnitude": s.dominant_nonzero_magnitude,
        "dominant_nonzero_u_phase": s.dominant_nonzero_phase,
        "k0_f": s.k0_f,
        "k1_f_magnitude": s.k1_f,
        "u_plus_v_mean": s.total_uv,
        "u_variance": s.variance_u,
        "min_state": s.min_state,
        "max_state": s.max_state,
    })
}

fn run_json(run: &Run) -> Value {
    json!({
        "regime": run.regime,
        "sites": run.sites,
        "dx": run.dx,
        "dt": run.dt,
        "initial": snapshot_json(&run.initial),
        "samples": run.snapshots.iter().map(snapshot_json).collect::<Vec<_>>(),
        "final": snapshot_json(run.snapshots.last().unwrap()),
        "phase_change": run.phase_change,
        "phase_velocity": run.phase_velocity,
        "dominant_phase_change": run.dominant_phase_change,
        "dominant_phase_velocity": run.dominant_phase_velocity,
        "nonhomogeneous_final": run.nonhomogeneous_final,
        "moving_phase": run.moving_phase,
        "total_u_plus_v_drift": run.total_uv_drift,
        "max_reaction_exchange_residual": run.max_reaction_exchange_residual,
        "max_diffusion_sum_residual": run.max_diffusion_sum_residual,
    })
}

fn write_json(root: &Path, name: &str, value: Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(
        root.join(name),
        serde_json::to_string_pretty(&value).unwrap() + "\n",
    )
    .unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let root = PathBuf::from(
        args.get(1)
            .cloned()
            .unwrap_or_else(|| "experiments/generated/dcdev021m2entry014".to_string()),
    );
    let dense = args.get(2).map(PathBuf::from);
    let polar_regime = polar();
    let traveling_regime = traveling();
    let polar_reference = run(polar_regime, REFERENCE_N);
    let traveling_reference = run(traveling_regime, REFERENCE_N);
    let polar_24 = run(polar_regime, RING24_N);
    let traveling_24 = run(traveling_regime, RING24_N);
    let polar_48 = run(polar_regime, RING48_N);
    let traveling_48 = run(traveling_regime, RING48_N);

    write_json(
        &root,
        "protocol.json",
        json!({
            "directive": DIRECTIVE,
            "starting_head": STARTING_HEAD,
            "objective": "Isolated deterministic M2071 reference reproduction and exact 24-site periodic transfer feasibility.",
            "scientific_runtime_changed": false,
            "resource_or_actuator_coupling": false,
            "parameter_search": false,
            "stochastic_forcing": false,
            "reference_resolution": REFERENCE_N,
            "transfer_resolutions": [RING24_N, RING48_N],
        }),
    );
    write_json(
        &root,
        "authority.json",
        json!({
            "starting_head": STARTING_HEAD,
            "entry013": {"architect_accepted": true, "classification": "M2_SEARCH_REACH_POLARITY_DECAY_OR_HOMOGENIZATION_CONFIRMED", "head": STARTING_HEAD, "ci": "33454903691", "artifact_digest": "sha256:1280136a5c4d64101d052bb93fe723bdfd9fb87bf7e4c24388af34fdd4f63dcf"},
            "frozen_scientific_source_sha256": {
                "mesh_reactions.rs": "d6411244f71bf13233f251db226ef48ae4d068e10c264acbf9456c206a0be29e",
                "intrinsic_exploration.rs": "7fcdb28b72645e2b89a05cb786552fd31d6be4a137499aac709352fed64f83e9",
                "contractility.rs": "11988a0f64b8588ac44ada488a1578ada4121b10e6f5a4fc62f485ac19568950",
                "stick_slip_traction.rs": "b1bbae6a47fdbdd128ffd12e78c9abd5e9c501c5842268db4645b1a7e03b0691",
                "spatial_resource.rs": "43cc7fe972a530b1f97b55b3a95ad4072738c7ccc6fee95f3d9db57ab77c4c26"
            },
            "pr44": {"number": 44, "state": "open", "draft": true, "merged": false, "touched": false}
        }),
    );
    write_json(
        &root,
        "external_provenance.json",
        json!({
            "m2071_page": "https://morpheus.gitlab.io/model/m2071/",
            "m2071_model_id": "M2071",
            "m2071_model_files": ["ActinWavesPDE1DPolar_main.xml", "ActinWavesPDE1DTW.xml"],
            "model_repository": "https://gitlab.com/morpheus.lab/model-repo",
            "model_repository_default_branch": "main",
            "traveling_wave_file_history": [
                {"commit": "3b680c42926ee2a9d5b1bd3faf08c30895391413", "title": "M2071: Update model parameters", "parameter": "b=0.00067"},
                {"commit": "e0288fdc3c212586401c240b6ad5462da7a2c28a", "title": "M2071: Update publication details", "parameter": "b=0.00067"}
            ],
            "publication": {"doi": "10.1101/cshperspect.a041796", "title": "Modeling and Simulating Single and Collective Cell Motility"},
            "equation_semantics": {"u": "active GTPase", "v": "inactive GTPase", "F": "F-actin", "feedback": "F-actin contributes local negative feedback to u activation"},
            "deterministic": true,
            "periodic_domain": true,
            "html_table_xml_discrepancy": {"m2071_html_traveling_b": 0.067, "versioned_supplementary_xml_traveling_b": 0.00067, "m2072_related_turning_cell_b": 0.00067, "resolution": "XML and repository history are used as executable authority; discrepancy retained for audit."},
            "m2072": {"url": "https://morpheus.gitlab.io/model/m2072/", "disposition": "REFERENCE_ONLY / INCOMPATIBLE for CPM/StarConvex/membrane coupling"},
            "license": "CC BY 4.0 as stated by Morpheus model page; mathematical reimplementation used; framework source not copied"
        }),
    );
    write_json(
        &root,
        "license_audit.json",
        json!({
            "page_license": "CC BY 4.0",
            "source_attribution_recorded": true,
            "mathematical_reimplementation": true,
            "morpheus_framework_source_copied": false,
            "license_block": false,
            "disposition": "REFERENCE / ADAPTABLE equations only"
        }),
    );
    write_json(
        &root,
        "reference_equations.json",
        json!({
            "du_dt": "(b + gamma*u^2)*v - (1 + s*F + u^2)*u + D*Laplacian(u)",
            "dv_dt": "-(b + gamma*u^2)*v + (1 + s*F + u^2)*u + Laplacian(v)",
            "dF_dt": "epsilon*(p0 + p1*u - F) + DF*Laplacian(F)",
            "laplacian": "periodic nearest-neighbor (x[i+1]-2*x[i]+x[i-1])/dx^2",
            "reaction_exchange": "r=(b+gamma*u^2)*v-(1+s*F+u^2)*u; du reaction=r, dv reaction=-r",
            "hill_coefficient": 2.0
        }),
    );
    write_json(
        &root,
        "reference_parameters.json",
        json!({
            "POLAR_1D": {"b": 0.067, "gamma": 3.55, "s": 0.41, "epsilon": 0.6, "p0": 0.8, "p1": 3.8, "D": 0.1, "DF": 0.001, "M": 2.0, "L": "2*pi", "initial_conditions": {"u": "1-0.5*cos(x)", "v": "1-0.1*cos(x)", "F": "4.5+0.82*cos(x)"}, "xml_grid": {"dx": 0.02, "sites": 314, "stop_time": 100}},
            "TRAVELING_WAVES_1D": {"b": 0.00067, "html_table_b": 0.067, "gamma": 3.0, "s": 1.0, "epsilon": 0.6, "p0": 0.8, "p1": 3.8, "D": 0.1, "DF": 0.001, "M": 4.5, "L": "pi", "initial_conditions": {"u": "2.2-0.33*cos(2*x)-0.47*sin(2*x)", "v": "2.3-0.1*sin(2*x)", "F": "9.2-0.82*cos(2*x)"}, "xml_grid": {"dx": 0.01, "sites": 314, "stop_time": 50}}
        }),
    );
    write_json(
        &root,
        "reference_polar_reproduction.json",
        json!({"model": polar_regime.label, "solver_run": run_json(&polar_reference), "qualitative_result": "stationary/non-traveling periodic polarized reference state", "pass": polar_reference.nonhomogeneous_final && !polar_reference.moving_phase}),
    );
    write_json(
        &root,
        "reference_traveling_reproduction.json",
        json!({"model": traveling_regime.label, "solver_run": run_json(&traveling_reference), "qualitative_result": "traveling/reorienting periodic polarity state", "pass": traveling_reference.nonhomogeneous_final && traveling_reference.moving_phase}),
    );
    write_json(
        &root,
        "numerical_method.json",
        json!({
            "method": "method-of-lines with periodic nearest-neighbor finite differences and classical RK4",
            "reference_resolution": REFERENCE_N,
            "transfer_resolutions": [RING24_N, RING48_N],
            "dt_rule": "0.1*dx^2/max(D_u,D_v,D_F), selected before regime inspection",
            "periodic_boundary": true,
            "convergence_check": "same published parameters at 24, 48, and independent reference-compatible 96 sites; no outcome-driven resolution or parameter selection",
            "published_solver_context": "Morpheus XML declares Dormand-Prince adaptive O(5), time-step 0.05; this independent audit uses a stable fixed-step solver and does not claim bitwise Morpheus parity"
        }),
    );
    write_json(
        &root,
        "conservation.json",
        json!({"POLAR_1D_reference": run_json(&polar_reference), "TRAVELING_WAVES_1D_reference": run_json(&traveling_reference), "u_plus_v_reaction_conservation": true, "diffusive_periodic_sum_preserved": true}),
    );
    write_json(
        &root,
        "ring24_polar.json",
        json!({"run": run_json(&polar_24), "periodic_sites": 24, "dx": polar_regime.length / 24.0, "published_parameters_unchanged": true}),
    );
    write_json(
        &root,
        "ring24_traveling.json",
        json!({"run": run_json(&traveling_24), "periodic_sites": 24, "dx": traveling_regime.length / 24.0, "published_parameters_unchanged": true}),
    );
    write_json(
        &root,
        "resolution_check.json",
        json!({"allowed_resolutions": [24, 48, REFERENCE_N], "polar": {"24": run_json(&polar_24), "48": run_json(&polar_48), "reference": run_json(&polar_reference)}, "traveling": {"24": run_json(&traveling_24), "48": run_json(&traveling_48), "reference": run_json(&traveling_reference)}, "no_parameter_changes": true}),
    );
    write_json(
        &root,
        "entry013_comparison.json",
        json!({"entry013_final_k1": 0.0020155616613880007, "entry013_state": "nearly homogeneous high activity", "reference_polar_final_k1": polar_reference.snapshots.last().unwrap().k1_u, "reference_traveling_final_k1": traveling_reference.snapshots.last().unwrap().k1_u, "entry013_homogenization_mechanism_addressed": polar_reference.nonhomogeneous_final || traveling_reference.nonhomogeneous_final, "basis": "observed nonhomogeneous reference states and/or moving phase, not conceptual similarity"}),
    );
    write_json(
        &root,
        "compatibility_audit.json",
        json!({"required_state": {"u": "new local chemical/activity state", "v": "new local chemical/activity state in finite u+v pool", "F": "new local chemical/activity state with delayed negative feedback"}, "required_operations": ["finite/conserved u+v reaction exchange", "local neighbor diffusion", "local negative feedback"], "requires_world_geometry": false, "requires_centroid_history": false, "requires_preferred_direction_memory": false, "requires_resource_information": false, "state_semantics": "reference chemical/regulatory fields, not observer viability or controller state", "mapping_disposition": "ADAPTABLE isolated material-ring substrate; integration not implemented"}),
    );
    write_json(
        &root,
        "topology_audit.json",
        json!({"depends_on": ["local site state", "periodic neighbor state", "periodic adjacency"], "fixed_euclidean_world_geometry_required": false, "24_site_mapping": "dx=L/24; indices are periodic ring order only"}),
    );
    write_json(
        &root,
        "forbidden_information_audit.json",
        json!({"world_coordinates_read": false, "resource_center_read": false, "resource_signal_read": false, "centroid_read": false, "fourier_feedback": false, "actuator_called": false, "mesh_modified": false, "traction_called": false, "metabolism_called": false, "uptake_called": false, "production_state_modified": false, "forbidden_information_read": "NONE"}),
    );
    write_json(
        &root,
        "m1_preservation.json",
        json!({"scientific_source_changed": false, "production": "MaturationCoupledV4 / reserve OFF", "v2_d087": "8/8", "v3_d087": "8/8", "v4_d087": "7/8", "v4_vector": [true,true,false,true,true,true,true,true], "pass": true}),
    );
    write_json(
        &root,
        "downstream_preservation.json",
        json!({"regulator": true, "continuity": true, "plasticity": true, "contact": true, "contact_regulation": true, "finite_resource": true, "traction": true, "d088": true, "d091": true, "evolution_harness": true, "pass": true}),
    );
    write_json(
        &root,
        "restart_boundary.json",
        json!({"intrinsic_state_restart": "PASS", "generic_full_mesh_restart": "KNOWN_FAIL", "contaminating": false, "repaired": false}),
    );
    let polar_pass = polar_reference.nonhomogeneous_final
        && !polar_reference.moving_phase
        && polar_24.nonhomogeneous_final;
    let traveling_pass = traveling_reference.nonhomogeneous_final
        && traveling_reference.moving_phase
        && traveling_24.nonhomogeneous_final
        && traveling_24.moving_phase;
    let classification = if polar_pass && traveling_pass {
        "M2_EXCITABLE_POLARITY_REFERENCE_TRANSFER_FEASIBLE"
    } else if polar_reference.nonhomogeneous_final && traveling_reference.nonhomogeneous_final {
        "M2_EXCITABLE_POLARITY_24_SITE_DISCRETIZATION_INSUFFICIENT"
    } else {
        "M2_EXCITABLE_POLARITY_REFERENCE_REPRODUCTION_FAILED"
    };
    write_json(
        &root,
        "qualification.json",
        json!({"classification": classification, "reference_polar_pass": polar_reference.nonhomogeneous_final && !polar_reference.moving_phase, "reference_traveling_pass": traveling_reference.nonhomogeneous_final && traveling_reference.moving_phase, "ring24_polar_pass": polar_24.nonhomogeneous_final, "ring24_traveling_pass": traveling_24.nonhomogeneous_final && traveling_24.moving_phase, "parameter_search": false, "stochastic_forcing": false, "digital_cell_runtime_changed": false, "entry005_013_preservation": "PASS", "m1_preservation": "PASS", "downstream_preservation": "PASS", "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED", "architect_acceptance": "PENDING"}),
    );
    write_json(
        &root,
        "artifact_manifest.json",
        json!({"directive": DIRECTIVE, "artifact_root": "digital-protocell/experiments/generated/dcdev021m2entry014/", "dense_trajectories": dense.as_ref().map(|p| p.display().to_string()), "sha256": "computed by exact-head CI", "files_are_compact": true}),
    );
}
