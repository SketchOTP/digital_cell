# DC-DEV-020-R3 two-substrate saturating-activation audit

## Authority and boundary

This observer-only audit starts from accepted DC-DEV-020-R2 head
`e394aa675a4f44d91d1a8729736679fb4b7e7ab8`, while production chemistry remains
rooted in clean scientific base `1e242f28152797b512e25cd56c7b718e45d6ca97`.
R2 evidence is immutable. The assay changes no production chemistry, resource,
transport, sink, reserve, decay, reaction-order, controller, or behavior code.

The frozen source remains `k_act * q_c * g_h * N * F`, with stoichiometry
`N + F -> A + W`. The observer records N, F, N*F, q_c, g_h, ordinary requested
and accepted source, constant-reference accepted source, source-saturated
accepted source, A production, ordinary A decay, and accelerated A decay for
every one of the 480 feed steps.

## Gate result

Gate 2 passed. On the frozen baseline trajectory, log required source gain is
almost perfectly anticorrelated with log N*F (`-0.9999987988614109`), and N*F
explains `0.9999975977242656` of that log variation. The fitted slope is
`-0.4987476149069492`. Bilinear low-substrate suppression is therefore material
in this bounded assay.

Gate 4 failed closed. The permitted symmetric family was

```text
J_sat = q_c * g_h * V_max * N*F /
        (K_S^2 + K_S*N + K_S*F + N*F)
```

With N=F on the observed trajectory, the closed-form linearization has slope
approximately zero (`-6.885614759741503e-17`). The data constrain only
`V_max/K_S^2`. Three deterministic asymptotic witnesses—not a parameter
sweep—show holdout relative error continuing to fall as K_S and V_max grow
together: `0.17001310075841633`, `0.019249003665085914`, and
`0.001950298654552313`. Their maximum source-capacity fractions remain below
`0.0046`. No unique finite V_max and K_S pair is identified.

The required classification is therefore:

```text
DCDEV020R3_SATURATING_KINETICS_NOT_IDENTIFIABLE
```

Later finite-feed qualification, dose robustness, sustained feeding, cycle,
production-integration, and behavior gates were not run. No implementation is
authorized and no next execution was started.

## Literature provenance

The architecture review used primary literature on explicit multi-substrate
rate equations and two-dimensional substrate-rate characterization. Cleland
(1963), Pettersson (1969), and Wang & Mittermaier (2021) support treating both
substrates explicitly and requiring observations across both substrate axes.
Link, Kochanowski & Sauer (2013) is reference-only for rapid nutrient-switch
dynamics. No constants, molecular identities, or species-specific mechanisms
were imported. Exact citations and classifications are recorded in
`literature_review.json`.

## Evidence

Authoritative artifacts are under `experiments/generated/dcdev020r3/`.
`kinetic_diagnosis.json` is the sole dense per-step record; `results.json` is a
compact summary and does not duplicate those ledgers.
