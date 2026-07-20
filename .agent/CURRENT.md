# CURRENT.md

## Active directive
- ID: D-20260720-d046-activated-resource-demand-topology-audit
- Project directive: D-046
- Goal: Activated-resource demand topology audit
- Status: done
- Acceptance: met — `D046_MIXED_A_DEMAND_TOPOLOGY`; no chemistry/C_star change
- Touched files: d046_analysis.rs, d046_tests.rs, d046.rs, lib.rs, main.rs, docs/d046_*, experiments/generated/d046, .agent/*
- Next action: shared A-pool sufficiency review; next_execution_started=false

## Repo facts needed now
- D-045 linearity rejection is PROVISIONAL (25% not in issued directive)
- Dominant sink: precursor (~76%); eps_V~0.95; eps_C~0.21
- Route M: valid sinks, no single local activation basis across load families

## Last validation
- Command: cargo test d046_tests; d046 pipeline Gates0-9
- Result: primary=D046_MIXED_A_DEMAND_TOPOLOGY

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; one shared A pool may be structurally insufficient
