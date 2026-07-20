# D-044 — Activation-Law Architecture Review

## Mission

Determine whether D-043 non-portability arises from invalid state selection, scaling defects, starvation contamination, excessive mass-action sensitivity, or deeper topology failure — and whether bounded substrate saturation restores portable activation capacity without raising historical `k_activation`.

## Frozen starting state

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Commit | `ff35e0f` |
| Tag | `D-043-activation-capacity-fail` |
| Record | `SCALAR_MASS_ACTION_RECALIBRATION_REJECTED_PENDING_LAW_REVIEW` |
| Historical law | `r = k·C·N·F`, `k = 0.020` |

## Gate results

| Gate | Result |
|------|--------|
| 0 D-043 reconstruction | **PASS** — span 3.38× reproduced; 7 k estimates within 10% of sealed values |
| 1 Balance-state eligibility | **PASS** — all 8 states `FORCED_DIAGNOSTIC` (A/C/N/F clamps); corrected span N/A → `D044_D043_PORTABILITY_FAILURE_UPHELD` |
| 2 Units/scaling audit | **PASS** — R16/R22/R32 per-catalyst rate deviation ≤ 0.005% |
| 3 Viable substrate domain | **PASS** — `low_nf` = `IRREVERSIBLE_STARVATION`; others = `SYNTHETIC_DIAGNOSTIC` |
| 4–5 Candidate fits | **FAIL** — no candidate passes all identification gates |
| 6–13 | Not started (stop-on-fail) |

### Gate 0 — D-043 reconstruction @ 3000 diagnostic steps

| State | k_required | Sealed | Match |
|-------|------------|--------|-------|
| R16 | 0.373 | 0.373 | yes |
| R22 | 0.226 | 0.226 | yes |
| R32 | 0.150 | 0.150 | yes |
| low_c | 0.491 | 0.491 | yes |
| med_c | 0.285 | 0.285 | yes |
| high_c | 0.189 | 0.189 | yes |
| low_nf | dominated | — | excluded |
| high_nf | 0.145 | 0.145 | yes |

- span: **3.38×**
- median k: **0.226**

### Gate 1 — State eligibility

All reconstruction states carry A clamp 0.5 plus C/N/F clamps → classified **FORCED_DIAGNOSTIC**, not balance-eligible steady states. Removing ineligible states does **not** reduce span (zero balance-eligible estimates). Outcome: **`D044_D043_PORTABILITY_FAILURE_UPHELD`**.

### Gate 2 — Scaling

Matched C/N/F at R16, R22, R32: integrated activation scales with interior volume; **R_activation/M_C** constant within 5×10⁻⁵ relative deviation. No implementation scaling defect.

### Gate 3 — Viable domain

- **low_nf**: internal N/F ≈ 0.3, influx does not cover consumption → **IRREVERSIBLE_STARVATION** (exclude from portability fitting)
- All clamped states: **SYNTHETIC_DIAGNOSTIC** (authorized-demand probes, not homeostatic operating points)

### Gate 5 — Candidate identification (training family, diagnostic 3000)

| Candidate | Key result | Pass |
|-----------|------------|------|
| A (mass action) | span 3.38× on reconstruction family | no |
| B (joint saturation) | K_NF ≈ 0.93, V_B ≈ 545, **span 2.63×**, LOO OK | **no** — bootstrap spread 1.33 > 0.50 limit |
| C (dual saturation) | K_N ≈ K_F ≈ 1.08, V_C ≈ 1219, span 2.66×, LOO OK | **no** — bootstrap spread 1.33 > 0.50 limit |

Joint saturation compresses span below 3× but fails parameter-identification robustness. Neither saturation law qualifies.

## Primary conclusion

**`D044_ACTIVATION_LAW_ARCHITECTURE_REJECTED`**

Secondary records:
- `D044_D043_PORTABILITY_FAILURE_UPHELD`
- Scaling and implementation parity confirmed (no `D044_ACTIVATION_SCALING_DEFECT`)

## Selected architecture

None. Historical `k_d008_activation = 0.020` unchanged. No v13 chemistry version created.

## Scientific conclusion

D-043 mass-action non-portability is **upheld** after eligibility and scaling audits. The failure is not an artifact of radius/volume miscounting or duplicated catalyst scaling. Bounded joint/dual substrate saturation **partially** compresses required-capacity span (B: 2.63×) but does not achieve robust parameter identification or held-out qualification under preregistered gates. The low-N/F diagnostic state is correctly classified as irreversible starvation, not a viable operating point — yet even excluding it, mass-action span remains >3× across catalyst levels.

Scalar rate increase remains forbidden. Activation-buffer, stoichiometry, transport, and downstream demand changes were not attempted. **Fundamental activation-topology review** is the mandated next step (separated fuel charging vs catalytic activation — outside D-044 scope).

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Tests

`cargo test -p chemistry-core --test d044_tests --release` — 16/16 PASS

## Pipeline

```bash
D044_DIAGNOSTIC_HORIZON=3000 D044_GATE0_HORIZON=25000 D044_MAX_ACCEPTED=50000 \
  experiment-runner d044 pipeline --output experiments/generated/d044
```

Runtime ~42 min on reference hardware.

## Artifacts

`digital-protocell/experiments/generated/d044/` — preservation through candidate_fits, decision, manifest

## Tag

`D-044-activation-law-fail` (recommended)

## Next directive

Fundamental activation-topology review: separated fuel charging and catalytic activation. Do not raise historical k. Do not begin Stage F. Re-enter complete D-008 Stage E only after a qualified activation law passes Gates 0–13.
