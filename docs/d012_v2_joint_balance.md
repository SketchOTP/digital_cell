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

## Full horizon (200k) — in progress

Started after diagnostic; logs: `/tmp/d012_stage_e_full.log`, status: `/tmp/d012_stage_e_full.status`  
Artifacts resume under `experiments/generated/d012/v2_stage_e_reference/` with horizon-scoped job IDs.

*(Full grid + solver/robust evidence updated when long runs complete.)*
