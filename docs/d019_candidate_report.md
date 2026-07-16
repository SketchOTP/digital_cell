# D-019 Candidate Report — Structural Scaling Repair

## Primary conclusion

**D019_SELECT_INTERFACE_LIMITED_TURNOVER**

Full pass `D019_STRUCTURAL_SCALING_REPAIR_PASS` **not met** — Stage E R22 did not
reach quasi-steady convergence.

## Preserved D-018

- Primary: `D018_SURFACE_VOLUME_SCALING_INCOMPATIBLE`
- Subsidiary: `D018_CONSTRAINT_WASTE_ARTIFACT_CONFIRMED`

## Selected mechanism

Interface-limited structure turnover under
`membrane_metabolism_v3_structural_scaling` (structural schema 1; stoich schema 2).

Decay: `r = k_structure_decay · φ · (0.05 + I(φ))`. Production unchanged
(`k · A · I(φ)`).

## Foundational gates

| Gate | Result |
| --- | --- |
| Conservation (v2 matrix / activation) | PASS |
| Stage B localization | `D019_STAGE_B_LOCALIZATION_PASS` |
| Stage C metabolism | `D019_STAGE_C_METABOLISM_PASS` |
| Stage D fixed compartments R16/24/32 | `D019_STAGE_D_FIXED_COMPARTMENT_PASS` |

## Structural pre-balance

- Restoring crossing at `k_center ≈ 0.2576`
- `g(R18)>0`, `g(R22)≈0`, `g(R26)<0`
- Max constraint contamination ≈ 0.0016 ≤ 0.05
- Unconstrained: `UNCONSTRAINED_STRUCTURE_STABLE` (improved vs D-018 collapse)

## Stage E R22

| Attempt | k_structure | Classification | Notes |
| --- | --- | --- | --- |
| 1 | 0.2576 (prebalance) | `NOT_CONVERGED_AT200K` | 0 rejections; windows valid but not qualifying; `Q_structure≈0.115` |
| 2 | 2.236 (Q-corrected) | `NOT_CONVERGED_AT200K` | `Q_structure≈0.216`; companion rates still unbalanced |

No R18/R26 validation (center did not converge).

## Status retained

- D-008 Stage E: **BLOCKED_NOT_RECOVERED** (not `STAGE_E_REFERENCE_RECOVERED`)
- D-012 solver: **CLOSED**
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: requires joint-rate remediation under v3

## Next

Progressive multi-rate Stage E screen / reopen four-rate joint balance under
`membrane_metabolism_v3_structural_scaling`, then re-run R22 → R18/R26.

## Artifacts

`digital-protocell/experiments/generated/d019/` (gitignored generated tree)
