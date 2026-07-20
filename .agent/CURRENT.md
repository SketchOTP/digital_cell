# CURRENT.md

## Active directive
- ID: D-20260720-d047-shared-activated-resource-pool-sufficiency
- Project directive: D-047
- Goal: Shared activated-resource pool sufficiency under fixed biochemistry
- Status: done
- Acceptance: met — `D047_HISTORICAL_ACTIVATION_FIXED_BIOLOGY_QUALIFIED`; Route H; no chemistry/C_star/activation implementation
- Touched files: d047_analysis.rs, d047_tests.rs, d047.rs, lib.rs, main.rs, docs/d047_*, experiments/generated/d047, .agent/*
- Next action: membrane-basin validation under frozen historical activation; next_execution_started=false

## Repo facts needed now
- Gate0: `D047_CROSS_PARAMETER_PORTABILITY_DEFECT` — Models A/B/C fail only with altered k_P/k_φ; fixed biology Model C max≈3%
- Shared pool capable under local sufficient A; precursor not product-regulated but does not destroy healthy fixed point
- Route H: no activation change authorized

## Last validation
- Command: cargo test d047_tests + d047_analysis; d047 pipeline Gates0-10
- Result: primary=D047_HISTORICAL_ACTIVATION_FIXED_BIOLOGY_QUALIFIED
- Commits: 10cb5b3, 16effd5, 5453ee9
- Tag: D-047-shared-activated-resource-audit

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
