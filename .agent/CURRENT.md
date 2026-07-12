# CURRENT.md

## Active directive
- ID: D-20260712-1618-phase1-protocell
- Project directive: D-001
- Goal: Phase 1 self-maintaining artificial chemistry protocell
- Status: done (partial — long-duration acceptance pending)
- Acceptance: 22/22 smoke tests PASS; experiments generated; no privileged Cell object
- Touched files: digital-protocell/**
- Next action: run `--features long-experiments` for 250k-substep acceptance

## Repo facts needed now
- Workspace: `digital-protocell/`
- Tests: `cargo test -p chemistry-core --release`
- Experiments: `cargo run --release -p experiment-runner -- all`

## Last validation
- Command: cargo test -p chemistry-core --release --test integration_tests
- Result: 22 passed

## Open blockers
- Full 250k-substep viability run not executed in default CI mode
