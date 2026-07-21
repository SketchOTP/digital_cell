# D-061: Structural Constraint Execution Repair and Size-Basin Revalidation

## Primary conclusion

`D061_UNMODIFIED_STRUCTURAL_RUNAWAY_GROWTH` (Route G)

## Execution repair

`D061_STRUCTURE_EXECUTION_REPAIR_QUALIFIED`

Typed mode:

```rust
enum StructureEvolutionMode {
    FixedGeometry,     // assays: accepted structure does not mutate φ
    DynamicStructure,  // organism: accepted structure mutates φ
}
```

Legacy `enforce_structure_constraint` remains as a synced mirror
(`true` ↔ FixedGeometry, `false` ↔ DynamicStructure). Mode participates in
configuration identity and fail-closed resume checks.

## D-060 defect reproduced (Gate 0)

Under FixedGeometry: analytic \(G_\phi - L_\phi > 0\) at every tested radius while
coupled \(dR/dt \approx 0\) because `apply_phi = false`.

## Corrected dynamics (DynamicStructure)

Unchanged structural kinetics + frozen shadow \(k_T = 1.4346157818803311\):

- drive surface: `POSITIVE_ALL_RADII`
- short trajectories: runaway growth across the physical radius domain
- no restoring zero crossing → Gates 8–9 skipped by stop rule

## Authorization (unchanged)

- selected architecture: none
- V15: unauthorized
- structural kinetic changes: unauthorized
- internal membrane: unauthorized
- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- production: `REQUIRES_REMEDIATION`

## Next directive

Review existing structural decay / maintenance under unchanged carrier kinetics
(Route G). Do not change carrier kinetics. Do not implement V15.

## Run

```bash
cargo test -p chemistry-core --test d061_tests
D061_MAX_ACCEPTED=2500 cargo run -p experiment-runner --release -- d061 pipeline
```

Artifacts: `digital-protocell/experiments/generated/d061/` →
`/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d061/`.
