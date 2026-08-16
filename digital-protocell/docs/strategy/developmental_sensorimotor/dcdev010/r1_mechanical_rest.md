# DC-DEV-010-R1 Mechanical-Rest Causal Isolation

DC-DEV-010-R1 starts from accepted PR #19 head `b4178417e30907835183c7f9c16a639bdd8d31db` and preserves the original negative result:

`DCDEV010_DIRECTIONAL_SUBSTRATE_TRANSLATION_NOT_ESTABLISHED`

The R1 assay adds no new substrate law and changes no production behavior. It first advances the exact seeded 24-vertex mesh with contractility, regulatory stimulus, chemistry reactions, reserve spending, growth, remeshing, fission, obstacles, contact systems, and plasticity disabled. The existing directional substrate remains enabled.

## Preregistered rest contract

The finite settling horizon is 5,000 accepted mechanics steps, or 100.0 accepted simulated-time units at `MechParams.dt = 0.02`. Rest requires all of the following for 16 consecutive accepted steps:

- maximum attempted local velocity <= `2.6645352591003757e-9`;
- maximum accepted local displacement per step <= `5.3290705182007514e-11`;
- maximum local internal force <= `2.6645352591003757e-9`;
- material-centroid displacement per accepted step <= `2.220446049250313e-13`.

The thresholds are fixed from the authoritative DC-DEV-010 translation tolerance `5.3290705182007514e-11` and the existing `MechParams.dt`; they are not selected from the R1 result.

## Result

The baseline did not satisfy the rest contract within the preregistered horizon. R1 therefore stopped at:

`DCDEV010R1_BASELINE_MECHANICAL_REST_NOT_ESTABLISHED`

No matched R1 qualification arms were executed. No parameter repair, second substrate, adhesion, anchoring, sensing, navigation, resource seeking, learning, reward, fitness, evolution, or DC-DEV-011 work was started.

Evidence is under `experiments/generated/dcdev010r1/`. The original DC-DEV-010 evidence remains under `experiments/generated/dcdev010/` and is not overwritten by the R1 workflow.
