//! Observer-only reconstruction and qualification for the sealed D-094 record.
//!
//! This module deliberately does not call into, mutate, or reimplement
//! `chemistry-core`.  Its inputs are the mechanism-specific observables that
//! the sealed D-094 artifacts already emitted.  Missing causal endpoints are
//! represented as `None`, never inferred from a clade label or a terminal
//! frequency.

use crate::{
    CampaignRole, FounderIdentityV1, HeredityEvidenceV1, PhenotypeEvidenceV1,
    SelectivePressureContractV1,
};
use serde::{Deserialize, Serialize};

pub const D094_ARCHITECTURE: &str = "autopoietic_material_mesh_autocatalytic_set_v1";
pub const D094_SOURCE_COMMIT: &str = "82bf09d13b1cad3f7734386c8060ad28315cef41";
pub const D094R_SOURCE_COMMIT: &str = "bf58edddef40753107ba18854eb85cc41ec78859";
pub const D094R2_SEALED_RESULT_COMMIT: &str = "935359eea2fcdb08cb1365f58128eaba3f10f3e8";
pub const D094R2_ATTEMPT_MANIFEST_SHA256: &str =
    "e9b03ff268da69b91a8ced053b311cc7e1e0c439c75503dde6e7002207d2e01e";
pub const D094R2_BINARY_SHA256: &str =
    "6c49dd04411cce128ddcb9008d5ecbd4b77afe5da2a7a0cdc3a88f8a4c25f8aa";
pub const D094R2_GENERATIONS: u32 = 8;
pub const D094R2_REPLICATES: u32 = 8;
pub const D094R2_FREQUENCY_THRESHOLD: f64 = 0.15;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum D094FounderArchitecture {
    H,
    B,
    Neutral,
}

impl D094FounderArchitecture {
    pub fn edge_types(self) -> &'static [&'static str] {
        match self {
            Self::H => &["E_AA", "E_AR", "E_RA", "E_RB", "E_BA"],
            Self::B => &["E_BB", "E_BR", "E_RB", "E_RA", "E_AB"],
            Self::Neutral => &["E_AR", "E_RB", "E_BA", "E_RA", "E_BR"],
        }
    }

    pub fn clade_label(self) -> &'static str {
        match self {
            Self::H => "H",
            Self::B => "B",
            Self::Neutral => "N",
        }
    }
}

/// The sealed runner used eight independent single-founder lineages per
/// replicate: four H and four B founders.  Neutral used the same mixed
/// construction under baseline catalytic efficiencies.
pub fn d094r2_founder_identities(replicate: u32) -> Vec<FounderIdentityV1> {
    let mut founders = Vec::with_capacity(8);
    for index in 0..4u64 {
        let h_seed = 100 + u64::from(replicate) * 20 + index;
        founders.push(founder_identity(
            h_seed,
            D094FounderArchitecture::H,
            index as usize,
        ));
        let b_seed = 200 + u64::from(replicate) * 20 + index;
        founders.push(founder_identity(
            b_seed,
            D094FounderArchitecture::B,
            (index + 4) as usize,
        ));
    }
    founders
}

