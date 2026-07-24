# CURRENT.md

## Active directive
- ID: D-20260723-d084-edge-boundary-structural-homeostasis
- Project directive: D-084
- Goal: Mixed bulk/interface structural turnover under edge boundary
- Status: done — D084_STRUCTURAL_BASIN_NOT_ESTABLISHED
- Acceptance: met (stop at Gate5; prescribed restoring found but basin not established)
- Touched files: structural_kinetics, config, d084_*, docs/d084, .agent/*
- Next action: multi-seed dynamic basin under η≈0.07535, k≈0.01963

## Repo facts needed now
- D-083 seal: b966502 / D-083-edge-dynamic-migration-repaired
- Gate4 restoring for η∈{0.075,0.20,0.36}; η=0 control fails
- Production default still legacy ε+I (mixed off)
- Stage E BLOCKED_NOT_RECOVERED

## Last validation
- Command: cargo test -p chemistry-core --test d084_tests; D084_SKIP_LATE_GATES=1 cargo run --release -- d084 pipeline
- Result: tests 10/10; pipeline D084_STRUCTURAL_BASIN_NOT_ESTABLISHED (gates0–4 true)

## Open blockers
- Dynamic basin (Gate5) not established
- Stage E not recovered
- Leave unstaged: .cursor/rules/*, AGENTS.md

## Mimir V2
- project_id: 7bff443192353517
- task_id: a48cb841669c4eefa35826d3ced85acd
- version: 2
- retrieval.session_id: 86f55685e01445198ccce15189a89bcc
