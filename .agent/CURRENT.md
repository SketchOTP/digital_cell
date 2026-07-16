# CURRENT.md

## Active directive
- ID: D-20260716-d019-structural-scaling-repair
- Project directive: D-019
- Goal: Structural scaling repair and Stage E recovery
- Status: partial
- Acceptance: partial — mechanism selected + B–D pass + restoring nullcline; Stage E NOT_CONVERGED so D019_STRUCTURAL_SCALING_REPAIR_PASS not met
- Touched files: structural_kinetics.rs, config EquationVersion V3, simulation rates, d019 runner/tests/docs/artifacts
- Next action: Joint four-rate recalibration under v3 (or progressive Stage E rate screen) then re-run R22/R18/R26

## Repo facts needed now
- Selected: interface_limited_turnover (membrane_metabolism_v3_structural_scaling)
- Preserve D018_SURFACE_VOLUME_SCALING_INCOMPATIBLE + CONSTRAINT_WASTE_ARTIFACT
- Prebalance k_center≈0.2576 restoring; Stage E Q_structure≪1 at frozen companion rates
- D-012 solver: CLOSED; D-008 Stage E: BLOCKED_NOT_RECOVERED
- Mimir slug: digital_cell

## Last validation
- Command: cargo test d008–d019 release; Stage B/C/D PASS; Stage E 200k NOT_CONVERGED (2 k attempts)
- Result: suites PASS; Stage E scientific fail (no quasi-steady)

## Open blockers
- Stage E joint quasi-steady not reached with prebalance k alone or single Q-corrected k
