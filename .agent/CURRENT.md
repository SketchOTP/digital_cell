# CURRENT.md

## Active directive
- ID: D-20260721-d066-smooth-membrane-activation-utilization-audit
- Project directive: D-066
- Goal: Audit smooth-membrane activation utilization and local substrate access (shadow-only)
- Status: done
- Acceptance: met — `D066_FROZEN_ACTIVATION_CAPACITY_LIMIT`
- Touched files: d066_analysis/tests, experiment-runner/d066, main.rs, lib.rs, docs/d066_*, experiments/generated/d066, .agent/*
- Next action: review frozen activation capacity under frozen stoichiometry; next_execution_started=false

## Repo facts needed now
- Primary: Route K `D066_FROZEN_ACTIVATION_CAPACITY_LIMIT`
- D-065 reproduced: static χ R16/22/32 ≥1.05; ordinary A≈0.36; Control-C A≈1.81; perfect exterior does not restore
- Redistribution / healthy C do not restore A; acceptance execution defect absent
- Activation: schema-2; accepted==requested on accepted steps; no hard N/F clip
- Frozen k_T: 1.4346157818803311
- Artifacts: experiments/generated/d066 → /mnt/storage1tb/.../d066

## Last validation
- Command: cargo test -p chemistry-core --test d066_tests; D066_MAX_ACCEPTED=1200 D066_SKIP_LATE_GATES=1 pipeline
- Result: 16/16 PASS; primary FrozenActivationCapacityLimit / Route K

## Open blockers
- Stage E remains BLOCKED_NOT_RECOVERED
- Unrelated dirty: .cursor/rules/*, AGENTS.md — excluded from D-066 staging
