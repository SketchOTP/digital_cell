# D-019 Selected Mechanism

## Equation version

`membrane_metabolism_v3_structural_scaling`

## Parent

`membrane_metabolism_v2_conservative` (candidate hash `9a452d34…`)

## Structural schema

version `1`

## Stoichiometric / transport schemas

unchanged: stoich `2`, transport `1`, D-015 environment retained

## Mechanism

**Interface-limited structure turnover**

- Production (unchanged from v2): `r_structure = k_d008_structure · A · I(φ)`
- Decay (changed): `r_decay = k_structure_decay · φ · (STRUCTURAL_EXPOSURE_FLOOR + I(φ))`
- `STRUCTURAL_EXPOSURE_FLOOR = 0.05` (frozen; enables full interior turnover)

## Forbidden encodings absent

No target radius, target mass, global curvature average, observer feedback, or new field.
