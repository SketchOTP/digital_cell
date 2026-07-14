# D-005 Long-Transient Continuation Report

**Directive:** D-005 §7  
**Target:** 250,000 accepted substeps from D-004 `snapshot_100000` (fresh seeds 1–3 × k_phi 0.5, 1.0, 2.0)

## Status

**9/9 continuations complete.** Identity verification passed for each run (`continuation_verification.json`).

## Results (all fresh-state continuations)

| Candidate | Seed | Final Qφ | Final QC | Stable windows | Classification |
|-----------|------|----------|----------|----------------|----------------|
| k_phi=0.5 | 1 | 0.404 | 0.107 | 0 | CONTINUED_DRIFT |
| k_phi=0.5 | 2 | 0.399 | 0.103 | 0 | CONTINUED_DRIFT |
| k_phi=0.5 | 3 | 0.395 | 0.099 | 0 | CONTINUED_DRIFT |
| k_phi=1.0 | 1 | 0.389 | 0.108 | 0 | CONTINUED_DRIFT |
| k_phi=1.0 | 2 | 0.383 | 0.104 | 0 | CONTINUED_DRIFT |
| k_phi=1.0 | 3 | 0.380 | 0.100 | 0 | CONTINUED_DRIFT |
| k_phi=2.0 | 1 | 0.376 | 0.108 | 0 | CONTINUED_DRIFT |
| k_phi=2.0 | 2 | 0.370 | 0.104 | 0 | CONTINUED_DRIFT |
| k_phi=2.0 | 3 | 0.366 | 0.101 | 0 | CONTINUED_DRIFT |

## Interpretation

No candidate reaches a D-005 stable window by 250k. All trajectories continue structural and catalyst decline (Qφ ≈ 0.37–0.40). Extending from 100k does not reveal late-time balance. No DEAD halt, but active organization is eroding uniformly across chemistry branches and seeds.

## Stable-window criteria (D-005 §7)

Requires three consecutive windows with 0.98≤Qφ,QC≤1.02, |slope|≤10⁻⁴, retention≥0.80, connected component≥95%.

**Result:** 0 qualifying windows across all 9 runs.

## Conclusion

Long-transient continuation supports **continued collapse / drift**, not late-time attractor entry, for the fresh seed family at R₀=24, C₀=0.35 under all three final calibrated candidates.
