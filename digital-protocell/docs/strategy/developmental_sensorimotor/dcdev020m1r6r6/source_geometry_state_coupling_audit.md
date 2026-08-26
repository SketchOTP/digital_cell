# DC-DEV-020-M1-R6-R6, source and geometry state-coupling audit

## Authority and boundary

This observer-only audit starts from `73067f702a8f5386c440629c454e40ab1e434e91`,
the architect-accepted R6-R5 head. It keeps `GeometryConservativeV3`,
`ConservativeV3`, reserve OFF, the finite R5 resource contract, production
coefficients, mechanics, transport, and the production selector unchanged.
No repair, source redesign, recycling, salvage, controller, or M2 work is
included.

Dense per-step records are written to:

`\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r6\`

Only compact protocol, results, qualification, preservation, and manifest
files are retained in the repository for CI and audit.

## Diagnostic design

The audit replays the accepted R6-R5 actual moving, geometry-frozen,
contact-upper, and matched-source arms. It adds one preregistered schedule:

`FRONTLOADED_EQUAL_TOTAL`

This schedule uses the ordered contact-upper delivery sequence, truncating only
the final delivery needed to match the successful frozen/static total of
`162.4646405383817` N and F exactly within floating-point tolerance. It is run
on static geometry, so source timing is isolated from moving geometry.

For the matched successful frozen schedule, the moving and static runs record
the per-step activation, A decay, catalyst turnover, structural turnover,
structural production, A/C/M state, area, perimeter, strain, turnover factor,
and material-time integrals. Observer-only factor swaps identify the measured
contribution of A concentration, catalyst saturation, strain gain, and edge
geometry to structural build demand. They never feed back into a trajectory.

## Local result

The accepted R6-R5 reference values reproduce:

| Arm | Organized-material delta | N/F delivered |
| --- | ---: | ---: |
| Actual moving | `-82.9654506509167` | `14.6275901001589` |
| Geometry frozen | `+0.342140676890381` | `162.464640538382` |
| Contact-upper moving | `-17.4947722071266` | `243.149248010538` |
| Matched source static | `-16.0257265275525` | `243.149248010538` |
| Matched frozen schedule static | `+0.342140676890352` | `162.464640538382` |
| Matched frozen schedule moving | `-9.95495920654304` | `162.464640538382` |

The equal-total source-history test is decisive:

| Static schedule | Organized-material delta | Last positive delivery |
| --- | ---: | ---: |
| Successful frozen | `+0.342140676890352` | `8000` |
| Front-loaded equal-total | `-36.9748746832668` | `998` |

The two schedules deliver the same `162.4646405383817` N and F. The
front-loaded schedule raises the A material-time integral from
`3696.226552818364` at the decay point in the frozen schedule to
`5222.48519549487`, and the C material-time integral from
`7499.073256470541` to `10713.515103301`. The corresponding excess sinks are
`12.220590145704` A-decay units and `32.1444184683056` catalyst-turnover
units. The source-history pathway is therefore measured, not inferred from
total input alone.

The same successful source schedule remains negative only when geometry moves.
Moving-minus-static excess is:

| Quantity | Difference |
| --- | ---: |
| Activation | `+10.346844205316412` |
| A decay | `+2.7193695206011395` |
| Catalyst turnover | `+4.482073601543119` |
| Structural turnover | `+13.44250096660451` |

The moving structural-turnover excess is positive. The zero-positive-strain
turnover shadows are `63.149461566251105` static and `51.75197411176439`
moving, while actual strain suppression is `26.20505085935331` static and
`1.3869253087649367` moving. This rules out positive strain suppression as the
source of the moving excess. The dense ledger retains the per-step M stock,
strain, turnover factor, and structural-build factor observations needed for
further review.

## Recovery correspondence

The existing no-reset deprivation then upper-source refeed remains a moving
geometry diagnostic. Deprivation changes organized material by
`-10.9790910223109`; the upper-source refeed changes it by
`-9.78821372150109` from the original entry and ends below the deprived state.
The same source-history and moving-state measurements are retained in the
recovery ledgers. No refeed strength was changed.

## Classification

```text
M1_SOURCE_FRONTLOAD_AND_GEOMETRY_STRUCTURAL_CYCLE_CONFIRMED
```

This is a causal audit result, not a homeostasis qualification and not a
repair authorization. M1 remains not established. Production remains
ConservativeV2/reserve OFF, and M2 remains unauthorized.

## Preservation

Fresh V2 and V3 D-087 reports are both `8/8`. The exact-head workflow also
re-runs the accepted R6-R4/R6-R5 references, GC conservation, Phase-1, D-088,
D-091, and evolution-harness checks. Diagnostic negatives are represented by
successful CI with an explicit diagnostic classification.
