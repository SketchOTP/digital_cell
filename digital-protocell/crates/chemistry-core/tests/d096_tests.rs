use chemistry_core::d096_allocation::{
    allocation_schema_load_ok, apply_assay_environment, expression_step, mutate_allocation_genotype,
    partition_catalysts, AllocationGenotype, AllocationState, pre_fission_assay, AllocationParams,
    AssayEnvironment,
    EQUATION_VERSION_FINITE_CATALYTIC_ALLOCATION,
    FINITE_ALLOCATION_SCHEMA_VERSION,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh, EQUATION_VERSION_MATERIAL_MESH};
use chemistry_core::mesh_reactions::{reactions_step, ReactionParams};
use chemistry_core::mesh_fission::{try_local_fission, FissionParams};

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
fn d096_selecting_environments_are_observable_only_as_local_resources_and_damage() {
    let mut h = mesh();
    let mut b = mesh();
    let mut neutral = mesh();
    let h_input = apply_assay_environment(&mut h, AssayEnvironment::H, 0);
    let b_input = apply_assay_environment(&mut b, AssayEnvironment::B, 0);
    let n_input = apply_assay_environment(&mut neutral, AssayEnvironment::Neutral, 0);

    assert!(h_input.nutrient / h_input.fuel > n_input.nutrient / n_input.fuel);
    assert!(b_input.structural_damage > 0.0 && b_input.membrane_damage > 0.0);
    assert_eq!(n_input.structural_damage + n_input.membrane_damage, 0.0);
    assert_ne!(h.exterior.n, neutral.exterior.n);
    assert!(b.edges[0].m < neutral.edges[0].m);
    for organism in [h, b, neutral] {
        let serialized = serde_json::to_string(&organism).unwrap();
        assert!(!serialized.contains("AssayEnvironment"));
        assert!(!serialized.contains("\"H\""));
        assert!(!serialized.contains("\"B\""));
        assert!(!serialized.contains("treatment"));
    }
}

