# D-070 — Mature-Membrane Seed and Capacity Contract Repair

## Primary conclusion

`D070_SEED_REPAIR_QUALIFIES_EXCHANGE_PRECURSOR_LIMIT_REMAINS`

## Route

`Route_P_exchange_repaired_precursor_limit_remains`

## Starting state

- Commit: `a4f1c59`
- Tag: `D-069-mature-membrane-exchange-audit`
- Preserved: `D069_MEMBRANE_EXCHANGE_EXECUTION_DEFECT` / `D069_DESORPTION_EXPLAINED_BY_OVER_CAPACITY_SEED`

## Capacity contract (`SEED_CAPACITY_CONTRACT_V1`)

| Item | Definition |
|---|---|
| S units | `S = δ·Γ` (Cartesian membrane mass density); mass `Σ S·V`, `V=DX²=1` |
| Local capacity | `S_max,i = δ_i · Γ_max` |
| Integrated capacity | `M_S,max = Σ_i S_max,i · V_i` |
| Occupancy | `θ_i = S_i / S_max,i` (must satisfy `0 ≤ θ_i ≤ 1+ε`) |
| Max valid occupancy | `1 + EXCHANGE_BOUND_TOLERANCE` |

Normalization: smooth matched capacities scale approximately as `∝ R` in 2D; capacity independent of timestep/orientation.

## Root cause

Diagnostic `seed_mature_s_on_interfaces(s_per_length=1)` allocates face-length S shares independent of `δ`. Local cells therefore start with `S > δ·Γ_max`. D-069 desorption ≈ over-capacity mass is reproduced exactly (Gate 0).

## Seed provenance

| Seed | Classification | Notes |
|---|---|---|
| Historical face-length (D-063…D-069) | `TOTAL_MEMBRANE_MATERIAL_UNAUTHORIZED` | excess vs capacity contract |
| Capacity-bounded θ=1 | `CAPACITY_VALID` | lawful reconstruction |

## Migration

- **Production default:** Policy A strict rejection (fail closed)
- **Selected scientific repair:** Policy D authorized-material reconstruction (`θ→1` on support; unauthorized excess removed and reported; not converted to P)
- Policy B (local S→P) conserved and idempotent; available when material is authorized, not used as the canonical historical repair

Snapshots/seeds with incompatible capacity fail closed unless migration is explicit.

## Revalidation (capacity-valid seeds; frozen kinetics)

- No initial over-capacity dump
- Occupancy remains ≤ 1
- Seed B (Policy D) at R22 / 1200: `S` retention ≈ 0.992, absolute occupancy ≈ 0.992, boundary coverage = 1.0, `max_θ₀=1`
- Coupled replay: A retention ≈ 0.35 (< 0.80); P inventory grows strongly (precursor demand remains dominant)
- Perfect W sink does not uniquely qualify the result

## Final disposition

| Item | Status |
|---|---|
| Selected route | Route P |
| Primary conclusion | `D070_SEED_REPAIR_QUALIFIES_EXCHANGE_PRECURSOR_LIMIT_REMAINS` |
| Canonical seed | Seed B / Policy D capacity reconstruction |
| Seed-contract authorization | repaired under `SEED_CAPACITY_CONTRACT_V1` |
| Exchange-law authorization | unauthorized (frozen law retained; capacity-valid seeds no longer dump) |
| Precursor-law authorization | unauthorized (limit remains) |
| Activation-law authorization | unauthorized (closed) |
| V15 | unauthorized |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## Next directive

Reopen precursor-demand regulation with:

- frozen exchange kinetics
- frozen capacity contract
- corrected capacity-valid seed identity

Do not change `K_eq`, `k_exchange`, or `Γ_max`.

## Artifacts

`experiments/generated/d070/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d070`
