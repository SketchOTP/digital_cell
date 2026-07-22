# CURRENT.md

## Active directive
- ID: D-20260722-d071-capacity-bounded-precursor-demand-regulation
- Project directive: D-071
- Goal: Capacity-bounded precursor demand regulation under frozen D-070 exchange + SEED_CAPACITY_CONTRACT_V1
- Status: done
- Acceptance: met — `D071_FAIL` (Gate5 repair unreachable constitutively)
- Touched files: d071_analysis/tests, experiment-runner/d071, membrane/config regulation hooks, docs/d071_*, experiments/generated/d071, .agent/*
- Next action: diagnose mature-membrane damage refill under frozen exchange; next_execution_started=false

## Repo facts needed now
- Selected candidate: reduced constitutive m_P≈0.00132 (product-inhibition mid-K_I missed A≥0.80 while bounding P)
- Gate5: regulated~0.894, constitutive~0.897, k_p=0~0.898 — shared failure
- Production defaults unchanged (m_P=1, K_I=0)
- Unrelated dirty: .cursor/rules/*, AGENTS.md — excluded

## Last validation
- Command: cargo test -p chemistry-core --test d071_tests; D071_MAX_ACCEPTED=1200 pipeline
- Result: 10/10 PASS; primary D071_FAIL

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
- Mature-membrane 10% damage refill <95% under frozen exchange even constitutively
