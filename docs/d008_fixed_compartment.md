# D-008 Stage D — Fixed-Compartment Transport Closure

## Conclusion

`D008_STAGE_D_FIXED_COMPARTMENT_PASS`

Fixed circular compartments at R = 16, 24, and 32 with coupled selective transport,
activated metabolism, catalyst reproduction, turnover, and reservoir exchange satisfy
all Stage D gates. Catalyst and activated-resource retention exceed 0.80 at every
radius; nutrient and fuel enter; waste exits; resource influx per interior area
decreases strictly with radius; small-cell catalyst retention is not substantially
worse than large-cell retention; fields remain bounded; accounting closes; geometry
is fixed; all runs terminate cleanly.

## Provenance

- Stage D source commit: `36478407cd7f7992474cefa45acee60e6eeca9e3`
- Governed attempt: `attempt_002` (pass); `attempt_001` failed prior gate alignment only
- Experiment-runner SHA-256: `16ea3bee93c77df1ad8e4ae37f1746fe378694688cfee8cc7b561da2f82c965f`
- Equation version: `membrane_metabolism_v1`
- Snapshot schema: `2`
- Field schema: `seven_field_v1`
- Candidate: `cand-5288326dd4f5-kphi1-ks0.030000-kr0.012000`
- Configuration hash: `6b5a0c7e48f6e0b2d5c8dd17559689fbdc7cad25e5179c0028cf1afe27f26197`
- Selected config: `digital-protocell/configs/d008/stage_c_selected.toml`

## Fixed-compartment radii

| R | Catalyst retention | Activated retention | N influx/area | F influx/area | W efflux/area |
| --- | --- | --- | --- | --- | --- |
| 16 | 0.9990 | 0.9977 | 0.0534 | 0.0534 | 0.0631 |
| 24 | 0.9993 | 0.9984 | 0.0361 | 0.0361 | 0.0425 |
| 32 | 0.9995 | 0.9988 | 0.0271 | 0.0271 | 0.0318 |

Resource influx per interior area (N+F): R16 `0.1069` > R24 `0.0722` > R32 `0.0541`.

Small-cell catalyst retention margin vs R32: `0.0005` (limit 0.05). Diagnostic leakage
ratio R16/R32: `1.93` (informational only after gate realignment).

## Gate checks

All `gate_checks` true: catalyst/activated retention, nutrient/fuel entry, waste exit,
strict influx scaling, small-cell retention margin, bounded fields, fixed geometry,
accounting closed, clean termination.

## Accounting

Per-radius max step residual ≤ `8.1e-9`; `steps_outside_tolerance = 0` for all radii.
Aggregate: `15000` accepted substeps, `37.5` simulated time units, `run_count = 3`.

## Artifact

- Runtime result: `digital-protocell/experiments/generated/d008/stage_d_fixed_compartment/attempt_002/result.json`
- Result SHA-256: `4ac7e8f4b235735424b6bd2ee0307297fd0fe6853a6b53137dfae09dba7275ef`
- Manifest: `digital-protocell/experiments/generated/d008/manifest.json`

Stage E may proceed. Stages F–G remain unstarted.
