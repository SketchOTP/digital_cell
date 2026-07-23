# CURRENT.md

## Active directive
- ID: D-20260723-d081-edge-membrane-reserve-causality-audit
- Project directive: D-081
- Goal: Edge-membrane reserve provenance and replenishment causality audit
- Status: done
- Acceptance: met — `D081_EDGE_MEMBRANE_PRODUCTION_METABOLICALLY_INFEASIBLE` (Gate 5)
- Touched files: d081_analysis/tests, experiment-runner/d081, docs/d081_*, experiments/generated/d081, .agent/*
- Next action: affordable A→L under frozen kinetics; next_execution_started=false

## Repo facts needed now
- D-080 preserved: f5dc5a5 / D-080-edge-network-requalification-fail
- D-080 Gate7: PROVISIONAL_PENDING_RESERVE_CAUSALITY_AUDIT (not upgraded)
- Seed: EDGE_MEMBRANE_SEED_CONTRACT_V1 / CAPACITY_VALID_FINITE_RESERVE (+25% over capacity)
- Gates 0–4 PASS; Gate5 A collapse under continuous produce
- Unrelated dirty: .cursor/rules/*, AGENTS.md — leave unstaged

## Last validation
- Command: cargo test -p chemistry-core --test d081_tests; cargo run --release d081 pipeline
- Result: d081 10/10; pipeline stopped Gate5

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED
- Membrane A→L metabolically unaffordable under frozen activated packet
