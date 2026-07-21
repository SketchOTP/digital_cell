# D-064 — Connected-Geometry Coupled Rejection and Membrane-Load Decomposition

## Primary conclusion

`D064_STATIC_COUPLED_RESOURCE_METRIC_DEFECT` (Route A)

## Finding

D-063’s narrative that connected geometry “passes static χ but fails coupled χ” was produced by **inconsistent resource-sufficiency accounting**, not by a measured accepted-throughput collapse.

| Metric | Definition | Radial R22 value |
|---|---|---|
| Legacy static χ | `k_T · 0.35 · L · dt / (0.01 · A_int · dt)` — **requested** analytical flux | ≈13.55 (≥1.05) |
| Legacy coupled χ proxy | `import / (0.01 · A_int · accepted_steps)` — treats **Δt≡1 per step** | ≈0.19 |
| Canonical coupled χ | accepted N/F supply / `(0.01 · A_int · window_time)` | ≈19.03 (≥1.05) |

Physical D-063 failure signature **is** reproduced:

- accepted before cascade ≈1076 / horizon 2500
- A retention ≈0.40
- S 368 → 227
- first reject: `IncomingStateInvalid` / `waste:excessive concentration` after carrier → `CARRIER_W_OVERDRAW`

So: **χ collapse was an accounting artifact**; A drain, S desorption load, multiface W overcommit, and waste-ceiling rejection remain real residual defects for later directives.

## Secondary diagnostics (not primary route)

- **Multiface budgeting:** t0 `max ω_N/F≈4.4`, `max ω_W≈20.9`; joint allocator does **not** remove the long-horizon cascade (`joint_allocator_rescues=false`).
- **Seed:** `PREBUILT_SEED_DESORPTION_LOADED` (over-θ fraction ≈0.18); Seed C exchange-relaxed still fails A retention ≥0.80.
- **Operator isolation:** `MULTIPLE_COUPLED_LOADS`.
- **Upper bound:** `CONNECTED_GEOMETRY_NOT_PRIMARY_COUPLED_REPAIR` (A still collapses with perfect exterior N/F hold).
- **Geometry:** channel width 1-cell on radial/branched → inconclusive / not classified as sole cause.

## Authorizations

- V15: unauthorized
- Morphogenesis: unauthorized
- Production carrier: unauthorized
- Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Next directive

Repair the **canonical accepted-flux χ evaluator** (static and coupled; physical time; no requested/unbounded numerator), then **re-run D-063 capacity selection** under that evaluator before any topology or morphogenesis decision.

## Artifacts

`digital-protocell/experiments/generated/d064/` → archive symlink under `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d064/`

Frozen `k_T = 1.4346157818803311` (shadow only).
