# D-079 — Conserved Edge-Network Membrane Feasibility

## Conclusion

`D079_EDGE_NETWORK_SELF_ASSEMBLY_FAILURE` (stopped at Gate 2)

## Scope amendment

`PHASE1_EDGE_NETWORK_BOUNDARY_RESEARCH_AUTHORIZED`

Explicit lipid particles, MD, and a second engine remain excluded.

## Mission

Implement and evaluate one experimental discrete boundary substrate: free membrane units `L` at cells and bound units `B` on horizontal/vertical grid faces. Stop at the first mandatory gate failure.

## Entry

| Item | Value |
|------|-------|
| Branch | `d008-membrane-metabolic-closure` |
| Start | `039044f` / `D-078-boundary-substrate-downselect` |
| D-078 | `D078_CONTINUUM_BOUNDARY_SUBSTRATE_REJECTED` |

## Equation / schema identity

| Field | Value |
|-------|-------|
| `equation_version` | `edge_network_membrane_v1` |
| `field_schema` | `edge_network_faces_v1` |
| schema version | 1 |

Production continuum defaults unchanged. Legacy snapshots cannot resume into this schema.

## Architecture (experimental)

\[
J_{\mathrm{bind},f}=k_{\mathrm{bind}}\,q(C)\,I_{\phi,f}\,\mathbf{1}_{\mathrm{cross},f}\,L_f\bigl(1-B_f/B_{\max}\bigr)
\]

\[
J_{\mathrm{unbind},f}=k_{\mathrm{unbind}}\,B_f\,r_f
\]

Local crossing indicator \(\mathbf{1}_{\mathrm{cross},f}\) uses only face-endpoint \(\phi\) values. Lateral transfer is restricted to pairs of local crossing faces. Damage: \(B\to W\). Production: \(A\to L\).

## Gate results

| Gate | Result |
|------|--------|
| 0 Preservation / schema | **PASS** |
| 1 Conservation / invariants | **PASS** |
| 2 Static interface self-assembly | **FAIL** — interface-localized binding with off-interface≈0, but coverage ≈0.85–0.92 < 0.95 and no closed cycle across R16/R22/R32 |
| 3–8 | skipped (stop rule) |

## Scientific conclusion

A conserved lattice-edge membrane can obey local conservation and localize to structural crossing faces without a continuum membrane field. Under one global local-parameter set, it does **not** spontaneously form a closed edge network meeting the ≥0.95 coverage / closed-cycle Gate 2 requirement without prescribing a ring. Do not prescribe the missing ring.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL` (edge-network research authorized; substrate not qualified)
- Production: `REQUIRES_REMEDIATION`
- Production continuum biology: unchanged

## Next directive

Do not prescribe a closed edge ring. Decide whether to revise local edge kinetics within the authorized discrete scope, reject the discrete edge-network substrate (`D079_DISCRETE_EDGE_NETWORK_REJECTED` path), or authorize a separate coarse-grained particle-membrane research phase.

`next_execution_started`: false

## Evidence

- `chemistry-core/src/edge_membrane.rs`
- `chemistry-core/src/d079_analysis.rs`
- `chemistry-core/tests/d079_tests.rs`
- `experiment-runner/src/d079.rs`
- `experiments/generated/d079/`
