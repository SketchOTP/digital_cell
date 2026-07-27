# CURRENT.md

## Active directive
- ID: D-20260727-d095-evolutionary-selection-coupling-architecture-review
- Project directive: D-095
- Goal: Diagnose the failed selection coupling and freeze at most one local, conservative D-096 architecture contract
- Status: in-progress — observational decomposition complete; replay not started
- Acceptance: normalized D-089–D-094 evidence; causal decomposition/replays; one exact classification; candidate decision; D-096 contract; Phase 3 blocked
- Touched files: d095 analysis/tests/runner/docs/artifacts, .agent/*
- Next action: reconstruct matched high/low D-094 organisms and run frozen pre-fission causal replays

## Repo facts needed now
- Terminated PID 1509382; reason DOWNSTREAM_EXECUTION_STOPPED_AFTER_GATE6_NONPASS
- D-094R2 fresh attempt `d094r/gate6/attempt_001` → 24/24 complete at generation 8, 192 atomic checkpoints
- D-094 result seal: 935359e / tag D-094-autocatalytic-selection-rejected
- D-093 sealed 973222e + tag D-093-template-network-heredity-qualified-selection-untestable
- D-095 is review/contract-only: no new hereditary substrate or evolutionary campaign

## Last validation
- Command: cargo test -p chemistry-core --release --locked --test d095_tests; experiment-runner d095 observational
- Result: 3/3 tests pass; 24 included/0 excluded; likely broken link PHENOTYPE_TO_DESCENDANT_COVARIANCE_ABSENT_OR_WEAK; classification remains provisional

## Open blockers
- D-008 test-only non-exhaustive SnapshotFields matches (lines 234, 255) are a separate repair candidate

## Session constraints
- Mimir V2 tools required by AGENTS.md are unavailable in this session; do not claim Mimir lifecycle completion.

## Mimir V2
- project: 7bff443192353517
- task: 6bf10e4654bc44b6900b66871dd6e4c5 version 2
- retrieval.session_id: bbc13ebee00142a2a58dff1f6537b25e
