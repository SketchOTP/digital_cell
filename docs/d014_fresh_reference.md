# D-014 Fresh Reference (R22)

Frozen candidate from analytic IC; repaired numerical method v2.

| Field | Value |
| --- | --- |
| Termination | `UNBOUNDED_ACCUMULATION` |
| Scientific classification | `UNBOUNDED_ACCUMULATION` |
| Accepted substeps | 161157 |
| Attempted / rejected | 161158 / 1 |
| Simulated time | ≈ 402.8925 |
| Final dt | 0.0025 (no floor) |
| Min attempted dt | 0.0025 |
| Checkpoints | 10k, 25k, 50k, 100k, 150k |
| Candidate hash | matches frozen |
| Activation relative residual | ≈ 5.43×10⁻¹³ |
| Clean termination | true |

No `TIMESTEP_FLOOR_FAILURE`. Directional Q/g/retention align with D-013 historical diagnostics.

Solver entry gate remains **CLOSED** (not quasi-steady).

Artifact: `experiments/generated/d014/fresh_reference_r22/result.json`
