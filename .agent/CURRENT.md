# CURRENT.md

## Active directive
- ID: D-20260715-d016-waste-transport-timescale
- Project directive: D-016
- Goal: Quantify/repair intracellular W transport timescale before chemistry changes
- Status: done (passive transport insufficient)
- Acceptance: One D016_* conclusion with governed evidence — met as PASSIVE_WASTE_TRANSPORT_INSUFFICIENT
- Touched files: d016_transport, d016_tests, d016 runner, config transport_schema, docs/d016_*, d015/d012 appends
- Next action: Next directive — compare activation-yield repair vs energy-coupled active W export

## Repo facts needed now
- D016_PASSIVE_WASTE_TRANSPORT_INSUFFICIENT + D016_INTERNAL_DIFFUSION_LIMIT_CONFIRMED
- D_W_required(50%)≈1.057 ≫ authorized bound 0.18; baseline D_W=0.25 already faster than N/F
- Fixed-source baseline: CONCENTRATION_BOUND_REACHED @ 175303 steps / t≈438
- Gate point D_W=0.18 β_W=0: still CONCENTRATION_BOUND_REACHED
- Chemistry/environment frozen; transport_schema remains 1
- D-012 solver: CLOSED
- Mimir slug: digital_cell

## Last validation
- Command: cargo test -p chemistry-core --release --test d008/d011/d012/d013/d014/d015/d016
- Result: all PASS (247 tests across suites)

## Open blockers
- BLOCKED: Mimir MCP unavailable (try HTTP at end)
- Passive W transport falsified inside small-solute bound; chemistry/active-export next
