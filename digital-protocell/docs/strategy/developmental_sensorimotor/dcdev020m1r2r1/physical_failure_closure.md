# DC-DEV-020-M1-R2-R1 — Physical-Failure Closure

Directive: `DC-DEV-020-M1-R2-R1-PHYSICAL-FAILURE-CLOSURE-001`

Entry head: `bc65098c3d26777aca2d1da5dab8cc118ecc6e19`

This is an observer-only continuation of the accepted M1-R2 chemistry path. It
replays the two R2 starvation arms from the exact accepted founder state,
requires the R2 endpoint hashes and state values to match, then continues each
trajectory for at most 150,000 additional accepted reaction steps. No mesh
mechanics, remeshing, rebonding, production chemistry, resource law, or death
rule was changed.

## Existing terminal boundary

The assay uses only existing conditions:

- at least half of the ring edges are ruptured;
- `observer_death_reason` is a non-starvation reason; or
- `physical_runtime_valid == false`.

`starvation_collapse` alone is not terminal. An isolated edge rupture is
recorded but does not stop the continuation. Failure margins record minimum,
median, and maximum edge structural mass, bond threshold, edge count, and
ruptured-edge count at the R2 endpoint, sparse continuation checkpoints, and
the terminal state.

## Restoration challenge

Only an arm that reaches the existing terminal boundary receives the exact
finite-resource restoration challenge:

```text
N = 14.588954880632265
F = 14.588954880632265
center = [4.8, 0.0]
radius = 1.5
steps = 5,000
```

The failed mesh is cloned without resetting A, C, M, B/L, waste, topology,
rupture flags, geometry, or observer state. Restoration reports delivery,
material change, closure, viability, topology, and coherent-body status. It is
not described as full-runtime organism death because this lineage advances the
chemistry path only; M0 mechanics/remesh/rebond are outside the assay.

## Result

The exact R2 endpoint replay passed for both arms. Scalar endpoint values are
checked with the accepted numeric tolerance; trajectory hashes remain recorded
as provenance but are not required to match across Windows and Linux
floating-point replays. Both arms subsequently
reached the existing `activated_catalyst_collapse` boundary without any edge
ruptures:

| Arm | First terminal step | Terminal reason | Restoration | Organized-material change during restoration |
| --- | ---: | --- | --- | ---: |
| production 4x | 45,422 | `activated_catalyst_collapse` | 5,000 steps | negative |
| ordinary decay | 45,831 | `activated_catalyst_collapse` | 5,000 steps | negative |

Both restoration runs delivered the requested N/F, closed the internal/world
material accounting within tolerance, and failed to restore coherent
organized material or a closed intact body. Therefore the exact bounded
classification is:

```text
M1_ORDINARY_DECAY_IRREVERSIBLE_FAILURE_ESTABLISHED
```

This means ordinary decay is sufficient for the bounded chemistry-path
terminal failure; the 4x term is not necessary for that failure class. It does
not establish full-runtime organism death, authorize a production change, or
authorize recycling/salvage, M2, or DC-DEV-021.

## Artifacts

Compact authoritative artifacts are in
`experiments/generated/dcdev020m1r2r1/`. Dense ledgers are not committed.

The final remote CI must also verify fresh actual D-087 `8/8`, strict material
closure, and preservation regressions for Phase 1, D-091, D-088, and
evolution-harness before architect review.
