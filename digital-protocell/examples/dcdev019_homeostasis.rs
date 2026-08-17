//! DC-DEV-019 Phase 2/3: one material homeostat and M0–M5 qualification.
//!
//! This is a compact, deterministic assay. It reuses the exact DC-DEV-016
//! settled/deprived body, the existing finite resource boundary, and the
//! existing N/F -> A reaction path. No behavior or exploration is executed.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{reactions_step, reactions_step_counterfactual, ReactionParams};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{
    stable_json_hash, FiniteSpatialResourceRegionV1, MetabolicAcquisitionHomeostatV1,
    HOMEOSTAT_SOURCE_GAIN_MAX, HOMEOSTAT_TAU,
};
use serde::Serialize;
use serde_json::json;
use std::{fs, path::PathBuf};

const ENTRY: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const SETTLE: usize = 5_000;
const WINDOW: usize = 480;
const M3_STEPS: usize = 4_000;
const CENTER: [f64; 2] = [4.8, 0.0];
const RADIUS: f64 = 1.5;
const M_SELECTED: f64 = 19.878372106390554;
const M3_NF: f64 = 0.1476710565778127;
const E_TARGET: f64 = 77.91027880846893;
const E_DEPRIVED: f64 = 60.82781514212436;
const DT: f64 = 0.02;

#[derive(Debug, Clone, Serialize)]
struct Observation {
    step: usize,
    e_stored: f64,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    h: f64,
    error: f64,
    alive: bool,
}

#[derive(Debug, Clone, Serialize)]
struct WindowSummary {
    steps: usize,
    initial_e_stored: f64,
    final_e_stored: f64,
    min_e_stored: f64,
    max_e_stored: f64,
    e_slope: f64,
    quarter_e_slopes: [f64; 4],
    q4_mean_h: f64,
    initial_h: f64,
    final_h: f64,
    max_h: f64,
    alive: bool,
    n_world_loss: f64,
    f_world_loss: f64,
    conservation_error: f64,
    trajectory_hash: String,
}

#[derive(Debug, Clone, Serialize)]
struct Qualification {
    entry: String,
    schema: &'static str,
    e_target: f64,
    e_deprived: f64,
    e0: f64,
    tau: f64,
    k_h: f64,
    g_source_max: f64,
    g_transport_max: f64,
    m_selected: f64,
    m0_feature_off_exact: bool,
    m1_starvation: WindowSummary,
    m1_h_increased: bool,
    m1_source_zero: bool,
    m1_no_material_created: bool,
    m2_finite_feeding: WindowSummary,
    m2_final_above_initial: bool,
    m2_distance_decreased: bool,
    m2_a_or_r_toward_replete: bool,
    m3_sustained: Option<WindowSummary>,
    m3_q4_abs_slope: Option<f64>,
    m3_q4_depletion_reference_slope: Option<f64>,
    m3_pass: bool,
    m4_controller_unwinding: Option<WindowSummary>,
    m5_three_cycle: Option<Vec<WindowSummary>>,
    m5_pass: bool,
    classification: &'static str,
    next_execution_started: bool,
}

fn seed() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        24, 5.0, 0.0, 0.0, DEFAULT_RHO_S, 0.7,
        LumpedChem { c: 0.8, a: 0.5, n: 0.0, f: 0.0, r: 0.6, ..Default::default() },
        LumpedChem::default(), 5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh
}

fn reaction_params(mesh: &MaterialMesh) -> ReactionParams {
    let mut p = ReactionParams::default();
    p.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    p
}

fn e_stored(mesh: &MaterialMesh) -> f64 {
    mesh.area() * (mesh.interior.a + mesh.interior.r).max(0.0)
}

fn observation(mesh: &MaterialMesh, h: &MetabolicAcquisitionHomeostatV1, step: usize) -> Observation {
    Observation {
        step,
        e_stored: e_stored(mesh),
        a: mesh.interior.a,
        r: mesh.interior.r,
        n: mesh.interior.n,
        f: mesh.interior.f,
        h: h.h,
        error: h.error(mesh.area(), mesh.interior.a, mesh.interior.r),
        alive: mesh.alive,
    }
}

fn slope(values: &[f64]) -> f64 {
    match (values.first(), values.last()) {
        (Some(first), Some(last)) if values.len() > 1 => (last - first) / (values.len() - 1) as f64,
        _ => 0.0,
    }
}

