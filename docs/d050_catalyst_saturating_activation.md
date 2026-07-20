# D-050 — Coupled Catalyst-Saturating Activation Repair

## Primary conclusion

`D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED`

Failed gate: **Gate 5** (bounded capacity selection).

## Preservation

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Starting commit | `479ca35` |
| Starting tag | `D-049-coupled-aps-collapse-audit` |
| Record | `COUPLED_HISTORICAL_ACTIVATION_CAPACITY_REJECTED` |
| Historical schema 1 | preserved: `r = 0.020 · C · N · F` |
| Membrane / demand / transport | unchanged |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## Exact candidate equation (schema 2)

```text
H(φ) = φ²(3 − 2φ)
q_C(C) = C / (K_C + C)
n = N / N_ref,  f = F / F_ref
r_activation = V_A · H(φ) · q_C(C) · n · f
```

Stoichiometry unchanged: `N + F → A + W` (C required, not consumed).

Equation version: `membrane_metabolism_v13_catalyst_saturating_activation`  
(eight-field, V8 surface/exchange path; activation schema 2 only).

## Gate results

| Gate | Result |
|------|--------|
| 0 D-049 reproduction | **PASS** — analytic A≈0.010, restored A≈0.035 (both `<0.10`) |
| 1 Fixed-biochem Model C ID | **PASS** — V_A≈0.1254, K_C=0.10; train med≈1.2%, hold med≈1.2%, hold max≈2.8% |
| 2 Shadow activation | **PASS** (observer-only) |
| 3 Schema / V13 identity | **PASS** |
| 4 Conservation / parity | **PASS** (parity, stoichiometry, zero-resource) |
| 5 V_A capacity screen | **FAIL** — no candidate reaches A retention ≥0.80 |
| 6–13 | not started (stop-on-fail) |

### Gate 5 candidate screen (horizon 5000)

| V_A | Multiplier | Analytic A retention |
|-----|------------|----------------------|
| 0.094 | 0.75× | ≈0.035 |
| 0.125 | 1.00× | ≈0.036 |
| 0.157 | 1.25× | ≈0.036 |
| 0.251 | 2.00× | ≈0.034 |
| 0.502 | 4.00× | ≈0.031 |

Schema 2 confirmed active (`activation_schema=2`). Isolated sim tests show activation extent scales with V_A, but **coupled free-A retention stays ~3%** across a 5× V_A span (slightly worse at higher V_A). Restored-branch bootstrap did not yield a ready healthy snapshot under the smoke horizon (`restored_ran=false`).

## Fitted parameters

| Param | Value |
|-------|-------|
| Fitted V_A | ≈0.125445 |
| Fitted K_C | 0.10 |
| N_ref, F_ref | 1.0, 1.0 |
| Identification basis | D-047 Model C proxy `L_A ≈ V_A · V · q(C)` on fixed-biochemistry rows |

## Scientific conclusion

Schema 2 is **identifiable** on fixed biochemistry and **implemented** with snapshot isolation from schema 1. It does **not** restore coupled activated-resource capacity: raising V_A does not lift free-A retention toward 0.80 in the complete organism. Likely mechanism: A-linear productive demand (especially precursor ∝ A) and/or N/F throughput limits absorb extra activation so free A remains collapsed. Per directive stop rules, do **not** immediately add product inhibition, activation buffers, or `C_star`. Next step is **coupled activation-topology review** using D-050 trajectory evidence.

## Selected architecture

None for production. Schema 2 remains available as `V13` for further diagnostics; historical schema 1 remains the frozen production default for non-V13 resumes.

## D-008 / Phase 1 status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Tests and artifacts

- `cargo test -p chemistry-core --test d050_tests --release` → **21/21 PASS**
- Artifacts: `digital-protocell/experiments/generated/d050/`
- Failure tag (recommended): `D-050-catalyst-saturating-activation-fail`

## Deviations

- Gates 8–12 in the runner are pragmatic stubs only if Gate 5+ pass; they were **not** reached.
- Gate 1 used D-047 sealed fixed-biology rows with explicit label mapping; `high_c`/`med_c` are not all present in `fixed_biology_family` and were mapped/blended.
- Gate 5 restored branch often `restored_ran=false` under short horizons; analytic branch alone is decisive for the fail.

## Next directive

Coupled activation-topology review (do not retune membrane exchange / precursor / transport; do not add C_star/buffer without new topology evidence).  
`next_execution_started=false`
