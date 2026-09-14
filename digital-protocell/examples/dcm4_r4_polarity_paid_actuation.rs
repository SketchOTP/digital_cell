//! DC-M4-R4: polarity-to-paid-actuation causal coupling.
//!
//! This is an opt-in diagnostic adapter around the current shared population
//! kernel.  The disconnected and connected arms execute the same polarity
//! chemistry; only the latter passes the sealed local activity input to the
//! existing activated-energy contractility operator.  No new force law or
//! reproduction controller is defined here.

#![recursion_limit = "512"]

#[path = "dcfinal001_r4_evolution.rs"]
mod current_population;

use regulatory_core::{
    ContractilityParamsV1, PolarityActuationParamsV1, PolarityMassParamsV1,
    FROZEN_MAX_ACTIVE_TENSION, FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME,
    POLARITY_ACTUATION_SCHEMA_V1, R3_TOTAL_POLARITY_CONCENTRATION,
};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

const DIRECTIVE: &str = "DC-M4-R4-POLARITY-TO-PAID-ACTUATION-CAUSAL-COUPLING-001";
const R3_HEAD: &str = "70f869bd90c48834e3fcd6e57d845eef99a09247";
const R3_CI: &str = "34790316854";
const R3_ARTIFACT: &str = "sha256:4e778d9ca4378cbd3a0bc9b3912797e26ecbb4a94fc280cc49fc98c72f2f9c99";
const RESOURCE_HORIZON: usize = 14_778;

fn parse_args() -> (String, String) {
    let mut stage = "all".to_string();
    let mut output = "experiments/generated/dcm4r4polaritypaidactuation/raw.json".to_string();
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
        fs::create_dir_all(parent).expect("create R4 output directory");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize R4 JSON"),
    )
    .expect("write R4 output");
}

fn seal() -> Value {
    let polarity = PolarityMassParamsV1::candidate();
    let actuation = PolarityActuationParamsV1::sealed_r3_candidate()
        .expect("R3-derived actuation reference must be valid");
    let contractility = ContractilityParamsV1::default();
    json!({
        "directive": DIRECTIVE,
        "entry_authority": {
            "r3_head": R3_HEAD,
            "r3_ci": R3_CI,
            "r3_artifact": R3_ARTIFACT,
            "r3_terminal_classification": "ROUTE_B_AUTONOMOUS_PREFISSION_MODE_GENERATION_DEMONSTRATED"
        },
        "coupling_schema": POLARITY_ACTUATION_SCHEMA_V1,
        "polarity_parameters": polarity,
        "actuation_parameters": actuation,
        "actuator_parameters": contractility,
        "equation": {
            "edge_active_concentration": "c_i = active_amount[i] / positive_edge_measure[i]",
            "vertex_local_concentration": "c_v = 0.5 * (c_edge[v-1] + c_edge[v])",
            "activity_input": "u_v = clamp((c_v - c_eq) / c_eq, 0, 1)",
            "reference_active_concentration": actuation.reference_active_concentration,
            "reference_total_concentration": R3_TOTAL_POLARITY_CONCENTRATION,
            "homogeneous_equilibrium_output": 0.0,
            "global_normalization": false,
            "locality": "two adjacent edge amounts only"
        },
        "actuator": {
            "source": "crates/regulatory-core/src/contractility.rs",
            "call_path": "examples/dcfinal001_r5_v4_neck.rs::r10_refractory_mechanics_step_with_polarity_diagnostics -> apply_local_activated_energy_contractility_with_funded_extra_and_passive_forces_self_contact",
            "input": "bounded per-vertex activity in [0,1]",
            "edge_tension": "max_active_tension * 0.5 * (activity[i] + activity[i+1])",
            "max_active_tension": FROZEN_MAX_ACTIVE_TENSION,
            "reserve_cost_per_force_length_time": FROZEN_RESERVE_COST_PER_FORCE_LENGTH_TIME,
            "mechanics_owner": "chemistry-core mechanics; actuator never writes coordinates",
            "existing_a_to_w_work_debit": true,
            "cap_changed": false,
            "cost_changed": false
        },
        "energy_separation": {
            "polarity_chemistry": "R3 conversion A->W transaction occurs before mechanics",
            "mechanical_actuation": "existing actuator independently debits A and credits W after accepted mechanics",
            "double_spend": false,
            "polarity_driven_mechanical_work_in_r3": false
        },
        "forbidden_inputs": [
            "centroid", "midpoint", "body_axis", "target_geometry", "neck_location",
            "apposition_state", "fission_state", "generation", "observer_output", "global_normalization"
        ],
        "parameter_selection": "reference concentration is derived from the sealed R3 homogeneous fixed point; no coefficient or held-out Resource outcome selects the mapping",
        "resource_horizon": RESOURCE_HORIZON,
        "feature_default": "OFF",
        "sealed_before_heldout": true
    })
}

fn qualification() -> Value {
    let disconnected = (0..10)
        .map(|index| current_population::run_r4_polarity_arm(index, false))
        .collect::<Vec<_>>();
    let connected = (0..10)
        .map(|index| current_population::run_r4_polarity_arm(index, true))
        .collect::<Vec<_>>();
    json!({
        "directive": DIRECTIVE,
        "kind": "held_out_coherent_resource_prefission_morphogenesis",
        "horizon": RESOURCE_HORIZON,
        "conditions": {
            "disconnected": "accepted R3 polarity dynamics; no mechanical output",
            "connected": "same R3 dynamics; sealed local input to existing paid actuator"
        },
        "disconnected": disconnected,
        "connected": connected,
        "observer_only": true,
        "reproduction_qualification": "not evaluated by this stage"
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
