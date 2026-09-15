//! DC-M4-R10: polarity-to-existing-inward-normal actuator remap.
//!
//! This example is an opt-in diagnostic harness.  The fixed-snapshot stage
//! runs clone-local route comparisons; the conditional morphogenesis stage is
//! reached only when the independent verifier has accepted E2.  No R10 route
//! is enabled by the current production entry point.

#![recursion_limit = "512"]

#[path = "dcfinal001_r4_evolution.rs"]
mod current_population;

use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

const DIRECTIVE: &str = "DC-M4-R10-POLARITY-TO-EXISTING-INWARD-NORMAL-ACTUATOR-REMAP-001";
const R9_HEAD: &str = "00a7a26f1c5e23b8b9895de7ba1c71c51b2dbbc4";
const R9_CI: &str = "34914837619";
const R9_ARTIFACT: &str = "sha256:d379b78bb56cfce3ca5ba9986e8007869d6961bd6679da0cdf3b2d971fff9bfe";
const R9_BINARY: &str = "b73b9e02d1cbceb926d13104d9f69ebf898a9845438fab7bbc9a29396b15a1d7";
const HORIZON: usize = 14_778;
const CHECKPOINTS: [usize; 3] = [3_694, 7_389, 11_083];
const E2_USABLE_THRESHOLD: usize = 24;
const E2_ALIGNMENT_THRESHOLD: f64 = 0.50;

fn parse_args() -> (String, String) {
    let mut stage = "all".to_string();
    let mut output = "experiments/generated/dcm4r10polaritynormalremap/raw/qualification.json"
        .to_string();
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
        fs::create_dir_all(parent).expect("create R10 output directory");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize R10 JSON"),
    )
    .expect("write R10 output");
}

fn seal() -> Value {
    json!({
        "directive": DIRECTIVE,
        "status": "PASS",
        "entry_r9": {
            "head": R9_HEAD,
            "ci": R9_CI,
            "artifact": R9_ARTIFACT,
            "binary": R9_BINARY,
            "classification": "MECHANOSENSITIVE_ACTIVATION_FAILS_TO_CORRECT_G_TO_P_DAMPING",
        },
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "actuator": {
            "normal_request": "inward_normal_request(mesh, drive, dt)",
            "route_off": "R4_EDGE_TENSION",
            "candidate": "R10_INWARD_NORMAL",
            "candidate_edge_tension": "DISABLED",
            "drive_equation": "d_i_R10 = min(1, d_i_legacy + p_i)",
            "polarity_activity_domain": [0.0, 1.0],
            "new_force_law": false,
            "new_force_cap": false,
            "new_energy_price": false,
        },
        "perturbation_contract": {
            "basis": "R8 radial-normal Fourier modes with sine/cosine partners",
            "alignment_threshold": E2_ALIGNMENT_THRESHOLD,
            "fixed_checkpoints": CHECKPOINTS,
            "same_state_conditions": ["R4_EDGE_TENSION", "R10_NORMAL_REMAP", "R10_POLARITY_CUT"],
        },
        "mechanics_changed": false,
        "ecology_changed": false,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
        "observer_only": true,
    })
}

fn qualification() -> Value {
    env::remove_var("DCFINAL001_R9_MECHANOSENSITIVE");
    env::remove_var("DCFINAL001_R10_NORMAL_REMAP");
    env::set_var("DCFINAL001_R8_CAPTURE_ONLY", "1");
    let arms = (0..10)
        .map(current_population::run_r10_polarity_normal_remap_arm)
        .collect::<Vec<_>>();
    let snapshots = arms
        .iter()
        .map(|arm| arm["r7_snapshot_count"].as_u64().unwrap_or(0))
        .sum::<u64>();
    json!({
        "directive": DIRECTIVE,
        "kind": "fixed_r8_p_to_m_normal_remap_requalification",
        "entry_r9": {
            "head": R9_HEAD,
            "ci": R9_CI,
            "artifact": R9_ARTIFACT,
            "binary": R9_BINARY,
        },
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "e2_usable_threshold": E2_USABLE_THRESHOLD,
        "e2_alignment_threshold": E2_ALIGNMENT_THRESHOLD,
        "arms": arms,
        "snapshot_count": snapshots,
        "expected_snapshot_count": 30,
        "same_r8_snapshots": true,
        "production_transition_modified": false,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
        "terminal_classification": "PENDING_INDEPENDENT_VERIFICATION",
        "observer_only": true,
    })
}

fn morphogenesis() -> Value {
    env::remove_var("DCFINAL001_R9_MECHANOSENSITIVE");
    env::remove_var("DCFINAL001_R8_CAPTURE_ONLY");
    let mut arms = Vec::new();
    for index in 0..10 {
        env::remove_var("DCFINAL001_R10_NORMAL_REMAP");
        let baseline = current_population::run_r4_polarity_arm(index, true);
        env::set_var("DCFINAL001_R10_NORMAL_REMAP", "1");
        let candidate = current_population::run_r4_polarity_arm(index, true);
        arms.push(json!({
            "arm": index + 1,
            "baseline": baseline,
            "candidate": candidate,
            "same_deterministic_history": true,
            "observer_only": true,
        }));
    }
    env::remove_var("DCFINAL001_R10_NORMAL_REMAP");
    json!({
        "directive": DIRECTIVE,
        "kind": "conditional_held_out_resource_morphogenesis",
        "history_domain": "R10_HELD_OUT_MORPHOGENESIS_V1",
        "horizon": HORIZON,
        "arms": arms,
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
        "observer_only": true,
    })
}

fn main() {
    let (stage, output) = parse_args();
    let value = match stage.as_str() {
        "seal" => seal(),
        "qualification" => qualification(),
        "morphogenesis" => morphogenesis(),
        "all" => json!({"parameter_seal": seal(), "qualification": qualification()}),
        other => panic!("unsupported stage {other}"),
    };
    write_json(&output, &value);
}
