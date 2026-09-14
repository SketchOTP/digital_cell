//! DC-M4-R6: atomic coupled-state capture and replay diagnostics.
//!
//! This example reuses the unchanged R4 current-kernel arm and enables only
//! its observer-side pre-mechanics capture hook. It does not change R3/R4
//! equations, parameters, actuator limits, ecology, or fission behavior.
//! The output is deliberately compact: complete causal snapshots and replay
//! results are retained, while the large forward trajectory is omitted.

#![recursion_limit = "512"]

#[path = "dcfinal001_r4_evolution.rs"]
mod current_population;

use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

const DIRECTIVE: &str = "DC-M4-R6-ATOMIC-COUPLED-STATE-REPLAY-AND-JACOBIAN-IDENTIFICATION-001";
const R5_HEAD: &str = "c1af2a6de5b43df2bf27dc99fca25d667b302598";
const R5_CI: &str = "34835887327";
const R5_ARTIFACT: &str = "sha256:a9deb83c7786f4f265cda5df34d45f9767b024a5c44557ce2fd68aa1cc859eee";
const R4_HEAD: &str = "38de33d6ed2741a60b3e825360a45b8ff4686792";
const R4_CI: &str = "34802371283";
const R4_ARTIFACT: &str = "sha256:1be6045d0fdc0b91dda666baefe363e9490ee1cc2be498169b5a0cd3219b677b";
const CHECKPOINTS: [usize; 3] = [3_694, 7_389, 11_083];

fn parse_args() -> (String, String) {
    let mut stage = "all".to_string();
    let mut output = "experiments/generated/dcm4r6atomiccoupledstate/raw.json".to_string();
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
        fs::create_dir_all(parent).expect("create R6 output directory");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize R6 JSON"),
    )
    .expect("write R6 output");
}

fn seal() -> Value {
    json!({
        "directive": DIRECTIVE,
        "entry_authority": {
            "r5_head": R5_HEAD,
            "r5_ci": R5_CI,
            "r5_artifact": R5_ARTIFACT,
            "r5_terminal_classification": "R4_ATTRIBUTION_INCONCLUSIVE",
            "r4_head": R4_HEAD,
            "r4_ci": R4_CI,
            "r4_artifact": R4_ARTIFACT,
        },
        "feature": "atomic pre-mechanics capture and same-state replay",
        "production_biology_changed": false,
        "reproduction_executed": false,
        "population_selection_executed": false,
        "checkpoints": CHECKPOINTS,
        "history_selection": "all ten fixed arm identities; checkpoint selection independent of endpoint labels",
        "perturbation_rule": {
            "scheme": "symmetric central differences",
            "scales": ["cbrt(f64::EPSILON)", "2*cbrt(f64::EPSILON)", "4*cbrt(f64::EPSILON)"],
            "normalization": "mean edge length for geometry; RMS positive local amount for polarity",
            "positivity": "deterministic amplitude reduction only if a perturbed polarity amount would be negative",
            "topology_boundary": "mark NONSMOOTH; do not differentiate across remesh/contact/fission branches",
        },
        "mode_basis": {
            "mechanical": "all non-rigid radial and tangential sine/cosine ring modes represented by the diagnostic adapter",
            "polarity": "active and inactive amount sine/cosine partners with zero first-order total perturbation",
            "absolute_ring_index_preference": false,
        },
        "sealed_before_response_execution": true,
        "status": "PASS",
    })
}

fn compact_arm(full: Value) -> Value {
    let mut full = full;
    let snapshots = full["atomic_replay_snapshots"].take();
    json!({
        "arm": full["arm"].take(),
        "connected": full["connected"].take(),
        "accepted": full["accepted"].take(),
        "accepted_steps": full["accepted_steps"].take(),
        "physical_fissions": full["physical_fissions"].take(),
        "post_bootstrap_physical_fissions": full["post_bootstrap_physical_fissions"].take(),
        "fission_attempts": full["fission_attempts"].take(),
        "mechanical_mode_ratio": full["mechanical_mode_ratio"].take(),
        "mechanical_mode_classification": full["mechanical_mode_classification"].take(),
        "maximum_mass_over_birth": full["maximum_mass_over_birth"].take(),
        "polarity_total_amount": full["polarity_total_amount"].take(),
        "polarity_active_amount": full["polarity_active_amount"].take(),
        "atomic_replay_snapshots": snapshots,
        "numerical_invalid": full["ledger"]["numerical_invalid"].take(),
        "rejected_steps": full["ledger"]["rejected_steps"].take(),
        "all_simple": full["all_simple"].take(),
        "all_runtime_valid": full["all_runtime_valid"].take(),
        "all_lifecycle_valid": full["all_lifecycle_valid"].take(),
    })
}

fn qualification() -> Value {
    let disconnected = (0..10)
        .map(|index| compact_arm(current_population::run_r6_atomic_replay_arm(index, false)))
        .collect::<Vec<_>>();
    let connected = (0..10)
        .map(|index| compact_arm(current_population::run_r6_atomic_replay_arm(index, true)))
        .collect::<Vec<_>>();
    json!({
        "directive": DIRECTIVE,
        "kind": "held_out_atomic_pre_mechanics_replay",
        "horizon": 14_778,
        "checkpoints": CHECKPOINTS,
        "conditions": {
            "disconnected": "accepted R3 polarity dynamics; no mechanical output",
            "connected": "same R3 dynamics; accepted R4 local input to existing paid actuator",
            "same_state_counterfactual": "each captured connected pre-mechanics state is replayed with polarity input disconnected",
        },
        "disconnected": disconnected,
        "connected": connected,
        "observer_only": true,
        "production_transition_modified": false,
        "reproduction_qualification": "not evaluated by this stage",
    })
}

fn main() {
    let (stage, output) = parse_args();
    let value = match stage.as_str() {
        "seal" => seal(),
        "qualification" => qualification(),
        "all" => json!({"parameter_seal": seal(), "qualification": qualification()}),
        other => panic!("unsupported stage {other}"),
    };
    write_json(&output, &value);
}
