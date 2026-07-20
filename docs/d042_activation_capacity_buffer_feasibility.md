# D-042 — Activated-Resource Capacity and Buffer Feasibility

## Mission

Architecture audit: determine whether the A collapse of D-040 / D-041 is a persistent activation-production deficit, excess authorized demand, a finite temporal mismatch a conserved buffer could bridge, or a spatial mismatch requiring an energy carrier.

No field, reaction, A-transport, passive-exchange, or constitutive S→W change. No buffer implementation. No Stage E / Stage F.

## Frozen starting state

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Commit | `6bc2e1f` |
| Tag | `D-041-structural-a-bootstrap-fail` |
| Architecture | v8 reversible surface exchange |
| Turnover | schema 3, no constitutive S→W |
| A transport | historical, `ρ_A = 1` |
| Passive exchange | frozen α, β, K |
| Prior | `D040_MEMBRANE_METABOLISM_BISTABILITY`, `D041_STRUCTURAL_A_RETENTION_NOT_SUFFICIENT` |
| Record | `STRUCTURAL_A_TRANSPORT_RETENTION_REJECTED` |

## Gate results (governed ≥25 000 accepted)

| Gate | Result |
|------|--------|
| 0 Route-F reproduction | **PASS** — exchange parity; `A_PRODUCTION_DECLINE` earliest; healthy-A / sufficient-P / healthy-perm improve at full horizon (D-041 short control windows corrected; sufficient-P holds P against exchange drain); basins evidenced. |
| 1 Complete A ledger | **PASS** — production / demand / transport / reservoir / numerical partition closes. |
| 2 Persistent capacity | **`D042_ACTIVATION_CAPACITY_DEFICIT`** — integrated mean `R_A` remains largely negative under healthy permeability and sufficient P; no single optional demand disable restores a nonnegative integrated balance (structural disable is least bad but still ≪ 0). Late-window `R_A≈0` after free A collapses is **not** surplus evidence. |
| 3 Temporal buffer | **Skipped** — finite buffer forbidden under persistent capacity deficit. |
| 4 Spatial binding | **Skipped** — buffer path closed. |
| 5 Observer multistart | **Skipped** — buffer path closed. |

### Gate 2 integrated balances (evidence)

| Control | ∫ R_A dt |
|---------|----------|
| historical baseline | ≈ −759 |
| healthy permeability | ≈ −698 |
| sufficient P | ≈ −759 |
| precursor synthesis disabled | ≈ −704 |
| structural production disabled | ≈ −673 |
| catalyst reproduction disabled | ≈ −760 |
| surface exchange disabled | ≈ −37 |

## Primary conclusion

`D042_ACTIVATION_CAPACITY_DEFICIT`

### Selected route

`ROUTE_A_ACTIVATION_PRODUCTION_REPAIR`

Next directive must audit or repair the **activation reaction** itself. Do **not** add a conserved activation buffer.

## Scientific conclusion

A conserved activation buffer can only smooth a temporary or spatial mismatch between production and demand. Over the full governed horizon, net activated-resource balance is persistently negative even when membrane health is diagnostically maintained and when each major optional demand is disabled in turn. The deficit is therefore a **persistent activation-production shortfall**, not a finite repayable mismatch. Structural A-transport retention remains rejected (`STRUCTURAL_A_TRANSPORT_RETENTION_REJECTED`).

## Status constraints

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Tests

`cargo test -p chemistry-core --test d042_tests --release` — 13/13 PASS.

## Artifacts

`digital-protocell/experiments/generated/d042/` — preservation, route_f_reproduction, a_ledger, capacity_controls, temporal_deficit, spatial_deficit, buffer_feasibility, multistart, route_decision, accounting, manifest.json.

## Tag

`D-042-activation-buffer-feasibility`
