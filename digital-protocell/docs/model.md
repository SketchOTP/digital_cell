# Model Equations

## Fields

| Symbol | Code | Role |
|--------|------|------|
| φ | `structure` | Phase-forming structural material |
| C | `catalyst` | Autocatalytic metabolic catalyst |
| N | `nutrient` | External matter |
| F | `fuel` | High-energy driver |
| W | `waste` | Low-value product |

## Reactions

- **R1:** C + N + F → 2C + W (`r_rep`)
- **R2:** C + N + F → C + φ + W (`r_structure`)
- **R3:** φ → W (`r_structure_decay`)
- **R4:** C → W (`r_catalyst_decay`, faster outside dense phase)
- **R5:** W → reservoir (`k_waste_decay`)

## Phase field

- f(φ) = A φ² (1−φ)²
- μ = 2Aφ(1−φ)(1−2φ) − κ∇²φ
- ∂φ/∂t = M∇²μ + Rφ
- h(φ) = φ²(3−2φ) — interior weighting (not a cell mask)

## Diffusion

- C: variable D_C(φ) = D_out + h(φ)(D_in − D_out), face flux form
- N, F, W: constant D, Laplacian with no-flux dish wall for φ and C

## Boundary

- Circular dish r ≤ 88; reservoir annulus relaxes N, F, W toward reservoir targets.
