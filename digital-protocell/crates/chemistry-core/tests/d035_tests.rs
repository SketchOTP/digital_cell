//! D-035 Gate 0 architecture-screen tests (observer-only).

use chemistry_core::d035_analysis::{
    architecture_review, identify_saturation_constants, provisional_saturation_constants,
    screen_law, CatalyticLawId, LINEAR_SURFACE_MATURATION_LAW_REJECTED, D035_CATALYTIC_SPAN_MAX,
};

#[test]
fn linear_rejection_constant_recorded() {
    assert_eq!(
        LINEAR_SURFACE_MATURATION_LAW_REJECTED,
        "LINEAR_SURFACE_MATURATION_LAW_REJECTED"
    );
}

#[test]
fn control_a_reproduces_nonportable_span() {
    let (k_a, k_u) = provisional_saturation_constants();
    let a = screen_law(CatalyticLawId::ControlALinear, 0.0, k_a, k_u);
    assert!(a.valid_count >= 5, "expected frozen family valid states");
    assert!(
        a.span_factor > D035_CATALYTIC_SPAN_MAX,
        "control A must remain non-portable, span={}",
        a.span_factor
    );
    assert!(!a.portable);
}

#[test]
fn architecture_review_emits_exact_conclusion() {
    let review = architecture_review();
    assert_eq!(
        review.linear_law_rejected,
        LINEAR_SURFACE_MATURATION_LAW_REJECTED
    );
    assert!(
        review.conclusion == "D035_MEMBRANE_CATALYTIC_ARCHITECTURE_REJECTED"
            || review.conclusion.starts_with("D035_GATE0_CANDIDATE_"),
        "unexpected conclusion {}",
        review.conclusion
    );
    // Pass requires selected law; reject requires none.
    if review.pass {
        assert!(review.selected_law.is_some());
    } else {
        assert!(review.selected_law.is_none());
        assert_eq!(
            review.conclusion,
            "D035_MEMBRANE_CATALYTIC_ARCHITECTURE_REJECTED"
        );
    }
}

#[test]
fn candidate_laws_have_finite_positive_estimates_or_honest_reject() {
    let review = architecture_review();
    for screen in [&review.candidate_b, &review.candidate_c] {
        for e in &screen.estimates {
            if e.valid {
                assert!(e.rate_required.is_finite() && e.rate_required > 0.0);
                assert!(e.basis.is_finite() && e.basis > 0.0);
            }
        }
    }
}

#[test]
fn saturation_dose_response_identifiable_or_honest_fail() {
    let id = identify_saturation_constants();
    assert!(id.a_response.zero_at_zero);
    assert!(id.u_response.zero_at_zero);
    assert!(id.a_response.monotonic);
    assert!(id.u_response.monotonic);
    if id.pass {
        assert!(id.a_response.identifiable && id.u_response.identifiable);
        assert!(id.a_response.k_in_range && id.u_response.k_in_range);
        assert_eq!(id.conclusion, "D035_CATALYTIC_KINETICS_IDENTIFIABLE");
    } else {
        assert_eq!(id.conclusion, "D035_CATALYTIC_KINETICS_NOT_IDENTIFIABLE");
    }
}

#[test]
fn v12_conservation_and_autocatalytic_signature() {
    let c = chemistry_core::d035_analysis::gate2_conservation();
    assert!(c.pass, "conservation failed: {:?}", c);
    let a = chemistry_core::d035_analysis::gate3_autocatalytic_signature();
    assert!(a.pass, "signature failed: {:?}", a);
    let r = chemistry_core::d035_analysis::reconstruct_catalytic_rate();
    assert!(
        r.portable,
        "catalytic rate not portable span={} median={}",
        r.span_factor,
        r.median_rate
    );
    assert!(r.span_factor <= chemistry_core::d035_analysis::D035_CATALYTIC_SPAN_MAX);
}

#[test]
fn v12_snapshot_isolation() {
    let p = chemistry_core::d035_analysis::v12_params(0.001, 0.02);
    assert_eq!(
        p.equation_version.as_str(),
        "membrane_metabolism_v12_membrane_catalytic_assembly"
    );
    assert!(p.equation_version.is_membrane_catalytic_assembly());
    assert!(p.equation_version.is_surface_maturation());
}