fn summarize(samples: &[Observation], n_loss: f64, f_loss: f64, conservation_error: f64) -> WindowSummary {
    let values: Vec<f64> = samples.iter().map(|s| s.e_stored).collect();
    let hs: Vec<f64> = samples.iter().map(|s| s.h).collect();
    let hashes: Vec<String> = samples.iter().map(|s| stable_json_hash(s).unwrap()).collect();
    let quarter = values.len() / 4;
    let quarter_e_slopes = [
        slope(&values[..quarter]),
        slope(&values[quarter..quarter * 2]),
        slope(&values[quarter * 2..quarter * 3]),
        slope(&values[quarter * 3..]),
    ];
    let q4_start = values.len() * 3 / 4;
    let q4_mean_h = if hs.len() > q4_start {
        hs[q4_start..].iter().sum::<f64>() / (hs.len() - q4_start) as f64
    } else {
        0.0
    };
    WindowSummary {
        steps: samples.len(),
        initial_e_stored: values.first().copied().unwrap_or(0.0),
        final_e_stored: values.last().copied().unwrap_or(0.0),
        min_e_stored: values.iter().copied().fold(f64::INFINITY, f64::min),
        max_e_stored: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        e_slope: slope(&values),
        quarter_e_slopes,
        q4_mean_h,
        initial_h: hs.first().copied().unwrap_or(0.0),
        final_h: hs.last().copied().unwrap_or(0.0),
        max_h: hs.iter().copied().fold(0.0, f64::max),
        alive: samples.iter().all(|s| s.alive),
        n_world_loss: n_loss,
        f_world_loss: f_loss,
        conservation_error,
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
    }
}

fn settle(mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = seed();
    for _ in 0..SETTLE {
        assert!(mechanics_step(&mut mesh, mechanics));
    }
    mesh
}

fn deprive(settled: &MaterialMesh, mechanics: &MechParams) -> MaterialMesh {
    let mut mesh = settled.clone();
    let params = reaction_params(&mesh);
    for _ in 0..WINDOW {
        reactions_step(&mut mesh, &params, mechanics.dt, true, true);
    }
    mesh
}

fn homeostat(enabled: bool) -> MetabolicAcquisitionHomeostatV1 {
    let e0 = (E_TARGET - E_DEPRIVED) / E_TARGET;
    MetabolicAcquisitionHomeostatV1::try_new(
        enabled, E_TARGET, e0, HOMEOSTAT_TAU, HOMEOSTAT_SOURCE_GAIN_MAX, 1.0,
    )
    .unwrap()
}

fn step(
    mesh: &mut MaterialMesh,
    params: &ReactionParams,
    h: &mut MetabolicAcquisitionHomeostatV1,
    mechanics: &MechParams,
    patch: Option<&mut FiniteSpatialResourceRegionV1>,
) -> (f64, f64, f64) {
    let gains = h.advance(mesh.area(), mesh.interior.a, mesh.interior.r, mechanics.dt);
    let (mut n_loss, mut f_loss, mut conservation_error) = (0.0, 0.0, 0.0);
    if let Some(resource) = patch {
        let ledger = resource.uptake_with_capacity_multiplier(
            mesh, &TransportParams::default(), mechanics.dt, gains.g_transport,
        );
        n_loss = ledger.n_world_loss;
        f_loss = ledger.f_world_loss;
        conservation_error = ledger.conservation_error;
    }
    reactions_step_counterfactual(
        mesh, params, mechanics.dt, true, true, gains.g_source,
    );
    (n_loss, f_loss, conservation_error)
}

fn run_window(
    mesh: &mut MaterialMesh,
    h: &mut MetabolicAcquisitionHomeostatV1,
    mechanics: &MechParams,
    steps: usize,
    patch_mass: Option<f64>,
    fresh_patch_each_step: bool,
    start_step: usize,
) -> WindowSummary {
    let params = reaction_params(mesh);
    let mut persistent_patch = patch_mass.map(|m| FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, m, m));
    let mut samples = Vec::with_capacity(steps);
    let (mut n_loss, mut f_loss, mut conservation_error) = (0.0, 0.0, 0.0);
    for i in 0..steps {
        let mut fresh = patch_mass.filter(|_| fresh_patch_each_step)
            .map(|m| FiniteSpatialResourceRegionV1::new(CENTER, RADIUS, m, m));
        let patch = if fresh_patch_each_step { fresh.as_mut() } else { persistent_patch.as_mut() };
        let (n, f, err) = step(mesh, &params, h, mechanics, patch);
        n_loss += n;
        f_loss += f;
        conservation_error += err;
        samples.push(observation(mesh, h, start_step + i + 1));
    }
    summarize(&samples, n_loss, f_loss, conservation_error)
}

fn run_sustained_clamp(
    mesh: &mut MaterialMesh,
    h: &mut MetabolicAcquisitionHomeostatV1,
    mechanics: &MechParams,
    steps: usize,
    start_step: usize,
) -> WindowSummary {
    let params = reaction_params(mesh);
    let mut samples = Vec::with_capacity(steps);
    for i in 0..steps {
        mesh.interior.n = M3_NF;
        mesh.interior.f = M3_NF;
        step(mesh, &params, h, mechanics, None);
        samples.push(observation(mesh, h, start_step + i + 1));
    }
    summarize(&samples, 0.0, 0.0, 0.0)
}

