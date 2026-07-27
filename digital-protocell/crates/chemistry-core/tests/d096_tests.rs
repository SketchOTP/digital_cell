use chemistry_core::d096_allocation::{
    allocation_schema_load_ok, expression_step, AllocationGenotype, AllocationParams,
    EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION, FINITE_ALLOCATION_SCHEMA_VERSION,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, EQUATION_VERSION_MATERIAL_MESH};

fn mesh() -> MaterialMesh {
    MaterialMesh::seed_regular(
        12,
        8.0,
        0.0,
        0.0,
        1.0,
        0.8,
        LumpedChem::default(),
        LumpedChem::default(),
        1.0,
    )
}

#[test]
fn d096_rejected_steps_are_atomic_and_organisms_have_no_treatment_label() {
    let params = AllocationParams::default();
    let mut candidate = mesh();
    candidate.interior.a = 1.0;
    candidate.enable_finite_allocation(AllocationGenotype::neutral(), &params);
    let before = serde_json::to_value(&candidate).unwrap();

    assert!(expression_step(&mut candidate, &params, f64::NAN).is_err());
    assert_eq!(serde_json::to_value(&candidate).unwrap(), before);
    let serialized = serde_json::to_string(&candidate).unwrap();
    assert!(!serialized.contains("treatment"));
    assert!(!serialized.contains("\"environment\""));
}

#[test]
fn d096_expression_conserves_budget_material_and_activation_accounting() {
    let params = AllocationParams::default();
    let mut candidate = mesh();
    candidate.interior.a = 2.0;
    candidate.enable_finite_allocation(AllocationGenotype::neutral(), &params);
    let area = candidate.area();
    let m0 = candidate.total_structural_mass();
    let a0 = candidate.interior.a * area;
    let ledger = expression_step(&mut candidate, &params, 0.1).unwrap();
    let state = candidate.finite_allocation.unwrap();

    assert!((state.genotype.0.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    assert!((ledger.material_consumed - state.catalysts.iter().sum::<f64>()).abs() < 1e-12);
    assert!(
        (m0 - candidate.total_structural_mass() - ledger.material_consumed).abs() < 1e-10
    );
    assert!(
        (a0 - candidate.interior.a * area
            - ledger.activation_consumed
            - ledger.maintenance_consumed)
            .abs()
            < 1e-10
    );
    assert!(ledger.synthesis.iter().all(|x| *x > 0.0));
}

#[test]
fn d096_no_expression_controls_and_complementarity_hold_without_normalization() {
    let params = AllocationParams::default();
    let mut no_a = mesh();
    no_a.enable_finite_allocation(AllocationGenotype::neutral(), &params);
    let before = serde_json::to_value(&no_a).unwrap();
    assert!(expression_step(&mut no_a, &params, 0.1).is_err());
    assert_eq!(serde_json::to_value(&no_a).unwrap(), before);

    let processing_heavy = AllocationGenotype([0.55, 0.25, 0.05, 0.15]);
    let repair_heavy = AllocationGenotype([0.10, 0.20, 0.55, 0.15]);
    assert!(processing_heavy.valid(&params));
    assert!(repair_heavy.valid(&params));
    assert!(processing_heavy.0[0] > repair_heavy.0[0]);
    assert!(processing_heavy.0[2] < repair_heavy.0[2]);
}

#[test]
fn d096_equation_snapshot_and_candidate_identity_are_isolated() {
    let legacy = mesh();
    let legacy_equation = legacy.equation_id.clone();
    let params = AllocationParams::default();
    let genotype = AllocationGenotype::neutral();
    let mut candidate = legacy.clone();
    candidate.enable_finite_allocation(genotype, &params);

    assert_eq!(legacy_equation, EQUATION_VERSION_MATERIAL_MESH);
    assert_eq!(
        candidate.equation_id,
        EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION
    );
    assert_eq!(candidate.schema_version, FINITE_ALLOCATION_SCHEMA_VERSION);
    assert!(!allocation_schema_load_ok(&legacy, &params));
    assert!(allocation_schema_load_ok(&candidate, &params));
    assert_ne!(
        genotype.candidate_hash(&params),
        AllocationGenotype::pulse().candidate_hash(&params)
    );
}
