use chemistry_core::d094_selection::{
    hard_blocked_downstream_gates, paired_effect_summary, provenance_is_complete,
};
use serde_json::json;

#[test]
fn paired_effects_are_seed_matched_and_deterministic() {
    let treatment = json!({"rows": [
        {"rep": 3, "f_h": 0.75, "desc_h_fraction": 0.80},
        {"rep": 4, "f_h": 0.60, "desc_h_fraction": 0.55}
    ]});
    let neutral = json!({"rows": [
        {"rep": 3, "f_h": 0.50, "desc_h_fraction": 0.50},
        {"rep": 4, "f_h": 0.55, "desc_h_fraction": 0.50}
    ]});
    let effects = paired_effect_summary(&treatment, &neutral, "f_h", "desc_h_fraction");
    assert_eq!(effects["paired_replicates"].as_array().unwrap().len(), 2);
    assert!((effects["frequency"]["mean"].as_f64().unwrap() - 0.15).abs() < 1e-12);
    assert!((effects["descendant_contribution"]["mean"].as_f64().unwrap() - 0.175).abs() < 1e-12);
}

#[test]
fn provenance_requires_identity_and_atomic_generation_checkpoints() {
    let valid = json!({"provenance": {
        "source_commit": "abc123",
        "binary_hash": "bin",
        "config_hash": "cfg",
        "atomic_generation_checkpoints": true,
        "lineage_ledger_complete": true
    }});
    assert!(provenance_is_complete(&valid));
    let invalid = json!({"provenance": {
        "source_commit": "UNCOMMITTED",
        "binary_hash": "bin",
        "config_hash": "cfg",
        "atomic_generation_checkpoints": true,
        "lineage_ledger_complete": true
    }});
    assert!(!provenance_is_complete(&invalid));
}

#[test]
fn gates_seven_and_eight_are_hard_blocked() {
    let blocked = hard_blocked_downstream_gates();
    assert_eq!(blocked["blocked"], true);
    assert_eq!(blocked["status"], "NOT_EXECUTED");
}
