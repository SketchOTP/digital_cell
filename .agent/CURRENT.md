# CURRENT.md

## Active directive
- ID: D-20260722-d073-mature-membrane-equilibrium-sufficiency-audit
- Project directive: D-073
- Goal: Audit whether D-072 Route X is upheld by equilibrium-sufficient fixed-P / long-horizon evidence
- Status: done
- Acceptance: met — `D073_ORGANISM_EXCHANGE_INTEGRATION_DEFECT` (Route E)
- Touched files: d073_analysis/tests, experiment-runner/d073, docs/d073_*, experiments/generated/d073, .agent/*
- Next action: D-074 repair organism exchange integration; next_execution_started=false

## Repo facts needed now
- D-072 sealed `28dcdc4` / `D-072-membrane-damage-refill-audit`; Route X provisional retained
- D-072 fixed_P = NOT_ACTUALLY_FIXED (P=1 once, not held)
- p_required(0.95)≈0.38; true holds at 1.0×/1.1× do not recover ≥0.95 within 5τ
- Long-horizon constitutive: SLOW_TRANSIENT_DECAY (0.998→0.668); A retention≈0.056
- Endogenous mean interface p≈0.18 < 0.38; total P large
- Unrelated dirty: .cursor/rules/*, AGENTS.md — exclude

## Last validation
- Command: cargo test -p chemistry-core --test d073_tests --release; D073 pipeline
- Result: 10/10 PASS; primary D073_ORGANISM_EXCHANGE_INTEGRATION_DEFECT

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
- Organism does not recover under analytically sufficient fixed interface p
