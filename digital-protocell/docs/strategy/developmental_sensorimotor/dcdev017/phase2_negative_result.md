# DC-DEV-017 Phase 2 negative result

The Phase 1 intrinsic-timescale challenge did not establish homeostasis, so
the one authorized opt-in repair was evaluated. No Phase 3, 4, or 5 work was
started.

## Bounded repair

`chemistry-core/src/demand_coupled_activation.rs` adds
`dcdev017_demand_coupled_activation_v1`. It multiplies only the existing
N+F -> A extent:

`multiplier = clamp(1 + (8.58379474604017 - 1) * demand / demand_reference, 1, 8.58379474604017)`

where `demand = K_low/(K_low+A) * R/(K_R+R)`. The feature is opt-in and
default-off; invalid reference values fail closed to multiplier `1.0`. It
does not write A, R, N, or F directly and uses the frozen DC-DEV-016 gain.

## Phase 2 arms

All arms used the existing finite spatial N/F inventory API, the same
deprived starting mesh, 4,000 accepted steps, and `MechParams.dt = 0.02`.

| Arm | Final E_stored | Final A | Final R | N delivered | N consumed | Q4 slope |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| P2-A resource-free control | 8.940681207197107 | 0.06943282142358279 | 0.05679890920716129 | 0 | 0 | -0.39364797274927393 |
| P2-B current resource | 17.01999194379292 | 0.13365324890887817 | 0.1066486910212497 | 3.0000000000000018 | 2.3806208806236144 | -0.38121564421048576 |
| P2-C derived resource | 20.698866355495912 | 0.15229240904948926 | 0.13995083971386285 | 14.588954880632254 | 13.878608445954079 | -0.4261371761734976 |
| P2-D sustained clamp | 72.35005237522168 | 0.6162639547523513 | 0.40523229490672286 | 0 | 128.32703718179275 | -0.04512837251548092 |

The P2-C derived-resource arm remains below the deprived starting
`E_stored = 60.82781514212436`. The P2-D Q4 slope remains more negative than
the allowed `1% * abs(P1-A Q4 slope) = 0.003936479727492739`. The feature-off
trajectory is exactly identical to the legacy reference, and the resource-
free enabled arm produces no additional A.

## Classification

`DCDEV017_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED`

This is a valid negative result for the single bounded repair, not a license
for another metabolic design or a parameter search. The critical stop applies:
no reversible foraging, behavior, seed campaign, or DC-DEV-018 work follows.
