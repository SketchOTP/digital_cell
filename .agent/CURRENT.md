# CURRENT.md

## Active directive
- ID: D-20260712-phase1-scientific-closure
- Project directive: D-002
- Goal: Phase 1 long-horizon scientific closure
- Status: in_progress — 250k baseline seed 1 running
- Acceptance: accounting+validation tests, 250k acceptance runs, manifest, approved conclusion
- Touched files: digital-protocell/crates/chemistry-core/**, experiment-runner/**, configs/phase1_candidate.toml, docs/**
- Next action: await baseline_seed_1; run replicates + interventions; finalize report

## Repo facts needed now
- D-001 savepoint: tag D-001-baseline commit 2123435
- phase1_candidate.toml frozen tuned params (k_structure=0.030 etc.)
- ~20 ms/substep release; 250k ≈ 83 min per seed

## Last validation
- Command: cargo test -p chemistry-core --release --test validation_tests stoichiometric
- Result: 4/4 PASS

## Open blockers
- Full 250k×5 seed + intervention suite not complete
- Mimir memory_record_outcome: 504 gateway timeout
