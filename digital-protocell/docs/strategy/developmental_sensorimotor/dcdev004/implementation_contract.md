# Implementation Contract

## Frozen resource coupling

The actuator uses the already-existing D-091 metabolic reserve `MaterialMesh.interior.r`. It spends reserve into the already-existing `MaterialMesh.interior.w`; no motor-energy currency is introduced. The frozen conversion is `0.05` reserve units per force-length-time. The frozen maximum active tension is `2.0`.

## Local actuator rule

For edge `i`, the requested active tension is:

`T_i = 2.0 × 0.5 × (activity_i + activity_(i+1))`

Only existing edge endpoints participate. Tension is bounded by available reserve and is passed to the existing overdamped mechanics kernel. The actuator never writes vertex coordinates.

If activity is zero, the exact legacy `mechanics_step` path is used. If reserve is zero, the exact legacy mechanics path is used and no reserve is spent.

## Excluded capabilities

There is no target coordinate, target shape, direction, planner, behavior label, reward, fitness, global action selector, new sensor, memory, learning, evolution, or fission-state inheritance.
