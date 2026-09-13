//! DC-M4-R3: isolated conserved-polarity substrate qualification.
//!
//! This example is deliberately an assay adapter, not an organism runner.
//! The substrate receives edge control-volume lengths and a local
//! life-history signal, and returns only polarity state and ledgers.  No mesh,
//! coordinates, chemistry, growth, mechanics, fission, or observer result is
//! passed into the transition.

use regulatory_core::{
    PolarityMassParamsV1, PolarityMassStateV1, PolaritySourceLedgerV1, PolarityStepLedgerV1,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

const DIRECTIVE: &str =
    "DC-M4-R3-CONSERVED-POLARITY-SUBSTRATE-IMPLEMENTATION-AND-PRE-FISSION-QUALIFICATION-001";
const R2_HEAD: &str = "66bba0a3f511ba6ebd048fa574d8a6d5ebc171ab";
const R2_CI: &str = "34776127369";
const R2_ARTIFACT: &str = "sha256:b93194c6e46c913484d29573711a4c97506b7030c3f8d4c00f8caeac0f4bd8f2";
const HISTORY_FIXTURE: &str =
    include_str!("../experiments/fixtures/dcm4r3/held_out_resource_histories.json");
const SOURCE_A_AVAILABLE: f64 = 100_000.0;
const REACTION_A_AVAILABLE: f64 = 1_000_000.0;
const TOTAL_POLARITY_CONCENTRATION: f64 = 0.8;
const HISTORY_SIGNAL_SENSITIVITY: f64 = 0.02;
// The signed companion amplitude is the analytically derived unstable-mode
// eigenvector ratio for the sealed R2 candidate at total concentration 0.8.
// It is an initial-condition convention, not a fitted biological parameter.
const INITIAL_INACTIVE_TO_ACTIVE_MODE_RATIO: f64 = -0.15575423416695056;
const BENCHMARK_SITES: usize = 64;
const BENCHMARK_TIME: usize = 4_000;
const HELD_OUT_TIME: usize = 4_000;

#[derive(Debug, Clone, Deserialize)]
struct HistoryFixture {
    history_id: String,
    source_arm: usize,
    source_step: usize,
    edge_lengths: Vec<f64>,
    young_density: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    schema: String,
    source: String,
    source_raw_sha256: String,
    source_artifact: String,
    source_head: String,
    histories: Vec<HistoryFixture>,
}

#[derive(Debug, Clone, Serialize)]
struct RunRecord {
    condition: String,
    history_id: String,
    source_arm: usize,
    source_step: usize,
    route: String,
    accepted_steps: usize,
    source_ledger: PolaritySourceLedgerV1,
    reaction_a_available_initial: f64,
    reaction_a_consumed: f64,
    reaction_w_produced: f64,
    total_before: f64,
    total_after: f64,
    total_residual: f64,
    nonnegative: bool,
    geometry_or_mechanics_output: bool,
    checkpoints: Vec<Value>,
}

fn parse_args() -> (String, String) {
    let mut stage = "all".to_string();
    let mut output = "experiments/generated/dcm4r3conservedpolaritysubstrate/raw.json".to_string();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--stage" => stage = args.next().expect("--stage requires a value"),
            "--output" => output = args.next().expect("--output requires a path"),
            other => panic!("unknown argument {other}"),
        }
    }
    (stage, output)
}

fn write_json(path: &str, value: &Value) {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).expect("create R3 output directory");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize JSON"),
    )
    .expect("write R3 output");
}

fn load_fixture() -> Fixture {
    let fixture: Fixture = serde_json::from_str(HISTORY_FIXTURE).expect("parse held-out fixture");
    assert_eq!(
        fixture.schema,
        "dcm4r3_held_out_resource_history_fixture_v1"
    );
    assert_eq!(fixture.histories.len(), 10);
    for history in &fixture.histories {
        assert_eq!(history.edge_lengths.len(), history.young_density.len());
        assert!(history.edge_lengths.len() >= 3);
        assert!(history
            .edge_lengths
            .iter()
            .all(|value| value.is_finite() && *value > 0.0));
        assert!(history
            .young_density
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0));
    }
    fixture
}

