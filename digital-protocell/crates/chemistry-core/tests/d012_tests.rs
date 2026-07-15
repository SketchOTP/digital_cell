//! D-012 stoichiometric descriptor, conservation analysis, and v1/v2 gate tests.

use chemistry_core::accounting::{build_field_ledger, StepAccounting};
use chemistry_core::activated_metabolism::{
    activation_isolated_delta, activated_metabolism_rates, catalyst_production_isolated_delta,
    turnover_isolated_delta,
};
use chemistry_core::config::{
    validate_v2_yields, EquationVersion, SimParams, STOICHIOMETRIC_SCHEMA_VERSION_V2,
};
use chemistry_core::d012_accounting::{
    activation_potential, build_activation_potential_step, build_material_equivalent_step,
    internal_extent_conserves_material, material_step_closes, material_weight_vector,
    reaction_delta_creates_activation_potential, waste_is_consumed_as_reactant, E_ACTIVATED,
    E_FUEL,
};
use chemistry_core::membrane::{
    membrane_loss_isolated_delta, membrane_synthesis_isolated_delta,
    structure_production_isolated_delta,
};
use chemistry_core::stoichiometry::*;
use chemistry_core::{candidate_hash, Simulation};

fn v2_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV2Conservative;
    p.eta_c = 1.0;
    p.eta_phi = 1.0;
    p.eta_m = 1.0;
    p
}

fn approx_eq(a: [f64; SEVEN_FIELD_COUNT], b: [f64; SEVEN_FIELD_COUNT], tol: f64) {
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert!(
            (x - y).abs() <= tol,
            "species {}: got {x}, expected {y}",
            SpeciesId::ALL[i].label()
        );
    }
}

#[test]
fn test_v1_descriptor_order_matches_governed_reaction_list() {
    let ids: Vec<_> = v1_internal_reactions()
        .iter()
        .map(|r| r.reaction)
        .collect();
    assert_eq!(
        ids,
        vec![
            ReactionId::Activation,
            ReactionId::CatalystProduction,
            ReactionId::StructureProduction,
            ReactionId::MembraneProduction,
            ReactionId::StructureDecay,
            ReactionId::CatalystDecay,
            ReactionId::ActivatedDecay,
            ReactionId::MembraneDecay,
            ReactionId::MembraneDetachment,
        ]
    );
}

#[test]
fn test_positive_conservation_vector_detection() {
    // A → W alone is conservative under all-ones material weight.
    let rx = [ReactionStoichiometry::new(
        ReactionId::ActivatedDecay,
        {
            let mut d = [Rational::ZERO; SEVEN_FIELD_COUNT];
            d[SpeciesId::A.index()] = Rational::from_i64(-1);
            d[SpeciesId::W.index()] = Rational::from_i64(1);
            d
        },
    )];
    let matrix = stoichiometric_matrix(&rx);
    let m = vec![Rational::ONE; SEVEN_FIELD_COUNT];
    assert!(verify_m_transpose_s_zero(&m, &matrix));
    assert_eq!(
        classify_conservation(&matrix),
        ConservationClass::StrictlyConservative
    );
}

#[test]
fn test_nonconservative_reaction_is_identified() {
    // A → C + W creates material under all-ones weight.
    let rx = [ReactionStoichiometry::new(
        ReactionId::CatalystProduction,
        {
            let mut d = [Rational::ZERO; SEVEN_FIELD_COUNT];
            d[SpeciesId::C.index()] = Rational::from_i64(1);
            d[SpeciesId::A.index()] = Rational::from_i64(-1);
            d[SpeciesId::W.index()] = Rational::from_i64(1);
            d
        },
    )];
    let matrix = stoichiometric_matrix(&rx);
    let positives = positive_conservation_vectors(&matrix);
    assert!(positives.is_empty());
    let all_ones = vec![Rational::ONE; SEVEN_FIELD_COUNT];
    let bad = nonconservative_reactions_under_vector(&all_ones, &rx);
    assert_eq!(bad, vec![ReactionId::CatalystProduction]);
}

