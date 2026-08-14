use crate::{FailureClass, ReplicateResultV1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectionAnalysisV1 {
    pub schema: String,
    pub treatment_mean: f64,
    pub neutral_mean: f64,
    pub difference: f64,
    pub relative_effect: f64,
    pub replicate_count: u32,
    pub direction_consistency: f64,
    pub interpretable: bool,
    pub classifications: Vec<FailureClass>,
}

pub trait SelectionObserver {
    fn observe(&self, result: &ReplicateResultV1) -> SelectionAnalysisV1;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultSelectionObserver;

impl SelectionObserver for DefaultSelectionObserver {
    fn observe(&self, result: &ReplicateResultV1) -> SelectionAnalysisV1 {
        let interpretable = matches!(
            result.classification.clone(),
            FailureClass::ValidNoSelectionEffect | FailureClass::ValidSelectionEffect
        );
        SelectionAnalysisV1 {
            schema: "SelectionAnalysisV1".into(),
            treatment_mean: result.birth_count as f64,
            neutral_mean: result.birth_count as f64,
            difference: 0.0,
            relative_effect: 0.0,
            replicate_count: 1,
            direction_consistency: 0.0,
            interpretable,
            classifications: vec![result.classification.clone()],
        }
    }
}
