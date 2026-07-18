# CURRENT.md

## Active directive
- ID: D-20260718-d031r-complete-isolated-renewal
- Project directive: D-031R / D-031
- Goal: Seal Gate 4 isolated renewal; classify D-031 terminal without Gate 5 unless PASS
- Status: done — `D031_TURNOVER_EXCHANGE_INCOMPATIBILITY_CONFIRMED`
- Acceptance: Gate 4 terminal sealed; Gate 5 not started — met
- Touched files: experiments/generated/d031/isolated_turnover/*, docs/d031_*, .agent/*, tag
- Next action: Architect follow-on; do not Stage F; do not Gate 5 under failed Gate 4

## Repo facts needed now
- Gate4: 206000 accepted, 0 capacity rejects, exited cleanly
- Brief near-balance at 10k (one window Q≈1.0026) then desorption-dominated divergence
- Late 3 windows: Q≈−10.29…−10.37, g≈−0.0226…−0.0227 (same direction)
- Commits: `3b3d033`, `f7a3dca`; Mimir BLOCKED at close

## Last validation
- Command: sealed isolated_turnover.json (process exited; conclusion printed)
- Result: D031_TURNOVER_EXCHANGE_INCOMPATIBILITY_CONFIRMED; Gate5 not started

## Open blockers
- Identified v8 reversible kinetics incompatible with sustained isolated biological renewal under invariant integration
- D-008 remains BLOCKED_NOT_RECOVERED
