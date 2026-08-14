use chemistry_core::d096_allocation::{
    expression_step, mutate_allocation_genotype, partition_catalysts, AllocationGenotype,
    AllocationParams, AllocationState,
};
use chemistry_core::material_mesh::{LumpedChem, MaterialMesh};
use chemistry_core::mesh_fission::{try_local_fission, FissionParams};
use evolution_harness::{
    d096_mutation_stream_seed, DigitalCellMeshAdapter, MutationContext, MutationProtocolV1,
    OrganismAdapter,
};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Serialize)]
struct MutationOffFounderEvidence {
    founder_class: &'static str,
    exact_copy: bool,
    hash_stable: bool,
    candidate_hash: String,
}

#[derive(Debug, Serialize)]
struct D096Gate6EvidenceV1 {
    schema: &'static str,
    equation: &'static str,
    mutation_probability: f64,
    mutation_sigma: f64,
    mutation_samples: u64,
    observed_mutations: u64,
    observed_mutation_frequency: f64,
    exact_copy_observed: bool,
    lawful_transfer_observed: bool,
    zero_capped_transfer_observed: bool,
    physical_partition_conserved: bool,
    physical_partition_max_residual: f64,
    stream_key_matrix_campaign_seeds: u64,
    stream_key_matrix_copy_ordinals: u64,
    stream_key_matrix_unique: u64,
    stream_pair_replay_equal: bool,
    stream_collision_pair_distinct: bool,
    frozen_candidate_hash: String,
    real_fission_daughter_count: u64,
    real_fission_partition_conserved: bool,
    real_daughter_mutation_on_observed: bool,
    real_daughter_mutation_provenance_recorded: bool,
    real_daughter_mutation_created_no_catalyst: bool,
    post_birth_expression_steps: u64,
    post_birth_expression_follows_mutated_genotype: bool,
    mutation_off_founder_classes: Vec<MutationOffFounderEvidence>,
}

fn real_fission_parent(genotype: AllocationGenotype, params: &AllocationParams) -> MaterialMesh {
    let mut parent = MaterialMesh::seed_regular(
        12,
        8.0,
        0.0,
        0.0,
        1.0,
        0.8,
        LumpedChem::default(),
        LumpedChem::default(),
        1.0,
    );
    let center = parent.centroid();
    for vertex in &mut parent.vertices {
        vertex[0] = center[0] + (vertex[0] - center[0]) * 1.55;
        vertex[1] = center[1] + (vertex[1] - center[1]) * 0.72;
    }
    parent.interior.a = 2.0;
    parent.interior.c = 1.0;
    parent.enable_finite_allocation(genotype, params);
    parent.finite_allocation = Some(AllocationState {
        genotype,
        catalysts: [0.11, 0.22, 0.33, 0.44],
    });
    parent
}

fn d096_mutation_protocol() -> MutationProtocolV1 {
    MutationProtocolV1 {
        schema: "MutationProtocolV1".into(),
        mutation_protocol_id: "d096_allocation_mutation_v1".into(),
        mutation_rate: 0.01,
        magnitude_distribution: "abs_normal".into(),
        bounds: "simplex_and_allocation_bounds".into(),
        provenance: "DC-SR-004B;D-096_GATE6;assay".into(),
    }
}

