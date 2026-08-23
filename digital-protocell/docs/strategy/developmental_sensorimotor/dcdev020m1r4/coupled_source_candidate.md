# DC-DEV-020-M1-R4 coupled-source candidate

This package qualifies `FINITE_SPATIAL_RESOURCE_COUPLED_ACTIVATION_V1` from
the accepted R3 head. It wraps the unchanged
`FiniteSpatialResourceRegionV1::uptake` calculation and transforms only the
paired N/F mass delivered during that boundary step:

```text
paired = min(newly_delivered_N, newly_delivered_F)
paired -> A + W
unmatched delivered material remains N/F
```

Pre-existing internal N/F is not consumed by the boundary candidate. The
candidate uses ConservativeV3 with reserve OFF and remains opt-in and
unselected. ConservativeV2 remains frozen and selected for production.

The preregistered experiment uses the accepted depleted M1 state, resource
center `[4.8, 0.0]`, radius `1.5`, inventory `14.588954880632265` each, `dt`
`0.02`, and exactly 480 accepted steps. It compares V1 baseline, the accepted
source-capacity observer reference, and the coupled candidate, then runs
physical no-contact, one-species, depletion, rupture, and pre-existing N/F
controls.

This is a source-realization candidate only. It does not change V1 transport,
V3 chemistry, kinetics, resource geometry, inventory, reserve, death rules,
mechanics, recycling, M2, or DC-DEV-021.
