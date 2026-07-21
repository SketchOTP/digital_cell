# D-055 — Strict Resource-Gate Replay and Passive-Architecture Review

## Primary conclusion

`D055_PASSIVE_RESOURCE_TRANSPORT_ARCHITECTURE_INSUFFICIENT`

Route: **Route_P**

Next directive: observer-only review of conservative carrier-mediated resource import. No free active pump authorized.

## Phase A — Harness repair

### Exact harness defects

| Path | Defect |
|------|--------|
| `d053.rs::gate5_screen` | `pass = capacity \|\| a_rise \|\| (chi_rise && a_ret≥0.5)` |
| `d053.rs::gate8_fixed` | `short_horizon_relaxed`: χ≥0.20 / A-ret≥0.15 when `h<10000` |

Recorded: `D053_INFORMAL_GATE5_AND_GATE8_PASSES_INVALID`

### Canonical evaluator

`chemistry-core::d053_analysis::{evaluate_gate5, evaluate_gate8}` — single shared contract for tests, runner, replay, and reports. Fixtures A–E pass invariance tests. `short_horizon_relaxed` removed from Gate 8 artifacts (`false`).

### Strict D-053 replay (Gate 3)

- Horizon 10000; 5 authorized candidates; dual-branch (analytic + structure-held)
- All verdicts: `FAIL_RESOURCE_SUFFICIENCY` (χ≪1.05)
- Conclusion: `D055_D053_STRICT_REPLAY_CONFIRMED_NOT_FOUND`
- Retains: `D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND`

### Gate 8 disposition

Biological Gate 8 not run. Max-pair R16/R24/R32 labeled `NONSELECTED_UPPER_BOUND_DIAGNOSTIC`. Informal χ≈0.53/0.38/0.29 fail strict contract → `D053_FIXED_COMPARTMENT_PASS_REVOKED`.

## Phase B — Architecture review

| Gate | Result |
|------|--------|
| Fixed vs dynamic | `NO_FIXED_DYNAMIC_CONTRADICTION` |
| Passive upper bound E | χ≈0.90 < 1 → `PASSIVE_RESOURCE_DELIVERY_HARD_BOUND_FAIL` |
| Environment | `NO_ENVIRONMENTAL_RESCUE` |
| Radius R8–R32 | all χ<1.05 → `NO_VIABLE_RADIUS_IN_TESTED_DOMAIN`; R_critical=null |
| Demand scaling | `DEMAND_DENSITY_STABLE` |
| Stage A band 0.20–0.50 | `EmpiricalCalibration`; Π>0.50 does not rescue → band not unsupported |
| Selectivity frontier | `PASSIVE_SELECTIVITY_THROUGHPUT_INCOMPATIBILITY` |
| Long @10k (Control E) | χ≈0.90 persists as failure; 25k–100k skipped (cap) |

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`
- V14: `V14_SCHEMA3_MIXED_RESOURCE_DELIVERY_EXPERIMENTAL_FAILED` (not selected)

## Deviations

- Phase B diagnostic horizon: `D055_DIAG_HORIZON=2500` (labeled diagnostic)
- Long validation horizons >10000 skipped under `D055_MAX_ACCEPTED=10000`; Control E already fails χ≥1 at 10k

## Tests

`cargo test -p chemistry-core --test d055_tests --test d053_tests --test d054_tests` — 32/32 PASS

## Artifacts

`digital-protocell/experiments/generated/d055/`

## Tag

`D-055-strict-resource-architecture-review`