fn main() {
    let params = AllocationParams::default();
    assert!((params.mutation_probability - 0.01).abs() <= 1e-12);
    assert!((params.mutation_sigma - 0.15).abs() <= 1e-12);

    let parent = AllocationGenotype::neutral();
    let mut observed_mutations = 0_u64;
    let mut exact_copy_observed = false;
    let mut lawful_transfer_observed = false;
    for seed in 0..10_000_u64 {
        let record = mutate_allocation_genotype(parent, &params, seed).unwrap();
        assert!(record.post_genotype.valid(&params));
        if record.mutation_occurred {
            observed_mutations += 1;
            assert_ne!(record.pre_genotype, record.post_genotype);
            assert!(record.source.is_some() && record.target.is_some());
            lawful_transfer_observed = true;
        } else {
            assert_eq!(record.pre_genotype, record.post_genotype);
            exact_copy_observed = true;
        }
    }
    assert!((70..=130).contains(&observed_mutations));

    let boundary = AllocationGenotype([0.0, 1.0, 0.0, 0.0]);
    let zero_capped_transfer_observed = (1..10_000_u64).any(|seed| {
        let mut boundary_params = params;
        boundary_params.mutation_probability = 1.0;
        let record = mutate_allocation_genotype(boundary, &boundary_params, seed).unwrap();
        record.mutation_occurred && record.applied_delta == 0.0
    });
    assert!(zero_capped_transfer_observed);

    let physical_parent = AllocationState {
        genotype: parent,
        catalysts: [0.3, 0.2, 0.1, 0.4],
    };
    let (daughter_a, daughter_b, audit) = partition_catalysts(physical_parent, 0.37, 0.63);
    assert_eq!(daughter_a.genotype, physical_parent.genotype);
    assert_eq!(daughter_b.genotype, physical_parent.genotype);
    assert!(audit.conserved);

    let keys = (0..64_u64)
        .flat_map(|campaign_seed| {
            (0..256_u64)
                .map(move |copy_ordinal| d096_mutation_stream_seed(campaign_seed, copy_ordinal))
        })
        .collect::<HashSet<_>>();
    assert_eq!(keys.len(), 64 * 256);
    assert_eq!(
        d096_mutation_stream_seed(17, 1),
        d096_mutation_stream_seed(17, 1)
    );
    assert_ne!(
        d096_mutation_stream_seed(17, 1),
        d096_mutation_stream_seed(18, 0)
    );

    let mutation_protocol = d096_mutation_protocol();
    let mutation_none = MutationProtocolV1::default();
    let mut adapter = DigitalCellMeshAdapter {
        allocation_params: Some(params),
        ..DigitalCellMeshAdapter::default()
    };

    let founder_classes = [
        (
            "processing_heavy",
            AllocationGenotype([0.55, 0.25, 0.05, 0.15]),
        ),
        ("repair_heavy", AllocationGenotype([0.10, 0.20, 0.55, 0.15])),
        ("neutral", AllocationGenotype::neutral()),
    ];
    let mutation_off_founder_classes = founder_classes
        .iter()
        .map(|(founder_class, genotype)| {
            let parent = real_fission_parent(*genotype, &params);
            let (daughter_a, daughter_b, event) =
                try_local_fission(&parent, &FissionParams::default())
                    .expect("controlled D-096 parent must physically fission");
            assert!(
                event
                    .partition
                    .catalyst_partition
                    .expect("D-096 fission audit")
                    .conserved
            );
            let mut exact_copy = true;
            let mut hash_stable = true;
            let mut candidate_hash = String::new();
            for mut daughter in [daughter_a, daughter_b] {
                let before = daughter.finite_allocation.expect("D-096 daughter state");
                let before_hash = before.genotype.candidate_hash(&params);
                let result = adapter
                    .apply_heredity_and_mutation(
                        &parent,
                        &mut daughter,
                        &mutation_none,
                        &MutationContext {
                            accepted_step: 1,
                            accepted_simulated_time: adapter.accepted_dt(),
                            seed: 7,
                            offspring_index: 0,
                            qualified_physical_copy: true,
                            qualified_copy_ordinal: 0,
                            parent_hereditary_state: adapter.hereditary_state(&parent),
                        },
                    )
                    .expect("mutation_none must accept actual fission daughters");
                let after = daughter.finite_allocation.expect("D-096 daughter state");
                exact_copy &= result.is_none()
                    && after.genotype == before.genotype
                    && after.catalysts == before.catalysts;
                hash_stable &= after.genotype.candidate_hash(&params) == before_hash;
                candidate_hash = before_hash;
            }
            assert!(exact_copy && hash_stable);
            MutationOffFounderEvidence {
                founder_class,
                exact_copy,
                hash_stable,
                candidate_hash,
            }
        })
        .collect::<Vec<_>>();

    let parent = real_fission_parent(AllocationGenotype::neutral(), &params);
    let (daughter_a, daughter_b, fission_event) =
        try_local_fission(&parent, &FissionParams::default()).expect("real D-096 fission");
    let partition_conserved = fission_event
        .partition
        .catalyst_partition
        .expect("D-096 fission audit")
        .conserved;
    assert!(daughter_a.alive && daughter_b.alive && partition_conserved);
    let catalysts_before_mutation = daughter_a
        .finite_allocation
        .expect("D-096 daughter")
        .catalysts;
    let mut mutated_daughter = None;
    let mut mutation_provenance_recorded = false;
    for ordinal in 0..10_000_u64 {
        let mut candidate = daughter_a.clone();
        let metadata = adapter
            .apply_heredity_and_mutation(
                &parent,
                &mut candidate,
                &mutation_protocol,
                &MutationContext {
                    accepted_step: 1,
                    accepted_simulated_time: adapter.accepted_dt(),
                    seed: d096_mutation_stream_seed(17, ordinal),
                    offspring_index: 0,
                    qualified_physical_copy: true,
                    qualified_copy_ordinal: ordinal,
                    parent_hereditary_state: adapter.hereditary_state(&parent),
                },
            )
            .expect("mutation-on must accept actual fission daughter")
            .expect("mutation-on provenance must be recorded");
        mutation_provenance_recorded |= metadata.contains_key("qualified_copy_ordinal");
        if metadata.get("mutation_occurred").map(String::as_str) == Some("true") {
            assert_ne!(
                candidate
                    .finite_allocation
                    .expect("D-096 daughter")
                    .genotype,
                parent.finite_allocation.expect("D-096 parent").genotype
            );
            assert_eq!(
                candidate
                    .finite_allocation
                    .expect("D-096 daughter")
                    .catalysts,
                catalysts_before_mutation
            );
            mutated_daughter = Some(candidate);
            break;
        }
    }
    let mut mutated_daughter = mutated_daughter.expect("frozen p=0.01 mutation observed");
    let post_genotype = mutated_daughter
        .finite_allocation
        .expect("D-096 daughter")
        .genotype;
    let pre_genotype = parent.finite_allocation.expect("D-096 parent").genotype;
    let mutation_created_no_catalyst = mutated_daughter
        .finite_allocation
        .expect("D-096 daughter")
        .catalysts
        .iter()
        .zip(catalysts_before_mutation.iter())
        .all(|(after, before)| (after - before).abs() <= 1e-12);
    assert!(mutation_created_no_catalyst);
    let source = (0..4)
        .find(|&index| post_genotype.0[index] < pre_genotype.0[index])
        .expect("mutation source");
    let target = (0..4)
        .find(|&index| post_genotype.0[index] > pre_genotype.0[index])
        .expect("mutation target");
    let mut inherited_control = mutated_daughter.clone();
    inherited_control
        .finite_allocation
        .as_mut()
        .unwrap()
        .genotype = pre_genotype;
    let mut mutated_synthesis = [0.0; 4];
    let mut inherited_synthesis = [0.0; 4];
    for _ in 0..3 {
        let mutated_ledger = expression_step(&mut mutated_daughter, &params, 0.1)
            .expect("mutated daughter expression");
        let inherited_ledger = expression_step(&mut inherited_control, &params, 0.1)
            .expect("inherited control expression");
        for index in 0..4 {
            mutated_synthesis[index] += mutated_ledger.synthesis[index];
            inherited_synthesis[index] += inherited_ledger.synthesis[index];
        }
    }
    let follows_mutated_genotype = mutated_synthesis[target] > inherited_synthesis[target]
        && mutated_synthesis[source] < inherited_synthesis[source];
    assert!(follows_mutated_genotype);

    let evidence = D096Gate6EvidenceV1 {
        schema: "D096Gate6EvidenceV1",
        equation: "autopoietic_material_mesh_finite_catalytic_allocation_v1",
        mutation_probability: params.mutation_probability,
        mutation_sigma: params.mutation_sigma,
        mutation_samples: 10_000,
        observed_mutations,
        observed_mutation_frequency: observed_mutations as f64 / 10_000.0,
        exact_copy_observed,
        lawful_transfer_observed,
        zero_capped_transfer_observed,
        physical_partition_conserved: audit.conserved,
        physical_partition_max_residual: audit.max_residual,
        stream_key_matrix_campaign_seeds: 64,
        stream_key_matrix_copy_ordinals: 256,
        stream_key_matrix_unique: keys.len() as u64,
        stream_pair_replay_equal: d096_mutation_stream_seed(17, 1)
            == d096_mutation_stream_seed(17, 1),
        stream_collision_pair_distinct: d096_mutation_stream_seed(17, 1)
            != d096_mutation_stream_seed(18, 0),
        frozen_candidate_hash: pre_genotype.candidate_hash(&params),
        real_fission_daughter_count: 2,
        real_fission_partition_conserved: partition_conserved,
        real_daughter_mutation_on_observed: mutated_daughter
            .finite_allocation
            .expect("D-096 daughter")
            .genotype
            != pre_genotype,
        real_daughter_mutation_provenance_recorded: mutation_provenance_recorded,
        real_daughter_mutation_created_no_catalyst: mutation_created_no_catalyst,
        post_birth_expression_steps: 3,
        post_birth_expression_follows_mutated_genotype: follows_mutated_genotype,
        mutation_off_founder_classes,
    };
    println!("{}", serde_json::to_string_pretty(&evidence).unwrap());
}
