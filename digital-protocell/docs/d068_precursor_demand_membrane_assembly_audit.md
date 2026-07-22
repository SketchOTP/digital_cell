# D-068 Precursor Demand and Membrane Assembly Closure Audit

## Primary conclusion

`D068_MEMBRANE_DESORPTION_DOMINANT`

## Route

`Route_S_s_desorption_limit`

## Preserved D-067 state

- Conclusion: `D067_NO_PORTABLE_ACTIVATION_CAPACITY_LAW`
- Records: `ACTIVATION_LAW_BRANCH_CLOSED`, `PRECURSOR_MEMBRANE_DEMAND_CAUSE_UNRESOLVED` (cause now resolved as desorption-dominant)
- Gate 0 reproduction: ordinary A≈0.355; χ_A≈0.117; unlimited local N/F restores A; precursor ~76% of A demand

## Precursor lineage (frozen)

- Equation: `r_P = k_precursor · A · q(C) · H(φ)`
- Stoichiometry: `A → P` (ν_W = 0 on synthesis)
- No P or S dependence (Candidate C not already present)
- Exchange: `dS/dt = δ · k_exchange · q(C) · Γ_max · (K_eq · p · (1−θ) − θ)`
- Schema 3 constitutive damage: none (λ_Γ = 0)

## Precursor utility (smooth R22, 1200 accepted)

| Metric | Value |
|---|---|
| A→P extent (syn_P) | ≈425.6 |
| P→S adsorption (accepted net+) | ≈2.77 |
| S→P desorption (accepted net−) | ≈99.7 |
| η_P→S | ≈0.0065 |
| futile fraction | ≈0.993 |
| ΔS | ≈−96.9 |
| S retention | ≈0.449 |
| fate | `PRECURSOR_ACCUMULATION` (P inventory rises while S falls) |

Observer note: continuous-rate `exchange_forward`/`exchange_reverse` proxies are **not** parity-safe for ledgers. Accepted-step `exchange_net` (actual xfer) is required; with that choice the S ledger closes.

## Membrane maintenance

- Fixed healthy P does **not** arrest S (S retention ≈0.48) → not precursor-supply limited
- Interface-weighted P redistribution does **not** rescue S
- Adsorption ≪ desorption under accepted exchange_net → `S_DESORPTION_DOMINANT`
- χ_S from accepted net extents ≪ 1

## Candidates

- Stopped before qualification: fixed healthy P cannot maintain S; desorption dominates
- Candidate B (global m_P) not admitted for authoritative selection
- Candidate C not implemented (no opt-in precursor dispatcher; would be production change)

## Final disposition

| Item | Status |
|---|---|
| Selected route | Route S |
| Primary conclusion | `D068_MEMBRANE_DESORPTION_DOMINANT` |
| Selected precursor law | none |
| Precursor-law authorization | unauthorized |
| Membrane-exchange authorization | unauthorized (audit next) |
| Activation-law authorization | unauthorized (D-067 closed) |
| V15 | unauthorized |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## Next directive

Audit the S→P desorption law / reversible exchange reverse branch under **frozen** precursor production. Do not increase precursor synthesis. Do not change activation.

## Artifacts

`experiments/generated/d068/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d068`
