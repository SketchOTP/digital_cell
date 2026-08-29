# DC-DEV-020-M1-REPLAN-002-R5-R3

## Live-resource irreversible-death qualification

This observer-only qualification starts at accepted R5-R2 head
`d0a9601aed170c43a5c288c8300f3fe65e64237f` and uses the repaired
`MaturationCoupledV4` contract. It derives three states from a fresh repaired
starvation replay: S0 is the accepted bounded-deprivation recovery control, S1
is the first accepted observer-nonviable state, and S2 is the last complete
state before the next authoritative `mechanics_step == false`.

Every refeed arm instantiates the unchanged
`FiniteSpatialBackingReservoirV1` and calls its live `uptake` method on the
clone. No recorded schedule, direct interior N/F insertion, resource movement,
stronger source, or post-failure state is permitted. The runtime order is
live uptake, reactions, mechanics, remesh, and local rebond; a failed mechanics
step stops the arm.

The qualification is fail-closed. S0 must recover with positive live N/F
uptake before S1/S2 can be interpreted. S2 can establish irreversible loss
only when the unchanged reservoir provides physical opportunity and the body
does not reconstruct without a latch. Otherwise the result is opportunity-not-
established or invalid, not a death claim.

Dense per-step ledgers belong at:

`/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r5r3/`

Compact audit evidence is kept under the matching
`experiments/generated/dcdev020m1replan002r5r3/` namespace. Production remains
`ConservativeV2 / reserve OFF`; M1, M2, and successor work remain unchanged.
