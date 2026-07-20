# D-051 — Coupled Activation Throughput Bottleneck Audit

## Primary conclusion

`D051_RESOURCE_THROUGHPUT_LIMIT`

Selected route: **R — Environmental resource throughput**.

## Preservation

| Item | Value |
|------|-------|
| D-050 commit | `0b0fb890383d8af1ec8633febbeaeb25f53e542d` (`0b0fb89`) |
| D-050 tag | `D-050-catalyst-saturating-activation-fail` |
| D-051 start | same as D-050 result commit |
| Record | `CATALYST_SATURATING_CAPACITY_REPAIR_REJECTED` |
| Frozen | `D049_COUPLED_ACTIVATION_CAPACITY_FAILURE`, `D050_COUPLED_ACTIVATION_CAPACITY_NOT_RECOVERED` |
| Schema 1 | preserved `r = 0.020 · C · N · F` |
| Schema 2 | experimental failed architecture retained (not deleted) |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## Gate −1 — D-050 seal

- Commit and tag verified.
- Schemas preserved.

## Gate 0 — D-050 reproduction (horizon 10000)

| Case | A retention | Gross activation |
|------|-------------|------------------|
| Schema 1 | ≈0.0105 | — |
| Schema 2 0.75× | ≈0.0353 | ≈502 |
| Schema 2 1.00× | ≈0.0357 | ≈559 |
| Schema 2 2.00× | ≈0.0343 | ≈682 |
| Schema 2 4.00× | ≈0.0312 | ≈779 |

- Isolated schema-2 rate scales with `V_A` (PASS).
- Coupled free-A remains collapsed (~3%) with weak `V_A` response (PASS).
- Gross accepted activation **does** rise with `V_A`.

## Gate 1 — Requested vs accepted

- Cap mode: `ACTIVATION_EXTENT_SCALES_WITH_V_A`
- Production volume activation has **no hard `min(N,F)` extent clip**; on accepted steps `ξ_accepted = ξ_requested`.
- Local N/F enter the **rate law**, not a post-rate bound.
- Not `ACTIVATION_EXTENT_RESOURCE_CAPPED` / not numerical extent defect.

## Gate 2 — Resource ceiling

- Ceiling ledger constructed for R22 / analytic-seed proxies.
- Ordinary delivery leaves effective N/F supply insufficient for sustained healthy free A under coupled demand.

## Gate 3 — Resource controls (diagnostic)

| Control | A retention (center, 4k) | Accepted activation |
|---------|--------------------------|---------------------|
| Baseline | ≈0.107 | ≈333 |
| A healthy N | ≈0.153 | ≈611 |
| B healthy F | ≈0.153 | ≈611 |
| **C healthy N+F** | **≈1.094** | **≈3154** |
| D reservoir ×5 | ≈0.107 | ≈333 |
| **E unlimited N/F for activation** | **≈2.002** | **≈12868** |

Classification: `D051_RESOURCE_THROUGHPUT_LIMIT` — healthy/unlimited N/F materially restores activation throughput and A retention; ordinary delivery does not.

## Gate 4 — Operator order

Documented ConstrainedRadius order:

1. reservoir update  
2. N/F/C/A transport rates  
3. structural production  
4. activation + reproduction + A decay + C turnover  
5. precursor diffusion  
6. surface precursor + P↔S  
7. precursor transport apply  
8. positivity reject  

- Shadow jointly-bounded schedules conserve material; timestep refinement stable.
- `operator_split_defect = false` (no production mutation; coupled suppression does not vanish under analysis-only reordering).

## Gate 5 — A cohort

- Label: `ACTIVATION_IMMEDIATE_PRODUCTIVE_CAPTURE`
- Gross activation rises with `V_A`, but free-A fraction stays tiny; productive sinks absorb additional A.

## Gate 6 — Product yields

- Precursor conversion bottleneck flag: false (under resource-limited baseline, S does not become the primary next target while N/F delivery dominates).

## Gate 7 — Spatial overlap

- Ω computed on activation vs local A demand; no spatial-allocation failure declared.
- Conservative mixing totals conserved (diagnostic only).

## Gate 8 — Free-pool interpretation

- `FREE_A_POOL_CAUSALLY_DEFICIENT` under ordinary delivery (`Q_A` / low pool with collapsed retention).
- Note: with healthy N/F (Gate 9), topology is capable — free-A deficiency is resource-conditioned.

## Gate 9 — Maximum coupled activation control

- Healthy local N/F + unlimited activation substrates + schema-2 4×:
  - A retention ≈2.00
  - Outcome: `COUPLED_ACTIVATION_TOPOLOGY_CAPABLE`

Activation topology can support coupled A when N/F are not delivery-limited.

## Gate 10 — Route

**Route R** → `D051_RESOURCE_THROUGHPUT_LIMIT`

Next directive may review only:

- N/F delivery / permeability / reservoir flux, or  
- activation stoichiometric yield  

Do **not** invent another activation law, `C_star`, product inhibition, or activation buffer from this result alone.

## Secondary findings

| Item | Result |
|------|--------|
| Requested vs accepted | Parity on accepted steps; scales with `V_A` |
| Cap classification | `ACTIVATION_EXTENT_SCALES_WITH_V_A` |
| N/F ceiling | Ordinary delivery insufficient; healthy N+F restores |
| Gross activation vs `V_A` | Increases (~502→779 at 10k) |
| A residence / destinations | Immediate productive capture |
| Product yields | Not primary under Route R |
| Spatial overlap | No allocation failure |
| Operator order | No split defect |
| Free-pool class | Causally deficient under ordinary delivery |
| Max control | Topology capable with healthy N/F |

## Tests / artifacts

- `cargo test -p chemistry-core --test d051_tests --release` — 13/13 PASS
- Artifacts: `digital-protocell/experiments/generated/d051/`
- Pipeline: `D051_MAX_ACCEPTED=10000`

## Deviations

- Diagnostic controls clamp interior N/F or raise `reservoir_rate` once; not promoted.
- Cohort tracer is window-integrated ledger fractions (noncausal), not a new field.
- Control horizons after Gate 0 capped at 4000 for cost; Gate 0 used full 10000.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`
- Next execution started: false

## Next directive

Review N/F delivery, permeability, reservoir flux, or activation stoichiometric yield under frozen activation schemas. No new activation topology authorized by D-051.
