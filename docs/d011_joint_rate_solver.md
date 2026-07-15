# D-011 Joint Rate Solver

## Trigger

Runs when failed Stage E rate replay shows no joint overlap under constrained-radius dynamics.

## Sensitivity

- `S_ij = ∂g_i / ∂ln(k_j)` via ±5% central differences on seven rates
- Report rank, condition number, singular values
- Flag `rank_deficient` when rank < 4

## Bounded correction

Solve `S Δp ≈ −g` in log-rate space:

| Constraint | Value |
| --- | --- |
| Global rate bounds | 0.5×–2.0× Stage E reference |
| Per-round bounds | 0.75×–1.33× multiplicative |
| Max rounds | 4 |
| Max candidates | 5 (including original) |
| Selection | Prefer smallest log-rate change norm |

## Validation

Each candidate re-run on full replay radius grid {14, 18, 22, 26, 30, 34}.

## Artifacts

`bounded_joint_solver/radius_22.json`, `radius_26.json`, plus `validation_results` in main result.
