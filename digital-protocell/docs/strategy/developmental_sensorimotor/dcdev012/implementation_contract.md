# DC-DEV-012 Implementation Contract

## Ownership

`EndogenousPolarityV1` is the sole production owner of the stochastic token
state, event propensities, deterministic RNG provenance, accepted polarity
time, token conservation, and bounded body-attached drive. It receives only
the supported topology size, provenance seed, and settled ring spacing.

It never reads coordinates, world axes, centroids, velocity, targets,
resources, obstacles, reward, fitness, or semantic regulatory labels. It never
writes mesh state or coordinates.

## Stochastic law

Each accepted `dt=0.02` is an exact finite-event continuous-time Markov step.
The event types are spontaneous uniform association, local recruitment at the
recruiting patch, one-edge nearest-neighbor membrane diffusion, and
dissociation to the cytosolic pool. Every patch uses the same rate law.

The ring hop rate is derived once from the settled body using the one-
dimensional continuous-time random-walk identity `D=q h^2`, hence
`q=D/h^2` for each of the two symmetric neighbor directions.

## Coupling

The module emits only `clamp(24 * bound_i / 1000, 0, 1)`. That vector is used
as the local input field for the existing `ContinuityNetworkV1` step, with
environmental stimulus held at zero. Existing DC-DEV-004 contractility,
reserve spending, DC-DEV-011 stick-slip, and chemistry-core mechanics remain
the authorities for force, energy, reaction, and movement.

The primary qualification keeps topology fixed at 24 patches, disables
plasticity, and uses no growth, remeshing, fission, chemistry, resource, or
environmental process.
