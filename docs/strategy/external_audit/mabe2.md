# MABE2

Source: `mercere99/MABE2`, commit `1fc9eb6d261b2cb4372cfb739f0a498cd7bd22e0`, tree `3d977079378e64eb5ce21b2e94a849d3ccd8bf40`; MIT in `LICENSE`; Makefile/C++.

The current modular architecture separates controller/population/organism (`MABE.hpp`, `Population.hpp`, `Organism.hpp`), modules and event signals (`Module.hpp`), evaluation (`EvalModule.hpp`), selection, placement, systematics and batch configuration. Selection modules consume assigned `fitness` traits and placement modules decide replacement; `SystematicsModule` records phylogeny.

## Transferable correspondence

| MABE2 | Digital Cell-owned analogue | Boundary |
|---|---|---|
| Controller/Batch | Campaign manifest and governed runner | orchestration only |
| Population | Mesh population | storage/identity only |
| Organism | material organism handle | no genome replacement |
| Module signals | phase/event hooks | preserve authority order |
| Eval/DataMap | observation and analysis records | observer-only |
| Selection/Placement | selection observation and placement analysis | never assigned causal fitness |
| Systematics | lineage recorder | parent/child provenance |

Architecture is `ADAPT`; experimental selection methodology is `BENCHMARK`; source is `REJECT_INTEGRATION` despite MIT because EMP/module dependencies and assigned-fitness births would silently violate Digital Cell's causal-selection boundary. The recommended evolution route is a thin Digital Cell-owned harness inspired by these interfaces.
