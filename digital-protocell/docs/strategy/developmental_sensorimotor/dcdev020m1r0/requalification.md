# DC-DEV-020-M1-R0 — Finite-resource requalification

This is an observer-only requalification of the preserved DC-DEV-015 and
DC-DEV-016 finite N/F assays against the accepted M0 production identity:

```text
mesh_contract = ConservativeV2
reserve_enabled = false
```

The assay starts at M0 head
`4895135deee7dbd782446dbfe25662181951afe0` and keeps the historical settlement
and 480-step comparison horizon. The historical founder geometry is retained,
but its initial reserve is zero so the former reserve-bearing physiology is not
silently carried into the M0 replay. No chemistry-core, uptake, transport,
resource inventory, death, or D-091 source was changed.

## Arms and provenance

| Arm | Protocol lineage | Inventory | Conversion |
| --- | --- | ---: | --- |
| A deprivation/no delivery | D-015 | 0 / 0 | on |
| B historical finite reference | D-015 | 3 / 3 | on |
| C historical high inventory | D-016 | 14.588954880632265 / 14.588954880632265 | on |
| D uptake-only control | D-016 | 14.588954880632265 / 14.588954880632265 | off |

Historical protocol values are read from the sealed
`experiments/generated/dcdev015/protocol.json` and
`experiments/generated/dcdev016/protocol.json`. Their reserve-bearing
physiology and the legacy D-016 accounting warning are historical context, not
selected M0 biology.

## Fresh local result

The 480-step deprivation replay reduced organized material by
`18.5240742455985`. During the matched comparison window:

| Arm | N/F consumed from finite world | A produced | Organized-material change | Closure residual |
| --- | ---: | ---: | ---: | ---: |
| A | 0 / 0 | 0 | `-13.3887965598302` | `5.68e-14` |
| B | `2.34938455157938` / `2.34938455157938` | `0.0433163220514441` | `-10.0932768691014` | `1.89e-13` |
| C | `11.4396891103526` / `11.4396891103526` | `0.945737637075` | `-9.20097842749806` | `2.34e-13` |
| D | `11.5964868177898` / `11.5964868177898` | 0 | `0` | `1.78e-14` |

All reserve flows are zero. All four arms remain physically and observer
viable at the 480-step endpoint. The high inventory arm improves the decline
relative to no delivery but does not restore organized material.

The no-resource continuation completed 2,057 accepted steps before the
existing observer death condition reported `starvation_collapse`. Organized
material declined from `131.806396226555` to `91.6513401125541`. No intervention
was applied.

## Classification

```text
DCDEV020M1R0_FINITE_RESOURCE_REQUALIFICATION_COMPLETE
current_m1_bottleneck = productive_allocation_or_replacement_limitation
```

This is a diagnostic classification, not authorization to modify production
allocation, degradation, transport, resource mass, or reserve physiology.
The result does not establish that a particular repair is correct. M1
production change, M2, recycling/salvage, and DC-DEV-021 remain unauthorized
pending exact-head CI and architect review.

Authoritative machine-readable evidence is in
`experiments/generated/dcdev020m1r0/`.
