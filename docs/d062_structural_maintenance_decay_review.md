# D-062 — Long-Horizon Structural Maintenance and Decay Review

## Status

**Complete.** Primary conclusion: `D062_NO_LOCAL_STRUCTURAL_MAINTENANCE_LAW` (Route N).

Diagnostic / shadow-only. No production biology change.

## Question

Is the D-061 positive structural drive (capped at 1,000 accepted steps):

1. a short-horizon transient that later restores via resource depletion;
2. a structural-decay execution/accounting defect;
3. a globally underpowered but otherwise valid constitutive decay;
4. evidence that structural loss must depend locally on metabolic maintenance availability?

## Answer (evidence)

| Gate | Result |
|---|---|
| Gate 0 D-061 reproduction | PASS — FixedGeometry immobile; DynamicStructure `POSITIVE_ALL_RADII` @ 1,000 |
| Gate 1 decay parity | PASS — no execution/accounting defect; fixed/dynamic counterfactual loss equal |
| Gate 2 scaling | `GAIN_AND_LOSS_VOLUME_MATCHED` — \(p_G\approx1.18\), \(p_L\approx1.22\) |
| Gate 3 long horizon | `EXISTING_STRUCTURAL_PERSISTENT_RUNAWAY_GROWTH` @ 5,000 and 10,000 accepted steps |
| Gate 4 scalar \(m_d^\star\) | span \(\approx1.28\le3\times\) but **flat vs radius** → cannot create restoring sign change |
| Gate 5–7 candidates | B rejected (nonportable/flat); C grid finds no restoring crossing |
| Gates 8–9 basin | skipped — no qualified frontier |
| Gate 10–11 | causality/foundational OK; Route N |

Authoritative completed horizon: **10,000** (5,000 archived). 25k/50k not completed; 5k→10k late \(dR/dt\) stayed positive at every tested radius with no restoring crossing.

## Frozen context

- Starting commit/tag: `1d4e2bb` / `D-061-structural-execution-size-revalidation`
- Shadow carrier only: \(k_T = 1.4346157818803311\)
- Organism assays: `StructureEvolutionMode::DynamicStructure`
- Structural synthesis, carrier kinetics, activation, production defaults: unchanged

## Route N implication

Next directive should close the external-carrier / small-size route and return to internally generated membrane area or another conservative import architecture. Do not implement a structural-maintenance kinetic change from D-062.

## Authorization until a later implementation directive passes

- Selected architecture: none
- V15: unauthorized
- Structural-maintenance change: unauthorized
- Internal membrane: unauthorized
- D-008 Stage E: `BLOCKED_NOT_RECOVERED`
- Phase 1: `PHASE1_SELF_MAINTENANCE_PARTIAL`
- Stage F: not authorized
- Production: `REQUIRES_REMEDIATION`

## Artifacts

`digital-protocell/experiments/generated/d062/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d062/`
