//! DC-M4-R12: state-normalized P_TO_M force-response requalification.
//!
//! R12 is observer/replay-only.  It extracts the native R4 edge-tension and
//! R10 inward-normal polarity force contributions, probes each contribution
//! on discarded clones at signed two-scale amplitudes, and compares the
//! biological differential response with that state's own susceptibility
//! envelope.  No R12 probe is connected to production biology, fission, or a
//! production energy ledger.

#![recursion_limit = "512"]

#[path = "dcfinal001_r4_evolution.rs"]
mod current_population;

use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

const DIRECTIVE: &str = "DC-M4-R12-STATE-NORMALIZED-P-TO-M-FORCE-RESPONSE-REQUALIFICATION-GATE-001";
const R11_HEAD: &str = "c5d5342b19e912c004dd87eaf33a1ce1f16d27d7";
const R11_CI: &str = "35034373866";
const R11_ARTIFACT: &str =
    "sha256:acade736638f3c607996e246457f5c17744aedb3df0cfb63df573aa4063f28c6";
const R11_BINARY: &str = "332c97a7e73dd3b3abd5da211ad47fea64722411e825b9081d074e1b7eab5a8f";
const HORIZON: usize = 14_778;
const CHECKPOINTS: [usize; 3] = [3_694, 7_389, 11_083];

fn parse_args() -> (String, String) {
    let mut stage = "seal".to_string();
    let mut output =
        "experiments/generated/dcm4r12statenormalizedptomforcerequalification/raw/qualification.json"
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
        fs::create_dir_all(parent).expect("create R12 output directory");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize R12 JSON"),
    )
    .expect("write R12 output");
}

fn seal() -> Value {
    json!({
        "directive": DIRECTIVE,
        "status": "PASS",
        "entry_r11": {
            "head": R11_HEAD,
            "ci": R11_CI,
            "artifact": R11_ARTIFACT,
            "binary": R11_BINARY,
            "classification": "P_TO_M_ALIGNMENT_GATE_INVALID_FOR_CURRENT_BODY_STATES",
        },
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "snapshot_count": 30,
        "routes": ["R4_EDGE_TENSION", "R10_INWARD_NORMAL"],
        "force_provenance": {
            "r4": "native funded polarity edge tensions plus the exact funded legacy normal field before frozen mechanics",
            "r10": "native funded full-minus-cut inward-normal vertex force field before frozen mechanics",
            "representation_rule": "retain native route; use exact generalized vertex projection only for signed susceptibility probes",
        },
        "susceptibility_contract": {
            "scales": [1.0, 0.5],
            "signs": [1.0, -1.0],
            "zero_force_baseline": "included for passive-displacement centering; it is not a production transition",
            "r1": "[r(+1)-r(-1)]/2",
            "r05": "[r(+0.5)-r(-0.5)]/2",
            "probe_force_cap": "preserve exact source direction and uniformly scale only when needed to satisfy MAX_EXTERNAL_FORCE_PER_VERTEX",
            "mechanics": "frozen mechanics plus self-contact and remesh on discarded clones",
            "branch_divergence": "NONSMOOTH",
            "sign_symmetry_tolerance": 0.05,
            "two_scale_tolerance": 0.05,
            "envelope_margin": 0.05,
            "qualification": "state-specific control envelope; no universal correlation cutoff",
        },
        "qualification_rule": {
            "usable_threshold": 24,
            "r4_or_r10": "a route qualifies when at least 24/30 usable states pass its own signed/two-scale susceptibility envelope",
            "outcome_selection": false,
        },
        "production_biology_modified": false,
        "morphogenesis": "NOT_REACHED",
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
        .map(current_population::run_r12_state_normalized_p_to_m_arm)
        .collect::<Vec<_>>();
    let snapshot_count = arms
        .iter()
        .map(|arm| arm["r7_snapshot_count"].as_u64().unwrap_or(0))
        .sum::<u64>();
    json!({
        "directive": DIRECTIVE,
        "kind": "state_normalized_native_p_to_m_force_response",
        "entry_r11": {
            "head": R11_HEAD,
            "ci": R11_CI,
            "artifact": R11_ARTIFACT,
            "binary": R11_BINARY,
        },
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "snapshot_count": snapshot_count,
        "expected_snapshot_count": 30,
        "same_r8_r10_snapshots": true,
        "arms": arms,
        "production_biology_modified": false,
        "morphogenesis": "NOT_REACHED",
        "reproduction": "NOT_REACHED",
        "population_selection": "NOT_REACHED",
        "reversal": "NOT_REACHED",
        "final_integration": "NOT_REACHED",
        "terminal_classification": "PENDING_INDEPENDENT_VERIFICATION",
        "observer_only": true,
    })
}

fn main() {
    let (stage, output) = parse_args();
    let value = match stage.as_str() {
        "seal" => seal(),
        "qualification" => qualification(),
        other => panic!("unsupported stage {other}"),
    };
    write_json(&output, &value);
}
