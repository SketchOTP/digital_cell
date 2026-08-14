# D-088 horizon transfer

The horizon authority is the non-smoke `steps(12_000)` mapping in
`chemistry-core/src/d088_analysis.rs`: `12,000 / 3 = 4,000` accepted steps at
`MechParams::default().dt = 0.02`, or 80.0 accepted simulated time.

The shadow uses the exact ten D-088 campaign seeds, selected `y_g=0.90`,
default mechanics/reactions/transport/fission parameters, and the documented
candidate perturbation followed by vertex perturbation `0.35` and bipolar
x-stretch `1.25`. A paired preparation removes those perturbations while
keeping the same seeds, mechanics, and 4,000-step horizon.

Observed result:

- perturbed preparation: 10/10 fissions within the horizon;
- unperturbed preparation: 10/10 fissions within the horizon;
- replacement horizon: not used.

Therefore this shadow does not invalidate transfer of the frozen 4,000-step
horizon. It also does not prove Gate 7 physiology parity; that separate parity
check fails on the Gate 5 versus adapter configuration differences.