#[test]
fn test_v1_stoichiometric_matrix_matches_reactions() {
    let reactions = v1_internal_reactions();
    let matrix = stoichiometric_matrix(reactions);
    assert_eq!(matrix.len(), SEVEN_FIELD_COUNT);
    assert_eq!(matrix[0].len(), ReactionId::INTERNAL_COUNT);

    for (col, rx) in reactions.iter().enumerate() {
        for (row, &expected) in rx.delta.iter().enumerate() {
            assert_eq!(
                matrix[row][col], expected,
                "species {:?} reaction {:?}",
                SpeciesId::ALL[row], rx.reaction
            );
        }
    }

    // Spot-check governed v1 columns against runtime-isolated deltas.
    assert_eq!(
        v1_runtime_isolated_delta(ReactionId::Activation),
        v1_runtime_activation_delta(1.0)
    );
    assert_eq!(
        v1_runtime_isolated_delta(ReactionId::CatalystProduction),
        v1_runtime_catalyst_production_delta(1.0)
    );
    assert_eq!(
        v1_runtime_isolated_delta(ReactionId::StructureProduction),
        v1_runtime_structure_production_delta(1.0)
    );
    assert_eq!(
        v1_runtime_isolated_delta(ReactionId::MembraneProduction),
        v1_runtime_membrane_production_delta(1.0)
    );
}

#[test]
fn test_v1_positive_conservation_vector_search() {
    let matrix = stoichiometric_matrix(v1_internal_reactions());
    let positives = positive_conservation_vectors(&matrix);
    assert!(
        positives.is_empty(),
        "v1 must not admit a strictly positive conservation vector: {:?}",
        positives
    );
}

#[test]
fn test_v1_nonconservative_productive_reaction_detection() {
    let reactions = v1_internal_reactions();
    let all_ones = vec![Rational::ONE; SEVEN_FIELD_COUNT];
    let bad = nonconservative_reactions_under_vector(&all_ones, reactions);
    assert!(bad.contains(&ReactionId::CatalystProduction));
    assert!(bad.contains(&ReactionId::MembraneProduction));
    assert!(bad.contains(&ReactionId::MembraneDecay));
    assert!(bad.contains(&ReactionId::MembraneDetachment));

    let audit = run_v1_stoichiometric_audit();
    assert_eq!(audit.primary_finding, "D012_NONCONSERVATIVE_V1_CONFIRMED");
    assert_eq!(
        audit.conservation_class,
        ConservationClass::NoPositiveConservationVector
    );
    assert_eq!(
        audit.d011_branch_recommendation,
        "SKIP_D011_EXPENSIVE_COMPLETION_SUPERSEDED_BY_INVALID_STOICHIOMETRY"
    );
}

#[test]
fn test_field_ledgers_can_close_while_total_stoichiometry_fails() {
    // Stage-C per-field ledgers close for activation (N,F,A,W balance) while
    // catalyst production creates net material under any strictly positive weight.
    let activation = v1_runtime_activation_delta(1.0);
    let sum_act: f64 = activation.iter().sum();
    assert!((sum_act).abs() < 1e-12, "activation conserves under sum check");

    let reproduction = v1_runtime_catalyst_production_delta(1.0);
    let sum_rep: f64 = reproduction.iter().sum();
    assert!((sum_rep - 1.0).abs() < 1e-12, "reproduction creates +1 net mass");

    let reactions = v1_internal_reactions();
    let matrix = stoichiometric_matrix(reactions);
    assert_eq!(
        classify_conservation(&matrix),
        ConservationClass::NoPositiveConservationVector
    );
}

#[test]
fn test_v2_unit_yield_is_strictly_conservative() {
    let one = Rational::ONE;
    let reactions = v2_internal_reactions(one, one, one);
    let matrix = stoichiometric_matrix(&reactions);
    assert_eq!(
        classify_conservation(&matrix),
        ConservationClass::StrictlyConservative
    );
    let m = vec![Rational::ONE; SEVEN_FIELD_COUNT];
    assert!(verify_m_transpose_s_zero(&m, &matrix));
}

#[test]
fn write_v1_audit_json_artifact() {
    let json = v1_audit_json_pretty();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("audit json valid");
    assert_eq!(
        parsed["primary_finding"].as_str(),
        Some("D012_NONCONSERVATIVE_V1_CONFIRMED")
    );
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../experiments/generated/d012/v1_stoichiometric_audit"
    );
    std::fs::create_dir_all(dir).expect("audit dir");
    std::fs::write(format!("{dir}/audit.json"), json).expect("write audit.json");
}

#[test]
fn test_v2_equation_version() {
    let p = v2_params();
    assert_eq!(
        p.equation_version,
        EquationVersion::MembraneMetabolismV2Conservative
    );
    assert_eq!(p.equation_version.as_str(), "membrane_metabolism_v2_conservative");
    assert_eq!(
        p.equation_version.stoichiometric_schema_version(),
        STOICHIOMETRIC_SCHEMA_VERSION_V2
    );
    p.validate_equation_config().expect("unit yields valid");
}

