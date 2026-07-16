# D-017 reaction-resolved W source decomposition

## Evidence

- D-016 frozen total source: **41.63147** mass/time (`experiments/generated/d016/manifest.json`)
- Extents: D-015 fresh R22 `checkpoint_150000` (η_c=η_φ=η_m=**1.0**)
- Channels scaled so totals match the frozen D-016 source

## Scaled channel rates (mass/time)

| Channel | Rate | Fraction |
| --- | ---: | ---: |
| direct activation W | 2.078 | 0.0499 |
| productive-yield W | 0.000 | 0.000 |
| structure turnover W | 36.890 | 0.8861 |
| catalyst turnover W | 0.999 | 0.0240 |
| membrane turnover W | 0.307 | 0.0074 |
| A turnover W | 0.565 | 0.0136 |
| membrane-detachment W | 0.793 | 0.0190 |
| **total** | **41.631** | **1.000** |

## Maximum activation W reduction

`maximum_immediate_reduction = direct_activation_W = 2.078` mass/time (~5%).

Structure-turnover dominates under constrained-radius Stage E (virtual decay to hold R).

## Spatial proxies

Per-channel interior/interface fractions use the frozen total-source mix
(interior≈0.806, interface≈0.169) — per-channel fields were not stored in D-016.

Artifact: `digital-protocell/experiments/generated/d017/source_decomposition/reaction_resolved.json`
