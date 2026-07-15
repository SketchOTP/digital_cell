# D-013 Accepted-Step Windows

Only accepted substeps may advance simulated time, rolling windows, convergence counters, ledgers, checkpoints, and terminal scientific state.

Rejected adaptive attempts may update only attempt/rejection diagnostics.

## Window validity

A window is valid only when:

- accepted sample count equals the governed window size
- simulated time increases strictly across samples
- first and final samples are distinct accepted states
- material-equivalent and activation-potential totals are present and finite
- no invalid numerical state occurs

Zero-duration or zero-motion windows generated from rejected attempts are discarded.

## Convergence

Three consecutive qualifying accepted-step windows are required for quasi-steady convergence, with normalized absolute slopes ≤ 1×10⁻⁴ and rolling reaction/transport changes ≤ 5% between consecutive windows.
