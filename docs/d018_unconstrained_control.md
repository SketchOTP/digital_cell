# D-018 Unconstrained Control

Frozen candidate, analytic R22 IC, `enforce_structure_constraint=false`.

| Metric | Value |
| --- | --- |
| Classification | STRUCTURE_COLLAPSE_LIMITS_W_SOURCE |
| Termination | STRUCTURE_LOSS_50PCT |
| Accepted substeps | 15001 |
| Structure fraction remaining | 0.4888 |
| Structure decay extent | 1072.0487 |
| Structure production extent | 289.4928 |
| Constraint flux cumulative | 0.0 |

## Constraint-artifact evidence

Constrained history: W climbs toward ceiling while φ is held fixed.
Unconstrained: φ declines substantially (≥50% mass loss) and the artificial rebuild loop is absent (`constraint_flux=0`).

This supports moving the upstream failure from waste export to **structural maintenance / scaling**, without authorizing a new waste pathway.
