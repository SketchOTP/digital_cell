# D-017 Candidate B — perfect-interface bound

## Assumption

`W_interface = 0` (infinitely fast membrane export).

## Result

With interior source 33.551 and `D_W=0.25`, R=22:

```text
center_W = q R² / (4 D_W) = 12.687
```

| Gate | Pass? |
| --- | --- |
| center W < CONC_SAFETY_LIMIT (10) | **NO** |
| center W < 9 | **NO** |
| center W < 5 | **NO** |

A finite pump cannot outperform this bound → membrane-localized active export **cannot** fix the interior diffusion limit.

Artifact: `digital-protocell/experiments/generated/d017/perfect_interface/bound.json`
