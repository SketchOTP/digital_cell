# CURRENT.md

## Active directive
- ID: D-20260722-0832-d070-mature-membrane-seed-capacity-contract-repair
- Project directive: D-070
- Goal: Repair mature-membrane seed/capacity contract; revalidate frozen exchange with capacity-valid seeds
- Status: done
- Acceptance: met — `D070_SEED_REPAIR_QUALIFIES_EXCHANGE_PRECURSOR_LIMIT_REMAINS`
- Touched files: d070_analysis/tests, experiment-runner/d070, experiments/generated/d070, docs/d070_*, .agent/*
- Next action: precursor-demand regulation under frozen exchange + capacity contract; next_execution_started=false

## Repo facts needed now
- Route P: capacity-valid Seed B maintains S (~0.99 occ); A retention ~0.35; P accumulates
- Contract: SEED_CAPACITY_CONTRACT_V1; Policy D selected for unauthorized historical excess
- Kinetics unchanged (K_eq, k_exchange, Γ_max frozen)
- Unrelated dirty: .cursor/rules/*, AGENTS.md — excluded

## Last validation
- Command: cargo test -p chemistry-core --test d070_tests; D070_MAX_ACCEPTED=1200 SKIP_LATE pipeline
- Result: 13/13 PASS; Route P

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
- Precursor/A demand remains the post-seed limiter
