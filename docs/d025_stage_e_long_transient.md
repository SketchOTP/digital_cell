# D-025 — Stage E long transient unresolved

## Conclusion

`D025_STAGE_E_LONG_TRANSIENT_UNRESOLVED`

## Status

| Item | Value |
|---|---|
| D-024 | `D024_PROVENANCE_SEALED` (tag `D-024-surface-density-pass-provenance-sealed` preserved; prior pass tag preserved) |
| Architecture | `INTERFACIAL_SURFACE_DENSITY_SELECTED` |
| D-008 Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Production | `REQUIRES_REMEDIATION` |

## Gate summary

| Gate | Result |
|---|---|
| 0 D-024 provenance | `D024_PROVENANCE_SEALED` |
| 1 Manufactured velocity | PASS |
| 2 Autonomous passive surface | PASS |
| 3 Chemistry growth/shrinkage | PASS |
| 4 Stage B | PASS |
| 5 Stage C | PASS |
| 6 Stage D R16/R24/R32 | PASS |
| 7 Dynamic R22 | PASS |
| 8 Stage E constrained-radius | FAIL — long transient |

## Formal Gate 8 reference (R22, 200k)

- Source commit: `b776d892c9c563771d8e5fb78e0b697b2751dd85`
- Binary SHA256: `cee16d847038e6264d4cf79090e7e1fb267e218e9f0bbf5a19af13bf96f7e1bb`
- Equation: `membrane_metabolism_v7_surface_density`
- Frozen `k_ads`: `0.0011111111111111111`
- Accepted substeps: `200000`
- Classification: `NOT_CONVERGED_AT200K`
- Consecutive qualifying windows: `0`
- Γ localization: ≈ `1.0`
- C retention: ≈ `0.927`
- A retention: ≈ `0.512` (fails ≥ 0.80)
- Material accounting: closed
- Activation residual: ≈ `9×10⁻¹⁴`
- Restoring-radius / solver / robustness: not entered (`solver_recommended=false`)

## Scientific reading

Autonomous interfacial surface density survives Gates 0–7. Constrained-radius Stage E under frozen D-024 adsorption does not reach three consecutive quasi-steady windows within 200k accepted substeps. Activated retention collapses below Stage E thresholds while localization remains interfacial. No joint fixed point was demonstrated; `NOT_CONVERGED` does not prove absence of a solution.

## Forbidden next steps

- Do not begin D-008 Stage F
- Do not return to bulk M / χ / A→M / volumetric localization / target mass-radius
