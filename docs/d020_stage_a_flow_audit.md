# D-020 Stage A — Flow Audit

Source: final valid rolling windows from both D-019 R22 attempts
(`experiments/generated/d019/stage_e_reference`, `stage_e_reference_kcorr`).
Balance metrics are governed end-state aggregates; last windows were `valid=true`,
`qualifying=false`, with no early-transient calibration.

## Baseline (`k_structure = 0.2576`, frozen companions)

| Flow | g | Q | Controlling rate |
| --- | --- | --- | --- |
| structure | −5.677 | 0.115 | `k_structure` |
| catalyst | −0.530 | 0.459 | `k_rep` |
| membrane | −0.599 | 0.513 | `k_membrane` |
| activated | −0.416 | 1.494 | `k_activation` |

Retention/localization: C=0.934 (ok), A=0.658 (**fail** <0.80), membrane loc=0.867 (**fail** <0.90).

Accounting: material relative residual ~3e−7; activation numerical correction = 0; rejected substeps = 0.

## kcorr (`k_structure = 2.236` only)

| Flow | g | Q |
| --- | --- | --- |
| structure | −5.029 | 0.216 |
| catalyst | −0.564 | 0.407 |
| membrane | −0.670 | 0.203 |
| activated | −0.446 | 1.059 |

Single-rate structure boost improves Q_structure/Q_activated slightly but worsens membrane
and does not recover joint quasi-steady. Companion rates must move together.

## Contamination

D-019 prebalance max constraint contamination ≈ 0.0016 ≪ 0.05. No constraint contamination
of Stage A windows.

## Joint Q-corrected seed (clamped 0.25×–4.00× analytical)

```text
k_structure   = 1.0305   (4.00× ceiling)
k_rep         = 0.02580  (2.18×)
k_membrane    = 1.1369   (1.95×)
k_activation  = 0.05264  (0.67×)
```

Turnovers frozen: `k_structure_decay=0.025`, `k_d008_catalyst_turnover=0.002`,
`k_d008_activated_decay=0.005`.
