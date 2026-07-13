# D-005 Final Closure

**Directive:** D-005  
**Agent memory ID:** D-20260713-d005-accessible-active-attractor  
**Closure session:** D-20260713-d006-surface-turnover-protocell / project:D-006

## Final scientific conclusion

```text
D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR
```

for the tested D-003 crowding kinetic architecture and machine-extracted parameter domain.

## Status codes

```text
D-004:
D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT

D-005:
D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR

D-003:
D003_RESULT_UNRESOLVED_PENDING_ACCESSIBLE_ATTRACTOR_SEARCH

Phase 1:
PHASE1_SELF_MAINTENANCE_PARTIAL
```

## Mandatory evidence — complete

| Item | Status | Result |
| --- | --- | --- |
| 9× fresh-state 250k continuations | complete | 9/9 `CONTINUED_DRIFT`; 0 stable windows; Qφ≈0.37–0.40 |
| Coarse basin map | complete | 25 points (K_phi=1.0 grid); mixed decline/near-balance/transient growth vs catalyst load |
| Macrostate flow field | complete | `experiments/generated/d005/macrostate_flow/flow.json` (25 points) |
| Radius nullcline analysis | complete | **0** intersections with joint v_R/v_C sign change |
| Catalyst nullcline analysis | complete | No stable fixed-point classification |
| Fixed-point classification | complete | `stable_fixed_points = 0` |
| Control runs | implemented in pipeline; not required for failure closure |
| Final D-005 manifest | `experiments/generated/d005/manifest.json` | conclusion above |
| Final D-005 reports | this file + historical `docs/d005_*.md` | |

## Why not a restoring active region

- Fresh analytic/calibration seeds do **not** approach a bounded active state at 250k.
- Coarse grid near `C₀≈0.35` shows extensive near-balance at **every** tested radius — production≈decay without `dR/dt` restoring about a unique `R*`.
- No radius–catalyst nullcline intersection with stable classification.
- Gate criteria satisfied: continued decline/drift from every fresh seed; no restoring radial flow about an accessible fixed point.

## Background processes

See `experiments/generated/d006/d005_closure/background_process_ledger.json`.

| Process | PID | Outcome |
| --- | --- | --- |
| Initial `d005 pipeline` | 65669 | Interrupted mid k_phi=1 seed2 (memory pressure; swap full) |
| Separate `d005 coarse-basin --k-phi 1.0` | 120389 | Completed 25/25 coarse points |
| Resume `d005 continuations` (skip-complete) | 454680 | Exit 0; 9/9 continuations; start 2026-07-13T19:22:21Z |

## Untracked large artifacts

Per `.gitignore` `experiments/generated/`:

- `digital-protocell/experiments/generated/d004/`
- `digital-protocell/experiments/generated/d005/`
- `digital-protocell/experiments/generated/d006/`

Tracked policy: source, tests, configs under `configs/`, and docs reports.

## Next

Proceed to D-006 `surface_turnover_v1` redesign (no accessible D-005 attractor found).
