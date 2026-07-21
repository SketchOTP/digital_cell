# D-063 Environmentally Connected Membrane Invagination Architecture

## Primary conclusion

`D063_CONNECTED_MEMBRANE_SHADOW_REPAIR_FAILURE`

Route label: `Route_I_inconclusive` (failure-mode conclusion; no architecture authorized).

## Governance

| Item | Value |
|---|---|
| Project directive | D-063 |
| Agent memory | `D-20260721-d063-environmentally-connected-membrane-invagination-architecture` |
| Starting commit | `47f2abb` |
| Starting tag | `D-062-structural-maintenance-decay-review` |
| Frozen `k_T` | `1.4346157818803311` (shadow only) |
| V15 | unauthorized |
| Morphogenesis | unauthorized |
| Internal membrane | unauthorized |
| Stage E | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_SELF_MAINTENANCE_PARTIAL` |
| Stage F | not authorized |
| Production | `REQUIRES_REMEDIATION` |

## What passed

- Gate −1 workspace isolation (unrelated `.cursor/rules` / `AGENTS.md` dirty files excluded).
- Gate 0 D-062 Route N reproduction (`p_G≈1.18`, `p_L≈1.22`, gain/loss band, no restoring law).
- Gate 1 topology classifier + reservoir flood-fill; closed vesicles sealed from exterior.
- Gates 2–4 explicit families A–E; physical connected-area amplification without free multipliers; S/P material identity.
- Gate 5 corrected carrier operator parity; closed faces contribute zero environmental import.
- Gate 6 FixedGeometry capacity: `p_A≈1.00`; connected invaginations/channels raise χ above 1.05 at R16/R22/R32.
- Gate 7 radial usable fraction ≈1 under the depletion model; deep branched access weaker but not the stop.
- Gate 9 short DynamicStructure probe: connected length ratio ≈1 (`TOPOLOGY_PERSISTS_PASSIVELY` at 200 accepted steps).
- Gate 10 seed-affordable incremental bootstrap with `R_i≈8.4` (`CONNECTED_AREA_BOOTSTRAP_FEASIBLE` on paper).
- Gate 11 channel-entrance seal updates connected invagination length.

## What failed

Gate 8 coupled prebuilt-geometry shadow (FixedGeometry + shadow carrier):

- Radial R22 @ target 2500: accepted 1076 then rejection cascade (`steps_ok=false`).
- A retention ≈0.40 (below 0.80).
- χ proxy ≈0.19 (below 1.05) despite nonzero import.
- Mature S mass declined 368 → ≈227 under existing turnover.
- Carrier-disabled control remained numerically quieter (no cascade at 250 steps) but provides no import rescue.

Therefore explicit connected area raises **static** carrier capacity, but does **not** qualify a coupled shadow maintenance repair under frozen biology + one global `k_T`.

## Architecture comparison (diagnostic)

| Family | Role | Connected α (R22) | Notes |
|---|---|---|---|
| Smooth | baseline | ≈1.27 (grid vs 2πR) | capacity floor |
| Corrugated | perimeter control | ≈1.50 | modest gain |
| Radial invaginations | primary candidate | ≈2.66 | static χ≫1.05; shadow fails |
| Branched channels | larger α | ≈3.94 | not required once invagination α suffices statically |
| Closed vesicles | negative control | closed length >0 | zero environmental carrier area |

## Authorizations

- Selected membrane topology for implementation: **none**
- Production carrier: **not enabled**
- Morphogenesis law: **not authorized**
- Closed internal membrane import: **rejected**

## Next directive

Diagnose Gate-8 shadow rejection / A-retention failure under prebuilt connected geometry (numerical vs metabolic vs membrane-turnover), without implementing morphogenesis or production carrier. Static area–throughput result (`p_A≈1`) remains available as capacity evidence.

## Artifacts

`digital-protocell/experiments/generated/d063/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d063/`

## Tests

`cargo test -p chemistry-core --test d063_tests` → 11/11 PASS
