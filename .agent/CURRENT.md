# CURRENT.md

## Active directive
- ID: D-20260721-d062-long-horizon-structural-maintenance-decay
- Project directive: D-062
- Goal: Long-horizon structural maintenance/decay review under DynamicStructure
- Status: done
- Acceptance: met — Route N `D062_NO_LOCAL_STRUCTURAL_MAINTENANCE_LAW`
- Touched files: d062_analysis/tests, experiment-runner/d062, main.rs, lib.rs, docs/d062_*, experiments/generated/d062, .agent/*
- Next action: next directive closes external-carrier/small-size route; next_execution_started=false

## Repo facts needed now
- Primary: `D062_NO_LOCAL_STRUCTURAL_MAINTENANCE_LAW` (Route N)
- Baseline @10k: `EXISTING_STRUCTURAL_PERSISTENT_RUNAWAY_GROWTH`
- Decay execution: OK (not Route X)
- Scalar m_d: span~1.28 but flat vs R → not identifiable for restoring crossing
- Candidate C: no restoring crossing on preregistered grid
- Frozen k_T: 1.4346157818803311 (shadow only)
- Artifacts: `experiments/generated/d062` → `/mnt/storage1tb/.../d062` (archive_5k preserved)

## Last validation
- Command: `cargo test -p chemistry-core --test d062_tests`; `D062_MAX_ACCEPTED=10000` pipeline
- Result: 12/12 PASS; primary Route N

## Open blockers
- External-carrier/small-size route exhausted without local structural maintenance law
- Stage E remains BLOCKED_NOT_RECOVERED
- Mimir validation_run allowlist rejected package-filtered cargo test; local tests used
