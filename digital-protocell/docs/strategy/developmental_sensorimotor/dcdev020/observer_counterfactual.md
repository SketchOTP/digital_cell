# DC-DEV-020 observer counterfactual

This bounded slice tests one fast, target-free A-product feedback law against
the accepted finite nutrient ecology. It starts from clean DC-DEV-016 entry
`1e242f28152797b512e25cd56c7b718e45d6ca97` and does not modify production
chemistry or the existing resource boundary.

## Frozen protocol

- settlement: 5,000 accepted mechanics steps;
- deprivation: 480 steps with no N/F;
- finite feeding: 480 steps;
- selected patch: N=F=`19.878372106390554`, center `[4.8, 0.0]`, radius `1.5`;
- long-horizon observer continuation: 8,000 no-resource steps;
- no parameter search and no production integration before observer success.

The new law is the single fixed counterfactual:

```text
k_act_effective = k_act * (1 + K_I / (K_I + A))
K_I = 1.0
```

It reads only current local A, has no stored state, does not read a target,
need, error, resource, or homeostat, and preserves the existing N/F→A/W
stoichiometric event. The unit scale follows the existing unit material
coordinates; it is not screened or tuned.

The historical D-017 comparison is replayed as an observer-only reference.
Its accepted evidence remains comparison-only and rejects runtime activation-
yield deployment because the alpha>0 law creates potential under frozen
weights and remains transport-ceiling limited. It is not silently imported as
production chemistry.

## Result

The selected ecology remains physically delivered and exactly conserved:

- feedback N/F delivered: `15.566438806699026` each;
- feedback N/F consumed: `2.741890690082671` each;
- maximum resource conservation error: `0`.

The feedback arm improves stored material relative to the matched baseline,
but does not restore the deprived state:

| arm | final E_stored |
| --- | ---: |
| pre-feed deprived | `60.82781514212436` |
| baseline existing N/F→A | `54.3584702923158` |
| A-product feedback | `55.30767262894278` |

The 8,000-step no-resource continuation also lost viability. The observer
gate therefore fails. Production integration, long-horizon qualification,
repeated persistence, exploration, encounter, and repeatability are not
authorized by this result.

Compact evidence is stored in
`digital-protocell/experiments/generated/dcdev020/observer/` and contains no
dense step ledger.
