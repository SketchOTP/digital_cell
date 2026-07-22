# CURRENT.md

## Active directive
- ID: D-20260722-d074-cellwise-exchange-integration-parity-repair
- Project directive: D-074
- Goal: Cellwise discrete-integrator parity audit of mature-membrane exchange
- Status: done
- Acceptance: met — `D074_EXCHANGE_TIMESCALE_CLASSIFICATION_DEFECT` (Route T)
- Touched files: d074_analysis/tests, experiment-runner/d074, docs/d074_*, experiments/generated/d074, .agent/*
- Next action: replace membrane horizon gates with capacity-weighted Λ_i; next_execution_started=false

## Repo facts needed now
- Start: `de407ca` / `D-073-mature-membrane-equilibrium-audit`
- Route T: mean-τ overstated cellwise exposure (fraction_ge5=0 on damaged capacity at “5τ”)
- Runtime matches production discrete predictor; no kinetic repair
- Exchange-isolated p=0.38 recovery≈0.952; D-073 with diffusion≈0.941
- Unrelated dirty: .cursor/rules/*, AGENTS.md — exclude

## Last validation
- Command: cargo test d070–d074; D074 pipeline release
- Result: d074 12/12; d073 10/10; d072 11/11; d071 10/10; d070 13/13; pipeline → Route T

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
- Horizon gates still use mean-τ until a follow-on directive replaces them
