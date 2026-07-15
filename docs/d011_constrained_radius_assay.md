# D-011 Constrained-Radius Assay

## Purpose

Replay the exact D-008 Stage E failed rates under **transport-coupled** dynamics with
**fixed φ** (constrained radius). Stage E used `STATIC_FIELD_BALANCE`; D-011 uses
`D008StageMode::ConstrainedRadius` (`try_d008_constrained_radius`).

## Geometry and seed

- Circular tanh φ profile, width 2, prescribed radius R
- Interior seed: C=0.4, A=0.2, N/F=0.2, W=0.5; exterior reservoir levels
- Membrane seeded at `I(φ)`; M evolves during assay
- Seed: 1

## Dynamics per accepted substep

| Field | Behavior |
| --- | --- |
| φ | **Fixed** — copied current→next unchanged |
| C, N, F, W, A | Reservoir exchange + old-state selective transport + activated metabolism |
| M | Production/decay/detachment/diffusion (`evolve_fixed_membrane`) |
| Structure chemistry | **Virtual** — `r_structure = k_d008_structure × A × I(φ)`, `r_structure_decay = k_structure_decay × φ`; consumes A, produces W; **does not change φ** |

## Constraint ledger (observer-only)

- `virtual_structure_flow = ∫(r_structure − r_structure_decay)`
- `structure_constraint_flux = −virtual_structure_flow`
- No feedback into dynamics

## Quasi-steady criterion

- Rolling windows (production: 10_000 steps; unit tests: 1_000)
- Three consecutive converged windows required
- Slopes ≤ 1e−4 for C/A/M masses and mean interior N/F/W
- Reaction/transport window totals within 5% between consecutive windows

## Joint overlap (§9–10)

All four Q ∈ [0.98, 1.02], |g| ≤ 1e−4, retention C/A ≥ 0.80, membrane localization ≥ 0.90,
N/F influx > 0, W efflux > 0.

## CLI

```bash
cargo run -p experiment-runner --release -- d011 run \
  --output experiments/generated/d011 \
  --max-steps 50000 \
  --window-size 10000
```

Use `--quick` for 5_000-step smoke (recorded in artifacts).

## Artifacts

`experiments/generated/d011/attempt_NNN/` with `failed_candidate_replay/R*/result.json`.
