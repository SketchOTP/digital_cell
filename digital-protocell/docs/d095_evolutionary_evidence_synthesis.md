# D-095 evolutionary evidence synthesis

Status: observational decomposition complete; matched replay not started.

The authoritative D-094 analytical table contains 24 terminal rows (8 H, 8 B,
8 neutral), with no exclusions. Every row is paired by replicate and verified
against its generation-8 atomic checkpoint, source commit, binary hash, and
configuration hash.

## Observational decomposition

| treatment | phenotype variance | descendant variance | opportunity `I` | `Cov(z,w)` | gradient |
| --- | ---: | ---: | ---: | ---: | ---: |
| H | 0.000771605 | 0.000235160 | 0.000932727 | -0.000094670 | -0.122692 |
| B | 0.000208333 | 0.001035243 | 0.004003185 | 0.000245664 | 1.179187 |
| neutral | 0 | 0.000041521 | 0.000166220 | 0 | 0 |

The sealed D-094 Gate 4 evidence reports parent-offspring edge-frequency
correlation `0.7261` and network-response correlation `0.8717`. Hereditary
transmission is therefore measurable. Mutation-generated variance is exactly
zero because mutation was disabled for Gate 6.

Leave-one-replicate-out effects remain nonqualifying in every omission:
H ranges from `-0.03175` to `-0.02381`; B ranges from `0.00476` to `0.00952`.
The likely first broken link is therefore
`PHENOTYPE_TO_DESCENDANT_COVARIANCE_ABSENT_OR_WEAK`, not provenance,
generation throughput, or loss of hereditary identity.

This remains provisional until phenotype-specific partition variance is
reconstructed and matched pre-fission replay tests the physiological link.
No causal classification, candidate selection, D-096 contract, or Phase 3
authorization is asserted here.

Artifacts:

- `experiments/generated/d095/normalized_evidence/d094_terminal_rows.json`
- `experiments/generated/d095/selection_opportunity/observational_decomposition.json`
- `experiments/generated/d095/manifest.json`
