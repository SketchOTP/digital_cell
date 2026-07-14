# D-008 Stage A — Static Selective Membrane Transport

## Conclusion

`D008_STAGE_A_TRANSPORT_PASS`

The approved fixed membrane selectively attenuates soluble transport without
changing structure, membrane, or total soluble mass. Membrane synthesis,
membrane dynamics, and productive metabolism remained disabled.

## Provenance

- Source commit: `9236be1bf1208bcc7b709c93b8855dd48e18c55e`
- Experiment-runner SHA-256: `4bca2cb6b8f958b15a5e714c686ae0b95ca6b8e712f99cd44f1b4f125106eb49`
- Equation version: `membrane_metabolism_v1`
- Snapshot schema: `2`
- Field schema: `seven_field_v1`
- Candidate: `cand-9b24255c7f30-kphi1-ks0.030000-kr0.012000`
- Candidate hash: `9b24255c7f30cf3b2f9bd71a8acb0e056bec64a9674e3a60e828f47b08bed2c0`
- Configuration hash: `b34e88f08ce5d038567acb66935a458f41d68c31fc517841f7b9d0a12fe1dac1`

## Planar transport result

Normalized permeability at `M = 1`, `I = 1`:

- Catalyst: `0.010051835744633641` — pass (`≤ 0.05`)
- Activated resource: `0.010051835744633574` — pass (`≤ 0.05`)
- Nutrient: `0.30119421191220197` — pass (`0.20–0.50`)
- Fuel: `0.30119421191220197` — pass (`0.20–0.50`)
- Waste: `0.8187307530779822` — pass (`≥ 0.70`)

For every species, measured flux decreased strictly across membrane densities
`0`, `0.25`, `0.50`, `0.75`, and `1.00`. Zero membrane reproduced the base
diffusivity. Each face contribution was equal and opposite, and every
reservoir-free case recorded zero net mass-change rate.

## Validation

- `cargo test -p chemistry-core --release --test d008_tests` — 29 passed.
- `cargo test -p chemistry-core --release --test d007_tests --test d006_tests --test d005_tests --test d004_tests --test d003_tests` — 131 passed.
- `cargo test -p experiment-runner --release -- stage_a_` — 3 passed.
- `cargo check -p experiment-runner --release` — passed.
- IDE diagnostics — no errors.

Existing compiler warnings remain; no unrelated warning cleanup was performed.

## Artifact

- Runtime result: `digital-protocell/experiments/generated/d008/stage_a_transport/attempt_001/result.json`
- Result SHA-256: `03e2b13e86658b1b7f505bdcc7ca4681f73688aa44b4d064011df55278853698`
- Manifest: `digital-protocell/experiments/generated/d008/manifest.json`
- Manifest SHA-256: `17a8bbd6ea0c1e1f79d8cbe1c28707460d39fb7af026ea41aeec4bb2cceea5f0`

Stage B may proceed. Stages C–G remain unstarted.
