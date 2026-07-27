# D-095 partition reconstruction and pre-fission causal replay

## Scope

This analysis uses the sealed D-094R2 Gate 6 checkpoints. It does not alter
organism biology, launch an evolutionary campaign, score D-096 candidates, or
authorize Phase 3.

## Partition reconstruction

Sixteen actual phenotype-bearing checkpoint organisms reached a reconstructable
fission. Parent network traits included both `+1` (H) and `-1` (B). Mutation and
edge loss were disabled while advancing to fission.

| Measure | Result |
|---|---:|
| Parent-to-daughter network displacement | 0 |
| Weighted phenotype displacement | 0 |
| Mutation variance | 0 |
| Partition variance | 0 |
| Pre-partition phenotype covariance | 1 |
| Post-partition phenotype covariance | 1 |
| High-parent phenotype loss rate | 0/16 |

`PARTITION_NOISE_ERASES_SELECTION` is eliminated for the observed H/B networks.
The deterministic spatial partition preserves edge identity and phenotype in
these events.

## Matched pre-fission replay

Sixteen actual H/B pairs were selected by measured network phenotype, exact
generation, treatment history, and replicate identity, then nearest body
material and reserve. Each pair was replayed under H, B, and neutral conditions
with mutation and edge loss disabled. Pair members received identical exposure
and both stopped when the first member reached mass-and-geometry-valid fission
readiness, or at 1,000 steps. Fission was never applied.

Individual age is not serialized by D-094. Generation and atomic checkpoint time
are exact proxies. H and B phenotypes originate from different founder networks,
so “founder background” is paired by replicate/seed identity rather than an
isogenic background; this limitation is explicit and prevents interpreting the
replay as an isogenic intervention.

Across 16 comparisons per replay environment:

| Environment | Mean maximum physiology difference | Mean structural-growth difference | Survival differences | Readiness differences |
|---|---:|---:|---:|---:|
| H | 0.0967 | 0.1417 | 0 | 8 |
| B | 0.0820 | 0.4668 | 0 | 8 |
| neutral | 0.0823 | 0.1998 | 0 | 8 |

B-bearing organisms had greater structural growth in 14/16 pairs in every
environment. H-bearing organisms showed greater activated-resource production
in 8/16 pairs in every environment. Thus the inherited network affects
conserved physiology and pre-fission fitness, but the H response does not become
an H-specific growth advantage. The B growth advantage is largely
environment-independent.

## Classification

```text
PRIMARY:
ENVIRONMENT_PHENOTYPE_INTERACTION_ABSENT

SECONDARY:
DEMOGRAPHIC_NOISE_DOMINATES_WEAK_DESCENDANT_DIFFERENCES
```

The causal chain first breaks at environment specificity: the physiological and
growth directions remain essentially unchanged across H, B, and neutral.
Measurable pre-fission differences are then diluted in population descendant
contribution: D-094 opportunity for selection was only `0.000933` in H and
`0.004003` in B, while trait-descendant covariance was `-0.0000947` and
`0.0002457`. Partition fidelity remains complete in this reconstruction.
