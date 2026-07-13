# CURRENT.md

## Active directive
- ID: D-20260713-d006-surface-turnover-protocell
- Project directive: D-006
- Goal: Close D-005 evidence; if no restoring region, implement surface_turnover_v1 restoring-radius chemistry
- Status: started — awaiting D-005 pipeline (PID 65669) before chemistry changes
- Acceptance: Mandatory D-005 evidence complete; one D-006 conclusion; surface_turnover artifacts+tests if executed
- Touched files: (pending D-005 closure) docs/d005_final_closure.md, experiments/generated/d006/, chemistry-core reactions
- Next action: Monitor/finish D-005 9×250k continuations + nullclines; then integrity commit/tags

## Repo facts needed now
- D-005 pipeline running since ~11:42 local; coarse basin 25/25 done; continuations 4/9 done
- Manifest prelim conclusion D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR is premature until full evidence
- D-004 commit/tag still blocked until local git identity set

## Last validation
- Command: prior session cargo test d005+d004
- Result: 41/41 PASS (historical)

## Open blockers
- D-005 pipeline still running (continuations kphi_1 seed2+)
- Dirty working tree; no D-004/D-005 commit yet
