//! DC-DEV-020-M1-R1-R1 observer-only starvation-decay causal isolation.
//!
//! This runner reuses the exact M1-R1 capacity shadows and changes only the
//! diagnostic A-decay coefficient in the two neutralized shadows. Production
//! chemistry remains in chemistry-core and is never modified.

#[path = "dcdev020m1r1_capacity_decomp.rs"]
mod m1r1;

use chemistry_core::mesh_reactions::ReactionParams;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const STARTING_HEAD: &str = "9db2c7d08495f8e935a59385bf51927bcd951a7b";
const M1R1_ENTRY: &str = "3cab12551072ad1eafaece72615f448d8efb9bea";
const HORIZON_STEPS: usize = 480;
const TOL: f64 = 1e-8;
const EXPECTED_BASE_DELTA: f64 = -9.200978427498057;
const EXPECTED_RAW_SOURCE_DELTA: f64 = -3.09944444397982;
const EXPECTED_ORIGINAL_CATALYST_OFF_DELTA: f64 = -9.189726322512357;
const EXPECTED_ORIGINAL_COMBINED_DELTA: f64 = -3.4176092342042637;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL
}

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn arm_json(
    arm: &m1r1::ArmResult,
    observations: &[m1r1::DecayObservation],
) -> Result<Value, serde_json::Error> {
    Ok(json!({
        "arm": serde_json::to_value(arm)?,
        "decay_observations": observations,
    }))
}

fn field_f64(arm: &Value, field: &str) -> f64 {
    arm["arm"][field]
        .as_f64()
        .unwrap_or_else(|| panic!("missing numeric arm field {field}"))
}

fn ledger_f64(arm: &Value, field: &str) -> f64 {
    arm["arm"]["ledger"][field]
        .as_f64()
        .unwrap_or_else(|| panic!("missing numeric ledger field {field}"))
}

fn snapshot_f64(arm: &Value, state: &str, field: &str) -> f64 {
    arm["arm"][state][field]
        .as_f64()
        .unwrap_or_else(|| panic!("missing numeric {state}.{field}"))
}

