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
use chemistry_core::d008_analysis::{membrane_calibration, membrane_candidates};
use chemistry_core::d008_diagnostics::membrane_partition;
use chemistry_core::membrane_transport::TransportSpecies;
use chemistry_core::config::D008StageMode;
use chemistry_core::reactions::interface_weight;
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
    let rates = activated_metabolism_rates(1.0, 1.0, 1.0, 1.0, 0.0, &v2_params());
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
    let rates = activated_metabolism_rates(1.0, 1.0, 0.0, 0.0, 1.0, &p);
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
        precursor: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
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
        precursor: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
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
        precursor: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
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
        precursor: build_field_ledger(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
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

fn d008_reference_transport_params() -> SimParams {
    let mut p = SimParams::default();
    p.equation_version = EquationVersion::MembraneMetabolismV1;
    p.d_a = 0.040;
    p.beta_c = 4.6;
    p.beta_a = 4.6;
    p.beta_n = 1.2;
    p.beta_f = 1.2;
    p.beta_w = 0.2;
    p.m_max = 1.0;
    p.d_m = 0.001;
    p.k_membrane_decay = 0.002;
    p.k_membrane_detach = 0.020;
    p.k_c_membrane = 0.10;
    p.reactions_enabled = false;
    p.phase_separation_enabled = false;
    p.d008_stage_mode = D008StageMode::Transport;
    p
}

fn prepare_planar_transport(sim: &mut Simulation, species: TransportSpecies, membrane: f64) {
    sim.observer_enabled = false;
    sim.fields.catalyst.fill(0.0);
    sim.fields.activated.fill(0.0);
    sim.fields.nutrient.fill(0.0);
    sim.fields.fuel.fill(0.0);
    sim.fields.waste.fill(0.0);
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) {
            sim.fields.structure[idx] = 0.5;
            sim.fields.membrane[idx] = membrane;
        }
    }
    let width = sim.grid.width;
    let center_x = sim.grid.cx;
    let dish_mask = sim.grid.dish_mask.clone();
    let field = match species {
        TransportSpecies::Catalyst => &mut sim.fields.catalyst,
        TransportSpecies::Activated => &mut sim.fields.activated,
        TransportSpecies::Nutrient => &mut sim.fields.nutrient,
        TransportSpecies::Fuel => &mut sim.fields.fuel,
        TransportSpecies::Waste => &mut sim.fields.waste,
    };
    for (idx, value) in field.iter_mut().enumerate() {
        if dish_mask[idx] && (idx % width) as f64 <= center_x {
            *value = 1.0;
        }
    }
}

fn species_transport_accounting(sim: &Simulation, species: TransportSpecies) -> f64 {
    match species {
        TransportSpecies::Catalyst => sim.transport_accounting.last_step.catalyst.absolute_crossed_face_flux,
        TransportSpecies::Activated => sim.transport_accounting.last_step.activated.absolute_crossed_face_flux,
        TransportSpecies::Nutrient => sim.transport_accounting.last_step.nutrient.absolute_crossed_face_flux,
        TransportSpecies::Fuel => sim.transport_accounting.last_step.fuel.absolute_crossed_face_flux,
        TransportSpecies::Waste => sim.transport_accounting.last_step.waste.absolute_crossed_face_flux,
    }
}

#[test]
fn test_v2_transport_matches_v1() {
    let membranes = [0.0, 0.5, 1.0];
    let species = [
        TransportSpecies::Catalyst,
        TransportSpecies::Activated,
        TransportSpecies::Nutrient,
        TransportSpecies::Fuel,
        TransportSpecies::Waste,
    ];
    for &membrane in &membranes {
        for &sp in &species {
            let mut v1 = Simulation::new(d008_reference_transport_params());
            let mut v2 = Simulation::new(v2_params());
            v2.params.d008_stage_mode = D008StageMode::Transport;
            v2.params.reactions_enabled = false;
            v2.params.phase_separation_enabled = false;
            prepare_planar_transport(&mut v1, sp, membrane);
            prepare_planar_transport(&mut v2, sp, membrane);
            assert!(v1.step());
            assert!(v2.step());
            assert_eq!(
                species_transport_accounting(&v1, sp),
                species_transport_accounting(&v2, sp),
                "flux mismatch species={sp:?} membrane={membrane}"
            );
            for (a, b) in v1.fields.catalyst.iter().zip(v2.fields.catalyst.iter()) {
                assert!((a - b).abs() <= 1e-14, "catalyst field drift");
            }
            for (a, b) in v1.fields.activated.iter().zip(v2.fields.activated.iter()) {
                assert!((a - b).abs() <= 1e-14, "activated field drift");
            }
            for (a, b) in v1.fields.nutrient.iter().zip(v2.fields.nutrient.iter()) {
                assert!((a - b).abs() <= 1e-14, "nutrient field drift");
            }
            for (a, b) in v1.fields.fuel.iter().zip(v2.fields.fuel.iter()) {
                assert!((a - b).abs() <= 1e-14, "fuel field drift");
            }
            for (a, b) in v1.fields.waste.iter().zip(v2.fields.waste.iter()) {
                assert!((a - b).abs() <= 1e-14, "waste field drift");
            }
        }
    }
}

