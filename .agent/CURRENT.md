# CURRENT.md

## Active directive
- ID: D-20260722-d077-cooperative-surface-condensation-review
- Project directive: D-077
- Goal: Observer/reduced-model review of cooperative P↔S surface condensation (χ cohesion)
- Status: done
- Acceptance: met — `D077_COOPERATIVE_COHESION_NOT_PORTABLE` (Route P)
- Touched files: d077_analysis/tests, experiment-runner/d077, docs/d077_*, experiments/generated/d077, .agent/*
- Next action: formal Phase 1 boundary-substrate redesign decision; next_execution_started=false

## Repo facts needed now
- Start: `d82628f` / `D-076-nonequilibrium-surface-cycle-review`
- Record: `ENERGY_DRIVEN_SURFACE_STATE_CYCLE_REJECTED`
- Gate2: χ span≈2.35× OK; LOO fail (constitutive ~0.73 vs reduced ~1.62)
- Selected diagnostic χ≈1.615; Gate3 secondary A_ret fail
- Unrelated dirty: .cursor/rules/*, AGENTS.md — leave unstaged
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; Production REQUIRES_REMEDIATION

## Last validation
- Command: cargo test -p chemistry-core --test d077_tests; D077 release pipeline
- Result: d077 12/12; pipeline → Route P

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED
- No portable metabolically affordable membrane law inside current P/S architecture
