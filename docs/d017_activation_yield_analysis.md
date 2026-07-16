# D-017 Candidate A — conservative activation-yield analysis

## Family (comparison-only; not runtime-authorized)

```text
N + F → (1+α) A + (1−α) W    ,  0 ≤ α ≤ 1
```

Material residual is exactly 0 for all tested α.

## Activation-potential gate

Frozen weights `E_F=E_A=1`:

- α=0: residual 0 → valid
- α>0: residual = α → **A_POTENTIAL_INVALID** (creates usable potential)

Revised partition `E_A(α)=E_F/(1+α)`:

- residual 0 for all α → **A_POTENTIAL_VALID**
- increased A yield requires **lower per-unit A potential**

## Fixed-extent counterfactual (`A_FIXED_EXTENT_COUNTERFACTUAL`)

| α | direct W | total W | center W (W_i=2) |
| ---: | ---: | ---: | ---: |
| 0.25 | 1.559 | 41.112 | 14.528 |
| 0.50 | 1.039 | 40.592 | 14.370 |
| 0.75 | 0.520 | 40.073 | 14.212 |
| 1.00 | 0.000 | 39.553 | 14.053 |

Ceiling-compatible source for center W<9 ≈ **22.97**. Even α=1 lower bound (39.55) remains far above.

## Transport class

All α: **A_CANNOT_AVOID_CEILING**

Artifact: `digital-protocell/experiments/generated/d017/activation_yield/`