#[test]
fn test_v2_membrane_localization() {
    let mut params = v2_params();
    params.d008_stage_b_enabled = true;
    let mut sim = Simulation::new(params.clone());
    sim.observer_enabled = false;
    let calibration = membrane_calibration(
        &sim.fields.structure,
        &sim.fields.catalyst,
        &sim.fields.activated,
        &sim.fields.membrane,
        &sim.grid.dish_mask,
        &params,
    );
    sim.params.k_membrane = membrane_candidates(calibration.k_required)[1];
    for idx in 0..sim.fields.membrane.len() {
        sim.fields.membrane[idx] = if sim.grid.in_dish(idx) {
            0.5 * interface_weight(sim.fields.structure[idx])
        } else {
            0.0
        };
    }
    let mut minimum_after_transient = f64::INFINITY;
    for _ in 0..16_000 {
        assert!(sim.step());
        if sim.substep > 15_000 {
            minimum_after_transient = minimum_after_transient.min(
                membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane)
                    .localization_fraction,
            );
        }
    }
    let partition = membrane_partition(&sim.grid, &sim.fields.structure, &sim.fields.membrane);
    assert!(
        minimum_after_transient >= 0.90,
        "localization={minimum_after_transient}"
    );
    assert!(sim.membrane_accounting.cumulative.synthesis > 0.0);
    assert!(sim.membrane_accounting.cumulative.decay > 0.0);
    assert!(sim.membrane_accounting.cumulative.detachment > 0.0);
    assert!(sim
        .fields
        .membrane
        .iter()
        .all(|&m| m.is_finite() && (0.0..=params.m_max).contains(&m)));
    assert!(partition.total_mass > f64::EPSILON);
    assert!(sim.membrane_accounting.cumulative.residual.abs() < 1e-8);
}

fn v2_stage_c_params() -> SimParams {
    let mut params = v2_params();
    params.d008_stage_mode = D008StageMode::ActivatedMetabolism;
    params.diffusion_enabled = false;
    params.phase_separation_enabled = false;
    params
}

#[test]
fn test_v2_metabolic_reactor_bounded() {
    let params = v2_stage_c_params();
    let mut sim = Simulation::new(params);
    sim.observer_enabled = false;
    for idx in 0..sim.fields.structure.len() {
        if sim.grid.in_dish(idx) {
            sim.fields.structure[idx] = 0.5;
            sim.fields.membrane[idx] = 0.5;
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.nutrient[idx] = 0.8;
            sim.fields.fuel[idx] = 0.7;
            sim.fields.activated[idx] = 0.2;
        }
    }
    let rates = activated_metabolism_rates(1.0, 0.4, 0.8, 0.7, 0.2, &sim.params);
    assert!(rates.activation > 0.0);
    assert_eq!(
        activated_metabolism_rates(1.0, 0.0, 0.8, 0.7, 0.2, &sim.params).activation,
        0.0
    );
    assert_eq!(
        activated_metabolism_rates(1.0, 0.4, 0.0, 0.7, 0.2, &sim.params).activation,
        0.0
    );
    assert_eq!(
        activated_metabolism_rates(1.0, 0.4, 0.8, 0.0, 0.2, &sim.params).activation,
        0.0
    );
    assert!(activated_metabolism_rates(1.0, 0.4, 0.0, 0.0, 0.2, &sim.params).reproduction > 0.0);
    assert_eq!(
        activated_metabolism_rates(1.0, 0.4, 1.0, 1.0, 0.0, &sim.params).reproduction,
        0.0
    );
    for _ in 0..100 {
        assert!(sim.step());
    }
    assert!(sim.metabolism_accounting.cumulative.activation > 0.0);
    assert!(sim.metabolism_accounting.cumulative.reproduction > 0.0);
    assert!(sim
        .fields
        .catalyst
        .iter()
        .all(|&v| v.is_finite() && (0.0..=sim.params.d008_c_max).contains(&v)));
    assert!(sim
        .fields
        .activated
        .iter()
        .all(|&v| v.is_finite() && (0.0..=sim.params.d008_a_max).contains(&v)));
    assert!(chemistry_core::stage_c_clamp_negligible(
        &sim.metabolism_accounting.cumulative
    ));
    let material = build_material_equivalent_step(&sim.accounting.last_step);
    assert!(material_step_closes(&material), "{material:?}");
    assert!(sim.accounting.cumulative_within_tolerance());
}

