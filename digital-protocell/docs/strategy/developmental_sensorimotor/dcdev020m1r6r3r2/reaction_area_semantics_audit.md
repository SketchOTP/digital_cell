# DC-DEV-020-M1-R6-R3-R2 reaction-area semantics audit

## Authority and scope

This is an observer-only diagnostic entered at commit `dac4973ea74e298744a69b5829c33dd0f7db85f4` on `strategy/dc-dev-020r9-mesh-contract-requalification`. It audits the historical reaction-area floor against the already-qualified `GeometryConservativeV3` material contract. It does not change reaction equations, coefficients, mechanics, transport, geometry conservation, remeshing, rebonding, death rules, tolerances, or production selection.

The runtime remains `GeometryConservativeV3` + `ConservativeV3` + reserve OFF + unchanged uncoupled V1 finite spatial transport, in the existing order: finite resource uptake, reactions, mechanics, remesh, then local rebond. The diagnostic clone/replay is never fed back into the live state.

## Static pathway map

Production reactions use `mesh.area().max(1e-6)` as the reaction area. A bridge is floor-sensitive when an absolute amount is converted to an interior concentration (or the reverse) using that area while GC accounting uses the actual positive area.

| Path | Transfer and representations | R6-R3 active? | Reserve OFF active? | GC floor risk |
| --- | --- | --- | --- | --- |
| `reactions_step_with_reserve_mode` activation | N/F, A, C and W concentration updates | yes | yes | none when both sides are concentrations under the same snapshot area |
| `reactions_step_with_reserve_mode` catalyst production/turnover | concentration to concentration | yes | yes | none from this floor bridge |
| `reactions_step_with_reserve_mode` A decay/W | concentration to concentration | yes | yes | none from this floor bridge |
| structural build | A or reserve concentration consumed as an amount; M edge amount produced; remainder may enter W concentration | yes | A source yes, reserve source no | `produced + waste*r - source*r`, where `r = actual_area/reaction_area` |
| structural turnover | M edge amount removed; W concentration receives `M/reaction_area` | yes | yes | `-M_to_W + M_to_W*r` |
| membrane production | A or reserve concentration consumed as an amount; free-L amount produced; remainder may enter W concentration | yes | A source yes, reserve source no | `produced + waste*r - source*r` |
| reserve allocation calls | amount/concentration bridges | no | no | not active in this assay |
| bind/unbind | amount to amount | no floor bridge | no | none |
| `try_local_rebond` | amount/concentration bookkeeping exists | no material transfer observed | no material transfer observed | no active R6-R3 contribution |
| damage, fission, growth, topology paths | separate area-sensitive paths | not reached | not reached | outside this runtime |

The audit ledger records structural build, structural turnover, membrane production, reserve, and other contributions separately. Reserve and other are explicitly zero for this reserve-OFF path rather than silently folded into numerical error.

## Dynamic method

At every reaction step the observer records actual area, floored reaction area, the reaction-stage residual, the pre/post material state, all active transfer magnitudes, largest concentration, smallest/largest transfer, ULP scale, and an estimated rounding increment. It predicts the GC residual for each active amount/concentration bridge using the unchanged production equation.

Selected states are cloned immediately before the live reaction and replayed with the exact reaction parameters and timestep used by that live segment, with mechanics, uptake, remesh, and rebond disabled. Stable hashes of the replay mesh and reaction ledger must match the original post-reaction mesh and ledger. This supplies a frozen one-step reproduction without altering the trajectory.

The causal test compares cumulative observed reaction residual with the sum of predicted floor-mediated transfer residuals, while reporting the floating-point remainder independently. The existing `1e-8` accounting tolerance is not relaxed.

## Platform boundary

The Windows run reaches physical topology rupture before the reaction area falls below `1e-6`; its local classification is therefore unresolved/preempted, not a cross-platform disproof. Linux exact-head CI is the authority for the long-deprivation floor-crossing and residual evidence. A diagnostic confirmation of a production defect is a valid scientific result and is not itself a CI infrastructure failure.

## Evidence and preservation

Compact protocol, results, qualification, preservation, and manifest files are kept under the versioned Git evidence namespace `experiments/generated/dcdev020m1r6r3r2/`. Dense JSONL ledgers are written only to the canonical Atlas evidence root:

`\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r3r2\`

The workflow checks exact authority, bounded paths, D-087 V2/V3 preservation, GC conservation tests, Phase-1 metrics, D-088, D-091, evolution-harness, compact-content agreement, frozen replay, and diagnostic classification. No production chemistry source is part of the R2 change set.

## Status boundary

R6-R3-R1 remains investigate/not accepted because the snapshot repair exposed reaction-stage nonconservation in the Linux full runtime. This R2 audit is diagnostic only. M1 remains not established; production reaction repair, M2, recycling/salvage, and DC-DEV-021 remain unauthorized pending architect review.
