# CURRENT.md

## Active directive
- ID: D-20260719-2116-d043-activation-reaction-capacity-repair
- Project directive: D-043
- Goal: Bounded k_activation recalibration if portable
- Status: done
- Acceptance: met (honest stop) — `D043_ACTIVATION_RATE_NOT_PORTABLE`; Gates 4–9 not started; Stage E remains BLOCKED_NOT_RECOVERED
- Touched files: d043_analysis.rs, d043_tests.rs, d043.rs, lib.rs, main.rs, docs/d043_*, experiments/generated/d043, .agent/*
- Next action: next directive must review activation saturation / catalyst normalization / topology; do not raise historical k; next_execution_started=false

## Repo facts needed now
- Exact law: r = k·C·N·F; historical k=0.020 unchanged
- Record: ACTIVATION_BUFFER_BRANCH_CLOSED
- Gate0@25k ∫R_A≈−760; Gate3 span≈3.38×
- Tag: D-043-activation-capacity-fail

## Last validation
- Command: cargo test d043_tests --release; D043_GATE0_HORIZON=25000 D043_DIAGNOSTIC_HORIZON=3000 d043 pipeline
- Result: 18/18 PASS; Gate0–2 PASS; Gate3 FAIL NOT_PORTABLE

## Open blockers
- Activation rate law non-portable; scalar k repair forbidden
