# CURRENT.md

## Active directive
- ID: D-20260719-1312-d036-membrane-bound-catalytic-complex
- Project directive: D-036
- Goal: Audit D-035 parity; membrane-bound catalytic complex if warranted
- Status: done — honest failure at Gate 1
- Acceptance: met (exact conclusion with stop-on-failure evidence)
- Touched files: chemistry-core d036_analysis/tests, experiment-runner/d036, experiments/generated/d036, docs/d036_*
- Next action: none for D-036; fundamental Phase 1 membrane-turnover review

## Repo facts needed now
- Conclusion: `D036_CATALYTIC_COMPLEX_ARCHITECTURE_REJECTED`
- Gate 0: `D035_RUNTIME_DEFICIT_CONFIRMED` (parity OK; ~8.5× instantaneous deficit)
- Gate 1: η span ≈60×; no v13
- D-008 Stage E remains BLOCKED_NOT_RECOVERED

## Last validation
- Command: `cargo test -p chemistry-core --release --test d036_tests`; `d036 pipeline`
- Result: unit PASS; pipeline Gate0 PASS Gate1 FAIL

## Open blockers
- Phase 1 membrane-turnover load/basis assumptions need fundamental review before new species
