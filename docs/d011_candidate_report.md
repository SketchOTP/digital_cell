# D-011 Candidate Report

## Reference candidate

Exact Stage E failed rates from `attempt_003/result.json` (preserved, not rounded):

```text
k_membrane              = 0.23697878259991778
k_d008_activation       = 0.024
k_d008_reproduction     = 0.032
k_d008_structure        = 0.6788558775098147
k_d008_activated_decay  = 0.005
k_d008_catalyst_turnover= 0.002
k_structure_decay       = 0.025
```

## Mode

`d008_stage_mode = constrained_radius`  
`equation_version = membrane_metabolism_v1`

## Replay grid

R ∈ {14, 18, 22, 26, 30, 34}, seed = 1.

## Metrics per radius

Each `result.json` includes:

- `Q_structure`, `Q_catalyst`, `Q_membrane`, `Q_activated`
- `g_structure`, `g_catalyst`, `g_membrane`, `g_activated`
- Retention, membrane localization, N/F influx, W efflux
- `field_accounting`, `constraint_ledger`, `convergence_classification`
- `source_commit`, `binary_sha256`, candidate hashes

## Report status

See latest `experiments/generated/d011/attempt_*/result.json` for governed metrics
and `scientific_conclusion`. This document template is filled from that artifact after
each governed run.
