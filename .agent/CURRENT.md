# CURRENT.md

## Active directive
- ID: D-20260720-d050-coupled-catalyst-saturating-activation-repair
- Project directive: D-050
- Goal: Catalyst-saturating volume activation repair for coupled organism Stage E
- Status: done
- Acceptance: met — primary `D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED` at Gate 5
- Touched files: activated_metabolism, config V13, d050_analysis/tests, d050.rs, main.rs, docs/d050_*, experiments/generated/d050, .agent/*
- Next action: coupled activation-topology review; next_execution_started=false

## Repo facts needed now
- Schema2 V13 implemented; schema1 historical k=0.020 preserved
- Gate0–4 PASS; Gate5 FAIL — A retention ~0.03 flat across V_A 0.75×–4× fitted
- Fitted V_A≈0.1254, K_C=0.10 (Model C recon hold max~2.8%)
- Record: COUPLED_HISTORICAL_ACTIVATION_CAPACITY_REJECTED

## Last validation
- Command: cargo test -p chemistry-core --test d050_tests --release (21/21); Gate5 screen D050_MAX_ACCEPTED=5000
- Result: primary=D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED; Phase1 PARTIAL; production REQUIRES_REMEDIATION
- Do not add C_star/buffer/product inhibition without topology review
