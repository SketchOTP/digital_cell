# CURRENT.md

## Active directive
- ID: D-20260715-1209-d012-tasks6-10
- Project directive: D-012
- Goal: Tasks 6-10 — v2 identity, runtime, accounting, conservation gate
- Status: done (Tasks 6-10)
- Acceptance: d012_tests all required tests PASS; conservation gate PASSED; v1 unchanged; five commits
- Touched files: config.rs, stoichiometry.rs, activated_metabolism.rs, membrane.rs, simulation.rs, snapshot.rs, candidate_identity.rs, d012_accounting.rs, d012_tests.rs, docs/d012_*
- Next action: Task 11 Stage A transport equivalence (Stage B–E still gated on Task 11+)

## Repo facts needed now
- Branch: d008-membrane-metabolic-closure
- v1 class: NO_POSITIVE_CONSERVATION_VECTOR; Task 5 superseded
- v2 unit-yield proven strictly conservative in stoichiometry.rs
- Tag: D-012-stoichiometric-audit

## Last validation
- Command: cargo test -p chemistry-core --release --test d012_tests; d008_tests; d011_tests
- Result: d012_tests 36/36 PASS; d008_tests 50/50 PASS; d011_tests 21/21 PASS
- Conservation gate: PASSED

## Open blockers
- Mimir MCP unavailable (server not in session)
