# DC-DEV-005: Local Experience-Dependent Plasticity

This package adds exactly one slow local state, `adaptation_i`, to each
regulatory patch. The state is bounded to `[0, 1]`, changes only from local
regulatory activity and accepted `MechParams.dt`, and recovers during local
inactivity.

The trace scales the already-qualified DC-DEV-004 local contractility input:

`effective_activity_i = activity_i * (1 - adaptation_i)`

No sensor, actuator, command, reward, fitness, optimizer, target behavior,
evolution, or fission inheritance is added. The draft package remains pending
architect review.

## Evidence

- [Implementation contract](implementation_contract.md)
- [Timescale preregistration](timescale_preregistration.json)
- [Habituation and recovery assay](habituation_assay.md)
- [Remesh and unsupported boundaries](remesh_and_boundaries.md)
- [Gate results](gate_results.md)
- [Final conclusion](final_conclusion.md)
- Generated artifacts: `digital-protocell/experiments/generated/dcdev005/`
