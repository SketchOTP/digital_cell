# CURRENT.md

## Active directive
- ID: D-20260724-d092-minimal-catalytic-template-heredity
- Project directive: D-092
- Goal: Minimal catalytic template polymer heredity
- Status: done
- Acceptance: One exact D092_* primary — met
- Touched files: template_*, material_mesh, mesh_reactions, mesh_fission, d092_*, docs/d092_*
- Next action: Follow-up architecture for reaction-network expression (not Phase 3)

## Conclusion
- Primary: D092_TEMPLATE_HEREDITY_QUALIFIED_MOTIF_SELECTION_REJECTED
- Schema: autopoietic_material_mesh_catalytic_template_v1
- Fidelity ≈ 0.013 per-site mismatch
- Phase 3: not authorized
- D-091 seal: 58817ac / D-091-metabolic-reserve-qualified-selection-rejected

## Last validation
- Command: cargo test -p chemistry-core --test d092_tests --release; D092_SMOKE=1 cargo run --release -p experiment-runner -- d092 pipeline
- Result: d092_tests 7/7; primary D092_TEMPLATE_HEREDITY_QUALIFIED_MOTIF_SELECTION_REJECTED

## Open blockers
- Full non-smoke Gate6–8 matrices not run (honest smoke failure recorded)

## Mimir V2
- task e74126b6fc934f198f7ddfc5ebdc3122 (close pending)
