# DC-DEV-009: Free-space motility feasibility audit

Status: coder-reported complete, architect review pending.

Entry authority: `79751bed5ad78d367b7409f0ec677e32a3b9d527` on
`strategy/dc-dev-008-spatial-resource-acquisition`.

The audit asks a narrow question: can the accepted fixed-topology mechanics and
local contractility translate the organism through free space? It does not add
an actuator, substrate, friction, adhesion, fluid physics, navigation, sensing,
or a controller.

The fixed 24-vertex ring was run for 240 accepted mechanics steps, `4.8` units
of simulated time at `MechParams.dt = 0.02`. Growth, remeshing, fission,
obstacle contact, external force, and spatial-resource acquisition were all
disabled. The active arm and motor-off arm received the same preregistered local
stimulus and produced identical regulatory state traces.

## Finding

`DCDEV009_MOTILITY_FEASIBILITY_AUDIT_COMPLETE`

`DCDEV009_EXISTING_FREE_SPACE_MOTILITY_NOT_ESTABLISHED`

Contractility deformed the body, but its endpoint forces remained equal and
opposite. The contractility-only centroid displacement was
`2.473548217003853e-18`. The active-minus-control centroid drift was
`0.001102180818121483`, and the observer ledger independently attributes the
same value to the changed baseline mechanics force field after deformation.
That drift is not accepted as locomotion.

See the generated evidence in
`digital-protocell/experiments/generated/dcdev009/` and the implementation
boundary in `implementation_contract.md`.

DC-DEV-010 is not started.
