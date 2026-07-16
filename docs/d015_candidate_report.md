# D-015 candidate report

## Frozen organism (unchanged)

- Candidate hash: `9a452d3470be34ccf3bdd7d1397341b64617834e77131cf2899efb327728d626`
- Configuration hash: `87ff7e6e4bd479972c3a02b0de4e6bc94a949041860b32b230e5b28863bb5ad6`

## Environment variant

- Schema version: 2
- Repaired `waste_sink_inner_radius = 30.0` (R=22)
- N/F peripheral reservoir unchanged

## Diagnosis

Primary: `D015_BULK_DIFFUSION_BOTTLENECK`

## Pass gate status

Preflight under repaired environment: **PASS** (waste budget closed; no concentration abort).

Fresh R22 result: see `experiments/generated/d015/fresh_reference_r22/` (populated when run completes).

`D015_WASTE_THROUGHPUT_CLOSURE_PASS` requires fresh R22 to terminate without waste `UNBOUNDED_ACCUMULATION` with finite source/sink equilibrium below ceiling and accounting gates.

## D-012 solver

**CLOSED** until fresh repaired R22 is `VALID_GOVERNED_ARTIFACT` + `QUASI_STEADY_CONVERGED`.

## Final scientific conclusion

**`D015_INTERNAL_WASTE_PRODUCTION_IMBALANCE`**

Subsidiary: peripheral sink geometry was also limiting and was repaired (W-only `waste_sink_inner_radius=30`); clearance law CORRECT; export implementation OK but insufficient vs continuous production.

Fresh repaired R22: `UNBOUNDED_ACCUMULATION` at 162073 steps. D-012 solver remains **CLOSED**.