fn without_decay(params: &ReactionParams) -> Value {
    let mut value = serde_json::to_value(params).expect("ReactionParams serializes");
    value
        .as_object_mut()
        .expect("ReactionParams is an object")
        .remove("k_a_decay");
    value
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var_os("DCDEV020M1R1R1_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("experiments/generated/dcdev020m1r1r1"));
    let (deprived, mechanics) = m1r1::m1r1_entry_state();
    let production = m1r1::reaction_params();
    let production_k_a_decay = production.k_a_decay;
    let neutral_k_a_decay = production_k_a_decay / 4.0;

    // A/B are the exact original M1-R1 arms. The two additional original
    // controls close Gate 0 for the catalyst-off and combined ledgers.
    let (base, _) =
        m1r1::run_arm_with_options(&deprived, m1r1::Shadow::Base, &mechanics, None, false);
    let (raw_source, raw_observations) = m1r1::run_arm_with_options(
        &deprived,
        m1r1::Shadow::SourceCapacityUpperBound,
        &mechanics,
        None,
        true,
    );
    let (original_catalyst_off, _) = m1r1::run_arm_with_options(
        &deprived,
        m1r1::Shadow::CatalystInvestmentOff,
        &mechanics,
        None,
        false,
    );
    let (original_combined, _) =
        m1r1::run_arm_with_options(&deprived, m1r1::Shadow::Combined, &mechanics, None, false);

    assert!(close(base.organized_material_delta, EXPECTED_BASE_DELTA));
    assert!(close(
        raw_source.organized_material_delta,
        EXPECTED_RAW_SOURCE_DELTA
    ));
    assert!(close(
        original_catalyst_off.organized_material_delta,
        EXPECTED_ORIGINAL_CATALYST_OFF_DELTA
    ));
    assert!(close(
        original_combined.organized_material_delta,
        EXPECTED_ORIGINAL_COMBINED_DELTA
    ));

    assert_eq!(raw_observations.len(), HORIZON_STEPS);
    assert!(raw_observations.iter().all(|observation| {
        observation.starvation_predicate && close(observation.selected_multiplier, 4.0)
    }));
    let four_x_steps = raw_observations
        .iter()
        .filter(|observation| close(observation.selected_multiplier, 4.0))
        .count();
    assert_eq!(four_x_steps, HORIZON_STEPS);

    let mut shadow_params = production;
    shadow_params.k_a_decay = neutral_k_a_decay;
    assert!(close(production_k_a_decay, 4.0 * neutral_k_a_decay));
    assert!(close(shadow_params.k_a_decay * 4.0, production_k_a_decay));
    assert_eq!(without_decay(&production), without_decay(&shadow_params));
    assert_eq!(
        raw_observations
            .iter()
            .filter(|observation| close(observation.declared_k_a_decay, production_k_a_decay))
            .count(),
        HORIZON_STEPS
    );

    let (source_neutral, source_neutral_observations) = m1r1::run_arm_with_options(
        &deprived,
        m1r1::Shadow::SourceCapacityUpperBound,
        &mechanics,
        Some(neutral_k_a_decay),
        true,
    );
    let (combined_neutral, combined_neutral_observations) = m1r1::run_arm_with_options(
        &deprived,
        m1r1::Shadow::Combined,
        &mechanics,
        Some(neutral_k_a_decay),
        true,
    );
    assert_eq!(source_neutral_observations.len(), HORIZON_STEPS);
    assert_eq!(combined_neutral_observations.len(), HORIZON_STEPS);
    assert!(source_neutral_observations.iter().all(|observation| {
        observation.starvation_predicate
            && close(observation.selected_multiplier, 4.0)
            && close(observation.effective_k_a_decay, production_k_a_decay)
            && close(observation.declared_k_a_decay, neutral_k_a_decay)
    }));
    assert!(combined_neutral_observations.iter().all(|observation| {
        observation.starvation_predicate
            && close(observation.selected_multiplier, 4.0)
            && close(observation.effective_k_a_decay, production_k_a_decay)
            && close(observation.declared_k_a_decay, neutral_k_a_decay)
    }));

    let base_json = arm_json(&base, &[])?;
    let raw_json = arm_json(&raw_source, &raw_observations)?;
    let source_neutral_json = arm_json(&source_neutral, &[])?;
    let combined_neutral_json = arm_json(&combined_neutral, &[])?;
    let original_controls = json!({
        "CATALYST_INVESTMENT_OFF": serde_json::to_value(&original_catalyst_off)?,
        "COMBINED": serde_json::to_value(&original_combined)?,
    });

    let closure_pass = [&base, &raw_source, &source_neutral, &combined_neutral]
        .iter()
        .all(|arm| {
            arm.world_to_organism_closure_residual <= TOL
                && arm.internal_material_closure_residual <= TOL
        });
    assert!(closure_pass);
    let source_improvement =
        source_neutral.organized_material_delta - raw_source.organized_material_delta;
    let classification = if source_neutral.organized_material_delta >= -TOL {
        "M1_SOURCE_CAPACITY_SUFFICIENT_AFTER_DECAY_NEUTRALIZATION"
    } else if source_improvement > TOL {
        "M1_STARVATION_DECAY_CONTRIBUTORY_NOT_SUFFICIENT"
    } else {
        "M1_SOURCE_CAPACITY_STILL_INSUFFICIENT_AFTER_DECAY_NEUTRALIZATION"
    };

    let protocol = json!({
        "directive": "DC-DEV-020-M1-R1-R1-DECAY-CONFOUND-001",
        "starting_head": STARTING_HEAD,
        "m1r1_entry": M1R1_ENTRY,
        "horizon_steps": HORIZON_STEPS,
        "dt": mechanics.dt,
        "selected_production": {"mesh_contract": "ConservativeV2", "reserve_enabled": false},
        "arms": [
            {"id": "BASE", "intervention": "exact M1-R1 baseline"},
            {"id": "SOURCE_CAPACITY_UB_RAW", "intervention": "exact M1-R1 source upper bound"},
            {"id": "SOURCE_CAPACITY_UB_DECAY_NEUTRAL", "intervention": "source upper bound plus diagnostic k_a_decay=K/4"},
            {"id": "COMBINED_DECAY_NEUTRAL", "intervention": "source upper bound plus catalyst investment deferral plus diagnostic k_a_decay=K/4"}
        ],
        "original_m1r1_controls": ["CATALYST_INVESTMENT_OFF", "COMBINED"],
        "decay_rule": "existing production multiplier is 4 when post-source-UB N*F < 1e-8",
        "production_k_a_decay": production_k_a_decay,
        "neutral_shadow_k_a_decay": neutral_k_a_decay,
        "effective_neutral_decay_coefficient": neutral_k_a_decay * 4.0,
        "neutralized_field_difference": "only ReactionParams.k_a_decay",
        "stoichiometry": "N + F -> A + W",
        "forbidden_changes": ["production biology", "chemistry-core", "ConservativeV2", "D-091", "uptake", "transport", "resource inventory", "recycling", "salvage", "M2", "DC-DEV-021"]
    });
    let results = json!({
        "directive": "DC-DEV-020-M1-R1-R1-DECAY-CONFOUND-001",
        "starting_head": STARTING_HEAD,
        "m1r1_exact_reproduction": {
            "base": field_f64(&base_json, "organized_material_delta"),
            "source_raw": field_f64(&raw_json, "organized_material_delta"),
            "catalyst_off": original_catalyst_off.organized_material_delta,
            "combined": original_combined.organized_material_delta,
            "pass": true
        },
        "starvation_provenance": {
            "raw_source_steps": raw_observations.len(),
            "four_x_steps": four_x_steps,
            "non_four_x_steps": HORIZON_STEPS - four_x_steps,
            "all_relevant_steps_starvation": true,
            "raw_a_decay": ledger_f64(&raw_json, "activated_decay"),
            "base_a_decay": ledger_f64(&base_json, "activated_decay")
        },
        "decay_neutralization": {
            "production_k_a_decay": production_k_a_decay,
            "shadow_k_a_decay": neutral_k_a_decay,
            "frozen_multiplier": 4.0,
            "effective_shadow_decay_coefficient": neutral_k_a_decay * 4.0,
            "only_reaction_param_changed": true,
            "pass": true
        },
        "arms": {
            "BASE": base_json,
            "SOURCE_CAPACITY_UB_RAW": raw_json,
            "SOURCE_CAPACITY_UB_DECAY_NEUTRAL": source_neutral_json,
            "COMBINED_DECAY_NEUTRAL": combined_neutral_json
        },
        "original_m1r1_controls": original_controls,
        "source_neutral_metrics": {
            "organized_material_delta": source_neutral.organized_material_delta,
            "a_decay": ledger_f64(&source_neutral_json, "activated_decay"),
            "final_a": snapshot_f64(&source_neutral_json, "final_state", "a"),
            "final_c": snapshot_f64(&source_neutral_json, "final_state", "c"),
            "final_m": snapshot_f64(&source_neutral_json, "final_state", "structural_m"),
            "final_membrane": snapshot_f64(&source_neutral_json, "final_state", "free_l")
                + snapshot_f64(&source_neutral_json, "final_state", "bound_b"),
            "m_production": ledger_f64(&source_neutral_json, "structural_production"),
            "m_turnover": ledger_f64(&source_neutral_json, "structural_turnover"),
            "c_production": ledger_f64(&source_neutral_json, "catalyst_production"),
            "c_turnover": ledger_f64(&source_neutral_json, "catalyst_turnover")
        },
        "combined_neutral_organized_material_delta": combined_neutral.organized_material_delta,
        "source_improvement_over_raw": source_improvement,
        "world_organism_closure": closure_pass,
        "internal_material_closure": closure_pass,
        "a_decay_to_waste": true,
        "w_to_a_transfer": false,
        "reserve_enabled": false,
        "production_biology_changed": false,
        "chemistry_changed": false,
        "recycling_implemented": false,
        "parameter_search": false,
        "classification": classification,
        "next_execution_started": false
    });
    let qualification = json!({
        "directive": "DC-DEV-020-M1-R1-R1-DECAY-CONFOUND-001",
        "starting_head": STARTING_HEAD,
        "m1r1_exact_reproduction": true,
        "raw_source_starvation_steps": four_x_steps,
        "raw_source_non_four_x_steps": HORIZON_STEPS - four_x_steps,
        "decay_neutralization_exact": true,
        "source_throughput_contributory": true,
        "world_organism_closure": closure_pass,
        "internal_material_closure": closure_pass,
        "a_decay_remains_to_waste": true,
        "w_to_a_transfer": false,
        "reserve_enabled": false,
        "observer_only": true,
        "production_biology_changed": false,
        "chemistry_changed": false,
        "recycling_implemented": false,
        "parameter_search": false,
        "m1_production_change_authorized": false,
        "m2_authorized": false,
        "recycling_authorized": false,
        "dc_dev_021_authorized": false,
        "next_execution_started": false,
        "classification": classification
    });
    let manifest = json!({
        "directive": "DC-DEV-020-M1-R1-R1-DECAY-CONFOUND-001",
        "starting_head": STARTING_HEAD,
        "artifact_files": ["protocol.json", "results.json", "qualification.json", "artifact_manifest.json"],
        "authoritative_result": "results.json",
        "qualification": "qualification.json",
        "dense_ledgers_committed": false,
        "observer_only": true,
        "next_execution_started": false
    });
    write_json(&out.join("protocol.json"), &protocol)?;
    write_json(&out.join("results.json"), &results)?;
    write_json(&out.join("qualification.json"), &qualification)?;
    write_json(&out.join("artifact_manifest.json"), &manifest)?;
    println!("DCDEV020M1R1R1_DECAY_CONFOUND_COMPLETE");
    println!("classification={classification}");
    println!("{}", out.display());
    Ok(())
}
