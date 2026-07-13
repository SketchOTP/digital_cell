# CURRENT.md

## Active directive
- ID: D-20260713-d004-candidate-provenance-attractor-audit
- Project directive: D-004
- Goal: Audit D-003 candidate provenance and active-attractor pipeline validity
- Status: in progress — handoff defect confirmed; cross-state runs pending
- Acceptance: D-004 conclusion + D003_RESULT_UNRESOLVED_PENDING_PIPELINE_AUDIT
- Touched files: chemistry-core/{candidate_identity,attractor,ledger_reconcile,radius_balance}, experiment-runner/{d003,d004}.rs, configs/d004/, docs/d004_*.md
- Next action: complete cross-state 100k runs; finalize docs

## Repo facts needed now
- Stage B screened analytical K_phi=1.0 estimate (k_s≈0.092), not calibrated final (k_s≈0.141)
- Calibration iter_05 Qφ≈0.983 reproducible; short screen Qφ≈0.65 on wrong candidate
- Overall Phase 1: PHASE1_SELF_MAINTENANCE_PARTIAL (unchanged)

## Last validation
- Command: cargo test -p chemistry-core --release --test d004_tests
- Result: 19/19 PASS

## Open blockers
- Full cross-state 100k×27 runs in progress
- D-003_FAIL invalidated pending corrected Stage B
