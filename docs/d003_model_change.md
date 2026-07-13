# D-003 Model Change

## Old reaction (D-002)

```text
r_structure = k_structure × C × N × F × max(0, 1 − φ)
```

## New reaction (D-003)

```text
g_structure(φ) = K_phi / (K_phi + max(φ, 0))

r_structure = k_structure × C × N × F × g_structure(φ)
```

Initial `K_phi = 1.0`. At φ = 0, g = 1; at φ = 1, g = 0.5; at φ = 2, g = 0.333…

## Rationale

Crowding attenuation permits non-zero dense-phase synthesis while still reducing production at high φ. This addresses the kinetic asymmetry where decay ∝ φ in the bulk but legacy production was interface-localized.

## Catalyst kinetics (unchanged in first experiment)

```text
r_rep = k_rep × C × N × F × h(φ) × max(0, 1 − C/C_max)
```

Control A restores legacy `max(0,1−φ)` via `use_legacy_structure_kinetics = true`.

## Numerical implications

- **Equation ID:** `d003-crowding-v1` (`reactions::EQUATION_VERSION`)
- **Field bounds:** soft diagnostic −1×10⁻⁶ ≤ φ ≤ 1.25; hard reject φ < −1×10⁻⁴ or φ > 1.50 via `validate_structure_field` after dt adaptation (not in pre-clamp loop)
- **No φ clamp to [0,1]** during integration; small negative clamp to 0 only for |φ| < 1×10⁻⁶
- **Parameters added:** `k_phi`, `use_legacy_structure_kinetics`

## Known limitations

- Analytical rate estimates use initial-field prefactor × D-002 simulated time (snapshot replay approximation); full field-history replay not yet implemented
- Calibration uses single seed (2) 20k windows; full multi-seed confirmation pending
- Long-horizon (250k×5) acceptance and controls A–E not completed in this session
