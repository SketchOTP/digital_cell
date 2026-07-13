# CURRENT.md

## Active directive
- ID: D-20260712-phase1-scientific-closure
- Project directive: D-002
- Goal: Phase 1 long-horizon scientific closure
- Status: partial — 250k×5 baselines complete; interventions pending
- Acceptance: partial (0/5 seed pass; accounting OK; no AUTOPOIETIC pass)
- Touched files: digital-protocell/**, docs/phase1_acceptance_report.md
- Next action: 250k intervention suite OR parameter search for turnover≥1.0

## Repo facts needed now
- Conclusion: PHASE1_SELF_MAINTENANCE_PARTIAL
- Best seed turnover @250k: struct_repl 0.659, synth 0.101 (seeds 2–5)
- Seed 1 retained more mass (1209) but lower repl ratio (0.395)

## Last validation
- Command: cargo test -p chemistry-core --release --test validation_tests
- Result: 17/17 PASS

## Open blockers
- Long-horizon interventions not run
- Mimir: 504 timeout
