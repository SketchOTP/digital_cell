# D-020 — V3 Joint-Rate Stage E Recovery

## Conclusion

`D020_REFERENCE_REMAINS_NONCONVERGENT`

The selected D-019 v3 chemistry received a bounded four-rate joint recovery attempt.
The local system was full-rank, and bounded Newton candidates improved short-run joint
flow, but the promoted R22 candidate remained `NOT_CONVERGED_AT200K`.

`D-008 Stage E` remains `BLOCKED_NOT_RECOVERED`; Stage F did not start.

## Stage A — Flow audit

Final valid R22 windows from both D-019 attempts were used; no early transient windows
were used for calibration.

### Baseline R22 (`k_structure=0.2576268689`)

```text
g_structure = -5.6771102428
 g_catalyst = -0.5297628167
 g_membrane = -0.5988219964
g_activated = -0.4163332810

Q_structure = 0.1152380161
 Q_catalyst = 0.4586569991
 Q_membrane = 0.5130047243
Q_activated = 1.4944310611
```

Controls: C retention 0.9337; A retention 0.6584 (fail); membrane localization 0.8670 (fail).
Accounting closed (material relative residual ~3e-7; activation correction 0). Constraint contamination was below gate.

### kcorr R22 (`k_structure=2.2356065966`, companion rates frozen)

```text
g_structure = -5.0285230028
 g_catalyst = -0.5643308184
 g_membrane = -0.6704281414
g_activated = -0.4460386768

Q_structure = 0.2163185498
 Q_catalyst = 0.4072913800
 Q_membrane = 0.2033290034
Q_activated = 1.0593400222
```

Single-rate correction did not recover joint balance.

## Stage B — Bounded preconditioner

- Productive rates only: `k_structure`, `k_rep`, `k_membrane`, `k_activation`.
- Frozen: topology, turnover, transport, yields, fields, environment.
- Candidate limit: 6.
- Correction rounds: 4.
- Global bounds: 0.25×–4.00× analytical v3 rates.
- Per-round bounds: 0.67×–1.50×.

Sensitivity matrix rank was 4; condition number ≈ 2.71. Not rank-deficient.

Top short-run candidate: `newton_round_3`.

```text
k_structure  = 0.4955475743
k_rep        = 0.0102931657
k_membrane   = 0.9527722790
k_activation = 0.3146636440
```

Short-run candidates improved ‖g‖, but membrane localization stayed around 0.88 < 0.90.

## Stage C — Candidate promotion

20k promotion gates rejected all six strict candidates. To complete the governed D-020 recovery attempt,
the best Stage-B ‖g‖ candidate (`newton_round_3`) was promoted for full R22.

## Stage D — Full R22

`newton_round_3` ran to 200,000 accepted substeps with 0 rejected substeps.

```text
classification = NOT_CONVERGED_AT200K
termination    = MAX_ACCEPTED_SUBSTEPS_REACHED
```

Final balance:

```text
Q_structure = 0.2134093933
 Q_catalyst = 0.4232382908
 Q_membrane = 0.5912677066
Q_activated = 1.4248516912

g_structure = -5.0471897203
 g_catalyst = -0.5607823095
 g_membrane = -0.5708759241
g_activated = -0.3571517334
```

Retention/localization:

```text
C retention           = 0.9358864580
A retention           = 0.3770473373
membrane localization = 0.8585595579
```

Accounting:

```text
material relative residual     = 2.8708619400e-7
activation numerical correction = 0.0
```

## Stage E — Restoring-radius confirmation

Skipped. No converged R22 candidate existed, so R18/R26 confirmation was not valid.

## Artifacts

- `digital-protocell/experiments/generated/d020/stage_a_flow_audit/flow_audit.json`
- `digital-protocell/experiments/generated/d020/stage_b_sensitivity/sensitivity.json`
- `digital-protocell/experiments/generated/d020/stage_c_promotion/promotion.json`
- `digital-protocell/experiments/generated/d020/stage_d_full_r22/full_r22.json`
- `digital-protocell/experiments/generated/d020/stage_d_full_r22/candidate_0/result.json`
- `digital-protocell/experiments/generated/d020/manifest.json`

## Next architecture directive

Sensitivity was full-rank, so this is not a rank-deficient calibration problem. The full R22 run returns to the
same long-run deficits: structure/catalyst/membrane underproduction, activated imbalance, A retention collapse,
and membrane localization below gate. Stop parameter calibration; next directive should address the coupled
spatial retention/localization mechanism rather than add another bounded rate screen.
