# D-011 Horizon Sensitivity

## Design

Horizon sensitivity tests whether quasi-steady joint-balance signatures depend on run
length or remain `NOT_CONVERGED` under honest short horizons.

## Grid

| Radius | Horizons (steps) |
| --- | --- |
| 18 | 20k, 50k, 100k, 200k (capped by `--max-steps`) |
| 24 | same |
| 30 | same |

## Interpretation

- If overlap emerges only at long horizons → slow relaxation, extend runs.
- If all horizons show large |g| or failed quasi-steady → not a horizon artifact.
- `quick_mode: true` in artifacts when `--quick` was used.

## Output

`attempt_NNN/result.json` → `horizon_sensitivity` map keyed by radius and horizon.
