# D-020 Stage B — Bounded Preconditioner

## Sensitivity (±10% at analytical v3 rates, short R22)

| Property | Value |
| --- | --- |
| Rank | 4 (full) |
| Condition number | ≈ 2.71 |
| Rank-deficient? | No |

Jacobian couples structure/membrane/activation strongly; catalyst column is well-conditioned.

## Newton rounds (frozen Jacobian, live g remeasure)

Starting from analytical rates, four correction rounds reduced ‖g‖ from ≈6.61 → ≈3.41
with Q vector approaching 1:

```text
round 3 rates:
  k_structure   = 0.5417
  k_rep         = 0.0103
  k_membrane    = 1.021
  k_activation  = 0.2655
short-run Q ≈ [0.87, 0.91, 0.92, 1.10]
```

All candidates remained inside 0.25×–4.00× analytical bounds; turnovers frozen.

## Hard gates

Every short-run candidate failed **membrane localization** (~0.88 < 0.90).
Retention C/A, contamination, accounting, extinction, and ceiling gates passed.

## Candidates tested (6 / max 6)

Ranked by ‖g‖: `newton_round_3`, `newton_round_2`, `newton_round_1`,
`newton_round_0`, `analytical_baseline`, `q_corrected_seed`.

Long-run Q-corrected seed scored worst on short-run ‖g‖ (overshoots activation).
