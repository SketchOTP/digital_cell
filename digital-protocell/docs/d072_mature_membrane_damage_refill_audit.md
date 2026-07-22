# D-072 — Mature-Membrane Damage Refill Causal Audit

## Primary conclusion

`D072_FROZEN_EXCHANGE_CANNOT_REFILL_DAMAGE`

## Route status (D-073 preservation)

`PROVISIONAL_PENDING_EQUILIBRIUM_SUFFICIENCY_AUDIT`

Original reported conclusion is retained above; Route X is not erased. D-073 must determine whether the D-072 fixed-P control and short-horizon maintenance classifications were equilibrium-sufficient before architecture escalation.

## Route

`Route_X_frozen_exchange_cannot_refill`

## Starting state

- Branch: `d008-membrane-metabolic-closure`
- Commit: `0611603`
- Tag: `D-071-precursor-demand-regulation-fail`
- Preserved: `D-070-mature-membrane-seed-capacity-repair`, D-071 conclusion `D071_FAIL`
- Frozen: `K_eq`, `k_exchange`, `Γ_max`, Seed B / Policy D, activation, carrier, topology, damage extent
- D-071 regulation remains opt-in diagnostic only (not promoted)

## No-repair floor

\[
0.992 \times 0.90 = 0.8928
\]

## Gate summary

| Gate | Result |
|---|---|
| 0 D-071 reproduction | PASS — constitutive ≈0.897, regulated ≈0.894, `k_p=0` ≈0.894; pre-occ ≈0.997; sim_time(1200)≈6.0 |
| 1 Intervention integrity | PASS — ΔS+ΔW=0; φ/C/P/A/δ/capacity unchanged; production path syncs current→next |
| 2 Synthetic refill parity | PASS — isolated BE exchange refills a 10% hole toward θ_eq; ledger ΔS≈−ΔP |
| 3 Local refill basis | `LOCAL_P_INSUFFICIENT` — damaged arc has δ, q≈0.42, free capacity, net ads>0, but p≈0.057 ⇒ θ_eq≈0.74 < 0.95 |
| 4 Timescale / horizon | FAIL as rescue — τ≈186; 1200-step horizon ≈0.032τ; recovery **worsens** 0.88→0.62 over 0.5–5τ (net desorption) |
| 5 Diagnostic controls | None restore ≥0.95 — fixed_P≈0.892, mixed_P≈0.688, exchange_only≈0.531, healthy_q≈0.693 |
| 6 Causal classification | Route X |

## Scientific conclusion

A lawful 10% mature-membrane S→W damage event sits at the immediate no-repair floor under D-071 accepted-step horizons because:

1. **Intervention is intact** (S→W conserved; δ/capacity preserved; buffers synchronized on the production path).
2. **Isolated frozen exchange can refill** a synthetic capacity-valid hole.
3. **Endogenous interface precursor activity is too low for the 95% occupancy target** (θ_eq(p≈0.057)≈0.74).
4. **Extending simulated time does not rescue repair** — longer horizons deepen net desorption.
5. **Diagnostic fixed sufficient P and exchange-only controls also fail** to restore ≥95% within multi-τ windows.

Therefore the failure is not a short-horizon artifact alone, not a damage-accounting bug, and not cured by simply clamping P in the organism assay. Under the D-072 stop rules this authorizes a later **exchange-architecture / organism-integration review**, not Stage E/F progression and not precursor-production increases.

## Secondary observations

- Immediate post-damage damaged cells show positive local adsorption basis, but undamaged interface remains above θ_eq and drives global desorption.
- `D072_DAMAGE_REFILL_HORIZON_QUALIFIED` is rejected: recovery falls with τ, not rises.
- `D072_LOCAL_PRECURSOR_DELIVERY_LIMIT` is not primary: fixed/mixed P controls did not restore refill (Route P requires a restoring control).

## Disposition

| Item | Status |
|---|---|
| Selected route | Route X / `D072_FROZEN_EXCHANGE_CANNOT_REFILL_DAMAGE` |
| Exchange kinetics | frozen (unchanged) |
| Seed capacity contract | preserved |
| D-071 regulation | opt-in diagnostic only |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Production | `REQUIRES_REMEDIATION` |
| Stage F / D-009 / V15 | not authorized |

## Next directive

D-073: exchange-architecture / organism-integration review of why capacity-valid damaged interfaces do not refill under frozen P↔S exchange despite synthetic parity — without raising total precursor production and without Stage F.

`next_execution_started=false`

## Artifacts

`experiments/generated/d072/` — preservation, d071_reproduction, intervention_audit, synthetic_refill, local_basis, timescale, diagnostic_controls, causal_classification, accounting, manifest.json
