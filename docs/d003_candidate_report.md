# D-003 Candidate Report

**Status:** D-005 accessible-attractor search complete (preliminary)  
**D-003 scientific conclusion:** `D003_RESULT_UNRESOLVED_PENDING_ACCESSIBLE_ATTRACTOR_SEARCH` (unchanged pending pass branch)  
**D-004 audit (2026-07-13):** original `D003_FAIL` **invalidated by pipeline defect** — Stage B screened analytical estimates, not calibrated candidates.  
**D-005 audit (2026-07-13):** No accessible active attractor from fresh analytical seeds; see `docs/d005_candidate_report.md`.

## Short-screen results (Stage B — legacy, defective handoff)

0/3 seeds pass. All classified Transient. Qφ ≈ 0.65–0.66, QC ≈ 1.75–1.81, retention ≈ 0.92. **These metrics apply to analytical `K_phi=1.0` params (k_structure≈0.092), not final calibrated candidate (k_structure≈0.141).**

Artifacts: `experiments/generated/d003/short_screen/seed_{1,2,3}.json` (preserved unchanged)

## Corrected Stage B (K_phi=1.0 calibrated candidate, 100k)

| Seed | Qφ | QC | Pass |
|------|-----|-----|------|
| 1 | 0.863 | 0.898 | yes |
| 2 | 0.757 | 0.754 | no |
| 3 | 0.757 | 0.754 | no |

## Calibration results

Three K_phi branches (0.5, 1.0, 2.0) each ran 6 iterations. None achieved two consecutive passing balance windows. Best Qφ ≈ 0.983; slopes remain ~3.7×10⁻⁴ (above 1×10⁻⁴ gate).

## Full acceptance results

Not run (250k×5). D-005 continuation of k_phi=0.5 fresh seed 1 to 250k shows continued decline (Qφ→0.40).

## Overall Phase 1 conclusion

**Unchanged:** `PHASE1_SELF_MAINTENANCE_PARTIAL`

Passing D-003 active steady-state would not upgrade Phase 1 until starvation, knockout, repair, damage, death, and Godot-equivalence suites complete (deferred per §23).

## Revision 2026-07-13 (preserved under D-006)

D-003 remains `D003_RESULT_UNRESOLVED_PENDING_ACCESSIBLE_ATTRACTOR_SEARCH`.
D-005 accessibility search completed without an accessible active attractor under crowding kinetics.


## Revision (D-006C)

No change to D-003 candidate identity. Downstream D-006C concluded `D006_NO_RESTORING_RADIUS` under `surface_turnover_v1`.


## Revision (D-007 start)

No change to D-003 candidate identity. D-007 searches within `surface_turnover_v1` after D-006 failure; Phase 1 remains `PHASE1_SELF_MAINTENANCE_PARTIAL`.
