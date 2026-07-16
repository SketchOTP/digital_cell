# D-014 dt Refinement / Non-stiff Equivalence

Comparison mode: **equal simulated time** `t = 25000 × MAX_DT = 62.5` at
`dt_cap ∈ {MAX_DT, 0.5×MAX_DT, 0.25×MAX_DT}`.

| dt_cap | accepted steps | simulated time |
| --- | --- | --- |
| 0.0025 | 25001 | ≈ 62.5025 |
| 0.00125 | 50001 | ≈ 62.50125 |
| 0.000625 | 100001 | ≈ 62.500625 |

Max relative mass error vs reference dt_cap: **≈ 2.36×10⁻⁵** (largest on W).
No timestep-floor or concentration-bound abort in this non-stiff window.

Artifacts: `experiments/generated/d014/nonstiff_equivalence/result.json`,
`experiments/generated/d014/dt_refinement/result.json`.
