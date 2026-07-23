# D-081 — Edge-Membrane Reserve Provenance and Replenishment Audit

## Conclusion

`D081_EDGE_MEMBRANE_PRODUCTION_METABOLICALLY_INFEASIBLE` (stopped at Gate 5)

**Conclusion status (D-082):** `SUPERSEDED_BY_D082_ACTIVATION_SUPPLY_AUDIT`

D-081 Gate 5 used a scalar `activated` bolus with no canonical N/F→A path; metabolic infeasibility is **not** proven. D-082 Route I restored affordability after integrating activation supply. D-080 Gate 7: `PASS_AFTER_D081_RESERVE_CAUSALITY_AUDIT`.

Reserve causality through Gate 4 is qualified: the D-080 seed is a lawful finite free-`L` reserve; one-time reserve-only repair conserves \(M_L+M_B\); repeated damage eventually fails without hidden regeneration; only normal A→L metabolism replenishes reserve and supports another repair. Sustained production under the frozen A→L law collapses A (`a_retention≈0`), so replenishment is metabolically unaffordable. Binding was **not** made to consume A.

## Entry

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| D-080 commit / tag | `f5dc5a5` / `D-080-edge-network-requalification-fail` |
| D-081 start | `f5dc5a5` |
| D-080 Gate 7 entry | `PROVISIONAL_PENDING_RESERVE_CAUSALITY_AUDIT` |
| Seed contract | `EDGE_MEMBRANE_SEED_CONTRACT_V1` |

## Seed classification

`CAPACITY_VALID_FINITE_RESERVE`

| R | initial \(M_L\) | initial \(M_B\) | full-ring capacity | over-capacity |
|---|----------------|----------------|--------------------|---------------|
| 16 | 155 | 0 | 124 | +25% |
| 22 | 215 | 0 | 172 | +25% |
| 32 | 315 | 0 | 252 | +25% |

Same face density `SEED_DENSITY=1.25` at every radius; no completed B ring; no hidden material at init.

## Gate results

| Gate | Result |
|------|--------|
| 0 D-080 reproduction | **PASS** — geom/connected cov=1.0; transport; replacement; recovery=1.0; no-A and no-production recover |
| 1 Seed provenance | **PASS** — `CAPACITY_VALID_FINITE_RESERVE` |
| 2 Reserve-only repair | **PASS** — recovery=1.0; \(M_L\)↓ / \(M_B\)↑; \(M_{mem}\) conserved; closed |
| 3 Reserve depletion | **PASS** — 9×10% quanta; recovery falls to ≈0.90 and opens; no hidden regen; cum rebound ≤ starting free |
| 4 Energy-causal replenishment | **PASS** — only normal metabolism increases \(M_{mem}\); A↔L stoichiometry OK; no-A / knockout flat; post-replenish repair=1.0 |
| 5 Metabolic affordability | **FAIL** — A retention ≈0 under continuous frozen A→L; C ret=1; L/B bounded+closed; N/F/W perms OK |
| 6 Corrected causality | not reached (Gate 5 stop) |
| 7 Resume D-080 | skipped |

## D-080 Gate 7 status

Remains `PROVISIONAL_PENDING_RESERVE_CAUSALITY_AUDIT` (not upgraded to `PASS_AFTER_D081_RESERVE_CAUSALITY_AUDIT` because Gate 6 did not pass).

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`

## Next directive

Do not raise activation production. Do not add A-for-binding. Under frozen edge kinetics, revise membrane A→L yield/demand (or couple a real A source) so reserve replenishment remains affordable. Then re-enter Gate 5→6→7.

`next_execution_started`: false
