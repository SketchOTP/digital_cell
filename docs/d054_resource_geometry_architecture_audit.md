# D-054 — Dynamic Resource Geometry and Passive-Transport Upper-Bound Review

## Primary conclusion

`D054_D053_PROVENANCE_RERUN_DIVERGED`

Route: stop at Gate −1 (no architecture selection among size / environment / band / passive redesign).

## Why D-054 stopped

Gate −1 sealed D-053 source commit `76c0898` and reran Gates 0–8 protocol from that commit.

| Claimed informal result | Sealed rerun result |
|-------------------------|---------------------|
| `D053_NO_HEALTHY_RESOURCE_REPAIRED_ATTRACTOR` (Gate 9 fail after selecting upper bracket) | `D053_BOUNDED_DELIVERY_REPAIR_NOT_FOUND` (Gate 5 fail; no selected pair) |

Upper-bracket metrics match exactly; Gate 5 pass labels do not. That is a material provenance divergence under the directive stop rule.

## D-053 governance

| Item | Value |
|------|-------|
| Source commit | `76c0898e297b0abf04362df3e848e32c9d228b15` |
| Result commit | (see git tag parent) |
| Result tag | `D-053-combined-resource-delivery-fail` |
| V14 record | `V14_SCHEMA3_MIXED_RESOURCE_DELIVERY_EXPERIMENTAL_FAILED` |
| Exhaustion | `BOUNDED_MIXED_DELIVERY_REPAIR_EXHAUSTED` |

## Secondary findings (not promoting a repair route)

1. **Fixed Gate 8 threshold defect (informal artifact):** `short_horizon_relaxed=true`; χ at R16/R24/R32 ≈0.53/0.38/0.29 — all below stated `χ≥1.05`. Likely `D054_D053_FIXED_COMPARTMENT_GATE_DEFECT` after harness repair + rerun.
2. **Informal dynamic Gate 9:** frozen max pair @10k → A≈0.047, χ≈0.47 (not a sealed selection).
3. Gates 1–10 of D-054 (trajectory, checkpoints, passive upper bounds, environment, radius, demand, band, frontier, long validation) were **not executed** because Gate −1 stop fired.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`
- D-054 cannot implement or qualify a repair

## Tests

`cargo test -p chemistry-core --test d054_tests`

## Artifacts

`digital-protocell/experiments/generated/d054/` — `d053_seal/`, `route_decision/`, `manifest.json`

## Next directive

Repair D-053 validation harness:

1. Align Gate 5 admission with the stated sealed contract (no silent χ-rise-only pass).
2. Enforce Gate 8 `χ_N,χ_F≥1.05` (and retention floors) without silent short-horizon weaken, or document an explicit alternate assay with distinct labels.
3. Rerun D-053 from sealed source; then reopen D-054 Gate 0 fixed/dynamic consistency.

`next_execution_started=false`
