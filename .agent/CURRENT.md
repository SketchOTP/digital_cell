# CURRENT.md

## Active directive
- ID: D-20260718-d031-invariant-domain-surface-exchange
- Project directive: D-031
- Goal: Invariant-domain v8 exchange integration; resume D-030 Gate7; recover Stage E if portable
- Status: partial — Gate0 OVERSHOOT_CONFIRMED; Gate3 PASS; Gate4 short diag accepted>0 Q=1.45; full Gate4 running (pid in /tmp/d031_gate4_full.log)
- Acceptance: One D031_* after Gates 0–13 — not met (Gates 5–13 not run; Gate4 horizons incomplete)
- Touched files: surface_density.rs, config.rs, d031_*, experiment-runner/d031, docs/d031_*, .agent/*
- Next action: Await Gate4 full horizons; if Q enters band → Gate5 portability; else conclude TURNOVER_EXCHANGE_INCOMPATIBILITY_CONFIRMED

## Repo facts needed now
- Commit: `3b3d033` — D-031 invariant integrator
- Integrator: `surface_exchange_integrator_v2_invariant_domain`
- Frozen: α≈0.167, β≈0.00334, K=50, k≈0.00334
- D-030 tag preserved: `D-030-exchange-identification-fail`
- Mimir task: ed5bca889bd54b1aab4e9131344adb4f version 2

## Last validation
- Command: cargo test d031/d029/d030; d031 gate0; gate3; gate4-diag
- Result: unit PASS; Gate0 OVERSHOOT_CONFIRMED; Gate3 PASS; Gate4 diag accepted=6020 capacity_reject=false Q=1.45

## Open blockers
- Gate4 full renewal windows not yet three consecutive Q∈[0.98,1.02]
- Gates 5–13 not started
- Disk ~6.3 GiB free
