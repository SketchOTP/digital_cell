# Parameter Log

## Baseline (initial)

See `crates/chemistry-core/src/config.rs` `SimParams::default()`.

### Tuned from directive defaults (2026-07-12)

| Parameter | Directive | Final | Reason |
|-----------|-----------|-------|--------|
| k_structure | 0.035 | 0.030 | balance synthesis/decay |
| k_structure_decay | 0.006 | 0.025 | measurable turnover |
| k_catalyst_decay_inside | 0.0008 | 0.005 | knockout/starvation response |
| k_catalyst_decay_outside | 0.025 | 0.050 | catalyst escape penalty |

Documented in `docs/phase1_report.md` deviations section.


## Sweep

Deterministic staged sweep over:

`k_rep`, `k_structure`, `k_structure_decay`, `k_catalyst_decay_inside`, `d_c_inside`, `mobility_m`, `kappa`

Factors: 0.5×, 0.75×, 1.0×, 1.25×, 1.5× baseline.

Results written to `experiments/generated/sweep/sweep_results.json`.

## Selection policy

Final parameters remain baseline until sweep evidence supports change. Document any change here with experiment ID and outcome class.
