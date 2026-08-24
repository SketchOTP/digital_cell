# DC-DEV-020-M1-R6-R3 Full-Runtime M1 Certification

Directive: `DC-DEV-020-M1-R6-R3-FULL-RUNTIME-M1-CERTIFICATION-001`

Starting authority: `0c56890d1f59c5dc2ffc66fd1d69181d7ca7b8c5`

## Contract

This is an observer-only certification of the already-qualified material
contract. The runtime identity is:

- material: `GeometryConservativeV3`;
- chemistry: `ConservativeV3`;
- reserve: `OFF`;
- transport: unchanged uncoupled V1 finite spatial transport;
- world: `FINITE_SPATIAL_BACKING_RESERVOIR_V1`;
- production default: unchanged `ConservativeV2` / reserve OFF.

The accepted R6 entry state is reused from the existing M1/R5 entry helper,
then explicitly stamped with the versioned GeometryConservativeV3 material
contract. Stamping does not alter material amounts. Every accepted step uses
the existing order:

```text
finite resource uptake -> reactions -> mechanics -> remesh -> try_local_rebond
```

No second transport call, reset, controller, target, repair law, or parameter
change is present.

## Accounting

The harness records strict material before and after uptake, reactions,
mechanics, remesh, and rebond. Uptake additionally subtracts the N+F delivered
on that step. It records both cumulative and maximum per-step residuals with a
fixed tolerance of `1e-8`. The remesh helper's boolean return is not treated as
the conservation authority; the before/after amount residual is independently
checked and fails closed.

Dense step ledgers are written to:

```text
\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r3
```

Only compact JSON evidence is kept in Git.

## Arms

1. Fed full runtime, 8,000 accepted steps, finite N/F reservoir.
2. Recoverable deprivation, 480 no-resource steps followed by 8,000 steps of
   refeeding without resetting the organism.
3. Zero-resource continuation to the existing 150,000-step bound.
4. Fed arm at step 8,000 followed by removal of all external N/F, continued to
   the same bound.
5. If physical topology loss occurs, the exact ruptured state is refed for
   5,000 steps without reset and without stronger source capacity.

## Local result

The first local run is a valid negative candidate. Stage accounting passes and
both fresh V2 and V3 D-087 reports are 8/8. The fed arm remains viable and
intact at 8,000 steps, but organized material changes by
`-82.9654506509167`. The 480-step deprivation lowers organized material by
`-10.9790910223109`; the subsequent refeed does not restore it and increases
the final deficit, so restoration is not established. Zero-resource and
feed-then-remove arms reach topology rupture at local steps `8867` and `11283`
respectively. The classification candidate is:

```text
M1_FULL_RUNTIME_HOMEOSTASIS_FAILED
```

This does not close M1 and does not authorize a repair. Remote Linux execution
and Architect review remain authoritative.

## Preservation boundary

The certification package does not modify chemistry equations, transport,
resource geometry or inventory, decay, structural/catalyst/membrane kinetics,
mechanics, remesh, rebond, observer death rules, D-087, D-091, production
selection, M2, recycling/salvage, or DC-DEV-021.

`NEXT_EXECUTION_STARTED:false`
