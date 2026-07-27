# D-094R2 selection analysis

## Result

The valid frozen Gate 6 campaign did not establish environment-dependent
selection. H and B each completed 8/8 paired replicates through generation 8,
but neither arm produced a qualifying phenotype-frequency or
descendant-contribution response. All mutations were disabled (`mutation supply
= 0`) by the frozen contract; all populations remained viable and no replicate
went extinct.

Founder H and B frequency was 0.5 in the treatment arms. Neutral has no H/B
phenotype frequency and is therefore a zero-change control for its matching
phenotype. The continuous effects below use the directive's definition:
`Effect_H = (final H - 0.5) - 0` and `Effect_B = (final B - 0.5) - 0`.

| arm | replicate delta values | mean | median | empirical 95% range | positive signs |
| --- | --- | ---: | ---: | --- | ---: |
| H frequency | -0.0556, 0, 0, -0.0556, 0, -0.0556, 0, -0.0556 | -0.0278 | -0.0278 | [-0.0556, 0] | 0/8 |
| B frequency | 0, 0, 0, 0.0333, 0, 0, 0, 0.0333 | 0.0083 | 0 | [0, 0.0333] | 2/8 |
| H descendant contribution | -0.0304, -0.0258, -0.0052, 0.0170, 0.0195, 0.0140, 0.0080, 0.0214 | 0.0023 | 0.0140 | [-0.0304, 0.0214] | 5/8 |
| B descendant contribution | -0.0300, 0.0058, 0.0292, 0.0350, -0.0463, 0, 0.0424, 0.0305 | 0.0083 | 0.0292 | [-0.0463, 0.0424] | 5/8 |

The emitted manifest's `treatment_effects.frequency` subtracts neutral's raw
zero label frequency rather than the treatment founder baseline. It is retained
as a machine artifact but is not used for this conclusion; the table above
applies the preregistered delta-from-founder definition directly to immutable
rows. This cannot convert the result to a pass: H is negative, B is 0.0083
against the frozen 0.15 requirement, and both descendant effects span zero.

Leave-one-replicate-out recalculation leaves the conclusion unchanged: H mean
remains between -0.0317 and -0.0238, B mean between 0.0048 and 0.0095, and no
omission reaches the 0.15 threshold or creates robust descendant evidence.

## Other required observations

- Completed generations: 8 in every replicate; no partial generation counted.
- Survival/viability: all 24 rows `extinct=false`, with 179–342 survivors.
- Reproduction: 171–334 fissions per treatment row; all lineage records closed.
- Parent-offspring heredity: phenotype labels and lineage links persisted in ledgers.
- Effective population size, generation duration, and phenotype-specific survival/reproduction are not separately emitted by the frozen runner; they are not inferred here.
- Neutral label drift was zero (`neut_shift=0`); no numerical or checkpoint failure occurred.

This is valid negative selection evidence, not an untestable run, numerical
failure, or implementation-defect verdict.
