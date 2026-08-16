# Implementation contract

## World

`StaticObstacleV1` is a deterministic inert circle. Its center and radius are
geometry only; there are no object labels, target positions, danger values, or
commands. No moving agents, resource gradients, procedural generation, or UI
are included.

For boundary vertex `i`, with local penetration `p_i` and outward normal
`n_i`, the frozen contact law is:

```text
|F_i| = clamp(0.5 * p_i, 0, 0.5)
F_i = |F_i| n_i
contact_stimulus_i = clamp(|F_i| / 0.5, 0, 1)
```

Zero penetration produces both zero force and zero stimulus. The obstacle
adapter only observes `MaterialMesh`; it never writes vertex coordinates.

## Boundary

`mechanics_step_with_external_forces` and its combined edge-tension variant
accept a force vector only after checking its length, finiteness, and bounded
per-vertex magnitude. The existing mechanics integrator then resolves movement
and material conservation. A zero force vector follows the same integration
path as legacy mechanics.

`contact_stimulus_i` is added to the existing local frame stimulus and then
passes through the existing two-neighbor regulatory dynamics. DC-DEV-005's
single `adaptation_i` trace is reused; no new learning mechanism is added.

## Exclusions

This package does not add vision, hearing, object recognition, resources,
locomotion optimization, reward, fitness, evolution, fission inheritance, or a
second actuator. Fission and unknown topology transfer remain fail-closed.

