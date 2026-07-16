# D-016 candidate report

## Primary conclusion

```text
D016_PASSIVE_WASTE_TRANSPORT_INSUFFICIENT
```

## Subsidiary conclusion

```text
D016_INTERNAL_DIFFUSION_LIMIT_CONFIRMED
```

## Frozen chemistry (unchanged)

- equation: `membrane_metabolism_v2_conservative`
- stoichiometric schema: 2
- rates / yields / initial conditions / ceiling: frozen
- organism hash: `9a452d…7728d626` (unchanged; no accepted transport repair)

## Frozen environment (D-015)

- `waste_sink_inner_radius = 30`
- environment hash: `ef1834ed…b25d39`

## Why passive repair fails inside D-016

1. Canonical interior source ≈ 33.6 mass/time with `q_area ≈ 0.026`.
2. Analytical `ΔW_center ≈ 12.7` already exceeds the safety headroom.
3. `D_W_required(50%) ≈ 1.06` and `D_W_required(90%) ≈ 0.45`.
4. Authorized bound `D_W ≤ max(D_N,D_F) = 0.18`.
5. Baseline `D_W = 0.25` is already **faster** than N/F and still fails biologically.
6. Resistance decomposition after sink repair is **internal-dominated** (~87%).

Therefore no authorized `(D_W, β_W)` pair can close the waste transport timescale
without exceeding the existing small-solute diffusivity scale.

## D-012 / D-008 status

- D-012 solver entry: **CLOSED**
- D-008 Stages 0–D: PASS; Stage E: BLOCKED; F–G: BLOCKED
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production verdict: `REQUIRES_REMEDIATION`

## Next directive (required alternatives)

Compare exactly:

- A. conservative activation-yield repair (reduce W generation / increase usable A)
- B. energy-coupled active waste export

Do not implement either inside D-016. Do not add an eighth field.
