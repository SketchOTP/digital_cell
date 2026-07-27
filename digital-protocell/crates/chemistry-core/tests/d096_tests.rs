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
