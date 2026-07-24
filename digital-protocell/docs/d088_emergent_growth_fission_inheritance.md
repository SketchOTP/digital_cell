# D-088: Emergent Growth, Topological Fission, and Material Inheritance

## Conclusion

`D088_CAUSAL_GROWTH_FISSION_INHERITANCE_QUALIFIED`

Records:

* `PHASE2_PHYSICAL_REPRODUCTION_QUALIFIED`
* `MATERIAL_STATE_INHERITANCE_QUALIFIED` (material-state, not genetic)
* `MULTI_GENERATION_MESH_LINEAGE_ESTABLISHED`

## D-087 runtime closure

Mandatory ≥90 wall-clock minute packaged run completed:

* `PHASE1_RESEARCH_RUNTIME_QUALIFIED`
* no crash; snapshot/resume OK; offline; no GPU

Entry correction honored: science was certified while runtime was provisional until this run closed.

## Frozen Phase 1

Center mechanical candidate and Phase 1 reaction/transport defaults unchanged.
Growth is **additive** via `mesh_growth` surplus flux only.

## Surplus / growth

\[
J_{A,\mathrm{surplus},i}=\max(0,J_{A,\mathrm{produced},i}-J_{A,\mathrm{maintenance},i})
\]

Maintenance = turnover replacement + catalyst share + membrane share (excludes Phase 1 build flux).

\[
J_{\mathrm{growth},i}=y_g\,J_{A,\mathrm{surplus},i}\,h(\varepsilon_i,\mathrm{turn})
\]

Selected `y_g` from `{0.90, 1.10, 1.30}` by maintenance-basin + surplus criteria.

## Fission

No `divide()` command. Local pinch: opposing vertices within scale-dependent rebond range form cross-bonds; parent becomes two closed meshes with conservative area/perimeter partition.

## Next

`D-089: Heritable Catalytic Variation and Selection` — `next_execution_started: true`
