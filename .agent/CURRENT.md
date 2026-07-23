# CURRENT.md

## Active directive
- ID: D-20260722-d078-phase1-boundary-substrate-redesign
- Project directive: D-078
- Goal: Phase 1 continuum boundary substrate downselect (structure-native φ vs single amphiphile M)
- Status: done
- Acceptance: met — `D078_CONTINUUM_BOUNDARY_SUBSTRATE_REJECTED` (Route N)
- Touched files: d078_analysis/tests, experiment-runner/d078, docs/d078_*, experiments/generated/d078, .agent/*
- Next action: operator Phase 1 scope decision; next_execution_started=false

## Repo facts needed now
- Start: `5026f9f` / `D-077-cooperative-surface-condensation-review`
- Record: `CURRENT_P_S_BOUNDARY_ARCHITECTURE_CLOSED`
- Both candidates novel (Gate0) and conservation-OK (Gate1); fail Gates 2–4; A fails Gate5 molecular replacement
- Unrelated dirty: .cursor/rules/*, AGENTS.md — leave unstaged
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; Production REQUIRES_REMEDIATION

## Last validation
- Command: cargo test -p chemistry-core --test d078_tests; D078 release pipeline
- Result: d078 12/12; pipeline → Route N

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED
- Continuum boundary substrate lineage closed pending Phase 1 scope revision
