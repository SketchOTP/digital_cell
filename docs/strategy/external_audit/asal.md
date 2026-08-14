# ASAL

Source: `SakanaAI/asal`, commit `677ba0ea4d3b3ca78273c9906c6e84d2b1481ce7`, tree `21833ce1ec5bed88fffa440eedbbc726f48c1f1a`; Apache-2.0 in `LICENSE`; Python/JAX/evosax.

`main_opt.py` and `main_illuminate.py` generate candidate substrates/configurations with CMA-ES or novelty/illumination loops. `rollout.py` uses JAX state initialization and scanned rollouts; metrics include target, softmax, open-endedness and embedding/novelty scores. These scores are discovery heuristics, not Digital Cell scientific PASS criteria.

Disposition: `ASAL_SIDECAR_RECOMMENDED` only as a future adapter that proposes hypotheses, invokes a governed Digital Cell runner, and stores immutable artifacts for human/contract evaluation. `ADAPT` the adapter pattern; `REJECT_INTEGRATION` source into the organism/runtime. No dependency or sidecar was added.
