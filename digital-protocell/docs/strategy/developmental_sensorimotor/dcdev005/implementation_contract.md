# Implementation Contract

Entry authority is `edf517e6b802a7cd9cf141980061127dbb697b21` on
`strategy/dc-dev-004-local-contractility`.

`PlasticityStateV1` contains one vector of local `adaptation_i` values and an
explicit enabled control. Each value is clamped to `[0, 1]`. On an accepted
step:

```text
load       = load_rate * activity_i * (1 - adaptation_i)
recovery   = recovery_rate * (1 - activity_i) * adaptation_i
adaptation = clamp(adaptation_i + MechParams.dt * (load - recovery), 0, 1)
```

The response for the current step uses the prior trace value. This makes an
all-zero trace exactly equal to DC-DEV-004 and makes prior exposure affect a
later response. The trace is committed only after the existing contractility
and mechanics path accepts the step.

Ordinary split/merge topology mappings reuse DC-DEV-003 local correspondences.
Fission, unknown topology, and invalid mappings fail closed.
