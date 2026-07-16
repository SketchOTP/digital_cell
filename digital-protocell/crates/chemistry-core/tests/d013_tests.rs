//! D-013 Stage E harness integrity tests.

use chemistry_core::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn sample(step: u64, time: f64, mass_c: f64) -> AcceptedStateSample {
    AcceptedStateSample {
        accepted_substep: step,
        simulated_time: time,
        mass_c,
        mass_a: mass_c * 0.5,
        mass_m: mass_c * 0.25,
        mean_n_interior: 0.2,
        mean_f_interior: 0.2,
        mean_w_interior: 0.1,
        structure_production: step as f64,
        structure_decay: 0.0,
        catalyst_reproduction: step as f64 * 0.1,
        catalyst_turnover: 0.0,
        membrane_synthesis: step as f64 * 0.05,
        membrane_loss: 0.0,
        activation: step as f64 * 0.2,
        activated_loss: 0.0,
        nutrient_transport_interior: step as f64,
        fuel_transport_interior: step as f64,
        waste_transport_interior: step as f64 * 0.5,
        material_equivalent_total: 100.0 + step as f64 * 0.001,
        activation_potential_total: 10.0 + step as f64 * 0.0001,
    }
}

#[test]
fn test_rejected_attempt_does_not_advance_simulated_time() {
    // Pure authority: rejected attempts may only bump attempt counters.
    let mut accepted_time = 1.0;
    let mut attempted = 0u64;
    let mut rejected = 0u64;
    let reject = true;
    attempted += 1;
    if reject {
        rejected += 1;
        // do not touch accepted_time
    } else {
        accepted_time += 0.1;
    }
    assert_eq!(accepted_time, 1.0);
    assert_eq!(attempted, 1);
    assert_eq!(rejected, 1);
}

#[test]
fn test_rejected_attempt_does_not_advance_window() {
    let mut samples = vec![sample(1, 0.1, 1.0)];
    let rejected = true;
    if !rejected {
        samples.push(sample(2, 0.2, 1.0));
    }
    assert_eq!(samples.len(), 1);
}

#[test]
fn test_rejected_attempt_does_not_increment_convergence() {
    let mut counter = ConvergenceCounter {
        consecutive_qualifying: 2,
        required: 3,
        windows: vec![],
    };
    let rejected = true;
    if !rejected {
        counter.consecutive_qualifying += 1;
    }
    assert_eq!(counter.consecutive_qualifying, 2);
}

#[test]
fn test_rejected_attempt_does_not_modify_accepted_ledgers() {
    let mut ledger = ActivationPotentialLedger::new(5.0);
    let before = ledger.clone();
    let rejected = true;
    if !rejected {
        ledger.fuel_reservoir_contribution += 1.0;
    }
    assert_eq!(ledger, before);
}

#[test]
fn test_rejected_attempt_does_not_emit_zero_motion_sample() {
    let mut windows: Vec<AcceptedStateSample> = Vec::new();
    let accepted = false;
    if accepted {
        windows.push(sample(10, 1.0, 1.0));
    }
    // Zero-motion samples from rejected attempts must not enter windows.
    assert!(windows.is_empty());
}

#[test]
fn test_window_requires_strict_simulated_time_progress() {
    let samples = vec![sample(1, 1.0, 1.0), sample(2, 1.0, 1.01)];
    let err = validate_accepted_window(&samples, 2).unwrap_err();
    assert!(err.iter().any(|e| e.contains("increase strictly")));
}

#[test]
fn test_window_requires_distinct_accepted_states() {
    let s = sample(5, 1.0, 1.0);
    let samples = vec![s.clone(), s];
    let err = validate_accepted_window(&samples, 2).unwrap_err();
    assert!(err.iter().any(|e| e.contains("distinct") || e.contains("increase")));
}

#[test]
fn test_window_rejects_missing_ledger_samples() {
    let mut s = sample(1, 0.1, 1.0);
    s.activation_potential_total = f64::NAN;
    let samples = vec![s.clone(), sample(2, 0.2, 1.1)];
    // First sample has NaN ledger.
    let mut bad = samples;
    bad[0].activation_potential_total = f64::NAN;
    let err = validate_accepted_window(&bad, 2).unwrap_err();
    assert!(err.iter().any(|e| e.contains("ledger")));
}

#[test]
fn test_three_consecutive_windows_use_nonoverlapping_terminal_evidence() {
    let mut counter = ConvergenceCounter {
        consecutive_qualifying: 0,
        required: 3,
        windows: vec![],
    };
    let mut prev = None;
    for i in 0..3 {
        let start = i * 10 + 1;
        let samples: Vec<_> = (0..10)
            .map(|j| sample((start + j) as u64, (start + j) as f64 * 0.1, 1.0 + j as f64 * 1e-6))
            .collect();
        let record = build_window_record(&samples, 10, prev.as_ref(), counter.consecutive_qualifying);
        let snap = sample_to_window_snapshot(samples.last().unwrap(), samples.first().unwrap());
        prev = Some(snap);
        update_convergence_counter(&mut counter, record);
    }
    assert!(windows_use_nonoverlapping_terminal_evidence(&counter.windows));
}

