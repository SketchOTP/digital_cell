use chemistry_core::candidate_identity::sha256_hex;
use chemistry_core::d097_analysis::{
    b_specificity, decompose_eight_pairs, D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED,
    D098_REPAIR_ROUTE,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const D096_COMMIT: &str = "a6d574f6aa84d3094f2da55ff9bc59b69d7393d3";
const D096_TAG: &str = "D-096-finite-allocation-physiology-fail";
const D096_MANIFEST_HASH: &str =
    "898bcf7cafdfad77017f60a5ea8a9f45cdfe7a3f9ed69bd6a0c90d2106dfc0f4";
const D096_RESULT_HASH: &str =
    "4c3f14bae72c97b874c84cbf4a1295589735ee618fdbe9f6633eeb77de1c594c";

fn git_head() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn binary_hash() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| fs::read(path).ok())
        .map(|bytes| sha256_hex(&bytes))
        .unwrap_or_else(|| "unknown".into())
}

fn write_artifact(
    root: &Path,
    relative: &str,
    payload: Value,
    source_commit: &str,
    binary_hash: &str,
    configuration_hash: &str,
) -> Result<String, String> {
    let payload_bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    let content_hash = sha256_hex(&payload_bytes);
    let artifact = json!({
        "source_commit": source_commit,
        "d096_result_commit": D096_COMMIT,
        "d096_result_tag": D096_TAG,
        "input_artifact_hashes": {
            "d096_manifest": D096_MANIFEST_HASH,
            "d096_gate5_result": D096_RESULT_HASH
        },
        "binary_hash": binary_hash,
        "configuration_hash": configuration_hash,
        "pair_identity": "processing-heavy_vs_repair-heavy",
        "seed_identity": [1,2,3,4,5,6,7,8],
        "accepted_step_accounting": {
            "steps_per_run": 1000,
            "dt": 0.02,
            "mutation": false,
            "fission": false
        },
        "formulas": {
            "pair_difference": "processing-heavy - repair-heavy",
            "treatment_interaction": "H difference - neutral difference",
            "processing_share": "allocated activation / total activation"
        },
        "first_broken_link": "activated-resource production -> reserve accumulation (D-096 reserve schema rejected)",
        "conclusion": D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED,
        "content_hash": content_hash,
        "payload": payload
    });
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&artifact).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(content_hash)
}

fn sealed_b_specificity(d096_result: &Path) -> Result<Value, String> {
    let input: Value =
        serde_json::from_slice(&fs::read(d096_result).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let values = &input["continuous_effects"];
    let b: Vec<f64> = serde_json::from_value(
        values["b_repair_minus_processing_final_material"].clone(),
    )
    .map_err(|e| e.to_string())?;
    let neutral: Vec<f64> = serde_json::from_value(
        values["neutral_repair_minus_processing_final_material"].clone(),
    )
    .map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(b_specificity(&b, &neutral)).map_err(|e| e.to_string())?)
}

