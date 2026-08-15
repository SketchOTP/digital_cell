# R3 shadow equivalence

The repaired production path replayed processing-heavy, repair-heavy, and neutral candidates in H, B, and Neutral for seeds 1 through 8, 1000 steps, `dt=0.02`, mechanics/topology on, fission off, and mutation off.

All `72/72` rows matched `experiments/generated/sr004cr3/shadow_counterfactual_results.json` under `abs(A-B) <= 1e-9 * (1 + abs(expected))`. The maximum numerical residual was `2.84217094304040074e-14`. No R3 artifact was regenerated or overwritten.
