# D-067 Activation Capacity Law Identification Under Frozen Stoichiometry

## Primary conclusion

`D067_NO_PORTABLE_ACTIVATION_CAPACITY_LAW`

## Route

`Route_N_no_portable_activation_capacity_law`

## Preserved D-066 state

- Conclusion: `D066_FROZEN_ACTIVATION_CAPACITY_LIMIT`
- Records: `ACTIVATION_HIGH_SUBSTRATE_CAPACITY_PRESENT`, `ORDINARY_SUBSTRATE_ACTIVATION_RESPONSE_INSUFFICIENT`
- Gate 0 reproduction: χ_smooth R16/22/32 ≈ 2.53 / 1.82 / 1.27; ordinary A≈0.355; unlimited local N/F A≈1.810; χ_A≈0.117

## Baseline activation lineage

- Equation: `membrane_metabolism_v13_catalyst_saturating_activation`
- Rate: `r = V_A · H(φ) · q_C(C) · N̂ · F̂`
- `N̂ = N/N_ref`, `F̂ = F/F_ref`, `N_ref = F_ref = 1.0` (no upper clip; may exceed 1)
- Product `N̂F̂` is effectively quadratic in the ordinary domain
- Ordinary weighted medians (R22): N̂≈F̂≈0.32, product≈0.104 → `ORDINARY_RESPONSE_LINEAR_LOW`
- Not equivalent to Michaelis `q_N q_F`

## Required multiplier / ceiling

- χ_A-target scale `m_V★ ≈ 1.05/χ_A ≈ 8.97`
- Radius proxy `m_A★` span remains portable (≤3×)
- High-resource ceiling: `HIGH_RESOURCE_CEILING_HAS_HEADROOM` (unlimited local N/F restores A)

## Candidates (≤3 including baseline)

| Candidate | Result |
|---|---|
| A baseline | Ordinary A≪0.80 |
| B global `m_V` | Scales that approach ordinary A≥0.80 (`m_V≳11`) cause high-N/F rejection cascades (`steps_ok=false`) → unsafe at needed scale |
| C bounded `q_N q_F` | Short 600-step windows can transiently show A≥0.80 (e.g. `K=0.01`), but at ≥1200 accepted steps ordinary A peaks ≈0.64–0.65 `<0.80` → fails durable qualification |

## Route rationale (N)

Neither one global capacity calibration nor one bounded low-substrate response passes durable ordinary coupled admission without high-resource instability or short-horizon false positives. No activation candidate is selected for implementation.

## Authorizations (unchanged)

- Selected activation law: **none**
- Activation-law change: unauthorized
- A-demand change: unauthorized
- V15: unauthorized
- Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Diagnostic note

`activation_schema=3` (`ACTIVATION_SCHEMA_BOUNDED_NF`) exists only as an opt-in diagnostic dispatcher for Candidate C shadows. Production defaults are unchanged.

## Artifacts

`experiments/generated/d067/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d067`

## Next directive

Return to the dominant A-demand / membrane-maintenance architecture (precursor demand and Stage E block) under frozen activation stoichiometry. Do not implement a production activation-law change from D-067.
