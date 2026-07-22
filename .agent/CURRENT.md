# CURRENT.md

## Active directive
- ID: D-20260722-1114-d069-mature-membrane-exchange-desorption-audit
- Project directive: D-069
- Goal: Audit reversible P↔S exchange equilibrium and desorption under frozen precursor (shadow-only)
- Status: done
- Acceptance: met — `D069_MEMBRANE_EXCHANGE_EXECUTION_DEFECT`
- Touched files: d069_analysis/tests, experiment-runner/d069, main.rs, lib.rs, docs/d069_*, experiments/generated/d069, .agent/*
- Next action: repair mature-S seed ≤ δ·Γ_max under frozen kinetics; next_execution_started=false

## Repo facts needed now
- Primary: Route X `D069_MEMBRANE_EXCHANGE_EXECUTION_DEFECT`
- D-068 desorption reproduced; des≈99.666 = over_capacity_mass≈99.667; S0/capacity≈2.31
- Analytical exchange law OK; defect is seed/capacity contract, not K_eq
- No exchange/precursor/activation production change

## Last validation
- Command: cargo test -p chemistry-core --test d069_tests; D069_MAX_ACCEPTED=1200 D069_SKIP_LATE_GATES=1 pipeline
- Result: 17/17 PASS; primary MembraneExchangeExecutionDefect / Route X

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
- Unrelated dirty: .cursor/rules/*, AGENTS.md — excluded from D-069 staging
