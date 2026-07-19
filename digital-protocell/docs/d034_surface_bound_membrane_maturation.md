# D-034 — Surface-Bound Membrane Precursor Maturation

## Conclusion

`D034_MATURATION_LAW_NOT_PORTABLE`

## Summary

D-034 replaced the failed soluble activated intermediate `X` with two interfacial membrane states under

`membrane_metabolism_v11_surface_maturation`

nine-field state `(φ, C, N, F, W, A, P, U, S)` where:

- `P` = soluble inactive precursor
- `U = δΓ_U` = immature interface-bound precursor
- `S = δΓ_S` = mature functional membrane

Shared capacity: `θ_total = θ_U + θ_S ≤ 1`.

Reactions:

1. passive exchange: `P ↔ U` with frozen α≈0.167, β≈0.00334 (invariant-domain V2)
2. maturation: `U + A → S + W`, `J_mature = k_mature q(C) a Γ_U`
3. turnover: `S → W` only (U has no independent biological decay)

Recorded: `SOLUBLE_ACTIVATED_INTERMEDIATE_REJECTED`.

v10 snapshots cannot resume as v11 (`NineFieldSurfaceMaturationV1` ≠ `NineFieldSurfaceDensityV1`).

| Gate | Result |
|------|--------|
| 0 Preservation / schema | PASS |
| 1 Conservation / causality (unit) | PASS 9/9 |
| 2 Passive U exchange regression | PASS (α/β within 2%) |
| 3 Dual-surface transport smoke | PASS |
| 4 Orthogonal maturation ID | PASS (planted k recovered within 15%) |
| 5 Functional maturation smoke | PASS |
| 6 Analytical rate reconstruction | **FAIL** |
| 7–15 | not started |

## Gate 6 mechanism

Across mandated fixed-interface renewal states (highU/lowS, balanced, lowU/highS, low/med/high A):

- six valid estimates
- `k_mature_required = L_S / B_mature` with `L_S = ∫ δ k_Γ Γ_S`, `B_mature = ∫ δ q a Γ_U`
- span ≈ **33×** (limit ≤3×)
- leave-one-out medians within 50% (passes LOO; fails span)

Algebraically `k_req ∝ Γ_S / (q a Γ_U)`, so forced U/S occupancy ratios produce non-portable required rates even when local maturation kinetics are identifiable (Gate 4).

## Stop rule

Causality, passive exchange, and local maturation ID pass, but portable bounded renewal rate reconstruction fails.

Preserved without:

- adding another bulk or surface species
- altering permeability or turnover
- retuning α, β, or productive rates
- starting Stage F

Next: architecture review of membrane-bound catalytic assembly (not automatic species escalation).

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`
- Stage F: not started

## Evidence

- Artifacts: `experiments/generated/d034/` (manifest, preservation, passive_exchange_regression, maturation_identification, rate_reconstruction)
- Unit tests: `cargo test -p chemistry-core --test d034_tests` (9/9)
- Tag: `D-034-surface-maturation-fail`
