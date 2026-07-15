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

## Operative status (D-012 correction)

Until the corrected long-horizon protocol and stoichiometric branch complete:

```text
D011_LONG_HORIZON_CONFIRMATION_INCOMPLETE
```

This does not erase the quick and supplementary evidence below. It records that the
mandatory 200,000-step protocol, three-window quasi-steady determination, up to
four correction rounds, five-candidate maximum, full corrected neighboring-radius
validation, and robust-overlap testing were not completed.

## Governed attempt

Primary corrected artifact: `experiments/generated/d011/attempt_017/result.json`  
Provisional quick conclusion (not definitive): **D011_TRANSPORT_COUPLED_BALANCE_NO_SOLUTION**  
`result_tag`: **D-011-transport-coupled-balance-fail-corrected**

`attempt_017` is a corrected four-rate quick protocol (`max_steps=5000`, `window=1000`) after narrowing D-011 sensitivity/solver to the authorized productive rates only: `k_structure`, `k_rep`, `k_membrane`, and `k_activation`.

The quick result remains evidence of failure to find balance under short horizons.
It is not definitive evidence that no balanced state exists.

Supplementary longer replay: `attempt_015` ran 50k-step constrained-radius reference replay before the four-rate solver correction. Its replay dynamics remain useful as horizon evidence; its seven-rate sensitivity/solver metadata is superseded by `attempt_017`.

Preservation Manifest: `experiments/generated/d012/preservation/manifest.json`  
Tag: `D-011-long-horizon-incomplete`

## D-012 stoichiometric supersession

Exact v1 audit (`docs/d012_v1_stoichiometric_audit.md`, tag `D-012-stoichiometric-audit`)
classified `membrane_metabolism_v1` as `NO_POSITIVE_CONSERVATION_VECTOR` with primary
finding `D012_NONCONSERVATIVE_V1_CONFIRMED`.

Operative D-011 status is therefore revised from incomplete confirmation to:

```text
D011_LONG_HORIZON_INCOMPLETE_SUPERSEDED_BY_INVALID_STOICHIOMETRY
```

Quick and 50k evidence remain historical. Exhaustive corrected 200k D-011 rate-domain
search is not executed for this invalid network.

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
