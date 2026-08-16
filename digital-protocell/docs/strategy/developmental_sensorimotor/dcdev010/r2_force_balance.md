# DC-DEV-010-R2: baseline force-balance audit

DC-DEV-010-R2 is an observer-only diagnostic from
`16503a73d91f2c1e239206b73e69af1fee0fcf60`. It does not repair mechanics,
change the substrate, tune parameters, or add locomotion capability.

The audit runs the exact seeded 24-vertex mesh for exactly 5,000 accepted
steps in three passive arms:

1. legacy mechanics with no substrate;
2. the existing isotropic resistance control;
3. the frozen directional substrate (`[1.0, 0.0]`, ratios `0.25`, `0.75`,
   `0.50`, maximum reaction `0.45`).

Contractility, regulatory/plasticity advancement, chemistry, resources,
reserve spending, growth, remeshing, fission, obstacles, and contact are
disabled. The existing `compute_forces` and mechanics-step functions remain
the trajectory authority.

## Observer parity

The assay reconstructs the existing spring, pressure, and bending terms only
for observation. It records per-vertex vectors, organism-wide vector sums,
maximum norms, median norms, motion, shape change, and substrate work in the
three fixed windows `0–999`, `2000–2999`, and `4000–4999`.

The reconstructed total matched `compute_forces` with maximum error
`1.2412670766236366e-16` against a `1e-12` tolerance. The instrumented legacy
trajectory matched an uninstrumented legacy trajectory hash at every accepted
step.

## Result

The legacy and isotropic controls reached the preserved R1 diagnostic
references in their late windows. The directional arm did not: it retained a
late material-centroid step of `7.300374194136508e-11`, maximum attempted
velocity `4.6144874144005144e-8`, and maximum net internal-force norm
`4.6144874144005144e-8`.

The standalone spring, pressure, and bending forces remain much larger than
the net residual and cancel component-wise. The late residual is therefore
classified as an unresolved interaction, with bending the largest standalone
component; no force term is repaired or altered by R2.

Required conclusion:

`DCDEV010R2_BASELINE_FORCE_BALANCE_AUDIT_COMPLETE`

Gate-6 classification:

`DCDEV010R2_DIRECTIONAL_SUBSTRATE_SPECIFIC_RESIDUAL_CONFIRMED`

Evidence is under `experiments/generated/dcdev010r2/`. DC-DEV-010-R3 and
DC-DEV-011 remain blocked.

`NEXT_EXECUTION_STARTED:false`
