# DC-DEV-020-R8 product-feedback attractor audit

## Scope

R8 is an observer-only feasibility audit rooted at accepted R7 head
`7d5f772f0db67b8d754d27c1182c933533f750fd` and clean scientific base
`1e242f28152797b512e25cd56c7b718e45d6ca97`. It tests one target-free,
memoryless product-feedback topology using only existing `N`, `F`, and `A`.
No chemistry, production behavior, parameter choice, observer feedback, or
DC-DEV-021 work is included.

The normalized maintenance surface is:

```text
Y_zero = S_zero / (q_c * g_h * area * dt * G_NF(N,F))
G_NF(N,F) = N^p_NF * F^p_NF
p_NF = 0.0003277429681759396
g_h = 1.0 (explicit neutral phenotype factor)
```

The candidate topology is evaluated only in reciprocal coordinates:

```text
1 / Y_FB = u + v*A
u = 1 / V_FB > 0
v = 1 / (V_FB*K_A) > 0
```

No grid search, random optimization, midpoint, or final `(V_FB, K_A)` is
selected.

## Gate result

The R5 training set contains 2,880 usable P0-P2 root states. The frozen
training min-max N/F scaling and distance limit `0.0024847602445668224`
produce 2,425 deterministic deduplicated pairs. Only 310 pairs have the
restorative sign, with maintenance demand falling as `A` rises; 2,115 pairs
have the opposite sign. The intersection of the required lower-A source-above-
maintenance and higher-A source-below-maintenance half-spaces is empty.

R8 therefore stops at Gate 3 with:

```text
DCDEV020R8_PRODUCT_FEEDBACK_TOPOLOGY_INCOMPATIBLE
```

P3/P4 portability, R7 on-policy portability, and capacity-region analysis are
not executed after this decisive failure. The compact artifacts record those
gates as not reached. The zero-substrate control remains explicit: source is
exactly zero when either substrate is absent.

This closes the tested single reciprocal product-feedback topology on the
frozen diagnostic surface. It does not close the wider NFA route, qualify a
production law, or authorize DC-DEV-021.

## Prior art

Goyal et al. 2010 supports product-feedback inhibition as an architectural
homeostatic motif, including multi-input metabolic modules. Disposition:
`ADAPTABLE_ARCHITECTURE_ONLY`; no constants or molecular identities were
imported.

Bi et al. 2023 supports a stability warning because metabolite-level feedback
networks can oscillate. Disposition: `REFERENCE_STABILITY_WARNING`; no values
were imported.

## Evidence

Compact evidence is in `digital-protocell/experiments/generated/dcdev020r8/`.
The dense pair/constraint ledger is externalized at:

```text
/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r8/6e2b03a7551409086c1a38d6cf5f62827fb91929/pair_constraint_ledger.json
```

Its SHA-256 is
`12b41f27c928635899a7ea3a8d496cfdd3af7d3fd83aaa93024724663e2df9ff`.
R5 and R7 input seals remain unchanged in `external_evidence_manifest.json`.

## Reproduction

```text
DCDEV020R8_R5_LEDGER=<sealed R5 ledger>
DCDEV020R8_R7_LEDGER=<sealed R7 ledger>
DCDEV020R8_OUTPUT_ROOT=digital-protocell/experiments/generated/dcdev020r8
DCDEV020R8_EXTERNAL_LEDGER=<external dense output>
cargo +1.89.0 run -p regulatory-core --example dcdev020r8_nfa_restorative_attractor --release --quiet
```

Production chemistry and behavior remain unchanged. `DC-DEV-021` remains
unauthorized.
