# DC-DEV-017 prior-art disposition

- **REFERENCE**: Pols et al., *Nature Communications* (2019), [A synthetic metabolic network for physicochemical homeostasis](https://www.nature.com/articles/s41467-019-12287-2). It supports the relevance of sustained ATP production, substrate import, dissipation, and load-sensitive homeostasis in a synthetic vesicle.
- **REFERENCE**: Covian et al., *PLOS ONE* (2021), [Energy homeostasis is a conserved process](https://pmc.ncbi.nlm.nih.gov/articles/PMC8575270/). It supports demand-coupled respiratory activation under changed energetic demand while high-energy state remains comparatively constrained.
- **COMPOSE**: use the general demand-coupled principle as a rationale for the single later opt-in repair only if the existing metabolism fails its intrinsic-timescale test.
- **BUILD**: implement only the bounded Digital Cell-native adapter authorized by DC-DEV-017, using existing A/R/N/F state and reserve demand.
- **REJECT**: no external code, model, parameter, species, ATP/ADP implementation, or world-behavior mechanism is imported.

Phase 0 and Phase 1 remain observer/assay-only; no production behavior is changed.
