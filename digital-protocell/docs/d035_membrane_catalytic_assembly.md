# D-035 Mature-Membrane-Catalyzed Assembly — Completion Report

## Conclusion

**`D035_ISOLATED_CATALYTIC_RENEWAL_FAILURE`**

## Preservation

- Branch: `d008-membrane-metabolic-closure`
- Starting commit: `9a3bef9`
- Tag `D-034-surface-maturation-fail` present
- Fields retained: `φ,C,N,F,W,A,P,U,S`
- `LINEAR_SURFACE_MATURATION_LAW_REJECTED` recorded

## Architecture screen (Gate 0)

| Law | Span | Portable |
|-----|------|----------|
| Control A (linear) | **33.33×** | no (rejected) |
| Candidate B (mass-action) | **6.67×** | no |
| Candidate C (saturating) | **2.86×** | **yes** |

Selected: **Candidate C** saturating mature catalysis.

## Gate 1 saturation ID

- `K_A = 0.45` (identifiable)
- `K_U = 0.22` (identifiable)
- Monotonic, zero-at-zero, LOO/bootstrap spreads ≪ 50%

## Selected law (v12)

`membrane_metabolism_v12_membrane_catalytic_assembly`

\[
J = q(C)\,f_A\,f_U\,(k_0\Gamma_{\max}+k_{\mathrm{cat}}\Gamma_S),\quad
f_A=\frac{a}{K_A+a},\quad f_U=\frac{\Gamma_U}{K_U+\Gamma_U}
\]

Reaction: `U + A → S + W`

## Gates 2–4

- Gate 2 conservation: **PASS**
- Gate 3 autocatalytic signature: **PASS**
- Gate 4 portable `k_cat` reconstruction (identified K): **PASS** (span ≤3×)
- Median analytical `k_cat ≈ 0.01265`

## Gate 5 isolated dual-surface renewal

**FAIL** — `D035_ISOLATED_CATALYTIC_RENEWAL_FAILURE`

Evidence (horizon 2000):

- `q_s ≈ 0.009` (need 0.98–1.02): maturation ≪ mature turnover (~100× gap)
- `q_u ≫ 1`: immature accumulation / exchange imbalance
- `CapacityExceeded` reject observed
- Basal fraction proxy ≈ 0.02 (≤5% structural) but catalytic renewal cannot balance S

Per stop rule: preserve failure; do **not** add an explicit membrane-bound catalyst field inside D-035.

## Not run (blocked by Gate 5)

Gates 6–10, Stage E recovery, Stage F.

## Status

- D-008 Stage E: **BLOCKED_NOT_RECOVERED** (unchanged)
- Phase 1: **PHASE1_SELF_MAINTENANCE_PARTIAL**
- Production: **REQUIRES_REMEDIATION**

## Artifacts

`digital-protocell/experiments/generated/d035/`

- preservation, architecture_review, saturation_identification
- conservation, autocatalytic_signature, rate_reconstruction
- isolated_renewal, manifest.json, result.json

## Tests

- `cargo test -p chemistry-core --release --test d035_tests` — 7 PASS
- `cargo test -p chemistry-core --release --test d034_tests` — 9 PASS (historical)

## Next execution

Architecture review of an **explicit membrane-bound catalyst field** (outside D-035). Do not resume Stage F from this state.