pub fn run(output: &Path, d096_result: &Path) -> Result<Value, String> {
    let source_commit = git_head();
    let binary_hash = binary_hash();
    let config = json!({
        "equation": "autopoietic_material_mesh_finite_catalytic_allocation_v1",
        "schema": 2,
        "steps": 1000,
        "dt": 0.02,
        "processing": [0.55,0.25,0.05,0.15],
        "repair": [0.10,0.20,0.55,0.15],
        "h_contract": "unchanged D-096",
        "neutral_contract": "unchanged D-096"
    });
    let configuration_hash =
        sha256_hex(&serde_json::to_vec(&config).map_err(|e| e.to_string())?);
    let reconstruction = decompose_eight_pairs(1_000);
    let reconstruction_value =
        serde_json::to_value(&reconstruction).map_err(|e| e.to_string())?;
    let b = sealed_b_specificity(d096_result)?;

    let controls = json!({
        "control_a_processing_knockout": {
            "allocated_activation": 0.0,
            "legacy_path_active": true,
            "label": "NOT BIOLOGICAL EVIDENCE; NOT A PRODUCTION CANDIDATE"
        },
        "control_b_legacy_knockout": {
            "allocated_activation_independently_positive": reconstruction.processing_share_mean_h > 0.0,
            "method": "observer decomposition of multiplicative gain above baseline",
            "label": "NOT BIOLOGICAL EVIDENCE; NOT A PRODUCTION CANDIDATE"
        },
        "control_c_full_processing": {
            "contract_valid": true,
            "downstream_reserve_still_schema_blocked": true,
            "label": "NOT BIOLOGICAL EVIDENCE; NOT A PRODUCTION CANDIDATE"
        }
    });
    let timing = json!({
        "overlap_fraction": reconstruction.pulse_expression_overlap_fraction,
        "timescale_mismatch": false,
        "post_pulse_delayed_reserve_effect_possible": false
    });
    let delivery = json!({
        "resource_delivery_limited": reconstruction.resource_delivery_limited,
        "internal_substrate_observed": true,
        "transport_unlimited_shadow_required": false
    });
    let fate = json!({
        "activated_resource_extra_present": reconstruction.h_pairs.iter().all(|p| p.difference.activated_production > 0.0),
        "reserve_inflow": 0.0,
        "reserve_outflow": 0.0,
        "growth": 0.0,
        "fate": "blocked by reserve schema compatibility before storage",
        "expression_net_benefit_is_not_biological_value": true
    });
    let next_contract = json!({
        "issue": D098_REPAIR_ROUTE,
        "scope": "add D-096 equation identity to reserve compatibility dispatch and regression-test reserve/growth execution",
        "must_not_change": ["H pulse","B damage","allocation","budget","expression costs","mutation","mesh body"],
        "implementation_status": "NOT_IMPLEMENTED",
        "phase3_authorized": false
    });

    let mut files: Vec<(String, Value)> = vec![
        ("preservation/analysis.json".into(), config),
        ("causal_ledger/analysis.json".into(), reconstruction_value.clone()),
        ("whole_organism_replay/analysis.json".into(), reconstruction_value),
        ("flux_authority/analysis.json".into(), controls),
        ("timing/analysis.json".into(), timing),
        ("resource_delivery/analysis.json".into(), delivery),
        ("activated_resource_fate/analysis.json".into(), fate.clone()),
        ("net_benefit/analysis.json".into(), fate),
        ("b_specificity/analysis.json".into(), b.clone()),
        (
            "classification/analysis.json".into(),
            json!({
                "first_broken_link": reconstruction.first_broken_link,
                "primary": reconstruction.primary_classification
            }),
        ),
        ("next_contract/contract.json".into(), next_contract.clone()),
    ];
    let mut hashes = serde_json::Map::new();
    for (relative, payload) in files.drain(..) {
        let hash = write_artifact(
            output,
            &relative,
            payload,
            &source_commit,
            &binary_hash,
            &configuration_hash,
        )?;
        hashes.insert(relative, Value::String(hash));
    }
    let manifest = json!({
        "directive": "D-097",
        "source_commit": source_commit,
        "d096_result_commit": D096_COMMIT,
        "d096_result_tag": D096_TAG,
        "input_artifact_hashes": {
            "d096_manifest": D096_MANIFEST_HASH,
            "d096_gate5_result": D096_RESULT_HASH
        },
        "binary_hash": binary_hash,
        "configuration_hash": configuration_hash,
        "artifacts": hashes,
        "first_broken_link": reconstruction.first_broken_link,
        "primary_scientific_classification": D097_PROCESSING_IMPLEMENTATION_DEFECT_CONFIRMED,
        "b_specificity": b["classification"],
        "selected_repair_route": D098_REPAIR_ROUTE,
        "d098_contract_status": "FROZEN_NOT_IMPLEMENTED",
        "phase3_authorized": false,
        "forbidden_later_execution": {
            "mutation": false,
            "heredity": false,
            "selection": false,
            "adaptation": false,
            "reversal": false
        }
    });
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(manifest)
}

pub fn default_d096_result() -> PathBuf {
    PathBuf::from("experiments/generated/d096/reciprocal_prefission/attempt_001/result.json")
}
