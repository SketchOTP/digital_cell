# D-012 v2 Joint Balance (Stage E)

**Status:** Reference terminated as `INVALID_ARTIFACT`; solver entry gate closed.

## Protocol

- **Assay:** D-011 constrained-radius mechanics on v2 (φ fixed; C/N/F/W/A/M evolve; membrane transport active; virtual structure consumes A per v2 stoichiometry).
- **Radii:** R ∈ {14, 18, 22, 26, 30, 34}; expansion order center R=22 → neighbors → full grid when center promising.
- **Convergence:** three consecutive quasi-steady 10k-step windows; max 200k accepted substeps.
- **Pass gates:** all four Q ∈ [0.98, 1.02]; |g| ≤ 1e-4; C/A retention ≥ 0.80; localization ≥ 0.90; N/F influx > 0; W efflux > 0; material-equivalent accounting; restoring g_structure crossing.

## Calibration (v2-specific)

Progressive screens (0.75 / 1.00 / 1.25), **not** Cartesian product:

1. `k_d008_activation`
2. `k_d008_reproduction`
3. `k_membrane`
4. `k_d008_structure`

Initial rates estimated from constrained-radius ledgers at R=22 (not v1 Stage E rates).

## Solver bounds (Task 16)

- Global: 0.25×–4.00× reference
- Per-round: 0.67×–1.50×
- Max 4 rounds, 5 candidates
- Transport/diffusion/turnover/reservoirs/ICs/yields frozen during first solver sequence

## Artifacts

| Path | Purpose |
| --- | --- |
| `experiments/generated/d012/v2_stage_e_reference/` | Reference assay + calibration |
| `experiments/generated/d012/v2_sensitivity/` | Center sensitivity matrix |
| `experiments/generated/d012/v2_joint_candidates/` | Bounded solver validation |
| `experiments/generated/d012/v2_yield_candidates/` | Conditional yield branch |
| `experiments/generated/d012/v2_robust_overlap/` | ±2% rates; ±5% C/A/M IC |

## CLI

```bash
cd digital-protocell
./target/release/experiment-runner d012 stage-e-diagnostic --output ../experiments/generated/d012/v2_stage_e_reference
./target/release/experiment-runner d012 stage-e --output ../experiments/generated/d012/v2_stage_e_reference
./target/release/experiment-runner d012 stage-e-solver --output ../experiments/generated/d012/v2_joint_candidates
./target/release/experiment-runner d012 stage-e-robust --output ../experiments/generated/d012/v2_robust_overlap
```

## Result (diagnostic, 5k steps / 1k windows)

**Commit:** `7281c56` (reference artifact after path fix)  
**Classification:** `LongTransientUnresolved` (diagnostic horizon; not a definitive no-solution)  
**Joint balance:** not found at 5k  
**Center R=22 Q/g:** Q_φ≈0.17, Q_C≈0.92, Q_M≈0.40, Q_A≈0.33; all |g|≫1e-4 (g_φ≈−32)  
**Restoring radius:** fail — g_structure negative at R18/22/26 (−21/−32/−46), no sign crossing  
**Throughput:** active (N/F influx > 0, W efflux > 0)  
**Sensitivity:** rank 4, cond≈3.34 (valid for solver entry at short horizon)

Calibrated v2 rates (ledger-estimated + progressive screens):  
`k_activation≈0.079`, `k_rep≈0.012`, `k_membrane≈0.583`, `k_structure≈1.081`

## Governed reference terminal result

The authoritative reference launched from source commit `15f9f21` and binary
SHA-256 `b044ac4083838e9ea6e21c32e093f2025f25b10876b26f9b67c917995e28e77d`.
It ran from 2026-07-15 18:12:06 UTC to 19:57:21 UTC.

Terminal classification:

```text
INVALID_ARTIFACT
```

This is an execution-artifact failure, not a Stage E scientific conclusion.
The reference runner continued attempted chunks after `Simulation::step()`
returned false. It advanced its attempted-step counter and appended no-motion
windows even though accepted substeps stopped. Those zero-motion windows
incorrectly satisfied the quasi-steady window helper at R=22 and R=26.
Every radius records `clean_termination=false`.

| Radius | Accepted substeps | Simulated time | Reported windows | Clean |
| --- | ---: | ---: | ---: | --- |
| 18 | 188,324 | 470.7861 | 1 | no |
| 22 | 161,166 | 402.8940 | 3 | no |
| 26 | 150,715 | 376.7708 | 4 | no |

The required 10k/25k/50k/100k/150k/200k atomic checkpoints were not written.
Activation-potential accounting and an explicit biological/numerical
termination reason were also absent. The material-equivalent summary reports a
relative residual of `1.103539015623602e-11`, but that alone cannot validate
the reference.

The center's recorded values are preserved only as invalid-artifact
diagnostics:

| Component | Q | g |
| --- | ---: | ---: |
| Structure | 0.0342457 | -36.9610 |
| Catalyst | 0.4561964 | -0.554489 |
| Membrane | 0.2940055 | -0.769932 |
| Activated | 1.1260094 | -0.548245 |

Center C retention was `0.939302`, A retention `0.744125`, and membrane
localization `0.899265`; N/F influx and W efflux remained positive. These
values are ineligible for balance or nullcline classification.

No sensitivity, bounded-solver, yield, or robustness phase was started from
this reference. The solver entry gate remains closed.

Classification artifact:
`digital-protocell/experiments/generated/d012/v2_stage_e_reference/reference_terminal_classification.json`.


## D-013 follow-on (append-only)

The Stage E reference at `v2_stage_e_reference/` remains `INVALID_ARTIFACT` /
`scientific_usable=false` under tag `D-012-stage-e-reference-invalid`. D-013
repairs harness integrity and recovers a new governed reference under
`experiments/generated/d013/` without altering chemistry or rates. No D-012
scientific Stage E conclusion is authorized until a valid D-013 reference exists.

## D-014 append (2026-07-15)

Stage E remains blocked on a numerically trustworthy fresh R22 under the frozen
conservative-v2 candidate. D-014 repairs `TIMESTEP_FLOOR_FAILURE` / activation residual
without chemistry changes. Solver entry stays closed unless the fresh R22 is
`QUASI_STEADY_CONVERGED`.
