//! D-019 structural scaling repair tests.

use chemistry_core::config::{D008StageMode, EquationVersion, SimParams, StructuralScalingMechanism};
use chemistry_core::d008_analysis::PrescribedInterior;
use chemistry_core::d011_analysis::STAGE_E_FAILED_RATES;
use chemistry_core::d012_accounting::{
    reaction_delta_creates_activation_potential, E_ACTIVATED, E_FUEL,
};
use chemistry_core::d018_analysis::D018_FROZEN_K_STRUCTURE;
use chemistry_core::stoichiometry::{
    run_v2_stoichiometric_audit, v2_runtime_activation_delta, v2_runtime_catalyst_production_delta,
    v2_runtime_structure_production_delta, ConservationClass, Rational, ReactionId,
};
use chemistry_core::structural_kinetics::*;
use chemistry_core::{
    build_candidate_identity, structure_decay_rate, structure_production_rate, Simulation,
};

fn v2_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV2Conservative;
    p.d008_stage_mode = D008StageMode::ConstrainedRadius;
    p.eta_c = 1.0;
    p.eta_phi = 1.0;
    p.eta_m = 1.0;
    STAGE_E_FAILED_RATES.apply_to(&mut p);
    p.k_d008_structure = D018_FROZEN_K_STRUCTURE;
    p.d019_mechanism_probe = None;
    p
}

fn v3_params() -> SimParams {
    let mut p = v2_params();
    p.equation_version = EquationVersion::MembraneMetabolismV3StructuralScaling;
    p
}

#[test]
fn test_mechanism_comparison_selects_interface_limited_turnover() {
    let results = compare_all_mechanisms_prescribed(&PrescribedInterior::default(), 0.025);
    assert!(
        results
            .iter()
            .any(|r| r.mechanism == StructuralScalingMechanism::InterfaceLimitedTurnover
                && r.passes_selection_gate),
        "mechanism B must pass prescribed selection gate"
    );
    let a = results
        .iter()
        .find(|r| r.mechanism == StructuralScalingMechanism::PhaseVolumeSynthesis)
        .unwrap();
    assert!(
        !a.passes_selection_gate,
        "phase-volume under uniform A is not restoring"
    );
    let selected = select_mechanism(&results).expect("a mechanism must pass");
    assert_eq!(selected, StructuralScalingMechanism::InterfaceLimitedTurnover);
    assert_eq!(selected, V3_SELECTED_MECHANISM);
}

#[test]
fn test_local_only_no_target_radius_or_mass() {
    for m in [
        StructuralScalingMechanism::PhaseVolumeSynthesis,
        StructuralScalingMechanism::InterfaceLimitedTurnover,
        StructuralScalingMechanism::LocalCurvatureMaintenance,
    ] {
        assert!(mechanism_is_local_only(m));
        assert!(!mechanism_encodes_forbidden_target(m));
    }
    let p = v3_params();
    assert!(p.d019_mechanism_probe.is_none());
}

#[test]
fn test_material_conservation_and_activation_accounting() {
    let audit = run_v2_stoichiometric_audit(Rational::ONE, Rational::ONE, Rational::ONE);
    assert_eq!(
        audit.conservation_class,
        ConservationClass::StrictlyConservative
    );
    for delta in [
        v2_runtime_activation_delta(1.0),
        v2_runtime_catalyst_production_delta(1.0, 1.0),
        v2_runtime_structure_production_delta(1.0, 1.0),
    ] {
        assert!(!reaction_delta_creates_activation_potential(&delta));
    }
    let _ = (E_FUEL, E_ACTIVATED, ReactionId::StructureDecay);
}

#[test]
fn test_full_structural_turnover_at_saturated_phi() {
    let p = v3_params();
    let decay = structure_decay_rate(1.0, 0.0, &p);
    assert!(
        decay > 1e-12,
        "φ=1 must retain nonzero decay under interface-limited turnover"
    );
    assert!((decay - p.k_structure_decay * STRUCTURAL_EXPOSURE_FLOOR).abs() < 1e-12);
}

