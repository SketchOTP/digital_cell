# DC-DEV-009 implementation contract

This directive is an investigation and falsification package, not a motility
implementation.

## Runtime boundary

The only runtime calls added by the assay are the existing
`ContinuityNetworkV1`, `apply_local_contractility`, `compute_forces`, and
`mechanics_step` paths. The assay records forces, edge tensions, geometric and
edge-mass-weighted centroids, shape change, topology size, and regulatory state
hashes. It never writes mesh coordinates and does not change production
mechanics.

The production movement authority remains
`chemistry-core/src/mesh_mechanics.rs`. Its overdamped update applies one
scalar `gamma` to every vertex. Existing contractility adds tension to an edge
as equal-and-opposite forces on its two endpoints. The observer reconstructs
those pair vectors only to audit their sum.

## Matched arms

The active and motor-off arms start from the same mesh, use the same fixed local
stimulus, and keep topology fixed. Their regulator frames use the same stimulus
and stable topology event at every step. Their regulatory trace hashes must be
identical. Motor-off uses `mechanics_step` without edge tension.

The material centroid is an observer metric based on existing edge structural
plus membrane mass at each edge midpoint. It does not alter any material field.

## Qualification rule

Shape change is not locomotion. A valid free-space translation would require a
deterministic active-minus-control body displacement attributable to the
accepted actuator and environment coupling. Here the contractile force sum and
contractility-only centroid displacement are within numerical tolerance, while
the residual centroid drift is explained by the baseline force-field difference
created after the active arm changes shape. The finding is therefore
`DCDEV009_EXISTING_FREE_SPACE_MOTILITY_NOT_ESTABLISHED`.

No recommendation in the audit is implemented. DC-DEV-010 remains stopped.
