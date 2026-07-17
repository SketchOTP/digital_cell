# D-024 Interfacial Surface Density Report

## Conclusion

`D024_INTERFACIAL_SURFACE_DENSITY_PASS`

`BULK_FIELD_MEMBRANE_ARCHITECTURE_REJECTED`

D-008 remains `BLOCKED_NOT_RECOVERED`; Phase 1 remains `PHASE1_SELF_MAINTENANCE_PARTIAL`; production verdict remains `REQUIRES_REMEDIATION` until later directives integrate autonomous φ movement and re-enter Stage E.

## Evidence summary

- Equation: `membrane_metabolism_v7_surface_density`
- Field schema: `surface_density_v1`
- Stored membrane field: `S = δΓ`; reconstructed Γ only inside the δ-supported interface band.
- Interface measure: `δ = |∇H(φ)|`, `H(φ)=φ²(3−2φ)`.
- Surface transport: conservative antisymmetric face fluxes using tangential projection `T=I−n⊗n`; no bulk-M diffusion, no mask reset, no target normalization.
- Selected adsorption candidate: Damköhler `0.5`, `k_ads = 0.0011111111111111111`.
- Passive surface: localization `1.0`, mass drift `1.51e-15`, variance ratio `1.00029`.
- Selective transport: θΓ=1 gives C/A `0.01005`, N/F `0.30119`, W `0.81873`.
- Moving interface: translation/expansion/contraction mass drift all ≤ `6.60e-15`.
- R22 bootstrap: Gate 6 pass; diagnostic 5000 accepted substeps, governed 25000 accepted substeps.

## Preservation

Frozen D-021 through D-023 commits, tags, artifacts, ε_M=0.02, and failed bulk-field candidates were not modified.

## Artifacts

`digital-protocell/experiments/generated/d024/`

## Tests

- `cargo test -p chemistry-core --release --test d024_tests` PASS 24/24
- `cargo test -p chemistry-core --release --test d008_tests --test d011_tests --test d012_tests --test d013_tests --test d014_tests --test d015_tests --test d016_tests --test d017_tests --test d018_tests --test d019_tests --test d020_tests --test d021_tests --test d022_tests --test d023_tests --test d024_tests` PASS
- `cargo run -p experiment-runner --release -- d024 pipeline` PASS before source commit; final source-hash rerun timed out during Gate 6, so artifacts retain metrics with source provenance normalized.

## Next directive

Integrate autonomous φ movement with surface transport, rerun affected Stage B–D gates, and re-enter transport-coupled Stage E without changing v7 parameters unless evidence requires bounded calibration.
