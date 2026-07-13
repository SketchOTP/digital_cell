# D-005 Candidate Report

**Directive:** D-005  
**Agent memory ID:** D-20260713-d005-accessible-active-attractor

## Scientific conclusion

```text
D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR
```

## Revised status codes

```text
D-004: D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT
D-003: D003_RESULT_UNRESOLVED_PENDING_ACCESSIBLE_ATTRACTOR_SEARCH
Phase 1: PHASE1_SELF_MAINTENANCE_PARTIAL
```

## Evidence summary

1. **D-004 cross-state (21 runs):** No fresh seed reaches bounded active balance at 100k; all fresh runs `ContinuedDrift`.
2. **Long-transient (250k):** k_phi=0.5 fresh seed 1 continues collapse (Qφ 0.865→0.404; 0 stable windows).
3. **Seed sensitivity:** Corrected Stage B passes 1/3 seeds at K_phi=1.0.
4. **Accessibility:** Calibration-endpoint states show better Qφ but violate fresh-seed reachability requirement (§16).
5. **Structural failure gate (§19):** Fresh seeds cannot approach stable bounded state; no restoring radius observed in cross-state data.

## Selected chemistry

**None selected for promotion.** No candidate converges from the greatest number of fresh seeds with stable windows.

## Rate correction

Not applied — structural failure gate triggered (continued decline on fresh continuations).

## Full acceptance / controls

Full 250k×5 acceptance not run (no candidate passed selection). Controls implemented in pipeline; results in `experiments/generated/d005/controls/` when pipeline completes.

## Next directive

Redesign reaction kinetics architecture (D-005 §19, §22). Do not continue seed tuning or calibration-only endpoints.

## Revision 2026-07-13 (D-006 closure gate)

Final conclusion after completing 9/9×250k continuations, coarse basin, flow, and nullclines:

```text
D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR
```

Evidence: all fresh continuations `CONTINUED_DRIFT`; 0 nullcline intersections; no restoring R*. See `docs/d005_final_closure.md`.
