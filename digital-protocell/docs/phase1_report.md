# Phase 1 Report

## Scientific conclusion

**PHASE1_SELF_MAINTENANCE_PARTIAL**

D-002 acceptance infrastructure complete. All five 250,000-substep baseline replicates completed with mass accounting within tolerance, but **0/5 seeds pass** D-002 criteria: final classification remains `Transient`, turnover ratios remain below 1.0, and long-horizon interventions were not executed. See `docs/phase1_acceptance_report.md`.

## Parameters (final vs directive)

| Parameter | Directive | Final |
|-----------|-----------|-------|
| k_structure | 0.035 | 0.030 |
| k_structure_decay | 0.006 | 0.025 |
| k_catalyst_decay_inside | 0.0008 | 0.005 |
| k_catalyst_decay_outside | 0.025 | 0.050 |

All other baseline values match directive §11.

## Experiment table (seed=1, 8000 substeps)

| Experiment | Duration | Classification | Structural | Catalyst | Pass/Fail |
|------------|----------|----------------|------------|----------|-----------|
| baseline | 8000 | Transient | ~1850 | ~635 | partial |
| starvation_nutrient | 8000 | Transient | stable | stable | partial |
| starvation_fuel | 8000 | Transient | stable | stable | partial |
| catalyst_knockout | 8000 | Transient | stable | decays w/o rep | pass (test) |
| structure_knockout | 8000 | Transient | decays w/o synth | stable | pass (test) |
| puncture_repair | 8000 | Transient | repair partial | — | partial |
| catastrophic_damage | 8000 | Transient | impaired | impaired | pass (test) |
| no_resurrection | 16000 | Transient | no respawn | 0 | pass (test) |
| static_control | 8000 | Transient | passive decay | — | pass (test) |

See `experiments/generated/*/summary.json` for measured values.

## Evidence of closure

- Catalyst reproduction requires C, N, F (zero-C test, nutrient/fuel tests).
- Structure synthesis requires catalyst (`test_structure_requires_catalyst`).
- Turnover active: synthesis, decay, waste production measured in `CellDetector`.
- Knockouts disable reproduction/synthesis with measurable decline.
- Starvation reduces nutrient pool; fuel removal stops growth.
- No privileged `Cell` object — only `CellDetector` observer.

## Deviations

- Decay rates tuned for testable turnover (documented in `parameter_log.md`).
- Default experiment substeps 8000 (not 250000) for CI smoke; long feature flag for full duration.
- Godot bridge uses `godot` crate 0.2 / API 4.3 (Godot 4.6 runtime).

## Known failures

- Parameter regimes with `unbounded_growth` / `immediate_collapse` recorded in sweep (run `experiment-runner sweep`).
