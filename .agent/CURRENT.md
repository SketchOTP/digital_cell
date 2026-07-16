# CURRENT.md

## Active directive
- ID: D-20260715-d014-stage-e-numerical-stability
- Project directive: D-014
- Goal: Repair constrained-radius TIMESTEP_FLOOR_FAILURE and activation residual; fresh R22 under frozen model
- Status: done
- Acceptance: cause classified + evidence-matched repair; preflight+fresh R22 valid; one D014_* conclusion
- Touched files: chemistry-core (simulation, fields, d012_accounting, d014_numerics), experiment-runner (d013, d014), docs/d014_*, d014 artifacts
- Next action: none — D-012 solver remains closed (UNBOUNDED_ACCUMULATION, not quasi-steady)

## Repo facts needed now
- Frozen candidate/config hashes unchanged
- D-013 R22 preserved; not overwritten
- Failure cause: FIELD_BOUND_VALIDATION waste ceiling (machine-scale then hard bound)
- Repair: Branch E projection + unbound mapping; activation step identity; dt recovery hygiene
- Fresh R22: UNBOUNDED_ACCUMULATION at 161157; activation rel residual ~5e-13
- Mimir slug: digital_cell (MCP unavailable this session)

## Last validation
- Command: d014_tests PASS; preflight PASS; diagnostic PASS; fresh-r22 UNBOUNDED_ACCUMULATION; nonstiff equal-time max_rel≈2.4e-5
- Result: D014_NUMERICAL_VALIDITY_RESTORED

## Open blockers
- BLOCKED: Mimir MCP unavailable
- D-012 solver CLOSED until quasi-steady R22
- Waste accumulation hits CONC_SAFETY_LIMIT (scientific, not numerical floor)
