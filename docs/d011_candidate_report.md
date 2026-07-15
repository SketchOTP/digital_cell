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

## Governed attempt

Primary corrected artifact: `experiments/generated/d011/attempt_017/result.json`  
`scientific_conclusion`: **D011_TRANSPORT_COUPLED_BALANCE_NO_SOLUTION**  
`result_tag`: **D-011-transport-coupled-balance-fail**

`attempt_017` is a corrected four-rate quick protocol (`max_steps=5000`, `window=1000`) after narrowing D-011 sensitivity/solver to the authorized productive rates only: `k_structure`, `k_rep`, `k_membrane`, and `k_activation`.

Supplementary longer replay: `attempt_015` ran 50k-step constrained-radius reference replay before the four-rate solver correction. Its replay dynamics remain useful as horizon evidence; its seven-rate sensitivity/solver metadata is superseded by `attempt_017`.

## Replay grid (seed=1, max_steps=50000, window=1000)

| R | Classification | joint_overlap | g_structure | g_catalyst | g_membrane | g_activated |
| --- | --- | --- | --- | --- | --- | --- |
| 14 | NOT_CONVERGED | false | −12.35 | 0.61 | −1.43 | −4.47 |
| 18 | NOT_CONVERGED | false | −21.33 | 1.12 | −1.82 | −6.52 |
| 22 | NOT_CONVERGED | false | −32.77 | 1.79 | −2.21 | −8.92 |
| 26 | NOT_CONVERGED | false | −46.86 | 2.62 | −2.60 | −11.47 |
| 30 | NOT_CONVERGED | false | −63.47 | 3.61 | −2.99 | −14.31 |
| 34 | NOT_CONVERGED | false | −82.40 | 4.74 | −3.39 | −17.62 |

All radii: quasi-steady not met; |g| ≫ 1e−4; Q outside [0.98, 1.02] for structure/membrane/activated.

## 50k-step spot check (attempt_006, window=10000)

| R | g_structure | Q_structure | Classification |
| --- | --- | --- | --- |
| 14 | −14.85 | 0.051 | NOT_CONVERGED |
| 18 | −24.66 | 0.041 | NOT_CONVERGED |

Imbalance persists at longer horizon; structure virtual-flow deficit dominates.

## Sensitivity (R=22, R=26)

Four-rate sensitivity at R=22 is full rank (rank=4, condition number ≈9.04). The bounded solver proposed one in-domain correction, but no validation radius reached joint overlap in the corrected quick protocol.

## Stage E revision

`stage_e_revised_to_pass_after_d011`: **false**  
Prior conclusion `D008_NO_JOINT_FIXED_POINT` stands.
