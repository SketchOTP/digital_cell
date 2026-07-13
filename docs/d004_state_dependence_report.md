# D-004 State Dependence Report

**Status:** in progress (cross-state 100k runs executing)

## Initial-state classes

| class | source |
|-------|--------|
| A Fresh | `Simulation::new` + radial seed, sim_time=0 |
| B Aged D-002 | `baseline_seed_{2}/checkpoint_050000` fields only; D-003 candidate params |
| C Calibration endpoint | Fresh seed 2, iter-5 params, 20k substeps then continue |

## Known asymmetry

Calibration iterations each restart from **fresh seed 2**, not prior iteration endpoint. State C approximates post-calibration-window morphology, not a carried-forward aged state.

## Preliminary expectation

- Fresh + analytical params → Qφ ≈ 0.65 (matches legacy Stage B)
- Fresh + calibrated params → intermediate regime (corrected screen running)
- Aged D-002 fields + calibrated params → distinct from fresh (legacy chemistry history)

Full cross-state matrix: `experiments/generated/d004/cross_state/`

Results appended when `d004 audit` completes.