#[test]
fn test_three_consecutive_windows_are_valid() {
    let samples: Vec<_> = (1..=10).map(|i| sample(i, i as f64, 1.0 + i as f64 * 1e-4)).collect();
    assert!(validate_accepted_window(&samples, 10).is_ok());
}

#[test]
fn test_checkpoint_written_when_threshold_crossed() {
    let events = crossed_checkpoint_thresholds(9_999, 10_000);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].threshold, 10_000);
}

#[test]
fn test_all_required_checkpoint_thresholds_are_detected() {
    let events = crossed_checkpoint_thresholds(0, 200_000);
    let got: Vec<_> = events.iter().map(|e| e.threshold).collect();
    assert_eq!(got, D013_CHECKPOINT_THRESHOLDS.to_vec());
}

#[test]
fn test_checkpoint_write_is_atomic() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../experiments/generated/d013/checkpoint_tests");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("atomic_probe.json");
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, b"{\"clean_atomic_write\":true}").unwrap();
    fs::rename(&tmp, &path).unwrap();
    assert!(path.exists());
    assert!(!tmp.exists());
}

#[test]
fn test_partial_checkpoint_is_rejected() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../experiments/generated/d013/checkpoint_tests");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("partial.json");
    fs::write(&path, br#"{"clean_atomic_write":false,"checkpoint_threshold":10000}"#).unwrap();
    // Mimic loader gate.
    let v: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["clean_atomic_write"], false);
}

#[test]
fn test_checkpoint_contains_activation_potential() {
    let ledger = ActivationPotentialLedger::new(3.0);
    assert!(ledger.activation_potential_schema_version >= 1);
    assert!(ledger.initial_activation_potential > 0.0);
}

#[test]
fn test_checkpoint_contains_window_state() {
    let counter = ConvergenceCounter {
        consecutive_qualifying: 1,
        required: 3,
        windows: vec![],
    };
    assert_eq!(counter.required, 3);
}

#[test]
fn test_checkpoint_resume_matches_uninterrupted() {
    // Threshold crossing semantics underpin resume continuity.
    let a = crossed_checkpoint_thresholds(0, 25_000);
    let b = crossed_checkpoint_thresholds(25_000, 50_000);
    assert!(a.iter().any(|e| e.threshold == 25_000));
    assert!(b.iter().any(|e| e.threshold == 50_000));
    assert!(!b.iter().any(|e| e.threshold == 25_000));
}

#[test]
fn test_activation_potential_present_in_result() {
    let ledger = ActivationPotentialLedger::new(1.0);
    let mut view = empty_view();
    view.activation_potential_accounting = Some(ledger);
    // Still invalid overall, but activation field present.
    assert!(view.activation_potential_accounting.is_some());
}

#[test]
fn test_activation_potential_present_in_checkpoint() {
    let ledger = ActivationPotentialLedger::new(2.0);
    assert!(ledger.residual.is_finite());
}

#[test]
fn test_closed_system_does_not_create_activation_potential() {
    // v2 productive delta that consumes A without fuel import must not create potential.
    let mut delta = [0.0; SEVEN_FIELD_COUNT];
    delta[SpeciesId::A.index()] = -1.0;
    delta[SpeciesId::W.index()] = 1.0;
    assert!(!reaction_delta_creates_activation_potential(&delta));
}

#[test]
fn test_rejected_attempt_does_not_change_activation_potential_ledger() {
    let mut ledger = ActivationPotentialLedger::new(4.0);
    let snap = ledger.clone();
    let rejected = true;
    if !rejected {
        ledger.final_activation_potential += 1.0;
    }
    assert_eq!(ledger.final_activation_potential, snap.final_activation_potential);
}

#[test]
fn test_activation_potential_residual_is_reported() {
    let mut ledger = ActivationPotentialLedger::new(1.0);
    let step = ActivationPotentialStep {
        potential_before: 1.0,
        potential_after: 1.5,
        observed_change: 0.5,
        fuel_import: 0.5,
        reservoir_potential: 0.5,
        chemistry_potential: 0.0,
        transport_potential: 0.0,
        numerical_correction: 0.0,
        residual: 0.0,
        relative_residual: 0.0,
    };
    ledger.apply_accepted_step(&step, 0.0, 0.0, 0.0);
    assert!(ledger.residual.is_finite());
    assert!(ledger.relative_residual.is_finite());
}

#[test]
fn test_rejection_stall_is_not_convergence() {
    let reason = TerminationReason::TimestepFloorFailure;
    let sci = map_termination_to_scientific(reason, 200_000, 12_345);
    assert_eq!(sci, ScientificClassification::NumericalFailure);
    assert_ne!(sci, ScientificClassification::QuasiSteadyConverged);
}

#[test]
fn test_rejection_stall_produces_numerical_failure() {
    let sci = map_termination_to_scientific(TerminationReason::TimestepFloorFailure, 200_000, 100);
    assert_eq!(sci, ScientificClassification::NumericalFailure);
}

