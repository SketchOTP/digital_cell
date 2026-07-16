# D-017 Candidate B — energy-coupled export cost

Comparison stoichiometries only (not implemented).

## B1 — A-coupled

```text
A_in + W_in → 2 W_out
```

- Net interior W removal / event: 1
- Environmental W / event: 2 (50% pump-generated)
- A cost / exported W: 1
- Classification under required flux vs A production: **B1_WORSENS_PRODUCTIVE_DEFICIT**

## B2 — F-coupled

```text
F_in + W_in → 2 W_out
```

- Same material accounting
- Classification: **B2_EXCESSIVE_FUEL_DEMAND**

## Internal delivery

Max delivery with W_interface=0 and W_center<10: **26.45** < interior production **33.55**
→ **B_INTERNAL_DELIVERY_INSUFFICIENT**

## Components

Defensible directional pump + saturation is not representable in the existing seven fields without hidden machinery → **B_REQUIRES_NEW_BIOLOGICAL_COMPONENT**

Artifacts: `active_export_A/`, `active_export_F/`, `internal_delivery/`
