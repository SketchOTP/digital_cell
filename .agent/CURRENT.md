# CURRENT.md

## Active directive
- ID: D-20260720-d051-coupled-activation-throughput-bottleneck-audit
- Project directive: D-051
- Goal: Diagnose coupled activation throughput bottleneck (diagnostic only)
- Status: done
- Acceptance: met — primary `D051_RESOURCE_THROUGHPUT_LIMIT`
- Touched files: d051_analysis, d051_tests, d051.rs, main.rs, docs/d051_*, experiments/generated/d051, .agent/*
- Next action: N/F delivery / permeability / reservoir / activation-yield review; next_execution_started=false

## Repo facts needed now
- D-050 sealed 0b0fb89 / D-050-catalyst-saturating-activation-fail
- Gross activation rises with V_A; free A ~3% until healthy N+F held
- Volume activation: no hard min(N,F) extent clip; rate-law soft + positivity
- Max control with healthy N/F: COUPLED_ACTIVATION_TOPOLOGY_CAPABLE
- Record: CATALYST_SATURATING_CAPACITY_REPAIR_REJECTED

## Last validation
- Command: cargo test d051_tests 13/13; D051_MAX_ACCEPTED=10000 pipeline
- Result: primary=D051_RESOURCE_THROUGHPUT_LIMIT

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
- Do not add C_star/buffer/product inhibition/new activation law from D-051 alone