pub fn founder_identity(
    seed: u64,
    architecture: D094FounderArchitecture,
    founder_index: usize,
) -> FounderIdentityV1 {
    let edges = architecture.edge_types().join(",");
    FounderIdentityV1::new(
        seed,
        D094_ARCHITECTURE,
        &format!("d094:{}:edges={edges}:copies=2", architecture.clade_label()),
        &format!("d094:{}:functional_node_channel", architecture.clade_label()),
        "NOT_RECORDED_SEALED_MATERIAL_STATE",
        seed,
        &format!("sealed_d094r2_founder_index_{founder_index}"),
    )
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct D094HeredityObservation {
    pub pairs: u32,
    pub closed_or_present_fraction: f64,
    pub parent_offspring_edge_frequency_correlation: f64,
    pub parent_offspring_network_response_correlation: f64,
    pub id_shuffle_no_effect: bool,
}

pub fn sealed_d094_heredity_observation() -> D094HeredityObservation {
    D094HeredityObservation {
        pairs: 40,
        closed_or_present_fraction: 1.0,
        parent_offspring_edge_frequency_correlation: 0.7261483678362296,
        parent_offspring_network_response_correlation: 0.8717486739032558,
        id_shuffle_no_effect: true,
    }
}

pub fn qualify_d094_heredity(observation: D094HeredityObservation) -> HeredityEvidenceV1 {
    let preserved = observation.pairs >= 40
        && observation.closed_or_present_fraction >= 1.0 - f64::EPSILON
        && observation.parent_offspring_edge_frequency_correlation.is_finite()
        && observation.parent_offspring_edge_frequency_correlation > 0.0
        && observation.parent_offspring_network_response_correlation.is_finite()
        && observation.parent_offspring_network_response_correlation > 0.0
        && observation.id_shuffle_no_effect;
    HeredityEvidenceV1 {
        observable: true,
        preserved,
        comparison_basis: "D-094 Gate 4 physical edge partition and network response".into(),
        metric: "closed_fraction; parent_offspring_edge_frequency_corr; parent_offspring_network_response_corr".into(),
        value: Some(observation.parent_offspring_network_response_correlation),
        qualification: preserved,
        reason: if preserved {
            "sealed D-094 Gate 4 observables satisfy the recorded heredity gate".into()
        } else {
            "sealed D-094 Gate 4 heredity observables do not satisfy the recorded gate".into()
        },
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct D094PhenotypeObservation {
    pub h_ka: f64,
    pub h_kb: f64,
    pub b_ka: f64,
    pub b_kb: f64,
    pub gap_on: f64,
    pub gap_off: f64,
    pub knockout_collapses: bool,
}

pub fn sealed_d094_phenotype_observation() -> D094PhenotypeObservation {
    D094PhenotypeObservation {
        h_ka: 0.43755563269331293,
        h_kb: 0.20052733033949802,
        b_ka: 0.20036003729573765,
        b_kb: 0.43705234668580734,
        gap_on: 0.4737206117438846,
        gap_off: 0.0,
        knockout_collapses: true,
    }
}

pub fn qualify_d094_phenotype(observation: D094PhenotypeObservation) -> PhenotypeEvidenceV1 {
    let expressed = observation.h_ka > observation.h_kb
        && observation.b_kb > observation.b_ka
        && observation.gap_on > observation.gap_off
        && observation.knockout_collapses;
    PhenotypeEvidenceV1 {
        observable: true,
        expressed,
        comparison_basis: "D-094 Gate 5 catalytic node-channel response with node-production knockout".into(),
        metric: "H_ka-H_kb; B_kb-B_ka; gap_on-gap_off; knockout_collapse".into(),
        value: Some(observation.gap_on - observation.gap_off),
        qualification: expressed,
        reason: if expressed {
            "functional catalytic-channel differentiation is present; clade labels are not used".into()
        } else {
            "functional D-094 phenotype evidence is incomplete or non-differential".into()
        },
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum D094SharedResourceClassification {
    TrueSharedFiniteResourceCompetition,
    CommonEnvironmentIndependentResources,
    PartiallyCoupledCompetition,
    Unresolved,
}

pub fn sealed_d094r2_resource_classification() -> D094SharedResourceClassification {
    D094SharedResourceClassification::CommonEnvironmentIndependentResources
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct D094PhenotypeDifferentialAssay {
    pub h_under_h_endpoint: Option<f64>,
    pub b_under_h_endpoint: Option<f64>,
    pub b_under_b_endpoint: Option<f64>,
    pub h_under_b_endpoint: Option<f64>,
    pub h_under_neutral_endpoint: Option<f64>,
    pub b_under_neutral_endpoint: Option<f64>,
}

impl D094PhenotypeDifferentialAssay {
    pub fn sealed_baseline() -> Self {
        Self {
            h_under_h_endpoint: None,
            b_under_h_endpoint: None,
            b_under_b_endpoint: None,
            h_under_b_endpoint: None,
            h_under_neutral_endpoint: None,
            b_under_neutral_endpoint: None,
        }
    }

    pub fn reciprocal_advantage(&self) -> Option<bool> {
        Some(
            self.h_under_h_endpoint?
                > self.b_under_h_endpoint?
                && self.b_under_b_endpoint? > self.h_under_b_endpoint?,
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct D094R2Reanalysis {
    pub h_complete: u32,
    pub b_complete: u32,
    pub neutral_complete: u32,
    pub generation_min: u32,
    pub generation_max: u32,
    pub all_viable: bool,
    pub h_frequency_effect_mean: f64,
    pub b_frequency_effect_mean: f64,
    pub h_descendant_effect_min: f64,
    pub h_descendant_effect_max: f64,
    pub b_descendant_effect_min: f64,
    pub b_descendant_effect_max: f64,
}

pub fn sealed_d094r2_reanalysis() -> D094R2Reanalysis {
    D094R2Reanalysis {
        h_complete: 8,
        b_complete: 8,
        neutral_complete: 8,
        generation_min: 8,
        generation_max: 8,
        all_viable: true,
        h_frequency_effect_mean: -0.0278,
        b_frequency_effect_mean: 0.0083,
        h_descendant_effect_min: -0.0304,
        h_descendant_effect_max: 0.0214,
        b_descendant_effect_min: -0.0463,
        b_descendant_effect_max: 0.0424,
    }
}

pub fn d094_pressure_contract(
    contrast_id: &str,
    treatment_environment: &str,
    neutral_environment: &str,
    role: CampaignRole,
    condition: &str,
) -> SelectivePressureContractV1 {
    SelectivePressureContractV1 {
        schema: "SelectivePressureContractV1".into(),
        contrast_id: contrast_id.into(),
        campaign_role: role,
        treatment_environment: treatment_environment.into(),
        neutral_environment: neutral_environment.into(),
        pressure_event_or_condition: condition.into(),
        pressure_start: 0.0,
        expected_phenotype_dimension: "functional_autocatalytic_node_channel".into(),
    }
}

pub fn d094_clade_can_influence_chemistry() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d094_h_founder_identity() {
        let founder = founder_identity(100, D094FounderArchitecture::H, 0);
        assert_eq!(founder.seed, 100);
        assert!(founder.heredity_hash.contains("E_AA"));
        assert_eq!(founder.organism_schema, D094_ARCHITECTURE);
    }

    #[test]
    fn d094_b_founder_identity() {
        let founder = founder_identity(200, D094FounderArchitecture::B, 4);
        assert_eq!(founder.seed, 200);
        assert!(founder.heredity_hash.contains("E_BB"));
        assert!(founder.heredity_hash.contains("copies=2"));
    }

    #[test]
    fn d094_neutral_identity() {
        let founder = founder_identity(100, D094FounderArchitecture::Neutral, 0);
        assert!(founder.heredity_hash.contains("E_AR"));
        assert_eq!(founder.phenotype_baseline, "d094:N:functional_node_channel");
    }

    #[test]
    fn d094_heredity_evidence_parent_offspring() {
        let evidence = qualify_d094_heredity(sealed_d094_heredity_observation());
        assert!(evidence.observable);
        assert!(evidence.qualification);
        assert!(evidence.metric.contains("edge_frequency"));
        assert!(evidence.metric.contains("network_response"));
    }

    #[test]
    fn d094_phenotype_evidence_is_functional_not_label_only() {
        let evidence = qualify_d094_phenotype(sealed_d094_phenotype_observation());
        assert!(evidence.qualification);
        assert!(evidence.comparison_basis.contains("catalytic"));
        assert!(!evidence.comparison_basis.contains("clade"));
    }

    #[test]
    fn d094_phenotype_differential_requires_causal_endpoint() {
        assert_eq!(D094PhenotypeDifferentialAssay::sealed_baseline().reciprocal_advantage(), None);
    }

    #[test]
    fn d094_clade_has_no_causal_feedback() {
        assert!(!d094_clade_can_influence_chemistry());
    }

    #[test]
    fn d094_shared_resource_classification() {
        assert_eq!(
            sealed_d094r2_resource_classification(),
            D094SharedResourceClassification::CommonEnvironmentIndependentResources
        );
    }

    #[test]
    fn d094_h_ecology_pressure_contract() {
        let protocol = crate::d094r2_protocols()[0].clone();
        let pressure = protocol.selective_pressure.expect("H pressure contract");
        assert_eq!(pressure.campaign_role, CampaignRole::Treatment);
        assert_eq!(pressure.treatment_environment, "d094r2_h_ecology");
        assert!(pressure.pressure_event_or_condition.contains("H pulse"));
    }

    #[test]
    fn d094_b_ecology_pressure_contract() {
        let protocol = crate::d094r2_protocols()[1].clone();
        let pressure = protocol.selective_pressure.expect("B pressure contract");
        assert_eq!(pressure.campaign_role, CampaignRole::Treatment);
        assert_eq!(pressure.treatment_environment, "d094r2_b_ecology");
        assert!(pressure.pressure_event_or_condition.contains("abrasion"));
    }

    #[test]
    fn d094_neutral_pressure_contract() {
        let protocol = crate::d094r2_protocols()[2].clone();
        let pressure = protocol.selective_pressure.expect("neutral pressure contract");
        assert_eq!(pressure.campaign_role, CampaignRole::Neutral);
        assert_eq!(pressure.neutral_environment, "d094r2_neutral_ecology");
        assert_eq!(protocol.replicates, D094R2_REPLICATES);
    }

    #[test]
    fn d094_sealed_protocol_reconstruction() {
        let protocols = crate::d094r2_protocols();
        assert_eq!(protocols.len(), 4);
        for protocol in protocols {
            assert!(protocol.validate().is_ok());
            assert_eq!(protocol.replicates, D094R2_REPLICATES);
            assert_eq!(protocol.maximum_generation, D094R2_GENERATIONS);
            assert!(!protocol.provenance.execution_authorized);
            assert!(protocol.validate_for_execution().is_err());
        }
    }

    #[test]
    fn d094_sealed_artifact_result_reanalysis() {
        let result = sealed_d094r2_reanalysis();
        assert_eq!(result.h_complete, 8);
        assert_eq!(result.b_complete, 8);
        assert_eq!(result.neutral_complete, 8);
        assert_eq!(result.generation_min, 8);
        assert!(result.all_viable);
        assert!(result.h_frequency_effect_mean < 0.0);
        assert!(result.b_frequency_effect_mean < D094R2_FREQUENCY_THRESHOLD);
        assert!(result.h_descendant_effect_min < 0.0 && result.h_descendant_effect_max > 0.0);
        assert!(result.b_descendant_effect_min < 0.0 && result.b_descendant_effect_max > 0.0);
    }
}
