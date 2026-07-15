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

## Report status (governed run `attempt_005`)

- `max_steps`: 5000, `window_size`: 1000 (first completed full protocol; 50k runs interrupted by wall-clock)
- `scientific_conclusion`: `D011_TRANSPORT_COUPLED_NO_SOLUTION`
- `any_joint_overlap_pass`: false
- `stage_e_revised_to_pass_after_d011`: false — Stage E remains `D008_NO_JOINT_FIXED_POINT`

### Key replay metrics (failed Stage E rates)

| R | Q_structure | g_structure | Q_catalyst | g_catalyst | joint_overlap |
| --- | --- | --- | --- | --- | --- |
| 14 | 0.21 | −12.35 | 2.21 | 0.61 | false |
| 22 | 0.14 | −32.77 | 2.43 | 1.79 | false |
| 26 | 0.09 | −63.47 | 2.12 | 1.42 | false |
| 34 | 0.05 | −82.40 | 2.08 | 1.38 | false |

All radii: `NOT_CONVERGED`, retention ≥ 0.99, localization ≥ 0.89, N/F influx > 0, W efflux > 0.

Artifact: `experiments/generated/d011/attempt_005/result.json` (gitignored generated output).
