# D-004 Candidate Provenance Audit

**Status:** complete (pipeline defect confirmed)  
**Conclusion:** `D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT`

## Root cause

Stage B short screen did **not** evaluate any final calibrated candidate. `experiment-runner` `D003 Pipeline` and `D003 Screen` loaded **analytical D-002 median estimates** for `K_phi=1.0`:

```text
k_structure ≈ 0.09241125380438656
k_rep       ≈ 0.026147379777114742
```

Final calibrated `K_phi=1.0` candidate (iteration 6, `calibration_result.json`):

```text
k_structure = 0.14145030659271887
k_rep       = 0.014489097664708522
candidate_hash = c37d597b0f8351ea5e4b8f385e63bca9fed5d72cb0e2f22a395bb21d32c4e184
```

## Stage B classification (seeds 1–3)

| seed | match class | Qφ | QC |
|-----:|-------------|---:|---:|
| 1 | `MATCH_ANALYTICAL_INITIAL_ESTIMATE` | 0.663 | 1.812 |
| 2 | `MATCH_ANALYTICAL_INITIAL_ESTIMATE` | 0.651 | 1.752 |
| 3 | `MATCH_ANALYTICAL_INITIAL_ESTIMATE` | 0.651 | 1.752 |

Legacy short-screen artifacts lack `candidate_hash`; reconstruction from pipeline source code confirms analytical params.

## Final calibrated candidates (iteration 6)

| K_phi | k_structure | k_rep | candidate_hash (prefix) |
|------:|------------:|------:|-------------------------|
| 0.5 | 0.20561790002463595 | 0.014467942127568812 | 9f3fa9cc32b2… |
| 1.0 | 0.14145030659271887 | 0.014489097664708522 | c37d597b0f83… |
| 2.0 | 0.10877067981213878 | 0.014507603272504265 | 7cc98b12fca0… |

Configs: `digital-protocell/configs/d004/final_kphi_{0_5,1_0,2_0}.toml`

## Calibration replay

All three branches: iteration 5 metrics reproduce stored JSON within **1×10⁻⁶** relative tolerance.

## Calibration stopping (iteration cap = 6)

| K_phi | stop class | Qφ at iter 5 | slope_φ at iter 5 |
|------:|------------|-------------:|------------------:|
| 0.5 | STILL_IMPROVING | 0.983 | −3.79×10⁻⁴ |
| 1.0 | STILL_IMPROVING | 0.983 | −3.76×10⁻⁴ |
| 2.0 | STILL_IMPROVING | 0.983 | −3.73×10⁻⁴ |

Six iterations inherited from D-003 directive; not scientifically converged on slope gate.

## D-003_FAIL status

**Invalidated by pipeline defect.** Original `D003_FAIL` reflected screening the wrong candidate, not failure of calibrated chemistry.

**Revised:** `D003_RESULT_UNRESOLVED_PENDING_PIPELINE_AUDIT` until corrected Stage B completes.

## Fix applied

- `CandidateIdentity` + canonical `candidate_hash`
- `short_screen` requires `CandidateIdentity`; records hash on every artifact
- Pipeline Stage B now loads `calibration_result.json` final params for `K_phi=1.0`
