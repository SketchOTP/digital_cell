# Clean scientific base

| Item | Required value |
|---|---|
| Base commit | `0d2c404c0874d5430dd5d01dbdcc059a842dd689` |
| Base branch | `strategy/d094-evolutionary-requalification` |
| Work branch | `strategy/dc-dev-001-architecture-selection` |
| Scientific source carryforward | none from later D-096/R4 stack |
| Current worktree | `/home/sketch/Projects/digital_cell_dcdev001` |

The new worktree was created from the exact base commit. Existing dirty operational work on `/home/sketch/Projects/digital_cell` and the R4 branch remain untouched. The only active changes in this branch are governance reconstruction and this architecture-selection package.

## Frozen authority

The material mesh, chemistry, transport, metabolism, growth, fission, certifier, and Phase 1 evidence remain authoritative. Future implementation must use their public bounded interfaces. No file under the scientific crates, experiment source, or workflow source is changed by this directive.

## Excluded state

Later R4/D-096 source, generated evidence, workflows, and mutable claims were not copied. Historical R4 governance is retained under `.agent/legacy/pre-dev001-r4-governance-20260815/` for audit, but it is not active state.

