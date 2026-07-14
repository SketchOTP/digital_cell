# D-006 Planar Calibration

**Directive:** D-006 §14  
**Equation version:** `surface_turnover_v1`

## Setup

Translationally invariant planar tanh interface with:

| Quantity | Value |
| --- | --- |
| φ_in | 1.0 |
| N, F | 1.0 |
| C | 0.35 |
| k_structure_decay | 0.025 |
| seed_interface_width | 3.0 |
| R_reference | 24 (scale choice only) |
| Integration dn | 0.125 over n∈[-40,40] |

## Measured

```text
B_interface = ∫ N F act(C) I(φ) dn ≈ 3.1111111111111067
```

## Derived rate

```text
k_structure_interface_initial
  = (k_structure_decay × φ_in × R_reference) / (2 × B_interface)
  ≈ 0.09642857142857159
```

Artifact: `experiments/generated/d006/planar_interface/calibration.json`

## Candidate set

| Factor | k_structure_interface |
| --- | ---: |
| 0.60× | 0.057857 |
| 0.80× | 0.077143 |
| 1.00× | 0.096429 |
| 1.20× | 0.115714 |
| 1.40× | 0.135000 |

All other parameters frozen from machine-extracted K_phi=1.0 non-structural baseline.
