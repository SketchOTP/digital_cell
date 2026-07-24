//! D-084 focused unit tests (no long dynamic sims).

use chemistry_core::d084_analysis::*;
use chemistry_core::reactions::interface_weight;
use chemistry_core::structural_kinetics::{
    apply_mixed_turnover_params, mixed_structure_loss_density, structure_decay_rate,
    STRUCTURAL_EXPOSURE_FLOOR,
};

#[test]
fn ids_and_conclusions() {
    assert_eq!(D084_STARTING_COMMIT, "b966502");
    assert_eq!(D084_STARTING_TAG, "D-083-edge-dynamic-migration-repaired");
    assert_eq!(
        D084Conclusion::NoRestoringStructuralCrossing.as_str(),
        "D084_NO_RESTORING_STRUCTURAL_CROSSING"
    );
    assert_eq!(
        D084Conclusion::EdgeStructuralHomeostasisQualified.as_str(),
        "D084_EDGE_STRUCTURAL_HOMEOSTASIS_QUALIFIED"
    );
}

#[test]
fn mixed_eta0_is_interface_only() {
    for phi in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!(eta0_equals_interface_only(phi, 0.025, 1e-12), "phi={phi}");
    }
}

#[test]
fn mixed_eta1_is_bulk() {
    let k = 0.04;
    let phi = 0.8;
    let got = mixed_structure_loss_density(phi, k, 1.0);
    assert!((got - k * phi).abs() < 1e-14);
}

#[test]
fn mixed_eta_half_interpolates() {
    let k = 0.03;
    let phi = 0.5;
    let i = interface_weight(phi);
    let got = mixed_structure_loss_density(phi, k, 0.5);
    let expect = k * phi * (0.5 + 0.5 * i);
    assert!((got - expect).abs() < 1e-14);
}

#[test]
fn legacy_mode_unchanged_when_mixed_off() {
    let p = ledger_params();
    assert!(!p.use_mixed_structure_turnover);
    let phi = 0.5;
    let got = structure_decay_rate(phi, 0.0, &p);
    let expect = p.k_structure_decay * phi * (STRUCTURAL_EXPOSURE_FLOOR + interface_weight(phi));
    assert!((got - expect).abs() < 1e-14);
}

#[test]
fn apply_mixed_enables_law() {
    let mut p = ledger_params();
    apply_mixed_turnover_params(&mut p, 0.2, 0.01);
    assert!(p.use_mixed_structure_turnover);
    assert!((p.structure_turnover_eta - 0.2).abs() < 1e-15);
    assert!((p.k_structure_decay - 0.01).abs() < 1e-15);
    let phi = 0.5;
    let got = structure_decay_rate(phi, 0.0, &p);
    let expect = mixed_structure_loss_density(phi, 0.01, 0.2);
    assert!((got - expect).abs() < 1e-14);
}

#[test]
fn phi_to_w_and_hash() {
    assert!(phi_to_w_conservation(-1.0, 1.0, 1.0, 1e-12));
    let h = candidate_hash(0.15, 0.02);
    assert_eq!(h.len(), 64);
    assert_eq!(h, candidate_hash(0.15, 0.02));
}

#[test]
fn restoring_sign_classification() {
    assert!(classify_restoring_nets(1.0, 0.0, -1.0, 0.08));
    assert!(!classify_restoring_nets(1.0, 1.0, -1.0, 0.08));
    assert!(!classify_restoring_nets(-1.0, 0.0, 1.0, 0.08));
}

#[test]
fn gate1_and_gate2_produce_candidates() {
    let legacy = ledger_params();
    let g1 = gate1_structural_ledger(&legacy);
    assert!(g1.pass, "{g1:?}");
    assert_eq!(g1.rows.len(), 6);
    assert!(g1.p_g.is_some() && g1.p_l.is_some());
    let g2 = gate2_identify_candidates(&legacy, &g1);
    assert!(g2.pass, "{g2:?}");
    assert!(g2.candidates.len() <= 4);
    assert!(g2.candidates.iter().any(|c| c.is_control && c.eta == 0.0));
    assert!(g2.accepted.iter().any(|c| c.is_control));
}

#[test]
fn gate3_and_gate4_screen() {
    let legacy = ledger_params();
    let g1 = gate1_structural_ledger(&legacy);
    let g2 = gate2_identify_candidates(&legacy, &g1);
    let ctrl = g2.accepted.iter().find(|c| c.is_control).unwrap();
    let g3 = gate3_conservation_safety(ctrl);
    assert!(g3.pass, "{g3:?}");
    let g4 = gate4_prescribed_radius_screen(&g2.accepted);
    eprintln!(
        "D084 ledger p_g={:?} p_l={:?} matched={} g22={:.6e}",
        g1.p_g, g1.p_l, g1.approximately_matched, g2.g22
    );
    for c in &g2.candidates {
        eprintln!(
            "  cand eta={:.4} k={:.6e} rejected={} reason={:?}",
            c.eta, c.k_phi_minus, c.rejected, c.reject_reason
        );
    }
    for row in &g4.rows {
        eprintln!(
            "  screen eta={:.4} net18={:.6e} net22={:.6e} net26={:.6e} restoring={}",
            row.eta, row.net_r18, row.net_r22, row.net_r26, row.restoring
        );
    }
    eprintln!("gate4 pass={} qualifying={}", g4.pass, g4.qualifying.len());
    // May or may not find restoring — assert R22 near zero for calibrated candidates.
    for row in &g4.rows {
        let scale = (row.net_r18.abs() + row.net_r26.abs()).max(1e-9);
        assert!(
            row.net_r22.abs() <= 0.15 * scale || row.net_r22.abs() < 1e-6,
            "R22 should be near balance after calibration: {row:?}"
        );
    }
}
