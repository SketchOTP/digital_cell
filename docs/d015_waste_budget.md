# D-015 waste budget

## Identity (accepted steps only)

```text
ΔW_observed
  = activation
  + catalyst_turnover
  + structure_turnover
  + membrane_turnover
  + activated_resource_turnover
  + membrane_detachment
  + productive_yield_waste
  + external_reservoir_waste_input
  − waste_clearance
  + numerical_correction
```

Internal conservative transport cancels globally and does not appear as a net source.

## Implementation

- Module: `digital-protocell/crates/chemistry-core/src/d015_waste.rs`
- Hook: `Simulation::record_waste_budget` on v2 constrained-radius accepts
- Relative residual tolerance: `WASTE_BUDGET_REL_TOL = 1e-8`

## Preflight evidence (repaired env, 25k)

| Metric | Value |
| --- | --- |
| `waste_budget_ok` | true |
| `waste_budget_max_relative_residual` | ≈ 2.6e-14 |
| Material relative residual | ≈ 1.8e-6 |
| Activation relative residual | ≈ 4.0e-14 |

## Tests

See `crates/chemistry-core/tests/d015_tests.rs` (budget closure, transport cancel, clearance once, rejection exclusion).
