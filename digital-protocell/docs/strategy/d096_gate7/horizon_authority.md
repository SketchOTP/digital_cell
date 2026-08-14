# Horizon authority

The horizon was frozen before results were generated:

- authority: D-088 reproduction-qualified non-smoke physical-fission campaign;
- source: `crates/chemistry-core/src/d088_analysis.rs`, `steps(12_000)`;
- non-smoke mapping: `12_000 / 3 = 4,000` executed steps;
- mechanics: `MechParams::default().dt = 0.02`;
- nominal maximum accepted simulated time: `4,000 * 0.02 = 80.0`.

The runtime accepted the common adapter `dt` on every step. The small final
floating-point representation above 80.0 in serialized endpoint times is the
binary sum of 4,000 accepted `0.02` increments, not an extended horizon.

The result did not cause any horizon, parameter, or mechanics change.

