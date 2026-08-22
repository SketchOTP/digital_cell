# DC-DEV-020-M1-R1-R1 decay-confound isolation

## Authority and scope

This observer-only audit starts at `9db2c7d08495f8e935a59385bf51927bcd951a7b`,
reuses the accepted M1-R1 entry state at
`3cab12551072ad1eafaece72615f448d8efb9bea`, and executes the fixed 480-step
deprivation horizon. It does not change chemistry-core production source,
ConservativeV2, D-091, uptake, transport, resource inventory, structural or
catalyst kinetics, reserve behavior, or death semantics.

## Causal isolation

The raw source-capacity arm converts all immediately paired internal N/F to A
using the existing `N + F -> A + W` stoichiometry. Per-step provenance shows
that the post-source-UB N×F starvation predicate is true for `480/480` steps,
so the existing production multiplier is `4×` on every relevant step.

The two neutralized shadows set only the diagnostic `ReactionParams.k_a_decay`
to `K/4`, where production `K = 0.008`. The frozen production multiplier then
gives an effective coefficient of `0.008`, matching ordinary production A
decay. This is a causal shadow, not a production parameter proposal.

## Result

The raw source-capacity arm reproduces M1-R1 with organized-material change
`-3.09944444397982` and A decay `6.064743776472449`. The decay-neutral source
shadow reaches organized-material change `+1.25718049040759`, while the
decay-neutral combined shadow reaches `+1.2755639121915`. The source shadow
therefore crosses the acute 480-step nonnegative threshold after removing the
secondary starvation-decay penalty.

Classification:

```text
M1_SOURCE_CAPACITY_SUFFICIENT_AFTER_DECAY_NEUTRALIZATION
```

This is an acute capacity result, not sustained M1 homeostasis. The existing
starvation-decay coupling is contributory, and source capacity is sufficient
under this bounded neutralization, but no production repair or homeostatic
claim is authorized by this audit.

## Accounting and preservation

World↔organism and internal material closure pass within `1e-8`; A decay still
enters W, no W→A transfer or recycling is present, and reserve remains off.
The compact artifacts are generated under
`experiments/generated/dcdev020m1r1r1/`. Exact-head CI additionally runs the
Phase-1 metrics, D-091, D-088, and evolution-harness preservation regressions.

M1 production change, M2, recycling/salvage, and DC-DEV-021 remain
unauthorized pending independent architect review.
