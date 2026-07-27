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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartitionSummary {
    pub observations: usize,
    pub mean_network_displacement: f64,
    pub mean_phenotype_displacement: f64,
    pub mutation_variance: f64,
    pub partition_variance: f64,
    pub pre_partition_covariance: f64,
    pub post_partition_covariance: f64,
    pub conditioned_trait_descendant_covariance: f64,
    pub high_parent_loss_rate: f64,
    pub destroys_phenotype: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CausalReplaySummary {
    pub physiology_differs: bool,
    pub growth_or_survival_differs: bool,
    #[serde(default)]
    pub environment_interaction_present: bool,
}

pub fn final_causal_classification(
    partition: &PartitionSummary,
    replay: &CausalReplaySummary,
) -> (&'static str, Option<&'static str>) {
    if partition.destroys_phenotype {
        ("PARTITION_NOISE_ERASES_SELECTION", None)
    } else if !replay.physiology_differs {
        ("PHENOTYPE_NOT_COUPLED_TO_CONSERVED_PHYSIOLOGY", None)
    } else if !replay.growth_or_survival_differs {
        ("PHYSIOLOGICAL_EFFECT_BUFFERED_BEFORE_FITNESS", None)
    } else if !replay.environment_interaction_present {
        (
            "ENVIRONMENT_PHENOTYPE_INTERACTION_ABSENT",
            Some("DEMOGRAPHIC_NOISE_DOMINATES_WEAK_DESCENDANT_DIFFERENCES"),
        )
    } else {
        ("SELECTION_COUPLING_PRESENT", None)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalEnvironment {
    pub nutrient_mean: f64,
    pub fuel_mean: f64,
    pub resource_timing_variance: f64,
    pub damage_per_350_steps: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentalContrast {
    pub h: LocalEnvironment,
    pub b: LocalEnvironment,
    pub neutral: LocalEnvironment,
    pub mechanistically_selectable: bool,
    pub equations_contain_environment_labels: bool,
    pub causal_pathways: Vec<String>,
}

/// Reconstructs the local fields applied by the sealed replay contract. The H/B
/// names identify records only; no label is an input to the proposed physiology.
pub fn environmental_contrast() -> EnvironmentalContrast {
    let h_high: f64 = 2.2 * 1.25;
    let h_low: f64 = 2.2 * 0.12;
    let h_mean = 0.25 * h_high + 0.75 * h_low;
    let h_variance =
        0.25 * (h_high - h_mean).powi(2) + 0.75 * (h_low - h_mean).powi(2);
    EnvironmentalContrast {
        h: LocalEnvironment {
            nutrient_mean: h_mean,
            fuel_mean: h_mean,
            resource_timing_variance: h_variance,
            damage_per_350_steps: 0.0,
        },
        b: LocalEnvironment {
            nutrient_mean: 2.2 * 0.90,
            fuel_mean: 2.2 * 0.90,
            resource_timing_variance: 0.0,
            damage_per_350_steps: 0.08 + 0.048,
        },
        neutral: LocalEnvironment {
            nutrient_mean: 2.2 * 0.70,
            fuel_mean: 2.2 * 0.70,
            resource_timing_variance: 0.0,
            damage_per_350_steps: 0.0,
        },
        mechanistically_selectable: true,
        equations_contain_environment_labels: false,
        causal_pathways: vec![
            "resource pulses -> processing/activation allocation -> reserve and growth".into(),
            "structural/membrane damage -> repair allocation -> retained mass and readiness".into(),
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Allocation {
    /// Resource processing, activation, repair, and growth synthesis.
    pub fractions: [f64; 4],
}

impl Allocation {
    pub fn new(fractions: [f64; 4]) -> Result<Self, String> {
        if fractions.iter().any(|x| !x.is_finite() || *x < 0.0 || *x > 1.0)
            || (fractions.iter().sum::<f64>() - 1.0).abs() > 1e-12
        {
            return Err("allocation fractions must be finite, bounded, and sum to one".into());
        }
        Ok(Self { fractions })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReciprocalInteraction {
    pub interaction_h: f64,
    pub interaction_b: f64,
    pub reciprocal: bool,
    pub universally_superior: bool,
}

/// Inputs are predicted benefits in [H, B, neutral] for two associated
/// allocations. This observer has no access to organism mutation or reproduction.
pub fn reciprocal_interaction(
    h_allocation: [f64; 3],
    b_allocation: [f64; 3],
) -> ReciprocalInteraction {
    let interaction_h = h_allocation[0] - h_allocation[2];
    let interaction_b = b_allocation[1] - b_allocation[2];
    let h_cross_cost = h_allocation[0] > b_allocation[0]
        && h_allocation[1] < b_allocation[1]
        && h_allocation[2] <= b_allocation[2];
    let b_cross_cost = b_allocation[1] > h_allocation[1]
        && b_allocation[0] < h_allocation[0]
        && b_allocation[2] <= h_allocation[2];
    let h_dominates = (0..3).all(|i| h_allocation[i] > b_allocation[i]);
    let b_dominates = (0..3).all(|i| b_allocation[i] > h_allocation[i]);
    ReciprocalInteraction {
        interaction_h,
        interaction_b,
        reciprocal: interaction_h > 0.0 && interaction_b > 0.0 && h_cross_cost && b_cross_cost,
        universally_superior: h_dominates || b_dominates,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateScore {
    pub candidate: String,
    pub scores: [u8; 10],
    pub total: u8,
    pub eligible: bool,
    pub selected: bool,
    pub reason: String,
}

fn scored(candidate: &str, scores: [u8; 10], selected: bool, reason: &str) -> CandidateScore {
    let eligible = scores[0] == 2
        && scores[1] == 2
        && scores[2] >= 1
        && scores[3] == 2
        && scores[4] >= 1
        && candidate != "A";
    CandidateScore {
        candidate: candidate.into(),
        total: scores.iter().sum(),
        scores,
        eligible,
        selected: selected && eligible,
        reason: reason.into(),
    }
}

/// Deterministic observer-only review. Score order follows the D-095C table.
pub fn candidate_review() -> Vec<CandidateScore> {
    vec![
        scored(
            "A",
            [2, 0, 0, 2, 2, 2, 2, 2, 2, 2],
            false,
            "sealed control lacks a finite tradeoff and treatment specificity",
        ),
        scored(
            "B",
            [2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
            true,
            "static finite allocation maps pulse utilization against damage repair",
        ),
        scored(
            "C",
            [2, 2, 1, 1, 1, 2, 2, 1, 1, 2],
            false,
            "contrast is not fundamentally transport-limited and Phase-1 boundary impact is larger",
        ),
        scored(
            "D",
            [2, 2, 2, 1, 1, 1, 2, 2, 0, 1],
            false,
            "static allocation is sufficient; regulation adds an unproved substrate and delay cost",
        ),
    ]
}

pub fn select_route(scores: &[CandidateScore]) -> Option<String> {
    let selected = scores
        .iter()
        .filter(|score| score.selected && score.eligible)
        .collect::<Vec<_>>();
    (selected.len() == 1).then(|| selected[0].candidate.clone())
}

pub fn freeze_d096_contract(route: Option<&str>) -> Option<Value> {
    (route == Some("B")).then(|| {
        json!({
            "status": "FROZEN",
            "selected_candidate": "B",
            "scientific_hypothesis": "Inherited finite catalytic allocations favor pulse processing under temporally concentrated N/F and favor repair under recurrent local damage, causing reciprocal descendant advantage.",
            "equation_identity": "autopoietic_material_mesh_finite_catalytic_allocation_v1",
            "hereditary_representation": {
                "encoded_values": ["resource_processing", "activation", "repair", "growth_synthesis"],
                "bounds": "[0,1] each; exact normalized sum=1",
                "initialization": {
                    "pulse_specialist": [0.45, 0.25, 0.10, 0.20],
                    "damage_specialist": [0.20, 0.20, 0.45, 0.15],
                    "neutral": [0.25, 0.25, 0.25, 0.25]
                },
                "mutation": "At each qualified copy, probability 0.01; choose ordered source!=target uniformly; delta=min(abs(N(0,0.15)), source, 1-target); subtract delta from source and add it to target.",
                "copying": "copy the ordered four-coordinate vector exactly before the bounded mutation operator",
                "fission_partition": "same conservative network partition operator qualified by D-095",
                "identity_hash": "canonical ordered IEEE-754 allocation bytes plus equation identity"
            },
            "conserved_expression": {
                "shared_synthesis_flux": "J_syn = 1e-3 * min(M_local, A_local/0.2) per accepted step",
                "allocation": "J_i = alpha_i * J_syn",
                "material": "Delta M = -sum_i(J_i)*dt",
                "activated_resource": "Delta A_synthesis = -0.2*sum_i(J_i)*dt; Delta A_maintenance = -1e-5*sum_i(C_i)*dt",
                "synthesis_time": "Delta C_i = (J_i - 1e-4*C_i)*dt",
                "turnover": "J_turn_i = 1e-4*C_i",
                "waste": "Delta W = sum_i(J_turn_i)*dt",
                "allocation_conservation": "sum_i allocation_i = 1"
            },
            "mandatory_tradeoff": "Increasing any allocation coordinate decreases at least one other coordinate under a fixed total catalyst-production budget.",
            "environmental_coupling": {
                "inputs": ["local nutrient", "local fuel", "local activated resource", "local reserve", "local structural damage", "local membrane damage"],
                "environment_labels_permitted": false,
                "flux_multiplier": "g_i = 1 + C_i/(0.1 + C_i); multiply only the corresponding existing local physiological flux",
                "mapping": {
                    "resource_processing": "local nutrient/fuel processing flux",
                    "activation": "local activated-resource production flux",
                    "repair": "existing structural and membrane repair flux",
                    "growth_synthesis": "existing reserve-funded structural synthesis flux"
                }
            },
            "gates": [
                "Gate 0 Preservation and schema",
                "Gate 1 Conservation and invariant domain",
                "Gate 2 Local expression identification",
                "Gate 3 Mandatory tradeoff",
                "Gate 4 Environmental input observability",
                "Gate 5 Reciprocal pre-fission physiological effect",
                "Gate 6 Heredity and mutation continuity",
                "Gate 7 Single-generation fitness consequence",
                "Gate 8 Multi-generation selection",
                "Gate 9 Adaptation",
                "Gate 10 Environmental reversal"
            ],
            "stop_rule": "No later gate runs after an earlier failure.",
            "phase_authority": "Phase 3 remains unauthorized until Gates 8, 9, and 10 pass.",
            "phase3_authorized": false,
            "implementation_status": "NOT_IMPLEMENTED"
        })
    })
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
