# DC-DEV-020-M1-R6-R5 — Embodied-sink causal decomposition

## Authority and boundary

This is an observer-only diagnostic entered from
`48ac1da5c6af6a9157d482d6fffecd32ee6e82c8`. It does not change production
chemistry, reaction coefficients, mechanics, transport, resources, GC
conservation, death semantics, or production selection. The `A_DECAY_OFF`,
`C_TURNOVER_OFF`, and `M_TURNOVER_OFF` arms are fixed non-production knockout
probes; no intermediate values, controller, recycling, or salvage are used.

The runtime identity is `GeometryConservativeV3` material,
`ConservativeV3` chemistry, reserve OFF, `dt=0.02`, and the unchanged finite
resource boundary from R6-R4. Dense per-step ledgers are written to
`\\atlas\ATLAS\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r5`;
compact authority is under `experiments/generated/dcdev020m1r6r5`.

## Diagnostic design

The organized-material identity is evaluated independently at every reaction
step:

```text
organized change = activation - A_decay - C_turnover - M_turnover
```

Reserve loss, structural damage, and membrane damage are recorded separately
and are expected to be zero in this reserve-OFF, no-damage runtime. Gross
`L<->B` binding/unbinding is recorded as redistribution, not as an organized
loss channel.

The package contains:

- actual moving full runtime;
- geometry-frozen reference;
- contact-preserved moving upper bound;
- static replay of the contact-upper source schedule;
- static and moving replay of the R5-like source schedule;
- three fixed single-sink knockouts;
- conditional pairwise knockouts only if all single probes fail;
- no-reset deprivation/refeed shadows for the single probes.

Source schedules are diagnostic finite-inventory replays. They are not fed back
into the production transport path.

## Local result

The local Rust 1.89.0 run reproduces the accepted R6-R4 values:

| Arm | Organized delta | N/F delivered each |
| --- | ---: | ---: |
| Actual moving | `-82.9654506509167` | `14.6275901001589` |
| Geometry frozen | `+0.342140676890381` | `162.464640538382` |
| Contact upper bound | `-17.4947722071266` | `243.149248010538` |
| Static, upper-bound schedule | `-16.0257265275525` | `243.149248010538` |
| Static, frozen schedule | `+0.342140676890352` | `162.464640538382` |
| Moving, frozen schedule | `-9.95495920654304` | `162.464640538382` |

The contact-upper sink totals are:

```text
activation       241.909754194421
A decay           61.2772437842493
C turnover       133.904433318958
M turnover        64.2228492983393
net organized    -17.4947722071266
```

All diagnostic single probes cross the existing organized-material endpoint
criterion under the contact-upper source schedule:

```text
A_DECAY_OFF       +15.4670751146907
C_TURNOVER_OFF   +118.251161839601
M_TURNOVER_OFF    +43.1502239080495
```

Because more than one single knockout is sufficient, no single sink satisfies
the directive's dominant-channel definition. Pairwise probes are therefore
not exercised, and the bounded classification is
`M1_EMBODIED_SINK_CAUSE_UNRESOLVED` pending exact-head remote validation.

The matched source replays show both effects are present in this fixed assay:
the contact-upper delivery profile makes static geometry decline, while moving
geometry declines under the R5-like schedule that keeps static geometry at
break-even. These are diagnostic observations, not repair authorization.

## Preservation and next boundary

The package requires fresh V2/V3 D-087 reports, GC conservation, R6-R3-R3,
R6-R4 reproduction, Phase-1, D-088, D-091, and evolution preservation. The
scoped workflow is the remote authority for those checks and for artifact
identity. `M1` remains not established, `M2` is not authorized, and no PR
merge or production selection change is permitted.
