use phase1_certifier::frozen::{verify_frozen_center, FROZEN_CENTER};
use phase1_certifier::metrics::{interpret_d086_tracer, replacement_report, E_INV};

#[test]
fn frozen_center_matches_default_mech() {
    assert!(verify_frozen_center(&FROZEN_CENTER));
    assert!(verify_frozen_center(&chemistry_core::mesh_mechanics::MechParams::default()));
}

#[test]
fn tracer_interpretation_is_label_fraction_not_rx() {
    let s = interpret_d086_tracer("m", 0.35);
    assert!(s.contains("NOT R_X"));
    assert!(s.contains("0.350") || s.contains("0.35"));
}

#[test]
fn dual_replacement_formulas() {
    let r = replacement_report("m", 10.0, 12.0, 1.0, 0.2, 5.0);
    assert!(r.r_x >= 1.0);
    assert!(r.f_label <= E_INV);
    assert!((r.f_pool - 0.04).abs() < 1e-9);
    assert!(r.r_x_ok && r.f_label_ok);
}

#[test]
fn gate1_full_turnover_probe() {
    let audit = phase1_certifier::sim::audit_turnover(5_000);
    eprintln!(
        "POOL m={:.3} b={:.3} c={:.3} soft={} | R_m={:.3} f_m={:.3} R_b={:.3} f_b={:.3} R_c={:.3} f_c={:.3} dual={}",
        audit.d086_pool_m,
        audit.d086_pool_b,
        audit.d086_pool_c,
        audit.d086_soft_pass,
        audit.structural.r_x,
        audit.structural.f_label,
        audit.membrane.r_x,
        audit.membrane.f_label,
        audit.catalyst.r_x,
        audit.catalyst.f_label,
        audit.dual_requirement_pass
    );
    assert!(audit.retention_c.increase_accounted_by_production);
    assert!(audit.retention_a.increase_accounted_by_production);
}
