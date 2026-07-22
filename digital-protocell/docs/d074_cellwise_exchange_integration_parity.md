# D-074 Cellwise Exchange Integration Parity Audit

## Mission

Determine why organism-level mature-membrane recovery differs from the frozen exchange law’s predicted recovery by auditing the exchange path **cell by cell** against the exact discrete runtime integrator.

## Frozen contracts

Unchanged: `K_eq`, `k_exchange`, `Γ_max`, `SEED_CAPACITY_CONTRACT_V1`, Seed B / Policy D, precursor production / D-071 regulation, activation / catalyst chemistry, transport / permeability, interface definition, repair threshold, damage extent, numerical tolerances.

## Discrete reference

- Bath-fixed (diagnostic):  
  \(\theta_{n+1}=\theta_{eq}+(\theta_n-\theta_{eq})/(1+\lambda\Delta t)\) with \(\lambda=k_{exchange}q(C)(K_{eq}p+1)\).
- **Runtime authority**: mild explicit Euler when the proposal stays in the invariant domain; otherwise `solve_exchange_backward_euler` with local inventory \(T=P+S\) conserved.
- D-074 parity compares runtime \(\Delta S\) to the **production-faithful** mild-FE / BE hybrid, not the continuous exponential approximation.

## Gates

| Gate | Content |
|------|---------|
| 0 | Reproduce D-073 fixed-P recovery anchors |
| 1 | Static / cellwise parity (exchange-isolated) |
| 2 | Reachable repair ceiling (inactive \(q=0\) retains post-damage \(S\)) |
| 3 | Capacity-weighted cumulative exposure \(\Lambda_i\) |
| 4 | Accepted-step replay (rejected steps: zero extent) |
| 5 | Integration-path inspection |
| 6 | Bounded repair only if proven defect |
| 7 | Requalification at \(p\in\{0.38,0.418,2.48\}\) |

## Routes

Exactly one of: `D074_EXCHANGE_INTEGRATION_DEFECT_REPAIRED` (E), `D074_LOCAL_CATALYTIC_EXPOSURE_LIMIT` (Q), `D074_INTERFACE_SUPPORT_COVERAGE_LIMIT` (I), `D074_EXCHANGE_TIMESCALE_CLASSIFICATION_DEFECT` (T), `D074_MEMBRANE_REPAIR_METRIC_DEFECT` (M), `D074_EXCHANGE_RUNTIME_PARITY_UNRESOLVED` (X).

## Artifacts

`digital-protocell/experiments/generated/d074/` (symlink → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d074/`).

## Status

**Primary conclusion:** `D074_EXCHANGE_TIMESCALE_CLASSIFICATION_DEFECT` (Route T).

### Key evidence

- D-073 recovery anchors reproduced (artifact + live exchange-only).
- Runtime cell \(\Delta S\) matches the production mild-FE / invariant-domain BE predictor (parity within \(10^{-5}\)).
- Damaged inactive \(q(C)=0\) capacity fraction: **0**.
- Unsupported capacity fraction: **0**.
- Reachable ceiling at \(p=0.38\): max theoretical repair fraction \(\approx 0.957\) (above the 0.95 gate).
- Under exchange-isolated fixed-\(p=0.38\), observed recovery \(\approx 0.952\) (meets gate); D-073’s \(\approx 0.941\) used surface diffusion enabled.
- At the mean-\(\tau\) “\(5\tau\)” horizon, **100%** of damaged lawful capacity is only `EXPOSURE_1_TO_5` (`fraction_ge5 = 0`). Mean-\(\tau\) therefore overstated cellwise exposure.

### Repair

None. No integration defect requiring a production code change.

### Next

Replace membrane horizon gates with capacity-weighted cumulative exposure \(\Lambda_i=\sum k\,q_i(K_{eq}p_i+1)\Delta t_n\). A run qualifies as five-timescale evidence only when \(\ge 95\%\) of damaged lawful capacity has \(\Lambda_i\ge 5\).

