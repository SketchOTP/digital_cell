# CURRENT.md

## Active directive
- ID: D-20260721-d068-precursor-demand-membrane-assembly-audit
- Project directive: D-068
- Goal: Audit precursor demand vs membrane assembly under frozen activation (shadow-only)
- Status: done
- Acceptance: met — `D068_MEMBRANE_DESORPTION_DOMINANT`
- Touched files: d068_analysis/tests, experiment-runner/d068, main.rs, lib.rs, docs/d068_*, experiments/generated/d068, .agent/*
- Next action: audit S→P desorption under frozen precursor; next_execution_started=false

## Repo facts needed now
- Primary: Route S `D068_MEMBRANE_DESORPTION_DOMINANT`
- D-067 reproduced (A≈0.355, χ_A≈0.117); activation branch closed
- Accepted exchange_net: des≈99.7 ≫ ads≈2.77; S ledger closes; fixed P does not arrest S
- Precursor accumulates (η_P→S≪1) but is not the S-loss cause
- No precursor/activation/membrane production change

## Last validation
- Command: cargo test -p chemistry-core --test d068_tests; D068_MAX_ACCEPTED=1200 D068_SKIP_LATE_GATES=1 pipeline
- Result: 11/11 PASS; primary MembraneDesorptionDominant / Route S

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
- Unrelated dirty: .cursor/rules/*, AGENTS.md — excluded from D-068 staging
