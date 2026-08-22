# DC-DEV-020-M1-R1 — Capacity decomposition

This is an observer-only four-arm decomposition of the exact accepted M1-R0
high-inventory finite-feed trajectory. The entry state is reproduced through
the accepted M1-R0 settlement and 480-step deprivation path; every arm then
uses the same `14.588954880632265 / 14.588954880632265` finite resource,
geometry, transport, ConservativeV2 contract, reserve-OFF state, and 480-step
horizon.

## Shadows

| Arm | Observer-only difference |
| --- | --- |
| BASE | Exact ordinary M1-R0 source and reactions |
| SOURCE_CAPACITY_UB | Convert immediately available paired internal N/F using the existing `N + F -> A + W` coefficients before ordinary reactions |
| CATALYST_INVESTMENT_OFF | Set only the current shadow's `k_c_prod` to zero; existing C and C turnover remain active |
| COMBINED | Both bounded shadows together |

The source upper bound never injects A, uses no future resource, and cannot
consume more paired N/F than is immediately present. It is a capacity bound,
not a proposed production law. Catalyst deferral is an acute allocation bound,
not recycling, salvage, or a production change.

## Result

The exact baseline reproduces the accepted M1-R0 high-inventory values. Source
capacity improves the 480-step organized-material decline, but does not make it
nonnegative. Catalyst-investment deferral alone likewise does not make the
decline nonnegative. The combined upper-bound shadow also remains negative.

```text
M1_SOURCE_AND_ALLOCATION_INSUFFICIENT
```

This classification means the two tested acute capacity bounds are jointly
insufficient on the current baseline. It does not select a production repair;
conversion throughput, productive allocation, and deeper maintenance/degradation
effects remain causally unresolved.

All world↔organism and internal material closures pass. A nonnegative
480-step shadow, had one occurred, would not establish sustained M1
homeostasis. Production chemistry, ConservativeV2, D-091, uptake, transport,
resource quantity, degradation, recycling, salvage, M2, and DC-DEV-021 remain
unchanged or unauthorized.

Authoritative compact evidence is in
`experiments/generated/dcdev020m1r1/`.
