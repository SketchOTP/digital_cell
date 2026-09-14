//! DC-M4-R7: full pre-polarity to next pre-polarity response diagnostics.
//!
//! R7 reuses the accepted R4 production arm and adds only the observer-side
//! full-cycle capture implemented in the current-kernel module.  The capture
//! boundary is immediately before `PolarityMassStateV1::advance`; the replay
//! runs the unchanged polarity, paid-mechanics, accepted-step, and next-step
//! prefix operators until that same boundary is reached again.

#![recursion_limit = "512"]

#[path = "dcfinal001_r4_evolution.rs"]
mod current_population;

use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

const DIRECTIVE: &str = "DC-M4-R7-FULL-CYCLE-SAME-BOUNDARY-COUPLED-RESPONSE-IDENTIFICATION-001";
const R6_FINAL_HEAD: &str = "477bf84fc902244731ea85cb0c639459160587c0";
const R6_CI: &str = "34844914011";
const R6_ARTIFACT: &str = "sha256:08cf354bebe0b660cc8f0e342b9d0e6d46d6de769d518475d158a4077e770d56";
const R6_BINARY: &str = "2689bfa8f1dfd0de83bbd3f7bf7b7de5a811882fe7a08f4b0aa1d25e396a7539";
const CHECKPOINTS: [usize; 3] = [3_694, 7_389, 11_083];

fn parse_args() -> (String, String) {
    let mut stage = "all".to_string();
    let mut output = "experiments/generated/dcm4r7fullcyclesameboundary/raw.json".to_string();
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
        fs::create_dir_all(parent).expect("create R7 output directory");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize R7 JSON"),
    )
    .expect("write R7 output");
}

fn seal() -> Value {
    json!({
        "directive": DIRECTIVE,
        "entry_authority": {
            "r6_head": R6_FINAL_HEAD,
            "r6_ci": R6_CI,
            "r6_artifact": R6_ARTIFACT,
            "r6_binary": R6_BINARY,
            "r6_terminal_classification": "R4_COUPLED_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE",
        },
        "boundary": {
            "input": "immediately before PolarityMassStateV1::advance",
            "output": "next execution of the same boundary after one complete accepted cycle",
            "map": "pre_polarity_advance_to_next_pre_polarity_advance",
        },
        "checkpoints": CHECKPOINTS,
        "r6_contract_reused": true,
        "production_biology_changed": false,
        "reproduction_executed": false,
        "population_selection_executed": false,
        "reversal_executed": false,
        "parameter_or_gain_change": false,
        "perturbation_rule": {
            "scheme": "symmetric central differences",
            "scales": ["4*cbrt(f64::EPSILON)", "2*cbrt(f64::EPSILON)", "cbrt(f64::EPSILON)"],
            "normalization": "mean edge length for geometry; RMS positive local amount for polarity",
            "basis": "R6 rotation-covariant non-rigid geometry, zero-sum active/inactive polarity, and local adaptation channels",
            "branch_crossing": "NONSMOOTH; no derivative across remesh/contact/topology changes",
        },
        "status": "PASS",
    })
}

fn compact_arm(full: Value) -> Value {
    let mut full = full;
    let snapshots = full["full_cycle_snapshots"].take();
    json!({
        "arm": full["arm"].take(),
        "connected": full["connected"].take(),
        "accepted": full["accepted"].take(),
        "accepted_steps": full["accepted_steps"].take(),
        "physical_fissions": full["physical_fissions"].take(),
        "post_bootstrap_physical_fissions": full["post_bootstrap_physical_fissions"].take(),
        "fission_attempts": full["fission_attempts"].take(),
        "maximum_mass_over_birth": full["maximum_mass_over_birth"].take(),
        "mechanical_mode_classification": full["mechanical_mode_classification"].take(),
        "mechanical_mode_ratio": full["mechanical_mode_ratio"].take(),
        "full_cycle_snapshots": snapshots,
        "numerical_invalid": full["ledger"]["numerical_invalid"].take(),
        "rejected_steps": full["ledger"]["rejected_steps"].take(),
        "all_simple": full["all_simple"].take(),
        "all_runtime_valid": full["all_runtime_valid"].take(),
        "all_lifecycle_valid": full["all_lifecycle_valid"].take(),
    })
}

fn qualification() -> Value {
    let disconnected = (0..10)
        .map(|index| compact_arm(current_population::run_r7_full_cycle_arm(index, false)))
        .collect::<Vec<_>>();
    let connected = (0..10)
        .map(|index| compact_arm(current_population::run_r7_full_cycle_arm(index, true)))
        .collect::<Vec<_>>();
    let all_arms = disconnected.iter().chain(&connected).collect::<Vec<_>>();
    let snapshots = all_arms
        .iter()
        .map(|arm| arm["full_cycle_snapshots"].as_array().map_or(0, Vec::len))
        .sum::<usize>();
    let exact = all_arms
        .iter()
        .flat_map(|arm| arm["full_cycle_snapshots"].as_array().into_iter().flatten())
        .filter(|snapshot| snapshot["identity"]["same_phase_boundary_exact"] == true)
        .count();
    json!({
        "directive": DIRECTIVE,
        "kind": "held_out_full_cycle_same_phase_response",
        "horizon": 14_778,
        "checkpoints": CHECKPOINTS,
        "conditions": {
            "disconnected": "accepted R3 polarity dynamics; no mechanical output",
            "connected": "same R3 dynamics; accepted R4 local input to existing paid actuator",
        },
        "disconnected": disconnected,
        "connected": connected,
        "snapshot_count": snapshots,
        "exact_identity_count": exact,
        "expected_snapshot_count": 60,
        "expected_exact_identity_count": 60,
        "full_cycle_operator_scope": "pre_polarity_advance_to_next_pre_polarity_advance_full_cycle",
        "full_cycle_jacobian": "NOT_IDENTIFIED",
        "observer_only": true,
        "production_transition_modified": false,
        "reproduction": "NOT_REACHED",
        "selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
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
