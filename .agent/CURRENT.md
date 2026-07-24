# CURRENT.md

## Active directive
- ID: D-20260724-d086-autopoietic-material-mesh-protocell
- Project directive: D-086
- Goal: Replace rejected φ body with conserved material mesh; Phase 1 Gates 0–8
- Seal commit: 41d24ec
- Tag: D-086-mesh-protocell-phase1-pass
- Implementation commit: 20e9f78
- Status: done
- Touched files: material_mesh, mesh_*, d086_*, docs/d086_*, PROJECT_GOAL/PROFILE, .agent/*
- Next action: independent causal audit / reproducibility (not started)

## Repo facts needed now
- Branch: `phase1-autopoietic-material-mesh`
- Schema: `autopoietic_material_mesh_v1` / `mesh_vertices_edges_v1`
- Mech: center (k_s=14, k_pi=0.22, κ_b=2, α≈0.022)
- Phase1: `PHASE1_AUTOPOIETIC_CANDIDATE_PASS`; production `MESH_PHASE1_LINEAGE_QUALIFIED`
- Leave unstaged: `.cursor/rules/*`, `AGENTS.md`

## Last validation
- Command: `cargo test -p chemistry-core --test d086_tests --release`; `cargo run --release -p experiment-runner -- d086 pipeline`
- Result: 9/9 tests; pipeline primary PASS gates 0–8; 15/15 basin

## Open blockers
- None for D-086 acceptance

## Mimir V2
- project_id: 7bff443192353517
- task_id: 94cb9a9d3e9a4afd8c256346c0e6491f
- status: closed completed
- retrieval.session_id: b5030d41ebdd4d7bb175cd7e7cbf44ef (feedback useful=[])
