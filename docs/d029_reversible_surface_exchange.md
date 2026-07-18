# D-029 — Reversible Thermodynamic Bulk–Surface Exchange

## Conclusion

`D029_REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE`

## Gate summary

| Gate | Result |
|------|--------|
| 0 Preservation | PASS (D-021–D-028 tags/commits present; disk ~7.3 GiB free) |
| 1 Schema / conservation / dissipation | PASS (`d029_tests` 11/11) |
| 2 Two-parameter identification | FAIL — β→0 under NNLS |
| 3–15 | Not run (Gate 2 stop) |

## Record

`IRREVERSIBLE_ADSORPTION_LAW_REJECTED` — rejects one-way adsorption only; does not reject P, S=δΓ, Γ-permeability, autonomous surface transport, or biological turnover.

## Fitting matrix (six D-027/D-028 states)

| State | A | B | L |
|-------|---|---|---|
| d024_fixed_interface_r22 | 3.192 | 40.133 | 0.163 |
| d025_dynamic_r22_endpoint | 5.460 | 34.546 | 0.160 |
| d026_stage_e_10000 | 3.688 | 42.531 | 0.158 |
| d026_stage_e_25000 | 4.349 | 41.217 | 0.147 |
| d026_stage_e_100000 | 4.494 | 33.968 | 0.126 |
| d026_stage_e_200000 | 4.447 | 34.089 | 0.127 |

## Fit

- rank: 2
- singular values: ≈ [206.16, 0.150]
- condition number: ≈ 1.38×10³ (< 10⁶)
- unconstrained / projected: α ≈ 0.0329, β → 0
- `k_exchange` = β = 0 (not positive)
- `K_exchange` = α/β undefined
- median relative error ≈ 16.0% (> 15%)
- max relative error ≈ 35.4% (> 35%)
- leave-one-out: unstable (β remains 0)

## Scientific reading

At stationary surface balance the reversible law requires `L ≈ α A − β B` with α,β > 0.
Across the six governed states, B ≫ A and turnover L does not decrease with B in the manner a positive desorption term requires.
Weighted nonnegative least squares therefore collapses to a one-parameter (irreversible) projection with β = 0.
The two-parameter reversible exchange law is not identifiable from these states; further dynamic / Stage E work under this law is not authorized.

## Architecture note (out of scope for D-029)

Later work may consider energy-coupled membrane assembly, immature/mature membrane states, or a chemically powered nonequilibrium surface reaction. Do not add those inside D-029.

## Status

- D-008: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`
- Next: architect directive for a different exchange / assembly architecture (not Stage F; not productive-rate-only repair)

## Artifacts

`digital-protocell/experiments/generated/d029/` — preservation, exchange_unit, parameter_identification, manifest.json
