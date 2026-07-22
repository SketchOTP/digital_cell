# D-066 Smooth-Membrane Activation Utilization and Local Substrate Access Audit

## Primary conclusion

`D066_FROZEN_ACTIVATION_CAPACITY_LIMIT`

## Route

`Route_K_frozen_activation_capacity_limit`

## Preserved D-065 state

- Conclusion: `D065_RESOURCE_DELIVERY_SUFFICIENT_ACTIVATION_LIMITED`
- Smooth static χ: R16≈2.53, R22≈1.82, R32≈1.27 (reproduced via `one_step_static_window`)
- Connected-membrane capacity branch: closed
- Record: `SMOOTH_MEMBRANE_RESOURCE_DELIVERY_SUFFICIENT`

## Activation lineage

- Equation: `membrane_metabolism_v13_catalyst_saturating_activation`
- Rate law: `V_A · H(φ) · q_C(C) · (N/N_ref) · (F/F_ref)`
- Parameters: V_A=0.12544510052968755, K_C=0.10, N_ref=F_ref=1.0
- Stoichiometry: N+F→A+W
- Production: no hard min(N,F) extent clip; on accepted steps accepted==requested

## Route rationale (K)

- Global smooth delivery sufficient (χ_min≈1.265394590249201)
- Mass-conservative N/F redistribution does not restore A
- Healthy/redistributed C does not restore A under ordinary delivery
- Unlimited local activation N/F restores A (≈1.8103485056949373)
- Perfect exterior N/F does not
- Acceptance execution defect: absent
- W does not mask usable windows
- Therefore: frozen activation capacity under ordinary local substrate access cannot cover A demand

## Authorizations (unchanged)

- Activation-law change: unauthorized
- A-demand change: unauthorized
- V15: unauthorized
- Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Artifacts

`experiments/generated/d066/` (symlink → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d066`)

## Next directive

Review frozen activation capacity law under frozen stoichiometry (no change in D-066). Do not increase environmental import until capacity review completes.
