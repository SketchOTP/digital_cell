# CURRENT.md

## Active directive
- ID: D-20260716-d018-structural-constraint-nullcline
- Project directive: D-018
- Goal: Structural constraint provenance and nullcline recovery diagnosis
- Status: done
- Acceptance: One primary D018_* conclusion with tracer/unconstrained/scaling evidence — met
- Touched files: d018_provenance, d018_analysis, simulation, d018_tests, experiment-runner/d018, docs/d018_*, append-only reports
- Next action: Next directive — compare phase-volume synthesis vs interface-limited turnover vs curvature/thickness coupling

## Repo facts needed now
- Primary: D018_SURFACE_VOLUME_SCALING_INCOMPATIBLE
- Subsidiary: D018_CONSTRAINT_WASTE_ARTIFACT_CONFIRMED
- Tag: D-018-surface-volume-scaling-incompatible
- Production~R^1 interface; decay~R^2 bulk; k_req rises with R
- Unconstrained: STRUCTURE_COLLAPSE_LIMITS_W_SOURCE
- D-012 solver: CLOSED; Stage E BLOCKED
- Mimir slug: digital_cell

## Last validation
- Command: cargo test -p chemistry-core --release --test d008/d011/d012/d013/d014/d015/d016/d017/d018
- Result: PASS (exit 0; d016 24, d017 17, d018 27 confirmed in log)

## Open blockers
- None for D-018; Stage E remains blocked pending spatial structure redesign