#[test]
fn test_radius_sign_crossing_prescribed() {
    let r = compare_mechanism_prescribed(
        StructuralScalingMechanism::InterfaceLimitedTurnover,
        &PrescribedInterior::default(),
        0.025,
    );
    assert!(r.restoring_crossing);
    assert!(r.g_below > 0.0);
    assert!(r.g_above < 0.0);
    assert!(r.g_center.abs() <= 0.25 * r.g_below.abs().max(r.g_above.abs()).max(1e-6));
}

#[test]
fn test_historical_v2_equivalence_without_probe() {
    let mut p = v2_params();
    let phi = 0.5;
    let a = 0.2;
    let c = 0.4;
    let prod_v2 = structure_production_rate(phi, a, c, &p);
    let decay_v2 = structure_decay_rate(phi, 0.0, &p);
    assert!(
        (prod_v2 - p.k_d008_structure * a * chemistry_core::interface_weight(phi)).abs() < 1e-15
    );
    assert!((decay_v2 - p.k_structure_decay * phi).abs() < 1e-15);

    p.d019_mechanism_probe = Some(StructuralScalingMechanism::InterfaceLimitedTurnover);
    let decay_probe = structure_decay_rate(phi, 0.0, &p);
    assert!(decay_probe > decay_v2);
}

#[test]
fn test_v3_candidate_identity_distinct_from_v2() {
    let v2 = build_candidate_identity(
        v2_params(),
        "test",
        Some("d019-v2"),
        None,
        "v2",
        None,
        None,
    );
    let v3 = build_candidate_identity(
        v3_params(),
        "test",
        Some("d019-v3"),
        None,
        "v3",
        None,
        None,
    );
    assert_ne!(v2.candidate_hash, v3.candidate_hash);
    assert_eq!(
        v3.equation_version,
        EquationVersion::MembraneMetabolismV3StructuralScaling
    );
    assert_eq!(v3.equation_version.stoichiometric_schema_version(), 2);
    assert_eq!(STRUCTURAL_SCHEMA_VERSION_V3, 1);
}

#[test]
fn test_constraint_contamination_ceiling() {
    assert_eq!(STRUCTURAL_EXPOSURE_FLOOR, 0.05);
}

#[test]
fn test_unconstrained_structural_persistence_improved_vs_v2_short() {
    let mut v2 = Simulation::new(v2_params());
    v2.enforce_structure_constraint = false;
    seed_disk(&mut v2, 22.0);
    let mass0 = chemistry_core::field_mass(&v2.grid, &v2.fields.structure);
    for _ in 0..200 {
        if !v2.step() {
            break;
        }
    }
    let mass_v2 = chemistry_core::field_mass(&v2.grid, &v2.fields.structure);

    let mut v3 = Simulation::new(v3_params());
    v3.enforce_structure_constraint = false;
    seed_disk(&mut v3, 22.0);
    for _ in 0..200 {
        if !v3.step() {
            break;
        }
    }
    let mass_v3 = chemistry_core::field_mass(&v3.grid, &v3.fields.structure);
    assert!(mass0 > 0.0);
    assert!(
        mass_v3 + 1e-6 >= mass_v2,
        "v3 mass {mass_v3} should be >= v2 mass {mass_v2}"
    );
}

fn seed_disk(sim: &mut Simulation, radius: f64) {
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let x = (idx % sim.grid.width) as f64 - sim.grid.cx;
        let y = (idx / sim.grid.width) as f64 - sim.grid.cy;
        let distance = (x * x + y * y).sqrt();
        let phi = 0.5 * (1.0 - ((distance - radius) / 2.0).tanh());
        sim.fields.structure[idx] = phi;
        sim.fields.membrane[idx] = chemistry_core::interface_weight(phi);
        if phi >= 0.5 {
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.activated[idx] = 0.2;
            sim.fields.nutrient[idx] = 0.2;
            sim.fields.fuel[idx] = 0.2;
            sim.fields.waste[idx] = 0.5;
        } else {
            sim.fields.catalyst[idx] = 0.0;
            sim.fields.activated[idx] = 0.0;
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
            sim.fields.waste[idx] = sim.params.w_reservoir;
        }
    }
    sim.fields.copy_current_to_next();
}
