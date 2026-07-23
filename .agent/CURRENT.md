# CURRENT.md

## Active directive
- ID: D-20260722-d079-conserved-edge-network-membrane-feasibility
- Project directive: D-079
- Goal: Conserved edge-network membrane feasibility (L/B face substrate)
- Status: done
- Acceptance: met — `D079_EDGE_NETWORK_SELF_ASSEMBLY_FAILURE` (Gate 2)
- Touched files: edge_membrane, d079_analysis/tests, experiment-runner/d079, docs/d079_*, experiments/generated/d079, .agent/*
- Next action: operator decision on edge-kinetics revise vs discrete reject vs particle phase; next_execution_started=false

## Repo facts needed now
- Start: `039044f` / `D-078-boundary-substrate-downselect`
- Scope: `PHASE1_EDGE_NETWORK_BOUNDARY_RESEARCH_AUTHORIZED`
- Gate2: coverage R16/22/32 ≈0.85/0.89/0.92; closed=false; off-interface=0
- Unrelated dirty: .cursor/rules/*, AGENTS.md — leave unstaged
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; Production REQUIRES_REMEDIATION

## Last validation
- Command: cargo test -p chemistry-core --test d079_tests; D079 release pipeline
- Result: d079 12/12; stopped Gate2 self-assembly failure

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED
- Edge-network not qualified; no closed ≥0.95 assembly without prescribing a ring
