# D-004 Attractor Report

**Status:** complete — cross-state matrix 21/21; D-005 aggregate appended  
**D-004 conclusion:** `D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT`

## Revision (2026-07-13, D-005 §5 aggregate)

Cross-state experiments confirm:

1. **No convergent active attractor** across fresh, aged, and calibration-endpoint classes for any k_phi candidate.
2. **Fresh seeds** uniformly classify `ContinuedDrift` at 100k with negative structural slopes.
3. **Calibration endpoints** can show improved Qφ on short windows but are **not** accessible from fresh initialization (state-dependent, not basin entry).
4. **D-002 aged state** leads to collapse under calibrated D-003 kinetics.

## Classification criteria (D-004 §12)

Convergent active attractor requires cross-state agreement within 10% mass/radius, retention ±0.05, Qφ/QC ±0.05, and final windows in [0.95, 1.05].

**Result:** criteria not met for any of the three final candidates.

## Corrected Stage B sensitivity (K_phi=1.0)

Only seed 1 passes short screen; seeds 2–3 fail with Qφ≈0.757. Demonstrates **narrow seed dependence**, not robust attractor.

## D-005 follow-on

Long-transient continuation (250k) of k_phi=0.5 fresh seed 1 continues decline (Qφ 0.865→0.404). Supports **inaccessible active attractor** from fresh seed family.

Historical pre-audit findings preserved above; interim hypothesis **confirmed**.

Final classification: see `experiments/generated/d004/manifest.json` and `docs/d005_d004_aggregate.md`.

## Revision 2026-07-13 (preserved under D-006)

D-004 conclusion remains `D004_PIPELINE_CANDIDATE_HANDOFF_DEFECT`.
D-005 closed as `D005_NO_ACCESSIBLE_ACTIVE_ATTRACTOR` for the audited D-003 crowding domain.


## Revision (D-006C)

D-004 pipeline-defect finding preserved. D-006C did not reopen crowding attractors; Stage D surface-turnover screen → `D006_NO_RESTORING_RADIUS`.
