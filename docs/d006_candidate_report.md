# D-006 Candidate Report

**Directive:** D-006 / D-006C  
**Agent memory:** D-20260713-d006c-surface-turnover-completion  
**Equation version:** `surface_turnover_v1`

## Derived interface rate

`k_structure_interface_initial ≈ 0.09642857142857159`

## Immutable candidates

Five candidates; Stage D scheduled only prescribed survivors:

| Factor | Prescribed crossing | Stage D coupled restoring |
| --- | --- | --- |
| 0.60× | fail (excluded) | n/a |
| 0.80× | pass | **fail** (all median v_R > 0) |
| 1.00× | pass | **fail** (all median v_R > 0) |
| 1.20× | pass | **fail** (all median v_R > 0) |
| 1.40× | pass | **fail** (all median v_R > 0) |

## Job matrix

`4 × 5 × 3 × 3 = 180` (not 225 — 0.60× rejected at prescribed-radius).

Confirmed from `candidates/index.json`, `prescribed_radius/*/result.json`, and `/tmp/d006_screen_jobs.txt`.

## Scientific conclusion

```text
D006_NO_RESTORING_RADIUS
```

Selected candidate: **none**.

Execution status that applied while Stage D ran:

```text
D006_RESULT_UNRESOLVED_STAGE_D_IN_PROGRESS
```
