# D-091 Metabolic Reserve and Ecological Timescale Closure

## Verdict (smoke campaign)

**Primary conclusion:** `D091_METABOLIC_RESERVE_QUALIFIED_COMPOSITIONAL_SELECTION_REJECTED`

Reserve physiology establishes the missing intermediate metabolic timescale. Valid pulse-lean (H) and abrasion (B) ecological couplings are reachable under the new schema (H: period=0.5×T_maint, 3 cycles, scarcity with A/R dip; B: 5% abrasion, repair-before-fission, sublethal). Full compositional `C_H/C_B` selection, mutation-driven adaptation, and environmental reversal were **not** established. The scalar catalyst tradeoff is closed as the evolutionary substrate. Phase 3 is **not** authorized.

**Note:** Campaign evidence is from `D091_SMOKE=1`. Full non-smoke matrices remain outstanding for production seal/tag.

## Entry

| Item | Value |
|---|---|
| Branch | `phase2-growth-division-inheritance` |
| Start commit | `d4835e6` |
| Start tag | `D-090-selection-ecology-invalid` |
| Schema | `autopoietic_material_mesh_metabolic_reserve_v1` |
| Fields | `mesh_vertices_edges_catalyst_composition_reserve_v1` |

## Reserve chemistry

\[
J_{store}=k_{store}\,q(C)\,\frac{A^2}{K_{store}^2+A^2}\left(1-\frac{R}{R_{max}}\right),\quad A\to R
\]

\[
J_{release}=k_{release}\,q(C)\,\frac{R}{K_R+R}\,\frac{K_{low}}{K_{low}+A},\quad R\to A
\]

\[
J_{R,loss}=k_{R,loss}R,\quad R\to W
\]

\[
J_{growth}=y_g\,g_{build}\,q(C)\,\frac{R}{K_{growth}+R}\,h(\varepsilon),\quad R\to m+W
\]

Frozen D-088 surplus-A growth remains when `reserve.enable=false`.

## Selected parameters (H=2×maintenance horizon)

Derived from Phase 1 replacement horizon `1/k_turn`, maintenance horizon from A-demand, median/Q25 viable A, and median fission A-cost.

See `experiments/generated/d091/reserve_schema/selected.json` and `preservation/parameter_derivation.json`.

Identity includes all reserve parameters. Old unmarked snapshots are rejected when reserve chemistry is enabled.

## Gate summary (smoke)

| Gate | Result |
|---|---|
| 0 Preservation / schema | PASS |
| 1 Conservation / causality | PASS |
| 2 Phase 1 maintenance | PASS |
| 3 Timescale separation | PASS (selected H=2) |
| 4 Reproduction requalification | PASS |
| 5 H/B ecology | PASS |
| 6 Identity/position | PASS |
| 7 Selection campaign | FAIL |
| 8 Mutation adaptation | provisional smoke PASS |
| 9 Reversal | FAIL |
| 10 Stability | FAIL (depends on selection) |

## Scientific records

- `STEADY_FLOW_SELECTION_ECOLOGY_CLOSED` (D-090)
- `INSTANTANEOUS_A_GROWTH_COUPLING_IDENTIFIED` (D-090)
- `PHASE2_METABOLIC_RESERVE_ARCHITECTURE_AUTHORIZED` → **qualified**
- `METABOLIC_RESERVE_PHYSIOLOGY_QUALIFIED`
- `COMPOSITIONAL_C_H_C_B_SELECTION_CLOSED_REJECTED`
- Phase 3 **not** authorized

## Next

`D-092: Minimal Catalytic Template Heredity` — `next_execution_started: true`. Do not increase `σ`.

## Tests

```bash
cargo test -p chemistry-core --test d091_tests --release
D091_SMOKE=1 cargo run --release -p experiment-runner -- d091 pipeline
```

Artifacts: `experiments/generated/d091/` (symlink to 1TB archive).
