# D-016 resistance decomposition

Serial path resistances estimated from characteristic timescales under the
D-015 repaired sink geometry:

| Segment | Resistance proxy | Fraction |
| --- | --- | --- |
| internal (center→interface) | 484 | ≈ 0.872 |
| membrane crossing | ≈ 4.89 | ≈ 0.009 |
| external (interface→sink) | 64 | ≈ 0.115 |
| sink clearance | 2 | ≈ 0.004 |

Fractions sum to 1.

## Dominant resistance

**internal**

Repair rule: do not change `β_W` when internal diffusion dominates.
Authorized `D_W` calibration is capped at `max(D_N, D_F) = 0.18`, which is
**below** the analytical `D_W_required ≈ 1.06` for a 50% ceiling target.

Artifact: `digital-protocell/experiments/generated/d016/resistance_decomposition/resistance.json`
