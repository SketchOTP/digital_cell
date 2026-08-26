# DC-DEV-020-M1-R6-R4 — Homeostasis/contact causal audit

## Authority and scope

This is an observer-only diagnostic entered from
`69b6133a5f76d3c7839705c78922c7452ad5d550`. It does not alter production
mechanics, chemistry, transport, resource geometry, parameters, death rules,
or the production selector. No controller, target size, resource following,
M2 behavior, recycling, or salvage is present.

The runtime identity is `GeometryConservativeV3` material,
`ConservativeV3` chemistry, reserve OFF, unchanged uncoupled V1 finite spatial
transport, and `FINITE_SPATIAL_BACKING_RESERVOIR_V1` with center `[4.8, 0.0]`,
radius `1.5`, boundary concentration `2.063914918930895`, inventory
`243.14924801053778` for both N and F, and zero replenishment.

## Diagnostic arms

- `ACTUAL_FULL_RUNTIME`: finite uptake, reactions, mechanics, remesh, and
  local rebond in the existing order.
- `R5_STATIC_REFERENCE`: finite uptake and reactions with mechanics, remesh,
  and rebond disabled, reproducing the accepted reaction-only reference.
- `GEOMETRY_FROZEN_DIAGNOSTIC`: the same no-mechanics diagnostic clone,
  explicitly kept separate in the evidence schema.
- `CONTACT_PRESERVED_UPPER_BOUND_DIAGNOSTIC`: normal reactions and mechanics,
  but the existing flux law is evaluated on all intact edges without applying
  the production exposure predicate. This is an observer-only upper bound and
  uses the same finite inventory; it is not production behavior.
- Recovery arms preserve the 480-step deprivation state and refeed it without
  resetting organism state.

Dense per-step ledgers are stored on Atlas at
`\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r4`.
The compact authority is under `experiments/generated/dcdev020m1r6r4/`.

## Local result

The actual fed arm reproduces the accepted repaired R6 reference:

| Measure | Actual full runtime |
| --- | ---: |
| Organized-material delta | `-82.9654506509167` |
| N/F delivered | `14.627590100158919` each |
| N/F remaining | `228.521657910379` each |
| First permanent zero exposure | step `850` |
| Final area | `0.858819873077677` |
| Closure | pass, maximum residual below `1e-8` |

The body has only two exposed edges at its maximum and stops receiving material
after step 849 even though the external reservoir remains mostly full. The
all-intact observer shadow diverges from the actual eligible flux from the
first step, and its cumulative exposure-only potential deficit is positive.

The accepted R5 static reference receives `162.464640538382` N/F and has
organized-material delta `+0.342140676890381`, while area and geometry remain
fixed. The geometry-frozen diagnostic reproduces that result.

The contact-preserved upper bound receives the entire fixed finite inventory
(`243.149248010538` N/F, zero remaining) but still ends with organized-material
delta `-17.4947722071266`. It remains closed/intact and observer viable. This
does not prove failure under an unlimited source; it proves that preserving
contact opportunity alone is insufficient under the preregistered finite
inventory and embodied runtime demand.

The actual 480-step deprivation entry is preserved without reset. Refeeding
the actual body changes organized material by `-75.902684394052` from the
deprived state; the contact-preserved upper bound changes it by
`-8.4357585982911`. Neither reaches the restoration criterion.

## Causal interpretation

The bounded result is classified:

`M1_FULL_RUNTIME_EMBODIED_DEMAND_DOMINANT`

Contact loss is an early and substantial contributor: it truncates actual
delivery at step 850 while accessible inventory remains. However, the
contact-preserved diagnostic receives more material than the static R5 arm and
the full fixed finite inventory, yet still fails sustained homeostasis and
restoration. Therefore contact preservation is not sufficient, and the
remaining embodied structural/membrane/reaction demand is dominant within this
fixed finite-resource assay.

This classification is bounded evidence, not a repair authorization. It does
not select a new resource geometry, impose a target area or SA/V ratio, or
authorize M1 repair, production changes, or M2.

## Preservation

The harness records V2 and V3 D-087 reports and requires both to pass. The
scoped remote workflow also runs GC conservation, metrics, D-088, D-091, and
evolution-harness preservation tests. `NEXT_EXECUTION_STARTED` is `false`.
