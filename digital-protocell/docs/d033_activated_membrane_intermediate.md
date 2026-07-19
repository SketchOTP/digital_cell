# D-033 — Activated Membrane Intermediate

## Conclusion

`D033_ISOLATED_RENEWAL_FAILURE`

## Summary

D-033 replaced the failed instantaneous `P + A → S + W` law with

`membrane_metabolism_v10_activated_intermediate`

nine-field state `(φ, C, N, F, W, A, P, X, S)` and two-stage reactions:

1. charge: `P + A → X + W`, `r_charge = k_charge H(φ) q(C) P A`
2. insert: `X → S`, `r_insert = k_insert δ X max(0,1−θ)`
3. relax: `X → P`, `r_relax = k_relax X`

Frozen substrate retained: α≈0.167, β≈0.00334, invariant-domain V2 exchange, Γ turnover, surface transport, `D_X = D_P`, no interface attraction.

| Gate | Result |
|------|--------|
| 0 Preservation / observability | PASS |
| 1 Conservation / causality (unit) | PASS 9/9 |
| 2 Orthogonal rate ID | PASS (assay truths 0.8 / 1.2 / 0.25 recovered) |
| 3 Buffering proof | PASS (insertion continues after A removal) |
| 4 Numerical safety | PASS |
| 5 Isolated biological renewal | **FAIL** |
| 6–11 | not started |

## Gate 5 mechanism

Operating-rate screens (including `k_insert` up to 80) show:

- early windows can approach `Q≈1`
- later windows develop a growing desorption deficit (`passive_net →` large negative)
- insertion plateaus while bulk `X` accumulates
- under frozen `D_X = D_P` with no interface attraction, charged intermediate does not deliver portable interface renewal

## Escalation (authorized next; not in D-033)

Stop rule met: kinetics are identifiable and causal, but no portable bounded renewal exists for the one-soluble-intermediate architecture.

Next permitted escalation: **separate immature and mature surface membrane states** — not another rate change or transport adjustment.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`
- Stage F: not started

## Evidence

- Commit: `D-033: Add activated membrane intermediate`
- Artifacts: `experiments/generated/d033/` (manifest, kinetics, buffering, numerical, isolated_renewal, screens)
- Tag: `D-033-activated-intermediate-fail`
