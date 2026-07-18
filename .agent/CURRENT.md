# CURRENT.md

## Active directive
- ID: D-20260717-d029-reversible-surface-exchange
- Project directive: D-029
- Goal: Conservative reversible P↔S exchange (v8); recover Stage E without controller
- Status: done — `D029_REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE`
- Acceptance: One D029_* conclusion; stop at first failed gate — met (Gate 2)
- Touched files: chemistry-core v8 exchange, d029_analysis/tests/runner, experiments/generated/d029, docs/d029_*, .agent/*
- Next action: Architect next exchange/assembly architecture; do not Stage F; do not productive-rate-only repair

## Repo facts needed now
- Gate0 PASS; Gate1 PASS (d029_tests 11/11); Gate2 FAIL β→0 NNLS
- α≈0.0329 β=0; rank 2; cond≈1.38e3; median rel err≈16%; max≈35.4%
- Record: IRREVERSIBLE_ADSORPTION_LAW_REJECTED
- Mimir: BLOCKED (Windows path mapping on project_register/resolve)

## Last validation
- Command: cargo test -p chemistry-core --release --test d029_tests; experiment-runner d029 gate2
- Result: 11/11 PASS; Gate2 D029_REVERSIBLE_EXCHANGE_NOT_IDENTIFIABLE

## Open blockers
- Two-parameter reversible exchange not identifiable on six governed states
- D-008 remains BLOCKED_NOT_RECOVERED
