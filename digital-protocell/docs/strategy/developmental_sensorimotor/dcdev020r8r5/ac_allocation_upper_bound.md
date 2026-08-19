# DC-DEV-020-R8-R5: A↔C allocation upper-bound audit

## Boundary

R8-R5 is an observer-only capacity test entered from accepted R8-R4 head
`37b47ec89e02418018a138f670e826c6945c8030`. It does not fit or propose a new
catalyst-production law. The sealed R6 source, catalyst turnover, activity
curve, transport, resources, sinks, reserve, A decay, mechanics, and certified
biology remain unchanged.

The counterfactual family repartitions the deprived state's existing
`T_AC = A + C` exactly:

```text
C = C_hold
A = T_AC - C_hold
0 <= C_hold <= T_AC
```

Each sustained arm then replaces exactly the frozen catalyst turnover extent,
`k_c_turn * C * dt`, through the existing 1:1 A→C accounting. If available A
cannot pay that extent, the arm is marked physically infeasible. No excess
catalyst production is permitted.

## External discovery disposition

The cited enzyme-cost method is recorded as `ADAPTABLE_ENZYME_COST_METHOD`,
and the cited flux-sensing study as `REFERENCE_FLUX_SIGNALING_MECHANISM`.
They justify examining allocation cost and throughput together. No biological
constant, target, or external equation was imported.

## Reproduction and envelope

The observer reproduced the accepted R8-R2 acute endpoints and the accepted
R8-R4 finite and sustained shared-affinity endpoints before running the new
capacity analysis. The full physical interval was sampled on a deterministic
65-point mesh. Event brackets were refined deterministically until relative
interval width was at most `1e-6`.

The best constant allocation had final `E_AR` approximately `57.63054549392781`
and did not satisfy the sustained qualification. No constant-C arm passed the
complete original R6 sustained gate. However, every deterministic late-state
sample from both the R8-R3 deferred and R8-R4 shared-affinity trajectories had
at least one conservative partition with nonnegative one-step `ΔE_AR`; the
largest observed local envelope value was approximately `0.00619040719167074`.

Therefore the result is intentionally mixed:

```text
DCDEV020R8R5_CATALYST_ALLOCATION_ENVELOPE_MIXED
```

This does not authorize a target, allocator, catalyst law, production change,
source tuning, sink change, or DC-DEV-021.

## Evidence

Compact evidence is stored under
`experiments/generated/dcdev020r8r5/`. The dense ledger is externalized under
the governed Atlas evidence root and referenced by
`external_evidence_manifest.json`.

## Preservation

Only the observer example, Cargo example registration, evidence, documentation,
governance, and scoped CI are in scope. Certified chemistry-core production
code and production behavior are unchanged.