fn main() {
    let output = std::env::var_os("DCDEV019_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev019/phase3"));
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() < 1e-12);
    let settled = settle(&mechanics);
    let deprived = deprive(&settled, &mechanics);
    let e0 = (E_TARGET - E_DEPRIVED) / E_TARGET;

    let mut legacy_a = deprived.clone();
    let mut parity_a = deprived.clone();
    let params = reaction_params(&deprived);
    for _ in 0..WINDOW {
        reactions_step(&mut legacy_a, &params, mechanics.dt, true, true);
        reactions_step_counterfactual(&mut parity_a, &params, mechanics.dt, true, true, 1.0);
    }
    let mut feature_off = deprived.clone();
    let mut feature_off_h = homeostat(false);
    for _ in 0..WINDOW {
        step(&mut feature_off, &params, &mut feature_off_h, &mechanics, None);
    }
    let m0_feature_off_exact = stable_json_hash(&legacy_a).unwrap()
        == stable_json_hash(&parity_a).unwrap()
        && stable_json_hash(&legacy_a).unwrap() == stable_json_hash(&feature_off).unwrap();

    let mut m1_mesh = deprived.clone();
    let mut m1_h = homeostat(true);
    let m1 = run_window(&mut m1_mesh, &mut m1_h, &mechanics, WINDOW, None, false, 0);
    let m1_h_increased = m1.final_h > m1.initial_h;
    let m1_source_zero = m1_mesh.interior.n.abs() <= 1e-12 && m1_mesh.interior.f.abs() <= 1e-12;
    let m1_no_material_created = m1.final_e_stored <= m1.initial_e_stored + 1e-10;

    let mut m2_mesh = deprived.clone();
    let mut m2_h = homeostat(true);
    let m2 = run_window(&mut m2_mesh, &mut m2_h, &mechanics, WINDOW, Some(M_SELECTED), false, 0);
    let m2_final_above_initial = m2.final_e_stored > m2.initial_e_stored + 1e-10;
    let m2_distance_decreased = (E_TARGET - m2.final_e_stored).abs() < (E_TARGET - E_DEPRIVED).abs();
    let replete = &settled;
    let a_toward = (replete.interior.a - m2_mesh.interior.a).abs() < (replete.interior.a - deprived.interior.a).abs();
    let r_toward = (replete.interior.r - m2_mesh.interior.r).abs() < (replete.interior.r - deprived.interior.r).abs();

    let mut m3_mesh = deprived.clone();
    let mut m3_h = homeostat(true);
    let m3 = run_sustained_clamp(&mut m3_mesh, &mut m3_h, &mechanics, M3_STEPS, 0);
    let m3_q4_abs_slope = m3.quarter_e_slopes[3].abs();
    let m3_q4_depletion_reference_slope = {
        let mut baseline_mesh = deprived.clone();
        let mut baseline_h = homeostat(false);
        let baseline = run_window(&mut baseline_mesh, &mut baseline_h, &mechanics, M3_STEPS, None, false, 0);
        Some(baseline.quarter_e_slopes[3].abs())
    };
    let m3_pass = m3.final_e_stored >= 0.95 * E_TARGET
        && m3.final_e_stored <= 1.05 * E_TARGET
        && m3_q4_abs_slope <= 0.01 * m3_q4_depletion_reference_slope.unwrap_or(f64::INFINITY)
        && m3.q4_mean_h < 0.95
        && m3.e_slope >= -1e-10
        && m3.max_e_stored <= 1.10 * E_TARGET
        && m3.alive
        && m3.conservation_error <= 1e-10;

    let (m4, m5, m5_pass) = if m3_pass {
        let mut m4_mesh = m3_mesh.clone();
        let mut m4_h = m3_h.clone();
        let before_h = m4_h.h;
        let starved = run_window(&mut m4_mesh, &mut m4_h, &mechanics, WINDOW, None, false, M3_STEPS);
        let fed = run_window(&mut m4_mesh, &mut m4_h, &mechanics, WINDOW, Some(M_SELECTED), true, M3_STEPS + WINDOW);
        let m4 = WindowSummary {
            steps: starved.steps + fed.steps,
            initial_e_stored: starved.initial_e_stored,
            final_e_stored: fed.final_e_stored,
            min_e_stored: starved.min_e_stored.min(fed.min_e_stored),
            max_e_stored: starved.max_e_stored.max(fed.max_e_stored),
            e_slope: fed.e_slope,
            quarter_e_slopes: fed.quarter_e_slopes,
            q4_mean_h: fed.q4_mean_h,
            initial_h: before_h,
            final_h: fed.final_h,
            max_h: starved.max_h.max(fed.max_h),
            alive: starved.alive && fed.alive,
            n_world_loss: fed.n_world_loss,
            f_world_loss: fed.f_world_loss,
            conservation_error: starved.conservation_error + fed.conservation_error,
            trajectory_hash: stable_json_hash(&(starved.trajectory_hash, fed.trajectory_hash)).unwrap(),
        };
        let mut cycle_mesh = m3_mesh;
        let mut cycle_h = m3_h;
        let mut cycles = Vec::new();
        for cycle in 0..3 {
            let starve = run_window(&mut cycle_mesh, &mut cycle_h, &mechanics, WINDOW, None, false, cycle * 2 * WINDOW);
            let feed = run_window(&mut cycle_mesh, &mut cycle_h, &mechanics, WINDOW, Some(M_SELECTED), true, cycle * 2 * WINDOW + WINDOW);
            cycles.push(WindowSummary {
                steps: WINDOW * 2,
                initial_e_stored: starve.initial_e_stored,
                final_e_stored: feed.final_e_stored,
                min_e_stored: starve.min_e_stored.min(feed.min_e_stored),
                max_e_stored: starve.max_e_stored.max(feed.max_e_stored),
                e_slope: feed.e_slope,
                quarter_e_slopes: feed.quarter_e_slopes,
                q4_mean_h: feed.q4_mean_h,
                initial_h: starve.initial_h,
                final_h: feed.final_h,
                max_h: starve.max_h.max(feed.max_h),
                alive: starve.alive && feed.alive,
                n_world_loss: feed.n_world_loss,
                f_world_loss: feed.f_world_loss,
                conservation_error: starve.conservation_error + feed.conservation_error,
                trajectory_hash: stable_json_hash(&(starve.trajectory_hash, feed.trajectory_hash)).unwrap(),
            });
        }
        let first = cycles[0].final_e_stored;
        let third = cycles[2].final_e_stored;
        let m5_pass = cycles.iter().all(|c| c.alive && c.conservation_error <= 1e-10)
            && third >= 0.90 * first;
        (Some(m4), Some(cycles), m5_pass)
    } else {
        (None, None, false)
    };

    let classification = if !m3_pass {
        "DCDEV019_COORDINATED_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED"
    } else if !m5_pass {
        "DCDEV019_COORDINATED_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED"
    } else {
        "DCDEV019_COORDINATED_METABOLIC_HOMEOSTASIS_ONLY_QUALIFIED"
    };
    let report = Qualification {
        entry: ENTRY.to_string(),
        schema: regulatory_core::METABOLIC_ACQUISITION_HOMEOSTAT_SCHEMA_V1,
        e_target: E_TARGET,
        e_deprived: E_DEPRIVED,
        e0,
        tau: HOMEOSTAT_TAU,
        k_h: 2.0 / (e0 * HOMEOSTAT_TAU),
        g_source_max: HOMEOSTAT_SOURCE_GAIN_MAX,
        g_transport_max: 1.0,
        m_selected: M_SELECTED,
        m0_feature_off_exact,
        m1_starvation: m1,
        m1_h_increased,
        m1_source_zero,
        m1_no_material_created,
        m2_finite_feeding: m2,
        m2_final_above_initial,
        m2_distance_decreased,
        m2_a_or_r_toward_replete: a_toward || r_toward,
        m3_sustained: Some(m3),
        m3_q4_abs_slope: Some(m3_q4_abs_slope),
        m3_q4_depletion_reference_slope,
        m3_pass,
        m4_controller_unwinding: m4,
        m5_three_cycle: m5,
        m5_pass,
        classification,
        next_execution_started: false,
    };
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("homeostasis_qualification.json"), serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    fs::write(output.join("protocol.json"), serde_json::to_vec_pretty(&json!({
        "entry": ENTRY, "settlement_steps": SETTLE, "window_steps": WINDOW,
        "m3_steps": M3_STEPS, "center": CENTER, "radius": RADIUS,
        "m_selected": M_SELECTED, "m3_nf": M3_NF, "e_target": E_TARGET,
        "e_deprived": E_DEPRIVED, "next_execution_started": false,
    })).unwrap()).unwrap();
    println!("DCDEV019_FINITE_NUTRIENT_HOMEOSTASIS_PHASE_3_COMPLETE");
    println!("{classification}");
    println!("M0_feature_off_exact={m0_feature_off_exact}");
    println!("M3_pass={m3_pass}");
    println!("M5_pass={m5_pass}");
    println!("M3_final_E_stored={}", report.m3_sustained.as_ref().unwrap().final_e_stored);
    println!("NEXT_EXECUTION_STARTED:false");
}