fn v2_stage_d_params() -> SimParams {
    let mut params = d008_reference_transport_params();
    params.equation_version = EquationVersion::MembraneMetabolismV2Conservative;
    params.eta_c = 1.0;
    params.eta_phi = 1.0;
    params.eta_m = 1.0;
    params.k_membrane = 0.19748231883326484;
    params.k_d008_activation = 0.020;
    params.k_d008_reproduction = 0.040;
    params.k_d008_activated_decay = 0.005;
    params.k_d008_catalyst_turnover = 0.002;
    params.d008_a_max = 1.0;
    params.d008_c_max = 1.0;
    params.d008_stage_mode = D008StageMode::FixedCompartment;
    params.d008_stage_b_enabled = false;
    params.diffusion_enabled = true;
    params.phase_separation_enabled = false;
    params.reactions_enabled = true;
    params
}

fn v2_stage_d_simulation(radius: f64) -> Simulation {
    let mut sim = Simulation::new(v2_stage_d_params());
    sim.observer_enabled = false;
    for idx in 0..sim.fields.structure.len() {
        if !sim.grid.in_dish(idx) {
            continue;
        }
        let x = (idx % sim.grid.width) as f64 - sim.grid.cx;
        let y = (idx / sim.grid.width) as f64 - sim.grid.cy;
        let distance = (x * x + y * y).sqrt();
        let phi = 0.5 * (1.0 - ((distance - radius) / 2.0).tanh());
        sim.fields.structure[idx] = phi;
        sim.fields.membrane[idx] = interface_weight(phi);
        if phi >= 0.5 {
            sim.fields.catalyst[idx] = 0.4;
            sim.fields.activated[idx] = 0.2;
            sim.fields.nutrient[idx] = 0.2;
            sim.fields.fuel[idx] = 0.2;
            sim.fields.waste[idx] = 0.5;
        } else {
            sim.fields.nutrient[idx] = sim.params.n_reservoir;
            sim.fields.fuel[idx] = sim.params.f_reservoir;
            sim.fields.waste[idx] = sim.params.w_reservoir;
        }
    }
    sim
}

#[test]
fn test_v2_fixed_compartment_retention() {
    let mut sim = v2_stage_d_simulation(16.0);
    let structure_hash = chemistry_core::field_sha256_stable(&sim.fields.structure);
    let membrane_hash = chemistry_core::field_sha256_stable(&sim.fields.membrane);
    assert!(sim.step());
    assert_eq!(
        chemistry_core::field_sha256_stable(&sim.fields.structure),
        structure_hash
    );
    assert_eq!(
        chemistry_core::field_sha256_stable(&sim.fields.membrane),
        membrane_hash
    );
    assert!(
        sim.transport_accounting
            .last_step
            .nutrient
            .interior_net_flux_rate
            > 0.0
    );
    assert!(
        sim.transport_accounting
            .last_step
            .fuel
            .interior_net_flux_rate
            > 0.0
    );
    assert!(
        sim.transport_accounting
            .last_step
            .waste
            .interior_net_flux_rate
            < 0.0
    );
    assert!(sim.metabolism_accounting.last_step.activation > 0.0);
    assert!(sim.metabolism_accounting.last_step.reproduction > 0.0);
    assert!(sim.accounting.cumulative_within_tolerance());
    let material = build_material_equivalent_step(&sim.accounting.last_step);
    assert!(material_step_closes(&material), "{material:?}");
}

