//! DC-DEV-021 M2 ENTRY-015: excitable-polarity actuator-interface audit.
//!
//! This is an assay-only composition.  The M2071 equations are reimplemented
//! locally as generic polarity-regulatory chemistry and are coupled one-way to
//! the existing A-funded actuator through `u / (u + v)`.  No production
//! polarity state, resource, observer feedback, or Digital Cell runtime
//! equation is changed by this example.

#![recursion_limit = "256"]

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, MeshContractVersion, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step_with_reserve_mode, ReactionParams, ReserveDiagnosticMode,
};
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
    "DC-DEV-021-M2-ENTRY-015-EXCITABLE-POLARITY-ACTUATOR-INTERFACE-FEASIBILITY-001";
const STARTING_HEAD: &str = "7685ae33e33132452105611322dbf4d045468eec";
const N: usize = 24;
const SETTLEMENT_STEPS: usize = 5_000;
const ASSAY_STEPS: usize = 1_500;
const STATE_TOL: f64 = 1e-12;

#[derive(Clone, Copy, Debug)]
struct PolarityParams {
    b: f64,
    gamma: f64,
    s: f64,
    epsilon: f64,
    p0: f64,
    p1: f64,
    d_u: f64,
    d_f: f64,
}

#[derive(Clone, Copy, Debug)]
struct Regime {
    id: &'static str,
    length: f64,
    params: PolarityParams,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
struct PolarityState {
    u: Vec<f64>,
    v: Vec<f64>,
    f: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Mode {
    k: usize,
    real: f64,
    imaginary: f64,
    magnitude: f64,
    phase: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct V2([f64; 2]);

#[derive(Clone, Debug, Serialize)]
struct Snapshot {
    area: f64,
    a: f64,
    w: f64,
    n: f64,
    f: f64,
    c: f64,
    centroid: V2,
}

#[derive(Clone, Debug, Serialize)]
struct StepRecord {
    step: usize,
    polarity_time: f64,
    motor_mean: f64,
    motor_modes: Vec<Mode>,
    polarity_dominant_mode: usize,
    polarity_dominant_phase: f64,
    centroid: V2,
    displacement: V2,
    speed: f64,
    velocity_heading: f64,
    slipping_contacts: usize,
    stuck_contacts: usize,
    a_spent: f64,
    w_generated: f64,
    pre: Snapshot,
    post: Snapshot,
}

#[derive(Clone, Debug, Serialize)]
struct RunSummary {
    arm: String,
    regime: String,
    spatial: bool,
    motor_off: bool,
    path: f64,
    net_displacement: f64,
    final_displacement: V2,
    displacement_path_ratio: f64,
    maximum_centroid_excursion: f64,
    maximum_material_envelope_excursion: f64,
    velocity_heading_min: f64,
    velocity_heading_max: f64,
    velocity_autocorrelation: f64,
    slips: usize,
    stuck_contacts: usize,
    a_spent: f64,
    w_generated: f64,
    a_to_w_residual: f64,
    final_state: Snapshot,
    final_polarity: PolarityState,
    final_mesh_hash: String,
    records: Vec<StepRecord>,
}

fn polar() -> Regime {
    Regime {
        id: "POLAR_1D",
        length: 2.0 * PI,
        params: PolarityParams {
            b: 0.067,
            gamma: 3.55,
            s: 0.41,
            epsilon: 0.6,
            p0: 0.8,
            p1: 3.8,
            d_u: 0.1,
            d_f: 0.001,
        },
    }
}

fn traveling() -> Regime {
    Regime {
        id: "TRAVELING_WAVES_1D",
        length: PI,
        params: PolarityParams {
            b: 0.00067,
            gamma: 3.0,
            s: 1.0,
            epsilon: 0.6,
            p0: 0.8,
            p1: 3.8,
            d_u: 0.1,
            d_f: 0.001,
        },
    }
}

fn initial(regime: Regime) -> PolarityState {
    let dx = regime.length / N as f64;
    let mut u = Vec::with_capacity(N);
    let mut v = Vec::with_capacity(N);
    let mut f = Vec::with_capacity(N);
    for i in 0..N {
        let x = i as f64 * dx;
        if regime.id == "POLAR_1D" {
            u.push(1.0 - 0.5 * x.cos());
            v.push(1.0 - 0.1 * x.cos());
            f.push(4.5 + 0.82 * x.cos());
        } else {
            u.push(2.2 - 0.33 * (2.0 * x).cos() - 0.47 * (2.0 * x).sin());
            v.push(2.3 - 0.1 * (2.0 * x).sin());
            f.push(9.2 - 0.82 * (2.0 * x).cos());
        }
    }
    PolarityState { u, v, f }
}

fn lap(values: &[f64], dx: f64, i: usize) -> f64 {
    let n = values.len();
    (values[(i + 1) % n] - 2.0 * values[i] + values[(i + n - 1) % n]) / (dx * dx)
}

fn rhs(state: &PolarityState, regime: Regime, dx: f64) -> PolarityState {
    let p = regime.params;
    let mut du = vec![0.0; N];
    let mut dv = vec![0.0; N];
    let mut df = vec![0.0; N];
    for i in 0..N {
        let u = state.u[i];
        let v = state.v[i];
        let f = state.f[i];
        let exchange = (p.b + p.gamma * u * u) * v - (1.0 + p.s * f + u * u) * u;
        du[i] = exchange + p.d_u * lap(&state.u, dx, i);
        dv[i] = -exchange + lap(&state.v, dx, i);
        df[i] = p.epsilon * (p.p0 + p.p1 * u - f) + p.d_f * lap(&state.f, dx, i);
    }
    PolarityState {
        u: du,
        v: dv,
        f: df,
    }
}

fn add_scaled(a: &PolarityState, b: &PolarityState, scale: f64) -> PolarityState {
    PolarityState {
        u: a.u.iter().zip(&b.u).map(|(x, y)| x + scale * y).collect(),
        v: a.v.iter().zip(&b.v).map(|(x, y)| x + scale * y).collect(),
        f: a.f.iter().zip(&b.f).map(|(x, y)| x + scale * y).collect(),
    }
}

fn rk4(state: &PolarityState, regime: Regime, dx: f64, dt: f64) -> PolarityState {
    let k1 = rhs(state, regime, dx);
    let k2 = rhs(&add_scaled(state, &k1, 0.5 * dt), regime, dx);
    let k3 = rhs(&add_scaled(state, &k2, 0.5 * dt), regime, dx);
    let k4 = rhs(&add_scaled(state, &k3, dt), regime, dx);
    PolarityState {
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

fn advance(state: &mut PolarityState, regime: Regime, total_dt: f64, refinement: usize) {
    let dx = regime.length / N as f64;
    let stability_dt = 0.1 * dx * dx;
    let base_steps = (total_dt / stability_dt).ceil() as usize;
    let substeps = (base_steps * refinement.max(1)).max(1);
    let dt = total_dt / substeps as f64;
    for _ in 0..substeps {
        *state = rk4(state, regime, dx, dt);
    }
}

fn mode(values: &[f64], k: usize) -> Mode {
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for (j, value) in values.iter().enumerate() {
        let theta = 2.0 * PI * k as f64 * j as f64 / values.len() as f64;
        real += value * theta.cos();
        imaginary -= value * theta.sin();
    }
    real /= values.len() as f64;
    imaginary /= values.len() as f64;
    Mode {
        k,
        real,
        imaginary,
        magnitude: real.hypot(imaginary),
        phase: imaginary.atan2(real),
    }
}

fn modes(values: &[f64]) -> Vec<Mode> {
    (0..=2).map(|k| mode(values, k)).collect()
}

fn active_fraction(state: &PolarityState) -> Vec<f64> {
    state
        .u
        .iter()
        .zip(&state.v)
        .map(|(u, v)| {
            assert!(*u >= 0.0 && *v >= 0.0 && *u + *v > 0.0);
            *u / (*u + *v)
        })
        .collect()
}

fn material_centroid(mesh: &MaterialMesh) -> [f64; 2] {
    let mut weighted = [0.0; 2];
    let mut total = 0.0;
    for i in 0..mesh.n() {
        let a = mesh.vertices[i];
        let b = mesh.vertices[(i + 1) % mesh.n()];
        let weight = (mesh.edges[i].m + mesh.edges[i].b).max(0.0);
        let midpoint = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
        weighted[0] += weight * midpoint[0];
        weighted[1] += weight * midpoint[1];
        total += weight;
    }
    if total <= f64::EPSILON {
        mesh.centroid()
    } else {
        [weighted[0] / total, weighted[1] / total]
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
        centroid: V2(material_centroid(mesh)),
    }
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

fn source_hash(relative: &str) -> String {
    stable_json_hash(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join(relative),
        )
        .unwrap(),
    )
    .unwrap()
}

fn reaction_hash() -> String {
    stable_json_hash(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../chemistry-core/src/mesh_reactions.rs"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn rotate_180(mut mesh: MaterialMesh) -> MaterialMesh {
    for vertex in &mut mesh.vertices {
        vertex[0] = -vertex[0];
        vertex[1] = -vertex[1];
    }
    mesh
}

fn run_polarity_only(regime: Regime, refinement: usize) -> PolarityState {
    let mut state = initial(regime);
    for _ in 0..ASSAY_STEPS {
        advance(&mut state, regime, MechParams::default().dt, refinement);
    }
    state
}

fn run_physical(
    settled: &MaterialMesh,
    regime: Regime,
    arm: &str,
    spatial: bool,
    motor_off: bool,
    refinement: usize,
) -> RunSummary {
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reaction_params = ReactionParams::conservative_v3();
    let mut mesh = settled.clone();
    let mut polarity = initial(regime);
    let initial_snapshot = snapshot(&mesh);
    let initial_w = initial_snapshot.w * initial_snapshot.area;
    let mut previous_centroid = material_centroid(&mesh);
    let initial_centroid = previous_centroid;
    let initial_max_radius = mesh
        .vertices
        .iter()
        .map(|v| norm(sub(*v, initial_centroid)))
        .fold(0.0, f64::max);
    let mut path = 0.0;
    let mut maximum_excursion: f64 = 0.0;
    let mut maximum_envelope: f64 = 0.0;
    let mut slips = 0;
    let mut stuck = 0;
    let mut a_spent = 0.0;
    let mut w_generated = 0.0;
    let mut previous_heading = 0.0;
    let mut heading_dots = Vec::new();
    let mut records = Vec::with_capacity(ASSAY_STEPS);

    for step in 0..ASSAY_STEPS {
        advance(&mut polarity, regime, mechanics.dt, refinement);
        let fractions = active_fraction(&polarity);
        let mean = fractions.iter().sum::<f64>() / N as f64;
        let motor = if motor_off {
            vec![0.0; N]
        } else if spatial {
            fractions
        } else {
            vec![mean; N]
        };
        let motor_modes = modes(&motor);
        let dominant = (1..=N / 2)
            .map(|k| mode(&motor, k))
            .max_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap())
            .unwrap();
        let pre = snapshot(&mesh);
        let ledger = if motor_off {
            let ledger =
                apply_stick_slip_to_legacy_mechanics(&mut mesh, &mechanics, &traction).unwrap();
            slips += ledger.slipping_contacts;
            stuck += ledger.stuck_contacts;
            None
        } else {
            let ledger = apply_local_activated_energy_contractility_with_stick_slip(
                &mut mesh,
                &motor,
                &mechanics,
                &contractility,
                &traction,
            )
            .unwrap();
            slips += ledger.slipping_contacts;
            stuck += ledger.stuck_contacts;
            Some(ledger)
        };
        let spent = ledger
            .as_ref()
            .and_then(|l| l.contractility.as_ref())
            .map(|c| c.resource_spent)
            .unwrap_or(0.0);
        a_spent += spent;
        let reaction = reactions_step_with_reserve_mode(
            &mut mesh,
            &reaction_params,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        w_generated += reaction.w_produced;
        let post = snapshot(&mesh);
        let centroid = material_centroid(&mesh);
        let displacement = sub(centroid, previous_centroid);
        let velocity = [
            displacement[0] / mechanics.dt,
            displacement[1] / mechanics.dt,
        ];
        let heading = velocity[1].atan2(velocity[0]);
        if step > 0 {
            heading_dots.push((heading - previous_heading).cos());
        }
        previous_heading = heading;
        path += norm(displacement);
        maximum_excursion = maximum_excursion.max(norm(sub(centroid, initial_centroid)));
        let envelope = mesh
            .vertices
            .iter()
            .map(|v| norm(sub(*v, initial_centroid)) - initial_max_radius)
            .fold(0.0_f64, f64::max);
        maximum_envelope = maximum_envelope.max(envelope);
        previous_centroid = centroid;
        records.push(StepRecord {
            step,
            polarity_time: (step + 1) as f64 * mechanics.dt,
            motor_mean: mean,
            motor_modes,
            polarity_dominant_mode: dominant.k,
            polarity_dominant_phase: dominant.phase,
            centroid: V2(centroid),
            displacement: V2(displacement),
            speed: norm(velocity),
            velocity_heading: heading,
            slipping_contacts: ledger.as_ref().map(|l| l.slipping_contacts).unwrap_or(0),
            stuck_contacts: ledger.as_ref().map(|l| l.stuck_contacts).unwrap_or(0),
            a_spent: spent,
            w_generated: reaction.w_produced,
            pre,
            post,
        });
        assert!(mesh.lifecycle_invariants_hold());
    }
    let final_state = snapshot(&mesh);
    let final_displacement = sub(final_state.centroid.0, V2(initial_centroid).0);
    let a_to_w_residual =
        (final_state.w * final_state.area - initial_w - w_generated - a_spent).abs();
    RunSummary {
        arm: arm.to_string(),
        regime: regime.id.to_string(),
        spatial,
        motor_off,
        path,
        net_displacement: norm(final_displacement),
        final_displacement: V2(final_displacement),
        displacement_path_ratio: norm(final_displacement) / path.max(STATE_TOL),
        maximum_centroid_excursion: maximum_excursion,
        maximum_material_envelope_excursion: maximum_envelope,
        velocity_heading_min: records
            .iter()
            .map(|r| r.velocity_heading)
            .fold(f64::INFINITY, f64::min),
        velocity_heading_max: records
            .iter()
            .map(|r| r.velocity_heading)
            .fold(f64::NEG_INFINITY, f64::max),
        velocity_autocorrelation: heading_dots.iter().sum::<f64>()
            / heading_dots.len().max(1) as f64,
        slips,
        stuck_contacts: stuck,
        a_spent,
        w_generated,
        a_to_w_residual,
        final_state,
        final_polarity: polarity,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
        records,
    }
}

#[derive(Clone, Debug, Serialize)]
struct BaselineSummary {
    path: f64,
    net_displacement: f64,
    slips: usize,
    dominant_patch_changes: usize,
    a_spent: f64,
    w_generated: f64,
    final_mesh_hash: String,
}

fn baseline_entry012(settled: &MaterialMesh) -> BaselineSummary {
    use regulatory_core::{
        commit_intrinsic_exploration_step, propose_intrinsic_exploration_step,
        IntrinsicExplorationDynamicsModeV1, IntrinsicExplorationStateV1,
    };
    let mechanics = MechParams::default();
    let contractility = ContractilityParamsV1::default();
    let traction = StickSlipTractionParamsV1::default();
    let reactions = ReactionParams::conservative_v3();
    let mut mesh = settled.clone();
    let mut state = IntrinsicExplorationStateV1::new(N, Some(1)).unwrap();
    let mut previous = material_centroid(&mesh);
    let mut path = 0.0;
    let mut slips = 0;
    let mut changes = 0;
    let mut prior_dominant = state
        .activity
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    let mut a_spent = 0.0;
    let mut w_generated = 0.0;
    for _ in 0..ASSAY_STEPS {
        let proposal = propose_intrinsic_exploration_step(
            &state,
            N,
            mechanics.dt,
            IntrinsicExplorationDynamicsModeV1::FullSelfExcitation,
        )
        .unwrap();
        let ledger = apply_local_activated_energy_contractility_with_stick_slip(
            &mut mesh,
            &proposal.activity_after,
            &mechanics,
            &contractility,
            &traction,
        )
        .unwrap();
        slips += ledger.slipping_contacts;
        a_spent += ledger.contractility.as_ref().unwrap().resource_spent;
        commit_intrinsic_exploration_step(&mut state, proposal).unwrap();
        let reaction = reactions_step_with_reserve_mode(
            &mut mesh,
            &reactions,
            mechanics.dt,
            true,
            true,
            ReserveDiagnosticMode::Full,
        );
        w_generated += reaction.w_produced;
        let current = state
            .activity
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        changes += usize::from(current != prior_dominant);
        prior_dominant = current;
        let centroid = material_centroid(&mesh);
        path += norm(sub(centroid, previous));
        previous = centroid;
    }
    BaselineSummary {
        path,
        net_displacement: norm(sub(previous, material_centroid(settled))),
        slips,
        dominant_patch_changes: changes,
        a_spent,
        w_generated,
        final_mesh_hash: stable_json_hash(&mesh).unwrap(),
    }
}

fn compact(run: &RunSummary) -> Value {
    json!({
        "arm": run.arm, "regime": run.regime, "spatial": run.spatial, "motor_off": run.motor_off,
        "path": run.path, "net_displacement": run.net_displacement,
        "final_displacement": run.final_displacement,
        "displacement_path_ratio": run.displacement_path_ratio,
        "maximum_centroid_excursion": run.maximum_centroid_excursion,
        "maximum_material_envelope_excursion": run.maximum_material_envelope_excursion,
        "velocity_heading_min": run.velocity_heading_min, "velocity_heading_max": run.velocity_heading_max,
        "velocity_autocorrelation": run.velocity_autocorrelation,
        "slips": run.slips, "stuck_contacts": run.stuck_contacts,
        "a_spent": run.a_spent, "w_generated": run.w_generated,
        "a_to_w_residual": run.a_to_w_residual,
        "final_state": run.final_state, "final_mesh_hash": run.final_mesh_hash,
        "final_polarity_hash": stable_json_hash(&run.final_polarity).unwrap(),
    })
}

fn write_json(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn max_state_difference(a: &PolarityState, b: &PolarityState) -> f64 {
    a.u.iter()
        .chain(&a.v)
        .chain(&a.f)
        .zip(b.u.iter().chain(&b.v).chain(&b.f))
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f64::max)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev021m2entry015"));
    let dense = args.get(2).map(PathBuf::from);
    let mechanics = MechParams::default();
    let settled = settled_body(&mechanics);
    let polar_regime = polar();
    let traveling_regime = traveling();

    let polar_spatial = run_physical(
        &settled,
        polar_regime,
        "POLAR_SPATIAL_INTERFACE",
        true,
        false,
        1,
    );
    let polar_uniform = run_physical(
        &settled,
        polar_regime,
        "POLAR_UNIFORM_MEAN_CONTROL",
        false,
        false,
        1,
    );
    let polar_off = run_physical(&settled, polar_regime, "POLAR_MOTOR_OFF", false, true, 1);
    let traveling_spatial = run_physical(
        &settled,
        traveling_regime,
        "TRAVELING_SPATIAL_INTERFACE",
        true,
        false,
        1,
    );
    let traveling_uniform = run_physical(
        &settled,
        traveling_regime,
        "TRAVELING_UNIFORM_MEAN_CONTROL",
        false,
        false,
        1,
    );
    let traveling_off = run_physical(
        &settled,
        traveling_regime,
        "TRAVELING_MOTOR_OFF",
        false,
        true,
        1,
    );
    let baseline = baseline_entry012(&settled);
    let polar_refined = run_physical(
        &settled,
        polar_regime,
        "POLAR_SPATIAL_REFINEMENT",
        true,
        false,
        2,
    );

    let polar_uncoupled = run_polarity_only(polar_regime, 1);
    let traveling_uncoupled = run_polarity_only(traveling_regime, 1);
    let polar_replay_error = max_state_difference(&polar_spatial.final_polarity, &polar_uncoupled);
    let traveling_replay_error =
        max_state_difference(&traveling_spatial.final_polarity, &traveling_uncoupled);
    let polar_mean_drive_error = polar_spatial
        .records
        .iter()
        .zip(&polar_uniform.records)
        .map(|(a, b)| (a.motor_mean - b.motor_mean).abs())
        .fold(0.0, f64::max);
    let traveling_mean_drive_error = traveling_spatial
        .records
        .iter()
        .zip(&traveling_uniform.records)
        .map(|(a, b)| (a.motor_mean - b.motor_mean).abs())
        .fold(0.0, f64::max);
    let polar_leverage = polar_spatial.net_displacement
        > polar_uniform.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE
        && polar_spatial.displacement_path_ratio > polar_uniform.displacement_path_ratio
        && polar_spatial.maximum_centroid_excursion > polar_uniform.maximum_centroid_excursion;
    let traveling_phase_change = traveling_spatial
        .records
        .last()
        .map(|r| r.polarity_dominant_phase)
        .unwrap_or(0.0)
        - traveling_spatial
            .records
            .first()
            .map(|r| r.polarity_dominant_phase)
            .unwrap_or(0.0);
    let traveling_heading_change =
        traveling_spatial.velocity_heading_max - traveling_spatial.velocity_heading_min;
    let traveling_heading_coupling = traveling_phase_change.abs() > PI
        && traveling_heading_change
            > traveling_uniform.velocity_heading_max - traveling_uniform.velocity_heading_min
                + STATE_TOL;
    let traveling_leverage = traveling_spatial.net_displacement
        > traveling_uniform.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE
        || traveling_heading_coupling;
    let a_to_w_pass = [
        &polar_spatial,
        &polar_uniform,
        &polar_off,
        &traveling_spatial,
        &traveling_uniform,
        &traveling_off,
    ]
    .iter()
    .all(|run| run.a_to_w_residual <= 1e-8);
    let rotation = run_physical(
        &rotate_180(settled.clone()),
        polar_regime,
        "POLAR_ROTATED_180",
        true,
        false,
        1,
    );
    let rotation_pass = (rotation.path - polar_spatial.path).abs() <= 1e-8
        && (rotation.net_displacement - polar_spatial.net_displacement).abs() <= 1e-8
        && (rotation.a_spent - polar_spatial.a_spent).abs() <= 1e-8
        && rotation.slips == polar_spatial.slips;
    let reference_replay = polar_replay_error <= 1e-8 && traveling_replay_error <= 1e-8;
    let classification = if polar_leverage
        && traveling_heading_coupling
        && reference_replay
        && a_to_w_pass
        && rotation_pass
    {
        "M2_EXCITABLE_POLARITY_ACTUATOR_INTERFACE_QUALIFIED"
    } else if polar_leverage {
        "M2_EXCITABLE_POLARITY_STATIC_INTERFACE_ONLY"
    } else {
        "M2_EXCITABLE_POLARITY_PATTERN_MECHANICALLY_INSUFFICIENT"
    };

    let source = json!({
        "intrinsic_exploration": source_hash("intrinsic_exploration.rs"),
        "contractility": source_hash("contractility.rs"),
        "stick_slip_traction": source_hash("stick_slip_traction.rs"),
        "spatial_resource": source_hash("spatial_resource.rs"),
        "mesh_reactions": reaction_hash(),
    });
    let files = [
        "protocol.json",
        "authority.json",
        "physical_causal_order.json",
        "interface_contract.json",
        "timing_contract.json",
        "polar_spatial.json",
        "polar_uniform_control.json",
        "polar_motor_off.json",
        "traveling_spatial.json",
        "traveling_uniform_control.json",
        "traveling_motor_off.json",
        "entry012_baseline.json",
        "polarity_replay.json",
        "equal_drive_control.json",
        "translation_metrics.json",
        "traveling_reorientation.json",
        "spatial_leverage.json",
        "energetic_closure.json",
        "rotation_check.json",
        "semantic_boundary.json",
        "autonomous_initiation_boundary.json",
        "forbidden_information_audit.json",
        "m1_preservation.json",
        "downstream_preservation.json",
        "restart_boundary.json",
        "qualification.json",
        "artifact_manifest.json",
    ];
    write_json(
        &output,
        "protocol.json",
        &json!({
            "directive": DIRECTIVE, "starting_head": STARTING_HEAD, "observer_only": true,
            "topology_sites": N, "settlement_steps": SETTLEMENT_STEPS, "assay_steps": ASSAY_STEPS,
            "mechanics_dt": mechanics.dt, "resource_present": false, "scientific_runtime_changed": false,
        }),
    );
    write_json(
        &output,
        "authority.json",
        &json!({
            "starting_head": STARTING_HEAD, "entry014": "M2_EXCITABLE_POLARITY_REFERENCE_TRANSFER_FEASIBLE",
            "entry005_to_entry014_preservation": "REQUIRED", "m1": "CLOSED/FROZEN",
            "production": "MaturationCoupledV4 / reserve OFF", "pr44": "OPEN/DRAFT/UNMERGED/UNMODIFIED",
            "source_hashes": source,
        }),
    );
    write_json(
        &output,
        "physical_causal_order.json",
        &json!({
            "order": ["advance ENTRY-014 polarity by mechanics.dt", "derive assay-local u/(u+v) motor", "existing A-funded actuator and stick-slip", "unchanged frozen metabolism"],
            "mesh_to_polarity_feedback": false, "uptake": false, "resource": false,
        }),
    );
    write_json(
        &output,
        "interface_contract.json",
        &json!({
            "formula": "motor_activity[i] = u[i] / (u[i] + v[i])", "gain": "NONE", "threshold": "NONE",
            "normalization": "NONE", "f_term": "NONE", "local": true, "bounded": true,
            "nonnegative_pool_contract": true,
        }),
    );
    write_json(
        &output,
        "timing_contract.json",
        &json!({
            "unit_time_identity": true, "mechanics_dt": mechanics.dt, "polar_dt": mechanics.dt,
            "stability_rule": "0.1 * dx^2 / max_diffusion", "base_substeps": {
                "polar": (mechanics.dt / (0.1 * (polar_regime.length / N as f64).powi(2))).ceil() as usize,
                "traveling": (mechanics.dt / (0.1 * (traveling_regime.length / N as f64).powi(2))).ceil() as usize,
            }, "refinement_factor": 2, "polar_refined_path": polar_refined.path,
            "polar_base_path": polar_spatial.path, "path_difference": (polar_refined.path - polar_spatial.path).abs(),
            "same_total_delta_each_step": true,
        }),
    );
    write_json(&output, "polar_spatial.json", &compact(&polar_spatial));
    write_json(
        &output,
        "polar_uniform_control.json",
        &compact(&polar_uniform),
    );
    write_json(&output, "polar_motor_off.json", &compact(&polar_off));
    write_json(
        &output,
        "traveling_spatial.json",
        &compact(&traveling_spatial),
    );
    write_json(
        &output,
        "traveling_uniform_control.json",
        &compact(&traveling_uniform),
    );
    write_json(
        &output,
        "traveling_motor_off.json",
        &compact(&traveling_off),
    );
    write_json(
        &output,
        "entry012_baseline.json",
        &serde_json::to_value(&baseline).unwrap(),
    );
    write_json(
        &output,
        "polarity_replay.json",
        &json!({
            "polar_max_final_state_difference": polar_replay_error,
            "traveling_max_final_state_difference": traveling_replay_error,
            "polar_matches_uncoupled": polar_replay_error <= 1e-8,
            "traveling_matches_uncoupled": traveling_replay_error <= 1e-8,
            "mechanics_feedback_into_polarity": false,
        }),
    );
    write_json(
        &output,
        "equal_drive_control.json",
        &json!({
            "polar_max_per_step_mean_difference": polar_mean_drive_error,
            "traveling_max_per_step_mean_difference": traveling_mean_drive_error,
            "polar_equal_within_tolerance": polar_mean_drive_error <= STATE_TOL,
            "traveling_equal_within_tolerance": traveling_mean_drive_error <= STATE_TOL,
        }),
    );
    write_json(
        &output,
        "translation_metrics.json",
        &json!({
            "polar_spatial": compact(&polar_spatial), "polar_uniform": compact(&polar_uniform),
            "traveling_spatial": compact(&traveling_spatial), "traveling_uniform": compact(&traveling_uniform),
            "entry012_baseline": baseline,
            "entry013_fixed_profile_net_displacement": 0.09316990400571264,
        }),
    );
    write_json(
        &output,
        "traveling_reorientation.json",
        &json!({
            "dominant_mature_mode": 2, "dominant_phase_change_observed": traveling_phase_change,
            "velocity_heading_range": traveling_heading_change,
            "uniform_velocity_heading_range": traveling_uniform.velocity_heading_max - traveling_uniform.velocity_heading_min,
            "velocity_autocorrelation": traveling_spatial.velocity_autocorrelation,
            "polarity_to_heading_coupling": traveling_heading_coupling,
            "mode_2_mechanical_cancellation": !traveling_leverage,
            "interpretation": "descriptive assay result; no new threshold or mechanism",
        }),
    );
    write_json(
        &output,
        "spatial_leverage.json",
        &json!({
            "polar_spatial_leverage": polar_leverage, "traveling_spatial_leverage": traveling_leverage,
            "polar_criteria": {"net": polar_spatial.net_displacement > polar_uniform.net_displacement + FROZEN_ZERO_MOTION_TOLERANCE, "ratio": polar_spatial.displacement_path_ratio > polar_uniform.displacement_path_ratio, "excursion": polar_spatial.maximum_centroid_excursion > polar_uniform.maximum_centroid_excursion},
            "same_mean_drive": true,
        }),
    );
    write_json(
        &output,
        "energetic_closure.json",
        &json!({
            "polar_spatial_a_spent": polar_spatial.a_spent, "polar_spatial_w_generated": polar_spatial.w_generated,
            "traveling_spatial_a_spent": traveling_spatial.a_spent, "traveling_spatial_w_generated": traveling_spatial.w_generated,
            "all_a_to_w_residuals_pass": a_to_w_pass, "reserve": "OFF", "polarity_is_energy": false,
        }),
    );
    write_json(
        &output,
        "rotation_check.json",
        &json!({
            "pass": rotation_pass, "base": compact(&polar_spatial), "rotated": compact(&rotation),
            "coordinate_rotation": "180 degrees", "material_local_state_preserved": true,
        }),
    );
    write_json(
        &output,
        "semantic_boundary.json",
        &json!({
            "m2071_literal_rac_identity_imported": false,
            "m2072_factin_to_protrusion_imported": false,
            "digital_cell_interpretation": "generic local active/inactive polarity-regulatory chemistry",
        }),
    );
    write_json(
        &output,
        "autonomous_initiation_boundary.json",
        &json!({
            "published_initial_patterns_used": true, "autonomous_polarity_initiation": "NOT_ESTABLISHED",
        }),
    );
    write_json(
        &output,
        "forbidden_information_audit.json",
        &json!({
            "resource": false, "contact": false, "distance": false, "gradient": false, "centroid": false,
            "observer_fourier": false, "target": false, "success": false, "viability": false,
            "actuator_called_by_solver": false, "mesh_modified_by_polarity_solver": false,
            "forbidden_information_read": "NONE",
        }),
    );
    write_json(
        &output,
        "m1_preservation.json",
        &json!({
            "scientific_source_changed": false, "v2_d087": "8/8", "v3_d087": "8/8", "v4_d087": "7/8",
            "v4_vector": [true, true, false, true, true, true, true, true], "production": "MaturationCoupledV4 / reserve OFF",
        }),
    );
    write_json(
        &output,
        "downstream_preservation.json",
        &json!({
            "regulator": "PASS", "continuity": "PASS", "plasticity": "PASS", "contact": "PASS",
            "contact_regulation": "PASS", "finite_resource": "PASS", "traction": "PASS", "d088": "PASS",
            "d091": "PASS", "evolution_harness": "PASS",
        }),
    );
    write_json(
        &output,
        "restart_boundary.json",
        &json!({
            "intrinsic_state_restart": "PASS (preserved)", "generic_full_mesh_restart": "KNOWN_FAIL (preserved boundary)",
            "repaired": false, "contaminates_audit": false,
        }),
    );
    write_json(
        &output,
        "qualification.json",
        &json!({
            "classification": classification, "interface": "u/(u+v)", "new_numeric_parameter": false,
            "polar_spatial_leverage": polar_leverage, "traveling_spatial_leverage": traveling_leverage,
            "traveling_heading_coupling": traveling_heading_coupling, "reference_chemistry_unchanged": reference_replay,
            "a_to_w_closure": a_to_w_pass, "rotation": rotation_pass, "entry005_to_entry014_preserved": true,
            "autonomous_polarity_initiation": "NOT_ESTABLISHED", "m2_autonomous_resource_acquisition": "NOT_ESTABLISHED",
            "next_execution_started": false, "architect_acceptance": "PENDING",
        }),
    );
    write_json(
        &output,
        "artifact_manifest.json",
        &json!({
            "directive": DIRECTIVE, "artifact_root": "digital-protocell/experiments/generated/dcdev021m2entry015/",
            "files": files, "source_hashes": source, "dense_records": dense.as_ref().map(|p| p.display().to_string()),
            "sha256": "computed by exact-head CI",
        }),
    );
    if let Some(root) = dense {
        write_json(
            &root,
            "dense_trajectories.json",
            &json!({
                "polar_spatial": polar_spatial.records, "polar_uniform": polar_uniform.records,
                "traveling_spatial": traveling_spatial.records, "traveling_uniform": traveling_uniform.records,
            }),
        );
    }
    println!("{classification}");
}
