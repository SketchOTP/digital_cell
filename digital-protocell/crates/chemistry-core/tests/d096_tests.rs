use chemistry_core::d096_allocation::{
    allocation_schema_load_ok, expression_step, AllocationGenotype, AllocationParams,
    EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION, FINITE_ALLOCATION_SCHEMA_VERSION,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, EQUATION_VERSION_MATERIAL_MESH};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};

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

fn expressed(genotype: AllocationGenotype) -> MaterialMesh {
    let params = AllocationParams::default();
    let mut candidate = mesh();
    candidate.interior.a = 2.0;
    candidate.enable_finite_allocation(genotype, &params);
    for _ in 0..20 {
        expression_step(&mut candidate, &params, 0.1).unwrap();
    }
    candidate
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
fn d096_processing_expression_is_monotonic_local_and_substrate_dependent() {
    let reaction = ReactionParams::default();
    let mut expression = Vec::new();
    let mut conversion = Vec::new();
    for processing in [0.0, 0.2, 0.4, 0.6, 0.8] {
        let mut candidate = expressed(AllocationGenotype([
            processing,
            0.1,
            0.0,
            0.9 - processing,
        ]));
        expression.push(candidate.finite_allocation.unwrap().catalysts[0]);
        candidate.interior.c = 1.0;
        candidate.interior.n = 1.0;
        candidate.interior.f = 1.0;
        conversion.push(reactions_step(&mut candidate, &reaction, 0.01, false, true).a_produced);
    }
    assert!(expression.windows(2).all(|w| w[1] > w[0]));
    assert!(conversion.windows(2).all(|w| w[1] > w[0]));

    let mut no_substrate = expressed(AllocationGenotype::pulse());
    no_substrate.interior.c = 1.0;
    no_substrate.interior.n = 0.0;
    no_substrate.interior.f = 1.0;
    assert_eq!(
        reactions_step(&mut no_substrate, &reaction, 0.01, false, true).a_produced,
        0.0
    );
}

#[test]
fn d096_repair_expression_is_monotonic_and_requires_local_damage_substrate() {
    let reaction = ReactionParams::default();
    let mut expression = Vec::new();
    let mut repair_flux = Vec::new();
    for repair in [0.0, 0.2, 0.4, 0.6, 0.8] {
        let mut candidate =
            expressed(AllocationGenotype([0.0, 0.1, repair, 0.9 - repair]));
        expression.push(candidate.finite_allocation.unwrap().catalysts[2]);
        candidate.interior.c = 1.0;
        candidate.interior.a = 1.0;
        candidate.edges[0].m *= 0.5;
        repair_flux.push(reactions_step(&mut candidate, &reaction, 0.01, true, false).m_produced);
    }
    assert!(expression.windows(2).all(|w| w[1] > w[0]));
    assert!(repair_flux.windows(2).all(|w| w[1] > w[0]));

    let mut no_damage = expressed(AllocationGenotype::damage());
    no_damage.interior.c = 1.0;
    no_damage.interior.a = 1.0;
    let before = no_damage.total_structural_mass();
    let repaired = reactions_step(&mut no_damage, &reaction, 0.01, true, false).m_produced;
    assert!(repaired <= before * 1e-3);
}

fn processing_flux(genotype: AllocationGenotype) -> f64 {
    let mut candidate = expressed(genotype);
    candidate.interior.c = 1.0;
    candidate.interior.n = 1.0;
    candidate.interior.f = 1.0;
    reactions_step(
        &mut candidate,
        &ReactionParams::default(),
        0.01,
        false,
        true,
    )
    .a_produced
}

fn repair_flux(genotype: AllocationGenotype) -> f64 {
    let mut candidate = expressed(genotype);
    candidate.interior.c = 1.0;
    candidate.interior.a = 1.0;
    candidate.edges[0].m *= 0.5;
    reactions_step(
        &mut candidate,
        &ReactionParams::default(),
        0.01,
        true,
        false,
    )
    .m_produced
}

#[test]
fn d096_tradeoff_occurs_in_conserved_processing_and_repair_fluxes() {
    let processing = AllocationGenotype([0.55, 0.25, 0.05, 0.15]);
    let balanced = AllocationGenotype::neutral();
    let repair = AllocationGenotype([0.10, 0.20, 0.55, 0.15]);
    let p = [processing_flux(processing), processing_flux(balanced), processing_flux(repair)];
    let r = [repair_flux(processing), repair_flux(balanced), repair_flux(repair)];

    assert!(p[0] > p[1] && p[1] > p[2]);
    assert!(r[2] > r[1] && r[1] > r[0]);
    assert!(!(p[1] >= p[0] && r[1] >= r[2]));
    assert!(!(p[0] >= p[2] && r[0] >= r[2]));
    assert!(!(p[2] >= p[0] && r[2] >= r[0]));
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