#[test]
fn write_v2_stage_bcd_artifact_paths_exist_after_runner() {
    // Document expected governed artifact locations (populated by experiment-runner D012).
    for subdir in [
        "v2_stage_b_localization",
        "v2_stage_c_metabolism",
        "v2_stage_d_fixed_compartment",
    ] {
        let path = format!(
            "{}/../../experiments/generated/d012/{subdir}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::create_dir_all(&path).expect("artifact dir");
    }
}

use chemistry_core::d011_analysis::{
    joint_overlap_pass, quasi_steady_report, ComponentBalance, ConvergenceClassification,
    JointBalanceMetrics, QuasiSteadyReport, SteadyWindowSnapshot, D011_TEST_WINDOW,
};
use chemistry_core::d012_analysis::{
    all_four_balances_pass, classify_v2_stage_e, count_yield_changes,
    restoring_radius_from_g_structure, resource_throughput_pass, v2_stage_e_pass,
    yield_adjustment_allowed, D012RadiusBalancePoint, D012StageEClassification,
    D012_V2_REQUIRED_WINDOWS, YieldComponent,
};
use chemistry_core::d012_accounting::MaterialEquivalentStep;

fn balanced_metrics() -> JointBalanceMetrics {
    JointBalanceMetrics {
        structure: ComponentBalance {
            q: 1.0,
            g: 0.0,
            production: 1.0,
            loss: 1.0,
        },
        catalyst: ComponentBalance {
            q: 1.0,
            g: 0.0,
            production: 1.0,
            loss: 1.0,
        },
        membrane: ComponentBalance {
            q: 1.0,
            g: 0.0,
            production: 1.0,
            loss: 1.0,
        },
        activated: ComponentBalance {
            q: 1.0,
            g: 0.0,
            production: 1.0,
            loss: 1.0,
        },
        catalyst_retention: 0.9,
        activated_retention: 0.9,
        membrane_localization: 0.95,
        nutrient_influx: 1.0,
        fuel_influx: 1.0,
        waste_efflux: 1.0,
    }
}

fn steady_windows() -> Vec<SteadyWindowSnapshot> {
    (0..4)
        .map(|i| SteadyWindowSnapshot {
            start_step: i * 1000,
            end_step: (i + 1) * 1000,
            simulated_time_start: i as f64,
            simulated_time_end: (i + 1) as f64,
            mass_c: 100.0,
            mass_a: 100.0,
            mass_m: 100.0,
            mean_n_interior: 0.2,
            mean_f_interior: 0.2,
            mean_w_interior: 0.5,
            structure_production: 100.0 + i as f64,
            structure_decay: 100.0 + i as f64,
            catalyst_reproduction: 100.0 + i as f64,
            catalyst_turnover: 100.0 + i as f64,
            membrane_synthesis: 100.0 + i as f64,
            membrane_loss: 100.0 + i as f64,
            activation: 100.0 + i as f64,
            activated_loss: 100.0 + i as f64,
            nutrient_transport_interior: 100.0 + i as f64,
            fuel_transport_interior: 100.0 + i as f64,
            waste_transport_interior: -100.0 - i as f64,
        })
        .collect()
}

fn closed_material() -> MaterialEquivalentStep {
    MaterialEquivalentStep {
        total_before: 10.0,
        total_after: 10.0,
        observed_change: 0.0,
        reservoir_input: 0.0,
        waste_clearance: 0.0,
        numerical_correction: 0.0,
        boundary_exchange: 0.0,
        residual: 0.0,
        relative_residual: 0.0,
    }
}

fn restoring_points() -> Vec<D012RadiusBalancePoint> {
    vec![
        D012RadiusBalancePoint {
            radius: 18.0,
            g_structure: 0.001,
            joint_overlap: false,
            quasi_steady: true,
        },
        D012RadiusBalancePoint {
            radius: 22.0,
            g_structure: 0.0,
            joint_overlap: true,
            quasi_steady: true,
        },
        D012RadiusBalancePoint {
            radius: 26.0,
            g_structure: -0.001,
            joint_overlap: false,
            quasi_steady: true,
        },
    ]
}

#[test]
fn test_v2_stage_e_requires_quasi_steady_state() {
    let metrics = balanced_metrics();
    let quasi = QuasiSteadyReport {
        window_size: D011_TEST_WINDOW,
        converged_windows: 0,
        required_windows: D012_V2_REQUIRED_WINDOWS,
        converged: false,
        window_slopes: vec![],
    };
    assert!(!v2_stage_e_pass(
        &quasi,
        &metrics,
        &closed_material(),
        &restoring_points()
    ));
    let converged = quasi_steady_report(&steady_windows(), D011_TEST_WINDOW, 3);
    assert!(converged.converged);
}

#[test]
fn test_v2_stage_e_requires_all_four_balances() {
    let mut metrics = balanced_metrics();
    assert!(all_four_balances_pass(&metrics));
    metrics.membrane.q = 1.5;
    assert!(!all_four_balances_pass(&metrics));
    assert!(!joint_overlap_pass(&metrics));
}

#[test]
fn test_v2_stage_e_requires_restoring_radius() {
    assert!(restoring_radius_from_g_structure(&restoring_points()));
    let flat = vec![
        D012RadiusBalancePoint {
            radius: 18.0,
            g_structure: 0.01,
            joint_overlap: false,
            quasi_steady: true,
        },
        D012RadiusBalancePoint {
            radius: 26.0,
            g_structure: 0.01,
            joint_overlap: false,
            quasi_steady: true,
        },
    ];
    assert!(!restoring_radius_from_g_structure(&flat));
}

#[test]
fn test_v2_stage_e_requires_resource_throughput() {
    let mut metrics = balanced_metrics();
    assert!(resource_throughput_pass(&metrics));
    metrics.waste_efflux = 0.0;
    assert!(!resource_throughput_pass(&metrics));
}

#[test]
fn test_v2_stage_e_requires_total_conservation() {
    let mut sim = v2_stage_d_simulation(22.0);
    sim.observer_enabled = false;
    for _ in 0..200 {
        assert!(sim.step());
    }
    assert!(sim.accounting.cumulative_within_tolerance());
    let material = build_material_equivalent_step(&sim.accounting.last_step);
    assert!(material_step_closes(&material), "{material:?}");
}

#[test]
fn test_yield_branch_changes_one_component() {
    let mut params = v2_params();
    let before = (params.eta_c, params.eta_phi, params.eta_m);
    chemistry_core::apply_yield_change(&mut params, YieldComponent::Structure, 0.85).unwrap();
    assert_eq!(count_yield_changes(before, (params.eta_c, params.eta_phi, params.eta_m)), 1);
}

#[test]
fn test_underproduced_component_yield_is_not_reduced() {
    let under = ComponentBalance {
        q: 0.95,
        g: 0.0,
        production: 1.0,
        loss: 1.0,
    };
    assert!(!yield_adjustment_allowed(under, 1.0, 0.85));
    let over = ComponentBalance {
        q: 1.05,
        g: 0.0,
        production: 1.0,
        loss: 1.0,
    };
    assert!(yield_adjustment_allowed(over, 1.0, 0.85));
}

#[test]
fn test_v2_stage_e_not_converged_cannot_claim_no_solution() {
    let metrics = balanced_metrics();
    let quasi = QuasiSteadyReport {
        window_size: 1000,
        converged_windows: 0,
        required_windows: 3,
        converged: false,
        window_slopes: vec![],
    };
    let class = classify_v2_stage_e(
        &quasi,
        &metrics,
        true,
        &restoring_points(),
        ConvergenceClassification::NotConverged,
        false,
    );
    assert_eq!(class, D012StageEClassification::NotConverged);
    let unresolved = classify_v2_stage_e(
        &quasi,
        &metrics,
        true,
        &restoring_points(),
        ConvergenceClassification::NotConverged,
        true,
    );
    assert_eq!(
        unresolved,
        D012StageEClassification::LongTransientUnresolved
    );
}

#[test]
fn write_v2_stage_e_artifact_paths_exist_after_runner() {
    for subdir in [
        "v2_stage_e_reference",
        "v2_sensitivity",
        "v2_joint_candidates",
        "v2_yield_candidates",
        "v2_robust_overlap",
    ] {
        let path = format!(
            "{}/../../experiments/generated/d012/{subdir}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::create_dir_all(&path).expect("artifact dir");
    }
}

