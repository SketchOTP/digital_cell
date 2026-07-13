# D-005 Coarse Basin Map

**Directive:** D-005 §10  
**Grid:** R₀ ∈ {16,20,24,28,32} × C₀ ∈ {0.20,0.275,0.35,0.425,0.50}, seed=1, 20,000 substeps

## Status

Coarse basin mapping runs via `experiment-runner d005 coarse-basin` and `d005 pipeline`. Artifacts under `experiments/generated/d005/coarse_basin/`.

## Results (k_phi=1.0, complete)

**25/25 grid points** classified **`slow_decline`**. **0** `near_balance` points.

Every tested (R₀, C₀) combination at seed=1 shows continued structural decline over 20k substeps. No contiguous viable patch (§15).

## Macrostate flow

See `experiments/generated/d005/macrostate_flow/flow.json` after pipeline completion.

## Basin acceptance (§15)

**Not met** — zero `near_balance` points across the full 5×5 coarse grid for k_phi=1.0.
