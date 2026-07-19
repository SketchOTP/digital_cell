# D-038 — Corrected Surface-Turnover Transfer and Membrane-Renewal Replay

## Primary conclusion

`D038_NO_MEMBRANE_ARCHITECTURE_RECOVERED` (Route A2)

## Preservation

- Branch: `d008-membrane-metabolic-closure`
- Starting commit: `67135e0`
- D-037 tag: `D-037-membrane-assumption-audit` (preserved)
- Recorded: `SURFACE_TURNOVER_TRANSFER_DEFECT_CONFIRMED`
- Historical D-021–D-037 tags/commits unchanged

## Turnover equations

| Schema | Equation |
|--------|----------|
| 1 (historical default) | `J = k_Γ · S` with `k_Γ = k_membrane_decay` |
| 2 (corrected) | `J = k_M · S · [ε_M + (1 − I(φ))]` |

`S = δΓ` is already embedded; schema 2 does **not** multiply by `δ` again.
`ε_M = 0.02`, `k_M = 0.002`.

## Gate results

| Gate | Result |
|------|--------|
| 0 Preservation | PASS |
| 1 D-021 equivalence | PASS (`max_rel ~ 1e-16`) |
| 2 Integrator / schema isolation | PASS |
| 3 D-024 substrate revalidation | PASS (`D024_SURFACE_SUBSTRATE_REVALIDATED_AFTER_D038`) |
| 4 Passive v8 multistart | FAIL — `D038_PASSIVE_RENEWAL_STILL_INCOMPATIBLE` |
| 6 Linear v11 candidates (≤5) | FAIL — `D038_LINEAR_MATURATION_STILL_INVALID` |
| 8 Catalytic v12 candidates (≤5) | FAIL — `D038_CATALYTIC_MATURATION_STILL_INVALID` |
| 10 Route | `ROUTE_A2` / no architecture selected |

## Passive multistart note

Under schema 2, `low_surface` showed Q descending from ~9.1 through ~0.92 (near band) then overshooting to reverse net exchange. High-occupancy starts remained Q ≪ 0.98. No three consecutive windows in `[0.98,1.02]` with `|g|≤1e-4`.

## Stage E / production

- D-008 Stage E: `BLOCKED_NOT_RECOVERED` (unchanged; turnover remains `MIXED_PURPOSE_TERM`)
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`

## Tests

`cargo test -p chemistry-core --release --test d038_tests` — 15/15 PASS

Omitted: full D-008–D-037 historical suite (equation defaults unchanged; schema 1 preserved).

## Artifacts

`digital-protocell/experiments/generated/d038/`

## Next directive

Fundamental turnover and membrane-renewal review. Do **not** auto-implement D-036. Do **not** begin Stage F.
