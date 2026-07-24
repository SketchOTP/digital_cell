# D-085 Decisive Structural Closure Campaign

## Entry

- Project directive: D-085
- Agent memory: `D-20260723-d085-decisive-structural-closure`
- Starting commit: `ed7be5e`
- Starting tag: `D-084-edge-structural-homeostasis-fail`
- Record: `D084_FIXED_RADIUS_RESTORING_CROSSING_QUALIFIED_DYNAMIC_BASIN_PENDING`

## Clarification on D-084

D-084 found a restoring fixed-radius crossing (η≈0.07535, k≈0.01963) then skipped the multi-seed dynamic campaign with `D084_SKIP_LATE_GATES=1`. That incomplete execution is **not** a scientific rejection of mixed turnover. D-085 finishes the experiment.

## Campaign

1. **Phase A** — 15 fully dynamic organisms (R18/R22/R26 × seeds 1–5) under sealed D-084 mixed turnover. No `D084_SKIP_LATE_GATES`.
2. **Parity** — freeze accepted geometry; recompute structural flow; require direction agreement with last-step runtime ledger.
3. **Phase B/C** — if basin fails and parity holds, one local curvature/strain mechanochemical architecture (weak/center/strong).
4. **Phase D** — energy/waste, puncture+controls, Stage E joint assay — only for a qualified basin.

## Equations (Phase B)

\[
f_\kappa = \frac{|\kappa|}{K_\kappa+|\kappa|},\quad
f_s=\tanh(s/K_s)
\]

\[
r_+ = r_+^0\bigl[1+g_\kappa f_\kappa-g_s f_s\bigr],\quad
r_- = r_-^0\bigl[1+g_s f_s\bigr]
\]

Response clamped to \([1/2,2]\). No target radius.

## Artifacts

`experiments/generated/d085/` → `/mnt/storage1tb/cache/project-artifacts/digital_cell/experiments/generated/d085/`

## Env

- `D085_MAX_ACCEPTED` (default 75000)
- `D085_WINDOW` (default 5000)
- `D085_SMOKE=1` — one R22×seed1 short run (non-scientific)

## Result

Pipeline primary: **`D085_PHASE_FIELD_STRUCTURAL_SUBSTRATE_REJECTED`**

| Item | Value |
|------|-------|
| D-084 preservation | Fixed-radius restoring crossing qualified; dynamic basin was pending |
| Phase A (15-run) | FAIL — 0/5 seeds at every radius; A retention ≈0.26 |
| Failure class | `RESOURCE_COUPLING_REVERSAL` |
| Static/dynamic parity | PASS (runtime vs frozen field flow agree) |
| Mechanochemical | weak/center/strong all fail same A-retention floor |
| Stage E | Not recovered |
| D-008 | `BLOCKED_NOT_RECOVERED` |
| Phase 1 | `PHASE1_STRUCTURAL_SUBSTRATE_CLOSED` |
| Production | `REQUIRES_SUBSTRATE_REDESIGN` |

### Governing closure

Scalar structural-rate tuning, mixed bulk/interface turnover, A-deficit turnover, and curvature/strain feedback on the current Cahn–Hilliard structural substrate are closed for Phase 1. Next work must redesign the organism body as an explicit conserved cellular or mesh material system.

Artifacts: `experiments/generated/d085/result.json`.
