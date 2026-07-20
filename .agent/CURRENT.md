# CURRENT.md

## Active directive
- ID: D-20260719-d044-activation-law-architecture-review
- Project directive: D-044
- Goal: Activation-law architecture review
- Status: done
- Acceptance: partial — honest `D044_ACTIVATION_LAW_ARCHITECTURE_REJECTED`; Gates 6–13 not run
- Touched files: d044_analysis.rs, d044.rs, d044_tests.rs, main.rs, lib.rs, docs/d044, experiments/generated/d044, .agent/*
- Next action: fundamental activation-topology review; next_execution_started=false

## Repo facts needed now
- D-043 span 3.38× reproduced; portability failure upheld after eligibility audit
- Scaling OK; low_nf irreversible starvation; Candidate B span 2.63× but bootstrap fail
- Tag recommended: D-044-activation-law-fail

## Last validation
- Command: cargo test -p chemistry-core --test d044_tests --release; D044 pipeline
- Result: 16/16 PASS; pipeline primary=D044_ACTIVATION_LAW_ARCHITECTURE_REJECTED

## Open blockers
- No qualified activation law; Stage E BLOCKED_NOT_RECOVERED
