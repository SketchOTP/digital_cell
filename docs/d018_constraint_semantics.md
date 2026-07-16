# D-018 Constraint Semantics

## Constrained-radius operation

For each accepted substep under `ConstrainedRadius`:

1. Virtual structure production: `η_φ · k_d008_structure · A · I(φ) · dt`
2. Virtual structure decay: `k_structure_decay · φ · dt`
3. Net virtual flow: `production − decay`
4. Constraint flux: `−(production − decay)` (identity residual ≈ 0)
5. A consumed by production: `r_structure · dt`
6. W produced by decay: `r_structure_decay · dt` (plus yield term if `η_φ < 1`)
7. φ field is **not** updated — the assay holds prescribed structure fixed

## Material-throughput loop

```
φ decay → W production → external constraint restores φ → restored φ decays again
```

This loop does **not** exist in an unconstrained organism. It is an assay artifact whenever decay exceeds endogenous production.

Evidence: `digital-protocell/experiments/generated/d018/constraint_semantics/semantics.json`
