# D-084 Edge-Boundary Structural Homeostasis Recovery

## Entry

- Project directive: D-084
- Agent memory: `D-20260723-d084-edge-boundary-structural-homeostasis`
- Starting commit: `b966502`
- Starting tag: `D-083-edge-dynamic-migration-repaired`
- Sealed records: `D083_EDGE_DYNAMIC_MIGRATION_REPAIRED`, `STRUCTURAL_RESTORING_BLOCKER_REMAINS`

## Architecture under test

Retain production:

\[
r_{\phi,+} = k_{\phi,+} A\, q(C)\, I_\phi
\]

(implemented as existing `structure_production_rate`).

Replace interface-dominated loss with mixed bulk/interface turnover:

\[
r_{\phi,-} = k_{\phi,-}\, \phi\, \bigl[\eta + (1-\eta) I_\phi\bigr]
\]

- `η = 0`: interface-only control
- `η > 0`: ordinary bulk floor with interface still maximum-turnover
- One global `η` and one global `k_φ,-` (calibrated at R22)

Legacy D-019 law `k φ (ε + I)` with `ε = 0.05` remains the default when `use_mixed_structure_turnover = false` (Gate 0 / Gate 1 baseline).

## Explicitly closed

Scalar decay multipliers, A-deficit loss, activation/production sweeps, target radius/mass, global feedback.

## Gates

| Gate | Role |
|------|------|
| 0 | Reproduce D-083 edge migration + universally positive structural drive |
| 1 | Structural gain/loss ledger + radius exponents |
| 2 | ≤3 positive η + η=0 control; calibrate k at R22 |
| 3 | φ→W conservation, η=0 equivalence, hashing, atomicity model |
| 4 | R18+/R22≈0/R26− restoring screen |
| 5 | Dynamic basin (multi-seed) |
| 6 | Energy/waste affordability |
| 7 | Damage + starvation causality |
| 8 | Stage E re-entry |

Stop at first mandatory failure.

## Env knobs

- `D084_SKIP_LATE_GATES=1` — stop after Gate 4 path with honest Gate5 failure if screen passes
- `D084_FULL_GATE0=1` — include full D-083 Gate5 reserve regressions in Gate 0

## Artifacts

`experiments/generated/d084/` → archive symlink under `/mnt/storage1tb/cache/project-artifacts/digital_cell/`.

## Result

Pipeline primary: **`D084_STRUCTURAL_BASIN_NOT_ESTABLISHED`** (stopped at Gate 5).

| Item | Value |
|------|-------|
| Legacy scaling | \(p_G \approx 1.059\), \(p_L \approx 1.312\) (approximately matched) |
| Selected candidate | \(\eta \approx 0.07535\), \(k_{\phi,-} \approx 0.01963\) |
| Prescribed signs (selected) | R18 \(+0.186\), R22 \(\approx 0\), R26 \(-0.591\) |
| \(\eta=0\) control | Not restoring (R18 and R26 both negative after R22 balance) |
| Dynamic basin | Not established (mandatory Gate 5) |
| Stage E | Not attempted |
| D-008 | `BLOCKED_NOT_RECOVERED` |

Artifacts: `experiments/generated/d084/result.json`.
