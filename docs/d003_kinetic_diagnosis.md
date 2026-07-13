# D-003 Kinetic Diagnosis

**Project directive:** D-003  
**Equation version:** `d003-crowding-v1`  
**D-002 reference commit:** `2c8404536332914adf21fc07341af619e4454443` (tag `D-002-failed-acceptance`)  
**Overall Phase 1 conclusion (unchanged):** `PHASE1_SELF_MAINTENANCE_PARTIAL`

## Why the D-002 model declines

D-002 established 5/5 completed 250k runs, 0/5 viability passes. Structural mass and catalyst mass both decline; turnover ratios at 250k remain far below 1.25 (e.g. seed 2: structural replacement 0.659, synthesis ratio 0.101).

The structural synthesis term used:

```text
r_structure = k_structure × C × N × F × max(0, 1 − φ)
```

while decay is `r_structure_decay = k_structure_decay × φ`. In the dense phase (φ ≈ 1), production is suppressed (`max(0,1−φ) ≈ 0`) but decay remains proportional to φ. Cumulative structural synthesis is much smaller than cumulative structural decay across all D-002 seeds — consistent with interface-localized production attempting to replace bulk degradation.

## Synthesis vs decay localization (D-002 evidence)

From D-002 turnover ledgers (250k, seeds 2–5): structural decay ≈ 1224 vs structural synthesis ≈ 124 (order-of-magnitude gap). Bottleneck diagnostics on the crowding candidate at calibration show non-zero dense-phase synthesis once `g_structure(φ)` replaces the vacancy factor; D-002 legacy kinetics concentrate synthesis at the interface.

## Transport analysis

Calibration windows for K_phi = 0.5 (seed 2, 20k substeps) report `transport_limited: false`. Interior-weighted nutrient/fuel remain above 10% of reservoir setpoints during short calibration runs. Full transport-limitation branch (§14) not triggered in initial calibration.

## Catalyst retention analysis

Short calibration runs report `retention_limited: false`. Fraction of catalyst outside φ ≥ 0.5 remains below the 0.25 threshold during 20k calibration windows. Outside decay does not dominate total catalyst loss in these windows.

## Adaptive dt and simulated time

D-002 full runs: accepted substeps 250,000; simulated time ≈ 39.06 per seed (mean dt ≈ 1.56×10⁻⁴). D-003 adds `DtTelemetry` recording accepted simulated time, dt percentiles, reductions, and recoveries. Reaction ledgers integrate `rate × accepted_dt` per substep.

**Implementation note:** An early D-003 regression applied φ hard-max rejection in the pre-clamp loop, stopping runs at ~15 substeps. Fixed by deferring `PHI_HARD_MAX` to `validate_structure_field` so adaptive dt can converge (as in D-002).

## Seed audit (substep 0)

Seeds 1–5 share identical nutrient/fuel/waste field hashes; structure and catalyst hashes differ only by noise. Structural mass CV < 0.02 across seeds; no seed-specific execution path detected. Seed 1 structural mass (1857.7) is within 0.02 of seeds 2–5 (~1858); prior D-002 divergence at 250k is runtime dynamics, not initialization defect.

Artifacts: `experiments/generated/d003/diagnosis/seed_audit.json`
