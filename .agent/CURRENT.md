# CURRENT.md

## Active directive
- ID: D-20260719-d042-activation-capacity-buffer-feasibility
- Project directive: D-042
- Goal: Architecture audit — A capacity vs demand vs finite buffer feasibility
- Status: done
- Acceptance: met — `D042_ACTIVATION_CAPACITY_DEFICIT` / Route A; buffer forbidden; Stage E remains BLOCKED_NOT_RECOVERED
- Touched files: d042_analysis, d042_tests, experiment-runner/d042, main.rs, lib.rs, docs/d042_*, experiments/generated/d042, .agent/*
- Next action: next directive must audit/repair activation production; do not add B_A buffer; next_execution_started=false

## Repo facts needed now
- Record: STRUCTURAL_A_TRANSPORT_RETENTION_REJECTED
- Integrated ∫R_A ≪ 0 under healthy-perm / sufficient-P and all demand disables
- Tag: D-042-activation-buffer-feasibility
- Stage E: BLOCKED_NOT_RECOVERED

## Last validation
- Command: cargo test -p chemistry-core --test d042_tests --release; d042 pipeline @25k
- Result: 13/13 PASS; Gate0/1 PASS; Gate2 → D042_ACTIVATION_CAPACITY_DEFICIT; Gates3–5 skipped

## Open blockers
- Activation production insufficient for membrane–metabolism bootstrap; buffer not authorized
