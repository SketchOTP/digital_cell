# D-036 Membrane-Bound Catalytic Complex — Completion Report

## Conclusion

**`D036_CATALYTIC_COMPLEX_ARCHITECTURE_REJECTED`**

## Phase A — Gate 0 parity

**`D035_RUNTIME_DEFICIT_CONFIRMED`**

At the restored D-035 Gate 5 pre-capacity state (`advance=2500`, `ConstrainedRadius`, `U≈8.64`):

| Path | Maturation rate | Notes |
|------|-----------------|-------|
| Gate 4 observer (`∫ δ · candidate_c_rate`) | `0.019137` | |
| Runtime unbounded (`∫ δ · maturation_rate_j`) | `0.019137` | rel ≈ `1e-15` |
| Runtime bounded apply | `0.019137` | no U/A limiting |
| Accepted-step ledger | `0.019132` | rel ≈ `3e-4` |
| Mature turnover `L_S` | `0.16223` | |

Instantaneous maturation/turnover ≈ **0.118** (≈8.5× deficit; window `Q_S≈0.009` in D-035 remains a stronger time-averaged suppression).

Explicit audits: no missing/duplicated `δ`, Γ embedding consistent, `a_reference=1.0` shared, per-time rates consistent.

Recorded: **`MATURE_MEMBRANE_AUTOCATALYSIS_REJECTED`**

## Phase B — Gate 1 architecture

Proposed law (observer):

\[
\eta_{\mathrm{required}}=\frac{L_S}{C\,\Gamma_U\,f_A},\quad
J_S=\eta\,C\,\Gamma_U\,f_A
\]

with QSS complex \(\Gamma_E\approx k_{\mathrm{on}}C\Gamma_U/(k_{\mathrm{off}}+k_{\mathrm{turn}}f_A)\).

| Check | Result |
|-------|--------|
| Valid states | 7 (≥6) |
| Finite positive C/U/A bases | PASS |
| Structural zero controls / capacity / fixed point / Jacobian | PASS |
| η span | **60.1×** (need ≤3×) |
| LOO median | PASS |
| No Γ_S dependence | **FAIL** (highU_lowS vs lowU_highS η ratio ≫3×) |

η tracks \(\Gamma_S/\Gamma_U\) the same way rejected linear `k_mature` did. Adding interfacial complex `E` does not remove that algebraic non-portability under the required-efficiency reconstruction.

## Stop rule

Architecture failed observer feasibility → **do not implement v13**, do not add another species, stop for fundamental review of Phase 1 membrane-turnover assumptions.

## Not run

Gates 2–12, Stage E, Stage F.

## Status

- D-008 Stage E: **BLOCKED_NOT_RECOVERED**
- Phase 1: **PHASE1_SELF_MAINTENANCE_PARTIAL**
- Production: **REQUIRES_REMEDIATION**

## Artifacts

`digital-protocell/experiments/generated/d036/`

- preservation, d035_parity, architecture_review, manifest.json, result.json

## Tests

- `cargo test -p chemistry-core --release --test d036_tests` — PASS
- `experiment-runner d036 pipeline` — Gate0 PASS deficit confirmed; Gate1 FAIL architecture rejected

## Next execution

Fundamental review of Phase 1 membrane-turnover / load-balance assumptions. Do not add fields or retune D-035 rates until that review.
