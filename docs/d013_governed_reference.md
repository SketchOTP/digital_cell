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
