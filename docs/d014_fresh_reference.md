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

## D-015 follow-up

D-015 diagnosed D-014 R22 `UNBOUNDED_ACCUMULATION` as bulk diffusion / transport-to-peripheral-sink limitation (W ceiling at cell center idx 18335; reservoir empty). Clearance law classified CORRECT. W-only environmental sink expansion (`waste_sink_inner_radius=30`) authorized under Branch B; organism hashes unchanged. See `docs/d015_waste_accumulation_postmortem.md`.
