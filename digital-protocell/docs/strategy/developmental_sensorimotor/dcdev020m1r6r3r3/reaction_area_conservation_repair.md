# DC-DEV-020-M1-R6-R3-R3

## GC reaction-area conservation repair

This bounded qualification repairs the causally confirmed reaction-area
semantic mismatch for `GeometryConservativeV3` only. HistoricalV1 and
ConservativeV2 continue to use the existing `max(area, 1e-6)` reaction area.

The implementation keeps two explicit concepts in
`crates/chemistry-core/src/mesh_reactions.rs`:

- `reaction_area`: the historical area used by concentration-only kinetics
  and legacy V1/V2 bookkeeping;
- `material_transfer_area`: the historical reaction area for V1/V2, and the
  actual finite positive mesh area for GC amount/concentration transfers.

The GC area is used for structural build, structural turnover, and membrane
production conversions. It is also used for the corresponding material ledger
amounts. The reaction equations, coefficients, force laws, transport, remesh,
rebond, death rules, and production default are unchanged.

## Qualification boundary

The focused unit tests cover:

1. historical and ConservativeV2 floored turnover behavior below `1e-6`;
2. GC structural turnover conservation below `1e-6`;
3. exact V2/GC state parity before the floor can activate;
4. GC structural build conservation below `1e-6`;
5. GC membrane production conservation below `1e-6`.

The integrated harness replays the accepted GeometryConservativeV3 /
ConservativeV3 / reserve-OFF runtime with the existing finite resource world,
including fed, deprivation, no-resource, feed/remove, and no-reset refeed
arms. Dense ledgers belong under the canonical Atlas evidence root:

`\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r3r3\`

The compact Git evidence is under
`experiments/generated/dcdev020m1r6r3r3/`. The Windows compact classification
may remain incomplete if topology rupture preempts the sub-floor regime. The
Linux exact-head run is authoritative for the long-deprivation boundary.

## Acceptance invariants

- HistoricalV1 and ConservativeV2 behavior is unchanged.
- GC material conversions use actual positive area.
- GC pre-floor behavior is identical to ConservativeV2.
- Post-floor structural and membrane transfers close in material units.
- Existing fed/deprivation/refeed negative evidence is preserved.
- No minimum-area floor, size controller, compensating source, recycling,
  salvage, or production selection is introduced.
- M1 remains open and M2/DC-DEV-021 remain unauthorized.
