# D-008 Stage C — Activated Internal Metabolism

## Conclusion

`D008_STAGE_C_METABOLISM_PASS`

Isolated zero-dimensional `membrane_metabolism_v1` chemistry satisfies the Stage C
gate: activation requires C, N, and F independently; catalyst reproduction
requires A; A declines without activation; C declines without reproduction;
waste is produced; C and A remain bounded without persistent clamping; and
stoichiometric / ledger residuals close. Structure and membrane stayed fixed.

## Provenance

- Source commit: `bdc2411e5fd0b8ff947835ce88bf5f02c5f1fb5e`
- Experiment-runner SHA-256: `0b27701b219e602662b840e6b6e3a40cdb9215c85c2c56203f860a72d5e471ff`
- Equation version: `membrane_metabolism_v1`
- Snapshot schema: `2`
- Field schema: `seven_field_v1`
- Candidate: `cand-b25c17cd6932-kphi1-ks0.030000-kr0.012000`
- Candidate hash: `b25c17cd6932db227d497dccee47075439b598e064187b1adf7ca6cc4c1a49ef`
- Configuration hash: `c8cd77a31ef81fdc6de9adc3bdcd3d20290cbfb5a3ec9bf03a44f800a1999e24`

## Stage C reference rates

Defaults used for the qualitative gate only; Stage E retains quantitative calibration:

- `k_d008_activation = 0.020`
- `k_d008_reproduction = 0.040`
- `k_d008_activated_decay = 0.005`
- `k_d008_catalyst_turnover = 0.002`
- `d008_a_max = 1.0`
- `d008_c_max = 1.0`

## Controls

Nine controls ran, all clean:

| Case | Result |
| --- | --- |
| bounded_reference | pass |
| missing_c | pass |
| missing_n | pass |
| missing_f | pass |
| missing_a_reproduction | pass (A=0, N/F>0, activation off, reproduction=0) |
| no_activation_decline | pass |
| no_reproduction_decline | pass |
| waste_positive | pass |
| stoichiometric_closure | pass (activation>0, reproduction>0, identities close) |

Aggregate: `run_count = 9`, `900` accepted substeps, simulated time `2.25`. Boundedness used non-tautological clamp residual gate (`|C/A clamp_correction| ≤ 1e-5`). All nine controls report structure/membrane hash invariance.

## Validation

- `cargo test -p chemistry-core --release --test d008_tests` — 45 passed (hardening tip).
- `cargo test -p experiment-runner --release d008::tests` — 8 passed.
- Governed Stage C run — `D008_STAGE_C_METABOLISM_PASS`.

## Artifact

- Runtime result: `digital-protocell/experiments/generated/d008/stage_c_metabolism/attempt_001/result.json`
- Result SHA-256: `a7e6a138eb9d21db1fbacd54810da4f296198ab06e1b844e6d3a20560696e427`
- Manifest: `digital-protocell/experiments/generated/d008/manifest.json`

Stage D may proceed. Stages E–G remain unstarted.
