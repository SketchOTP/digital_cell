# DC-DEV-020-M1-REPLAN-002-R1

## Versioned maturation-coupled production candidate

This qualification implements the Architect-accepted REPLAN-002 shadow as the
experimental `MaturationCoupledV4` contract. `edge.m` remains the single
authoritative total structural-material amount. `edge.m_young` records newly
synthesized material, and mature material is derived as
`max(edge.m - edge.m_young, 0)`.

New structural build enters the young pool. Maturation transfers young material
to mature material using the existing structural-turnover timescale and does
not change total material. Only mature material supplies the load-bearing rest
length and ordinary structural turnover. Explicit damage removes total
material proportionally, and remesh/fission preserve lifecycle composition.
HistoricalV1, ConservativeV2, and GeometryConservativeV3 retain their prior
semantics; the production default remains ConservativeV2 with reserve OFF.

## Structural-state ownership map

The R1 audit classifies every V4 structural-material mutation path:

| Path | `edge.m` authority | `edge.m_young` behavior |
| --- | --- | --- |
| Seed/deserialization | existing total is authoritative | zero for historical snapshots; V4 loads the saved lifecycle value |
| Ordinary build | add newly produced physical material | add the same amount as young |
| Maturation | unchanged | subtract the matured amount |
| Ordinary turnover | subtract mature material | unchanged |
| Explicit damage/rupture | remove real total material | remove proportionally or clear on rupture |
| Remesh split/merge | split or sum total material | apply the identical split or sum |
| Rebond | assign existing physical bond amount | V4 rebond material is newly load-bearing state and enters young |
| Fission | partition inherited physical material and add cross-bonds | partition inherited lifecycle state; cross-bonds enter young |

No second authoritative structural-material total is introduced. V4 invariants
require finite `0 <= edge.m_young <= edge.m` after each accepted operation.

## Qualification status

The runner starts from accepted REPLAN-002 authority
`4becff4fff7d096c70468b759ace09f747c4eb56` and uses the sealed REPLAN-002
source schedule. The local result is fail-closed pending exact-head Linux CI:

- immutable shadow parity: PASS (`+1.3323122170185968` organized delta);
- fed moving homeostasis: PASS;
- no-reset recovery: PASS;
- starvation structural decline: PASS;
- material closure: PASS (maximum fed residual `2.712413627037335e-13`);
- V2 D-087: `8/8`;
- V3 D-087: `8/8`;
- V4 D-087: `6/8`, with dual-retention and starvation gates failing.

Accordingly, the candidate is not qualified and no production selection or M1
closure is claimed. The bounded classification is
`M1_MATURATION_COUPLED_PRODUCTION_PRESERVATION_REGRESSION` unless exact-head
CI identifies an infrastructure failure.

## Evidence and boundaries

Compact evidence is stored under
`digital-protocell/experiments/generated/dcdev020m1replan002r1/`.
Dense per-step ledgers are stored on Atlas under
`/srv/ATLAS/100_ACTIVE/Projects/DIGITAL_CELL/evidence/dcdev020m1replan002r1/dense/`.

This is a bounded qualification only. It does not authorize D-087 repair,
parameter search, a physical-death follow-up, reserve or recycling changes,
M2, behavior, evolution, or DC-DEV-021.
