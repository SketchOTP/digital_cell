# D-030 — Orthogonal Transient Identification of Reversible Exchange

## Conclusion

`D030_TURNOVER_EXCHANGE_INCOMPATIBILITY`

## Operative status

| Item | Status |
|------|--------|
| D-021–D-029 | Preserved (commits, tags, artifacts unchanged) |
| D-029 historical | `D029_REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE` (unchanged) |
| D-029 operative reinterpretation | `REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE_FROM_NATURAL_BALANCE_STATES` |
| Record preserved | `IRREVERSIBLE_ADSORPTION_LAW_REJECTED` |
| D-030 | `D030_TURNOVER_EXCHANGE_INCOMPATIBILITY` |
| D-008 | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Production | `REQUIRES_REMEDIATION` |

## Branch / starting commit

- Branch: `d008-membrane-metabolic-closure`
- Starting result commit: `9c4a4ea` (`D-029: Preserve reversible surface-exchange failure`)
- D-029 failure tag: `D-029-reversible-exchange-fail`

## Equation / schema

- Equation: `membrane_metabolism_v8_reversible_surface_exchange`
- Exchange schema: `2`
- Law retained: `J_exchange = J_forward − J_reverse` with α = k K, β = k

## Gate summary

| Gate | Result |
|------|--------|
| 0 Preservation | PASS |
| 1 Observability | PASS (forward-only at θ=0; reverse-only at P=0) |
| 2 Forward α ID | PASS |
| 3 Reverse β ID | PASS |
| 4 Parameter recovery | PASS (`D030_EXCHANGE_PARAMETERS_IDENTIFIED`) |
| 5 Mixed cross-validation | PASS |
| 6 Equilibrium families | PASS (fixed-inventory partition independence) |
| 7 Isolated turnover | FAIL — `D030_TURNOVER_EXCHANGE_INCOMPATIBILITY` |
| 8–17 | Not run (Gate 7 stop) |

## Orthogonal identification (Gates 2–4)

Direct first-substep estimators recover planted seed kinetics within numerical noise:

- α_direct ≈ k × K (median over 3×3 adsorption matrix; spread ≪ 10%)
- β_direct ≈ k (median over 3×3 desorption matrix; spread ≪ 10%)
- q(C) normalization: spread ≪ 10%
- Bootstrap spread factor ≈ 1.0; leave-one-out within 25%
- Reconstructed `k_exchange = β_direct`, `K_exchange = α_direct/β_direct`

This confirms: when reverse exchange is orthogonally excited, β is measurable. D-029’s β→0 was an excitation/identifiability failure of natural balance states, not proof that reverse kinetics are absent from the law.

## Seed screen (pre-orthogonal)

Isolated-renewal seed search at fixed K (values tried include K=20 and K=50) did not produce three consecutive windows with `0.98 ≤ Q_renewal ≤ 1.02` before orthogonal ID. Best screened seeds still under-adsorb relative to biological Γ turnover (Q ≪ 1) while keeping both forward and reverse extents active.

## Gate 7 failure mode

Under D-027 isolated surface-renewal with orthogonally identified (α, β):

- Short seed screens can show active forward/reverse exchange and finite Q
- Full Gate 7 horizon (12k accepted-step design) hits `surface_exchange_reject:CapacityExceeded`
- Measurement windows then accept 0 steps → Q/turnover read as zero
- No three consecutive windows with `0.98 ≤ Q_renewal ≤ 1.02`
- Conclusion per directive: `D030_TURNOVER_EXCHANGE_INCOMPATIBILITY`

Reversible exchange architecture is **not** rejected by this gate. Rejection requires portability/regression failure after successful direct kinetics (Gate 8+), which were not authorized after Gate 7 stop.

### Identified pair at stop (example from final run)

- `α_direct ≈ 0.167`
- `β_direct ≈ 0.00334`
- `k_exchange ≈ 0.00334`
- `K_exchange ≈ 50`

## Disk

Compact transient artifacts only (~6–7 GiB free during run; 98% disk used). No full checkpoints created.

## Artifacts

`digital-protocell/experiments/generated/d030/` — preservation, exchange_observability, adsorption_transients, desorption_transients, parameter_recovery (incl. seed_screen), mixed_cross_validation, equilibrium_families, isolated_turnover, manifest.json

## Tests

`cargo test -p chemistry-core --release --test d030_tests` — 11/11 PASS (observability, α/β recovery, bootstrap/LOO, mixed direction, equilibrium partition independence, v8 immutability)

## Next

Do not Stage F. Do not productive-rate-only repair. Architect follow-on may:

1. Expand renewal-compatible (α, β) search under orthogonal constraints; or
2. Only after orthogonal+portability failure, consider energy-coupled irreversible assembly / immature–mature membrane states per D-030 §23.
