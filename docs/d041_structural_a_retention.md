# D-041 — Primitive Structural A Retention and Basin Accessibility

## Mission

Determine whether a weak, local activated-resource retention property of the structural `φ` interface can make the healthy membrane–metabolism attractor autonomously accessible, without changing the validated schema-3 passive exchange law.

## Frozen evidence

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Commit | `e05564b` |
| Tag | `D-040-exchange-precursor-decomposition` |
| Architecture | `membrane_metabolism_v8_reversible_surface_exchange` |
| Turnover | `surface_turnover_schema_3_exchange_damage_only` |
| α / β / K | ≈0.167 / ≈0.00334 / ≈50 |
| Record | `VALIDATED_EXCHANGE_LAW_FROZEN` |
| Prior | `D040_MEMBRANE_METABOLISM_BISTABILITY` (Route F) |

## Transport equation (implemented)

On φ-crossing faces, for species A only:

\[
\Pi_A = \rho_A \exp(-\beta_A \theta_S)
\]

Schema: `membrane_transport_schema_3_structural_a_retention` (`TRANSPORT_SCHEMA_VERSION_V3`).

Historical defaults remain unchanged (`ρ_A = 1`, schema V1). Non-A species and co-phase faces are unaffected. Candidate hashes include schema name and `ρ_A` when schema 3 is active.

## Gate results

| Gate | Result |
|------|--------|
| 0 Route confirmation | **PASS** — D-040 Route F reproduced (frozen-binary 2k pipeline + d041 Gate0 at 3–4k): exchange parity pass; `A_PRODUCTION_DECLINE` earliest; healthy-A / sufficient-P / healthy-perm improve; basins distinguishable |
| 1 Transport isolation | Unit tests **PASS** (antisymmetry, A-only, historical ρ=1 equivalence, mass closure, hash identity) |
| 2 Bounded ρ_A screen | **FAIL** — no permitted `ρ_A` recovers healthy basin |
| 3–10 | Not started (stop-on-fail) |

### Gate 2 diagnostic (12 000 accepted steps)

Zero-S / 5% S conservative redistributions across `ρ_A ∈ {1.0, 0.4, 0.2, 0.05}`:

- Late θ remains ≈0.28–0.32 for all candidates (not healthy).
- Historical `ρ_A = 1` is **best or tied**; lowering `ρ_A` does not improve θ or S.
- A retention ≈0.011 for all — A loss is production-limited, not rescued by interface transport attenuation.

Artifacts: `experiments/generated/d041/retention_candidates/bootstrap_diagnostic.json`.

## Primary conclusion

`D041_STRUCTURAL_A_RETENTION_NOT_SUFFICIENT`

## Stop rule

Reject permanent structural A attenuation as the basin-access fix. Do **not** weaken the validated exchange law or restore constitutive mature S→W. Next review should consider a **local conserved A-binding or activation-buffer** mechanism (out of scope for D-041).

## Status constraints

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Tests

`cargo test -p chemistry-core --test d041_tests --release` — 9/9 PASS.

## Tag

`D-041-structural-a-bootstrap-fail`