#[test]
fn test_v2_snapshot_rejects_v1_resume() {
    let mut v1 = v2_params();
    v1.equation_version = EquationVersion::MembraneMetabolismV1;
    let snap = Simulation::new(v1).snapshot();
    let v2 = v2_params();
    let err = snap.can_resume_into(&v2).unwrap_err();
    assert!(err.contains("stoichiometric_schema_version") || err.contains("incompatible"));
}

#[test]
fn test_yield_cannot_exceed_one() {
    assert!(validate_v2_yields(1.0, 1.0, 1.0).is_ok());
    assert!(validate_v2_yields(0.5, 17.0 / 20.0, 7.0 / 10.0).is_ok());
    assert!(validate_v2_yields(1.1, 1.0, 1.0).is_err());
    assert!(validate_v2_yields(0.0, 1.0, 1.0).is_err());
    assert!(validate_v2_yields(-0.1, 1.0, 1.0).is_err());
}

#[test]
fn test_v2_activation_stoichiometry() {
    let d = v2_runtime_activation_delta(1.0);
    approx_eq(d, activation_isolated_delta(1.0), 1e-12);
    let desc = v2_descriptor_isolated_delta(
        ReactionId::Activation,
        1.0,
        Rational::ONE,
        Rational::ONE,
        Rational::ONE,
    );
    approx_eq(d, desc, 1e-12);
}

#[test]
fn test_v2_catalyst_yield_stoichiometry() {
    let p = v2_params();
    let d = catalyst_production_isolated_delta(1.0, &p);
    approx_eq(d, v2_runtime_catalyst_production_delta(1.0, 1.0), 1e-12);
    assert!((d[SpeciesId::C.index()] - 1.0).abs() < 1e-12);
    assert!((d[SpeciesId::A.index()] + 1.0).abs() < 1e-12);
    assert!(d[SpeciesId::W.index()].abs() < 1e-12);
}

#[test]
fn test_v2_structure_yield_stoichiometry() {
    let d = structure_production_isolated_delta(1.0, 1.0);
    approx_eq(d, v2_runtime_structure_production_delta(1.0, 1.0), 1e-12);
}

#[test]
fn test_v2_membrane_yield_stoichiometry() {
    let d = membrane_synthesis_isolated_delta(1.0, 1.0);
    approx_eq(d, v2_runtime_membrane_production_delta(1.0, 1.0), 1e-12);
    assert!((d[SpeciesId::A.index()] + 1.0).abs() < 1e-12);
}

#[test]
fn test_v2_turnover_converts_to_waste() {
    for (rx, src) in [
        (ReactionId::StructureDecay, SpeciesId::Phi.index()),
        (ReactionId::CatalystDecay, SpeciesId::C.index()),
        (ReactionId::ActivatedDecay, SpeciesId::A.index()),
        (ReactionId::MembraneDecay, SpeciesId::M.index()),
        (ReactionId::MembraneDetachment, SpeciesId::M.index()),
    ] {
        let d = v2_runtime_turnover_delta(rx, 1.0);
        assert!((d[src] + 1.0).abs() < 1e-12);
        assert!((d[SpeciesId::W.index()] - 1.0).abs() < 1e-12);
        approx_eq(d, turnover_isolated_delta(src, 1.0), 1e-12);
    }
}

#[test]
fn test_runtime_activation_delta_matches_matrix() {
    let rates = activated_metabolism_rates(1.0, 1.0, 1.0, 0.0, &v2_params());
    let extent = rates.activation;
    let mut runtime = [0.0; SEVEN_FIELD_COUNT];
    runtime[SpeciesId::N.index()] = -extent;
    runtime[SpeciesId::F.index()] = -extent;
    runtime[SpeciesId::A.index()] = extent;
    runtime[SpeciesId::W.index()] = extent;
    approx_eq(runtime, v2_runtime_activation_delta(extent), 1e-12);
}

#[test]
fn test_runtime_catalyst_delta_matches_matrix() {
    let p = v2_params();
    let rates = activated_metabolism_rates(1.0, 0.0, 0.0, 1.0, &p);
    let extent = rates.reproduction;
    let mut runtime = [0.0; SEVEN_FIELD_COUNT];
    runtime[SpeciesId::C.index()] = p.eta_c * extent;
    runtime[SpeciesId::A.index()] = -extent;
    runtime[SpeciesId::W.index()] = (1.0 - p.eta_c) * extent;
    approx_eq(runtime, v2_runtime_catalyst_production_delta(extent, p.eta_c), 1e-12);
}

