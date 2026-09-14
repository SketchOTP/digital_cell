//! DC-M4-R8: clone-local modular mechanochemical edge decomposition.
//!
//! R8 deliberately does not construct a global Jacobian.  It recreates the
//! accepted R7 same-phase snapshots, cuts P->M and G->P separately on cloned
//! states, and emits enough raw modal response for an independent verifier to
//! recompute the attribution.

#![recursion_limit = "512"]

#[path = "dcfinal001_r4_evolution.rs"]
mod current_population;

use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

const DIRECTIVE: &str = "DC-M4-R8-NONLINEAR-MECHANOCHEMICAL-CAUSAL-EDGE-DECOMPOSITION-001";
const R7_FINAL_HEAD: &str = "73d2e420da8a704926ebdd47e0d0b76dd63d2fbb";
const R7_CI: &str = "34856311075";
const R7_ARTIFACT: &str = "sha256:7a93e00fce1152d39edf582e7004acc1b34b1a2f5e0a821b63b31b91b1102623";
const R7_BINARY: &str = "430496f154b4019e651bfa4d9232fb6d22f8cf2897eff1bce93bfa6736ef1ae2";
const HORIZON: usize = 14_778;
const CHECKPOINTS: [usize; 3] = [3_694, 7_389, 11_083];
const USABLE_SNAPSHOT_THRESHOLD: usize = 24;

fn parse_args() -> (String, String) {
    let mut stage = "all".to_string();
    let mut output = "experiments/generated/dcm4r8nonlinearmechanochemicalcausaledge/raw/qualification.json".to_string();
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
        fs::create_dir_all(parent).expect("create R8 output directory");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize R8 JSON"),
    )
    .expect("write R8 output");
}

fn seal() -> Value {
    json!({
        "directive": DIRECTIVE,
        "entry_authority": {
            "r7_final_head": R7_FINAL_HEAD,
            "r7_ci": R7_CI,
            "r7_artifact": R7_ARTIFACT,
            "r7_binary": R7_BINARY,
            "r7_terminal_classification": "R4_FULL_CYCLE_RESPONSE_NONSMOOTH_OR_UNIDENTIFIABLE",
        },
        "same_phase": {
            "input": "immediately before PolarityMassStateV1::advance",
            "output": "next execution of the same boundary after one accepted cycle",
            "r7_checkpoints": CHECKPOINTS,
            "horizon": HORIZON,
        },
        "modular_graph": [
            "PolarityMassStateV1 active/inactive amounts -> derive_local_activity vertex activity",
            "vertex activity -> existing paid R4 polarity-derived edge-tension input",
            "existing actuator -> mechanics/remesh -> physical edge measures",
            "physical edge measures -> next PolarityMassStateV1::advance",
        ],
        "coordinates": {
            "polarity": "active vertex-activity spatial harmonics with cosine and sine partners",
            "mechanics": "non-rigid radial-normal displacement harmonics with cosine and sine partners",
            "excluded": ["rigid translation", "rigid rotation", "target geometry", "fission outcome"],
        },
        "p_to_m": {
            "full": "normal accepted R4 polarity-derived activity",
            "cut": "only polarity-derived activity removed on a cloned post-polarity state",
            "retained": ["legacy drive", "passive mechanics", "adaptation", "actuator cap", "actuator A->W cost"],
        },
        "g_to_p": {
            "full": "perturbed physical edge measures passed to the next polarity advance",
            "cut": "baseline edge-measure vector passed only to that diagnostic polarity advance",
            "normalized_amplitudes": [0.001, 0.0005],
            "signs": [1.0, -1.0],
        },
        "classification_rule": {
            "usable_snapshot_threshold": USABLE_SNAPSHOT_THRESHOLD,
            "historical_endpoint_labels": "post-hoc only; never used for settings",
            "global_jacobian": "NOT_CONSTRUCTED",
        },
        "production_biology_changed": false,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "status": "PASS",
    })
}

fn qualification() -> Value {
    let arms = (0..10)
        .map(current_population::run_r8_modular_edge_arm)
        .collect::<Vec<_>>();
    let snapshots = arms
        .iter()
        .map(|arm| arm["r7_snapshot_count"].as_u64().unwrap_or(0))
        .sum::<u64>();
    json!({
        "directive": DIRECTIVE,
        "kind": "clone_local_nonlinear_mechanochemical_causal_edge_decomposition",
        "entry_r7": {
            "head": R7_FINAL_HEAD,
            "ci": R7_CI,
            "artifact": R7_ARTIFACT,
            "binary": R7_BINARY,
        },
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "usable_snapshot_threshold": USABLE_SNAPSHOT_THRESHOLD,
        "arms": arms,
        "snapshot_count": snapshots,
        "expected_snapshot_count": 30,
        "historical_r4_endpoint_groups": "NOT_USED_UNTIL_AFTER_MODULAR_RESPONSES",
        "global_jacobian": "NOT_CONSTRUCTED",
        "observer_only": true,
        "production_transition_modified": false,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "terminal_classification": "PENDING_INDEPENDENT_VERIFICATION",
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
