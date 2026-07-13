# D-003 Calibration Report

**Status:** pipeline complete (diagnosis + 3× calibration + short screen)  
**Outcome:** no K_phi calibration reached two consecutive passing windows; short screen 0/3 pass

## K_phi summary (final iteration, seed 2, 20k window)

| K_phi | k_structure | k_rep   | Qφ    | QC    | slope_φ   | pass |
|------:|------------:|--------:|------:|------:|----------:|:----:|
| 0.5   | 0.204       | 0.0146  | 0.983 | 1.020 | −3.79e−4  | no   |
| 1.0   | (see iter_05) |       | ~0.98 | ~1.02 | ~−3.7e−4 | no   |
| 2.0   | 0.108       | 0.0146  | 0.983 | 1.019 | −3.73e−4  | no   |

All three K_phi values converge Qφ/QC toward 1.0 but mass slopes remain above the 1×10⁻⁴ threshold after 6 iterations.

## Short screen (seeds 1–3, 20k, analytical K_phi=1.0 params)

| seed | Qφ   | QC   | retention | pass |
|-----:|-----:|-----:|----------:|:----:|
| 1    | 0.663 | 1.812 | 0.92     | no   |
| 2    | 0.651 | 1.751 | 0.92     | no   |
| 3    | 0.651 | 1.751 | 0.92     | no   |

Retention adequate; balance ratios fail Stage B gates (Qφ < 0.80).

Artifacts: `experiments/generated/d003/calibration/`, `experiments/generated/d003/short_screen/`
