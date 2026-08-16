# Implementation contract

## Entry and authority

- Entry commit: `3a5971be332f94848250196e8148b722464066f2`
- Source branch: `strategy/dc-dev-006-spatial-contact-environment`
- Implementation branch: `strategy/dc-dev-007-active-contact-regulation`
- Time authority: accepted `MechParams.dt` from each accepted mechanics step.
- Coordinate authority: chemistry-core mechanics only.

## Existing components only

The assay calls `StaticObstacleV1::observe`,
`augment_frame_with_contact`, `ContinuityNetworkV1`,
`PlasticityStateV1`, `apply_local_plasticity_with_external_forces`, and the
existing mechanics solver. Motor-off changes only `max_active_tension` to
zero; it retains contact physics, regulation, and adaptation updates. The
zero-reserve arm sets the existing material reserve `R` to zero and therefore
uses the already-qualified passive external-contact path.

No direct coordinate assignment is present in the assay.

## Matched arms

All arms begin from identical mesh, obstacle, regulator, adaptation, reserve,
mechanics, and simulation-time state. The one fixed horizon is not screened or
changed after observing results.
