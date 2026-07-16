# D-014 Activation Accounting Audit

## Prior residual (D-013 R22)

| Metric | Value |
| --- | --- |
| Absolute residual | ≈ 410.81 |
| Relative residual | ≈ 1.754×10⁻² |

Root cause of the large residual: ledger budget mixed diagnostic reaction extents with
step residuals incorrectly in earlier harness iterations; D-014 closes each accepted step
against the full F/A partition.

## Step identity (v2, e_F = e_A = 1)

```text
ΔP_obs = P_after − P_before
ΔP_pred = reservoir_potential + chemistry_potential + transport_potential + numerical_correction
residual = ΔP_obs − ΔP_pred
```

Where:

- `fuel_import` = max(fuel.reservoir_delta, 0) (diagnostic)
- `reservoir_potential` = e_F·ΔF_res + e_A·ΔA_res
- `chemistry_potential` = e_F·ΔF_rxn + e_A·ΔA_rxn
- `transport_potential` = e_F·ΔF_diff + e_A·ΔA_diff

Internal face transport cancels globally under equal weights. F→A transfer is
potential-neutral and is **not** counted as creation. Attempted (rejected) steps do not
enter the ledger.

## Target

Controlled tests: relative residual ≤ 1×10⁻⁶ (`activation_step_closes`).
