# CURRENT.md

## Active directive
- ID: D-20260721-d063-environmentally-connected-membrane-invagination-architecture
- Project directive: D-063
- Goal: Environmentally connected membrane invagination architecture review (shadow-only)
- Status: done
- Acceptance: met — `D063_CONNECTED_MEMBRANE_SHADOW_REPAIR_FAILURE`
- Touched files: d063_analysis/tests, experiment-runner/d063, main.rs, lib.rs, docs/d063_*, experiments/generated/d063, .agent/*
- Next action: diagnose Gate-8 shadow rejection under connected geometry; next_execution_started=false

## Repo facts needed now
- Primary: `D063_CONNECTED_MEMBRANE_SHADOW_REPAIR_FAILURE`
- Static capacity: p_A≈1; radial α(R22)≈2.66; χ≫1.05 under FixedGeometry assay
- Shadow: radial@2500 rejected after 1076; A≈0.40; S 368→227
- Bootstrap on paper: FEASIBLE with seed-affordable increments (R_i≈8.4)
- Closed vesicles: environmental carrier area = 0
- Frozen k_T: 1.4346157818803311 (shadow only)
- Artifacts: `experiments/generated/d063` → `/mnt/storage1tb/.../d063`

## Last validation
- Command: `cargo test -p chemistry-core --test d063_tests`; `D063_MAX_ACCEPTED=2500 D063_SKIP_LATE_GATES=1` pipeline
- Result: 11/11 PASS; primary ShadowRepairFailure

## Open blockers
- Coupled shadow repair under connected geometry not qualified
- Stage E remains BLOCKED_NOT_RECOVERED
