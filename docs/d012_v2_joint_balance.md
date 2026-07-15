# D-012 v2 Joint Balance (Stage E)

**Status:** In progress — transport-coupled constrained-radius assay for `membrane_metabolism_v2_conservative`.

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

## Result

*(Updated after governed runs complete.)*
