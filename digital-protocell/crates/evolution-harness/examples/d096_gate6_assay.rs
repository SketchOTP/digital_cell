use chemistry_core::d096_allocation::{
    mutate_allocation_genotype, partition_catalysts, AllocationGenotype, AllocationParams,
    AllocationState,
};
use evolution_harness::d096_mutation_stream_seed;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Serialize)]
struct Gate6Evidence {
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

    let evidence = Gate6Evidence {
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
        frozen_candidate_hash: parent.candidate_hash(&params),
    };
    println!("{}", serde_json::to_string_pretty(&evidence).unwrap());
}