#[test]
fn test_runtime_structure_delta_matches_matrix() {
    let p = v2_params();
    let extent = 0.03;
    let d = structure_production_isolated_delta(extent, p.eta_phi);
    approx_eq(d, v2_runtime_structure_production_delta(extent, p.eta_phi), 1e-12);
}

#[test]
fn test_runtime_membrane_delta_matches_matrix() {
    let p = v2_params();
    let extent = 0.02;
    let d = membrane_synthesis_isolated_delta(extent, p.eta_m);
    approx_eq(d, v2_runtime_membrane_production_delta(extent, p.eta_m), 1e-12);
}

#[test]
fn test_runtime_turnover_deltas_match_matrix() {
    for rx in [
        ReactionId::StructureDecay,
        ReactionId::CatalystDecay,
        ReactionId::ActivatedDecay,
        ReactionId::MembraneDecay,
        ReactionId::MembraneDetachment,
    ] {
        let extent = 0.01;
        let desc = v2_descriptor_isolated_delta(rx, extent, Rational::ONE, Rational::ONE, Rational::ONE);
        approx_eq(desc, v2_runtime_turnover_delta(rx, extent), 1e-12);
    }
}

#[test]
fn test_v2_each_internal_reaction_is_conservative() {
    let reactions = v2_internal_reactions(Rational::ONE, Rational::ONE, Rational::ONE);
    let m = material_weight_vector();
    for rx in &reactions {
        assert!(
            internal_extent_conserves_material(&rx.delta),
            "reaction {:?} not conservative",
            rx.reaction
        );
        let mut sum = Rational::ZERO;
        for (i, &mi) in m.iter().enumerate() {
            sum = sum.add(mi.mul(rx.delta[i]));
        }
        assert!(sum.is_zero(), "{:?}", rx.reaction);
    }
}

#[test]
fn test_v2_positive_conservation_vector() {
    let matrix = stoichiometric_matrix(&v2_internal_reactions(
        Rational::ONE,
        Rational::ONE,
        Rational::ONE,
    ));
    let positives = positive_conservation_vectors(&matrix);
    assert!(!positives.is_empty());
    let all_ones = vec![Rational::ONE; SEVEN_FIELD_COUNT];
    assert!(verify_m_transpose_s_zero(&all_ones, &matrix));
}

#[test]
fn test_v2_has_strictly_positive_conservation_vector() {
    test_v2_positive_conservation_vector();
}

#[test]
fn test_v2_total_accounting_closes() {
    test_v2_total_change_equals_boundary_exchange();
}

#[test]
fn test_v2_total_change_equals_boundary_exchange() {
    // Conservative internal step: activation extent 0.2 (N,F,A,W) + catalyst turnover 0.1 (C,W).
    let step = StepAccounting {
        structure: build_field_ledger(10.0, 0.0, 0.0, 0.0, 10.0, 10.0),
        catalyst: build_field_ledger(2.0, -0.1, 0.0, 0.0, 1.9, 1.9),
        nutrient: build_field_ledger(5.0, -0.2, 0.0, 0.0, 4.8, 4.8),
        fuel: build_field_ledger(4.0, -0.2, 0.0, 0.5, 4.3, 4.3),
        waste: build_field_ledger(1.0, 0.3, 0.0, -0.1, 1.2, 1.2),
        activated: build_field_ledger(0.5, 0.2, 0.0, 0.0, 0.7, 0.7),
        membrane: build_field_ledger(0.2, 0.0, 0.0, 0.0, 0.2, 0.2),
    };
    let material = build_material_equivalent_step(&step);
    assert!(material_step_closes(&material), "{material:?}");
}

#[test]
fn test_v2_waste_clearance_is_explicit_output() {
    let step = StepAccounting {
        structure: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        catalyst: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        nutrient: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        fuel: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        waste: build_field_ledger(2.0, 0.0, 0.0, -0.4, 1.6, 1.6),
        activated: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        membrane: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    };
    let material = build_material_equivalent_step(&step);
    assert!((material.waste_clearance - 0.4).abs() < 1e-12);
}

#[test]
fn test_v2_membrane_detachment_converts_to_waste() {
    let d = membrane_loss_isolated_delta(1.0);
    approx_eq(d, v2_runtime_turnover_delta(ReactionId::MembraneDetachment, 1.0), 1e-12);
}

