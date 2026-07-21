# CURRENT.md

## Active directive
- ID: D-20260721-d061-structural-constraint-execution-repair
- Project directive: D-061
- Goal: Repair structure-constraint execution; revalidate unmodified structural size dynamics
- Status: done
- Acceptance: met — Route G `D061_UNMODIFIED_STRUCTURAL_RUNAWAY_GROWTH`; execution repair QUALIFIED
- Touched files: StructureEvolutionMode, simulation/snapshot/candidate_identity, d061_analysis/tests, d061.rs, main.rs, docs/d061_*, experiments/generated/d061, .agent/*
- Next action: next directive reviews structural decay/maintenance; next_execution_started=false

## Repo facts needed now
- Primary: `D061_UNMODIFIED_STRUCTURAL_RUNAWAY_GROWTH` (Route G)
- Execution: `D061_STRUCTURE_EXECUTION_REPAIR_QUALIFIED`
- Drive: `POSITIVE_ALL_RADII` under DynamicStructure; FixedGeometry still immobilizes φ
- Frozen k_T: 1.4346157818803311 (shadow only)
- No kinetic/carrier/V15 change; Stage E remains BLOCKED_NOT_RECOVERED
- Artifacts: `digital-protocell/experiments/generated/d061` → `/mnt/storage1tb/.../d061`

## Last validation
- Command: `cargo test -p chemistry-core --test d061_tests`; `D061_MAX_ACCEPTED=1000 cargo run -p experiment-runner --release -- d061 pipeline`
- Result: 12/12 PASS; primary Route G

## Open blockers
- Unmodified structural law runaway-grows; decay/maintenance review required before Stage E
