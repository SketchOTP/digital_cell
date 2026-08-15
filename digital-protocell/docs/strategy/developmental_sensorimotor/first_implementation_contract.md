# First implementation contract

This is a future implementation boundary, not implementation in DC-DEV-001A.

## Minimal contract

`LocalRegulatoryPatch` stores a versioned local state, reads a bounded `LocalSensorFrame`, updates at the accepted simulation time, and returns bounded `LocalEffectorIntent` values. The patch is attached to existing material topology; it cannot create or destroy material, declare birth/death, bypass fission, or read global population state.

The update function must be deterministic for `(initial_local_state, sensor_frame, neighbor_frame, accepted_dt, seed, implementation_version)`. Every update records input provenance, output intent, and accepted time. Unsupported sensor or effector mapping returns `HARNESS_ADAPTER_UNAVAILABLE` rather than a silent no-op.

## First assay boundary

The first implementation should demonstrate one local sensor, one local state variable, one bounded effector, perturbation response, persistence across several accepted steps, and no material-conservation residual. It must run beside the frozen certifier and use a neutral/no-op control with the same execution machinery. It must not include selection, learning, reproduction qualification, or population fitness.

## Exit criteria for the next directive

The next directive may implement only this contract after independent review. It must add source-level tests, causal perturbation tests, replay evidence, and governance artifacts before any expansion.

