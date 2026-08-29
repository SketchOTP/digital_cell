# DC-DEV-020-R7 on-policy zero-drift audit

## Authority and scope

R7 starts at architect-accepted R6 head `f01b716d9051c9f0114f3c5c0d1b123e2df037cf`. Production biology remains rooted in clean DC-DEV-016 head `1e242f28152797b512e25cd56c7b718e45d6ca97`. R6 is preserved as `DCDEV020R6_ACCEPTED_NEGATIVE`, and its exact N/F power-law restoration route is closed as `NF_POWER_LAW_RESTORATION_ROUTE_CLOSED`. This does not close N/F information sufficiency.

The audit is observer-only. It does not refit R6, identify another law, add A to runtime biology, alter resources or sinks, integrate a controller, change behavior, or begin DC-DEV-021.

## Exact R6 reproduction

The selected balanced patch contains `N=F=19.878372106390554` and runs for 480 accepted steps after the frozen settlement and deprivation phases. Every state is captured after passive uptake and before reaction execution.

The scientific endpoint reproduces the accepted R6 result within `1e-10`:

| Quantity | Result |
|---|---:|
| Trajectory hash | `97010613dc36e447` |
| Final `E_stored` | `60.06203101178377` |
| Final A | `0.3423623895976825` |
| Final R | `0.5056416879564652` |
| Final N/F | `0.10185789865759344` |
| Final C | `0.7722488011667238` |

The explicit endpoint snapshot is the portable scientific parity authority. The committed Windows trajectory hash is retained as an additional realization seal because R5 established that non-authoritative mechanics internals can differ bytewise across operating systems.

## Physical on-policy roots

The frozen R5 procedure evaluates each source response at `S/S_sat = 0, 0.25, 0.50, 0.75, 1.00` and then uses bracketed bisection through the first monotone crossing.

| Quantity | Result |
|---|---:|
| States audited | 480 |
| Finite zero-drift roots | 480 |
| Capacity-insufficient states | 0 |
| Non-monotonic states | 0 |
| Maximum relative root interval | `9.53674316472667e-7` |
| Maximum root accounting residual | `1.660910992073994e-14` |
| Median `S_zero/S_sat` | `0.005191326141357422` |

The dense ledger is external and sealed at SHA-256 `abdaea6d075c700e36d14d369dba62982f4a65cea47d2d1f162b5dfe8afa59f8`.

## R6 source against on-policy need

| Measure | All | Early | Middle | Late |
|---|---:|---:|---:|---:|
| Relative RMSE | `0.08817182658022692` | `0.04048140514611145` | `0.08554071356367664` | `0.11986181697227127` |
| p95 absolute relative error | `0.12975721113594293` | `0.060920259295746286` | `0.10199551537157936` | `0.1322058087993389` |
| Median `S_R6/S_zero` | `0.9147362580292289` | `0.962142912389411` | `0.9147362580292289` | `0.8800795889615289` |
| Materially below root | `0.37083333333333335` | `0` | `0.1125` | `1.0` |

R6 is below the physical root on all 480 states, although its aggregate RMSE and p95 remain inside the frozen R5 prediction limits. The sum of actual local drifts is exactly `-0.7657841303405846`, equal to the observed endpoint change with zero closure residual. This directly accounts for the R6 stored-material loss.

## Frozen observer replay and support

The R5 observers are unchanged: P0/P1/P2 training, 16 unweighted Euclidean neighbors, training-only min-max scaling, and target `S_zero/(q_c*area*dt)`. No R6 state is used for fitting.

| Observer | RMSE | p95 | Ambiguity | Verdict |
|---|---:|---:|---:|---|
| NF | `0.09065623148766569` | `0.11892915522003528` | `0.26505161065124994` | fails ambiguity limit `0.25` |
| NFA | `0.017325292104497104` | `0.04444189891888537` | `0.010315383793568476` | passes |

R6 remains extremely close to the R5 training support in NF space: median nearest distance `0.00018078006908544842`, compared with P3/P4 holdout median `0.23138383306023516`. In NFA space its median nearest distance is `0.09226203046986303`, still below the accepted holdout median `0.24618462092372878`. No binary OOD threshold was introduced.

The NF failure is therefore not simple geometric remoteness. Its maximum local target spread crosses the frozen ambiguity limit on the R6-induced trajectory, while A disambiguates the same states under the unchanged diagnostic.

## Exact-root oracle

The oracle solves a fresh physical root after each uptake and has no memory or target state. It is diagnostic only.

| Quantity | Result |
|---|---:|
| Initial `E_stored` | `60.82781514212436` |
| Final `E_stored` | `60.82857175353983` |
| Final A | `0.35264288496585117` |
| Final R | `0.5061838315112626` |
| N/F consumed | `9.473769689679441` |
| Accelerated-decay steps | 0 |
| Maximum accounting residual | `1.9808590531744663e-14` |

The `0.00075661141547` stored-material increase is only about `0.00124%` of the deprived store and is below the frozen R5 1% material-state criterion. The oracle does move the A/R state closer to the settled state (`0.2014186190010067` to `0.1746871283784084`), but does not create meaningful restorative stored-material gain.

## Classification

`DCDEV020R7_NFA_COORDINATE_REQUIRED_ON_POLICY`

NF fails the frozen on-policy observer only on ambiguity, while NFA passes all unchanged limits. R7 does not construct an NFA production law. The exact N/F power-law restoration route remains closed, but no wider coordinate or mechanism is authorized without architect review.

## Reproduction

From `digital-protocell/`, with the accepted R5 ledger available:

```text
DCDEV020R5_EXTERNAL_LEDGER=target/dcdev020r5-statewise-ledger.json \
DCDEV020R7_OUTPUT_ROOT=target/dcdev020r7-reproduction \
DCDEV020R7_EXTERNAL_LEDGER=target/dcdev020r7-on-policy-ledger.json \
DCDEV020R7_EXTERNAL_LOCATION=/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/dcdev020r7/3ddae9ea3c954431c8b3ae2ecbf2d6fc94278e56/on_policy_root_ledger.json \
DCDEV020R7_LEDGER_SHA256=abdaea6d075c700e36d14d369dba62982f4a65cea47d2d1f162b5dfe8afa59f8 \
DCDEV020R7_SOURCE_COMMIT=3ddae9ea3c954431c8b3ae2ecbf2d6fc94278e56 \
cargo +1.89.0 run -p regulatory-core --example dcdev020r7_on_policy_zero_drift_audit --release --quiet
```

Piroddi 2008 is used only as `ADAPTABLE_VALIDATION_METHOD` support for free-running validation. Ross, Gordon, and Bagnell 2011 is `REFERENCE_ONLY` for the on-policy distribution-shift analogy. No external algorithm, model, constant, or architecture was imported.