#[test]
fn d096_reciprocal_prefission_effect_exceeds_neutral_difference() {
    let processing = AllocationGenotype([0.55, 0.25, 0.05, 0.15]);
    let repair = AllocationGenotype([0.10, 0.20, 0.55, 0.15]);
    let mut h_effects = Vec::new();
    let mut b_effects = Vec::new();
    let mut n_reserve = Vec::new();
    let mut n_material = Vec::new();
    for seed in 1..=8 {
        let hp = pre_fission_assay(processing, AssayEnvironment::H, seed, 1_000);
        let hr = pre_fission_assay(repair, AssayEnvironment::H, seed, 1_000);
        let bp = pre_fission_assay(processing, AssayEnvironment::B, seed, 1_000);
        let br = pre_fission_assay(repair, AssayEnvironment::B, seed, 1_000);
        let np = pre_fission_assay(processing, AssayEnvironment::Neutral, seed, 1_000);
        let nr = pre_fission_assay(repair, AssayEnvironment::Neutral, seed, 1_000);
        h_effects.push(hp.reserve_change - hr.reserve_change);
        b_effects.push(br.final_material - bp.final_material);
        n_reserve.push(np.reserve_change - nr.reserve_change);
        n_material.push(nr.final_material - np.final_material);
        assert!(hp.survived && hr.survived && bp.survived && br.survived);
    }
    let mean = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;
    eprintln!(
        "processing_hash={} repair_hash={}; H reserve effects={h_effects:?} mean={} neutral={n_reserve:?} mean={}; B material effects={b_effects:?} mean={} neutral={n_material:?} mean={}",
        processing.candidate_hash(&AllocationParams::default()),
        repair.candidate_hash(&AllocationParams::default()),
        mean(&h_effects),
        mean(&n_reserve),
        mean(&b_effects),
        mean(&n_material)
    );
    assert!(h_effects.iter().all(|x| *x > 0.0));
    assert!(b_effects.iter().all(|x| *x > 0.0));
    assert!(mean(&h_effects) > mean(&n_reserve));
    assert!(mean(&b_effects) > mean(&n_material));
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

#[test]
fn d096_mutation_off_is_exact_and_environment_blind() {
    let mut params = AllocationParams::default();
    params.mutation_probability = 0.0;
    let parent = AllocationGenotype::pulse();
    let records = ["H", "B", "Neutral"]
        .into_iter()
        .map(|_| mutate_allocation_genotype(parent, &params, 0xD096_004B).unwrap())
        .collect::<Vec<_>>();
    for record in &records {
        assert!(!record.mutation_occurred);
        assert_eq!(record.pre_genotype, record.post_genotype);
        assert_eq!(record.pre_genotype.candidate_hash(&params), record.post_genotype.candidate_hash(&params));
    }
    assert!(records.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn d096_mutation_frequency_and_simplex_transfer_are_bounded() {
    let mut params = AllocationParams::default();
    params.mutation_probability = 0.01;
    params.mutation_sigma = 0.15;
    let parent = AllocationGenotype::neutral();
    let mut changed = 0usize;
    let mut checked_transfer = false;
    for seed in 0..10_000_u64 {
        let record = mutate_allocation_genotype(parent, &params, seed).unwrap();
        changed += usize::from(record.mutation_occurred);
        assert!(record.post_genotype.valid(&params));
        if record.mutation_occurred {
            checked_transfer = true;
            let source = record.source.unwrap();
            let target = record.target.unwrap();
            assert_ne!(source, target);
            assert!(record.raw_abs_normal >= record.applied_delta);
            assert!(record.applied_delta > 0.0);
            for i in 0..4 {
                let delta = record.post_genotype.0[i] - record.pre_genotype.0[i];
                if i == source {
                    assert!((delta + record.applied_delta).abs() <= 1e-15);
                } else if i == target {
                    assert!((delta - record.applied_delta).abs() <= 1e-15);
                } else {
                    assert!(delta.abs() <= 1e-15);
                }
            }
        }
    }
    assert!((70..=130).contains(&changed), "observed mutation count={changed}");
    assert!(checked_transfer);
}

#[test]
fn d096_bounded_mutation_allows_zero_capped_transfer() {
    let mut params = AllocationParams::default();
    params.mutation_probability = 1.0;
    let parent = AllocationGenotype([0.0, 1.0, 0.0, 0.0]);
    let mut found_zero = false;
    for seed in 1..10_000_u64 {
        let record = mutate_allocation_genotype(parent, &params, seed).unwrap();
        if record.applied_delta == 0.0 {
            found_zero = true;
            assert!(record.mutation_occurred);
            assert!(record.post_genotype.valid(&params));
            assert_eq!(record.source.is_some(), record.target.is_some());
            break;
        }
    }
    assert!(found_zero, "expected at least one capped zero-transfer decision");
}

#[test]
fn d096_physical_fission_partitions_catalysts_and_copies_genotype() {
    let params = AllocationParams::default();
    let mut parent = mesh();
    let center = parent.centroid();
    for vertex in &mut parent.vertices {
        vertex[0] = center[0] + (vertex[0] - center[0]) * 1.55;
        vertex[1] = center[1] + (vertex[1] - center[1]) * 0.72;
    }
    parent.interior.a = 2.0;
    parent.interior.c = 1.0;
    parent.enable_finite_allocation(AllocationGenotype::pulse(), &params);
    parent.finite_allocation = Some(AllocationState {
        genotype: AllocationGenotype::pulse(),
        catalysts: [0.11, 0.22, 0.33, 0.44],
    });
    let (daughter_a, daughter_b, event) =
        try_local_fission(&parent, &FissionParams::default()).expect("controlled D-096 fission");
    let audit = event
        .partition
        .catalyst_partition
        .expect("D-096 catalyst audit");
    assert!(audit.conserved);
    assert!(audit.max_residual <= 1e-12);
    assert_eq!(daughter_a.finite_allocation.unwrap().genotype, parent.finite_allocation.unwrap().genotype);
    assert_eq!(daughter_b.finite_allocation.unwrap().genotype, parent.finite_allocation.unwrap().genotype);
    for i in 0..4 {
        assert!((daughter_a.finite_allocation.unwrap().catalysts[i]
            + daughter_b.finite_allocation.unwrap().catalysts[i]
            - parent.finite_allocation.unwrap().catalysts[i]).abs() <= 1e-12);
    }
}

#[test]
fn d096_partition_helper_preserves_genotype_and_physical_sum() {
    let parent = AllocationState {
        genotype: AllocationGenotype::damage(),
        catalysts: [0.3, 0.2, 0.1, 0.4],
    };
    let (a, b, audit) = partition_catalysts(parent, 0.37, 0.63);
    assert_eq!(a.genotype, parent.genotype);
    assert_eq!(b.genotype, parent.genotype);
    assert!(audit.conserved);
    assert_eq!(audit.pre_catalyst, parent.catalysts);
}
