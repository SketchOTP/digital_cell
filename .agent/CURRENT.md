# CURRENT.md

## Active directive
- ID: D-20260715-d015-waste-throughput-closure
- Project directive: D-015
- Goal: Diagnose waste UNBOUNDED_ACCUMULATION; repair throughput or classify scientific failure
- Status: done (scientific failure after env repair)
- Acceptance: Primary D015_* conclusion with waste causal chain — met as INTERNAL_WASTE_PRODUCTION_IMBALANCE
- Touched files: d015_waste, reservoir, config, simulation, d015 runner/tests, docs/d015_*, d015 artifacts
- Next action: Next directive — reaction-network / waste-processing repair from source decomposition (do not open D-012 solver)

## Repo facts needed now
- Frozen organism hashes unchanged (9a452d… / 87ff7e…)
- Env repair: waste_sink_inner_radius=30 (schema v2); N/F mask unchanged
- Clearance CORRECT; bulk sink idle was real; after repair exterior clears but interior still ceilings
- Fresh R22 repaired: UNBOUNDED_ACCUMULATION @ 162073; waste budget residual ~3e-14
- D-012 solver: CLOSED
- Mimir slug: digital_cell (MCP unavailable)

## Last validation
- Command: regression gate PASS; d015_tests 32/32; preflight PASS; fresh-r22 UNBOUNDED_ACCUMULATION
- Result: D015_INTERNAL_WASTE_PRODUCTION_IMBALANCE

## Open blockers
- BLOCKED: Mimir MCP unavailable
- D-012 solver CLOSED
- Stage E quasi-steady blocked by intrinsic interior W production vs export throughput
