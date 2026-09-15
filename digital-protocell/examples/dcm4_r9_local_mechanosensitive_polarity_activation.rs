//! DC-M4-R9: local tensile mechanosensitive polarity activation.
//!
//! R9 is a bounded requalification of the R8 G_TO_P edge.  It compares the
//! frozen R4 polarity chemistry with one opt-in local strain multiplier on
//! cloned R8 checkpoints.  No R9 diagnostic result is connected to mechanics.

#![recursion_limit = "512"]

#[path = "dcfinal001_r4_evolution.rs"]
mod current_population;

use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

const DIRECTIVE: &str = "DC-M4-R9-LOCAL-MECHANOSENSITIVE-POLARITY-ACTIVATION-001";
const R8_HEAD: &str = "18cd5daff29267469db838acbba9bf9691e94ba6";
const R8_CI: &str = "34865782025";
const R8_ARTIFACT: &str = "sha256:30b75d9389adda97bde60654d00caf8b86b80c4ff0998efc52d43764b9303073";
const R8_BINARY: &str = "ec852cd9110577406103c40003f5f39292aad0d0f58d1927864211cf2b920812";
const HORIZON: usize = 14_778;
const CHECKPOINTS: [usize; 3] = [3_694, 7_389, 11_083];
const E2_USABLE_THRESHOLD: usize = 24;

fn parse_args() -> (String, String) {
    let mut stage = "all".to_string();
    let mut output =
        "experiments/generated/dcm4r9localmechanosensitivepolarityactivation/raw/qualification.json"
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
        fs::create_dir_all(parent).expect("create R9 output directory");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize R9 JSON"),
    )
    .expect("write R9 output");
}

fn seal() -> Value {
    json!({
        "directive": DIRECTIVE,
        "status": "PASS",
        "entry_r8": {
            "head": R8_HEAD,
            "ci": R8_CI,
            "artifact": R8_ARTIFACT,
            "binary": R8_BINARY,
            "classification": "R4_GEOMETRY_TO_POLARITY_FEEDBACK_DAMPING",
        },
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "input": {
            "source": "MaterialMesh::load_bearing_strain(i)",
            "units": "dimensionless signed strain; mature_structural_fraction * ((edge_length - rest_length) / rest_length)",
            "timing": "same pre-PolarityMassStateV1::advance mesh at the R8 pre-polarity boundary",
            "signal": "s_i = max(0, load_bearing_strain_i)",
            "forbidden_inputs": ["observer", "centroid", "axis", "target geometry", "apposition", "fission", "success"],
        },
        "equation": "rate_i = (1 + max(0, load_bearing_strain_i)) * (k0 + k_fb * active_i^2) * inactive_i - (kd + kq * active_i^2) * active_i",
        "parameters": {
            "source": "sealed R3 PolarityMassParamsV1::candidate",
            "new_mechanosensitive_coefficient": "NONE",
            "unit_multiplier": 1.0,
            "r3_parameter_digest": "derived from the unchanged R3 candidate at execution",
        },
        "perturbation_contract": {
            "normalized_amplitudes": [0.001, 0.0005],
            "definition": "mean edge length times R8 radial normal Fourier mode",
            "signs": [1.0, -1.0],
            "measurement": "FULL minus baseline edge-measure cut immediately after polarity advance and before mechanics",
            "fixed_checkpoints": CHECKPOINTS,
        },
        "conditions": ["R4_BASELINE", "R9_MECHANOSENSITIVE"],
        "same_snapshots": true,
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
    env::set_var("DCFINAL001_R8_CAPTURE_ONLY", "1");
    let arms = (0..10)
        .map(current_population::run_r9_mechanosensitive_edge_arm)
        .collect::<Vec<_>>();
    let snapshots = arms
        .iter()
        .map(|arm| arm["r7_snapshot_count"].as_u64().unwrap_or(0))
        .sum::<u64>();
    json!({
        "directive": DIRECTIVE,
        "kind": "fixed_r8_g_to_p_mechanosensitive_requalification",
        "entry_r8": {
            "head": R8_HEAD,
            "ci": R8_CI,
            "artifact": R8_ARTIFACT,
            "binary": R8_BINARY,
        },
        "horizon": HORIZON,
        "checkpoints": CHECKPOINTS,
        "e2_usable_threshold": E2_USABLE_THRESHOLD,
        "arms": arms,
        "snapshot_count": snapshots,
        "expected_snapshot_count": 30,
        "same_r8_snapshots": true,
        "mechanical_output": false,
        "production_transition_modified": false,
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
        "all" => json!({"parameter_seal": seal(), "qualification": qualification()}),
        other => panic!("unsupported stage {other}"),
    };
    write_json(&output, &value);
}
