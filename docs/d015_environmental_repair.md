# D-015 environmental repair

## Branch

**Branch B item 1** — increase waste sink region (W-only).

Shared `reservoir_mask` widening would increase N/F supply and is forbidden.

## Change

| Field | Baseline | Repaired R22 |
| --- | --- | --- |
| `waste_sink_inner_radius` | 83.0 (`DISH_RADIUS − RESERVOIR_WIDTH`) | **30.0** (`R + NEAR_EXTERIOR_BUFFER`) |
| N/F reservoir mask | unchanged | unchanged |
| Organism rates / β / D / η | frozen | frozen |

## Identity

| Hash | Baseline | Repaired |
| --- | --- | --- |
| Candidate / configuration (canonical params) | `9a452d…` / `87ff7e…` | **unchanged** (`waste_sink_inner_radius` not in canonical param bytes) |
| `organism_frozen_hash` | `ed995ddd…` | unchanged |
| `environment_configuration_hash` | `f49ddb2b…` | `ef1834ed…` |
| `D015_ENVIRONMENT_SCHEMA_VERSION` | 2 | 2 |

## Code

- `SimParams::waste_sink_inner_radius`
- `reservoir::apply_reservoir` / `waste_sink_cell`
- `apply_d015_repaired_environment`, `d015_repaired_waste_sink_inner_radius`
