//! DC-DEV-018-R1 observer-only closed-loop source-demand feasibility audit.
//!
//! This assay starts from clean DC-DEV-016 and uses the canonical reaction
//! sequence on cloned meshes. The only counterfactual is a multiplier on the
//! existing N/F -> A extent; no controller or persistent state is introduced.

use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, DEFAULT_RHO_S};
use chemistry_core::mesh_mechanics::{mechanics_step, MechParams};
use chemistry_core::mesh_reactions::{
    reactions_step, reactions_step_counterfactual, ReactionLedger, ReactionParams,
};
use chemistry_core::mesh_transport::TransportParams;
use chemistry_core::metabolic_reserve::{stamp_reserve_equation, ReserveParams};
use regulatory_core::{stable_json_hash, FiniteSpatialResourceRegionV1};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

const ENTRY: &str = "1e242f28152797b512e25cd56c7b718e45d6ca97";
const SETTLE_STEPS: usize = 5_000;
const DEPRIVATION_STEPS: usize = 480;
const STORAGE_STEPS: usize = 4_000;
const FINITE_STEPS: usize = 480;
const DT: f64 = 0.02;
const PRECURSOR_CLAMP: f64 = 0.1476710565778127;
const FINITE_INVENTORY: f64 = 14.588954880632265;
const EPS: f64 = 1.0e-10;
const BISECTION_TOLERANCE: f64 = 1.0e-10;
const MAX_BISECTION: usize = 64;
const RESPONSE_FRACTIONS: [f64; 5] = [0.0, 0.25, 0.50, 0.75, 1.0];

