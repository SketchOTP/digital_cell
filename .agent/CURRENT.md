# CURRENT.md

## Active directive
- ID: D-20260719-d041-structural-a-retention-basin-accessibility
- Project directive: D-041
- Goal: Weak structural φ-interface A retention for autonomous healthy-basin accessibility
- Status: done
- Acceptance: met (honest stop) — `D041_STRUCTURAL_A_RETENTION_NOT_SUFFICIENT` at Gate 2
- Touched files: membrane_transport, config, candidate_identity, d041_analysis/tests, experiment-runner/d041, docs/d041_*, experiments/generated/d041, .agent/*
- Next action: architect review of local conserved A-binding / activation-buffer (not in D-041); do not weaken exchange or restore constitutive S→W

## Repo facts needed now
- Record: VALIDATED_EXCHANGE_LAW_FROZEN
- Transport schema 3 implemented; historical defaults unchanged
- Gate0 Route F confirmed; Gate2: no permitted ρ_A recovers basin
- Stage E: BLOCKED_NOT_RECOVERED

## Last validation
- Command: cargo test -p chemistry-core --test d041_tests --release; d041 Gate0; diagnose-rho --steps 12000
- Result: 9/9 PASS; Gate0 pass; diagnostic shows ρ_A≤1 does not lift late θ into healthy basin

## Open blockers
- Basin accessibility still unresolved; structural A retention rejected as permanent fix
