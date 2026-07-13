# D-005 Long-Transient Continuation Report

**Directive:** D-005 §7  
**Target:** 250,000 accepted substeps from D-004 `snapshot_100000` (fresh seeds 1–3 × k_phi 0.5, 1.0, 2.0)

## Status

Continuations run from verified D-004 checkpoints with candidate identity preserved (`continuation_verification.json` per run).

## Completed: k_phi=0.5, fresh seed 1

| Metric | At 100k (D-004) | At 250k (D-005 continuation) |
|--------|-----------------|------------------------------|
| Mφ | 1809 | 1021 |
| MC | 633 | 383 |
| Qφ | 0.865 | 0.404 |
| QC | 0.901 | 0.107 |
| slope_φ | −3.4×10⁻³ | −1.45×10⁻² |
| retention | 0.933 | 0.528 |
| stable windows (D-005 gate) | 0 | 0 |
| classification | ContinuedDrift | CONTINUED_DRIFT |

**Interpretation:** Extending to 250k does not reveal a late-time balance window. The trajectory continues structural and catalyst decline. No DEAD observer halt yet at 250k, but the active organization is eroding.

## Remaining runs

Additional fresh-state continuations (k_phi 0.5 seeds 2–3; k_phi 1.0 seeds 1–3; k_phi 2.0 seeds 1–3) execute via `experiment-runner d005 pipeline` or `d005 continuations`.

## Stable-window criteria (D-005 §7)

Requires three consecutive windows with 0.98≤Qφ,QC≤1.02, |slope|≤10⁻⁴, retention≥0.80, connected component≥95%.

**First completed run:** 0 qualifying windows.

## Preliminary conclusion

Long-transient continuation supports **continued collapse / drift**, not late-time attractor entry, for the default fresh seed family at R₀=24, C₀=0.35.
