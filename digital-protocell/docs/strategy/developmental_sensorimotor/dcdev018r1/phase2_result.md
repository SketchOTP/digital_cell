# DC-DEV-018-R1 completion result

Final conclusion:

```text
DCDEV018R1_CLOSED_LOOP_METABOLIC_FEASIBILITY_AUDIT_COMPLETE
DCDEV018R1_SOURCE_SIDE_HOMEOSTASIS_FEASIBLE_FINITE_RESOURCE_LIMIT_CONFIRMED
```

The ideal source-only arm passed: no infeasible step, final `E_stored=77.9102788085218` versus target `77.9102788084689`, alive, finite, and nonnegative. The finite-resource source-saturated upper bound failed the preregistered sufficiency gate: final `E_stored=59.1464166923814` versus deprived `60.8278151421244`, despite conservation and survival.

The response envelope separately demonstrates state-dependent demand amplification. This invalidates the prior static gain estimate as a sufficient feasibility proof, but does not change the A-E classification because the ideal sustained source arm and finite-resource upper-bound arm independently establish class B.

The old controller reachability calculation remains provenance-limited: the committed DC-DEV-018 evidence did not contain the per-step error trace. No failed controller was rerun or promoted. The observed capacity is `1.07088866298817`, frozen cap `2.368462987851295`, predicted fraction `0.452144985368632`, and `could_reach_cap_in_horizon=false`; the record explicitly marks exact-trace provenance incomplete.

Production behavior changed: `false`. Certified Phase-1 equations: unchanged. Downstream phases: not started. Next execution: `false`. No DC-DEV-018-R2 and no DC-DEV-019.
