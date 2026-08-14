# D-096 gain callsite trace

`d096_allocation::function_gain(mesh, index)` maps index 0 to processing, 1 to activation, 2 to repair, and 3 to growth.

Runtime callsites audited:

- `mesh_reactions::structural_build_flux`: coordinate 2 multiplies `g_strain(eps)`, including `g0` and the strain-responsive term.
- `mesh_reactions::reactions_step`: coordinates 0 and 1 enter activation extent; coordinate 2 also enters the free-membrane production branch.
- `mesh_growth::growth_step`: coordinate 3 multiplies reserve-funded growth synthesis.

The complete machine-readable map is `experiments/generated/sr004cr3/gain_callsite_map.json`.
