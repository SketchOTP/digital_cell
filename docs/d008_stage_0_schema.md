# D-008 Stage 0 — Schema and Legacy Compatibility

## Conclusion

`D008_STAGE_0_SCHEMA_PASS`

The fixed-buffer engine now allocates, validates, serializes, and atomically
swaps all seven `membrane_metabolism_v1` fields. No membrane transport or
productive D-008 chemistry is enabled.

## Provenance

- Source commit: `afa9b37a68a715a70c4ec31feb2725c6394afbb9`
- Experiment-runner SHA-256: `3718506ce5ff8ef4f3e58fa13d0966f5762ba595b299abf30b546506f4e3ea17`
- Equation version: `membrane_metabolism_v1`
- Snapshot schema: `2`
- Field schema: `seven_field_v1`
- Candidate: `cand-e214e56943f0-kphi1-ks0.030000-kr0.012000`
- Candidate hash: `e214e56943f0574e2239430e3391d8d9f4930cf12b0c692d68ec03d6a786a428`
- Configuration hash: `d6484c3144370bcfb7ffeaf446e70272bc0d54927994b71492342fe067e349ba`
- Selected configuration: code-defined Stage 0 scaffold using `SimParams::default`
  with only `equation_version = membrane_metabolism_v1`

## Gate evidence

- All seven current and next buffers allocate independently.
- Accepted D-008 steps swap every field pair; rejected attempts swap none.
- Seven-field JSON snapshots restore all seven distinct field values.
- Historical flat five-field snapshots restore all five legacy field values.
- Five-field payloads are rejected for `membrane_metabolism_v1`, including
  in-memory payloads that bypass JSON parsing.
- Malformed field lengths and unknown schema versions return errors.
- Historical candidate and configuration hashes remain unchanged.
- A fixed SHA-256 digest reproduces the legacy ten-step numerical state.
- Stage 0 D-008 dispatch produces no reactions or membrane transport.

## Validation

- `cargo test -p chemistry-core --release --test d008_tests` — 17 passed.
- `cargo test -p chemistry-core --release --test d007_tests --test d006_tests --test d005_tests --test d004_tests --test d003_tests` — 131 passed.
- `cargo test -p chemistry-core --release --test integration_tests --test validation_tests` — 39 passed.
- `cargo check -p experiment-runner --release` — passed.
- IDE diagnostics — no errors.

The full fresh release validation completed in 661 seconds. Existing compiler
warnings remain; no warning-suppression or unrelated formatting changes were
made.

## Artifact

- Runtime artifact: `digital-protocell/experiments/generated/d008/stage_0_schema/attempt_001/result.json`
- Artifact SHA-256: `1ead10ee1b1ee91b84c31368d0eba27de3074217d31f275254a1aea9a046c887`
- Manifest: `digital-protocell/experiments/generated/d008/manifest.json`
- Manifest SHA-256: `ee9e023e61b40d5eec7ea54053d2ac8b50232f6c1adb183c6933269764c2cb6b`

Stage A may proceed. Stages B–G remain unstarted.
