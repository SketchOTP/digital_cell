# D-004 Metric Consistency Audit

**Status:** complete  
**Conclusion:** metrics consistent between paths; discrepancy was **candidate identity**, not definitions

## Shared implementation

Calibration and screening both call `run_balance_window()` in `bottleneck.rs`.

| metric | definition |
|--------|------------|
| Qφ | Σ `r_structure` / Σ `r_structure_decay` over all substeps in window |
| QC | Σ `r_rep` / Σ `r_catalyst_decay` over all substeps in window |
| slope_φ | `(Mφ_end − Mφ_start) / Δt_sim / mean(Mφ)` |
| slope_C | `(MC_end − MC_start) / Δt_sim / mean(MC)` |

Window: **20,000 accepted substeps** (`BALANCE_WINDOW_SUBSTEPS`) for calibration; Stage B used same count before D-004 correction.

## Simulated time

Both paths accumulate `sim_time` from accepted `dt` only. Short-screen `sim_time ≈ 1.56` for 20k substeps matches calibration (adaptive dt at protocell scale).

## Replay verification

`test_candidate_replay_reproduces_saved_metrics`: K_phi=1.0 iter 5 Qφ relative error **< 1×10⁻⁶**.

## Balance vs ledger

Per-step rate sums align with accounting turnover integrals (Qφ parity). Mass-delta reconciliation requires diffusion terms; full-window ledger needs start-of-window accounting snapshot (documented in `ledger_reconcile.rs`).

## Verdict

**No `D004_METRIC_INCONSISTENCY`.** Reported Qφ 0.98 vs 0.65 reflects different `k_structure` (0.141 vs 0.092), not different metric code.
