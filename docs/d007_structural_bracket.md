# D-007 Structural Bracket

## Protocol

- Factors relative to D-006 derived interface rate `0.09642857142857159`:
  `0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80`
- Frozen `k_rep = 0.014489097664708522`
- `R0 ∈ {16,24,32}`, `C0 = 0.35`, seeds `{1,2,3}`, `30_000` accepted substeps
- Total: **63 / 63** clean strict-schema runs

Artifact: `experiments/generated/d007/structural_bracket/aggregate.json`

## Median `v_R` by factor

| Factor | med v_R (R16) | med v_R (R24) | med v_R (R32) | Class |
| --- | ---: | ---: | ---: | --- |
| 0.50× | −0.06940 | −0.01989 | +0.00408 | DISORDERED |
| 0.55× | −0.05265 | −0.01149 | +0.01145 | DISORDERED |
| 0.60× | −0.03182 | −0.00310 | +0.01670 | DISORDERED |
| 0.65× | −0.03182 | +0.01642 | +0.02331 | DISORDERED |
| 0.70× | −0.02353 | +0.02476 | +0.03170 | DISORDERED |
| 0.75× | −0.00287 | +0.03032 | +0.04398 | DISORDERED |
| 0.80× | +0.01190 | +0.04141 | +0.05474 | ALL_GROW |

## Provisional structural pass rule

Requires:

```text
median v_R (R16) > 0
median v_R (R32) < 0
near-monotonic decrease with R0
```

**Zero** factors passed. Retained structural factors: none.

## Pattern

Across the bounded domain the radius response is opposite-of-restoring:

- smaller seeds decline (or least-grow)
- larger seeds grow (or least-decline)

Raising the interface rate toward the D-006 Stage D floor (`0.80×`) recovers the all-growth regime already documented under D-006C.

## Structural gate

```text
D007_NO_STRUCTURAL_NULLCLINE
```

Per D-007 §10: do **not** tune catalyst reproduction after this failure.
