# CURRENT.md

## Active directive
- ID: D-20260722-d076-nonequilibrium-surface-state-cycle-review
- Project directive: D-076
- Goal: Observer/reduced-model architecture review of P⇄U + U+A→S+W + conservative S→U
- Status: done
- Acceptance: met — `D076_SURFACE_CYCLE_ENERGY_INFEASIBLE` (Route E)
- Touched files: d076_analysis/tests, experiment-runner/d076, docs/d076_*, experiments/generated/d076, .agent/*
- Next action: broader Phase 1 boundary-architecture review; next_execution_started=false

## Repo facts needed now
- D-075 seal: `983c01f` / `D-075-exposure-gated-membrane-audit`
- Record: `PASSIVE_EXCHANGE_KINETICALLY_VALID_METABOLICALLY_UNREACHABLE`
- Gate0: S→U never executed historically (D-034 used S→W)
- Algebra: θ_S≥0.95 at endogenous p needs r*≈21; Jacobian stable
- Energy: measured A_ret≈0.06 ⇒ surplus=0; maturation sink infeasible
- Unrelated dirty: .cursor/rules/*, AGENTS.md — leave unstaged
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; Production REQUIRES_REMEDIATION

## Last validation
- Command: cargo test d070–d076; D076 release pipeline
- Result: d076 10/10; d075 12/12; pipeline → Route E

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED
- No viable membrane exchange architecture inside measured metabolic budget
