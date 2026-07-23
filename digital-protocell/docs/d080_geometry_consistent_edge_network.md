# D-080 — Geometry-Consistent Edge-Network Repair and Requalification

## Conclusion

`D080_EDGE_NETWORK_REPAIR_OR_CAUSALITY_FAILURE` (stopped at Gate 7)

**Gate 7 status (D-081 entry):** `PROVISIONAL_PENDING_RESERVE_CAUSALITY_AUDIT`

Geometry defect repaired. Self-assembly, transport, and replacement pass under the cut-cell support graph. Damage recovery reaches 1.0, and no-A / no-production controls also recover via local free-`L` rebinding. D-080 treated this as causality failure; D-081 audits whether that is valid finite-reserve repair before requiring A dependence of binding or rejecting the edge network.

## Entry

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Start | `99c0236` / `D-079-edge-network-boundary-fail` |
| D-079 | `D079_EDGE_NETWORK_SELF_ASSEMBLY_FAILURE` |
| Record | `D079_SELF_ASSEMBLY_FAILURE_PENDING_GEOMETRIC_SUPPORT_AUDIT` |

## Root cause (Gate 1)

Legacy cell-endpoint adjacency fragments collinear Cartesian boundary faces. Cut-cell corner adjacency yields a closed geometric support graph.

Gap classification: `LEGACY_CELL_ENDPOINT_ALIASING`

## Gate results

| Gate | Result |
|------|--------|
| 0 D-079 reproduction | **PASS** — R16/22/32 ≈0.848/0.889/0.923; closed=false; off≈0 |
| 1 Gap provenance | **PASS** — legacy fragmented; cut-cell closed |
| 2 Cut-cell support | **implemented** (`edge_support.rs`) |
| 3 Geometry qualification | **PASS** — length err ~0.1%; geom cov=1; closed; offset inv ≪2% |
| 4 Self-assembly | **PASS** — occupied=connected=1.0; closed; off=0; `k_lateral_scale=1.0` |
| 5 Transport | **PASS** — Stage A envelope at R16/22/32 |
| 6 Replacement | **PASS** — tracer turnover; closed retained |
| 7 Damage / causality | **FAIL** — recovery=1.0 and hole raises perm, but no-A and no-production controls do not fail |
| 8–9 | skipped (stop rule) |

## Frozen mechanisms

D-079 bind/unbind/produce/damage equations and default parameter vector unchanged. No particles, no ring prescription, no global coverage in chemistry.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`

## Next directive

Keep cut-cell support fixed. Audit local rebinding / A dependence so no-A and reserve-depleted no-production controls fail while ≥0.95 recovery still requires metabolism. Do not prescribe a ring. Do not authorize particles inside this repair path.

`next_execution_started`: false
