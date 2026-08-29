# DC-DEV-020-M1-R2-R2 — Topology-Death Closure

Directive: `DC-DEV-020-M1-R2-R2-TOPOLOGY-DEATH-CLOSURE-001`

Entry head: `40a066424a5a0fe08db9609c4ec71a708b44115f`

This is an observer-only continuation from the accepted M1-R2 endpoint. It
replays both frozen starvation arms to accepted step `20,480`, then continues
the unchanged production reaction path past observer collapse until the
existing edge-material rupture rule actually marks an edge. It does not treat
`activated_catalyst_collapse` or `starvation_collapse` as topology death.

## Frozen execution

- Production 4× arm: declared `k_a_decay = 0.008`, existing starvation factor
  4×.
- Ordinary-decay arm: declared `k_a_decay = 0.002`, existing starvation factor
  4×.
- Rupture search: at most `1,000,000` additional accepted reaction steps.
- Refeeding: `5,000` accepted steps from the exact ruptured state.
- Finite resource: `N = F = 14.588954880632265`, center `[4.8, 0.0]`, radius
  `1.5`.
- Source-capacity shadow: the already-qualified M1-R1 paired internal N/F→A
  upper-bound shadow, applied before each unchanged reaction step.

The assay changes no chemistry-core source, death rule, resource law,
transport, uptake, reserve, recycling, salvage, M2, behavior, or DC-DEV-021
implementation.

## Topology boundary

The authoritative physical-failure event is an existing `reactions_step`
edge rupture (`edge.m < mesh.bond_threshold`), not observer viability. The
post-rupture branches record `closed_intact`, rupture count, physical runtime
validity, delivery, and strict material closure. A branch passes topology
closure only if it preserves the ruptured state rather than silently
reconstituting an intact body.

## Result

Both arms reached actual edge rupture while runtime geometry remained valid:

| Arm | First edge rupture | Ruptured edges at state | Closed intact | Runtime valid |
| --- | ---: | ---: | --- | --- |
| production 4× | 124,249 | 24 | false | true |
| ordinary decay | 124,717 | 24 | false | true |

Both exact ruptured states remained `closed_intact = false` after both the
ordinary finite refeed branch and the source-capacity upper-bound branch for
5,000 accepted steps. Strict world/organism closure residuals remained below
`1e-8`. Because rupture prevents the finite boundary from delivering through
the ruptured perimeter, the evidence records zero delivered N/F in those
branches rather than implying successful refeeding.

The bounded classification is:

```text
M1_TOPOLOGY_DEATH_ESTABLISHED
```

This establishes irreversible topology loss under the frozen chemistry path;
it does not authorize a production repair, recycling/salvage, M2, or
DC-DEV-021.

## Evidence

Compact authoritative artifacts are in
`experiments/generated/dcdev020m1r2r2/`. Dense ledgers are not committed.
The scoped workflow also reruns actual D-087 and the Phase-1, D-091, D-088,
and evolution-harness preservation suite before architect review.
