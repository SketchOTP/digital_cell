# DC-DEV-020-M1-R6-R3-R1 GC Snapshot Area Semantics Correction

Directive: `DC-DEV-020-M1-R6-R3-R1-GC-SNAPSHOT-AREA-SEMANTICS-CORRECTION-001`

Starting authority: `3b86680465dbfed26f2bcf6ac9765468f67a0afb`

## Scope

This is an observer/accounting correction only. `GeometryConservativeV3`
concentration-derived snapshot amounts now use the actual positive mesh area,
matching the qualified geometry-conservation primitive. `HistoricalV1` and
`ConservativeV2` retain the historical `max(area, 1e-9)` snapshot semantics.
No mechanics, remesh, rebond, chemistry, transport, resource, death, or
production behavior was changed.

The `snapshot()` callers audited for this directive are diagnostic, evidence,
or certification callers. No production biology path consumes snapshot values.

## Defect and correction

Before the correction, `mesh_contracts::snapshot()` used the historical area
floor for all three material contracts. In a sub-floor GC fixture, the same
positive concentration therefore represented different amounts under the
qualified GC law and the observer snapshot. The focused tests reproduce the
inflated floored amount, verify actual-area amount identity for all snapshot
concentration fields, and verify strict conservation across a sub-floor
geometry change. Historical V1 and V2 floor behavior remains covered.

The correction is versioned on `MeshContractVersion` and does not alter
serialized contract schemas or physical state.

## R6-R3 replay

The unchanged full-runtime R6-R3 package is written to the append-only compact
evidence directory:

```text
experiments/generated/dcdev020m1r6r3r1/
```

Dense ledgers are stored on canonical Atlas only:

```text
\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r3r1\
```

The local replay preserves the previously observed biology: fed organized
material delta `-82.9654506509167`, deprivation delta
`-10.979091022310868`, and no-reset refeed delta
`-75.90268439405197`. Stage accounting closes within `1e-8` for fed,
recovery, zero-resource, and feed-then-remove arms. Local topology rupture
occurs at steps `8867` and `11283`; no minimum-area death rule was added.

Local classification:

```text
M1_FULL_RUNTIME_HOMEOSTASIS_FAILED
```

The result does not establish M1 and does not authorize production changes,
homeostasis repair, M2, recycling, salvage, or DC-DEV-021.

`NEXT_EXECUTION_STARTED:false`
