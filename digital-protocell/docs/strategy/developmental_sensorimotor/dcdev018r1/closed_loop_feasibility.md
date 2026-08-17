# DC-DEV-018-R1 closed-loop source-demand feasibility

## Authority and boundary

This observer-only audit starts exactly at `1e242f28152797b512e25cd56c7b718e45d6ca97`, the accepted clean DC-DEV-016 head. It does not import the failed DC-DEV-018 homeostat or any persistent controller state. The ordinary production reaction path remains gain `1.0`; the assay-facing entry point runs the same canonical reaction sequence on a clone with only the existing N/F-to-A activation extent multiplied.

Frozen values are `dt=0.02`, 5,000 settlement steps, 480 deprivation steps, 4,000 storage steps, 480 finite-resource steps, sustained matched precursor `N=F=0.1476710565778127`, and finite inventory `N=F=14.588954880632265`.

## Entry parity

The clean reconstruction reproduced settled body hash `c985c08ab226a061`, deprived body hash `990c1abe7e178d30`, target `E_stored=77.91027880846893`, deprived `E_stored=60.82781514212436`, and the 4,000-step gain-1 legacy trajectory hash `de16bdafafe3b662`. The gain-1 counterfactual recovered ordinary `reactions_step` mesh and ledger hashes exactly.

## Source law

From the live implementation:

```text
B = k_act * q_catalyst(C) * g_harvest * N * F * dt * area
J_source(g) = min(g * B, N * area, F * area)
g_sat = min(N * area, F * area) / B, when B > epsilon
```

For the response envelope, `u={0,.25,.50,.75,1}` maps to `g=1+u*(g_sat-1)`. Every point is a full-reaction clone; downstream sinks are not held fixed. A/R transfer is excluded from irreversible demand. The recorded sink components are A decay, A-to-structure, A-to-catalyst, A-to-membrane, A-to-R, R-to-A, R-to-waste, and R-to-structure.

## Result

The ideal source-only trajectory reaches the target within `5.3e-11` material units at step 4,000, remains alive and nonnegative, and never requires gain above substrate saturation. Its maximum required gain is `6.97512279078733`; maximum required/g_sat is `0.00372132512187368`.

The finite-resource source-saturated upper bound delivers and consumes `12.7655055551733` N and F with zero resource accounting error, but ends at `E_stored=59.1464166923814`, below the deprived `60.82781514212436`. Thus finite delivery is insufficient even when each accepted reaction step uses the maximum gain permitted by the existing matched precursor.

Increasing source materially increases state-dependent sink demand. Median irreversible demand at `u=1` versus `u=0` is:

| Window | gain-1 demand | saturated demand | saturated delta E median |
|---|---:|---:|---:|
| Q1 | 0.0168981940475050 | 0.0395053379651957 | 10.4196702892335 |
| Q2 | 0.0148748960259272 | 0.0364643750404453 | 10.4227112521583 |
| Q3 | 0.0133759412253106 | 0.0342715327530128 | 10.4249040944457 |
| Q4 | 0.0111476664519663 | 0.0307147761480009 | 10.4284608510507 |

The source-response is monotonic over the registered states, but the old gain-1 demand/source ratio is not a valid closed-loop feasibility criterion because sinks rise with source.

The committed DC-DEV-018 artifacts contain the observed M4 capacity (`1.07088866298817`) but no per-step error trace. The reachability report therefore records the exact capacity implied by the committed observation and `k_integral`, but does not claim an exact trace reconstruction.

## Classification

`DCDEV018R1_SOURCE_SIDE_HOMEOSTASIS_FEASIBLE_FINITE_RESOURCE_LIMIT_CONFIRMED`

This is a feasibility statement, not a controller qualification. It says the existing source law can restore under sustained matched precursor, while the existing finite uptake/inventory contract cannot. No replacement controller, source tuning, sink modification, behavior, or DC-DEV-019 work is authorized by this result.
