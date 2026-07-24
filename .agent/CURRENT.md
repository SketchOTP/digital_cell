# CURRENT.md

## Active directive
- ID: D-20260723-d085-decisive-structural-closure
- Project directive: D-085
- Goal: Complete D-084 dynamic basin; one mechanochemical fallback; Stage E or reject phase-field substrate
- Status: done — D085_PHASE_FIELD_STRUCTURAL_SUBSTRATE_REJECTED
- Acceptance: met (Phase A 15-run + Phase C 45-run; parity OK; substrate rejected)
- Touched files: d085_analysis/tests, experiment-runner/d085, structural_kinetics, config, simulation, docs/d085, .agent/*
- Next action: redesign organism body as conserved cellular/mesh material system (not scalar/curvature rate patch)

## Repo facts needed now
- D-084 candidate dynamically fails: A retention ≈0.26 at R18/22/26 × seeds 1–5
- Failure class: RESOURCE_COUPLING_REVERSAL (parity PASS)
- Mechano weak/center/strong all fail same floor
- Phase-field structural substrate closed for Phase 1
- Leave unstaged: .cursor/rules/*, AGENTS.md

## Last validation
- Command: cargo test -p chemistry-core --test d085_tests --release; d085 pipeline (60 dynamic runs)
- Result: tests 6/6; primary D085_PHASE_FIELD_STRUCTURAL_SUBSTRATE_REJECTED

## Open blockers
- Stage E BLOCKED_NOT_RECOVERED
- Requires substrate redesign (not another rate directive)

## Mimir V2
- project_id: 7bff443192353517
- task_id: 687909e3d0544fedac616905bda535d8
- status: completed (version 4)
- retrieval_feedback: yes (1 useful)
- validation_run: BLOCKED (allowlist/active-observed-task); local evidence used
- commit: c2651ae
- tag: D-085-phase-field-structure-rejected
