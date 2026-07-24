# D-083 — Conservative Dynamic Edge-Membrane Migration Repair

## Mission

Repair cut-cell edge-membrane migration when the structural interface moves, using **local geometric continuity only**. Structural kinetics remain frozen and out of scope.

## Entry

- Start: `01d9afd` / `D-082-edge-activation-integration-repaired`
- Dynamic-interface failure root: Gate 8 rebuilt support without migrating B (coverage fail; conservation held).

## Operator

`chemistry-core/src/edge_migration.rs` — `migrate_bound_across_support`:

1. Retain B on overlapping old/new faces; spill capacity excess to local appear neighbors, else L.
2. Transfer B from disappearing faces to new faces within a bounded local hop neighborhood (shared-cell halo + support adjacency on old∪new).
3. Prefer newly appearing targets; unmatched remainder → nearby L.
4. Local continuity fill: appear faces pull surplus from nearby retained faces.
5. Clear residual unsupported B → L.
6. Conserve `L+B`. No analytic circle, global remapping, target ring, or nonlocal projection.

Wired into `gate8_dynamic_interface` (migrate=true). Unmigrated path retained as `gate8_dynamic_interface_unmigrated` for Gate 0 reproduction.

## Gates

| Gate | Result |
|------|--------|
| 0 Reproduction | Unmigrated dynamic fails; D-082 activation/affordability/static/transport retained |
| 1 Provenance | B on disappearing fragments identified as first divergence |
| 3 Synthetic motion | Translation, expansion, contraction, mild bulge/indent pass |
| 4 Autonomous R16/R22/R32 | Migrated dynamic + radius schedules pass |
| 5 Regressions | Prior edge-network gates under D-081 reserve contract + D-082 activation (not obsolete D-080 Gate7 free-L assay) |
| 6 Structural separation | Universally positive drive (independent blocker) |

## Gate5 note

An earlier incomplete run reported `D083_EDGE_NETWORK_REGRESSION` because Gate5 called obsolete `gate7_damage_and_causality`, which still recovers via free-L rebinding (`no_a_fails=false`). D-081 replaced that assay with finite-reserve repair/depletion/A-causal replenishment; D-082 recorded Gate7 as `PASS_AFTER` on that contract. Gate5 now uses those D-081/D-082 checks.

## Conclusion

`D083_EDGE_DYNAMIC_MIGRATION_REPAIRED` + `STRUCTURAL_RESTORING_BLOCKER_REMAINS`

Not `D083_EDGE_NETWORK_BOUNDARY_QUALIFIED` — restoring structural crossing is absent.

## Status

- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Production: `REQUIRES_REMEDIATION`

## Next

Audit structural gain/loss under the qualified edge boundary and select one bounded structural-homeostasis architecture. Do not raise activation or change membrane migration.
