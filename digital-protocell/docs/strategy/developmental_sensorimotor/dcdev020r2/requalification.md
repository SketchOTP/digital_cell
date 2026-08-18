# DC-DEV-020-R2 observer requalification

## Authority and scope

This bounded observer-only assay starts from clean scientific entry
`1e242f28152797b512e25cd56c7b718e45d6ca97` and the prior DC-DEV-020
provisional head `876012f8888b074285c55167613471a59d4be25d`. It runs on
`strategy/dc-dev-020r2-allosteric-requalification` and writes only to the new
`experiments/generated/dcdev020r2/` evidence namespace. The historical
`experiments/generated/dcdev020/` artifacts are preserved unchanged.

No production chemistry, chemistry parameters, resource law, transport,
reserve law, controller, behavior, exploration, or DC-DEV-021 work is included.
The assay calls the existing reaction and finite-resource APIs from an
observer-local counterfactual wrapper; it does not change their implementation.

## Prior protocol audit

The R1 protocol is classified as
`DCDEV020_PROTOCOL_NONCONFORMANCE_CONFIRMED`. Its fixed-law result was not a
requalification of the broader A-only allosteric architecture because it did
not establish a source-actuation envelope, did not audit the post-activation
A-decay sequence, and replayed the July D-017 comparison rather than the
August DC-DEV-017 observer contract. R2 adds those missing audits and stops
fail-closed when its A-only sufficiency gate fails.

## Frozen protocol

The assay reproduces 5,000 accepted settlement steps, 480 deprivation steps,
and a 480-step finite-feed window with the selected existing finite ecology:

- resource center `[4.8, 0.0]`, radius `1.5`;
- N and F mass `19.878372106390554` each;
- `dt = 0.02`;
- sustained no-resource reference `E_stored = 77.91027880846893`.

The source-saturated arm is an observer upper bound: each step previews the
ordinary reaction, calculates the existing finite N/F capacity-normalized
gain required to consume the available material, and then replays the
reaction with that gain. It is not a production source law.

## Gate results

Gate 0 passed: the entry, prior head, fixed constants, horizon, and observer
boundary are recorded in `protocol.json`. The prior finite-feed and exact-law
artifacts remain untouched.

Gate 1 passed: the source envelope records ordinary requested/accepted N/F,
source-saturated accepted N/F, required gain, applied gain, A decay, catalyst,
structural, membrane, reserve-transfer, and reserve-loss terms for every feed
step. The ordinary preview is taken from the exact pre-step mesh clone used by
the source-saturated calculation; no preview state is installed in the arm.

Gate 2 passed as a sequencing audit. The ordinary arm had zero accelerated-A
steps and A decay `1.5209688786954632`. The source-saturated arm had 480
accelerated-A steps and A decay `7.3706916369499655`, of which
`7.3706916369499655` was attributed to the accelerated branch. The reported
ratio is accelerated A decay divided by A production; it is not a claim of
particle-level cohort tracing. Reaction order remained unchanged.

Gate 3 passed as the correct August DC-DEV-017 replay. The observer uses the
sealed August demand form with max gain `8.58379474604017` and demand
reference `0.9427183336627594`; it does not import the unrelated July D-017
comparison module.

Gate 4 failed and therefore terminates the requalification:
`DCDEV020_A_ONLY_ALLOSTERIC_COORDINATE_INSUFFICIENT`.

The deprived start was `E_stored = 60.82781514212436`. The measured finite-feed
source-saturated upper bound ended at `61.6843481847883`, but the constant-gain
break-even root was `13.9482421875`. Across the source envelope, the required
gain increased as A decreased (from approximately `594372.493647309` at the
first sampled A to `1482.08151596166` at the last sampled A), which is
opposite the permitted monotone product-inhibition family
`1 + (G_max - 1)/(1 + (A/K_A)^n)`. The bounded A-only fit therefore remained
unidentified and no R2 candidate law was executed.

Because Gate 4 failed, Gates 5 through 8 were not run: no derived law, finite
feed qualification, 8,000-step sustained assay, or three-cycle assay is
claimed. `implementation_authorized` and `next_execution_started` are false.

## Literature classification

The external review is recorded in `literature_review.json`. No constants or
species-specific allosteric identities were imported.

- Goyal et al. 2010 — `ADAPTABLE`: product feedback can be homeostatic but
  simple feedback can create large metabolite pools; ultrasensitivity can
  constrain pools. [Source](https://pmc.ncbi.nlm.nih.gov/articles/PMC2880561/)
- Link, Kochanowski & Sauer 2013 — `ADAPTABLE`: rapid nutrient switches and
  allosteric interactions can control flux on short timescales.
  [Source](https://www.nature.com/articles/nbt.2489)
- Buffing et al. 2018 — `ADAPTABLE`: rapid allosteric flux reversal is
  organism-dependent and may require additional interactions or transcription.
  [Source](https://pmc.ncbi.nlm.nih.gov/articles/PMC6079084/)
- Goyal & Wingreen 2007 — `REFERENCE_ONLY`: coupled product feedback can
  become oscillatory, motivating an explicit stability gate without importing
  model constants. [Source](https://pmc.ncbi.nlm.nih.gov/articles/PMC1995071/)

## Evidence and preservation

The authoritative compact protocol, qualification, results, source envelope,
and literature record are under `experiments/generated/dcdev020r2/`. The old
R1 evidence namespace is not overwritten. The preservation suite must continue
to verify the exact clean entry and the unchanged prior DC-DEV-020 artifact
hashes. This R2 result authorizes neither production integration nor
DC-DEV-021.