#[test]
fn test_closed_v2_network_does_not_create_material() {
    let reactions = v2_internal_reactions(Rational::ONE, Rational::ONE, Rational::ONE);
    let total: Rational = reactions.iter().fold(Rational::ZERO, |acc, rx| {
        let mut sum = Rational::ZERO;
        for d in rx.delta {
            sum = sum.add(d);
        }
        acc.add(sum)
    });
    assert!(total.is_zero() || reactions.iter().all(|rx| internal_extent_conserves_material(&rx.delta)));
    let step = StepAccounting {
        structure: build_field_ledger(3.0, 0.0, 0.0, 0.0, 3.0, 3.0),
        catalyst: build_field_ledger(2.0, 0.0, 0.0, 0.0, 2.0, 2.0),
        nutrient: build_field_ledger(4.0, -1.0, 0.0, 0.0, 3.0, 3.0),
        fuel: build_field_ledger(4.0, -1.0, 0.0, 0.0, 3.0, 3.0),
        waste: build_field_ledger(1.0, 2.0, 0.0, 0.0, 3.0, 3.0),
        activated: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        membrane: build_field_ledger(0.5, 0.0, 0.0, 0.0, 0.5, 0.5),
    };
    let material = build_material_equivalent_step(&step);
    assert!((material.reservoir_input).abs() < 1e-12);
    assert!((material.observed_change).abs() < 1e-12);
}

#[test]
fn test_closed_v2_network_does_not_create_activation_potential() {
    let reactions = v2_internal_reactions(Rational::ONE, Rational::ONE, Rational::ONE);
    for rx in &reactions {
        let mut delta = [0.0; SEVEN_FIELD_COUNT];
        for (i, r) in rx.delta.iter().enumerate() {
            delta[i] = r.num as f64 / r.den as f64;
        }
        assert!(
            !reaction_delta_creates_activation_potential(&delta),
            "{:?}",
            rx.reaction
        );
    }
}

#[test]
fn test_fuel_is_only_external_activation_potential_source() {
    let step = StepAccounting {
        structure: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        catalyst: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        nutrient: build_field_ledger(1.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        fuel: build_field_ledger(1.0, 0.0, 0.0, 0.5, 1.5, 1.5),
        waste: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        activated: build_field_ledger(0.5, 0.0, 0.0, 0.0, 0.5, 0.5),
        membrane: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    };
    let pot = build_activation_potential_step(&step);
    assert!((pot.fuel_import - 0.5).abs() < 1e-12);
    assert!((pot.observed_change - 0.5).abs() < 1e-12);
    assert!(pot.relative_residual <= 1e-6);
}

#[test]
fn test_waste_cannot_reactivate_spontaneously() {
    let reactions = v2_internal_reactions(Rational::ONE, Rational::ONE, Rational::ONE);
    for rx in &reactions {
        let mut delta = [0.0; SEVEN_FIELD_COUNT];
        for (i, r) in rx.delta.iter().enumerate() {
            delta[i] = r.num as f64 / r.den as f64;
        }
        assert!(!waste_is_consumed_as_reactant(&delta), "{:?}", rx.reaction);
    }
}

#[test]
fn test_v1_v2_candidate_hashes_differ() {
    let mut v1 = v2_params();
    v1.equation_version = EquationVersion::MembraneMetabolismV1;
    let v2 = v2_params();
    let grid = chemistry_core::GridConfiguration::default();
    assert_ne!(candidate_hash(&v1, &grid), candidate_hash(&v2, &grid));
}

#[test]
fn test_v2_conservation_gate() {
    let audit = run_v2_stoichiometric_audit(Rational::ONE, Rational::ONE, Rational::ONE);
    assert_eq!(audit.primary_finding, "D012_CONSERVATIVE_V2_CONFIRMED");
    assert_eq!(audit.conservation_class, ConservationClass::StrictlyConservative);
}

#[test]
fn write_v2_audit_and_accounting_artifacts() {
    let matrix_json = v2_audit_json_pretty();
    let matrix_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../experiments/generated/d012/v2_stoichiometric_matrix"
    );
    std::fs::create_dir_all(matrix_dir).expect("matrix dir");
    std::fs::write(format!("{matrix_dir}/audit.json"), matrix_json).expect("write v2 audit");

    let accounting = serde_json::json!({
        "material_identity": "observed_change = reservoir_input - waste_clearance + numerical_correction",
        "activation_weights": { "e_F": E_FUEL, "e_A": E_ACTIVATED },
        "relative_tolerance": 1e-6,
    });
    let acct_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../experiments/generated/d012/accounting"
    );
    std::fs::create_dir_all(acct_dir).expect("accounting dir");
    std::fs::write(
        format!("{acct_dir}/ledger_spec.json"),
        serde_json::to_string_pretty(&accounting).unwrap(),
    )
    .expect("write ledger spec");
}

