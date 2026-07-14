# D-007 Candidate Report

**Directive:** D-007  
**Agent memory:** D-20260714-d007-joint-kinetic-nullclines  
**Equation version:** `surface_turnover_v1` (frozen)

## Governing status

```text
D-005: D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR
D-006: D006_NO_RESTORING_RADIUS
Phase 1: PHASE1_SELF_MAINTENANCE_PARTIAL
```

## D-006 preservation

| Item | Value |
| --- | --- |
| Commit | `6b25d9b64d44d682518d97271e5ed92cd4475b7c` |
| Tags | `D-006-surface-turnover-stage-d`, `D-006-surface-turnover-closed` |
| Manifest content hash | `dd0b5e5ed5d713bf7ba6e5bee6f1abec9a7aa66ead1924fdc696f29bada97f14` |

## Strict schema

Runner `experiment-runner d007 run-one` emits the full §6 provenance schema. D-007 tests require schema completeness, clean termination, accounting fields, and hash-parameter matching (`d007_tests`: 26 PASS).

## Reference replay

| Horizon | v_R | v_C_inside | Direction vs D-006 failure |
| --- | ---: | ---: | --- |
| 10k (directive) | −0.02486 | −0.00190 | fails (`v_R` not yet > 0) |
| 50k corroboration | +0.06637 | −0.00146 | **passes** (matches Stage D) |

Configuration hash matches D-006 1.0× survivor: `53c5fd482d171d8a5d20dfbc16e7fdc1f1fc782d06d98c659c1a82fd23a172bb`.

## Catalyst-rate estimate

`k_rep_center ≈ 0.014908` (within 3× bound). Unused for screening after structural gate.

## Structural bracket

63/63 runs. No provisional restoring factor. Gate: `D007_NO_STRUCTURAL_NULLCLINE`.

## Later stages

Catalyst bracket, joint candidates, J1/J2, basin, puncture, controls, full acceptance: **not run** (§10).

## Selected candidate

**none**

## Scientific conclusion

```text
D007_NO_STRUCTURAL_NULLCLINE
```

Bounded rate correction alone cannot create a structural radius nullcline for `surface_turnover_v1`. The next model needs a deeper physical mechanism (explicit local transport boundary or additional chemical intermediate), not another `k_structure_interface`/`k_rep` sweep.

## Phase 1

```text
PHASE1_SELF_MAINTENANCE_PARTIAL
```
