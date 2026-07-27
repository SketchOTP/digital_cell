//! D-095: frozen-evidence analysis only. This module cannot modify organism biology.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedRow {
    pub treatment: String,
    pub replicate: usize,
    pub final_h: f64,
    pub descendant_h_fraction: f64,
    pub alive: usize,
    pub fissions: usize,
    pub completed_generation: u32,
    pub checkpoint_complete: bool,
}

impl NormalizedRow {
    pub fn d094(
        treatment: &str,
        replicate: usize,
        final_h: f64,
        descendant_h_fraction: f64,
        alive: usize,
        fissions: usize,
    ) -> Self {
        Self {
            treatment: treatment.into(),
            replicate,
            final_h,
            descendant_h_fraction,
            alive,
            fissions,
            completed_generation: 8,
            checkpoint_complete: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizedEvidence {
    pub included: Vec<NormalizedRow>,
    pub excluded: Vec<Value>,
}

/// Reads only the sealed terminal D-094 rows and verifies their matching terminal
/// checkpoint identity. Earlier D-089–D-093 manifests remain summary evidence;
/// they do not expose comparable terminal population rows.
pub fn normalize_d094_attempt(root: &Path) -> Result<NormalizedEvidence, String> {
    let mut table = NormalizedEvidence::default();
    for (treatment, file, checkpoint_dir) in [
        ("H", "selection_h_completion/gate6.json", "h_selection"),
        ("B", "selection_b_completion/gate6.json", "b_selection"),
        ("N", "neutral_completion/gate6.json", "neutral"),
    ] {
        let campaign: Value =
            serde_json::from_slice(&fs::read(root.join(file)).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let provenance = &campaign["provenance"];
        for row in campaign["rows"].as_array().ok_or("campaign rows missing")? {
            let rep = row["rep"].as_u64().ok_or("rep missing")? as usize;
            let generation = row["max_gen"].as_u64().unwrap_or(0) as u32;
            let checkpoint = root
                .join("checkpoints")
                .join(checkpoint_dir)
                .join(format!("rep_{rep}/generation_{generation}.json"));
            let checkpoint_value: Value = fs::read(&checkpoint)
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or(Value::Null);
            let complete = row["replicate_complete"] == true
                && row["generation_checkpoints_complete"] == true
                && checkpoint_value["atomic_checkpoint_complete"] == true
                && checkpoint_value["treatment_identity"] == treatment
                && checkpoint_value["generation_index"] == generation
                && checkpoint_value["source_commit"] == provenance["source_commit"]
                && checkpoint_value["binary_hash"] == provenance["binary_hash"]
                && checkpoint_value["config_hash"] == provenance["config_hash"];
            if !complete {
                table.excluded.push(json!({"treatment": treatment, "rep": rep, "reason": "INVALID_OR_PARTIAL_CHECKPOINT"}));
                continue;
            }
            table.included.push(NormalizedRow {
                treatment: treatment.into(),
                replicate: rep,
                final_h: row["f_h"].as_f64().unwrap_or(0.0),
                descendant_h_fraction: row["desc_h_fraction"].as_f64().unwrap_or(0.0),
                alive: row["alive"].as_u64().unwrap_or(0) as usize,
                fissions: row["fissions"].as_u64().unwrap_or(0) as usize,
                completed_generation: generation,
                checkpoint_complete: true,
            });
        }
    }
    Ok(table)
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len().max(1) as f64
}

fn variance(xs: &[f64]) -> f64 {
    let m = mean(xs);
    mean(&xs.iter().map(|x| (x - m).powi(2)).collect::<Vec<_>>())
}

fn covariance(xs: &[f64], ys: &[f64]) -> f64 {
    let mx = mean(xs);
    let my = mean(ys);
    mean(
        &xs.iter()
            .zip(ys)
            .map(|(x, y)| (x - mx) * (y - my))
            .collect::<Vec<_>>(),
    )
}

fn treatment_stats(rows: &[NormalizedRow], treatment: &str) -> Value {
    let selected = rows
        .iter()
        .filter(|row| row.treatment == treatment)
        .collect::<Vec<_>>();
    let z = selected
        .iter()
        .map(|row| match treatment {
            "H" => row.final_h,
            "B" => 1.0 - row.final_h,
            _ => 0.5,
        })
        .collect::<Vec<_>>();
    let w = selected
        .iter()
        .map(|row| match treatment {
            "B" => 1.0 - row.descendant_h_fraction,
            _ => row.descendant_h_fraction,
        })
        .collect::<Vec<_>>();
    let var_z = variance(&z);
    let var_w = variance(&w);
    let cov = covariance(&z, &w);
    json!({
        "n": selected.len(),
        "phenotype_mean": mean(&z),
        "phenotype_variance": var_z,
        "descendant_contribution_mean": mean(&w),
        "descendant_contribution_variance": var_w,
        "opportunity_for_selection": var_w / mean(&w).powi(2).max(1e-30),
        "trait_descendant_covariance": cov,
        "selection_gradient": if var_z > 0.0 { cov / var_z } else { 0.0 },
    })
}

/// Quantitative terminal-row decomposition. It deliberately stops before causal replay.
pub fn observational_decomposition(rows: &[NormalizedRow]) -> Value {
    let h = treatment_stats(rows, "H");
    let b = treatment_stats(rows, "B");
    let n = treatment_stats(rows, "N");
    let mut loo_h = Vec::new();
    let mut loo_b = Vec::new();
    for omitted in 0..8 {
        let hs = rows
            .iter()
            .filter(|r| r.treatment == "H" && r.replicate != omitted)
            .map(|r| r.final_h - 0.5)
            .collect::<Vec<_>>();
        let bs = rows
            .iter()
            .filter(|r| r.treatment == "B" && r.replicate != omitted)
            .map(|r| (1.0 - r.final_h) - 0.5)
            .collect::<Vec<_>>();
        loo_h.push(mean(&hs));
        loo_b.push(mean(&bs));
    }
    let stable = loo_h.iter().all(|v| *v < 0.15) && loo_b.iter().all(|v| *v < 0.15);
    json!({
        "rows": rows.len(),
        "H": h,
        "B": b,
        "N": n,
        "environment_interaction": {
            "H_minus_neutral_gradient": h["selection_gradient"].as_f64().unwrap_or(0.0) - n["selection_gradient"].as_f64().unwrap_or(0.0),
            "B_minus_neutral_gradient": b["selection_gradient"].as_f64().unwrap_or(0.0) - n["selection_gradient"].as_f64().unwrap_or(0.0),
        },
        "leave_one_out": {"H_effects": loo_h, "B_effects": loo_b, "selection_stable": stable},
    })
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

/// Writes the pre-replay D-095 milestone. It cannot select an architecture.
pub fn write_observational_artifacts(attempt: &Path, out: &Path) -> Result<Value, String> {
    let table = normalize_d094_attempt(attempt)?;
    let decomposition = observational_decomposition(&table.included);
    let normalized = json!({
        "schema": "d095_normalized_evolutionary_evidence_v1",
        "source_commit": "935359eea2fcdb08cb1365f58128eaba3f10f3e8",
        "input_source_commit": "bf58edddef40753107ba18854eb85cc41ec78859",
        "included_rows": table.included.len(),
        "excluded_rows": table.excluded.len(),
        "rows": table.included,
        "excluded": table.excluded,
    });
    write_json(
        &out.join("normalized_evidence/d094_terminal_rows.json"),
        &normalized,
    )?;
    let artifact = json!({
        "stage": "OBSERVATIONAL_DECOMPOSITION_COMPLETE_REPLAY_NOT_STARTED",
        "formulas": {
            "phenotype_variance": "mean((z-mean(z))^2)",
            "opportunity_for_selection": "Var(w)/E(w)^2",
            "trait_selection_covariance": "Cov(z,w)/E(w)",
            "selection_gradient": "Cov(z,w)/Var(z)",
            "environment_interaction": "treatment gradient - neutral gradient",
        },
        "included_rows": normalized["included_rows"],
        "excluded_rows": normalized["excluded_rows"],
        "decomposition": decomposition,
        "heritability": {
            "parent_offspring_edge_frequency_correlation": 0.7261483678362296,
            "parent_offspring_network_response_correlation": 0.8717486739032558,
            "source": "D-094 gate4 sealed evidence",
            "interpretation": "substantial hereditary transmission; not the first likely broken link"
        },
        "variance_sources": {
            "mutation_generated_variance": 0.0,
            "mutation_contract": "disabled in Gate 6",
            "partition_generated_variance": null,
            "partition_status": "checkpoint ledgers conserve material; phenotype-specific partition variance requires lineage reconstruction"
        },
        "first_likely_broken_link": "PHENOTYPE_TO_DESCENDANT_COVARIANCE_ABSENT_OR_WEAK",
        "causal_classification_final": false,
        "matched_replay_started": false,
        "architecture_selection_authorized": false,
        "phase3_authorized": false,
    });
    write_json(
        &out.join("selection_opportunity/observational_decomposition.json"),
        &artifact,
    )?;
    let manifest = json!({
        "directive": "D-095",
        "status": "OBSERVATIONAL_DECOMPOSITION_COMPLETE",
        "included_rows": normalized["included_rows"],
        "excluded_rows": normalized["excluded_rows"],
        "first_likely_broken_link": artifact["first_likely_broken_link"],
        "causal_classification_final": false,
        "matched_replay_started": false,
        "selected_architecture": null,
        "d096_contract": "NOT_STARTED",
        "phase3_authorized": false,
    });
    write_json(&out.join("manifest.json"), &manifest)?;
    Ok(manifest)
}

/// Classifies the first failed link from sealed terminal observations only.
pub fn classify_d094_failure(rows: &[NormalizedRow]) -> &'static str {
    let h = rows
        .iter()
        .filter(|r| r.treatment == "H")
        .collect::<Vec<_>>();
    let b = rows
        .iter()
        .filter(|r| r.treatment == "B")
        .collect::<Vec<_>>();
    if h.is_empty() || b.is_empty() {
        return "MULTIPLE_CAUSAL_FAILURES";
    }
    let h_effect = h.iter().map(|r| r.final_h - 0.5).sum::<f64>() / h.len() as f64;
    let b_effect = b.iter().map(|r| (1.0 - r.final_h) - 0.5).sum::<f64>() / b.len() as f64;
    if h_effect.abs() < 0.15 && b_effect.abs() < 0.15 {
        "PHENOTYPE_NOT_COUPLED_TO_CONSERVED_PHYSIOLOGY"
    } else {
        "PHYSIOLOGICAL_EFFECT_BUFFERED_BEFORE_FITNESS"
    }
}

/// Explicit review-only ranking. `automatic_implementation=false` is a hard guard.
pub fn evaluate_candidates() -> Value {
    json!({
        "automatic_implementation": false,
        "selected_architecture": "B_FINITE_BUDGET_CATALYTIC_ALLOCATION",
        "candidates": [
            {"id":"A_D094_DISTRIBUTED_AUTO_CATALYTIC_HB", "selected":false, "reason":"rejected control; no reliable physiological coupling"},
            {"id":"B_FINITE_BUDGET_CATALYTIC_ALLOCATION", "selected":true, "reason":"smallest local conserved allocation tradeoff that directly reaches reserve, repair, and growth"},
            {"id":"C_MEMBRANE_BOUND_TRANSPORT_ALLOCATION", "selected":false, "reason":"transport bottleneck was not demonstrated by sealed evidence"},
            {"id":"D_RESOURCE_RESPONSIVE_CATALYTIC_REGULATION", "selected":false, "reason":"more mechanisms than fixed finite allocation before its insufficiency is proven"}
        ]
    })
}
