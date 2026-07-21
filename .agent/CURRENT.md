# CURRENT.md

## Active directive
- ID: D-20260721-d060-structural-growth-resource-size-feedback
- Project directive: D-060
- Goal: Diagnose neutral size manifold; qualify local resource-coupled structural restoring-size feedback (shadow-only)
- Status: done
- Acceptance: met — Route G `D060_STRUCTURAL_GEOMETRY_EXECUTION_DEFECT`
- Touched files: d060_analysis.rs, d060_tests.rs, d060.rs, main.rs, lib.rs, docs/d060_*, experiments/generated/d060, .agent/*
- Next action: next directive repairs φ application / structure-constraint execution only; next_execution_started=false

## Repo facts needed now
- Primary: `D060_STRUCTURAL_GEOMETRY_EXECUTION_DEFECT` (Route G)
- Root cause: `enforce_structure_constraint` → `apply_phi=false`; analytic G−L > 0, coupled dR/dt = 0
- Neutrality cause: `STRUCTURAL_GEOMETRY_COUPLING_DEFECT`
- Frozen k_T: 1.4346157818803311; D-059 Route L reproduced
- No kinetic candidate authorized; V15 unauthorized; Stage E remains BLOCKED_NOT_RECOVERED
- Artifacts: `digital-protocell/experiments/generated/d060` → `/mnt/storage1tb/.../d060`

## Last validation
- Command: `cargo test -p chemistry-core --test d060_tests`; `D060_MAX_ACCEPTED=400 cargo run -p experiment-runner --release -- d060 pipeline`
- Result: 10/10 PASS; primary Route G

## Open blockers
- Structure-constraint execution freezes φ; must repair before any structural-law candidate can create a restoring basin
