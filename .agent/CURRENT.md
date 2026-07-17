# CURRENT.md

## Active directive
- ID: D-20260716-d021-retention-localization-repair
- Project directive: D-021
- Goal: Interface-protected membrane retention/localization repair then Stage E recovery
- Status: started — preserving D-020 before v4 implementation
- Acceptance: One D021_* conclusion with ε screen evidence; Stage E only after retention/localization gates
- Touched files: (pending after D-020 commit)
- Next action: Commit+tag D-020, then implement membrane_metabolism_v4_interface_protected

## Repo facts needed now
- D-020 best R22: A retention 0.377, M localization 0.859, Q=[0.213,0.423,0.591,1.425]
- Membrane decay is currently uniform: k_M_decay * M; detachment already off-interface
- Target: r_M_decay = k_M_decay * M * [ε_M + (1 - I(φ))]; screen ε∈{0.02,0.05,0.10}
- Frozen: seven fields, stoich schema 2, transport schema 1, interface-limited structure, rates/yields/env
- Mimir slug: digital_cell

## Last validation
- Command: (pending)
- Result:

## Open blockers
- None yet
