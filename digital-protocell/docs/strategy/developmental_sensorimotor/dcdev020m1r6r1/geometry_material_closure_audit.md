# DC-DEV-020-M1-R6-R1 Geometry/Material Closure Audit

This is an observer-only audit entered at `adea13fafa1f2a85e521a44b5d77249820d107bd` after R6 was accepted as a valid invalidation. It replays the unchanged packaged order:

```text
S0 step entry -> S1 finite uptake -> S2 reactions -> S3 mechanics -> S4 remesh -> S5 rebond
```

No production equation, parameter, transport law, mechanics law, remesh rule, rebond rule, death rule, or selector is changed. Dense stage ledgers are canonical at:

`\\RPI5\RPI5SharedDrive\100_ACTIVE\Projects\DIGITAL_CELL\evidence\dcdev020m1r6r1\dense\stage_ledger.jsonl`

## Authority and parity

- R6 authority: `9ff1bba4a48caf582e4598b4030d746e4360a61b`
- R6 exact-head CI: `32673647585` SUCCESS
- R6-R1 entry: `adea13fafa1f2a85e521a44b5d77249820d107bd`
- Plain and instrumented trajectory hashes: `be91ed02266a0078`
- Final mesh hash: `e4c4dd4ff2e443d8`
- Stored R6 checkpoints at 0, 480, 1000, 2000, 3466, 4000, 6000, 6931, and 8000 all agree.

The observer path has exact trajectory parity with a plain replay. Its additional calls are reads, clones, and serialization only.

## Stage attribution

The 8,000-step fed R6 closure residual is:

```text
observed signed residual:              -125.27019370631045
uptake residual:                         -4.0351749719391705e-13
reaction residual:                        7.460698725481052e-13
mechanics residual:                    -122.35109442317224
remesh residual:                         -2.919099283138749
rebond residual:                          0
reconstructed residual:                -125.27019370631065
unexplained residual:                     1.9895196601282805e-13
```

For mechanics and remesh, the sum of the six interior species' fixed-concentration area effects is:

```text
geometry residual:                    -125.27019370631099
fixed-concentration area effect:      -125.27019370631108
```

The mechanics-only entry-state isolation changes strict material by `-0.40689615197109674` while changing area from `70.8275261895172` to `70.52125138656744`; no uptake or reaction occurs. The first existing lawful remesh fixture changes strict material by `-1.2188082055896814` while changing area from `16.395992230363394` to `15.837312342923823`; no uptake, reaction, or mechanics occurs. No successful rebond fixture was available without manufacturing a state, so rebond isolation is `NOT_EXERCISED`.

The first permanent resource delivery loss is step `612`; the first geometry change is step `1`, and the contact-loss chronology follows geometry (`YES`). The reservoir remains externally conserved; the defect is internal geometry/concentration bookkeeping.

## Classification

```text
M1_RUNTIME_GEOMETRY_MASS_COUPLING_CONFIRMED
```

This confirms the R6 closure attribution only. It does not authorize a repair, production selection change, recycling/salvage, M2, or DC-DEV-021. The next action is Architect review.