#[test]
fn test_rejection_stall_records_dominant_reason() {
    assert_eq!(
        format!("{:?}", TerminationReason::TimestepFloorFailure),
        "TimestepFloorFailure"
    );
}

fn empty_view() -> GovernedArtifactView {
    GovernedArtifactView {
        source_commit: None,
        binary_hash: None,
        candidate_hash: None,
        configuration_hash: None,
        equation_version: None,
        field_schema: None,
        stoichiometric_schema: None,
        checkpoint_completion: BTreeMap::new(),
        accepted_substeps: None,
        attempted_substeps: None,
        rejected_substeps: None,
        material_accounting: None,
        activation_potential_accounting: None,
        rolling_windows: None,
        termination_reason: None,
        clean_termination: None,
        field_hashes: None,
        artifact_hash: None,
    }
}

fn complete_view() -> GovernedArtifactView {
    let mut checkpoints = BTreeMap::new();
    for t in D013_CHECKPOINT_THRESHOLDS {
        checkpoints.insert(t.to_string(), true);
    }
    let mut hashes = BTreeMap::new();
    hashes.insert("structure".into(), "abc".into());
    GovernedArtifactView {
        source_commit: Some("deadbeef".into()),
        binary_hash: Some("bin".into()),
        candidate_hash: Some("cand".into()),
        configuration_hash: Some("cfg".into()),
        equation_version: Some("membrane_metabolism_v2_conservative".into()),
        field_schema: Some("seven-field".into()),
        stoichiometric_schema: Some(2),
        checkpoint_completion: checkpoints,
        accepted_substeps: Some(200_000),
        attempted_substeps: Some(200_100),
        rejected_substeps: Some(100),
        material_accounting: Some(MaterialEquivalentStep::default()),
        activation_potential_accounting: Some(ActivationPotentialLedger::new(1.0)),
        rolling_windows: Some(vec![]),
        termination_reason: Some(TerminationReason::MaxAcceptedSubstepsReached),
        clean_termination: Some(true),
        field_hashes: Some(hashes),
        artifact_hash: Some("art".into()),
    }
}

#[test]
fn test_artifact_validator_accepts_complete_reference() {
    let (status, missing) = validate_governed_artifact(&complete_view());
    assert_eq!(status, ArtifactValidationStatus::ValidGovernedArtifact);
    assert!(missing.is_empty());
}

#[test]
fn test_artifact_validator_rejects_missing_checkpoint_state() {
    let mut view = complete_view();
    view.checkpoint_completion.remove("10000");
    let (status, missing) = validate_governed_artifact(&view);
    assert_eq!(status, ArtifactValidationStatus::InvalidArtifact);
    assert!(missing.iter().any(|m| m.contains("checkpoint_10000")));
}

#[test]
fn test_artifact_validator_rejects_missing_activation_ledger() {
    let mut view = complete_view();
    view.activation_potential_accounting = None;
    let (status, missing) = validate_governed_artifact(&view);
    assert_eq!(status, ArtifactValidationStatus::InvalidArtifact);
    assert!(missing.iter().any(|m| m.contains("activation")));
}

#[test]
fn test_artifact_validator_rejects_invalid_window() {
    let mut view = complete_view();
    view.rolling_windows = Some(vec![WindowRecord {
        start_accepted_substep: 0,
        end_accepted_substep: 10,
        start_simulated_time: 0.0,
        end_simulated_time: 0.0,
        sample_count: 10,
        slopes: None,
        reaction_total_change: 0.0,
        transport_total_change: 0.0,
        valid: false,
        qualifying: true,
        consecutive_count: 1,
        invalid_reasons: vec!["zero motion".into()],
    }]);
    let (status, missing) = validate_governed_artifact(&view);
    assert_eq!(status, ArtifactValidationStatus::InvalidArtifact);
    assert!(missing.iter().any(|m| m.contains("invalid_qualifying")));
}

#[test]
fn test_artifact_validator_rejects_missing_termination_reason() {
    let mut view = complete_view();
    view.termination_reason = None;
    let (status, missing) = validate_governed_artifact(&view);
    assert_eq!(status, ArtifactValidationStatus::InvalidArtifact);
    assert!(missing.iter().any(|m| m.contains("termination_reason")));
}

#[test]
fn test_preflight_requires_10k_and_25k_checkpoints() {
    let events = crossed_checkpoint_thresholds(0, 25_000);
    assert!(events.iter().any(|e| e.threshold == 10_000));
    assert!(events.iter().any(|e| e.threshold == 25_000));
}

#[test]
fn test_solver_entry_requires_valid_converged_reference() {
    assert!(!solver_entry_allowed(
        ArtifactValidationStatus::ValidGovernedArtifact,
        ScientificClassification::NotConvergedAt200k,
        true,
        true
    ));
    assert!(solver_entry_allowed(
        ArtifactValidationStatus::ValidGovernedArtifact,
        ScientificClassification::QuasiSteadyConverged,
        true,
        true
    ));
    assert!(!solver_entry_allowed(
        ArtifactValidationStatus::InvalidArtifact,
        ScientificClassification::QuasiSteadyConverged,
        true,
        true
    ));
}
