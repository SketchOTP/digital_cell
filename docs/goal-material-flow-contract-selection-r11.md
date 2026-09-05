# Goal-mode material-flow contract selection R11

Status: `GOAL_AGENT_PROVISIONAL_ARCHITECTURE_SELECTED`

This is an architecture-selection milestone after the R10 unified flux ledger.
It does not implement or execute a new organism/world mechanism.

## Largest unresolved end-goal gap

```text
finite spatial environmental material
→ adequate local transfer
→ existing internal N/F processing
→ structural growth
→ resource-causal physical fission
```

R10 localized the first measured loss to environmental N/F transfer. The
current spatial-field plus assimilation path has nonzero downstream processing
and growth, but remains below the unchanged reproduction gate. Assimilation
preservation passed; assimilation itself remains `INVESTIGATE_NOT_ACCEPTED`.

## Superseded alternatives

The following are closed or rejected for the next runtime cycle:

- CLOSURE-006 through CLOSURE-014 local active-work/contact/material formulas:
  exhausted.
- More gains, thresholds, complements, ratios, attenuation, or local active
  work: not authorized.
- Another intracellular pool or assimilation-buffer variant: not authorized.
- Another field-placement or per-cell request-allocation variant: not
  authorized.
- Whole-membrane fixed-boundary reproduction reference: useful calibration,
  but not a spatial shared ecology contract.

## Selected contract

`SHARED_FINITE_EXTRACELLULAR_MEDIUM_LOCAL_MEMBRANE_EXCHANGE_V1`

The next implementation, if this contract survives independent review, must
use:

```text
finite world-owned extracellular N/F medium
→ local membrane-segment flux
→ existing organism interior N/F
→ existing frozen metabolism
→ existing A/W and growth
→ existing fission law
```

The contract deliberately removes the unaccepted `assimilation_n` /
`assimilation_f` intermediate from the causal path. Environmental material is
credited directly to the existing internal N/F state through the existing
transport boundary. This is materially different from the current Route-B/R4
composition, not another representation of the same assimilation buffer.

## Invariants

1. The world owns every positive environmental N/F unit.
2. Every delivered unit is debited once from the shared medium.
3. All organisms submit local membrane-segment requests from the same
   pre-step medium and mesh state.
4. Requests are applied simultaneously and order-independently.
5. There is no per-organism copied resource, hidden bath, request-weight gain,
   or observer allocation.
6. Existing permeability, `k_flux`, membrane edge length, `dt`, and
   boundary-minus-interior transport semantics remain the only membrane
   transfer terms.
7. Existing reaction, growth, topology, fission, damage, and death laws remain
   unchanged.
8. The causal order is medium state → local transfer → existing metabolism →
   existing growth/topology/fission.
9. Transfer-disabled replay must be identical except for the absence of
   environmental N/F delivery.
10. World loss must equal organism delivery for N and F at every accepted
    step and across fission/remesh/checkpoint boundaries.

## Why this is the highest-value next architecture

It addresses the first measured divergence directly at the organism/world
boundary, preserves the already-qualified downstream biology, and tests the
missing completion chain without adding a new intracellular controller or
buffer. It also gives a clean comparison against the whole-membrane reference:
same existing membrane law, finite shared ownership, explicit local exposure,
and no per-cell proportional allocator.

It is more valuable than further assimilation work because R10 already showed
that processing is downstream of the first loss. It is more valuable than
reopening motility because resource-causal fission is blocked before a new
local active-work expression could be causally relevant.

## Required next execution boundary

The next runtime increment must be one end-to-end contract test with:

- spatially separated finite medium and organisms;
- active and transfer-disabled controls;
- simultaneous shared-medium allocation;
- direct existing N/F transport;
- unchanged metabolism, growth, fission, death, and checkpoint behavior;
- environmental transfer before the first reproductive divergence;
- no new assimilation state in the candidate causal path.

It must report the same R10 ledger stages and preserve the unchanged `1.35 ×
birth_mass` gate. It must not change resource inventory merely to obtain
fission.

## Stop conditions

Stop and replan if any of the following occurs:

- a new biological rate, gain, threshold, timer, or allocator is required;
- direct existing N/F transfer still diverges at the same boundary after the
  medium contract is normalized and conservation is proven;
- spatial material must be copied, globally broadcast, or observer allocated;
- existing metabolism, growth, fission, or death laws must change;
- preservation or restart continuity fails;
- success requires a resource coordinate, target, gradient, or behavior input.

If the contract fails at its first transfer boundary, classify the current
organism/world material-flow architecture as insufficient and stop local
material-flow implementation. Do not create another pool or field variant.

## External prior-art disposition

- Extracellular compartment plus local cross-membrane transport: `DIRECTLY_ADAPTABLE`.
- Uptake distinct from intracellular assimilation: `ADAPTABLE`, with no
  biological values imported.
- Diffusion-limited versus reaction-limited growth: `REFERENCE_ONLY` for
  interpretation, not parameter import.
- External transport kinetics or quota thresholds: `INCOMPATIBLE` unless
  already present in Digital Cell.

References:

- [Discretised flux-balance reaction–diffusion model](https://pmc.ncbi.nlm.nih.gov/articles/PMC11390822/)
- [Diffusion-limited growth of microbial colonies](https://pmc.ncbi.nlm.nih.gov/articles/PMC5902472/)
- [Quantitative modelling of nutrient-limited growth](https://pmc.ncbi.nlm.nih.gov/articles/PMC5832723/)

No external numerical value or biological controller is imported.

## Authority boundary

- Selection is `GOAL_AGENT_PROVISIONAL_ARCHITECTURE_SELECTED` only.
- Independent Architect acceptance is not claimed.
- No runtime implementation or successor execution has started.
- Autonomous resource acquisition and resource-causal reproduction remain
  `NOT_ESTABLISHED`.
