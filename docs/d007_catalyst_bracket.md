# D-007 Catalyst Bracket

## Status

**Not run.**

Structural gate returned `D007_NO_STRUCTURAL_NULLCLINE` (§10). Catalyst-rate bracketing is forbidden after that failure.

## Prepared estimate (unused for screening)

From `experiments/generated/d007/diagnosis/catalyst_rate_estimate.json`:

| Field | Value |
| --- | --- |
| median required `k_rep` | ≈ `0.014908` (~1.029× D006) |
| outside 3× bound | no |
| classification | `D007_CATALYST_RATE_WITHIN_BOUNDED_RANGE` |

Even though the estimator is in-bounds, D-007 does not proceed without a structural nullcline.
