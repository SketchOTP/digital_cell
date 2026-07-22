# D-071 — Capacity-Bounded Precursor Demand Regulation

## Primary conclusion

`D071_FAIL`

## Route

`Route_U_fail`

## Starting state

- Commit: `0ac93bb`
- Tag: `D-070-mature-membrane-seed-capacity-repair`
- Preserved: `D070_SEED_REPAIR_QUALIFIES_EXCHANGE_PRECURSOR_LIMIT_REMAINS`
- Canonical seed: Seed B / Policy D under `SEED_CAPACITY_CONTRACT_V1`
- Frozen: `K_eq`, `k_exchange`, `Γ_max`, activation, carrier (diagnostic only)

## Gate summary (D071_MAX_ACCEPTED=1200)

| Gate | Result |
|---|---|
| 0 Control reproduction | PASS — A≈0.35, occ≈0.992, coverage=1, P≈76→493 |
| 1 Demand ledger | PASS — A→P dominant avoidable demand; ρ_P≈379 |
| 2 Candidate ID | PASS — selected reduced constitutive `m_P≈0.00132` (product-inhibition mid-K_I missed A≥0.80 while bounding P) |
| 3 Accounting | PASS — A→P stoichiometry identity; capacity bounded; regulation schema hashed |
| 4 R22 maintenance | PASS — three windows A≥0.80, occ≥0.95, coverage=1, P slope ≤1e−4 |
| 5 Membrane repair | FAIL — 10% arc damage recovery ≈0.894 (<0.95) |
| 6 Causal controls | PASS |
| 7 R16/R22/R32 portability | PASS |
| 8 Stage E screen | FAIL (depends on repair) |

## Critical Gate 5 evidence

Under the same Seed B / frozen exchange assay:

| Condition | S recovery ratio |
|---|---|
| Selected regulation (`m_P≈0.00132`) | ≈0.894 |
| Constitutive (`m_P=1`, `K_I=0`) | ≈0.897 |
| `k_precursor=0` | ≈0.898 |

Gate 5 failure is **not unique to regulation**: constitutive mature-membrane refill also fails the 95% recovery bar at 1200 accepted steps. Therefore this is not classified as `D071_PRECURSOR_REGULATION_STARVES_MEMBRANE_REPAIR`.

## Regulation schema

Opt-in only (production defaults unchanged: `m_P=1`, `K_I=0`):

- Candidate A: `r_P = m_P · r_{P,0}`
- Candidate B: `r_P = r_{P,0} · K_I/(K_I+P)` with old-state **local concentration** `P`

`K_I` must be identified in concentration units, not total P mass.

## Disposition

| Item | Status |
|---|---|
| Selected route | Route U / `D071_FAIL` |
| Exchange kinetics | frozen (unchanged) |
| Seed capacity contract | preserved |
| Activation | frozen (unchanged) |
| V15 / production promotion | unauthorized |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Production | `REQUIRES_REMEDIATION` |
| Stage F | not authorized |

## Next directive

Diagnose why capacity-valid mature membrane does not refill to ≥95% after declared 10% arc damage under frozen exchange (constitutive and regulated both fail), then revisit precursor-demand qualification.

## Artifacts

`experiments/generated/d071/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d071`
