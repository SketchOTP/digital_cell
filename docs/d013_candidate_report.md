# D-013 Candidate Report

## Status

Harness repair for Stage E governed reference recovery under frozen
`membrane_metabolism_v2_conservative` candidate.

## Frozen identity

| Field | Value |
| --- | --- |
| equation version | `membrane_metabolism_v2_conservative` |
| stoichiometric schema | 2 |
| field schema | seven-field |
| candidate hash | `9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626` |
| configuration hash | `87ff7e6e4bd479972c3a02b0de4e6bc94a949041860b32b230e5b28863bb5ad6` |

Rates, yields, transport, diffusion, turnover, reservoirs, ICs, radii, and
thresholds remain frozen. D-013 changes only measurement/harness integrity.

## Invalid prior reference

Preserved in place under
`digital-protocell/experiments/generated/d012/v2_stage_e_reference/` with
`scientific_usable=false` and tag `D-012-stage-e-reference-invalid`.

See `docs/d013_invalid_reference_postmortem.md`.

## Harness repairs

- Accepted-step authority for windows/convergence/time
- Atomic checkpoint thresholds with lossless field resume
- Activation-potential ledger on every governed result
- Explicit termination reasons and rejection-stall numerical failure
- Artifact validator gate before scientific admission

## Artifacts

```text
digital-protocell/experiments/generated/d013/
```

## Conclusion

`D013_REFERENCE_NUMERICAL_FAILURE`

Preflight passed. The frozen R22 reference produced a `VALID_GOVERNED_ARTIFACT`
with accepted-step windows, atomic checkpoints through 150k, activation-potential
and material accounting, and clean termination
`TIMESTEP_FLOOR_FAILURE` / `NUMERICAL_FAILURE` at 161,166 accepted substeps.

R18/R26 were not run. The four-rate solver remains closed. Repair the numerical
timestep-floor cause before any scientific Stage E pass claim or solver entry.
