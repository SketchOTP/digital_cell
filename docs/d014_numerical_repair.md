# D-014 Numerical Repair

## Classification

```text
FIELD_BOUND_STIFFNESS
```

## Branch E repair

1. **Machine-scale ceiling projection** (`D014_CONC_CEILING_PROJECT_EPS = 1e-9`):
   concentrations in `(CONC_SAFETY_LIMIT, CONC_SAFETY_LIMIT + eps]` are projected to the
   limit before validation; mass change enters numerical-correction accounting via
   pre/post clamp masses.

2. **Hard bound abort**: `excessive concentration` rejects no longer cascade-halve `dt`
   to the floor. `step()` returns false immediately.

3. **Termination mapping**: such failures map to `UNBOUNDED_ACCUMULATION` instead of
   `TIMESTEP_FLOOR_FAILURE` / `NUMERICAL_FAILURE`.

4. **Biological check**: `soluble_max >= CONC_SAFETY_LIMIT` → `UNBOUNDED_ACCUMULATION`.

## Branch A hygiene (subsidiary)

Bounded `dt` recovery `×1.25` per accepted step toward `dt_cap` (default `MAX_DT`).

## Not changed

Chemistry, stoichiometry, rates, yields, transport coefficients, thresholds, initial
fields, frozen candidate/configuration hashes.

## Versions

| Item | Value |
| --- | --- |
| numerical_method_version | 2 |
| adaptive_controller_version | 2 |
