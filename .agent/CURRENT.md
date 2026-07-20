# CURRENT.md

## Active directive
- ID: D-20260720-d045-fuel-charged-catalyst-activation-cycle
- Project directive: D-045
- Goal: Fuel-charged catalyst activation cycle
- Status: done
- Acceptance: met — honest `D045_CATALYST_LINEARITY_REJECTED`; C_star not implemented
- Touched files: d045_analysis.rs, d045.rs, d045_tests.rs, lib.rs, main.rs, docs/d045_*, experiments/generated/d045, .agent/*
- Next action: A-demand topology review; next_execution_started=false

## Repo facts needed now
- D-044 sealed 1473f07 / D-044-activation-law-fail
- Gate0: d_C span 2.59× OK; radius bias OK; catalyst-linear err 52.7% FAIL
- Demand scales with volume/radius, not C loading

## Last validation
- Command: cargo test d045_tests 5/5; d045 pipeline Gate-1/0
- Result: primary=D045_CATALYST_LINEARITY_REJECTED

## Open blockers
- Authorized A demand topology incompatible with catalyst-linear production
