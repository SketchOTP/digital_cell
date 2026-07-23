# D-082 — Edge-Membrane Activation Supply Integration Audit

## Conclusion

`D082_EDGE_ACTIVATION_INTEGRATION_REPAIRED` (Route I)

D-081 Gate 5 failed because the edge assay used a scalar `activated` bolus and never dispatched canonical N/F→A. Integrating `activated_metabolism_rates` (unchanged kinetics) and sourcing edge A→L from field A restores replenishment affordability (A ret≈0.82, post-repair=1.0; no-N/F/C and production-knockout controls flat).

## Entry

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Start | `41e9936` / `D-081-edge-reserve-causality-fail` |
| D-081 entry status | `PROVISIONAL_PENDING_ACTIVATION_SUPPLY_AUDIT` |

## Gate results

| Gate | Result |
|------|--------|
| 0 D-081 reproduce | **PASS** — Gate5 a_ret≈0; activation extent/N/F/W = 0 |
| 1 Lineage | **ACTIVATION_NOT_DISPATCHED** |
| 2 Parity | **PASS** — before fail / after match; integration repaired |
| 3 Energy ledger | **PASS** — A from N/F recorded; membrane A→L accounted |
| 4 Affordability | **PASS** — normal ok; controls flat |
| 5 Demand | diagnostic: continuous L overshoots seed (overproduction signal) while A holds |
| 6 Route | **Route I** |

## Resume after Route I

| Item | Result |
|------|--------|
| D-081 Gate 6 / D-080 Gate 7 | `PASS_AFTER_D081_RESERVE_CAUSALITY_AUDIT` |
| D-080 Gate 8 dynamic | **FAIL** — `D080_EDGE_NETWORK_DYNAMIC_INTERFACE_FAILURE` |
| D-080 Gate 9 coupled rows | **PASS** (R16/22/32 coverage=1; C/A ret≳0.97) |
| D-080 Gate 9 structural | **FAIL** — universally positive drive (`STRUCTURAL_INCOMPATIBILITY`) |

## Integration (no biology change)

- Activation: `activated_metabolism_rates` (schema/default k unchanged)
- Produce: existing `k_produce` / `yield_l_from_a`; A from field mass
- Supply proxy: interior N/F chemostat at reservoir (documents entry; not a kinetic retune)

## Status

- D-008: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`
- D-081: `SUPERSEDED_BY_D082_ACTIVATION_SUPPLY_AUDIT`

## Next directive

Repair dynamic-interface migration under fixed support/kinetics (D-080 Gate 8), then address structural restoring separately. Do not raise activation or change A→L yield first.

`next_execution_started`: false
