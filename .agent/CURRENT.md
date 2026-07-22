# CURRENT.md

## Active directive
- ID: D-20260722-d076-nonequilibrium-surface-state-cycle-review
- Project directive: D-076
- Goal: Observer/reduced-model architecture review of P⇄U + U+A→S+W + conservative S→U cycle
- Status: started
- Acceptance: One exact D076_* route with Gates 0–6; D-075 sealed+tagged; no production chemistry change
- Touched files: d076_analysis/tests, experiment-runner/d076, docs/d076_*, experiments/generated/d076, .agent/*
- Next action: Seal D-075 commit+tag; then Gates 0–6 reduced-model review

## Repo facts needed now
- D-075: `D075_FROZEN_EXCHANGE_METABOLICALLY_UNREACHABLE` (Route M); endogenous p≈0.19 → θ_eq≈0.90; A_ret≈0.06
- Record: `PASSIVE_EXCHANGE_KINETICALLY_VALID_METABOLICALLY_UNREACHABLE`
- Candidate: P⇄U (frozen D-030), U+A→S+W, S→U conservative (not previously executed; D-034 used S→W)
- Unrelated dirty: .cursor/rules/*, AGENTS.md — leave unstaged
- Stage E: BLOCKED_NOT_RECOVERED; Phase1: PARTIAL; Production: REQUIRES_REMEDIATION

## Last validation
- Command: cargo test -p chemistry-core --test d075_tests
- Result: 12/12 PASS (D-075 reproduce before seal)

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED
- Mimir task 0cee98668c3a47c7a38149e27a623d84 v2; retrieval session 8e99243cb1c14de5bd9ffb97e7a58173
