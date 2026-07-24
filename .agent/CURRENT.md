# CURRENT.md

## Active directive
- ID: D-20260723-d084-edge-boundary-structural-homeostasis
- Project directive: D-084
- Goal: Recover restoring structural-size region via mixed bulk/interface turnover (η)
- Status: in_progress — D-083 sealed; implementing D-084
- Acceptance: One D084_* primary with Gates 0–8 evidence or stop at first mandatory fail; no Stage F
- Touched files: pending d084_* + structural_kinetics η
- Next action: implement mixed loss law; Gate0 reproduce D-083; Gate1 ledger

## Repo facts needed now
- D-083 sealed tag: D-083-edge-dynamic-migration-repaired
- Records: D083_EDGE_DYNAMIC_MIGRATION_REPAIRED + STRUCTURAL_RESTORING_BLOCKER_REMAINS
- Loss candidate: r_φ,- = k_φ,- φ [η+(1-η)I_φ]; η=0 control; no production/activation/membrane change
- Closed: scalar m_d, A-deficit loss, target R/mass

## Last validation
- Command: cargo test -p chemistry-core --release --test d083_tests; cargo run --release -- d083 pipeline
- Result: PASS — D083_EDGE_DYNAMIC_MIGRATION_REPAIRED

## Open blockers
- STRUCTURAL_RESTORING_BLOCKER_REMAINS (D-084 target)
- Stage E BLOCKED_NOT_RECOVERED
- Leave unstaged: .cursor/rules/*, AGENTS.md

## Mimir V2
- project_id: 7bff443192353517
- task_id: a48cb841669c4eefa35826d3ced85acd
- version: 1
- retrieval.session_id: 86f55685e01445198ccce15189a89bcc
