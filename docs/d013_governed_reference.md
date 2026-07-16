# D-013 Governed Reference

Frozen candidate (unchanged by D-013):

| Field | Value |
| --- | --- |
| equation version | `membrane_metabolism_v2_conservative` |
| stoichiometric schema | 2 |
| field schema | seven-field |
| candidate hash | `9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626` |
| configuration hash | `87ff7e6e4bd479972c3a02b0de4e6bc94a949041860b32b230e5b28863bb5ad6` |

Order:

1. Preflight PASS
2. Center R=22 up to 200,000 accepted substeps
3. Neighbors R=18 and R=26 only if R22 is valid and quasi-steady

Solver entry remains closed until a valid quasi-steady R22 governed artifact exists with complete material and activation accounting.

## D-014 numerical repair note (2026-07-15)

D-013 R22 remains the historical numerical-failure artifact (`TIMESTEP_FLOOR_FAILURE` at
accepted substep 161166). D-014 preserved it under
`experiments/generated/d014/preservation/` and did **not** overwrite
`experiments/generated/d013/reference_r22/`.

Cause of the floor: `FIELD_BOUND_VALIDATION` on `waste_next` at `CONC_SAFETY_LIMIT`
(overshoot ≈ 1×10⁻¹⁰ at terminal cascade). See `docs/d014_timestep_floor_postmortem.md`.

Fresh scientific trajectory after numerical repair lives under
`experiments/generated/d014/fresh_reference_r22/`.
