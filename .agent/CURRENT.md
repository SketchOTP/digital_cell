# CURRENT.md

## Active directive
- ID: D-20260717-d026-stage-e-activated-resource-recovery
- Project directive: D-026
- Goal: Diagnose Stage E A-budget / quasi-steady failure under sealed v7; smallest causal correction
- Status: done — `D026_SURFACE_COVERAGE_MAINTENANCE_DEFICIT` (Stage E not recovered)
- Acceptance: Gates 0–4 identify mechanism before rate changes; one D026_* conclusion — met
- Touched files: d026_analysis.rs, d026_tests.rs, d026.rs, simulation.rs, d013/d025/main, docs/d026_*, .agent/*
- Next action: Follow-on may revisit frozen ads/turnover balance; do not start Stage F; no productive-rate sweep

## Repo facts needed now
- HEAD starts at c87b540 (D-025 result); Gate 0 parity PASS
- Gate 6: SURFACE_COVERAGE_MAINTENANCE_DEFICIT; θΓ decline precedes A_ret collapse
- Gate 8 infeasible: ads≪Γ turnover while delta_P≫ads; need ~31× ads boost vs ≤4× bound
- Frozen: k_ads, d_gamma, k_gamma_decay, betas
- Mimir: BLOCKED (Windows path mapping / project register)

## Last validation
- Command: cargo test -p chemistry-core --release --test d026_tests; --test d025_tests
- Result: 21/21 PASS; 15/15 PASS

## Open blockers
- Stage E not recovered under sealed surface params
- D-008 remains BLOCKED_NOT_RECOVERED
