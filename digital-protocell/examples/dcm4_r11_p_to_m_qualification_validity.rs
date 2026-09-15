//! DC-M4-R11: validate the R8/R10 P_TO_M alignment predicate against
//! state-specific frozen-mechanics susceptibility.
//!
//! This is an observer-only diagnostic.  It reconstructs the exact thirty
//! R8/R10 snapshots, derives each snapshot's dominant non-DC activity mode,
//! and applies only discarded ideal pure-mode local-normal probes to cloned
//! bodies.  No probe is connected to polarity, production, fission or any
//! campaign ledger.

#![recursion_limit = "512"]

#[path = "dcfinal001_r4_evolution.rs"]
mod current_population;

use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

const DIRECTIVE: &str =
    "DC-M4-R11-P-TO-M-QUALIFICATION-VALIDITY-AND-MECHANICAL-SUSCEPTIBILITY-GATE-001";
const R10_HEAD: &str = "8bdcadf4b03c503d0871d02c600849fb2ebfe915";
const R10_CI: &str = "34947560845";
const R10_ARTIFACT: &str =
    "sha256:a1956be7b49cc475e95d0806370e4dacf8e162a19777a0466dc717d2b111c938";
const R10_BINARY: &str =
    "025ea9bbcfd2d9b19b1470a589a4526231a631b96e4b2ae34af2f6ef8d2d4ca6";
const HORIZON: usize = 14_778;
const CHECKPOINTS: [usize; 3] = [3_694, 7_389, 11_083];
const ALIGNMENT_THRESHOLD: f64 = 0.50;

fn parse_args() -> (String, String) {
    let mut stage = "seal".to_string();
    let mut output =
        "experiments/generated/dcm4r11ptomqualificationvalidity/raw/qualification.json".to_string();
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
        fs::create_dir_all(parent).expect("create R11 output directory");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize R11 JSON"),
    )
    .expect("write R11 output");
}

fn seal() -> Value {
    json!({
        "directive": DIRECTIVE,
        "status": "PASS",
        "entry_r10": {
            "head": R10_HEAD,
            "ci": R10_CI,
            "artifact": R10_ARTIFACT,
            "binary": R10_BINARY,
            "classification": "POLARITY_NORMAL_REMAP_FAILS_TO_CORRECT_P_TO_M_ALIGNMENT",
        },
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "snapshot_count": 30,
        "old_alignment_metric": {
            "threshold": ALIGNMENT_THRESHOLD,
            "definition": "corresponding dominant nonzero activity-harmonic mechanical modal energy divided by all non-rigid modal energy",
            "source": "R8/R10 independent P_TO_M alignment verifier",
            "qualification_status": "UNDER_VALIDATION",
        },
        "spectral_audit": {
            "stages": [
                "edge_active_concentration_deviation",
                "signed_vertex_deviation_before_clipping",
                "final_r4_clipped_activity",
            ],
            "dc_and_all_nonzero_harmonics": true,
            "dominant_harmonic_source": "same-snapshot final R4 activity, excluding DC",
            "outcome_selection": false,
        },
        "ideal_susceptibility_probe": {
            "force": "local current inward normals weighted by a pure dominant-harmonic sine/cosine partner",
            "amplitude_primary": "actual R10 polarity-specific incremental active-force RMS from the same snapshot",
            "amplitude_secondary": "one half of the actual R10 polarity-specific incremental active-force RMS",
            "signs": [1.0, -1.0],
            "phase_partners": ["cosine", "sine"],
            "mechanics": "frozen mechanics plus local self-contact and conservative remesh on a discarded clone",
            "polarity_feedback": false,
            "production_ledger_debit": false,
            "target_geometry": false,
        },
        "qualification_rule": {
            "usable_threshold": 24,
            "invalid_gate": "at least 24 usable snapshots have ideal pure-mode corresponding-harmonic fraction below 0.50 at both amplitudes",
            "adapter_loss": "only considered if ideal susceptibility preserves the old threshold in at least 24 snapshots",
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
        .map(current_population::run_r11_p_to_m_qualification_arm)
        .collect::<Vec<_>>();
    let snapshots = arms
        .iter()
        .map(|arm| arm["r7_snapshot_count"].as_u64().unwrap_or(0))
        .sum::<u64>();
    json!({
        "directive": DIRECTIVE,
        "kind": "fixed_r8_r10_p_to_m_alignment_validity_and_ideal_susceptibility",
        "entry_r10": {
            "head": R10_HEAD,
            "ci": R10_CI,
            "artifact": R10_ARTIFACT,
            "binary": R10_BINARY,
        },
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "snapshot_count": snapshots,
        "expected_snapshot_count": 30,
        "same_r8_r10_snapshots": true,
        "old_alignment_threshold": ALIGNMENT_THRESHOLD,
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
