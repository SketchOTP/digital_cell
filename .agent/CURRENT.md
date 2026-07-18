# CURRENT.md

## Active directive
- ID: D-20260718-d031-invariant-domain-surface-exchange
- Project directive: D-031
- Goal: Replace explicit v8 exchange integration with invariant-domain BE + Strang turnover; resume D-030 Gate 7; classify numerical vs biological failure
- Status: in_progress
- Acceptance: One D031_* conclusion; Gates 0–13 per directive; α/β/K/k frozen; no Stage F until Stage E recovered
- Touched files: surface_density.rs, config.rs, d031_analysis/tests, experiment-runner/d031, experiments/generated/d031, docs, .agent/*
- Next action: Gate 0 reproduce CapacityExceeded; implement v2 integrator; unit gates then renewal

## Repo facts needed now
- D-030 result commit `921bd42`, tag `D-030-exchange-identification-fail`
- Identified: α≈0.167, β≈0.00334, k≈0.00334, K≈50
- Gate7: CapacityExceeded, accepted_in_window=0 → operative `D030_NUMERICAL_CAPACITY_INTEGRATION_FAILURE`
- Equation unchanged: `membrane_metabolism_v8_reversible_surface_exchange`
- Integrator schema only: `surface_exchange_integrator_v2_invariant_domain`
- Disk ~6.3 GiB free — keep artifacts compact
- Mimir task: ed5bca889bd54b1aab4e9131344adb4f version 1

## Last validation
- Command: (pending)
- Result: (pending)

## Open blockers
- None yet; Gate 0 classification required before biology judgment
