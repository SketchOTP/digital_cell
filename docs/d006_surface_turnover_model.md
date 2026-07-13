# D-006 Surface-Turnover Model

**Equation version:** `surface_turnover_v1`

## Reaction terms

Catalyst reproduction (unchanged):

```text
r_rep = k_rep × C × N × F × h(φ) × max(0, 1 − C/C_max)
```

Interface structural assembly:

```text
r_structure = k_structure_interface × N × F × catalyst_activation(C) × I(φ)
catalyst_activation(C) = C / (K_C_structure + C)
I(φ) = 16 × φ̂² × (1 − φ̂)² ,  φ̂ = clamp(φ,0,1)
```

Bulk structural turnover:

```text
r_structure_decay = k_structure_decay × max(φ, 0)
Rφ = r_structure − r_structure_decay
```

## Required causal absences

`C=0`, `N=0`, `F=0`, or `I(φ)=0` ⇒ no structural assembly.

## Stoichiometric ledger

Tracked: interface assembly, bulk decay, catalyst reproduction/decay,
nutrient/fuel consumed by assembly and by catalyst reproduction,
waste from assembly, structural decay, and catalyst reactions.

Spatial partitions: dense interior / interface / dilute exterior
(via interface weight and interior weight).

Localization gate: ≥90% of structural assembly where `I(φ) ≥ 0.25`.

## Forbidden controllers

No runtime `target_radius`, `desired_mass`, radius/mass feedback, or observer→kinetics coupling.
