# DC-DEV-019-R1 Phase 0–3 Requalification

This is an observer-only requalification of the unchanged
`digital_cell_metabolic_acquisition_homeostat_v1`. It starts at
`59633ebcc37c936e2d04ca5d53477129ab1dca13` and does not modify the production
homeostat, chemistry, resource, reserve, mechanics, or behavioral paths.

## Candidate seal

- Production homeostat blob: `1172bf792292cfb50269f3f19c01034f446f1af6`
- `E_target`: `77.91027880846893`
- `E_deprived`: `60.82781514212436`
- `tau`: `80.0`
- `k_h`: `0.11402084665627849`
- `G_source_max`: `6.97512279078733`
- `G_transport_max`: `1.0`
- Selected N/F mass: `19.878372106390554`
- Resource center/radius: `[4.8, 0.0]` / `1.5`

## Gate 0

The settled material hash was `c985c08ab226a061`, the legacy deprived
material hash was `990c1abe7e178d30`, and the continuous-state deprivation
reproduced the same deprived hash. The accepted Phase-1 selected-mass result
(`61.68434818478833`), original DC-DEV-019 M2 result
(`55.84948101858201`), and original M3 result (`76.82632823803954`) were all
reproduced within the recorded tolerance. Gate 0 passed.

## Gate 1 continuous finite refeeding

The enabled homeostat remained alive during 480-step deprivation. It started
at `h=0.0` and ended at `h=0.12874703683936634`; the deprivation error
trajectory is recorded by hash in the compact qualification artifact. No
N/F source was present and no material was created during deprivation.

The carried-state finite refeed used the same deprived body, chemistry, A/R,
N/F ecology, and homeostat without reset. It delivered
`15.590364956434858` N and F with zero conservation error, but stored material
fell from `60.82781514212436` to `56.54155735320212`. The primary Gate-1
restoration criterion therefore failed.

The reset-state matched control reproduced the original M2 final
`E_stored=55.84948101858201`. The source-saturated observer reached
`61.68434818478833`, confirming that the finite ecology can raise stored
material under its already-accepted upper-bound observation. These controls
separate the carried-state homeostat failure from finite-resource sufficiency.

## Stop disposition

Classification:

`DCDEV019R1_CONTINUOUS_COORDINATED_METABOLIC_HOMEOSTASIS_NOT_ESTABLISHED`

Gate 1 failed, so the directive-required stop was applied. The 8,000-step
Phase 2 settling/unwinding assay and Phases 3–6 were not executed. No tuning,
alternate controller, resource change, horizon extension, behavior, foraging,
or DC-DEV-020 work was started.

Evidence is compact and records reproduction commands, artifact hashes, code
SHA, and the external raw-output location. Dense step ledgers are not stored
in Git.
