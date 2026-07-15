# D-012 Conservation Proof (v2 gate)

## Exact stoichiometric proof

The governed 7×9 internal-reaction matrix for `membrane_metabolism_v2_conservative` with unit yields
admits the strictly positive left null vector **m = (1,1,1,1,1,1,1)**.

```
mᵀ S = 0   (exact rationals)
```

Classification: `STRICTLY_CONSERVATIVE`  
Primary finding: `D012_CONSERVATIVE_V2_CONFIRMED`

Artifact: `digital-protocell/experiments/generated/d012/v2_stoichiometric_matrix/audit.json`

## Runtime equivalence

Each isolated runtime delta (activation, catalyst, structure, membrane, turnovers) matches the
shared `stoichiometry::v2_internal_reactions` descriptor column at unit extent.
Verified in `d012_tests` (`test_runtime_*_delta_matches_matrix` family).

## Material-equivalent ledger

Observer identity:

```text
observed total change = reservoir input − waste clearance + numerical correction
```

- Internal reaction extents contribute **zero** net material under v2.
- Membrane detachment is `M → W` (internal conversion), not deletion.
- Controlled-test tolerance: relative residual ≤ `1e-6`.

## Activation-potential ledger

Initial governed weights (exact):

| Symbol | Weight | Role |
| --- | --- | --- |
| `e_F` | 1 | Fuel carries usable chemical potential |
| `e_A` | 1 | Activated resource carries transferred potential |
| Other components | 0 | Unless a later gate requires otherwise |

```text
E_chemical = e_F · F + e_A · A
```

Controls (closed system):

- No internal reaction increases total activation potential.
- Fuel reservoir import is the only external potential source.
- No v2 reaction consumes `W` to produce `F` or `A`.

## Conservation gate checklist

| Predicate | Test | Result |
| --- | --- | --- |
| Each internal reaction conservative | `test_v2_each_internal_reaction_is_conservative` | PASS |
| Strictly positive conservation vector | `test_v2_positive_conservation_vector` | PASS |
| Runtime deltas match matrix | `test_runtime_*_delta_matches_matrix` | PASS |
| Material accounting closes | `test_v2_total_change_equals_boundary_exchange` | PASS |
| Waste clearance explicit | `test_v2_waste_clearance_is_explicit_output` | PASS |
| Closed material | `test_closed_v2_network_does_not_create_material` | PASS |
| Closed activation potential | `test_closed_v2_network_does_not_create_activation_potential` | PASS |
| Fuel-only external potential | `test_fuel_is_only_external_activation_potential_source` | PASS |
| Waste cannot reactivate | `test_waste_cannot_reactivate_spontaneously` | PASS |

**Gate verdict: PASSED** — governed v2 Stage B–E experiments may proceed from Task 11.
