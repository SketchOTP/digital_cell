# DC-DEV-015 metabolic intake-to-restoration audit

## Disposition

`DCDEV015_METABOLIC_INTAKE_TO_RESTORATION_AUDIT_COMPLETE`

Gate-8 classification: `DCDEV015_RESOURCE_CONVERSION_WITHOUT_HOMEOSTATIC_RESTORATION`

This is an observer-only diagnostic from the accepted DC-DEV-013 head. It does
not repair metabolism, change resource uptake, add hunger state, or add any
behavior. The audit reproduces the accepted 5,000-step mechanics settlement
and 480-step resource-free deprivation, then runs matched 480-step arms.

## Arms and findings

- A: resource-free maintenance with existing reactions.
- B: finite N/F region at `[4.8, 0.0]`, radius `1.5`, initial N/F `3.0/3.0`, existing uptake and reactions.
- C: identical resource geometry with zero inventory and existing reactions.
- D: finite N/F uptake with reactions disabled, to isolate precursor ingress.

Feeding delivered `2.3416256627929997` N and the same F mass, with world loss
equal to delivery and zero conservation residual at the resource boundary.
The feeding arm accumulated `0.043177692011290694` N/F consumption and A
production, a matched-precursor conversion fraction of
`0.01843919491375493`. Thus intake and conversion are real within the window.

The strict restoration tests failed for A, R, E_stored, and E_available:
feeding improved each relative to no-delivery, but each fed value was farther
from its replete reference than the deprived value. Better-than-control is
therefore not reported as restoration. The reaction-arm material reconciliation
residuals are preserved in `results.json`; no missing mass is invented.

## Observer boundary and verification

The assay reads existing `MaterialMesh` fields and `ReactionLedger` /
`ReserveLedger` values. A-decay is a balance-derived observer residual after
the explicit existing ledger destinations; no chemistry equation is duplicated.
E_stored, E_precursor, and E_available are derived observers and do not feed
the simulation. The instrumented and non-instrumented feeding trajectories
matched exactly.

No production behavior changed. DC-DEV-014 was not imported. DC-DEV-016 was
not started.
