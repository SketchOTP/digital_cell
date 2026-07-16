# D-014 Timestep Floor Postmortem

## Verdict

```text
FIELD_BOUND_STIFFNESS
```

Primary mechanism: `waste` cell 18335 reached `CONC_SAFETY_LIMIT` (10.0). The next-state value
`10.000000000099783` failed `validate_soluble_field`, the adaptive controller halved `dt` until
the governed floor (~1e-8), and the run terminated as `TIMESTEP_FLOOR_FAILURE` /
`NUMERICAL_FAILURE`.

## Not the primary cause

| Hypothesis | Evidence |
| --- | --- |
| Adaptive ratcheting alone | At 150k, `dt=0.0025` healthy; failure is a bound reject, not stale min latch |
| Reaction stiffness | Terminal limiter is `FIELD_BOUND_VALIDATION`, not a reaction label |
| Transport / membrane diffusion | No transport limiter transition at failure |
| Reservoir stiffness | Not reported |
| Non-finite values | Finite concentrations |

Controller recovery (1.25× after accept) was still active during reproduction; failure still
reproduced → recovery alone does not fix this floor event.

## Repair branch

**Branch E** — machine-scale ceiling projection (`≤ 1e-9` overshoot) plus hard concentration
rejects abort without floor cascade and map to `UNBOUNDED_ACCUMULATION`.

Subsidiary: bounded `dt` recovery (Branch A hygiene) and activation-potential residual identity fix.
