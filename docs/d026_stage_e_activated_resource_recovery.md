# D-026 — Stage E Activated-Resource Decomposition and Continuation Recovery

## Conclusion

`D026_SURFACE_COVERAGE_MAINTENANCE_DEFICIT`

Stage E under sealed `membrane_metabolism_v7_surface_density` is **not recovered**. Gates 0–6 identify a dominant upstream mechanism; Gates 7–8 are not feasible within frozen D-024 surface parameters and the D-025/D-026 productive-rate envelope.

## Operative status (unchanged pending recovery)

| Item | Status |
|------|--------|
| D-024 | `D024_PROVENANCE_SEALED` |
| D-025 | `D025_STAGE_E_LONG_TRANSIENT_UNRESOLVED` |
| D-008 | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Production | `REQUIRES_REMEDIATION` |

## Preservation

All listed D-021–D-025 commits/tags verified. Starting commit `c87b540`. Artifacts: `digital-protocell/experiments/generated/d026/preservation/`.

## Gate results

### Gate 0 — Runner parity — PASS

Dynamic (`enforce_structure_constraint=false`) vs constrained (`true`) one-step chemistry from identical settled state: max abs diff ≈ `3.13×10⁻¹³` on `delta_s`; all other compared chemistry/transport metrics exact. Constrained path: φ fixed, constraint ledger isolated, surface advection disabled, tangential diffusion retained. **Not** `D026_STAGE_E_HARNESS_MISMATCH`.

### Gate 1 — Observability — PASS

`StageEObservabilitySample` records field masses, Γ occupancy quantiles / low-coverage fractions, A production/demand split, and A interface flux diagnostics without altering chemistry.

### Gate 2 — Reference history

Earliest upstream divergence: **`SURFACE_COVERAGE_DECLINE`** (θΓ 0.575→0.540 between 10k and 25k accepted steps), preceding A-retention collapse (0.977→0.856→0.512). Terminal `Q_membrane≈0.032`. Localization remains ≈1.0 throughout (localization ≠ coverage).

### Gate 3 — Initial-condition dependence

Not required for this failure class after Gate 6 + Gate 8 infeasibility. Deferred (no Stage E pass claimed).

### Gate 4 / 6 — Mechanism

Principal mechanism: **`SURFACE_COVERAGE_MAINTENANCE_DEFICIT`**.

Late-time behavior is monotonic coverage/retention erosion under constrained radius, not a proven absence of a joint fixed point and not a harness mismatch.

### Gate 5 — Causal controls (3k steps from 100k checkpoint)

| Control | ΔA retention | Interpretation |
|---------|-------------|----------------|
| A no A transport | ~0 | Leakage not primary causal channel |
| **B freeze surface** | **+0.139** | **Surface S/Γ maintenance causal** |
| C no virtual structure | ~0 | Structural A demand not dominant |
| D no catalyst repro | ~0 | Catalyst demand not dominant |
| E freeze + no precursor syn | +0.139 | Matches B |

### Gate 7 — Continuation

Not warranted (mechanism is coverage maintenance under frozen surface kinetics, not an unresolved reduced-system transient).

### Gate 8 — Productive-rate correction

Mapped surface-maintenance path would suggest `k_precursor`, but live-step evidence shows **`delta_p ≫ adsorption`** while `adsorption ≪ Γ turnover`. Precursor supply is not limiting. Closing `Q_membrane` to ~1 would require ~31× adsorption boost, exceeding global `4×` and per-candidate `1.5×` bounds. Frozen `k_ads` / `k_gamma_decay` must not be changed under D-026. **Gate 8 infeasible.**

## Scientific reading

Γ localization ≈ 1 shows existing surface mass stays interfacial. It does not imply adequate occupancy. Declining mean θΓ and rising low-coverage fraction raise effective A permeability, collapsing A retention while C retention remains high. Adsorption cannot match Γ turnover under the sealed D-024 surface parameter set when P is already abundant.

## Artifacts

`digital-protocell/experiments/generated/d026/` — preservation, runner_parity, reference_history, surface_coverage, a_budget, causal_controls, late_time_classification, analytical_candidates, manifest.json.

## Next directive

A follow-on that is allowed to revisit sealed surface adsorption/turnover balance (or an alternate interfacial maintenance mechanism) without reopening bulk membrane fields — not further unconstrained productive-rate sweeps under the current freeze.