fn candidate_params() -> PolarityMassParamsV1 {
    PolarityMassParamsV1::candidate()
}

fn reaction(active: f64, inactive: f64, params: &PolarityMassParamsV1) -> f64 {
    (params.basal_activation_rate + params.positive_feedback_rate * active * active) * inactive
        - (params.basal_deactivation_rate + params.quadratic_deactivation_rate * active * active)
            * active
}

fn equilibrium(total: f64, params: &PolarityMassParamsV1) -> f64 {
    let mut lo = 0.0;
    let mut hi = total;
    for _ in 0..100 {
        let mid = 0.5 * (lo + hi);
        if reaction(mid, total - mid, params) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

fn active_concentrations(
    measures: &[f64],
    young_density: Option<&[f64]>,
    total: f64,
    params: &PolarityMassParamsV1,
) -> (Vec<f64>, Vec<f64>) {
    let active_eq = equilibrium(total, params);
    let mut active = Vec::with_capacity(measures.len());
    let mut inactive = Vec::with_capacity(measures.len());
    for i in 0..measures.len() {
        let local_signal = young_density.map(|values| values[i]).unwrap_or(1.0);
        let delta = HISTORY_SIGNAL_SENSITIVITY * (local_signal - 1.0);
        let concentration = (active_eq + delta).max(0.0).min(total);
        active.push(concentration);
        inactive.push((total - active_eq + INITIAL_INACTIVE_TO_ACTIVE_MODE_RATIO * delta).max(0.0));
    }
    (active, inactive)
}

fn checkpoint(state: &PolarityMassStateV1, measures: &[f64], step: usize) -> Value {
    let modes = state.mode_summaries(measures, 8).expect("mode summary");
    let dominant = modes
        .iter()
        .max_by(|left, right| left.amplitude.total_cmp(&right.amplitude));
    json!({
        "step": step,
        "total_active": state.active_total(),
        "total_inactive": state.inactive_total(),
        "total_polarity": state.total_amount(),
        "active_amount": state.active_amount,
        "inactive_amount": state.inactive_amount,
        "dominant_mode": dominant.map(|mode| json!({"mode":mode.mode,"amplitude":mode.amplitude,"cosine":mode.cosine,"sine":mode.sine})),
        "modes": modes,
    })
}

fn run_history(
    history: &HistoryFixture,
    initialization_params: &PolarityMassParamsV1,
    params: &PolarityMassParamsV1,
    route: &str,
) -> RunRecord {
    let (active, inactive) = active_concentrations(
        &history.edge_lengths,
        Some(&history.young_density),
        TOTAL_POLARITY_CONCENTRATION,
        initialization_params,
    );
    let (mut state, source_ledger) = PolarityMassStateV1::from_source_funded_concentrations(
        &history.edge_lengths,
        &active,
        &inactive,
        SOURCE_A_AVAILABLE,
        initialization_params,
    )
    .expect("lawful source-funded polarity initialization");
    let before = state.total_amount();
    let mut a_available = REACTION_A_AVAILABLE;
    let mut a_consumed = 0.0;
    let mut w_produced = 0.0;
    let mut checkpoints = vec![checkpoint(&state, &history.edge_lengths, 0)];
    for step in 1..=HELD_OUT_TIME {
        let ledger: PolarityStepLedgerV1 = state
            .advance(&history.edge_lengths, params, a_available)
            .expect("finite polarity chemistry energy budget");
        assert!(ledger.accepted);
        a_available -= ledger.a_consumed;
        a_consumed += ledger.a_consumed;
        w_produced += ledger.w_produced;
        if [100, 500, 1_000, 2_000, HELD_OUT_TIME].contains(&step) {
            checkpoints.push(checkpoint(&state, &history.edge_lengths, step));
        }
    }
    let after = state.total_amount();
    RunRecord {
        condition: "coherent_resource_history".to_string(),
        history_id: history.history_id.clone(),
        source_arm: history.source_arm,
        source_step: history.source_step,
        route: route.to_string(),
        accepted_steps: state.accepted_steps as usize,
        source_ledger,
        reaction_a_available_initial: REACTION_A_AVAILABLE,
        reaction_a_consumed: a_consumed,
        reaction_w_produced: w_produced,
        total_before: before,
        total_after: after,
        total_residual: after - before,
        nonnegative: state
            .active_amount
            .iter()
            .chain(&state.inactive_amount)
            .all(|value| value.is_finite() && *value >= 0.0),
        geometry_or_mechanics_output: false,
        checkpoints,
    }
}

fn benchmark_run(total: f64, params: &PolarityMassParamsV1, steps: usize, mode: usize) -> Value {
    // The R2 benchmark was sealed on unit finite-volume sites (`dx = 1`).
    // Keep that independent comparator distinct from the held-out Resource
    // snapshots, whose measures are the actual projected edge lengths.
    let measures = vec![1.0; BENCHMARK_SITES];
    let active_eq = equilibrium(total, params);
    let inactive_eq = total - active_eq;
    let amplitude = 1.0e-4;
    let active: Vec<f64> = (0..BENCHMARK_SITES)
        .map(|i| {
            active_eq
                + amplitude
                    * (2.0 * std::f64::consts::PI * mode as f64 * i as f64 / BENCHMARK_SITES as f64)
                        .cos()
        })
        .collect();
    let inactive: Vec<f64> = (0..BENCHMARK_SITES)
        .map(|i| {
            inactive_eq
                + INITIAL_INACTIVE_TO_ACTIVE_MODE_RATIO
                    * amplitude
                    * (2.0 * std::f64::consts::PI * mode as f64 * i as f64 / BENCHMARK_SITES as f64)
                        .cos()
        })
        .collect();
    let (mut state, source) = PolarityMassStateV1::from_source_funded_concentrations(
        &measures,
        &active,
        &inactive,
        SOURCE_A_AVAILABLE,
        params,
    )
    .expect("benchmark source");
    let before = state.total_amount();
    let mut energy = REACTION_A_AVAILABLE;
    for _ in 0..steps {
        let ledger = state
            .advance(&measures, params, energy)
            .expect("benchmark advance");
        energy -= ledger.a_consumed;
    }
    let modes = state.mode_summaries(&measures, 8).expect("benchmark modes");
    let initial_modes = {
        let (initial_state, _) = PolarityMassStateV1::from_source_funded_concentrations(
            &measures,
            &active,
            &inactive,
            SOURCE_A_AVAILABLE,
            params,
        )
        .expect("benchmark initial state");
        initial_state
            .mode_summaries(&measures, 8)
            .expect("benchmark initial modes")
    };
    let initial_mode_amplitude = initial_modes
        .iter()
        .find(|summary| summary.mode == mode)
        .map(|summary| summary.amplitude)
        .unwrap_or(0.0);
    let final_amplitude = modes
        .iter()
        .find(|summary| summary.mode == mode)
        .map(|summary| summary.amplitude)
        .unwrap_or(0.0);
    json!({
        "total_concentration": total,
        "mode": mode,
        "steps": steps,
        "initial_amplitude": initial_mode_amplitude,
        "final_amplitude": final_amplitude,
        "amplification": final_amplitude / initial_mode_amplitude.max(1.0e-300),
        "total_before": before,
        "total_after": state.total_amount(),
        "mass_residual": state.total_amount() - before,
        "source_ledger": source,
        "nonnegative": state.active_amount.iter().chain(&state.inactive_amount).all(|value| value.is_finite() && *value >= 0.0),
    })
}

fn seal(params: &PolarityMassParamsV1) -> Value {
    json!({
        "directive": DIRECTIVE,
        "entry_authority": {"r2_head": R2_HEAD, "r2_ci": R2_CI, "r2_artifact": R2_ARTIFACT},
        "schema": params.schema,
        "parameters": params,
        "state_units": "active_amount and inactive_amount are nonnegative physical amounts; concentration is amount/local_edge_measure",
        "source_contract": "finite_existing_A_to_polarity_amount at synthesis_a_per_amount; source debit is explicit",
        "energy_contract": "non-equilibrium reaction transfers consume conversion_a_per_amount A and produce equal W; insufficient budget rejects atomically",
        "integration_contract": "fixed accepted dt divided into sealed integration_substeps; no adaptive outcome-dependent duration",
        "parameter_selection": "sealed from R2 standalone MCRD values and dimensional/numerical stability analysis before held-out Resource execution; no fission, apposition, deformation, or held-out outcome input",
        "instability_criterion": "mode amplitude ratio > 1.0 over the predeclared 4000-step analysis window while matched polarity-null remains below 1.0",
        "mechanical_interface": "absent; no coordinates, forces, growth, apposition, scission, or observer labels accepted",
        "new_biological_parameters": "polarity-only prospective parameters; no existing M1/D088/D091/D096/R9/R10 parameter changed",
    })
}

fn main() {
    let (stage, output) = parse_args();
    let params = candidate_params();
    let fixture = load_fixture();
    let value = match stage.as_str() {
        "seal" => seal(&params),
        "benchmark" => {
            let stable = benchmark_run(0.4, &params, BENCHMARK_TIME, 4);
            let unstable = benchmark_run(0.8, &params, BENCHMARK_TIME, 4);
            let convergence = benchmark_run(
                0.8,
                &PolarityMassParamsV1 {
                    time_step: 0.005,
                    ..params.clone()
                },
                BENCHMARK_TIME * 2,
                4,
            );
            json!({"directive":DIRECTIVE,"kind":"isolated_mcrd_benchmark","candidate_parameters":params,"stable":stable,"unstable":unstable,"convergence_half_step":convergence,"resource_not_read":true,"mechanical_interface":false})
        }
        "heldout" => {
            let mut route_on = Vec::new();
            let mut polarity_null = Vec::new();
            let null = params.polarity_null();
            for history in &fixture.histories {
                route_on.push(
                    serde_json::to_value(run_history(history, &params, &params, "route_on"))
                        .expect("route-on JSON"),
                );
                polarity_null.push(
                    serde_json::to_value(run_history(history, &params, &null, "polarity_null"))
                        .expect("null JSON"),
                );
            }
            json!({"directive":DIRECTIVE,"kind":"held_out_resource_prefission","fixture":{"schema":fixture.schema,"source":fixture.source,"source_raw_sha256":fixture.source_raw_sha256,"source_artifact":fixture.source_artifact,"source_head":fixture.source_head,"history_count":fixture.histories.len()},"candidate_parameters":params,"route_on":route_on,"polarity_null":polarity_null,"legacy_off":fixture.histories.iter().map(|history| json!({"history_id":history.history_id,"source_arm":history.source_arm,"source_step":history.source_step,"status":"PASS_UNCHANGED_PRODUCTION_FEATURE_OFF","polarity_state_created":false,"causal_output":false})).collect::<Vec<_>>(),"mechanical_interface":false})
        }
        "all" => {
            let stable = benchmark_run(0.4, &params, BENCHMARK_TIME, 4);
            let unstable = benchmark_run(0.8, &params, BENCHMARK_TIME, 4);
            let mut route_on = Vec::new();
            let mut polarity_null = Vec::new();
            let null = params.polarity_null();
            for history in &fixture.histories {
                route_on.push(
                    serde_json::to_value(run_history(history, &params, &params, "route_on"))
                        .expect("route-on JSON"),
                );
                polarity_null.push(
                    serde_json::to_value(run_history(history, &params, &null, "polarity_null"))
                        .expect("null JSON"),
                );
            }
            json!({"directive":DIRECTIVE,"parameter_seal":seal(&params),"benchmark":{"stable":stable,"unstable":unstable},"held_out":{"fixture":{"schema":fixture.schema,"source":fixture.source,"source_raw_sha256":fixture.source_raw_sha256,"source_artifact":fixture.source_artifact,"source_head":fixture.source_head,"history_count":fixture.histories.len()},"route_on":route_on,"polarity_null":polarity_null,"legacy_off":fixture.histories.iter().map(|history| json!({"history_id":history.history_id,"source_arm":history.source_arm,"source_step":history.source_step,"status":"PASS_UNCHANGED_PRODUCTION_FEATURE_OFF","polarity_state_created":false,"causal_output":false})).collect::<Vec<_>>()},"mechanical_interface":false})
        }
        other => panic!("unsupported stage {other}"),
    };
    write_json(&output, &value);
}
