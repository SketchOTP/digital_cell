# CURRENT.md

## Active directive
- ID: D-20260724-d088-growth-division-inheritance
- Project directive: D-088
- Goal: Metabolically coupled growth, division, and inheritance on certified mesh
- Status: started
- Acceptance: Growth/division/inheritance without divide() or genetics; both offspring can survive
- Touched files: docs/d088_*, experiments/generated/d088, .agent/*
- Next action: Implement surplus-driven growth and local neck instability on frozen Phase 1 mesh

## Repo facts needed now
- Phase 1 certified: D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED / MESH_PHASE1_V1_FROZEN
- Branch: phase2-growth-division-inheritance
- Frozen center: γ=1 ks=14 κb=2 kπ=0.22 dt=0.02
- Do not retune Phase 1 biology for certification stats

## Last validation
- Command: cargo test -p phase1-certifier --release --test metrics_semantics; d087 pipeline
- Result: 4/4 PASS; D087_PHASE1_AUTOPOIETIC_PROTOCELL_CERTIFIED

## Open blockers
- None

## Mimir V2
- D-087 task closed (4b681b5c0cba48f7be4c24e8cceb375b); validation_run BLOCKED (allowlist/active-observed)
- D-088: begin new Mimir task on next coding session
