# D-016 transport calibration

## Diffusivity candidates

Derived from analytical `D_W_required` at the 50% ceiling target, then capped:

```text
candidates ⊆ { baseline D_W, 0.75×, 1.00×, 1.25× D_W_required }
after bound: D_W ≤ max(D_N, D_F) = 0.18
```

Because `D_W_required ≈ 1.06 > 0.18`, every authorized candidate collapses to
the bound (and baseline 0.25 is already above the authorized repair ceiling
yet still fails biologically).

## Membrane branch

Entered only when membrane resistance is material. With dominant internal
resistance and `D_W_required` far above the bound, the insufficiency gate is
the single point:

```text
D_W = max(D_N, D_F) = 0.18
β_W = 0
```

## Feasibility gate

Passive transport is feasible only if a candidate within bounds reaches a
finite fixed-source steady state with center W < 50% of the safety ceiling and
sink removal matching source within 2%.

When the bound-and-β_W=0 assay still cannot, record:

```text
D016_PASSIVE_WASTE_TRANSPORT_INSUFFICIENT
```

Do not raise the diffusivity bound inside D-016.
