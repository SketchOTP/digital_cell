# CURRENT.md

## Active directive
- ID: D-20260721-d064-connected-geometry-coupled-rejection-decomposition
- Project directive: D-064
- Goal: Connected-geometry coupled rejection and membrane-load decomposition (shadow-only)
- Status: done
- Acceptance: met — `D064_STATIC_COUPLED_RESOURCE_METRIC_DEFECT`
- Touched files: d064_analysis/tests, experiment-runner/d064, main.rs, lib.rs, docs/d064_*, experiments/generated/d064, .agent/*
- Next action: repair canonical accepted-flux χ evaluator; rerun D-063 capacity selection; next_execution_started=false

## Repo facts needed now
- Primary: `D064_STATIC_COUPLED_RESOURCE_METRIC_DEFECT` (Route A)
- D-063 physical repro: accepted≈1076; A≈0.40; S 368→227; reject waste ceiling after carrier
- Legacy coupled χ≈0.19 (Δt≡1 defect); canonical χ≈19
- Static used requested analytical flux (`k_T*0.35*L*dt`)
- Multiface ω_W≫1; joint allocator does not rescue cascade
- Seed: PREBUILT_SEED_DESORPTION_LOADED; upper bound still collapses A
- Frozen k_T: 1.4346157818803311
- Artifacts: `experiments/generated/d064` → `/mnt/storage1tb/.../d064`

## Last validation
- Command: `cargo test -p chemistry-core --test d064_tests`; `D064_MAX_ACCEPTED=1200 D064_SKIP_LATE_GATES=1` pipeline
- Result: 11/11 PASS; primary StaticCoupledResourceMetricDefect

## Open blockers
- Canonical χ evaluator not yet wired into D-063 capacity selection
- Stage E remains BLOCKED_NOT_RECOVERED
