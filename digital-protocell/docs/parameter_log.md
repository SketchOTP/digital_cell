# Parameter Log

## Baseline (D-002 frozen candidate)

Frozen in `configs/phase1_candidate.toml` for acceptance runs (immutable during first acceptance).

Commit savepoint: `2123435` (tag `D-001-baseline`).

Key tuned values (unchanged from D-001):

| Parameter | Value |
|-----------|-------|
| k_structure | 0.030 |
| k_structure_decay | 0.025 |
| k_catalyst_decay_inside | 0.005 |
| k_catalyst_decay_outside | 0.050 |

Grid: 192×192, dish radius 88, seed r0 24, max dt 0.0025, seed 1.


## Sweep

Deterministic staged sweep over:

`k_rep`, `k_structure`, `k_structure_decay`, `k_catalyst_decay_inside`, `d_c_inside`, `mobility_m`, `kappa`

Factors: 0.5×, 0.75×, 1.0×, 1.25×, 1.5× baseline.

Results written to `experiments/generated/sweep/sweep_results.json`.

## Selection policy

Final parameters remain baseline until sweep evidence supports change. Document any change here with experiment ID and outcome class.
