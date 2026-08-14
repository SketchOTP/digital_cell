use crate::{CampaignRole, FailureClass, ReplicateResultV1, SelectivePressureContractV1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CampaignResultV1 {
    pub schema: String,
    pub experiment_id: String,
    pub protocol_hash: String,
    pub control_signature: String,
    pub selective_pressure: Option<SelectivePressureContractV1>,
    pub replicate_results: Vec<ReplicateResultV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectionAnalysisV1 {
    pub schema: String,
    pub treatment_mean: f64,
    pub neutral_mean: f64,
    pub absolute_effect: f64,
    pub relative_effect: f64,
    pub replicate_count: u32,
    pub direction_consistency: f64,
    pub uncertainty_half_width: f64,
    pub interpretable: bool,
    pub classification: FailureClass,
    pub classifications: Vec<FailureClass>,
}

pub trait SelectionObserver {
    fn observe(
        &self,
        treatment: &CampaignResultV1,
        neutral: &CampaignResultV1,
    ) -> SelectionAnalysisV1;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultSelectionObserver;

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

impl SelectionObserver for DefaultSelectionObserver {
    fn observe(
        &self,
        treatment: &CampaignResultV1,
        neutral: &CampaignResultV1,
    ) -> SelectionAnalysisV1 {
        let paired = treatment
            .replicate_results
            .iter()
            .zip(&neutral.replicate_results)
            .collect::<Vec<_>>();
        let mut classifications = treatment
            .replicate_results
            .iter()
            .chain(&neutral.replicate_results)
            .map(|result| result.classification.clone())
            .collect::<Vec<_>>();
        let contract_match = match (&treatment.selective_pressure, &neutral.selective_pressure) {
            (Some(t), Some(n)) => {
                t.contrast_id == n.contrast_id
                    && t.treatment_environment == n.treatment_environment
                    && t.neutral_environment == n.neutral_environment
                    && t.pressure_event_or_condition == n.pressure_event_or_condition
                    && t.expected_phenotype_dimension == n.expected_phenotype_dimension
                    && t.campaign_role == CampaignRole::Treatment
                    && n.campaign_role == CampaignRole::Neutral
            }
            _ => false,
        };
        let valid = contract_match
            && treatment.control_signature == neutral.control_signature
            && treatment.replicate_results.len() == neutral.replicate_results.len()
            && treatment
                .replicate_results
                .iter()
                .map(|result| result.seed)
                .eq(neutral.replicate_results.iter().map(|result| result.seed))
            && !paired.is_empty()
            && paired.iter().all(|(t, n)| {
                t.classification == FailureClass::ReplicateQualified
                    && n.classification == FailureClass::ReplicateQualified
                    && t.actual_reproduction
                    && n.actual_reproduction
                    && t.heredity_preserved
                    && n.heredity_preserved
                    && t.phenotype_measurable
                    && n.phenotype_measurable
                    && t.event_ledger_valid
                    && n.event_ledger_valid
                    && t.environment_supported
                    && n.environment_supported
                    && t.campaign_role == Some(CampaignRole::Treatment)
                    && t.pressure_reached
                    && t.pressure_before_reproduction
                    && n.campaign_role == Some(CampaignRole::Neutral)
                    && n.neutral_comparator_valid
            });
        let treatment_values = treatment
            .replicate_results
            .iter()
            .map(|r| r.birth_count as f64)
            .collect::<Vec<_>>();
        let neutral_values = neutral
            .replicate_results
            .iter()
            .map(|r| r.birth_count as f64)
            .collect::<Vec<_>>();
        let treatment_mean = mean(&treatment_values);
        let neutral_mean = mean(&neutral_values);
        let differences = paired
            .iter()
            .map(|(t, n)| t.birth_count as f64 - n.birth_count as f64)
            .collect::<Vec<_>>();
        let absolute_effect = treatment_mean - neutral_mean;
        let relative_effect = if neutral_mean.abs() > f64::EPSILON {
            absolute_effect / neutral_mean
        } else {
            absolute_effect
        };
        let direction_consistency = if differences.is_empty() {
            0.0
        } else {
            differences.iter().filter(|value| **value > 0.0).count() as f64
                / differences.len() as f64
        };
        let difference_mean = mean(&differences);
        let variance = if differences.len() > 1 {
            differences
                .iter()
                .map(|value| (value - difference_mean).powi(2))
                .sum::<f64>()
                / (differences.len() - 1) as f64
        } else {
            0.0
        };
        let uncertainty_half_width = if differences.is_empty() {
            0.0
        } else {
            1.96 * (variance / differences.len() as f64).sqrt()
        };
        let classification = if !valid {
            classifications.push(FailureClass::NeutralComparatorInvalid);
            FailureClass::NeutralComparatorInvalid
        } else if absolute_effect.abs() <= uncertainty_half_width {
            FailureClass::ValidNoSelectionEffect
        } else {
            FailureClass::ValidSelectionEffect
        };
        SelectionAnalysisV1 {
            schema: "SelectionAnalysisV1".into(),
            treatment_mean,
            neutral_mean,
            absolute_effect,
            relative_effect,
            replicate_count: paired.len() as u32,
            direction_consistency,
            uncertainty_half_width,
            interpretable: valid,
            classification,
            classifications,
        }
    }
}
