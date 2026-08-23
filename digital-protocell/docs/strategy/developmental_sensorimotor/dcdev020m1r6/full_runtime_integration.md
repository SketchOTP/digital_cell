# DC-DEV-020-M1-R6 — full-runtime integration certification

This bounded assay tests the simplest R5-qualified candidate through the
packaged Phase 1 runtime ordering:

```text
finite V1 transport -> ConservativeV3 reactions -> mechanics -> remesh -> local rebond
```

The candidate is `ConservativeV3`, reserve OFF, uncoupled V1 finite-resource
transport, `FINITE_SPATIAL_BACKING_RESERVOIR_V1`, and coupled-source OFF. The
production default remains `ConservativeV2`; this branch does not select V3.

The assay starts from the exact R5 entry state and uses `dt=0.02`, 8,000-step
fed/no-resource arms, the R4 480-step feed-then-remove control, and a bounded
150,000-step no-resource death continuation followed by a 5,000-step fresh
finite-resource restoration challenge if rupture occurs.

The dense per-step ledgers are written directly to the governed shared-drive
root:

```text
\\RPI5\RPI5SharedDrive\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6\dense
```

R6 is integration evidence only. It does not modify chemistry equations,
mechanics, remeshing, rebonding, D-091, D-087, production selection, or any
downstream M2 behavior.

The result is accepted only after exact-head remote CI and Architect review.
