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
- Ending commit/tag: e8abef2 / D-069-mature-membrane-exchange-audit
- Primary: Route X — accepted des≈99.666 equals over_capacity_mass≈99.667; S0/capacity≈2.31
- Analytical exchange OK; do not change K_eq/k_exchange to mask overseed
- Unrelated dirty: .cursor/rules/*, AGENTS.md — excluded

## Last validation
- Command: cargo test -p chemistry-core --test d069_tests; D069_MAX_ACCEPTED=1200 SKIP_LATE pipeline
- Result: 17/17 PASS; Route X

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
