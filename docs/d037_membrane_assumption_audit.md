# D-037 — Membrane-Turnover Provenance and Renewal-Gate Audit

## Primary conclusion

**`D037_TURNOVER_AND_GATE_DEFECTS`**

## Selected route

**`ROUTE_A_TURNOVER_TRANSFER_REPAIR`**

Next execution **not** started.

## Operative qualification

`D036_ARCHITECTURE_REJECTION_PENDING_ASSUMPTION_AUDIT` → secondary:

- `D036_ARCHITECTURE_REJECTION_NOT_UPHELD` (rejection criterion used invalid pointwise balance)
- `D034_PORTABILITY_REJECTION_NOT_UPHELD` (same)
- `D035_DYNAMIC_FAILURE_UPHELD_UNDER_INHERITED_LAMBDA` (isolated-renewal failure remains empirical under the inherited λ; λ itself is not certified)

Historical tags and conclusions for D-021–D-036 are **unchanged**.

---

## Gate 0 — Turnover lineage

| Directive | Loss equation | Rate | Spatial factor | Acts on | Introduced as |
|-----------|---------------|------|----------------|---------|---------------|
| D-008 | `k_M M` (+ detach) | `k_membrane_decay=0.002` | detach `(1−I)` | bulk `M` | mixed localization + Stage B turnover |
| D-019 | retained | `0.002` frozen | structural scaling elsewhere | bulk `M` | retained companion |
| D-021 | `k_M M [ε_M+(1−I)]` | `0.002`, `ε_M=0.02` | `ε_M+(1−I)` | bulk `M` | interface-protection localization |
| D-024 | `S←S e^{−k_Γ Δt}` ≡ `k_Γ S` | `k_gamma_decay=default_k_membrane_decay()` | **none** | embedded `S=δΓ` | mirrored historical scale |
| D-025…D-035 | same S→W | `0.002` inherited | δ in integrals | mature `S` | inherited load |

**Exact transfer commit:** `06477f6` — *D-024: Add conserved interfacial membrane surface density*  
Config comment: `Mirror historical membrane decay scale.` — **no `×ε_M`**.

Lineage status: resolved (not `D037_TURNOVER_LINEAGE_UNRESOLVED`).

### D-021 vs D-024 equations

```text
D-021:  L_bulk    = ∫ k_M M [ε_M + (1 − I(φ))] dV
D-024+: L_surface = ∫ k_Γ S dV   with   k_Γ := k_M
```

---

## Gate 1 — Bulk↔surface loss equivalence

Matched states: identical φ (R∈{16,22,32}, interface width ∈{2,3,4}), identical embedded membrane mass (`S=δΓ=M`), no synthesis/ads/transport/φ motion, detachment excluded from `L_bulk`.

| Metric | Result |
|--------|--------|
| Samples | 9/9 fail 5% gate |
| Max relative error | **≈1.91** (≫0.05) |
| Surface/bulk loss factor | **≈2.70–2.91** |
| Per-mass surface hazard | radius/width stable within 5% |
| δ audit | `δ·k·Γ ≡ k·S` (no duplicated δ) |
| ε audit | surface omits `ε_M` protection |

Idealized `I≡1` localization would inflate by `1/ε_M=50×`. Diffuse matched seeds place mass across `I<1` wings, raising mean bulk hazard above `ε_M k_M`, so observed inflation is ~2.7–2.9× — still a hard transfer defect.

**`D037_SURFACE_TURNOVER_TRANSFER_DEFECT`**

---

## Gate 2 — Provenance classification

**`MIXED_PURPOSE_TERM`**

Evidence: D-008 Stage B decay/detach for localization; D-021 `ε_M` as localization support; D-024 mirror comment with no independent biological assay after surface localization solved structurally.

Flag: **`D037_TURNOVER_PROVENANCE_UNSUPPORTED`** (not certified as `CONSTITUTIVE_BIOLOGICAL_TURNOVER`).  
No replacement rate selected inside D-037.

---

## Gate 3 — State classification (D-034–D-036)

| State | Class | Pointwise-balance eligible |
|-------|-------|----------------------------|
| `highU_lowS`, `balanced`, `lowU_highS`, `lowA`, `medA`, `highA` | forced synthetic | **no** |
| `gate5_pre_capacity` | restored failing | **no** |
| Gate4 planted-k assay | diagnostic control | **no** (ID only) |

Flag: **`POINTWISE_BALANCE_APPLIED_TO_NONEQUILIBRIUM_STATES`**

Zero true/quasi-steady reconstruction states were used for production=loss requirements.

---

## Gate 4 — Renewal-gate semantics

| Criterion | Zero net S imposed? | States steady? | Span | Valid? |
|-----------|---------------------|----------------|------|--------|
| D-034 `k_mature_required` | yes | no | ~33× | **no** |
| D-035 Candidate C algebraic | yes | no | ~2.86× | **no** |
| D-036 `η_required` | yes | no | ~60× | **no** |

**`D037_RENEWAL_GATE_SEMANTICS_DEFECT`**

This invalidates the **portability rejection criterion**, not the empirical D-035 isolated-renewal trajectory under inherited λ.

---

## Gates 5–6 — Reduced dynamics (observer-only)

Capacity-respecting lumped ODEs (`θ≤1`, `J_p` scaled so `S*=J_p/λ≤1` under inherited λ):

| Architecture | Physical FP | Locally stable | Multistart common attractor |
|--------------|-------------|----------------|----------------------------|
| D-034 linear | yes (under scaled `J_p`) | yes | yes (when capacity-safe) |
| D-035 Candidate C | conditional | conditional | not certified vs full PDE |
| D-036 proposed complex | conditional | conditional | not certified; no v13 |

Do **not** require arbitrary initials to be instantaneously balanced. Attractors are necessary, not sufficient, while Gate1/2 fail.

---

## Gate 7 — Route

| Field | Value |
|-------|-------|
| Primary | `D037_TURNOVER_AND_GATE_DEFECTS` |
| Route | **A — Turnover transfer repair** |
| Next directive | Correct D-021→surface loss representation mapping only; revalidate D-024 substrate, D-031 isolated renewal, D-034/D-035 under corrected loss; preserve historical results |
| Next execution started | **false** |

Routes B–E not authorized as the next execution.

---

## Status (unchanged until a later scientific pass)

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

---

## Artifacts

`digital-protocell/experiments/generated/d037/`

preservation, turnover_lineage, bulk_surface_equivalence, turnover_provenance, state_classification, gate_semantics, reduced_dynamics, multistart, route_decision, manifest.json

## Tests

```text
cargo test -p chemistry-core --release --test d037_tests
```

11/11 PASS

```text
cargo run -p experiment-runner --release -- d037 pipeline
```

PASS → `D037_TURNOVER_AND_GATE_DEFECTS` / `ROUTE_A_TURNOVER_TRANSFER_REPAIR`

## Tag

`D-037-membrane-assumption-audit`
