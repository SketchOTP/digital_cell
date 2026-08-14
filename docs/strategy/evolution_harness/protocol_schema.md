# Versioned protocol schema

The crate defines `ExperimentProtocolV1`, `EnvironmentProtocolV1`, `MutationProtocolV1`, `FounderIdentityV1`, `ReplicateResultV1`, and `SelectionAnalysisV1`. Each serializable type carries a schema name and stable FNV-1a hash over deterministic JSON serialization.

`ExperimentProtocolV1` declares organism and heredity schemas, mutation/environment/placement protocols, an optional `SelectivePressureContractV1`, replicate seeds, accepted horizon, generation requirements, termination rules, and endpoints. Protocol validation rejects empty identities, zero replicates, seed-count mismatch, invalid mutation rates, impossible generation bounds, and malformed pressure contracts.

The schema is intentionally generic: prior D-090–D-093 designs are represented as evidence-grounded, non-executable fixtures returned by `historical_protocols()`, without adding runner-specific execution branches. `ProtocolProvenanceV1` records source artifacts, derived values, unresolved values, and execution authorization. `validate_for_execution` rejects unresolved or unauthorized translations.