#[derive(Debug, Clone, Copy, Serialize)]
struct Snapshot {
    step: usize,
    time: f64,
    area: f64,
    a: f64,
    r: f64,
    n: f64,
    f: f64,
    e_stored: f64,
    alive: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
struct SinkBreakdown {
    source: f64,
    a_decay: f64,
    a_to_structural: f64,
    a_to_catalyst: f64,
    a_to_membrane: f64,
    a_to_r: f64,
    r_to_a: f64,
    r_to_waste: f64,
    r_to_structural: f64,
    irreversible_demand: f64,
}

#[derive(Debug, Clone)]
struct Counterfactual {
    gain: f64,
    g_sat: f64,
    source_clamped: bool,
    delta_e_stored: f64,
    before: Snapshot,
    after: Snapshot,
    sinks: SinkBreakdown,
    ledger: ReactionLedger,
    after_mesh: MaterialMesh,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Stats {
    minimum: f64,
    p05: f64,
    median: f64,
    p95: f64,
    maximum: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ResponseWindow {
    window: String,
    g_sat: Stats,
    source_by_u: Vec<Stats>,
    demand_by_u: Vec<Stats>,
    delta_e_stored_by_u: Vec<Stats>,
    source_saturated_fraction: f64,
    max_source_net_negative_fraction: f64,
    non_monotonic_fraction: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ResponseEnvelope {
    fractions: Vec<f64>,
    windows: Vec<ResponseWindow>,
    records: Vec<ResponseRecord>,
    total_states: usize,
    trajectory_hash: String,
}

#[derive(Debug, Clone, Serialize)]
struct ResponsePoint {
    gain: f64,
    source: f64,
    irreversible_demand: f64,
    delta_e_stored: f64,
    source_clamped: bool,
    sinks: SinkBreakdown,
}

#[derive(Debug, Clone, Serialize)]
struct ResponseRecord {
    step: usize,
    g_sat: f64,
    points: Vec<ResponsePoint>,
}

#[derive(Debug, Clone, Serialize)]
struct IdealSummary {
    first_infeasible_step: Option<usize>,
    final_state: Snapshot,
    target_e_stored: f64,
    final_e_ref: f64,
    max_required_gain: f64,
    median_required_gain: f64,
    max_required_over_g_sat: f64,
    source_total: f64,
    irreversible_demand_total: f64,
    sink_totals: SinkBreakdown,
    records: Vec<IdealRecord>,
    saturated_upper_bound_final: Snapshot,
    alive: bool,
    finite_nonnegative: bool,
}

#[derive(Debug, Clone, Serialize)]
struct IdealRecord {
    step: usize,
    gain: f64,
    g_sat: f64,
    required_over_g_sat: f64,
    e_ref: f64,
    e_stored: f64,
    source: f64,
    irreversible_demand: f64,
    sinks: SinkBreakdown,
    source_clamped: bool,
    feasible: bool,
    alive: bool,
}

#[derive(Debug, Clone, Serialize)]
struct FiniteSummary {
    steps: usize,
    initial: Snapshot,
    final_state: Snapshot,
    delivered_n: f64,
    delivered_f: f64,
    consumed_n: f64,
    consumed_f: f64,
    source_total: f64,
    irreversible_demand_total: f64,
    sink_totals: SinkBreakdown,
    records: Vec<FiniteRecord>,
    initial_resource_n: f64,
    final_resource_n: f64,
    initial_resource_f: f64,
    final_resource_f: f64,
    max_resource_error: f64,
    resource_conservation: bool,
    alive: bool,
    finite_nonnegative: bool,
}

#[derive(Debug, Clone, Serialize)]
struct FiniteRecord {
    step: usize,
    delivered_n: f64,
    delivered_f: f64,
    internal_n: f64,
    internal_f: f64,
    gain: f64,
    g_sat: f64,
    source: f64,
    irreversible_demand: f64,
    sinks: SinkBreakdown,
    resource_n: f64,
    resource_f: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ControllerReachability {
    source: String,
    integrated_error: f64,
    k_integral: f64,
    capacity_max: f64,
    predicted_capacity: f64,
    observed_capacity: f64,
    fraction_of_required_capacity_reached: f64,
    could_reach_cap_in_horizon: bool,
    trace_provenance_complete: bool,
    finding: String,
}

#[derive(Debug, Clone, Serialize)]
struct Audit {
    directive: String,
    entry_commit: String,
    settled_body_hash: String,
    deprived_body_hash: String,
    settled_snapshot: Snapshot,
    deprived_snapshot: Snapshot,
    target_e_stored: f64,
    deprived_e_stored: f64,
    baseline_trajectory_hash: String,
    counterfactual_parity: bool,
    response_envelope: ResponseEnvelope,
    ideal: IdealSummary,
    finite_resource: FiniteSummary,
    controller_reachability: ControllerReachability,
    classification: String,
    production_behavior_changed: bool,
    certified_phase1_equations_unchanged: bool,
    downstream_started: bool,
    next_execution_started: bool,
}

fn write(root: &Path, name: &str, value: &Value) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn snapshot(mesh: &MaterialMesh, step: usize) -> Snapshot {
    Snapshot {
        step,
        time: step as f64 * DT,
        area: mesh.area(),
        a: mesh.interior.a,
        r: mesh.interior.r,
        n: mesh.interior.n,
        f: mesh.interior.f,
        e_stored: mesh.area() * (mesh.interior.a + mesh.interior.r),
        alive: mesh.alive,
    }
}

fn seed() -> MaterialMesh {
    let mut mesh = MaterialMesh::seed_regular(
        24,
        5.0,
        0.0,
        0.0,
        DEFAULT_RHO_S,
        0.7,
        LumpedChem {
            c: 0.8,
            a: 0.5,
            r: 0.6,
            ..Default::default()
        },
        LumpedChem::default(),
        5.0,
    );
    stamp_reserve_equation(&mut mesh);
    mesh
}

fn params(mesh: &MaterialMesh) -> ReactionParams {
    let mut p = ReactionParams::default();
    p.reserve = ReserveParams::derived(80.0, 40.0, 0.5, 0.3, 2.0, 0.1, mesh.area());
    p
}

fn settle() -> MaterialMesh {
    let mut mesh = seed();
    let mechanics = MechParams::default();
    assert!((mechanics.dt - DT).abs() <= EPS);
    for _ in 0..SETTLE_STEPS {
        assert!(mechanics_step(&mut mesh, &mechanics));
    }
    mesh
}

fn deprive(settled: &MaterialMesh) -> MaterialMesh {
    let mut mesh = settled.clone();
    let p = params(&mesh);
    for _ in 0..DEPRIVATION_STEPS {
        reactions_step(&mut mesh, &p, DT, true, true);
    }
    mesh
}

fn inferred_decay(
    before: LumpedChem,
    after: LumpedChem,
    ledger: &ReactionLedger,
    area: f64,
) -> f64 {
    let value = area * (before.a - after.a) + ledger.a_produced + ledger.reserve.r_to_a
        - ledger.c_produced
        - ledger.a_consumed_build
        - ledger.l_produced
        - ledger.reserve.a_to_r;
    assert!(value >= -1.0e-8, "negative inferred A decay {value}");
    value.max(0.0)
}

fn irreversible(
    before: LumpedChem,
    after: LumpedChem,
    ledger: &ReactionLedger,
    area: f64,
) -> SinkBreakdown {
    let a_decay = inferred_decay(before, after, ledger, area);
    let sink = SinkBreakdown {
        source: ledger.a_produced,
        a_decay,
        a_to_structural: ledger.a_consumed_build,
        a_to_catalyst: ledger.c_produced,
        a_to_membrane: ledger.l_produced,
        a_to_r: ledger.reserve.a_to_r,
        r_to_a: ledger.reserve.r_to_a,
        r_to_waste: ledger.reserve.r_to_w,
        r_to_structural: ledger.reserve.r_to_m,
        irreversible_demand: a_decay
            + ledger.a_consumed_build
            + ledger.c_produced
            + ledger.l_produced
            + ledger.reserve.r_to_w
            + ledger.reserve.r_to_m,
    };
    assert!(sink.irreversible_demand.is_finite());
    sink
}

fn source_saturation_gain(mesh: &MaterialMesh, gain_one_source: f64) -> f64 {
    if gain_one_source <= EPS {
        f64::INFINITY
    } else {
        (mesh.interior.n.max(0.0) * mesh.area()).min(mesh.interior.f.max(0.0) * mesh.area())
            / gain_one_source
    }
}

fn counterfactual(
    mesh: &MaterialMesh,
    p: &ReactionParams,
    gain: f64,
    step: usize,
) -> Counterfactual {
    let mut after_mesh = mesh.clone();
    let before = snapshot(mesh, step);
    let before_chem = mesh.interior;
    let area = mesh.area().max(1.0e-15);
    let ledger = reactions_step_counterfactual(&mut after_mesh, p, DT, true, true, gain);
    let after = snapshot(&after_mesh, step + 1);
    let sinks = irreversible(before_chem, after_mesh.interior, &ledger, area);
    let capacity = (mesh.interior.n.max(0.0) * area).min(mesh.interior.f.max(0.0) * area);
    Counterfactual {
        gain,
        g_sat: source_saturation_gain(mesh, sinks.source),
        source_clamped: (ledger.n_consumed - capacity).abs() <= EPS
            || (ledger.f_consumed - capacity).abs() <= EPS,
        delta_e_stored: after.e_stored - before.e_stored,
        before,
        after,
        sinks,
        ledger,
        after_mesh,
    }
}

fn quantile(mut values: Vec<f64>, q: f64) -> f64 {
    values.retain(|v| v.is_finite());
    if values.is_empty() {
        return f64::NAN;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let pos = q.clamp(0.0, 1.0) * (values.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    values[lo] + (values[hi] - values[lo]) * (pos - lo as f64)
}

fn stats(values: &[f64]) -> Stats {
    Stats {
        minimum: quantile(values.to_vec(), 0.0),
        p05: quantile(values.to_vec(), 0.05),
        median: quantile(values.to_vec(), 0.5),
        p95: quantile(values.to_vec(), 0.95),
        maximum: quantile(values.to_vec(), 1.0),
    }
}

fn legacy_trajectory(deprived: &MaterialMesh, p: &ReactionParams) -> (String, MaterialMesh) {
    let mut mesh = deprived.clone();
    let mut hashes = vec![stable_json_hash(&snapshot(&mesh, 0)).unwrap()];
    for step in 0..STORAGE_STEPS {
        mesh.interior.n = PRECURSOR_CLAMP;
        mesh.interior.f = PRECURSOR_CLAMP;
        mesh = counterfactual(&mesh, p, 1.0, step).after_mesh;
        hashes.push(stable_json_hash(&snapshot(&mesh, step + 1)).unwrap());
    }
    (stable_json_hash(&hashes).unwrap(), mesh)
}

fn response_envelope(deprived: &MaterialMesh, p: &ReactionParams) -> ResponseEnvelope {
    let mut mesh = deprived.clone();
    let mut raw_g = vec![Vec::<f64>::new(); 4];
    let mut raw_source = vec![vec![Vec::<f64>::new(); 5]; 4];
    let mut raw_demand = vec![vec![Vec::<f64>::new(); 5]; 4];
    let mut raw_delta = vec![vec![Vec::<f64>::new(); 5]; 4];
    let mut saturated = [0usize; 4];
    let mut max_negative = [0usize; 4];
    let mut non_monotonic = [0usize; 4];
    let mut records = Vec::with_capacity(STORAGE_STEPS);
    let mut hashes = vec![stable_json_hash(&snapshot(&mesh, 0)).unwrap()];
    for step in 0..STORAGE_STEPS {
        mesh.interior.n = PRECURSOR_CLAMP;
        mesh.interior.f = PRECURSOR_CLAMP;
        let base = counterfactual(&mesh, p, 1.0, step);
        let gs = base.g_sat;
        let wi = step / 1000;
        raw_g[wi].push(gs);
        let mut deltas = Vec::with_capacity(5);
        let mut points = Vec::with_capacity(5);
        for (ui, u) in RESPONSE_FRACTIONS.iter().enumerate() {
            let gain = if gs.is_finite() && gs > 1.0 {
                1.0 + u * (gs - 1.0)
            } else {
                1.0
            };
            let point = counterfactual(&mesh, p, gain, step);
            raw_source[wi][ui].push(point.sinks.source);
            raw_demand[wi][ui].push(point.sinks.irreversible_demand);
            raw_delta[wi][ui].push(point.delta_e_stored);
            deltas.push(point.delta_e_stored);
            points.push(ResponsePoint {
                gain,
                source: point.sinks.source,
                irreversible_demand: point.sinks.irreversible_demand,
                delta_e_stored: point.delta_e_stored,
                source_clamped: point.source_clamped,
                sinks: point.sinks,
            });
            if ui == 4 && point.source_clamped {
                saturated[wi] += 1;
            }
        }
        if deltas.windows(2).any(|w| w[1] + EPS < w[0]) {
            non_monotonic[wi] += 1;
        }
        if deltas.last().copied().unwrap_or(0.0) < -EPS {
            max_negative[wi] += 1;
        }
        records.push(ResponseRecord {
            step,
            g_sat: gs,
            points,
        });
        mesh = base.after_mesh;
        hashes.push(stable_json_hash(&snapshot(&mesh, step + 1)).unwrap());
    }
    let windows = (0..4)
        .map(|wi| ResponseWindow {
            window: format!("Q{}", wi + 1),
            g_sat: stats(&raw_g[wi]),
            source_by_u: raw_source[wi].iter().map(|v| stats(v)).collect(),
            demand_by_u: raw_demand[wi].iter().map(|v| stats(v)).collect(),
            delta_e_stored_by_u: raw_delta[wi].iter().map(|v| stats(v)).collect(),
            source_saturated_fraction: saturated[wi] as f64 / 1000.0,
            max_source_net_negative_fraction: max_negative[wi] as f64 / 1000.0,
            non_monotonic_fraction: non_monotonic[wi] as f64 / 1000.0,
        })
        .collect();
    ResponseEnvelope {
        fractions: RESPONSE_FRACTIONS.to_vec(),
        windows,
        records,
        total_states: STORAGE_STEPS,
        trajectory_hash: stable_json_hash(&hashes).unwrap(),
    }
}

fn add_sinks(total: &mut SinkBreakdown, sink: SinkBreakdown) {
    total.source += sink.source;
    total.a_decay += sink.a_decay;
    total.a_to_structural += sink.a_to_structural;
    total.a_to_catalyst += sink.a_to_catalyst;
    total.a_to_membrane += sink.a_to_membrane;
    total.a_to_r += sink.a_to_r;
    total.r_to_a += sink.r_to_a;
    total.r_to_waste += sink.r_to_waste;
    total.r_to_structural += sink.r_to_structural;
    total.irreversible_demand += sink.irreversible_demand;
}

fn ideal_source(deprived: &MaterialMesh, p: &ReactionParams, target: f64) -> IdealSummary {
    let mut mesh = deprived.clone();
    let mut saturated_mesh = deprived.clone();
    let initial = snapshot(&mesh, 0);
    let mut first_infeasible = None;
    let mut required = Vec::new();
    let mut max_ratio: f64 = 0.0;
    let mut max_gain: f64 = 1.0;
    let mut total = SinkBreakdown::default();
    let mut records = Vec::with_capacity(STORAGE_STEPS);
    for step in 0..STORAGE_STEPS {
        mesh.interior.n = PRECURSOR_CLAMP;
        mesh.interior.f = PRECURSOR_CLAMP;
        saturated_mesh.interior.n = PRECURSOR_CLAMP;
        saturated_mesh.interior.f = PRECURSOR_CLAMP;
        let one = counterfactual(&mesh, p, 1.0, step);
        let gs = one.g_sat.max(1.0);
        let high = counterfactual(&mesh, p, gs, step);
        let target_next = initial.e_stored
            + (target - initial.e_stored) * (step + 1) as f64 / STORAGE_STEPS as f64;
        let feasible = high.after.e_stored + EPS >= target_next
            && high.after.e_stored + EPS >= one.after.e_stored;
        let chosen = if !feasible {
            if first_infeasible.is_none() {
                first_infeasible = Some(step);
            }
            high
        } else if target_next <= one.after.e_stored + BISECTION_TOLERANCE {
            one
        } else {
            let mut lo = 1.0;
            let mut hi = gs;
            for _ in 0..MAX_BISECTION {
                let mid = (lo + hi) * 0.5;
                let probe = counterfactual(&mesh, p, mid, step);
                if probe.after.e_stored >= target_next {
                    hi = mid;
                } else {
                    lo = mid;
                }
                if (probe.after.e_stored - target_next).abs() <= BISECTION_TOLERANCE {
                    break;
                }
            }
            counterfactual(&mesh, p, hi, step)
        };
        let ratio = if gs > EPS {
            chosen.gain / gs
        } else {
            f64::INFINITY
        };
        required.push(chosen.gain);
        max_gain = max_gain.max(chosen.gain);
        max_ratio = max_ratio.max(ratio);
        add_sinks(&mut total, chosen.sinks);
        records.push(IdealRecord {
            step,
            gain: chosen.gain,
            g_sat: gs,
            required_over_g_sat: ratio,
            e_ref: target_next,
            e_stored: chosen.after.e_stored,
            source: chosen.sinks.source,
            irreversible_demand: chosen.sinks.irreversible_demand,
            sinks: chosen.sinks,
            source_clamped: chosen.source_clamped,
            feasible,
            alive: chosen.after.alive,
        });
        let sat = counterfactual(&saturated_mesh, p, gs, step);
        saturated_mesh = sat.after_mesh;
        mesh = chosen.after_mesh;
    }
    let final_state = snapshot(&mesh, STORAGE_STEPS);
    IdealSummary {
        first_infeasible_step: first_infeasible,
        final_state,
        target_e_stored: target,
        final_e_ref: target,
        max_required_gain: max_gain,
        median_required_gain: quantile(required, 0.5),
        max_required_over_g_sat: max_ratio,
        source_total: total.source,
        irreversible_demand_total: total.irreversible_demand,
        sink_totals: total,
        records,
        saturated_upper_bound_final: snapshot(&saturated_mesh, STORAGE_STEPS),
        alive: mesh.alive && saturated_mesh.alive,
        finite_nonnegative: [
            mesh.interior.a,
            mesh.interior.r,
            mesh.interior.n,
            mesh.interior.f,
            saturated_mesh.interior.a,
            saturated_mesh.interior.r,
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= -EPS),
    }
}

fn finite_resource(deprived: &MaterialMesh, p: &ReactionParams) -> FiniteSummary {
    let mut mesh = deprived.clone();
    let mut region =
        FiniteSpatialResourceRegionV1::new([0.0, 0.0], 5.0, FINITE_INVENTORY, FINITE_INVENTORY);
    let transport = TransportParams::default();
    let initial = snapshot(&mesh, 0);
    let initial_n = region.n_mass;
    let initial_f = region.f_mass;
    let mut delivered_n = 0.0;
    let mut delivered_f = 0.0;
    let mut consumed_n = 0.0;
    let mut consumed_f = 0.0;
    let mut total = SinkBreakdown::default();
    let mut records = Vec::with_capacity(FINITE_STEPS);
    let mut max_error: f64 = 0.0;
    let mut finite = true;
    for step in 0..FINITE_STEPS {
        let uptake = region.uptake(&mut mesh, &transport, DT);
        delivered_n += uptake.n_delivered;
        delivered_f += uptake.f_delivered;
        max_error = max_error.max(uptake.conservation_error.abs());
        let one = counterfactual(&mesh, p, 1.0, step);
        let gain = one.g_sat.max(1.0);
        let before = mesh.interior;
        let mut after = mesh.clone();
        let ledger = reactions_step_counterfactual(&mut after, p, DT, true, true, gain);
        let sinks = irreversible(before, after.interior, &ledger, after.area().max(1.0e-15));
        consumed_n += ledger.n_consumed;
        consumed_f += ledger.f_consumed;
        add_sinks(&mut total, sinks);
        records.push(FiniteRecord {
            step,
            delivered_n: uptake.n_delivered,
            delivered_f: uptake.f_delivered,
            internal_n: after.interior.n,
            internal_f: after.interior.f,
            gain,
            g_sat: one.g_sat,
            source: sinks.source,
            irreversible_demand: sinks.irreversible_demand,
            sinks,
            resource_n: region.n_mass,
            resource_f: region.f_mass,
        });
        mesh = after;
        finite &= mesh.alive
            && [
                mesh.interior.a,
                mesh.interior.r,
                mesh.interior.n,
                mesh.interior.f,
                mesh.interior.c,
                mesh.interior.w,
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= -EPS);
    }
    let final_state = snapshot(&mesh, FINITE_STEPS);
    let conservation = max_error <= EPS
        && (initial_n - region.n_mass - delivered_n).abs() <= EPS
        && (initial_f - region.f_mass - delivered_f).abs() <= EPS
        && (initial.n * initial.area + delivered_n - consumed_n - final_state.n * final_state.area)
            .abs()
            <= EPS
        && (initial.f * initial.area + delivered_f - consumed_f - final_state.f * final_state.area)
            .abs()
            <= EPS;
    FiniteSummary {
        steps: FINITE_STEPS,
        initial,
        final_state,
        delivered_n,
        delivered_f,
        consumed_n,
        consumed_f,
        source_total: total.source,
        irreversible_demand_total: total.irreversible_demand,
        sink_totals: total,
        records,
        initial_resource_n: initial_n,
        final_resource_n: region.n_mass,
        initial_resource_f: initial_f,
        final_resource_f: region.f_mass,
        max_resource_error: max_error,
        resource_conservation: conservation,
        alive: mesh.alive,
        finite_nonnegative: finite,
    }
}

fn controller_reachability() -> ControllerReachability {
    // The committed DC-DEV-018 artifact contains the observed M4 capacity but
    // not its per-step error trace. Preserve that provenance boundary rather
    // than fabricating an exact trace from rounded quarter means.
    let capacity_max: f64 = 2.368462987851295;
    let k_integral: f64 = 0.029605787348141187;
    let observed_capacity: f64 = 1.07088866298817;
    let integrated_error = observed_capacity / k_integral;
    let predicted = (k_integral * integrated_error).min(capacity_max);
    ControllerReachability {
        source: "DC-DEV-018 committed M4 observed capacity; per-step error trace absent from committed artifacts".into(),
        integrated_error,
        k_integral,
        capacity_max,
        predicted_capacity: predicted,
        observed_capacity,
        fraction_of_required_capacity_reached: observed_capacity / capacity_max,
        could_reach_cap_in_horizon: predicted >= capacity_max - EPS,
        trace_provenance_complete: false,
        finding: "DCDEV018R1_CONTROLLER_REACHABILITY_TRACE_NOT_RECONSTRUCTIBLE_FROM_COMMITTED_EVIDENCE".into(),
    }
}

fn main() {
    let out = std::env::var_os("DCDEV018R1_OUTPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev018r1"));
    let settled = settle();
    let deprived = deprive(&settled);
    let p = params(&deprived);
    let settled_hash = stable_json_hash(&settled).unwrap();
    let deprived_hash = stable_json_hash(&deprived).unwrap();
    let target = settled.area() * (settled.interior.a + settled.interior.r);
    let deprived_e = deprived.area() * (deprived.interior.a + deprived.interior.r);
    let (legacy_hash, _) = legacy_trajectory(&deprived, &p);
    let mut ordinary = deprived.clone();
    let ordinary_ledger = reactions_step(&mut ordinary, &p, DT, true, true);
    let mut cf = deprived.clone();
    let cf_ledger = reactions_step_counterfactual(&mut cf, &p, DT, true, true, 1.0);
    let counterfactual_parity = stable_json_hash(&ordinary).unwrap()
        == stable_json_hash(&cf).unwrap()
        && stable_json_hash(&ordinary_ledger).unwrap() == stable_json_hash(&cf_ledger).unwrap();
    assert!(counterfactual_parity, "gain=1 counterfactual parity failed");
    let response = response_envelope(&deprived, &p);
    let ideal = ideal_source(&deprived, &p, target);
    let finite = finite_resource(&deprived, &p);
    let controller = controller_reachability();
    let sustained_pass = ideal.first_infeasible_step.is_none()
        && (ideal.final_state.e_stored - target).abs() <= BISECTION_TOLERANCE
        && ideal.alive
        && ideal.finite_nonnegative;
    let finite_pass = finite.final_state.e_stored > deprived_e + EPS
        && finite.resource_conservation
        && finite.alive
        && finite.finite_nonnegative;
    let demand_coupling = response
        .windows
        .iter()
        .any(|w| w.demand_by_u[4].median > w.demand_by_u[0].median + EPS)
        || response
            .windows
            .iter()
            .any(|w| w.non_monotonic_fraction > 0.0);
    let classification = if sustained_pass && finite_pass && controller.trace_provenance_complete {
        "DCDEV018R1_SOURCE_SIDE_HOMEOSTASIS_FEASIBLE_CONTROLLER_DERIVATION_DEFECT_CONFIRMED"
    } else if sustained_pass && !finite_pass {
        "DCDEV018R1_SOURCE_SIDE_HOMEOSTASIS_FEASIBLE_FINITE_RESOURCE_LIMIT_CONFIRMED"
    } else if !sustained_pass && controller.trace_provenance_complete && !demand_coupling {
        "DCDEV018R1_SOURCE_SIDE_HOMEOSTASIS_INFEASIBLE_CONFIRMED"
    } else if demand_coupling {
        "DCDEV018R1_STATE_DEPENDENT_SOURCE_DEMAND_COUPLING_INVALIDATES_STATIC_GAIN_MODEL"
    } else {
        "DCDEV018R1_FEASIBILITY_AUDIT_INCONCLUSIVE"
    };
    let audit = Audit {
        directive: "DC-DEV-018-R1".into(),
        entry_commit: ENTRY.into(),
        settled_body_hash: settled_hash.clone(),
        deprived_body_hash: deprived_hash.clone(),
        settled_snapshot: snapshot(&settled, SETTLE_STEPS),
        deprived_snapshot: snapshot(&deprived, DEPRIVATION_STEPS),
        target_e_stored: target,
        deprived_e_stored: deprived_e,
        baseline_trajectory_hash: legacy_hash,
        counterfactual_parity,
        response_envelope: response,
        ideal,
        finite_resource: finite,
        controller_reachability: controller,
        classification: classification.into(),
        production_behavior_changed: false,
        certified_phase1_equations_unchanged: true,
        downstream_started: false,
        next_execution_started: false,
    };
    write(
        &out,
        "protocol.json",
        &json!({
            "directive":"DC-DEV-018-R1", "entry_commit":ENTRY,
            "base_branch":"strategy/dc-dev-016-metabolic-break-even",
            "storage_steps":STORAGE_STEPS, "finite_steps":FINITE_STEPS, "dt":DT,
            "precursor_clamp":PRECURSOR_CLAMP, "finite_inventory":FINITE_INVENTORY,
            "source_law":"B=k_act*q_catalyst(C)*g_harvest*N*F*dt*area; J_source=min(g*B,N*area,F*area)",
            "g_sat":"min(N*area,F*area)/B when B>epsilon",
            "gains":"g=1+u*(g_sat-1)", "fractions":RESPONSE_FRACTIONS,
            "observer_only":true, "production_behavior_changed":false,
            "downstream_started":false
        }),
    );
    write(
        &out,
        "entry_parity.json",
        &json!({
            "settled_body_hash":settled_hash, "deprived_body_hash":deprived_hash,
            "target_e_stored":target, "deprived_e_stored":deprived_e,
            "baseline_trajectory_hash":audit.baseline_trajectory_hash,
            "counterfactual_parity":counterfactual_parity, "pass":counterfactual_parity
        }),
    );
    write(
        &out,
        "response_envelope.json",
        &serde_json::to_value(&audit.response_envelope).unwrap(),
    );
    write(
        &out,
        "ideal_source.json",
        &serde_json::to_value(&audit.ideal).unwrap(),
    );
    write(
        &out,
        "finite_resource.json",
        &serde_json::to_value(&audit.finite_resource).unwrap(),
    );
    write(
        &out,
        "controller_reachability.json",
        &serde_json::to_value(&audit.controller_reachability).unwrap(),
    );
    write(&out, "results.json", &serde_json::to_value(&audit).unwrap());
    write(
        &out,
        "artifact_manifest.json",
        &json!({
            "directive":"DC-DEV-018-R1", "entry_commit":ENTRY,
            "evidence_files":["protocol.json","entry_parity.json","response_envelope.json","ideal_source.json","finite_resource.json","controller_reachability.json","results.json","artifact_manifest.json"],
            "classification":classification, "production_behavior_changed":false,
            "downstream_started":false, "next_execution_started":false
        }),
    );
    println!("DCDEV018R1_CLOSED_LOOP_METABOLIC_FEASIBILITY_AUDIT_COMPLETE\n{classification}\nsettled={settled_hash}\ndeprived={deprived_hash}\ncontroller_trace_provenance_complete=false");
}
