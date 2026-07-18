# D-032 — Activated Nonequilibrium Surface Assembly

## Conclusion

`D032_ACTIVE_ASSEMBLY_LAW_NOT_PORTABLE`

## Record

`PASSIVE_REVERSIBLE_EXCHANGE_INSUFFICIENT_FOR_MEMBRANE_MAINTENANCE`

## Summary

D-032 added `membrane_metabolism_v9_activated_surface_assembly` with metabolically powered

`P + A → S + W`, `J_active = k_active q(C) a p max(0,1−θ)`,

on the frozen v8 reversible exchange substrate (α≈0.167, β≈0.00334, invariant-domain integrator unchanged).

Gate 0 preservation and Gate 1 conservation/causality unit tests passed.

Gate 2 regenerated compact v8 isolated-turnover states and reconstructed

`k_active_required = (turnover − passive_net) / B_active`

with `B_active = ∫ δ q(C) a p (1−θ) dV`.

Five valid late states were obtained, but estimates are **not portable**:

| horizon | R_required | B_active | k_active_required |
|--------:|-----------:|---------:|------------------:|
| 25k | 0.816 | ~0.00527 | 154.6 |
| 50k | 1.157 | ~0.00664 | 174.2 |
| 100k | 1.319 | ~0.00374 | 352.6 |
| 150k | 1.504 | ~0.00183 | 823.6 |
| 200k | 1.661 | ~0.00090 | 1854.3 |

- span factor ≈ **12.0×** (limit 3×)
- leave-one-out median stability: **fail**
- median ≈ 352.6 (not usable under portability gate)

As coverage erodes, productive unsaturated interface measure collapses (`B_active`↓) while the desorption deficit (`R_required`) grows, so a single constant `k_active` cannot represent the required assembly rate across states.

## Escalation (authorized next architecture; not in D-032)

Directive escalation rule is met: required active-rate estimates are non-portable.

Next architecture may introduce an explicit activated or immature membrane species.
**Do not add that field inside D-032.**

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`
- Gates 3–15: not started (Gate 2 stop)

## Evidence

- Source commit (v9 chemistry): see git log `D-032: Add metabolically activated surface assembly`
- Artifacts: `experiments/generated/d032/`
- Tag: `D-032-activated-assembly-fail`
