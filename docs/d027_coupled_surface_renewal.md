# D-027 — Coupled Surface-Renewal Calibration

## Conclusion

`D027_ISOLATED_SURFACE_RENEWAL_FAILURE`

Gates 0–2 pass under sealed `membrane_metabolism_v7_surface_density`. The analytical adsorption correction is portable (~30× D-024 `k_ads`), but none of the three mandated candidates (`0.5× / 1.0× / 2.0×` median `k_ads_required`) reach sustained late-window surface balance `0.98 ≤ Q_surface ≤ 1.02` on the isolated fixed-interface screen. Intermediate values were not screened (directive forbid).

## Operative status

| Item | Status |
|------|--------|
| D-024 | `D024_PROVENANCE_SEALED` (preserved) |
| D-025 | `D025_STAGE_E_LONG_TRANSIENT_UNRESOLVED` (preserved) |
| D-026 | `D026_SURFACE_COVERAGE_MAINTENANCE_DEFICIT` (preserved) |
| D-027 | `D027_ISOLATED_SURFACE_RENEWAL_FAILURE` |
| D-008 | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Production | `REQUIRES_REMEDIATION` |

## Preservation

Starting commit `77f7ab2` (`D-026-stage-e-recovery-fail`). Tags D-021–D-026 verified present. Artifacts: `experiments/generated/d027/preservation/`.

## Gate results

### Gate 0 — Checkpoint window-local surface ledgers — PASS

Surface cumulative + window baseline persisted in governed checkpoints. Restore re-anchors window-local extents so rates do not depend on pre-checkpoint cumulative history. Restored vs uninterrupted max abs rate diff ≈ `1.4×10⁻¹⁴` (with matching `dt_cap`). Precursor added to lossless field payload for bit-exact v7 continuation.

### Gate 1 — Adsorption basis audit — PASS (portable)

| State | `k_ads_required` |
|-------|------------------|
| D-024 fixed R22 | 0.0510 |
| D-025 dynamic R22 | 0.0292 |
| Stage E 10k | 0.0428 |
| Stage E 25k | 0.0338 |
| Stage E 100k | 0.0281 |
| Stage E 200k | 0.0286 |

- Span: **1.81×** (≤ 3) → portable
- Median: **0.03378** ≈ **30.4 ×** frozen D-024 `k_ads`
- Basis positive; P available; not permanently saturated

### Gate 2 — Analytical candidates — PASS

Exactly three candidates at `0.5 / 1.0 / 2.0 ×` median. No intermediate screen.

### Gate 3 — Numerical safety — partial (unit)

Exact P→S and S→W identities covered by unit tests. Full spatial stiffness campaign not required after Gate 4 failure.

### Gate 4 — Isolated surface renewal — FAIL

Late-window Q after 12k-step screens (⅔ burn-in, ⅓ measure):

| Candidate | `k_ads` | Q_surface | result |
|-----------|---------|-----------|--------|
| 0.5× | 0.0169 | 0.615 | fail (under) |
| 1.0× | 0.0338 | 0.907 | fail (under) |
| 2.0× | 0.0676 | 1.057 | fail (over) |

Localization ≈ 1.0 throughout; occupancy stable; active ads/turnover. No candidate enters `[0.98, 1.02]`. The 1×–2× bracket straddles balance, but intermediates are forbidden.

### Gates 5–12

Not started (stop on Gate 4 failure).

## Scientific reading

Adsorption deficit vs Γ turnover is quantitatively real and portable across fixed, dynamic, and Stage E states (~30×). A single constant `k_ads` on the mandated 0.5/1/2 grid does not land inside the isolated surface-balance window: undershoot at 1×, overshoot at 2×. Next architecture should improve the bulk–surface exchange law (or allow a bounded non-grid calibration) rather than further Stage E productive-rate sweeps under frozen surface kinetics.

## Artifacts

`digital-protocell/experiments/generated/d027/` — preservation, ledger_restore, adsorption_basis, analytical_candidates, isolated_surface, manifest.json.

## Next directive

Thermodynamically/kinetically improved bulk–surface exchange (or architect-authorized interpolation within the 1×–2× bracket with new acceptance rules). Do not begin Stage F. Do not reopen bulk membrane fields.
