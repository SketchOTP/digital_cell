# D-006 Scientific Basis

**Directive:** D-006  
**Agent memory:** D-20260713-d006-surface-turnover-protocell

## Motivation

D-003/D-005 crowding kinetics produce structural assembly throughout the dense phase.
Integrated production and decay both scale primarily with protocell **area**.

Matching two extensive rates at one configuration does not produce restoring organism-level
dynamics: smaller and larger states can both sit near production≈decay without an `R*`.

## Surface-production / bulk-turnover hypothesis

| Process | Localization | Area scaling (2D) |
| --- | --- | --- |
| Structural assembly | Phase interface `I(φ)` | ∼ perimeter ∼ `R` |
| Structural decay | Dense bulk | ∼ area ∼ `R²` |

Reduced-order prediction (evaluation only — never used as a controller):

```text
dR/dt ∝ J_build − 0.5 × d_bulk × R
R* ≈ 2 × J_build / d_bulk
```

## Equation version

```text
surface_turnover_v1
```

Historical versions retained: `d001-bulk-v1`, `d003-crowding-v1`.

Fresh seeds only for D-006 accessibility tests. Old crowding snapshots are rejected under
`surface_turnover_v1`.

## Field set

Unchanged: `φ, C, N, F, W`.

## Interface weight

```text
φ̂ = clamp(φ, 0, 1)   # diagnostic only
I(φ) = 16 × φ̂² × (1 − φ̂)²
```

## Assembly / decay

```text
r_assembly = k_structure_interface × N × F × (C / (K_C_structure + C)) × I(φ)
r_decay    = k_structure_decay × max(φ, 0)
```

Initial: `K_C_structure = 0.10`, `k_structure_decay = 0.025`.
Catalyst reproduction unchanged from machine-extracted `K_phi=1.0` baseline.
